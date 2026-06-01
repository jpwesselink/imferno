//! ST 2067-3 normative constraint validation.
//!
//! Implements all "shall" rules from SMPTE ST 2067-3 (all editions: 2013, 2016, 2020).
//! Issue codes follow the canonical shape: `ST2067-3:{year}:{clause}/{cause}`
//!
//! Covered clauses (per normative test matrices):
//! - §5.5.1.2  ContentKind vocabulary under SMPTE scope
//! - §6.1.9    ContentVersion/Id uniqueness within ContentVersionList
//! - §6.4.2    SourceEncoding → EssenceDescriptor cross-reference
//! - §6.11     ContentVersionList / ContentVersion constraints
//! - §6.12     Locale language tags (RFC 5646)
//! - §6.14.1.2 Marker label vocabulary (2013 and 2016 scopes)
//! - §7.3      TrackId uniqueness within segment; sequence duration integer edit-units
//! - §7.4      Marker offset bounds
//!
//! Note: §7.2.2 (segment duration homogeneity) is enforced by the st2067-21
//! core structure validator which handles edit-rate normalisation across all
//! track types and runs on every CPL regardless of application profile.

use std::collections::HashSet;

use crate::diagnostics::{Category, Location, Severity, ValidationIssue};

use crate::cpl::codes::St2067_3Code;
use crate::cpl::{
    CompositionPlaylist, ContentKind, CplNamespace, MarkerLabel, CONTENT_KIND_DEFAULT_SCOPE,
};

// ─── Code dispatch helper ─────────────────────────────────────────────────────

fn code_fn(ns: &CplNamespace) -> fn(St2067_3Code) -> &'static str {
    use crate::cpl::codes::{St2067_3_2013, St2067_3_2016, St2067_3_2020};
    match ns {
        CplNamespace::Dci429_7 | CplNamespace::Smpte2067_3_2013 => St2067_3_2013::for_code,
        CplNamespace::Smpte2067_3_2016 => St2067_3_2016::for_code,
        CplNamespace::Smpte2067_3_2020 | CplNamespace::Unknown(_) => St2067_3_2020::for_code,
    }
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Validate a `CompositionPlaylist` against the normative constraints of ST 2067-3.
///
/// Runs normative rules across §5.5.1.2, §6.4.2, §6.11, §6.12, §7.3, §7.4.
/// The spec year in issue codes is derived from the CPL's detected namespace.
pub fn validate_cpl(cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let code = code_fn(&cpl.namespace);

    validate_content_kind_vocabulary(cpl, code, &mut issues);
    validate_source_encoding_refs(cpl, code, &mut issues);
    validate_content_versions(cpl, code, &mut issues);
    validate_content_version_uniqueness(cpl, code, &mut issues);
    validate_locale_language_tags(cpl, code, &mut issues);
    validate_track_id_uniqueness(cpl, code, &mut issues);
    validate_sequence_duration_integer_edit_units(cpl, code, &mut issues);
    validate_marker_offsets(cpl, code, &mut issues);
    validate_marker_labels(cpl, code, &mut issues);

    issues
}

// ─── §5.5.1.2 ContentKind vocabulary ────────────────────────────────────────

/// ST 2067-3 §5.5.1.2: Under the SMPTE scope, ContentKind shall use a valid vocabulary value.
fn validate_content_kind_vocabulary(
    cpl: &CompositionPlaylist,
    code: fn(St2067_3Code) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    if cpl.content_kind.effective_scope() != CONTENT_KIND_DEFAULT_SCOPE {
        return;
    }
    if let ContentKind::Other(ref s) = cpl.content_kind.kind {
        issues.push(
            ValidationIssue::new(
                Severity::Warning,
                Category::Metadata,
                code(St2067_3Code::ContentKindUnknown),
                format!(
                    "ContentKind '{s}' is not a recognized value under the SMPTE scope \
                     (expected: feature, trailer, test, promo, teaser, rating-bump, \
                     advertisement, episode, short, commercial, psa)",
                ),
            )
            .with_location(Location::new().with_cpl(cpl.id)),
        );
    }
}

