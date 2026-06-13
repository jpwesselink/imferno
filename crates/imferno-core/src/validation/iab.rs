//! SMPTE ST 2067-201 IAB Level 0 Plug-in validator.
//!
//! Implements the IAB (Immersive Audio Bitstream) plug-in constraints for
//! SMPTE ST 2067-201:2019 and ST 2067-201:2021.
//!
//! Both plug-in validators run App2E base validation (ST 2067-21) internally,
//! then apply IAB-specific descriptor and sequence constraints.

// codes live in iab_codes.rs (declared from validation/mod.rs)

use std::collections::HashSet;

use crate::cpl::{CompositionPlaylist, McaTagSymbol};
use crate::diagnostics::{Category, Location, Severity, ValidationIssue};
use crate::validation::{App2E2021, ConstraintsValidator};

use crate::validation::iab_codes::{self as iab_codes, IabCode};

// ── Public API ───────────────────────────────────────────────────────────────

/// ST 2067-201:2019 IAB Level 0 Plug-in validator.
///
/// Runs App2E base validation plus ST 2067-201:2019-specific IAB constraints.
/// In the 2019 edition ChannelCount shall be the distinguished value 0.
pub struct AppIabPlugin2019;

impl ConstraintsValidator for AppIabPlugin2019 {
    fn spec_id(&self) -> &str {
        "ST 2067-201:2019 (IAB Level 0 Plug-in)"
    }

    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        App2E2021.validate_all(cpl, true, &mut issues);
        validate_iab_descriptors(cpl, iab_codes::St2067_201_2019::for_code, true, &mut issues);
        validate_iab_sequences(cpl, iab_codes::St2067_201_2019::for_code, &mut issues);
        issues
    }
}

/// ST 2067-201:2021 IAB Level 0 Plug-in validator.
///
/// Runs App2E base validation plus ST 2067-201:2021-specific IAB constraints.
/// In the 2021 edition ChannelCount is ignored (not checked).
///
/// **Selected by explicit caller request, not by document namespace.**
/// Real-world IAB documents — including those validated against the
/// 2021 prose — declare the 2019 namespace
/// `http://www.smpte-ra.org/ns/2067-201/2019` (verified firsthand from
/// the 2021 PDF line 642 and the 2026 publication's inline schema).
/// Callers wanting the 2021 channel-count-off semantics select
/// `AppIabPlugin2021` explicitly via
/// `ValidatorSelection::app_specs = vec![AppSpecTarget::St2067_201_2021]`.
pub struct AppIabPlugin2021;

impl ConstraintsValidator for AppIabPlugin2021 {
    fn spec_id(&self) -> &str {
        "ST 2067-201:2021 (IAB Level 0 Plug-in)"
    }

    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        App2E2021.validate_all(cpl, true, &mut issues);
        validate_iab_descriptors(
            cpl,
            iab_codes::St2067_201_2021::for_code,
            false,
            &mut issues,
        );
        validate_iab_sequences(cpl, iab_codes::St2067_201_2021::for_code, &mut issues);
        issues
    }
}

/// ST 2067-201:2026 IAB Level 0 Plug-in validator.
///
/// Runs the 2021 base validation verbatim (no normative changes to
/// any existing rule between 2021 and 2026 per the spec-diff), then
/// adds the single Annex E recommendation introduced in the 2026
/// revision: every IAB Track File should expose `IABChannelSubDescriptor`
/// entries mirroring the channels of each `BedDefinition`. The check
/// fires at Warning severity and **only fires from this validator** —
/// 2019 and 2021 documents are not held to the recommendation.
///
/// Like `AppIabPlugin2021`, this is selected by explicit caller
/// request (`ValidatorSelection::app_specs`), not by namespace —
/// the 2026 publication continues to declare
/// `http://www.smpte-ra.org/ns/2067-201/2019` per its inline HTML
/// schema (firsthand-confirmed 2026-06-13).
pub struct AppIabPlugin2026;

impl ConstraintsValidator for AppIabPlugin2026 {
    fn spec_id(&self) -> &str {
        "ST 2067-201:2026 (IAB Level 0 Plug-in)"
    }

    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        let mut issues = AppIabPlugin2021.validate_cpl(cpl);
        validate_iab_channel_sub_descriptor_recommendation(cpl, &mut issues);
        issues
    }
}

