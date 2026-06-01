//! Photon-parity ST 2067-2 §5.3 audio essence rules, evaluated against
//! the RegXML produced by `mxf::metadata::parse_mxf_to_regxml`.
//!
//! These are the checks Photon enforces over the MXF audio descriptor +
//! its MCA (Multi-Channel Audio) sub-descriptors per ST 2067-2 §5.3:
//!
//! - **§5.3.4.1** — sound essence MUST use `WAVEPCMDescriptor`.
//! - **§5.3.2.2** — `AudioSampleRate` ∈ {48 000, 96 000} Hz.
//! - **§5.3.2.3** — `QuantizationBits` = 24.
//! - **§5.3.6.2** — number of `AudioChannelLabelSubDescriptor`s equals
//!   the descriptor's `ChannelCount`.
//! - **§5.3.6.3** — exactly one `SoundfieldGroupLabelSubDescriptor`.
//!
//! The RegXML format is regular enough (machine-emitted by
//! `regxmllib-rs`) that targeted substring scans are sufficient and
//! avoid pulling another XML parser into the dependency graph. If the
//! emitter format changes meaningfully we'll switch to event-based
//! parsing via `quick_xml`.
//!
//! Native-only — same constraint as the rest of `mxf::essence` /
//! `mxf::metadata` (browser callers don't see MXF binaries).

use std::path::Path;

use crate::diagnostics::{Category, Location, Severity, ValidationIssue};

/// Walk a RegXML document for the WAVEPCMDescriptor and apply the
/// audio MCA rules. Returns a list of `ValidationIssue`s the caller
/// can fold into the unified `ValidationReport`.
///
/// `regxml` is the output of `mxf::metadata::parse_mxf_to_regxml`;
/// `path` is the source MXF file path used for `Location` attribution
/// and human-readable messages. When no audio descriptor is present
/// in the RegXML (the file is video or timed-text), this returns an
/// empty Vec — these checks only fire on sound essence.
pub fn check_audio_mca(regxml: &str, path: &Path) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // §5.3.4.1 — sound essence MUST use WAVEPCMDescriptor.
    // If we see *any* audio-shaped surface (ChannelCount + audio
    // sample rate) but no WAVEPCMDescriptor element, that's a
    // structural violation. We detect "audio-shaped" by the presence
    // of an AudioSampleRate element which only appears on sound
    // descriptors per ST 377-2.
    let has_audio_sample_rate = regxml.contains("AudioSampleRate");
    let has_wave_pcm = regxml.contains("WAVEPCMDescriptor");
    if has_audio_sample_rate && !has_wave_pcm {
        issues.push(
            ValidationIssue::new(
                Severity::Error,
                Category::Audio,
                "ST2067-2:2016:5.3.4.1/SoundDescriptorNotWAVEPCM",
                format!(
                    "MXF {} carries audio essence but its descriptor is not a WAVEPCMDescriptor — ST 2067-2 §5.3.4.1 requires WAVE PCM",
                    path.display()
                ),
            )
            .with_location(Location::new().with_file(path.to_path_buf())),
        );
    }

    // Only run the rest of the checks if there's a sound descriptor.
    if !has_wave_pcm {
        return issues;
    }

    // §5.3.2.2 — AudioSampleRate ∈ {48 000, 96 000}. The RegXML
    // emits sample rates as `num/den` rationals.
    if let Some(rate) = extract_field(regxml, "AudioSampleRate") {
        if !is_acceptable_audio_rate(&rate) {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Audio,
                    "ST2067-2:2016:5.3.2.2/AudioSampleRateUnsupported",
                    format!(
                        "MXF {} declares AudioSampleRate = {} — ST 2067-2 §5.3.2.2 requires 48000 Hz or 96000 Hz",
                        path.display(),
                        rate
                    ),
                )
                .with_location(Location::new().with_file(path.to_path_buf())),
            );
        }
    }

    // §5.3.2.3 — QuantizationBits = 24.
    if let Some(qb) = extract_field(regxml, "QuantizationBits") {
        if qb.trim() != "24" {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Audio,
                    "ST2067-2:2016:5.3.2.3/QuantizationBitsNot24",
                    format!(
                        "MXF {} declares QuantizationBits = {} — ST 2067-2 §5.3.2.3 requires 24-bit audio",
                        path.display(),
                        qb
                    ),
                )
                .with_location(Location::new().with_file(path.to_path_buf())),
            );
        }
    }

    // §5.3.6.2 — number of AudioChannelLabelSubDescriptors must equal
    // ChannelCount on the WAVEPCMDescriptor.
    let channel_count = extract_field(regxml, "ChannelCount").and_then(|c| c.trim().parse::<u32>().ok());
    let channel_labels = count_elements(regxml, "AudioChannelLabelSubDescriptor");
    if let Some(cc) = channel_count {
        if (channel_labels as u32) != cc {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Audio,
                    "ST2067-2:2016:5.3.6.2/ChannelLabelCountMismatch",
                    format!(
                        "MXF {} declares ChannelCount = {} but carries {} AudioChannelLabelSubDescriptor(s) — \
                         ST 2067-2 §5.3.6.2 requires one label per channel",
                        path.display(),
                        cc,
                        channel_labels,
                    ),
                )
                .with_location(Location::new().with_file(path.to_path_buf())),
            );
        }
    }

    // §5.3.6.3 — exactly one SoundfieldGroupLabelSubDescriptor.
    let soundfield_count = count_elements(regxml, "SoundfieldGroupLabelSubDescriptor");
    if soundfield_count != 1 {
        issues.push(
            ValidationIssue::new(
                Severity::Error,
                Category::Audio,
                "ST2067-2:2016:5.3.6.3/SoundFieldGroupLabelCount",
                format!(
                    "MXF {} carries {} SoundfieldGroupLabelSubDescriptor(s) — ST 2067-2 §5.3.6.3 requires exactly one",
                    path.display(),
                    soundfield_count,
                ),
            )
            .with_location(Location::new().with_file(path.to_path_buf())),
        );
    }

    issues
}

