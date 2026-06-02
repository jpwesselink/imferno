//! ST 2067-2 §5.4 timed-text essence rules, evaluated against the
//! RegXML produced by `mxf::metadata::parse_mxf_to_regxml`.
//!
//! Four rules:
//!
//! - **§5.4 / ST 429-5** — Mapping Kind byte in the essence container
//!   UL must be `0x13` (IMSC mapping).
//! - **§5.4** — `UCSEncoding` must be UTF-8.
//! - **§5.4** — Timed-text `NamespaceURI` must be one of the IMSC1
//!   profiles.
//! - **§5.4.5/6** — `TimeTextResourceSubDescriptor.MIMEType` must be
//!   `image/png` or `application/x-font-opentype`.
//!
//! Unit-tested against synthetic snippets only; no IMF timed-text MXF
//! is vendored. End-to-end verification lands when one is added.
//!
//! Native-only (same constraint as other `mxf::*` essence modules).

use std::path::Path;

use crate::diagnostics::{Category, Location, Severity, ValidationIssue};

/// Walk a RegXML document for the TimedTextDescriptor and apply the
/// timed-text rules. Returns an empty Vec when no timed-text descriptor
/// is present (the file is audio/video) — these checks only fire on
/// timed-text essence.
pub fn check_timed_text(regxml: &str, path: &Path) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Sniff for timed-text essence shape. ST 2067-2 timed text uses
    // `TimedTextDescriptor` (the AAF name). Some emitters use
    // `IMFTimedTextDescriptor` for the IMF-specific subclass.
    let has_timed_text = regxml.contains(":TimedTextDescriptor")
        || regxml.contains(":IMFTimedTextDescriptor");
    if !has_timed_text {
        return issues;
    }

    // §5.4 / ST 429-5 §7 — timed-text essence container UL byte 15
    // (Mapping Kind, 1-indexed) must be 0x13 for IMSC text. This is
    // the same UL position the audio module checks for the wrapping
    // octet — different value per essence type.
    if let Some(cf) = extract_field(regxml, "ContainerFormat") {
        if let Some(bytes) = crate::mxf::audio_mca::parse_ul_bytes(&cf) {
            if bytes[14] != 0x13 {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Container,
                        "ST2067-2:2016:5.4/TimedTextMappingKindNot0x13",
                        format!(
                            "MXF {} timed-text ContainerFormat UL byte 15 = 0x{:02x} \
                             — ST 429-5 §7 requires Mapping Kind = 0x13 for IMSC. \
                             ContainerFormat = {}",
                            path.display(),
                            bytes[14],
                            cf.trim(),
                        ),
                    )
                    .with_location(Location::new().with_file(path.to_path_buf())),
                );
            }
        }
    }

    // §5.4 — UCSEncoding must be UTF-8.
    if let Some(enc) = extract_field(regxml, "UCSEncoding") {
        let enc = enc.trim();
        if !enc.eq_ignore_ascii_case("UTF-8") && !enc.eq_ignore_ascii_case("UTF8") {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Subtitle,
                    "ST2067-2:2016:5.4/TimedTextUCSEncodingNotUTF8",
                    format!(
                        "MXF {} TimedTextDescriptor UCSEncoding = '{}' — ST 2067-2 §5.4 requires UTF-8.",
                        path.display(),
                        enc,
                    ),
                )
                .with_location(Location::new().with_file(path.to_path_buf())),
            );
        }
    }

    // §5.4 — TimedText NamespaceURI must be one of the IMSC1 profiles.
    // The exact set: IMSC1 text profile + IMSC1 image profile, plus
    // their TTML-superset namespaces.
    if let Some(ns) = extract_field(regxml, "NamespaceURI") {
        let ns = ns.trim();
        const ACCEPTABLE: &[&str] = &[
            "http://www.w3.org/ns/ttml/profile/imsc1/text",
            "http://www.w3.org/ns/ttml/profile/imsc1/image",
            "http://www.w3.org/ns/ttml/profile/imsc1.1/text",
            "http://www.w3.org/ns/ttml/profile/imsc1.1/image",
        ];
        if !ACCEPTABLE.iter().any(|a| ns == *a) {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Subtitle,
                    "ST2067-2:2016:5.4/TimedTextNamespaceNotIMSC",
                    format!(
                        "MXF {} TimedTextDescriptor NamespaceURI = '{}' — ST 2067-2 §5.4 \
                         requires one of the IMSC1 profile namespaces (text or image, 1.0 or 1.1).",
                        path.display(),
                        ns,
                    ),
                )
                .with_location(Location::new().with_file(path.to_path_buf())),
            );
        }
    }

    // §5.4.5/6 — TimeTextResourceSubDescriptor MIME types must be
    // image/png (sub-image resources) or application/x-font-opentype
    // (font resources). Anything else breaks IMSC playback.
    for mime in extract_all_fields(regxml, "MIMEType") {
        let mime = mime.trim();
        const ACCEPTABLE: &[&str] = &["image/png", "application/x-font-opentype"];
        if !ACCEPTABLE.iter().any(|a| mime == *a) {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Subtitle,
                    "ST2067-2:2016:5.4.5/TimedTextResourceMIMETypeUnsupported",
                    format!(
                        "MXF {} TimeTextResourceSubDescriptor MIMEType = '{}' — ST 2067-2 \
                         §5.4.5/6 requires image/png or application/x-font-opentype.",
                        path.display(),
                        mime,
                    ),
                )
                .with_location(Location::new().with_file(path.to_path_buf())),
            );
        }
    }

    issues
}