/// ST 2067-201:2026 Annex E §E.2 — Warning-level recommendation.
///
/// Walks every `IABEssenceDescriptor` in the CPL and flags those
/// whose `SubDescriptors` block carries zero `IABChannelSubDescriptor`
/// entries. The 2026 prose says "should contain one instance for each
/// channel of each BedDefinition" — without parsing the IAB bitstream
/// we can't count BedDefinition channels, so we use the conservative
/// "any presence" heuristic: zero entries → warn; one or more → assume
/// the writer met the recommendation.
fn validate_iab_channel_sub_descriptor_recommendation(
    cpl: &CompositionPlaylist,
    issues: &mut Vec<ValidationIssue>,
) {
    use crate::diagnostics::codes::ValidationCode;
    use crate::validation::iab_codes::St2067_201_2026Delta;

    let edl = match &cpl.essence_descriptor_list {
        Some(edl) => edl,
        None => return,
    };

    for ed in &edl.essence_descriptors {
        let iab = match &ed.iab_essence_descriptor {
            Some(iab) => iab,
            None => continue,
        };

        let n = iab
            .sub_descriptors
            .as_ref()
            .map(|sd| sd.iab_channel_sub_descriptors.len())
            .unwrap_or(0);
        if n == 0 {
            let code = St2067_201_2026Delta::IabChannelSubDescriptorRecommended;
            let ed_loc = Location::new()
                .with_cpl(cpl.id)
                .with_path(format!("EssenceDescriptor/{}", ed.id));
            issues.push(
                ValidationIssue::new(
                    code.default_severity(),
                    code.category(),
                    code.code(),
                    format!(
                        "IABEssenceDescriptor {}: no <IABChannelSubDescriptor> entries — \
                         ST 2067-201:2026 Annex E §E.2 recommends one per channel of each BedDefinition.",
                        ed.id,
                    ),
                )
                .with_location(ed_loc),
            );
        }
    }
}

// ── Namespace URIs ────────────────────────────────────────────────────────────
//
// Both the `/ns/` and `/schemas/` forms appear on SMPTE-RA — the `/schemas/`
// path is the older convention; `/ns/` is the modern one used by ST 2067-201
// since the 2019 publication. Both URIs identify the same logical namespace
// and IAB documents use either interchangeably in xmlns declarations.
//
// **There is no separate 2021 URI.** The 2021 publication
// (`st2067-201-20201109-pub.zip`) embeds a schema with
// `targetNamespace="http://www.smpte-ra.org/ns/2067-201/2019"`, and the 2026
// publication's inline HTML schema does the same. Every IAB document in our
// corpus declares the 2019 URI regardless of which spec edition's rules
// authored it. The previously-exported `URI_2021` / `URI_2021_SCHEMAS`
// consts were removed — they never matched any real document.