/// Extract the text content of the first occurrence of a SMPTE field
/// element in the RegXML stream. Matches by local name (suffix after
/// the namespace prefix) so we don't depend on which prefix the
/// `regxml` emitter happens to pick.
///
/// Returns `None` if the field isn't found. Returns the trimmed text
/// (whitespace stripped) on success.
fn extract_field(xml: &str, local_name: &str) -> Option<String> {
    // Match `<…:Local …>BODY</…:Local>` — strip the namespace
    // prefix by scanning back from `:Local` to `<`. This handles
    // both `<ns2:ChannelCount>…</ns2:ChannelCount>` and any other
    // prefix the writer chose.
    let open_token = format!(":{local_name}");
    let mut search_from = 0;
    while let Some(rel) = xml[search_from..].find(&open_token) {
        let abs = search_from + rel;
        // Confirm this is a tag-open by walking back to the nearest `<`.
        let prefix_start = xml[..abs].rfind('<')?;
        // Reject if there's a closing-slash or it's actually a closing tag.
        if xml[prefix_start..].starts_with("</") {
            search_from = abs + open_token.len();
            continue;
        }
        // Find the end of the open tag.
        let tag_end = xml[abs..].find('>')?;
        let body_start = abs + tag_end + 1;
        // Find the matching close tag — same local name.
        let close_marker = format!("</");
        let mut body_end_search = body_start;
        while let Some(close_rel) = xml[body_end_search..].find(&close_marker) {
            let close_abs = body_end_search + close_rel;
            let after_slash = close_abs + 2;
            // Find the `:` after the prefix (or empty prefix).
            let close_local_start = xml[after_slash..]
                .find(':')
                .map(|p| after_slash + p + 1)
                .unwrap_or(after_slash);
            let close_local_end = xml[close_local_start..].find('>')?;
            let close_local = &xml[close_local_start..close_local_start + close_local_end];
            if close_local == local_name {
                let body = xml[body_start..close_abs].trim();
                return Some(body.to_string());
            }
            body_end_search = close_abs + close_marker.len();
        }
        return None;
    }
    None
}