// ─── §6.4.2 SourceEncoding cross-reference ───────────────────────────────────

/// ST 2067-3 §6.4.2: `SourceEncoding` shall reference an `EssenceDescriptor/Id` in the CPL.
fn validate_source_encoding_refs(
    cpl: &CompositionPlaylist,
    code: fn(St2067_3Code) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let cpl_id = cpl.id;
    let loc = Location::new().with_cpl(cpl_id);

    let ed_ids: HashSet<String> = cpl
        .essence_descriptor_list
        .as_ref()
        .map(|edl| {
            edl.essence_descriptors
                .iter()
                .map(|ed| ed.id.to_string())
                .collect()
        })
        .unwrap_or_default();

    let has_edl = cpl.essence_descriptor_list.is_some();

    for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
        let sl = &segment.sequence_list;

        let all_seqs = sl.all_sequences_typed();

        for (seq, track_type) in all_seqs {
            for (res_idx, resource) in seq.resource_list().resources.iter().enumerate() {
                if let Some(ref se) = resource.source_encoding {
                    let se_str = se.to_string();
                    let res_loc = Location::new()
                        .with_cpl(cpl_id)
                        .with_segment(seg_idx)
                        .with_resource(res_idx);

                    if !has_edl {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                Category::Reference,
                                code(St2067_3Code::SourceEncodingNoEssenceDescriptorList),
                                format!(
                                    "{track_type} resource {res_idx}: SourceEncoding '{se_str}' \
                                     present but EssenceDescriptorList is absent",
                                ),
                            )
                            .with_location(res_loc),
                        );
                    } else if !ed_ids.contains(&se_str) {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                Category::Reference,
                                code(St2067_3Code::SourceEncodingUnresolved),
                                format!(
                                    "{track_type} resource {res_idx}: SourceEncoding '{se_str}' \
                                     does not match any EssenceDescriptor Id",
                                ),
                            )
                            .with_location(res_loc),
                        );
                    }
                }
            }
        }
    }

    // Warn if EDL is present but empty
    if let Some(ref edl) = cpl.essence_descriptor_list {
        if edl.essence_descriptors.is_empty() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Structure,
                    code(St2067_3Code::EssenceDescriptorListEmpty),
                    "EssenceDescriptorList is present but contains no EssenceDescriptors",
                )
                .with_location(loc),
            );
        }
    }
}

// ─── §6.11 ContentVersionList ─────────────────────────────────────────────────

/// ST 2067-3 §6.11: ContentVersionList, ContentVersion/Id and ContentVersion/LabelText constraints.
fn validate_content_versions(
    cpl: &CompositionPlaylist,
    code: fn(St2067_3Code) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let loc = Location::new().with_cpl(cpl.id);

    let cvl = match &cpl.content_version_list {
        Some(cvl) => cvl,
        None => return,
    };

    if cvl.content_versions.is_empty() {
        issues.push(
            ValidationIssue::new(
                Severity::Error,
                Category::Structure,
                code(St2067_3Code::ContentVersionListEmpty),
                "ContentVersionList shall contain at least one ContentVersion",
            )
            .with_location(loc.clone()),
        );
        return;
    }

    for (i, cv) in cvl.content_versions.iter().enumerate() {
        if cv.id.trim().is_empty() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Metadata,
                    code(St2067_3Code::ContentVersionIdInvalid),
                    format!("ContentVersion[{i}] has an empty Id (shall be a URI)"),
                )
                .with_location(loc.clone()),
            );
        }
        if cv.label_text.is_none() {
            issues.push(
                ValidationIssue::new(
                    Severity::Warning,
                    Category::Metadata,
                    code(St2067_3Code::ContentVersionLabelTextMissing),
                    format!("ContentVersion[{i}] (Id: '{}') is missing LabelText", cv.id,),
                )
                .with_location(loc.clone()),
            );
        }
    }
}

// ─── §6.12 Locale language tags ──────────────────────────────────────────────

