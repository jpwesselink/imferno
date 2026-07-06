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

use crate::diagnostics::{Location, ValidationIssue};
use crate::mxf::codes::St2067_2_2016;

/// Walk a RegXML document for the TimedTextDescriptor and apply the
/// timed-text rules. Returns an empty Vec when no timed-text descriptor
/// is present (the file is audio/video) — these checks only fire on
/// timed-text essence.
pub fn check_timed_text(regxml: &str, path: &Path) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Sniff for timed-text essence shape. ST 2067-2 timed text uses
    // `TimedTextDescriptor` (the AAF name). Some emitters use
    // `IMFTimedTextDescriptor` for the IMF-specific subclass.
    let has_timed_text =
        regxml.contains(":TimedTextDescriptor") || regxml.contains(":IMFTimedTextDescriptor");
    if !has_timed_text {
        return issues;
    }

    // §5.4 / ST 429-5 §7 — timed-text essence container UL byte 14
    // (Mapping Kind, 1-indexed; index 13) must be 0x13 for IMSC text.
    // The canonical ST 429-5 container UL is
    // 060e2b34.0401010a.0d010301.02130101: 0x13 sits at index 13 and
    // index 14 is 0x01 (AUDIT-7 — checking index 14 false-positived on
    // every conformant timed-text file).
    if let Some(cf) = extract_field(regxml, "ContainerFormat") {
        if let Some(bytes) = crate::mxf::audio_mca::parse_ul_bytes(&cf) {
            if bytes[13] != 0x13 {
                issues.push(
                    ValidationIssue::from_code(
                        St2067_2_2016::TimedTextMappingKindNot0x13,
                        format!(
                            "MXF {} timed-text ContainerFormat UL byte 14 = 0x{:02x} \
                             — ST 429-5 §7 requires Mapping Kind = 0x13 for IMSC. \
                             ContainerFormat = {}",
                            path.display(),
                            bytes[13],
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
                ValidationIssue::from_code(St2067_2_2016::TimedTextUCSEncodingNotUTF8,
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
        if !ACCEPTABLE.contains(&ns) {
            issues.push(
                ValidationIssue::from_code(
                    St2067_2_2016::TimedTextNamespaceNotIMSC,
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

    // §5.4.5/6 — TimeTextResourceSubDescriptor MIME types: image/png
    // (image resources) or an OpenType font MIME (font resources).
    //
    // Edition note (AUDIT-13): 2016 allows only
    // application/x-font-opentype; 2020 §5.4.6 allows "font/otf, as
    // specified in RFC 8081, or application/x-font-opentype" (and says
    // x-font-opentype "should not be used" except for backwards
    // compatibility). MXF-level checks can't see the CPL edition, so the
    // whitelist is the union of the editions — rejecting font/otf would
    // false-Error on every valid 2020-edition file. The full §5.4:2020
    // edition model (IMSC 1.1, §5.4.1/§5.4.7) is tracked in AUDIT-13.
    for mime in extract_all_fields(regxml, "MIMEType") {
        let mime = mime.trim();
        const ACCEPTABLE: &[&str] = &["image/png", "application/x-font-opentype", "font/otf"];
        if !ACCEPTABLE.contains(&mime) {
            issues.push(
                ValidationIssue::from_code(
                    St2067_2_2016::TimedTextResourceMIMETypeUnsupported,
                    format!(
                        "MXF {} TimeTextResourceSubDescriptor MIMEType = '{}' — ST 2067-2 \
                         §5.4.5/6 requires image/png, font/otf (2020) or \
                         application/x-font-opentype.",
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
        // Mapping Kind (index 13, zero-indexed) = 0x12, not 0x13; index 14
        // is 0x01 as in the canonical UL, so only the correct byte trips it.
        let xml = r#"<ns1:TimedTextDescriptor>
            <ns2:ContainerFormat>urn:smpte:ul:060e2b34.0401010a.0d010301.02120101</ns2:ContainerFormat>
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

    /// AUDIT-7 regression: the canonical ST 429-5 container UL
    /// (060e2b34.0401010a.0d010301.02130101 — Mapping Kind 0x13 at
    /// index 13, 0x01 at index 14) must NOT trip the check. The old
    /// code compared `bytes[14]` and false-positived on every
    /// conformant IMF timed-text file.
    #[test]
    fn accepts_canonical_st429_5_container_ul() {
        let xml = r#"<ns1:TimedTextDescriptor>
            <ns2:ContainerFormat>urn:smpte:ul:060e2b34.0401010a.0d010301.02130101</ns2:ContainerFormat>
            <ns2:UCSEncoding>UTF-8</ns2:UCSEncoding>
            <ns2:NamespaceURI>http://www.w3.org/ns/ttml/profile/imsc1/text</ns2:NamespaceURI>
        </ns1:TimedTextDescriptor>"#;
        let issues = check_timed_text(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("TimedTextMappingKindNot0x13")),
            "canonical ST 429-5 UL must not trip Mapping Kind check (AUDIT-7), got: {:#?}",
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

    /// AUDIT-13 regression: ST 2067-2:2020 §5.4.6 allows "font/otf, as
    /// specified in RFC 8081, or application/x-font-opentype" for font
    /// resources — font/otf must not draw a MIME-type Error.
    #[test]
    fn accepts_font_otf_mime_type_2020() {
        let xml = r#"<ns1:TimedTextDescriptor>
            <ns2:UCSEncoding>UTF-8</ns2:UCSEncoding>
            <ns2:NamespaceURI>http://www.w3.org/ns/ttml/profile/imsc1.1/text</ns2:NamespaceURI>
            <ns1:TimeTextResourceSubDescriptor>
                <ns2:MIMEType>font/otf</ns2:MIMEType>
            </ns1:TimeTextResourceSubDescriptor>
        </ns1:TimedTextDescriptor>"#;
        let issues = check_timed_text(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("TimedTextResourceMIMETypeUnsupported")),
            "font/otf is valid per ST 2067-2:2020 §5.4.6 (AUDIT-13), got: {issues:#?}"
        );
    }
}