/// Count the number of occurrences of an element by local name. Uses
/// the same prefix-agnostic matching as `extract_field`. Counts open
/// tags only (so self-closing forms are also detected if the writer
/// emits them).
fn count_elements(xml: &str, local_name: &str) -> usize {
    let open_token = format!(":{local_name}");
    let mut count = 0;
    let mut search_from = 0;
    while let Some(rel) = xml[search_from..].find(&open_token) {
        let abs = search_from + rel;
        let next_char = xml.as_bytes().get(abs + open_token.len()).copied();
        // Match boundary so `:Channel` doesn't match `:ChannelCount`.
        if matches!(next_char, Some(b'>') | Some(b' ') | Some(b'/') | Some(b'\t') | Some(b'\n')) {
            if let Some(prefix_start) = xml[..abs].rfind('<') {
                if !xml[prefix_start..].starts_with("</") {
                    count += 1;
                }
            }
        }
        search_from = abs + open_token.len();
    }
    count
}

fn is_acceptable_audio_rate(rate_text: &str) -> bool {
    // RegXML emits rationals as `num/den`. Parse defensively — anything
    // we can't parse is flagged elsewhere (XSD layer catches malformed
    // RationalType); here we only judge supported rates.
    let parts: Vec<&str> = rate_text.trim().split('/').collect();
    let (num, den) = match parts.as_slice() {
        [n, d] => (n.trim().parse::<i64>().ok(), d.trim().parse::<i64>().ok()),
        [n] => (n.trim().parse::<i64>().ok(), Some(1)),
        _ => return false,
    };
    let (Some(n), Some(d)) = (num, den) else {
        return false;
    };
    if d == 0 {
        return false;
    }
    let hz = n / d;
    matches!(hz, 48_000 | 96_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mxf::metadata::parse_mxf_to_regxml;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest.join("tests/fixtures/mxf").join(name)
    }

    fn audio1_regxml() -> (String, PathBuf) {
        let path = fixture("audio1.mxf");
        let opts = regxml::MxfFragmentOptions {
            partition: regxml::PartitionTarget::Header,
            ..Default::default()
        };
        let xml = parse_mxf_to_regxml(&path, opts).expect("audio1 → RegXML");
        (xml, path)
    }

    #[test]
    fn audio1_clean_fixture_passes_all_audio_mca_checks() {
        let (xml, path) = audio1_regxml();
        let issues = check_audio_mca(&xml, &path);
        assert!(
            issues.is_empty(),
            "audio1.mxf is a well-formed PCM stereo fixture and should pass all §5.3 checks. \
             Got: {:#?}",
            issues
        );
    }

    #[test]
    fn extract_field_handles_namespaced_tags() {
        let xml = r#"
            <ns2:ChannelCount xmlns:ns2="x">2</ns2:ChannelCount>
            <ns2:QuantizationBits xmlns:ns2="x">24</ns2:QuantizationBits>
        "#;
        assert_eq!(extract_field(xml, "ChannelCount").as_deref(), Some("2"));
        assert_eq!(extract_field(xml, "QuantizationBits").as_deref(), Some("24"));
        assert_eq!(extract_field(xml, "AbsentField"), None);
    }

    #[test]
    fn count_elements_respects_local_name_boundary() {
        let xml = r#"
            <ns1:AudioChannelLabelSubDescriptor/>
            <ns1:AudioChannelLabelSubDescriptor></ns1:AudioChannelLabelSubDescriptor>
            <ns1:ChannelCount>2</ns1:ChannelCount>
        "#;
        // `:ChannelCount` substring overlaps with `:AudioChannelLabelSubDescriptor`'s
        // `:Channel`; boundary check prevents the wrong count.
        assert_eq!(count_elements(xml, "AudioChannelLabelSubDescriptor"), 2);
        assert_eq!(count_elements(xml, "ChannelCount"), 1);
        assert_eq!(count_elements(xml, "Channel"), 0);
    }

    #[test]
    fn flags_quantization_other_than_24() {
        // Synthetic minimal RegXML carrying a 16-bit quant figure.
        // Should fire §5.3.2.3.
        let xml = r#"<ns1:WAVEPCMDescriptor>
            <ns2:AudioSampleRate>48000/1</ns2:AudioSampleRate>
            <ns2:QuantizationBits>16</ns2:QuantizationBits>
            <ns2:ChannelCount>1</ns2:ChannelCount>
            <ns1:SoundfieldGroupLabelSubDescriptor/>
            <ns1:AudioChannelLabelSubDescriptor/>
        </ns1:WAVEPCMDescriptor>"#;
        let issues = check_audio_mca(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("QuantizationBitsNot24")),
            "expected QuantizationBitsNot24, got: {:#?}",
            issues
        );
    }

    #[test]
    fn flags_unsupported_sample_rate() {
        let xml = r#"<ns1:WAVEPCMDescriptor>
            <ns2:AudioSampleRate>44100/1</ns2:AudioSampleRate>
            <ns2:QuantizationBits>24</ns2:QuantizationBits>
            <ns2:ChannelCount>1</ns2:ChannelCount>
            <ns1:SoundfieldGroupLabelSubDescriptor/>
            <ns1:AudioChannelLabelSubDescriptor/>
        </ns1:WAVEPCMDescriptor>"#;
        let issues = check_audio_mca(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("AudioSampleRateUnsupported")),
            "expected AudioSampleRateUnsupported, got: {:#?}",
            issues
        );
    }

    #[test]
    fn flags_channel_label_count_mismatch() {
        // ChannelCount=2 but only one AudioChannelLabelSubDescriptor.
        let xml = r#"<ns1:WAVEPCMDescriptor>
            <ns2:AudioSampleRate>48000/1</ns2:AudioSampleRate>
            <ns2:QuantizationBits>24</ns2:QuantizationBits>
            <ns2:ChannelCount>2</ns2:ChannelCount>
            <ns1:SoundfieldGroupLabelSubDescriptor/>
            <ns1:AudioChannelLabelSubDescriptor/>
        </ns1:WAVEPCMDescriptor>"#;
        let issues = check_audio_mca(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("ChannelLabelCountMismatch")),
            "expected ChannelLabelCountMismatch, got: {:#?}",
            issues
        );
    }

    #[test]
    fn flags_soundfield_group_label_count_not_one() {
        // Two SoundfieldGroupLabelSubDescriptors — must be exactly 1.
        let xml = r#"<ns1:WAVEPCMDescriptor>
            <ns2:AudioSampleRate>48000/1</ns2:AudioSampleRate>
            <ns2:QuantizationBits>24</ns2:QuantizationBits>
            <ns2:ChannelCount>2</ns2:ChannelCount>
            <ns1:SoundfieldGroupLabelSubDescriptor/>
            <ns1:SoundfieldGroupLabelSubDescriptor/>
            <ns1:AudioChannelLabelSubDescriptor/>
            <ns1:AudioChannelLabelSubDescriptor/>
        </ns1:WAVEPCMDescriptor>"#;
        let issues = check_audio_mca(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("SoundFieldGroupLabelCount")),
            "expected SoundFieldGroupLabelCount, got: {:#?}",
            issues
        );
    }

    #[test]
    fn skips_when_no_sound_descriptor() {
        // Video-only RegXML — no audio fields. All audio checks
        // must be silent.
        let xml = r#"<ns1:Preface>
            <ns1:CDCIDescriptor>
                <ns2:SampleRate>24000/1001</ns2:SampleRate>
            </ns1:CDCIDescriptor>
        </ns1:Preface>"#;
        let issues = check_audio_mca(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues.is_empty(),
            "video-only RegXML should produce no audio diagnostics, got: {:#?}",
            issues
        );
    }
}