/// ST 2067-3 §6.12: Locale language tags shall conform to RFC 5646.
fn validate_locale_language_tags(
    cpl: &CompositionPlaylist,
    code: fn(St2067_3Code) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let loc = Location::new().with_cpl(cpl.id);

    let ll = match &cpl.locale_list {
        Some(ll) => ll,
        None => return,
    };

    for (i, locale) in ll.locales.iter().enumerate() {
        if let Some(ref lang_list) = locale.language_list {
            for tag in &lang_list.languages {
                let s = tag.as_str();
                if s.is_empty() || !s.chars().next().unwrap_or(' ').is_ascii_alphabetic() {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Warning,
                            Category::Metadata,
                            code(St2067_3Code::LocaleLanguageTagInvalid),
                            format!(
                                "Locale[{i}]: language tag '{s}' does not conform to RFC 5646 \
                                 (must start with an ASCII letter)",
                            ),
                        )
                        .with_location(loc.clone()),
                    );
                }
            }
        }
    }
}

// ─── §7.3 TrackId uniqueness ──────────────────────────────────────────────────

/// ST 2067-3 §7.3: TrackId values shall be unique within each segment.
fn validate_track_id_uniqueness(
    cpl: &CompositionPlaylist,
    code: fn(St2067_3Code) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let cpl_id = cpl.id;

    for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
        let sl = &segment.sequence_list;
        let mut seen: HashSet<String> = HashSet::new();

        // Include marker sequences for track ID uniqueness check
        let mut all_seqs = sl.all_sequences_typed();
        for s in &sl.marker_sequences {
            all_seqs.push((s, "MarkerSequence"));
        }

        for (seq, track_type) in all_seqs {
            let id = seq.track_id().to_string();
            if !seen.insert(id.clone()) {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Structure,
                        code(St2067_3Code::TrackIdNotUnique),
                        format!(
                            "Duplicate TrackId '{id}' in segment {seg} ({track_type})",
                            seg = seg_idx + 1,
                        ),
                    )
                    .with_location(Location::new().with_cpl(cpl_id).with_segment(seg_idx)),
                );
            }
        }
    }
}

// ─── §7.4 Marker offset bounds ───────────────────────────────────────────────

/// ST 2067-3 §7.4: Marker offset shall be within the temporal bounds of its resource.
fn validate_marker_offsets(
    cpl: &CompositionPlaylist,
    code: fn(St2067_3Code) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let cpl_id = cpl.id;

    for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
        for seq in &segment.sequence_list.marker_sequences {
            for (res_idx, resource) in seq.resource_list.resources.iter().enumerate() {
                let effective_duration = resource
                    .source_duration
                    .unwrap_or(resource.intrinsic_duration - resource.entry_point.unwrap_or(0));

                for marker in &resource.markers {
                    if marker.offset >= effective_duration {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                Category::Timing,
                                code(St2067_3Code::MarkerOffsetOutOfRange),
                                format!(
                                    "Marker '{}' offset {} exceeds resource effective duration {}",
                                    marker.label, marker.offset, effective_duration,
                                ),
                            )
                            .with_location(
                                Location::new()
                                    .with_cpl(cpl_id)
                                    .with_segment(seg_idx)
                                    .with_resource(res_idx),
                            ),
                        );
                    }
                }
            }
        }
    }
}

// ─── §7.4 Marker label vocabulary ────────────────────────────────────────────

const SMPTE_MARKER_SCOPE: &str = "http://www.smpte-ra.org/schemas/2067-3/2013#standard-markers";

const SMPTE_MARKER_SCOPE_2016: &str =
    "http://www.smpte-ra.org/schemas/2067-3/2016#standard-markers";