pub const URI_2019: &str = "http://www.smpte-ra.org/ns/2067-201/2019";
pub const URI_2019_SCHEMAS: &str = "http://www.smpte-ra.org/schemas/2067-201/2019";

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Normalize a SMPTE UL by zeroing byte 8 (ST 298M version byte) before comparison.
fn ul_matches(provided: &str, canonical: &str) -> bool {
    fn parse_ul(s: &str) -> Option<[u8; 16]> {
        let hex = s.strip_prefix("urn:smpte:ul:")?;
        let groups: Vec<&str> = hex.split('.').collect();
        if groups.len() != 4 {
            return None;
        }
        let mut bytes = [0u8; 16];
        let mut idx = 0;
        for group in &groups {
            if group.len() != 8 {
                return None;
            }
            for i in (0..8).step_by(2) {
                bytes[idx] = u8::from_str_radix(&group[i..i + 2], 16).ok()?;
                idx += 1;
            }
        }
        bytes[7] = 0; // zero byte 8 (ST 298M: version byte shall be ignored)
        Some(bytes)
    }
    match (parse_ul(provided), parse_ul(canonical)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// ST 2067-201 §5.9: Validate IABEssenceDescriptor constraints.
///
/// `code`: edition dispatch function (e.g. `St2067_201_2019::for_code`).
/// `check_channel_count`: true for 2019 (ChannelCount shall be 0);
///                        false for 2021 (ChannelCount is ignored).
fn validate_iab_descriptors(
    cpl: &CompositionPlaylist,
    code: fn(IabCode) -> &'static str,
    check_channel_count: bool,
    issues: &mut Vec<ValidationIssue>,
) {
    const IAB_CONTAINER_FORMAT: &str = "urn:smpte:ul:060e2b34.0401010d.0d010301.021d0101";
    const IAB_SOUND_COMPRESSION: &str = "urn:smpte:ul:060e2b34.04010105.0e090604.00000000";
    const IAB_MCA_LABEL_DICT_ID: &str = "urn:smpte:ul:060e2b34.0401010d.03020221.00000000";

    let edl = match &cpl.essence_descriptor_list {
        Some(edl) => edl,
        None => return,
    };

    for ed in &edl.essence_descriptors {
        let iab = match &ed.iab_essence_descriptor {
            Some(iab) => iab,
            None => continue,
        };

        let ed_loc = Location::new()
            .with_cpl(cpl.id)
            .with_path(format!("EssenceDescriptor/{}", ed.id));

        // §5.9: Codec item shall NOT be present.
        if iab.codec.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Audio,
                    code(IabCode::CodecForbidden),
                    format!(
                        "IABEssenceDescriptor {}: Codec item shall not be present \
                         (ST 2067-201 §5.9)",
                        ed.id,
                    ),
                )
                .with_location(ed_loc.clone()),
            );
        }

        // §5.9: ElectrospatialFormulation shall NOT be present.
        if iab.electrospatial_formulation.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Audio,
                    code(IabCode::ElectrospatialFormulationForbidden),
                    format!(
                        "IABEssenceDescriptor {}: ElectrospatialFormulation shall not be \
                         present (ST 2067-201 §5.9)",
                        ed.id,
                    ),
                )
                .with_location(ed_loc.clone()),
            );
        }

        // §5.9: QuantizationBits shall be 24.
        match iab.quantization_bits {
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        Category::Audio,
                        code(IabCode::QuantizationBitsMissing),
                        format!(
                            "IABEssenceDescriptor {}: QuantizationBits is missing; \
                             ST 2067-201 §5.9 requires 24",
                            ed.id,
                        ),
                    )
                    .with_location(ed_loc.clone()),
                );
            }
            Some(qb) if qb != 24 => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Audio,
                        code(IabCode::QuantizationBitsInvalid),
                        format!(
                            "IABEssenceDescriptor {}: QuantizationBits {} shall be 24 \
                             (ST 2067-201 §5.9)",
                            ed.id, qb,
                        ),
                    )
                    .with_location(ed_loc.clone()),
                );
            }
            _ => {}
        }

        // §5.3: ContainerFormat shall be the IAB essence container UL.
        match &iab.container_format {
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        Category::Audio,
                        code(IabCode::ContainerFormatMissing),
                        format!(
                            "IABEssenceDescriptor {}: ContainerFormat is missing; \
                             should be {IAB_CONTAINER_FORMAT}",
                            ed.id,
                        ),
                    )
                    .with_location(ed_loc.clone()),
                );
            }
            Some(cf) if !ul_matches(cf, IAB_CONTAINER_FORMAT) => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Audio,
                        code(IabCode::EssenceContainerInvalid),
                        format!(
                            "IABEssenceDescriptor {}: ContainerFormat `{cf}` is not the \
                             required IAB container UL {IAB_CONTAINER_FORMAT} \
                             (ST 2067-201 §5.3)",
                            ed.id,
                        ),
                    )
                    .with_location(ed_loc.clone()),
                );
            }
            _ => {}
        }

        // §5.9: AudioSampleRate shall be 48000/1.
        match &iab.audio_sample_rate {
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        Category::Audio,
                        code(IabCode::AudioSamplingRateMissing),
                        format!(
                            "IABEssenceDescriptor {}: AudioSampleRate is missing; \
                             ST 2067-201 §5.9 requires 48000/1",
                            ed.id,
                        ),
                    )
                    .with_location(ed_loc.clone()),
                );
            }
            Some(rate) if rate.numerator != 48000 || rate.denominator != 1 => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Audio,
                        code(IabCode::AudioSamplingRateInvalid),
                        format!(
                            "IABEssenceDescriptor {}: AudioSampleRate {}/{} is not 48000/1 \
                             (ST 2067-201 §5.9)",
                            ed.id, rate.numerator, rate.denominator,
                        ),
                    )
                    .with_location(ed_loc.clone()),
                );
            }
            _ => {}
        }

        // §5.9: SoundCompression shall be the IAB compression UL.
        match &iab.sound_compression {
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        Category::Audio,
                        code(IabCode::SoundCompressionMissing),
                        format!(
                            "IABEssenceDescriptor {}: SoundCompression is missing; \
                             should be {IAB_SOUND_COMPRESSION}",
                            ed.id,
                        ),
                    )
                    .with_location(ed_loc.clone()),
                );
            }
            Some(sc) if !ul_matches(sc, IAB_SOUND_COMPRESSION) => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Audio,
                        code(IabCode::SoundCompressionInvalid),
                        format!(
                            "IABEssenceDescriptor {}: SoundCompression `{sc}` is not the \
                             required IAB UL {IAB_SOUND_COMPRESSION} (ST 2067-201 §5.9)",
                            ed.id,
                        ),
                    )
                    .with_location(ed_loc.clone()),
                );
            }
            _ => {}
        }

        // §5.9: ChannelCount shall be the distinguished value 0 (2019 only).
        if check_channel_count {
            match iab.channel_count {
                Some(cc) if cc != 0 => {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Audio,
                            code(IabCode::ChannelCountNotZero),
                            format!(
                                "IABEssenceDescriptor {}: ChannelCount {cc} shall be the \
                                 distinguished value 0 (ST 2067-201 §5.9, ST 377-1 Annex F.5)",
                                ed.id,
                            ),
                        )
                        .with_location(ed_loc.clone()),
                    );
                }
                _ => {}
            }
        }

        // §5.9: IABSoundfieldLabelSubDescriptor shall be present.
        let soundfield = iab
            .sub_descriptors
            .as_ref()
            .and_then(|sd| sd.iab_soundfield_label_sub_descriptor.as_ref());

        if soundfield.is_none() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Audio,
                    code(IabCode::SubDescriptorMissing),
                    format!(
                        "IABEssenceDescriptor {}: IABSoundfieldLabelSubDescriptor shall be \
                         present (ST 2067-201 §5.9)",
                        ed.id,
                    ),
                )
                .with_location(ed_loc.clone()),
            );
            continue; // remaining sub-descriptor checks not applicable
        }

        let sf = soundfield.unwrap();
        let sub_loc = Location::new().with_cpl(cpl.id).with_path(format!(
            "EssenceDescriptor/{}/IABSoundfieldLabelSubDescriptor",
            ed.id
        ));

        // §5.9: MCATagSymbol shall be "IAB".
        match &sf.mca_tag_symbol {
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Audio,
                        code(IabCode::MCATagSymbolMissing),
                        format!(
                            "IABEssenceDescriptor {}: IABSoundfieldLabelSubDescriptor \
                             MCATagSymbol shall be \"IAB\" (ST 2067-201 §5.9)",
                            ed.id,
                        ),
                    )
                    .with_location(sub_loc.clone()),
                );
            }
            Some(sym) if *sym != McaTagSymbol::Iab => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Audio,
                        code(IabCode::MCATagSymbolInvalid),
                        format!(
                            "IABEssenceDescriptor {}: IABSoundfieldLabelSubDescriptor \
                             MCATagSymbol shall be \"IAB\", got \"{sym:?}\" \
                             (ST 2067-201 §5.9)",
                            ed.id,
                        ),
                    )
                    .with_location(sub_loc.clone()),
                );
            }
            _ => {}
        }

        // §5.9: MCATagName shall be "IAB".
        match sf.mca_tag_name.as_deref() {
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Audio,
                        code(IabCode::MCATagNameMissing),
                        format!(
                            "IABEssenceDescriptor {}: IABSoundfieldLabelSubDescriptor \
                             MCATagName shall be \"IAB\" (ST 2067-201 §5.9)",
                            ed.id,
                        ),
                    )
                    .with_location(sub_loc.clone()),
                );
            }
            Some(name) if name != "IAB" => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Audio,
                        code(IabCode::MCATagNameInvalid),
                        format!(
                            "IABEssenceDescriptor {}: IABSoundfieldLabelSubDescriptor \
                             MCATagName shall be \"IAB\", got \"{name}\" \
                             (ST 2067-201 §5.9)",
                            ed.id,
                        ),
                    )
                    .with_location(sub_loc.clone()),
                );
            }
            _ => {}
        }

        // §5.9: MCALabelDictionaryID shall be the IAB label UL.
        match sf.mca_label_dictionary_id.as_deref() {
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Audio,
                        code(IabCode::MCALabelDictionaryIDMissing),
                        format!(
                            "IABEssenceDescriptor {}: IABSoundfieldLabelSubDescriptor \
                             MCALabelDictionaryID is missing; shall be {IAB_MCA_LABEL_DICT_ID} \
                             (ST 2067-201 §5.9)",
                            ed.id,
                        ),
                    )
                    .with_location(sub_loc.clone()),
                );
            }
            Some(id) if !ul_matches(id, IAB_MCA_LABEL_DICT_ID) => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Audio,
                        code(IabCode::MCALabelDictionaryIDInvalid),
                        format!(
                            "IABEssenceDescriptor {}: IABSoundfieldLabelSubDescriptor \
                             MCALabelDictionaryID `{id}` shall be {IAB_MCA_LABEL_DICT_ID} \
                             (ST 2067-201 §5.9)",
                            ed.id,
                        ),
                    )
                    .with_location(sub_loc.clone()),
                );
            }
            _ => {}
        }
    }
}

