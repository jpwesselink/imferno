//! ST 2067-2 §5.3 audio essence rules, evaluated against the RegXML
//! produced by `mxf::metadata::parse_mxf_to_regxml`.
//!
//! Covers the MXF audio descriptor + its MCA (Multi-Channel Audio)
//! sub-descriptors per ST 2067-2 §5.3:
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

use crate::diagnostics::{Location, ValidationIssue};
use crate::mxf::codes::{St2067_2_2016, St377_4_2012};

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

    // ST 2067-203:2023 (S-ADM) audio is wrapped as MGA (Multichannel
    // Group Audio) per §6 and carries an `MGASoundEssenceDescriptor`
    // rather than a `WAVEPCMDescriptor`. ST 2067-2 §5.3.4.1's "must be
    // WAVE PCM" pre-dates that addition; when SMPTE introduced SADM/MGA
    // they explicitly opened a parallel audio essence path for IMF.
    // None of the §5.3.* MCA rules below apply to MGA tracks (they have
    // their own descriptor hierarchy + their own subdescriptors —
    // `MGAAudioMetadataSubDescriptor`, `MGASoundfieldGroupLabelSubDescriptor`,
    // and `SADMAudioMetadataSubDescriptor`). Skip the whole §5.3 audio
    // block here so ST 2067-203 packages don't get a false-positive
    // `SoundDescriptorNotWAVEPCM` plus a cascade of channel-count and
    // sample-rate mismatches.
    //
    // ST 2067-203 native validation is tracked separately; this branch
    // only suppresses the WAVE-PCM-specific rules that don't apply.
    //
    // ST 2067-201 (IAB) opens the same kind of parallel audio essence
    // path: IAB tracks carry an `IABEssenceDescriptor` (+
    // `IABSoundfieldLabelSubDescriptor`), not a WAVEPCMDescriptor —
    // confirmed against the real Atmos track in the IAB CompleteIMP
    // corpus fixture. The IAB descriptor has its own rule set in
    // `validation/iab.rs` (ST 2067-201 §5.9); the §5.3 WAVE PCM rules
    // below don't apply to it either. (Spec-conformance audit AUDIT-1.)
    let is_plugin_audio_essence =
        regxml.contains("MGASoundEssenceDescriptor") || regxml.contains("IABEssenceDescriptor");
    if is_plugin_audio_essence {
        return issues;
    }

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
            ValidationIssue::from_code(St2067_2_2016::SoundDescriptorNotWAVEPCM,
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
                ValidationIssue::from_code(St2067_2_2016::AudioSampleRateUnsupported,
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
                ValidationIssue::from_code(St2067_2_2016::QuantizationBitsNot24,
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

    // ST 2067-2 §5.3.6 supports two channel-labeling modes:
    //
    //   - **Mode A** — the WAVEPCMDescriptor declares a
    //     `ChannelAssignment` UL pointing at a SMPTE-registered
    //     channel layout (e.g. `urn:smpte:ul:…04020210.05010000`
    //     for an ADM channel layout). The standard label is
    //     authoritative; no per-channel sub-descriptors are required.
    //   - **Mode B** — `AudioChannelLabelSubDescriptor` per
    //     channel + exactly one `SoundfieldGroupLabelSubDescriptor`,
    //     each with an `MCALinkID` linking them.
    //
    // The §5.3.6.2 / §5.3.6.3 rules below only apply in Mode B.
    // We detect Mode A by: `ChannelAssignment` present and no
    // `AudioChannelLabelSubDescriptor` elements (a deliberate
    // omission, not an accidental one). Surfaced by the Fraunhofer
    // SMPTE WG ST 2067-204 corpus, where bed-channel audio tracks
    // use Mode A.
    let channel_count =
        extract_field(regxml, "ChannelCount").and_then(|c| c.trim().parse::<u32>().ok());
    let channel_labels = count_elements(regxml, "AudioChannelLabelSubDescriptor");
    let soundfield_count = count_elements(regxml, "SoundfieldGroupLabelSubDescriptor");
    let channel_assignment = extract_field(regxml, "ChannelAssignment")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let is_mode_a = channel_labels == 0 && channel_assignment.is_some();

    if !is_mode_a {
        // §5.3.6.2 — Mode B: number of AudioChannelLabelSubDescriptors
        // must equal ChannelCount on the WAVEPCMDescriptor.
        if let Some(cc) = channel_count {
            if (channel_labels as u32) != cc {
                issues.push(
                    ValidationIssue::from_code(St2067_2_2016::ChannelLabelCountMismatch,
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

        // §5.3.6.3 — Mode B: exactly one SoundfieldGroupLabelSubDescriptor.
        if soundfield_count != 1 {
            issues.push(
                ValidationIssue::from_code(St2067_2_2016::SoundFieldGroupLabelCount,
                    format!(
                        "MXF {} carries {} SoundfieldGroupLabelSubDescriptor(s) — ST 2067-2 §5.3.6.3 requires exactly one",
                        path.display(),
                        soundfield_count,
                    ),
                )
                .with_location(Location::new().with_file(path.to_path_buf())),
            );
        }
    }

    // ST 377-4 §6.3.2 — every AudioChannelLabelSubDescriptor MUST
    // carry an MCALinkID that points to its enclosing SoundfieldGroup,
    // and the SoundfieldGroupLabelSubDescriptor MUST carry its own
    // MCALinkID. We count occurrences relative to the count of each
    // sub-descriptor type; a deficit means at least one label is
    // missing the linking UUID — the SoundfieldGroupLinkID field on
    // an AudioChannelLabelSubDescriptor must be non-null.
    //
    // Skip in Mode A (ChannelAssignment UL only) — the linking rules
    // are conditional on the presence of per-channel sub-descriptors,
    // which don't exist in Mode A.
    let mca_link_count = count_elements(regxml, "MCALinkID");
    let expected_link_count = channel_labels + soundfield_count;
    if !is_mode_a && mca_link_count < expected_link_count {
        issues.push(
            ValidationIssue::from_code(
                St377_4_2012::MCALinkIDMissing,
                format!(
                    "MXF {} carries {} MCALinkID(s) but expected {} ({} channel-label + {} \
                     soundfield-group). ST 377-4 §6.3.2 requires every MCA sub-descriptor to \
                     carry an MCALinkID.",
                    path.display(),
                    mca_link_count,
                    expected_link_count,
                    channel_labels,
                    soundfield_count,
                ),
            )
            .with_location(Location::new().with_file(path.to_path_buf())),
        );
    }

    // ST 377-4 §6.3.2 — each AudioChannelLabelSubDescriptor's
    // SoundfieldGroupLinkID MUST equal the SoundfieldGroup's MCALinkID
    // so the channel's group membership is unambiguous. Mode B only.
    if !is_mode_a {
        if let Some(sg_link) = extract_field(regxml, "MCALinkID") {
            // Walk every SoundfieldGroupLinkID and confirm it matches
            // the SoundfieldGroup's MCALinkID (the first MCALinkID in
            // the RegXML emit-order is the SoundfieldGroup's, since
            // that descriptor is emitted before its channel children).
            for sf_link in extract_all_fields(regxml, "SoundfieldGroupLinkID") {
                if sf_link.trim() != sg_link.trim() {
                    issues.push(
                        ValidationIssue::from_code(
                            St377_4_2012::SoundfieldGroupLinkIDMismatch,
                            format!(
                                "MXF {} carries an AudioChannelLabelSubDescriptor with \
                                 SoundfieldGroupLinkID '{}' that doesn't match the SoundfieldGroup \
                                 MCALinkID '{}' (ST 377-4 §6.3.2).",
                                path.display(),
                                sf_link.trim(),
                                sg_link.trim(),
                            ),
                        )
                        .with_location(Location::new().with_file(path.to_path_buf())),
                    );
                    break; // one diagnostic is enough; aggregation collapses repeats
                }
            }
        }
    }

    // ST 2067-2 §5.3.6.2 — every channel ID 1..ChannelCount must have
    // an AudioChannelLabelSubDescriptor. Mode A skips (no per-channel
    // descriptors by definition — the ChannelAssignment UL carries the
    // full layout).
    if !is_mode_a {
        if let Some(cc) = channel_count {
            let channel_ids: std::collections::HashSet<u32> =
                extract_all_fields(regxml, "MCAChannelID")
                    .into_iter()
                    .filter_map(|s| s.trim().parse::<u32>().ok())
                    .collect();
            for expected in 1..=cc {
                if !channel_ids.contains(&expected) {
                    issues.push(
                        ValidationIssue::from_code(
                            St2067_2_2016::MCAChannelIDMissing,
                            format!(
                                "MXF {} declares ChannelCount = {} but no \
                                 AudioChannelLabelSubDescriptor carries MCAChannelID = {} — \
                                 every channel 1..N must have a label per ST 2067-2 §5.3.6.2.",
                                path.display(),
                                cc,
                                expected,
                            ),
                        )
                        .with_location(Location::new().with_file(path.to_path_buf())),
                    );
                }
            }
        }
    }

    // ST 2067-2 §5.3.4.2 — ChannelAssignment UL must be one of the
    // SMPTE 428-12 MCA channel-layout ULs (prefix
    // `urn:smpte:ul:060e2b34.0401010d.04020210.…`). Anything else
    // breaks downstream channel-layout-aware tooling.
    if let Some(ca) = extract_field(regxml, "ChannelAssignment") {
        let ca = ca.trim();
        // Tolerate the variable byte-7 (registry version) by checking
        // both the documented `0401010d` form and any future
        // `0401010X` revision, plus the structural body bytes that
        // identify ST 428-12 MCA layouts.
        let prefix_ok =
            ca.starts_with("urn:smpte:ul:060e2b34.0401010") && ca.contains(".04020210.");
        if !prefix_ok {
            issues.push(
                ValidationIssue::from_code(
                    St2067_2_2016::ChannelAssignmentNotMCA,
                    format!(
                        "MXF {} declares ChannelAssignment = {} — ST 2067-2 §5.3.4.2 \
                         requires a SMPTE 428-12 MCA channel-layout UL.",
                        path.display(),
                        ca
                    ),
                )
                .with_location(Location::new().with_file(path.to_path_buf())),
            );
        }
    }

    // ST 2067-2 §5.3.3 / ST 382:2007 §10 — audio essence MUST be
    // Wave Clip-Wrapped. The "wrapping" octet sits at byte 15
    // (1-indexed, = index 14 zero-indexed) of the ContainerFormat UL.
    // Confirmed by inspecting regxmllib-rs's audio1.mxf fixture, whose
    // UL `…0d010301.02060200` has 0x02 at index 14 — clip-wrapped.
    // Anything other than 0x02 is non-conformant for IMF audio.
    if let Some(cf) = extract_field(regxml, "ContainerFormat") {
        if let Some(bytes) = parse_ul_bytes(&cf) {
            if bytes[14] != 0x02 {
                issues.push(
                    ValidationIssue::from_code(
                        St2067_2_2016::AudioNotClipWrapped,
                        format!(
                            "MXF {} audio ContainerFormat UL byte 15 = 0x{:02x} \
                             — ST 2067-2 §5.3.3 / ST 382 §10 require Clip-Wrapped (0x02). \
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

    // ST 2067-2 §5.3 — sound essence SHOULD carry an RFC-5646 spoken
    // language tag (`RFC5646SpokenLanguage`). Most delivery pipelines
    // require it for routing; emitted as Warning so operators can
    // tighten via `RulesConfig` when their pipeline mandates it.
    if !regxml.contains(":RFC5646SpokenLanguage") {
        issues.push(
            ValidationIssue::from_code(
                St2067_2_2016::RFC5646SpokenLanguageMissing,
                format!(
                    "MXF {} sound descriptor is missing RFC5646SpokenLanguage — ST 2067-2 \
                     §5.3 recommends declaring the spoken-language BCP-47 tag.",
                    path.display(),
                ),
            )
            .with_location(Location::new().with_file(path.to_path_buf())),
        );
    }

    // ST 2067-2 §5.3.6.5 — delivery-grade audio MCA: SoundfieldGroup
    // SHALL carry MCATitle, MCATitleVersion, MCAAudioContentKind,
    // MCAAudioElementKind. Emitted as Warning since not every IMF
    // profile requires them, but mainstream delivery pipelines
    // (Netflix etc.) do.
    if soundfield_count > 0 {
        // Pair each field name (used to grep the RegXML + render the
        // diagnostic message) with the typed enum variant that carries
        // its catalogue code, severity, and category.
        for (required, code) in &[
            ("MCATitle", St2067_2_2016::SoundfieldGroupMissingMCATitle),
            (
                "MCATitleVersion",
                St2067_2_2016::SoundfieldGroupMissingMCATitleVersion,
            ),
            (
                "MCAAudioContentKind",
                St2067_2_2016::SoundfieldGroupMissingMCAAudioContentKind,
            ),
            (
                "MCAAudioElementKind",
                St2067_2_2016::SoundfieldGroupMissingMCAAudioElementKind,
            ),
        ] {
            if !regxml.contains(&format!(":{required}")) {
                issues.push(
                    ValidationIssue::from_code(
                        *code,
                        format!(
                            "MXF {} SoundfieldGroupLabelSubDescriptor is missing {} — \
                             ST 2067-2 §5.3.6.5 recommends this field for delivery-grade \
                             audio MCA.",
                            path.display(),
                            required,
                        ),
                    )
                    .with_location(Location::new().with_file(path.to_path_buf())),
                );
            }
        }
    }

    issues
}

/// Extract the text content of every occurrence of a SMPTE field
/// element (`extract_field` returns only the first). Used for fields
/// that legitimately repeat per channel — e.g. `MCAChannelID`,
/// `SoundfieldGroupLinkID`.
///
/// Walks every `:<LocalName>` occurrence and probes whether it is the
/// open or close form via the preceding character. Handles open tags
/// with attributes (e.g. `<ns2:Foo xmlns:ns2="…">`) which the simple
/// `:Foo>` substring probe misses.
///
/// # Assumptions about the upstream RegXML serializer
///
/// This walker is byte-substring based, not event-driven, so it relies
/// on the `regxml` emitter producing output that follows a few rules.
/// They hold for `regxml` 0.x as integrated here; an upstream change
/// that violates any of these would silently mis-parse:
///
/// 1. **No CDATA sections** — `<![CDATA[ ... ]]>` blocks could contain
///    the literal `</…>` close-tag pattern and confuse the close-tag
///    detector. RegXML emits all values as plain escaped text.
/// 2. **No XML comments inside elements** — `<!-- … -->` could contain
///    `<` characters that the `<`-rewind logic misinterprets as a tag.
/// 3. **No `>` in attribute values** — the open-tag scan walks to the
///    first `>` after the local-name match. A literal `&gt;`-escaped
///    `>` is fine; a raw `>` inside an attribute would prematurely end
///    the open tag.
/// 4. **One root namespace style** — `regxml` consistently emits
///    `<nsN:LocalName>` with a single colon, never multiple prefixes
///    or DOM-style namespace overrides on every element.
///
/// If any of these change in a future `regxml` release, switch to an
/// event-driven `quick_xml::reader::Reader` pass — `quick_xml` already
/// powers the higher-level CPL/AssetMap deserialisers.
pub(crate) fn extract_all_fields(xml: &str, local_name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let probe = format!(":{local_name}");
    let close_form = format!(":{local_name}>");
    let mut cursor = 0;
    while let Some(rel) = xml[cursor..].find(&probe) {
        let abs = cursor + rel;
        // Boundary check: the char *after* the local name must be a
        // legal tag terminator. Otherwise we hit a name with this as
        // a prefix (e.g. `:MCAChannel` matched on `:MCAChannelID`).
        let next = xml.as_bytes().get(abs + probe.len()).copied();
        if !matches!(
            next,
            Some(b'>') | Some(b' ') | Some(b'/') | Some(b'\t') | Some(b'\n')
        ) {
            cursor = abs + probe.len();
            continue;
        }
        // Walk back to the nearest `<` and decide open vs close.
        let Some(tag_start) = xml[..abs].rfind('<') else {
            break;
        };
        let is_close = xml[tag_start..].starts_with("</");
        if is_close {
            cursor = abs + close_form.len();
            continue;
        }
        // Open tag — body starts after this tag's `>` (which may be
        // far past the local name if attributes are present).
        let Some(open_end_rel) = xml[abs..].find('>') else {
            break;
        };
        let body_start = abs + open_end_rel + 1;
        // Find the next close form `…:LocalName>` after the body.
        let Some(close_rel) = xml[body_start..].find(&close_form) else {
            break;
        };
        let close_abs = body_start + close_rel;
        // Confirm it's actually a closing tag (preceded by `</`).
        let Some(close_tag_start) = xml[..close_abs].rfind('<') else {
            break;
        };
        if !xml[close_tag_start..].starts_with("</") {
            cursor = close_abs + close_form.len();
            continue;
        }
        // Body ends where the close tag STARTS (the `<` of `</…>`),
        // not where the `:LocalName>` substring lives mid-close-tag.
        let body = xml[body_start..close_tag_start].trim();
        out.push(body.to_string());
        cursor = close_abs + close_form.len();
    }
    out
}

/// Extract the text content of the first occurrence of a SMPTE field
/// element in the RegXML stream. Matches by local name (suffix after
/// the namespace prefix) so we don't depend on which prefix the
/// `regxml` emitter happens to pick.
///
/// Returns `None` if the field isn't found. Returns the trimmed text
/// (whitespace stripped) on success.
pub(crate) fn extract_field(xml: &str, local_name: &str) -> Option<String> {
    // `extract_all_fields` already does the heavier lifting with the
    // boundary check `extract_field` was missing (so `:Channel`
    // doesn't false-match on `:ChannelCount`). Reuse it and take the
    // first hit — cheaper than maintaining two near-identical walkers,
    // and removes a real bug where the original `extract_field`
    // accepted prefix-only matches and then failed to find a close
    // tag, returning `None` on perfectly valid input.
    extract_all_fields(xml, local_name).into_iter().next()
}

/// Count the number of occurrences of an element by local name. Uses
/// the same prefix-agnostic matching as `extract_field`. Counts open
/// tags only (so self-closing forms are also detected if the writer
/// emits them).
pub(crate) fn count_elements(xml: &str, local_name: &str) -> usize {
    let open_token = format!(":{local_name}");
    let mut count = 0;
    let mut search_from = 0;
    while let Some(rel) = xml[search_from..].find(&open_token) {
        let abs = search_from + rel;
        let next_char = xml.as_bytes().get(abs + open_token.len()).copied();
        // Match boundary so `:Channel` doesn't match `:ChannelCount`.
        if matches!(
            next_char,
            Some(b'>') | Some(b' ') | Some(b'/') | Some(b'\t') | Some(b'\n')
        ) {
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

/// Decode a `urn:smpte:ul:XXXXXXXX.XXXXXXXX.XXXXXXXX.XXXXXXXX` UL into
/// a 16-byte array. Returns `None` for any unexpected shape (URN
/// prefix missing, wrong byte count, non-hex characters) so callers
/// can fall through to skipping the check rather than panicking on
/// malformed input.
pub(crate) fn parse_ul_bytes(urn: &str) -> Option<[u8; 16]> {
    let body = urn.trim().strip_prefix("urn:smpte:ul:")?;
    let hex: String = body.chars().filter(|c| *c != '.').collect();
    if hex.len() != 32 {
        return None;
    }
    let mut out = [0u8; 16];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let hi = (chunk[0] as char).to_digit(16)?;
        let lo = (chunk[1] as char).to_digit(16)?;
        out[i] = ((hi as u8) << 4) | (lo as u8);
    }
    Some(out)
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
    use crate::diagnostics::Severity;
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
    fn audio1_clean_fixture_passes_all_error_level_audio_mca_checks() {
        // audio1.mxf is a well-formed PCM stereo fixture for Error-level
        // §5.3 / ST 377-4 checks (WAVEPCM, sample rate, quant bits,
        // channel-label count, soundfield-group count, MCALinkID
        // presence, SoundfieldGroupLinkID match, MCAChannelID coverage).
        //
        // It legitimately lacks Netflix-grade Warning fields (MCATitle
        // et al. per §5.3.6.5) since it's a regxmllib test asset, not a
        // Netflix delivery. So we filter to Error severity here —
        // Warnings are allowed and tested separately in
        // `audio1_fires_warnings_for_missing_netflix_grade_mca_fields`.
        let (xml, path) = audio1_regxml();
        let issues = check_audio_mca(&xml, &path);
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Error || i.severity == Severity::Critical)
            .collect();
        assert!(
            errors.is_empty(),
            "audio1.mxf should pass all Error-level §5.3 checks. Got Errors: {:#?}",
            errors
        );
    }

    #[test]
    fn audio1_fires_warnings_for_missing_netflix_grade_mca_fields() {
        // audio1.mxf doesn't carry MCATitle / MCATitleVersion /
        // MCAAudioContentKind / MCAAudioElementKind — these are
        // Netflix-grade requirements per §5.3.6.5 and fire as
        // Warnings on this fixture.
        let (xml, path) = audio1_regxml();
        let issues = check_audio_mca(&xml, &path);
        for field in &[
            "MCATitle",
            "MCATitleVersion",
            "MCAAudioContentKind",
            "MCAAudioElementKind",
        ] {
            assert!(
                issues
                    .iter()
                    .any(|i| i.code.contains(&format!("SoundfieldGroupMissing/{field}"))),
                "expected SoundfieldGroupMissing/{field} warning on audio1.mxf, got: {:#?}",
                issues
            );
        }
    }

    #[test]
    fn extract_all_fields_returns_every_occurrence() {
        let xml = r#"
            <ns1:MCAChannelID>1</ns1:MCAChannelID>
            <ns1:MCAChannelID>2</ns1:MCAChannelID>
            <ns1:MCAChannelID>3</ns1:MCAChannelID>
        "#;
        let vs = extract_all_fields(xml, "MCAChannelID");
        assert_eq!(vs, vec!["1", "2", "3"]);
    }

    #[test]
    fn flags_soundfield_group_link_id_mismatch() {
        let xml = r#"<ns1:WAVEPCMDescriptor>
            <ns2:AudioSampleRate>48000/1</ns2:AudioSampleRate>
            <ns2:QuantizationBits>24</ns2:QuantizationBits>
            <ns2:ChannelCount>1</ns2:ChannelCount>
            <ns1:SoundfieldGroupLabelSubDescriptor>
                <ns2:MCALinkID>urn:uuid:11111111-0000-0000-0000-000000000001</ns2:MCALinkID>
            </ns1:SoundfieldGroupLabelSubDescriptor>
            <ns1:AudioChannelLabelSubDescriptor>
                <ns2:MCALinkID>urn:uuid:99999999-0000-0000-0000-000000000099</ns2:MCALinkID>
                <ns2:MCAChannelID>1</ns2:MCAChannelID>
                <ns2:SoundfieldGroupLinkID>urn:uuid:99999999-0000-0000-0000-000000000099</ns2:SoundfieldGroupLinkID>
            </ns1:AudioChannelLabelSubDescriptor>
        </ns1:WAVEPCMDescriptor>"#;
        let issues = check_audio_mca(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("SoundfieldGroupLinkIDMismatch")),
            "expected SoundfieldGroupLinkIDMismatch, got: {:#?}",
            issues
        );
    }

    #[test]
    fn flags_missing_mca_channel_id_in_range() {
        // ChannelCount=3 but only MCAChannelID=1 and 3 present — id 2 missing.
        let xml = r#"<ns1:WAVEPCMDescriptor>
            <ns2:AudioSampleRate>48000/1</ns2:AudioSampleRate>
            <ns2:QuantizationBits>24</ns2:QuantizationBits>
            <ns2:ChannelCount>3</ns2:ChannelCount>
            <ns1:SoundfieldGroupLabelSubDescriptor>
                <ns2:MCALinkID>urn:uuid:11111111-0000-0000-0000-000000000001</ns2:MCALinkID>
            </ns1:SoundfieldGroupLabelSubDescriptor>
            <ns1:AudioChannelLabelSubDescriptor>
                <ns2:MCALinkID>urn:uuid:11111111-0000-0000-0000-000000000001</ns2:MCALinkID>
                <ns2:MCAChannelID>1</ns2:MCAChannelID>
                <ns2:SoundfieldGroupLinkID>urn:uuid:11111111-0000-0000-0000-000000000001</ns2:SoundfieldGroupLinkID>
            </ns1:AudioChannelLabelSubDescriptor>
            <ns1:AudioChannelLabelSubDescriptor>
                <ns2:MCALinkID>urn:uuid:11111111-0000-0000-0000-000000000001</ns2:MCALinkID>
                <ns2:MCAChannelID>3</ns2:MCAChannelID>
                <ns2:SoundfieldGroupLinkID>urn:uuid:11111111-0000-0000-0000-000000000001</ns2:SoundfieldGroupLinkID>
            </ns1:AudioChannelLabelSubDescriptor>
            <ns1:AudioChannelLabelSubDescriptor>
                <ns2:MCALinkID>urn:uuid:11111111-0000-0000-0000-000000000001</ns2:MCALinkID>
                <ns2:MCAChannelID>3</ns2:MCAChannelID>
                <ns2:SoundfieldGroupLinkID>urn:uuid:11111111-0000-0000-0000-000000000001</ns2:SoundfieldGroupLinkID>
            </ns1:AudioChannelLabelSubDescriptor>
        </ns1:WAVEPCMDescriptor>"#;
        let issues = check_audio_mca(xml, std::path::Path::new("/synth.mxf"));
        let missing_ids: Vec<_> = issues
            .iter()
            .filter(|i| i.code.contains("MCAChannelIDMissing"))
            .collect();
        assert!(
            !missing_ids.is_empty(),
            "expected MCAChannelIDMissing for id 2, got: {:#?}",
            issues
        );
        assert!(
            missing_ids
                .iter()
                .any(|i| i.message.contains("MCAChannelID = 2")),
            "expected diagnostic to name channel id 2, got: {:#?}",
            missing_ids
        );
    }

    #[test]
    fn extract_field_handles_namespaced_tags() {
        let xml = r#"
            <ns2:ChannelCount xmlns:ns2="x">2</ns2:ChannelCount>
            <ns2:QuantizationBits xmlns:ns2="x">24</ns2:QuantizationBits>
        "#;
        assert_eq!(extract_field(xml, "ChannelCount").as_deref(), Some("2"));
        assert_eq!(
            extract_field(xml, "QuantizationBits").as_deref(),
            Some("24")
        );
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

    // ── FIX-20: pin the documented assumptions about RegXML output ──────────

    /// Open tags with attributes (e.g. `xmlns:nsN="…"`) must still match.
    /// The walker has to skip past the attribute payload to the `>` that
    /// terminates the open tag before scanning for the body.
    #[test]
    fn extract_field_handles_open_tag_with_attribute() {
        let xml = r#"<ns2:ChannelCount xmlns:ns2="http://example/ns">2</ns2:ChannelCount>"#;
        assert_eq!(extract_field(xml, "ChannelCount").as_deref(), Some("2"));
    }

    /// Self-closing forms must NOT register as content-bearing in
    /// `extract_field` (no body to extract). They are however legal in
    /// `count_elements`.
    #[test]
    fn extract_field_skips_self_closing_form() {
        let xml = r#"<ns1:SoundfieldGroupLabelSubDescriptor/>
            <ns1:ChannelCount>5</ns1:ChannelCount>"#;
        assert_eq!(
            extract_field(xml, "SoundfieldGroupLabelSubDescriptor"),
            None
        );
        assert_eq!(extract_field(xml, "ChannelCount").as_deref(), Some("5"));
        assert_eq!(count_elements(xml, "SoundfieldGroupLabelSubDescriptor"), 1);
    }

    /// Tag bodies surrounded by extra whitespace are trimmed.
    #[test]
    fn extract_field_trims_whitespace() {
        let xml = "<ns1:ChannelCount>   42\n  </ns1:ChannelCount>";
        assert_eq!(extract_field(xml, "ChannelCount").as_deref(), Some("42"));
    }

    /// Nested elements with the same local name are handled in order —
    /// `extract_all_fields` returns each occurrence's body, not the
    /// concatenated outer body.
    #[test]
    fn extract_all_fields_does_not_concatenate_siblings() {
        let xml = r#"
            <ns1:MCAChannelID>1</ns1:MCAChannelID>
            <ns1:MCAChannelID>2</ns1:MCAChannelID>
            <ns1:MCAChannelID>3</ns1:MCAChannelID>
        "#;
        assert_eq!(
            extract_all_fields(xml, "MCAChannelID"),
            vec!["1".to_string(), "2".to_string(), "3".to_string()]
        );
    }

    /// Tags that share a prefix with the queried local name (e.g.
    /// `:Channel` vs `:ChannelCount`) must not collide. This regression
    /// pins the boundary-check on the byte after the local name.
    #[test]
    fn extract_field_does_not_collide_on_prefix() {
        let xml = r#"<ns1:Channel>X</ns1:Channel>
            <ns1:ChannelCount>2</ns1:ChannelCount>"#;
        assert_eq!(extract_field(xml, "Channel").as_deref(), Some("X"));
        assert_eq!(extract_field(xml, "ChannelCount").as_deref(), Some("2"));
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
    fn flags_missing_mca_link_ids() {
        // 2 channel labels + 1 soundfield group → expected 3 MCALinkIDs.
        // This snippet has 0 → trips ST377-4 §6.3.2.
        let xml = r#"<ns1:WAVEPCMDescriptor>
            <ns2:AudioSampleRate>48000/1</ns2:AudioSampleRate>
            <ns2:QuantizationBits>24</ns2:QuantizationBits>
            <ns2:ChannelCount>2</ns2:ChannelCount>
            <ns1:SoundfieldGroupLabelSubDescriptor/>
            <ns1:AudioChannelLabelSubDescriptor/>
            <ns1:AudioChannelLabelSubDescriptor/>
        </ns1:WAVEPCMDescriptor>"#;
        let issues = check_audio_mca(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues.iter().any(|i| i.code.contains("MCALinkIDMissing")),
            "expected MCALinkIDMissing, got: {:#?}",
            issues
        );
    }

    #[test]
    fn parse_ul_bytes_decodes_canonical_urn() {
        // audio1.mxf's ContainerFormat: bytes by position are
        // 06 0e 2b 34 04 01 01 01 0d 01 03 01 02 06 02 00.
        let ul = "urn:smpte:ul:060e2b34.04010101.0d010301.02060200";
        let bytes = parse_ul_bytes(ul).unwrap();
        assert_eq!(bytes[0], 0x06);
        assert_eq!(
            bytes[14], 0x02,
            "byte 15 (1-indexed) is the wrapping octet — 0x02 = clip"
        );
        assert_eq!(bytes[15], 0x00);
    }

    #[test]
    fn parse_ul_bytes_rejects_malformed_input() {
        assert!(parse_ul_bytes("not-a-urn").is_none());
        assert!(parse_ul_bytes("urn:smpte:ul:short").is_none());
        assert!(parse_ul_bytes("urn:smpte:ul:zzzzzzzz.zzzzzzzz.zzzzzzzz.zzzzzzzz").is_none());
    }

    #[test]
    fn flags_audio_not_clip_wrapped() {
        // ContainerFormat UL byte 13 = 0x01 (frame-wrapped) — must trip.
        let xml = r#"<ns1:WAVEPCMDescriptor>
            <ns2:AudioSampleRate>48000/1</ns2:AudioSampleRate>
            <ns2:QuantizationBits>24</ns2:QuantizationBits>
            <ns2:ChannelCount>1</ns2:ChannelCount>
            <ns2:ChannelAssignment>urn:smpte:ul:060e2b34.0401010d.04020210.04010000</ns2:ChannelAssignment>
            <ns2:ContainerFormat>urn:smpte:ul:060e2b34.04010101.0d010301.02060001</ns2:ContainerFormat>
            <ns1:SoundfieldGroupLabelSubDescriptor>
                <ns2:MCALinkID>urn:uuid:11111111-0000-0000-0000-000000000001</ns2:MCALinkID>
            </ns1:SoundfieldGroupLabelSubDescriptor>
            <ns1:AudioChannelLabelSubDescriptor>
                <ns2:MCALinkID>urn:uuid:11111111-0000-0000-0000-000000000001</ns2:MCALinkID>
                <ns2:MCAChannelID>1</ns2:MCAChannelID>
                <ns2:SoundfieldGroupLinkID>urn:uuid:11111111-0000-0000-0000-000000000001</ns2:SoundfieldGroupLinkID>
            </ns1:AudioChannelLabelSubDescriptor>
        </ns1:WAVEPCMDescriptor>"#;
        let issues = check_audio_mca(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("AudioNotClipWrapped")),
            "expected AudioNotClipWrapped, got: {:#?}",
            issues
        );
    }

    #[test]
    fn flags_non_mca_channel_assignment_ul() {
        let xml = r#"<ns1:WAVEPCMDescriptor>
            <ns2:AudioSampleRate>48000/1</ns2:AudioSampleRate>
            <ns2:QuantizationBits>24</ns2:QuantizationBits>
            <ns2:ChannelCount>1</ns2:ChannelCount>
            <ns2:ChannelAssignment>urn:smpte:ul:060e2b34.0401010d.04020110.04010000</ns2:ChannelAssignment>
            <ns1:SoundfieldGroupLabelSubDescriptor>
                <ns2:MCALinkID>urn:uuid:11111111-0000-0000-0000-000000000001</ns2:MCALinkID>
            </ns1:SoundfieldGroupLabelSubDescriptor>
            <ns1:AudioChannelLabelSubDescriptor>
                <ns2:MCALinkID>urn:uuid:11111111-0000-0000-0000-000000000001</ns2:MCALinkID>
                <ns2:MCAChannelID>1</ns2:MCAChannelID>
                <ns2:SoundfieldGroupLinkID>urn:uuid:11111111-0000-0000-0000-000000000001</ns2:SoundfieldGroupLinkID>
            </ns1:AudioChannelLabelSubDescriptor>
        </ns1:WAVEPCMDescriptor>"#;
        let issues = check_audio_mca(xml, std::path::Path::new("/synth.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("ChannelAssignmentNotMCA")),
            "expected ChannelAssignmentNotMCA, got: {:#?}",
            issues
        );
    }

    #[test]
    fn video1_fixture_produces_zero_audio_mca_diagnostics() {
        // video1.mxf is a CDCI/JPEG2000 video — no audio descriptor
        // at all. The audio MCA pipeline must be silent: false-firing
        // on video assets would dump nonsensical channel/sample-rate
        // complaints on every video track of every IMF package.
        let path = fixture("video1.mxf");
        let opts = regxml::MxfFragmentOptions {
            partition: regxml::PartitionTarget::Header,
            ..Default::default()
        };
        let xml = parse_mxf_to_regxml(&path, opts).expect("video1 → RegXML");
        let issues = check_audio_mca(&xml, &path);
        assert!(
            issues.is_empty(),
            "audio MCA pipeline must be silent on video1.mxf (no audio descriptor), got: {:#?}",
            issues
        );
    }

    #[test]
    fn fires_warning_when_rfc5646_spoken_language_missing() {
        // audio1 lacks RFC5646SpokenLanguage — should fire a Warning.
        let (xml, path) = audio1_regxml();
        let issues = check_audio_mca(&xml, &path);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("RFC5646SpokenLanguageMissing")),
            "expected RFC5646SpokenLanguageMissing on audio1, got: {:#?}",
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

    #[test]
    fn skips_when_mga_sound_essence_descriptor_present() {
        // ST 2067-203 SADM/MGA audio uses `MGASoundEssenceDescriptor`
        // instead of `WAVEPCMDescriptor`. The §5.3 WAVE-PCM rules do
        // not apply — firing `SoundDescriptorNotWAVEPCM` (or any of the
        // channel-count / sample-rate cascade) would be a false positive
        // on every valid SADM IMF.
        let xml = r#"<ns1:Preface>
            <ns1:MGASoundEssenceDescriptor>
                <ns2:AudioSampleRate>48000/1</ns2:AudioSampleRate>
                <ns2:ChannelCount>2</ns2:ChannelCount>
                <ns2:SubDescriptors>
                    <ns1:MGAAudioMetadataSubDescriptor/>
                    <ns1:MGASoundfieldGroupLabelSubDescriptor/>
                    <ns1:SADMAudioMetadataSubDescriptor/>
                </ns2:SubDescriptors>
            </ns1:MGASoundEssenceDescriptor>
        </ns1:Preface>"#;
        let issues = check_audio_mca(xml, std::path::Path::new("/sadm.mxf"));
        assert!(
            issues.is_empty(),
            "MGA/SADM RegXML should produce no audio diagnostics (WAVE PCM rules don't apply), got: {:#?}",
            issues
        );
    }

    #[test]
    fn skips_when_iab_essence_descriptor_present() {
        // ST 2067-201 IAB audio carries an `IABEssenceDescriptor`
        // (+ IABSoundfieldLabelSubDescriptor), not a WAVEPCMDescriptor.
        // Same false-positive class as the SADM case: the §5.3 rules
        // must not fire. Shape mirrors the RegXML emitted for the real
        // Atmos track in test-data/IAB/CompleteIMP. (AUDIT-1.)
        let xml = r#"<ns1:Preface>
            <ns1:IABEssenceDescriptor>
                <ns2:AudioSampleRate>48000/1</ns2:AudioSampleRate>
                <ns2:ChannelCount>0</ns2:ChannelCount>
                <ns2:SubDescriptors>
                    <ns1:IABSoundfieldLabelSubDescriptor/>
                </ns2:SubDescriptors>
            </ns1:IABEssenceDescriptor>
        </ns1:Preface>"#;
        let issues = check_audio_mca(xml, std::path::Path::new("/iab.mxf"));
        assert!(
            issues.is_empty(),
            "IAB RegXML should produce no §5.3 audio diagnostics (WAVE PCM rules don't apply), got: {:#?}",
            issues
        );
    }

    #[test]
    fn real_iab_fixture_produces_no_wave_pcm_false_positive() {
        // End-to-end against the real Atmos MXF from the IAB CompleteIMP
        // corpus package.
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-data/IAB/CompleteIMP/meridian_2398_Atmos_20190603_first10s.mxf");
        if !path.exists() {
            // Corpus MXFs are fetched via scripts/fetch-test-data.sh —
            // skip quietly when absent (same convention as the other
            // fixture-gated tests).
            return;
        }
        let xml = parse_mxf_to_regxml(
            &path,
            regxml::MxfFragmentOptions {
                partition: regxml::PartitionTarget::Header,
                ..Default::default()
            },
        )
        .expect("IAB Atmos MXF → RegXML");
        let issues = check_audio_mca(&xml, &path);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("SoundDescriptorNotWAVEPCM")),
            "IAB essence must not fire SoundDescriptorNotWAVEPCM: {issues:#?}"
        );
    }

    #[test]
    fn mode_a_channel_assignment_skips_mca_subdescriptor_rules() {
        // Mode A per ST 2067-2 §5.3.6: the WAVEPCMDescriptor declares a
        // ChannelAssignment UL naming a SMPTE-registered layout; no
        // per-channel AudioChannelLabelSubDescriptors are required.
        // Surfaced by the Fraunhofer ST 2067-204 corpus (bed-channel
        // WAVE PCM tracks use Mode A) — previously fired a cascade of
        // ChannelLabelCountMismatch + SoundFieldGroupLabelCount +
        // MCAChannelIDMissing false positives.
        let xml = r#"<ns1:Preface>
            <ns1:WAVEPCMDescriptor>
                <ns2:AudioSampleRate>48000/1</ns2:AudioSampleRate>
                <ns2:QuantizationBits>24</ns2:QuantizationBits>
                <ns2:ChannelCount>5</ns2:ChannelCount>
                <ns2:ChannelAssignment>urn:smpte:ul:060e2b34.04010101.04020210.05010000</ns2:ChannelAssignment>
                <ns2:RFC5646SpokenLanguage>en</ns2:RFC5646SpokenLanguage>
            </ns1:WAVEPCMDescriptor>
        </ns1:Preface>"#;
        let issues = check_audio_mca(xml, std::path::Path::new("/mode-a.mxf"));
        let mode_b_codes: Vec<_> = issues
            .iter()
            .filter(|i| {
                i.code.contains("ChannelLabelCountMismatch")
                    || i.code.contains("SoundFieldGroupLabelCount")
                    || i.code.contains("MCAChannelIDMissing")
                    || i.code.contains("MCALinkIDMissing")
                    || i.code.contains("SoundfieldGroupLinkIDMismatch")
            })
            .collect();
        assert!(
            mode_b_codes.is_empty(),
            "Mode A (ChannelAssignment UL, no channel labels) must not fire Mode B \
             sub-descriptor rules: {mode_b_codes:#?}"
        );
    }

    #[test]
    fn missing_labels_without_channel_assignment_still_fires_mode_b_rules() {
        // Negative control for the Mode A carve-out: a WAVE PCM
        // descriptor with NEITHER ChannelAssignment NOR channel labels
        // is a plain §5.3.6.2 violation — the Mode B rules must fire.
        let xml = r#"<ns1:Preface>
            <ns1:WAVEPCMDescriptor>
                <ns2:AudioSampleRate>48000/1</ns2:AudioSampleRate>
                <ns2:QuantizationBits>24</ns2:QuantizationBits>
                <ns2:ChannelCount>2</ns2:ChannelCount>
            </ns1:WAVEPCMDescriptor>
        </ns1:Preface>"#;
        let issues = check_audio_mca(xml, std::path::Path::new("/no-labels.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("ChannelLabelCountMismatch")),
            "missing labels with no ChannelAssignment must still fire §5.3.6.2: {issues:#?}"
        );
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("SoundFieldGroupLabelCount")),
            "missing soundfield group must still fire §5.3.6.3: {issues:#?}"
        );
    }
}