/// ST 2067-3 §7.4: SMPTE marker labels shall use the normative vocabulary.
fn validate_marker_labels(
    cpl: &CompositionPlaylist,
    code: fn(St2067_3Code) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let cpl_id = cpl.id;

    for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
        for seq in &segment.sequence_list.marker_sequences {
            for (res_idx, resource) in seq.resource_list.resources.iter().enumerate() {
                for marker in &resource.markers {
                    let scope = marker.label.effective_scope();
                    if scope == SMPTE_MARKER_SCOPE {
                        if let MarkerLabel::Other(ref s) = marker.label.label {
                            issues.push(
                                ValidationIssue::new(
                                    Severity::Warning,
                                    Category::Metadata,
                                    code(St2067_3Code::MarkerLabelUnknown),
                                    format!(
                                        "Marker label '{s}' is not a recognized SMPTE standard marker \
                                         (expected: FFOC, LFOC, FFAC, LFAC, FFMC, LFMC, FFHC, LFHC)",
                                    ),
                                )
                                .with_location(
                                    Location::new()
                                        .with_cpl(cpl_id)
                                        .with_segment(seg_idx)
                                        .with_resource(res_idx),
                                ),
                            );
                        }
                    } else if scope == SMPTE_MARKER_SCOPE_2016 {
                        if let MarkerLabel::Other(ref s) = marker.label.label {
                            issues.push(
                                ValidationIssue::new(
                                    Severity::Warning,
                                    Category::Metadata,
                                    code(St2067_3Code::MarkerLabelUnknown),
                                    format!(
                                        "Marker label '{s}' is not a recognized SMPTE 2016 standard \
                                         marker (expected: FFDC, LFDC)",
                                    ),
                                )
                                .with_location(
                                    Location::new()
                                        .with_cpl(cpl_id)
                                        .with_segment(seg_idx)
                                        .with_resource(res_idx),
                                ),
                            );
                        }
                    }
                }
            }
        }
    }
}

// ─── §6.1.9 ContentVersion Id uniqueness ──────────────────────────────────────

/// ST 2067-3 §6.1.9: No two ContentVersion elements shall have identical Id values.
fn validate_content_version_uniqueness(
    cpl: &CompositionPlaylist,
    code: fn(St2067_3Code) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let loc = Location::new().with_cpl(cpl.id);
    let cvl = match &cpl.content_version_list {
        Some(cvl) => cvl,
        None => return,
    };
    let mut seen: HashSet<String> = HashSet::new();
    for (i, cv) in cvl.content_versions.iter().enumerate() {
        let id = cv.id.trim().to_string();
        if id.is_empty() {
            continue; // already reported by validate_content_versions
        }
        if !seen.insert(id.clone()) {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Structure,
                    code(St2067_3Code::ContentVersionIdDuplicate),
                    format!(
                        "ContentVersion[{i}] Id '{id}' is duplicated; \
                         no two ContentVersion elements shall have identical Id values",
                    ),
                )
                .with_location(loc.clone()),
            );
        }
    }
}

// ─── §7.3 Sequence duration integer edit units ────────────────────────────────

/// ST 2067-3 §7.3: The duration of a Sequence shall be an integer number of Composition Edit Units.
///
/// When a resource carries a resource-level `EditRate` different from the Composition `EditRate`,
/// the resource's contribution to the sequence duration (in Composition Edit Units) is:
///
///   `source_duration × cpl_er.numerator × res_er.denominator`
///   ─────────────────────────────────────────────────────────
///   `res_er.numerator × cpl_er.denominator`
///
/// The sum across all resources must reduce to an integer (denominator = 1 after GCD reduction).
// `clippy::manual_checked_ops` would suggest `checked_div`, but that returns Option
// and changes the structure of the GCD-normalize step here; the explicit `if g > 0`
// guard is clearer for non-Option arithmetic.
#[allow(clippy::manual_checked_ops)]
fn validate_sequence_duration_integer_edit_units(
    cpl: &CompositionPlaylist,
    code: fn(St2067_3Code) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let cpl_er = match &cpl.edit_rate {
        Some(er) => *er,
        None => return,
    };

    let cpl_id = cpl.id;

    for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
        let sl = &segment.sequence_list;

        let all_seqs = sl.all_sequences_typed();

        for (seq, track_type) in all_seqs {
            let mut sum_num: u64 = 0;
            let mut sum_den: u64 = 1;
            let mut has_cross_rate = false;

            for resource in &seq.resource_list().resources {
                let res_er = resource.edit_rate.unwrap_or(cpl_er);
                if res_er.numerator != cpl_er.numerator || res_er.denominator != cpl_er.denominator
                {
                    has_cross_rate = true;
                }

                let src_dur = resource.source_duration.unwrap_or_else(|| {
                    resource
                        .intrinsic_duration
                        .saturating_sub(resource.entry_point.unwrap_or(0))
                });

                let contrib_num = src_dur
                    .saturating_mul(cpl_er.numerator as u64)
                    .saturating_mul(res_er.denominator as u64);
                let contrib_den =
                    (res_er.numerator as u64).saturating_mul(cpl_er.denominator as u64);

                let l = lcm_u64(sum_den, contrib_den);
                sum_num = sum_num
                    .saturating_mul(l / sum_den)
                    .saturating_add(contrib_num.saturating_mul(l / contrib_den));
                sum_den = l;

                let g = gcd_u64(sum_num, sum_den);
                if g > 0 {
                    sum_num /= g;
                    sum_den /= g;
                }
            }

            if has_cross_rate && sum_den != 1 {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Timing,
                        code(St2067_3Code::SegmentDurationIntegerEditUnits),
                        format!(
                            "Segment {seg} {track_type} sequence duration is not an integer \
                             number of Composition Edit Units ({sum_num}/{sum_den})",
                            seg = seg_idx + 1,
                        ),
                    )
                    .with_location(Location::new().with_cpl(cpl_id).with_segment(seg_idx)),
                );
            }
        }
    }
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