// Reuse the audio_mca module's XML helpers — they live there because
// they were written first; export-and-import is cleaner than duplicating.
use crate::mxf::audio_mca::{extract_all_fields, extract_field};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_non_timed_text_mxf() {
        let xml = r#"<ns1:Preface><ns1:WAVEPCMDescriptor/></ns1:Preface>"#;
        let issues = check_timed_text(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues.is_empty(),
            "timed text pipeline must be silent on non-timed-text MXF, got: {:#?}",
            issues
        );
    }

    #[test]
    fn flags_timed_text_mapping_kind_not_0x13() {
        // ContainerFormat byte 14 (zero-indexed) = 0x12, not 0x13.
        let xml = r#"<ns1:TimedTextDescriptor>
            <ns2:ContainerFormat>urn:smpte:ul:060e2b34.04010101.0d010301.02061200</ns2:ContainerFormat>
            <ns2:UCSEncoding>UTF-8</ns2:UCSEncoding>
            <ns2:NamespaceURI>http://www.w3.org/ns/ttml/profile/imsc1/text</ns2:NamespaceURI>
        </ns1:TimedTextDescriptor>"#;
        let issues = check_timed_text(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("TimedTextMappingKindNot0x13")),
            "expected TimedTextMappingKindNot0x13, got: {:#?}",
            issues
        );
    }

    #[test]
    fn flags_non_utf8_encoding() {
        let xml = r#"<ns1:TimedTextDescriptor>
            <ns2:UCSEncoding>ISO-8859-1</ns2:UCSEncoding>
            <ns2:NamespaceURI>http://www.w3.org/ns/ttml/profile/imsc1/text</ns2:NamespaceURI>
        </ns1:TimedTextDescriptor>"#;
        let issues = check_timed_text(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("TimedTextUCSEncodingNotUTF8")),
            "expected TimedTextUCSEncodingNotUTF8, got: {:#?}",
            issues
        );
    }

    #[test]
    fn flags_non_imsc_namespace() {
        let xml = r#"<ns1:TimedTextDescriptor>
            <ns2:UCSEncoding>UTF-8</ns2:UCSEncoding>
            <ns2:NamespaceURI>http://example.org/not-imsc</ns2:NamespaceURI>
        </ns1:TimedTextDescriptor>"#;
        let issues = check_timed_text(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("TimedTextNamespaceNotIMSC")),
            "expected TimedTextNamespaceNotIMSC, got: {:#?}",
            issues
        );
    }

    #[test]
    fn flags_unsupported_resource_mime_type() {
        let xml = r#"<ns1:TimedTextDescriptor>
            <ns2:UCSEncoding>UTF-8</ns2:UCSEncoding>
            <ns2:NamespaceURI>http://www.w3.org/ns/ttml/profile/imsc1/text</ns2:NamespaceURI>
            <ns1:TimeTextResourceSubDescriptor>
                <ns2:MIMEType>application/json</ns2:MIMEType>
            </ns1:TimeTextResourceSubDescriptor>
        </ns1:TimedTextDescriptor>"#;
        let issues = check_timed_text(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("TimedTextResourceMIMETypeUnsupported")),
            "expected TimedTextResourceMIMETypeUnsupported, got: {:#?}",
            issues
        );
    }

    #[test]
    fn accepts_clean_imsc_timed_text_descriptor() {
        let xml = r#"<ns1:TimedTextDescriptor>
            <ns2:UCSEncoding>UTF-8</ns2:UCSEncoding>
            <ns2:NamespaceURI>http://www.w3.org/ns/ttml/profile/imsc1.1/text</ns2:NamespaceURI>
            <ns1:TimeTextResourceSubDescriptor>
                <ns2:MIMEType>image/png</ns2:MIMEType>
            </ns1:TimeTextResourceSubDescriptor>
            <ns1:TimeTextResourceSubDescriptor>
                <ns2:MIMEType>application/x-font-opentype</ns2:MIMEType>
            </ns1:TimeTextResourceSubDescriptor>
        </ns1:TimedTextDescriptor>"#;
        let issues = check_timed_text(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues.is_empty(),
            "clean IMSC1.1 + acceptable MIME types should produce zero diagnostics, got: {:#?}",
            issues
        );
    }
}
