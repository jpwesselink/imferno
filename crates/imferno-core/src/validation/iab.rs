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

// ── Namespace URIs ────────────────────────────────────────────────────────────

pub const URI_2019: &str = "http://www.smpte-ra.org/ns/2067-201/2019";
pub const URI_2019_SCHEMAS: &str = "http://www.smpte-ra.org/schemas/2067-201/2019";
pub const URI_2021: &str = "http://www.smpte-ra.org/ns/2067-201/2021";
pub const URI_2021_SCHEMAS: &str = "http://www.smpte-ra.org/schemas/2067-201/2021";

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
            .with_cpl(cpl.id.to_string())
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
        let sub_loc = Location::new()
            .with_cpl(cpl.id.to_string())
            .with_path(format!(
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
            .with_cpl(cpl.id.to_string())
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
                .with_cpl(cpl.id.to_string())
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