#[allow(clippy::manual_checked_ops)]
fn lcm_u64(a: u64, b: u64) -> u64 {
    let g = gcd_u64(a, b);
    if g == 0 {
        0
    } else {
        a / g * b
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assetmap::ImfUuid;
    use crate::cpl::{
        ContentKind, ContentKindElement, ContentVersion, ContentVersionList, EditRate,
        LanguageString, MainImageSequence, Resource, ResourceList, Segment, SegmentList,
        SequenceList,
    };

    fn dummy_uuid() -> ImfUuid {
        ImfUuid::parse("urn:uuid:00000000-0000-0000-0000-000000000001").unwrap()
    }

    fn empty_sequence_list() -> SequenceList {
        SequenceList {
            marker_sequences: vec![],
            main_image_sequences: vec![],
            main_audio_sequences: vec![],
            subtitles_sequences: vec![],
            hearing_impaired_captions_sequences: vec![],
            forced_narrative_sequences: vec![],
            iab_sequences: vec![],
            isxd_sequences: vec![],
        }
    }

    fn minimal_cpl() -> CompositionPlaylist {
        CompositionPlaylist {
            id: dummy_uuid(),
            namespace: CplNamespace::Smpte2067_3_2016,
            annotation: None,
            issue_date: String::new(),
            issuer: None,
            creator: None,
            content_originator: None,
            content_title: LanguageString::default(),
            content_kind: ContentKindElement {
                kind: ContentKind::Other("unknown".to_string()),
                scope: None,
            },
            content_version_list: None,
            essence_descriptor_list: None,
            edit_rate: Some(EditRate::new(24, 1)),
            total_running_time: None,
            locale_list: None,
            extension_properties: None,
            composition_timecode: None,
            has_signer: false,
            has_signature: false,
            source_xml: None,
            segment_list: SegmentList { segments: vec![] },
        }
    }

    fn make_resource(
        intrinsic_duration: u64,
        source_duration: Option<u64>,
        edit_rate: Option<EditRate>,
    ) -> Resource {
        Resource {
            id: dummy_uuid(),
            annotation: None,
            edit_rate,
            intrinsic_duration,
            entry_point: None,
            source_duration,
            source_encoding: None,
            track_file_id: None,
            repeat_count: None,
            key_id: None,
            hash: None,
            markers: vec![],
        }
    }

    fn image_segment_with_resources(resources: Vec<Resource>) -> Segment {
        let mut sl = empty_sequence_list();
        sl.main_image_sequences = vec![MainImageSequence {
            id: dummy_uuid(),
            track_id: dummy_uuid(),
            resource_list: ResourceList { resources },
        }];
        Segment {
            id: dummy_uuid(),
            sequence_list: sl,
        }
    }

    // ── ContentVersionIdDuplicate ────────────────────────────────────────────

    #[test]
    fn content_version_duplicate_id_emits_error() {
        let mut cpl = minimal_cpl();
        cpl.content_version_list = Some(ContentVersionList {
            content_versions: vec![
                ContentVersion {
                    id: "urn:uuid:aaa".to_string(),
                    label_text: None,
                },
                ContentVersion {
                    id: "urn:uuid:aaa".to_string(),
                    label_text: None,
                },
            ],
        });
        let issues = validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("ContentVersionIdDuplicate")),
            "expected ContentVersionIdDuplicate, got: {issues:?}",
        );
    }

    #[test]
    fn content_version_distinct_ids_ok() {
        let mut cpl = minimal_cpl();
        cpl.content_version_list = Some(ContentVersionList {
            content_versions: vec![
                ContentVersion {
                    id: "urn:uuid:aaa".to_string(),
                    label_text: None,
                },
                ContentVersion {
                    id: "urn:uuid:bbb".to_string(),
                    label_text: None,
                },
            ],
        });
        let issues = validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("ContentVersionIdDuplicate")),
            "unexpected ContentVersionIdDuplicate",
        );
    }

    // ── SegmentDurationIntegerEditUnits ──────────────────────────────────────

    #[test]
    fn sequence_non_integer_edit_units_emits_error() {
        // Composition ER = 24/1, resource ER = 48000/1.
        // source_duration = 47999 → 47999 × 24 / 48000 = non-integer.
        let mut cpl = minimal_cpl();
        cpl.segment_list = SegmentList {
            segments: vec![image_segment_with_resources(vec![make_resource(
                48000,
                Some(47999),
                Some(EditRate::new(48000, 1)),
            )])],
        };
        let issues = validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("SegmentDurationIntegerEditUnits")),
            "expected SegmentDurationIntegerEditUnits, got: {issues:?}",
        );
    }

    #[test]
    fn sequence_integer_edit_units_ok() {
        // Composition ER = 24/1, resource ER = 48000/1.
        // source_duration = 48000 → exactly 1 composition frame.
        let mut cpl = minimal_cpl();
        cpl.segment_list = SegmentList {
            segments: vec![image_segment_with_resources(vec![make_resource(
                48000,
                Some(48000),
                Some(EditRate::new(48000, 1)),
            )])],
        };
        let issues = validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("SegmentDurationIntegerEditUnits")),
            "unexpected SegmentDurationIntegerEditUnits",
        );
    }

    // ── MarkerLabelUnknown 2016 scope ────────────────────────────────────────

    #[test]
    fn marker_label_unknown_2016_scope_emits_warning() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2016"
                     xmlns:cc="http://www.smpte-ra.org/ns/2067-2/2020">
  <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
  <IssueDate>2024-01-01T00:00:00Z</IssueDate>
  <ContentTitle>Test</ContentTitle>
  <EditRate>24 1</EditRate>
  <ContentKind>feature</ContentKind>
  <SegmentList>
    <Segment>
      <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
      <SequenceList>
        <MarkerSequence>
          <Id>urn:uuid:00000000-0000-0000-0000-000000000003</Id>
          <TrackId>urn:uuid:00000000-0000-0000-0000-000000000004</TrackId>
          <ResourceList>
            <Resource>
              <Id>urn:uuid:00000000-0000-0000-0000-000000000005</Id>
              <IntrinsicDuration>100</IntrinsicDuration>
              <Marker>
                <Label scope="http://www.smpte-ra.org/schemas/2067-3/2016#standard-markers">UNKNOWN_LABEL</Label>
                <Offset>0</Offset>
              </Marker>
            </Resource>
          </ResourceList>
        </MarkerSequence>
      </SequenceList>
    </Segment>
  </SegmentList>
</CompositionPlaylist>"#;
        let cpl = crate::cpl::parse_cpl(xml).expect("parse failed");
        let issues = validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("MarkerLabelUnknown") && i.message.contains("FFDC, LFDC")),
            "expected 2016 MarkerLabelUnknown, got: {issues:?}",
        );
    }
}