/// ST 2067-201 §6.2: Validate IABSequence constraints at the CPL timeline level.
///
/// - Each segment with an IABSequence shall also have a MainAudioSequence.
/// - Each IABSequence shall contain at least one Resource.
/// - Each IABSequence Resource.SourceEncoding shall reference an IABEssenceDescriptor.
fn validate_iab_sequences(
    cpl: &CompositionPlaylist,
    code: fn(IabCode) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let iab_ed_ids: HashSet<String> = cpl
        .essence_descriptor_list
        .as_ref()
        .map(|edl| {
            edl.essence_descriptors
                .iter()
                .filter(|ed| ed.iab_essence_descriptor.is_some())
                .map(|ed| ed.id.to_string())
                .collect()
        })
        .unwrap_or_default();

    for segment in &cpl.segment_list.segments {
        let sl = &segment.sequence_list;

        if sl.iab_sequences.is_empty() {
            continue;
        }

        let seg_loc = Location::new()
            .with_cpl(cpl.id)
            .with_path(format!("Segment/{}", segment.id));

        // §6.2: MainAudioSequence shall accompany IABSequence in the same segment.
        if sl.main_audio_sequences.is_empty() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Audio,
                    code(IabCode::MainAudioMissing),
                    format!(
                        "Segment {}: IABSequence present but no MainAudioSequence found; \
                         ST 2067-201 §6.2 requires a MainAudioSequence alongside IABSequence",
                        segment.id,
                    ),
                )
                .with_location(seg_loc.clone()),
            );
        }

        for iab_seq in &sl.iab_sequences {
            let seq_loc = Location::new()
                .with_cpl(cpl.id)
                .with_path(format!("Segment/{}/IABSequence/{}", segment.id, iab_seq.id));

            // §6.2: IABSequence shall contain at least one Resource.
            if iab_seq.resource_list.resources.is_empty() {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Audio,
                        code(IabCode::IABSequenceNoResources),
                        format!(
                            "IABSequence {}: shall contain at least one Resource \
                             (ST 2067-201 §6.2)",
                            iab_seq.id,
                        ),
                    )
                    .with_location(seq_loc.clone()),
                );
            }

            // §6.2: Each Resource.SourceEncoding shall reference an IABEssenceDescriptor.
            for resource in &iab_seq.resource_list.resources {
                if let Some(ref se) = resource.source_encoding {
                    let se_str = se.to_string();
                    if !iab_ed_ids.contains(&se_str) {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                Category::Audio,
                                code(IabCode::IABSequenceSourceEncodingInvalid),
                                format!(
                                    "IABSequence {}: Resource {} SourceEncoding {se_str} \
                                     does not reference an IABEssenceDescriptor \
                                     (ST 2067-201 §6.2)",
                                    iab_seq.id, resource.id,
                                ),
                            )
                            .with_location(seq_loc.clone()),
                        );
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod namespace_tests {
    use super::*;

    /// Pin the firsthand-confirmed IAB namespace contract:
    /// - The `/ns/2067-201/2019` URI is the canonical IAB namespace declared by
    ///   2019, 2021, and 2026 IAB documents alike (firsthand-verified from the
    ///   2021 PDF line 642 and the 2026 publication's inline HTML schema).
    /// - The `/schemas/2067-201/2019` URI is the legacy form.
    /// - There is no 2021 or 2026 URI — both publications reuse 2019.
    #[test]
    fn iab_namespace_uris_match_pdf_evidence() {
        assert_eq!(URI_2019, "http://www.smpte-ra.org/ns/2067-201/2019");
        assert_eq!(
            URI_2019_SCHEMAS,
            "http://www.smpte-ra.org/schemas/2067-201/2019"
        );
    }
}

#[cfg(test)]
mod plugin_2026_tests {
    use super::*;
    use crate::cpl::parse_cpl;
    use crate::diagnostics::Severity;

    /// Minimal CPL embedding an IABEssenceDescriptor with the given
    /// `<SubDescriptors>` body. Picks a real-shape CPL skeleton (so
    /// `parse_cpl` succeeds) and substitutes the SubDescriptors body
    /// in. The CPL namespace doesn't matter for the Annex E check —
    /// the test exercises the per-validator dispatch directly.
    fn cpl_xml_with_iab_subdescriptors(subdescriptors_body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2016">
    <Id>urn:uuid:00000000-0000-0000-0000-000000000cc1</Id>
    <IssueDate>2026-06-13T12:00:00Z</IssueDate>
    <ContentTitle>T</ContentTitle>
    <EditRate>24 1</EditRate>
    <EssenceDescriptorList>
        <EssenceDescriptor>
            <Id>urn:uuid:00000000-0000-0000-0000-000000000aaa</Id>
            <IABEssenceDescriptor>
                <SubDescriptors>
                    {subdescriptors_body}
                </SubDescriptors>
            </IABEssenceDescriptor>
        </EssenceDescriptor>
    </EssenceDescriptorList>
    <SegmentList/>
</CompositionPlaylist>"#,
        )
    }

    /// 2026 fires the Annex E warning when the IAB descriptor has zero
    /// `IABChannelSubDescriptor` entries.
    #[test]
    fn plugin_2026_warns_when_iab_channel_subdescriptors_absent() {
        let xml = cpl_xml_with_iab_subdescriptors("<IABSoundfieldLabelSubDescriptor/>");
        let cpl = parse_cpl(&xml).expect("CPL should parse");
        let issues = AppIabPlugin2026.validate_cpl(&cpl);
        let hit = issues.iter().find(|i| {
            i.code == "ST2067-201:2026:Annex-E/IabChannelSubDescriptorRecommended"
        });
        assert!(hit.is_some(), "expected Annex E warning, got: {issues:#?}");
        assert_eq!(hit.unwrap().severity, Severity::Warning);
    }

    /// 2026 does NOT fire the warning when at least one
    /// `IABChannelSubDescriptor` is present.
    #[test]
    fn plugin_2026_silent_when_iab_channel_subdescriptors_present() {
        let xml = cpl_xml_with_iab_subdescriptors(
            "<IABSoundfieldLabelSubDescriptor/>\
             <IABChannelSubDescriptor><IABBedMetaID>1</IABBedMetaID><IABChannelID>1</IABChannelID></IABChannelSubDescriptor>",
        );
        let cpl = parse_cpl(&xml).expect("CPL should parse");
        let issues = AppIabPlugin2026.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("IabChannelSubDescriptorRecommended")),
            "should NOT fire Annex E warning when ≥1 entry present, got: {issues:#?}"
        );
    }

    /// **The 2021 plugin must NOT fire the Annex E warning even when
    /// `IABChannelSubDescriptor` entries are absent** — the
    /// recommendation only exists in the 2026 prose.
    #[test]
    fn plugin_2021_silent_on_annex_e_recommendation() {
        let xml = cpl_xml_with_iab_subdescriptors("<IABSoundfieldLabelSubDescriptor/>");
        let cpl = parse_cpl(&xml).expect("CPL should parse");
        let issues = AppIabPlugin2021.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("IabChannelSubDescriptorRecommended")),
            "AppIabPlugin2021 must not emit the 2026-only Annex E code"
        );
    }
}
