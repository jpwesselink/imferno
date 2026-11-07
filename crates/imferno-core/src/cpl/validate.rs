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
//! - §6.13     Marker offset bounds
//!
//! Note: §7.2.2 (segment duration homogeneity) is enforced by the st2067-21
//! core structure validator which handles edit-rate normalisation across all
//! track types and runs on every CPL regardless of application profile.

use std::collections::HashSet;

use crate::diagnostics::{Location, ValidationIssue};

use crate::catalogue::st2067_3::{self, St2067_3Edition};
use crate::cpl::{CompositionPlaylist, ContentKind, CplNamespace, CONTENT_KIND_DEFAULT_SCOPE};

// ─── Edition dispatch ──────────────────────────────────────────────────────────

/// The ST 2067-3 edition whose rules are applied to a CPL of namespace `ns`.
pub fn st2067_3_edition(ns: &CplNamespace) -> St2067_3Edition {
    match ns {
        CplNamespace::Dci429_7 | CplNamespace::Smpte2067_3_2013 => St2067_3Edition::V2013,
        // ST 2067-3:2020 reuses the 2016 namespace (canonical XSD is byte-identical
        // to 2016), so 2020-era CPLs land here too. Unknown namespaces default to
        // the latest real ruleset.
        CplNamespace::Smpte2067_3_2016 | CplNamespace::Unknown(_) => St2067_3Edition::V2016,
    }
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Validate a `CompositionPlaylist` against the normative constraints of ST 2067-3.
///
/// Runs normative rules across §5.5.1.2, §6.4.2, §6.11, §6.12, §7.3, §7.4.
/// The spec year in issue codes is derived from the CPL's detected namespace.
pub fn validate_cpl(cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let spec_ed = st2067_3_edition(&cpl.namespace);

    validate_content_kind_vocabulary(cpl, spec_ed, &mut issues);
    validate_source_encoding_refs(cpl, spec_ed, &mut issues);
    validate_content_versions(cpl, spec_ed, &mut issues);
    validate_content_version_uniqueness(cpl, spec_ed, &mut issues);
    validate_locale_language_tags(cpl, spec_ed, &mut issues);
    validate_track_id_uniqueness(cpl, spec_ed, &mut issues);
    validate_sequence_duration_integer_edit_units(cpl, spec_ed, &mut issues);
    validate_marker_offsets(cpl, spec_ed, &mut issues);
    validate_marker_labels(cpl, spec_ed, &mut issues);

    issues
}

// ─── §5.5.1.2 ContentKind vocabulary ────────────────────────────────────────

/// ST 2067-3:2013 / 2016 §6.1.8 Table 3: ContentKind values under the
/// default scope (identical in both editions).
const CONTENT_KINDS_TABLE_3: &[&str] = &[
    "advertisement",
    "feature",
    "psa",
    "rating",
    "short",
    "teaser",
    "test",
    "trailer",
    "transitional",
    "episode",
    "highlights",
    "event",
];

/// ST 2067-3:2016 §6.1.8 Table 4: ContentKind values under the
/// `2016#content-kind` scope. This scope does not exist in the 2013 edition.
const CONTENT_KINDS_TABLE_4_2016: &[&str] = &["supplemental", "documentary"];

const CONTENT_KIND_SCOPE_2016: &str = "http://www.smpte-ra.org/schemas/2067-3/2016#content-kind";

/// The CompositionPlaylist's top-level `ContentKind` element in the source XML.
enum RawContentKind {
    /// Element present; its text exactly as written (xs:string, untrimmed).
    Present(String),
    /// The root element closed without a `ContentKind` child.
    Absent,
    /// The XML could not be scanned.
    Unreadable,
}

fn raw_content_kind(xml: &str) -> RawContentKind {
    raw_content_kind_text(xml).unwrap_or(RawContentKind::Unreadable)
}

fn raw_content_kind_text(xml: &str) -> Option<RawContentKind> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut depth = 0usize;
    let mut in_content_kind = false;
    let mut text = String::new();
    loop {
        match reader.read_event().ok()? {
            Event::Start(e) => {
                depth += 1;
                if depth == 2 && e.local_name().as_ref() == "ContentKind" {
                    in_content_kind = true;
                }
            }
            Event::End(_) => {
                if in_content_kind && depth == 2 {
                    return Some(RawContentKind::Present(text));
                }
                if depth == 1 {
                    return Some(RawContentKind::Absent);
                }
                depth = depth.saturating_sub(1);
            }
            Event::Empty(e) => {
                if depth == 1 && e.local_name().as_ref() == "ContentKind" {
                    return Some(RawContentKind::Present(String::new()));
                }
            }
            Event::Text(t) if in_content_kind => {
                text.push_str(&quick_xml::escape::unescape(&t).ok()?);
            }
            Event::GeneralRef(r) if in_content_kind => {
                // Predefined and character references; anything else
                // makes the raw value unrecoverable here.
                let entity = format!("&{};", r.into_inner());
                text.push_str(&quick_xml::escape::unescape(&entity).ok()?);
            }
            Event::CData(c) if in_content_kind => {
                text.push_str(&c);
            }
            Event::Eof => return None,
            _ => {}
        }
    }
}

/// ST 2067-3 §6.1.8: under the default scope ContentKind shall match a value
/// of Table 3; under the 2016 scope (2016 edition only) a value of Table 4.
/// Any other scope is outside the specification.
fn validate_content_kind_vocabulary(
    cpl: &CompositionPlaylist,
    spec_ed: St2067_3Edition,
    issues: &mut Vec<ValidationIssue>,
) {
    let scope = cpl.content_kind.effective_scope();
    let is_2013_edition = matches!(
        cpl.namespace,
        CplNamespace::Dci429_7 | CplNamespace::Smpte2067_3_2013
    );
    let (allowed, table) = if scope == CONTENT_KIND_DEFAULT_SCOPE {
        (CONTENT_KINDS_TABLE_3, "Table 3")
    } else if scope == CONTENT_KIND_SCOPE_2016 && !is_2013_edition {
        (CONTENT_KINDS_TABLE_4_2016, "Table 4")
    } else {
        return;
    };

    use RawContentKind::{Absent, Present, Unreadable};
    // "shall match" is an exact, case-sensitive comparison. `ContentKind::parse`
    // folds case variants and synonyms ("Feature", "PSA", ...) into named
    // variants, so compare the raw element text from the source XML when
    // it is available. ContentKindType extends xs:string, so leading and
    // trailing whitespace is part of the value and is not trimmed.
    let raw = cpl
        .source_xml
        .as_deref()
        .map_or(RawContentKind::Unreadable, raw_content_kind);
    let value = match raw {
        Present(raw) => raw,
        // ContentKind is optional (minOccurs="0" in the 2013 and 2016
        // schemas): nothing to check.
        Absent => return,
        // No usable source XML (e.g. a CPL built in memory or deserialised
        // from JSON): best effort from the parsed variant. Folded
        // spellings cannot be recovered here, and the parser's default for
        // a missing element (`Other("unknown")`, no scope) is treated as
        // absent.
        Unreadable => match &cpl.content_kind.kind {
            ContentKind::Other(s) if s == "unknown" && cpl.content_kind.scope.is_none() => {
                return;
            }
            ContentKind::Feature => "feature".to_string(),
            ContentKind::Trailer => "trailer".to_string(),
            ContentKind::Test => "test".to_string(),
            ContentKind::Teaser => "teaser".to_string(),
            ContentKind::Advertisement => "advertisement".to_string(),
            ContentKind::Episode => "episode".to_string(),
            ContentKind::Short => "short".to_string(),
            ContentKind::PublicServiceAnnouncement => "psa".to_string(),
            ContentKind::Promo => "promo".to_string(),
            ContentKind::RatingBump => "rating-bump".to_string(),
            ContentKind::Commercial => "commercial".to_string(),
            ContentKind::Other(s) => s.clone(),
        },
    };

    if !allowed.contains(&value.as_str()) {
        issues.push(
            ValidationIssue::from_rule(
                &st2067_3::ContentKindUnknown,
                spec_ed,
                format!(
                    "ContentKind {:?} is not a recognized value under scope {scope} \
                     (expected one of {table}: {})",
                    value,
                    allowed.join(", "),
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
    spec_ed: St2067_3Edition,
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
                            ValidationIssue::from_rule(
                                &st2067_3::SourceEncodingNoEssenceDescriptorList,
                                spec_ed,
                                format!(
                                    "{track_type} resource {res_idx}: SourceEncoding '{se_str}' \
                                     present but EssenceDescriptorList is absent",
                                ),
                            )
                            .with_location(res_loc),
                        );
                    } else if !ed_ids.contains(&se_str) {
                        issues.push(
                            ValidationIssue::from_rule(
                                &st2067_3::SourceEncodingUnresolved,
                                spec_ed,
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
                ValidationIssue::from_rule(
                    &st2067_3::EssenceDescriptorListEmpty,
                    spec_ed,
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
    spec_ed: St2067_3Edition,
    issues: &mut Vec<ValidationIssue>,
) {
    let loc = Location::new().with_cpl(cpl.id);

    let cvl = match &cpl.content_version_list {
        Some(cvl) => cvl,
        None => return,
    };

    if cvl.content_versions.is_empty() {
        issues.push(
            ValidationIssue::from_rule(
                &st2067_3::ContentVersionListEmpty,
                spec_ed,
                "ContentVersionList shall contain at least one ContentVersion",
            )
            .with_location(loc.clone()),
        );
        return;
    }

    for (i, cv) in cvl.content_versions.iter().enumerate() {
        if cv.id.trim().is_empty() {
            issues.push(
                ValidationIssue::from_rule(
                    &st2067_3::ContentVersionIdInvalid,
                    spec_ed,
                    format!("ContentVersion[{i}] has an empty Id (shall be a URI)"),
                )
                .with_location(loc.clone()),
            );
        }
        if cv.label_text.is_none() {
            issues.push(
                ValidationIssue::from_rule(
                    &st2067_3::ContentVersionLabelTextMissing,
                    spec_ed,
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
    spec_ed: St2067_3Edition,
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
                        ValidationIssue::from_rule(
                            &st2067_3::LocaleLanguageTagInvalid,
                            spec_ed,
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
    spec_ed: St2067_3Edition,
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
                    ValidationIssue::from_rule(
                        &st2067_3::TrackIdNotUnique,
                        spec_ed,
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

// ─── §6.13 Marker offset bounds ──────────────────────────────────────────────

/// ST 2067-3 §6.13: a Marker Offset shall not exceed the IntrinsicDuration of its resource.
fn validate_marker_offsets(
    cpl: &CompositionPlaylist,
    spec_ed: St2067_3Edition,
    issues: &mut Vec<ValidationIssue>,
) {
    let cpl_id = cpl.id;

    for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
        for seq in &segment.sequence_list.marker_sequences {
            for (res_idx, resource) in seq.resource_list.resources.iter().enumerate() {
                // §6.13: IntrinsicDuration shall be equal or larger than the
                // largest marker Offset. EntryPoint/SourceDuration (the
                // playable region) do not constrain Offset.
                let intrinsic_duration = resource.intrinsic_duration;

                for marker in &resource.markers {
                    if marker.offset > intrinsic_duration {
                        issues.push(
                            ValidationIssue::from_rule(
                                &st2067_3::MarkerOffsetOutOfRange,
                                spec_ed,
                                format!(
                                    "Marker '{}' offset {} exceeds resource IntrinsicDuration {}",
                                    marker.label, marker.offset, intrinsic_duration,
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

// ─── Marker label vocabulary (§6.14.2 / §6.14.1.2) ───────────────────────────

const SMPTE_MARKER_SCOPE: &str = "http://www.smpte-ra.org/schemas/2067-3/2013#standard-markers";

const SMPTE_MARKER_SCOPE_2016: &str =
    "http://www.smpte-ra.org/schemas/2067-3/2016#standard-markers";

/// Labels for the default `2013#standard-markers` scope: ST 2067-3:2013
/// §6.14.2 Table 19, repeated unchanged in ST 2067-3:2016 §6.14.1.2 Table 21.
const STANDARD_MARKER_LABELS_2013: &[&str] = &[
    "FFBT", "FFCB", "FFCL", "FFDL", "FFEC", "FFHS", "FFMC", "FFOB", "FFOC", "FFOI", "FFSP", "FFTC",
    "FFTS", "FTXC", "FTXE", "FTXM", "LFBT", "LFCB", "LFCL", "LFDL", "LFEC", "LFHS", "LFMC", "LFOB",
    "LFOC", "LFOI", "LFSP", "LFTC", "LFTS", "LTXC", "LTXE", "LTXM", "FPCI", "FFCO", "LFCO", "FFOA",
    "LFOA",
];

/// Labels for the `2016#standard-markers` scope: ST 2067-3:2016 §6.14.1.2
/// Table 21. This scope does not exist in the 2013 edition.
const STANDARD_MARKER_LABELS_2016: &[&str] = &["FFDC", "LFDC"];

/// ST 2067-3:2013 §6.14.2 / ST 2067-3:2016 §6.14.1.2: labels under the SMPTE
/// standard-marker scopes shall use the values of Table 19 / Table 21.
/// Other scopes are outside the scope of the specification. The 2016 scope is
/// only checked on 2016 (and later/unknown) CPLs; for a 2013 CPL it is a
/// non-default scope and therefore out of scope.
fn validate_marker_labels(
    cpl: &CompositionPlaylist,
    spec_ed: St2067_3Edition,
    issues: &mut Vec<ValidationIssue>,
) {
    let cpl_id = cpl.id;
    let check_2016_scope = !matches!(
        cpl.namespace,
        CplNamespace::Dci429_7 | CplNamespace::Smpte2067_3_2013
    );

    for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
        for seq in &segment.sequence_list.marker_sequences {
            for (res_idx, resource) in seq.resource_list.resources.iter().enumerate() {
                for marker in &resource.markers {
                    let scope = marker.label.effective_scope();
                    let (allowed, which) = if scope == SMPTE_MARKER_SCOPE {
                        (STANDARD_MARKER_LABELS_2013, "SMPTE standard marker")
                    } else if scope == SMPTE_MARKER_SCOPE_2016 && check_2016_scope {
                        (STANDARD_MARKER_LABELS_2016, "SMPTE 2016 standard marker")
                    } else {
                        continue;
                    };
                    let label = marker.label.label.to_string();
                    if !allowed.contains(&label.as_str()) {
                        issues.push(
                            ValidationIssue::from_rule(
                                &st2067_3::MarkerLabelUnknown,
                                spec_ed,
                                format!(
                                    "Marker label '{label}' is not a recognized {which} \
                                     (expected one of: {})",
                                    allowed.join(", "),
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

// ─── §6.1.9 ContentVersion Id uniqueness ──────────────────────────────────────

/// ST 2067-3 §6.1.9: No two ContentVersion elements shall have identical Id values.
fn validate_content_version_uniqueness(
    cpl: &CompositionPlaylist,
    spec_ed: St2067_3Edition,
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
                ValidationIssue::from_rule(
                    &st2067_3::ContentVersionIdDuplicate,
                    spec_ed,
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
    spec_ed: St2067_3Edition,
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
                    ValidationIssue::from_rule(
                        &st2067_3::SegmentDurationIntegerEditUnits,
                        spec_ed,
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
            assetmap_asset_id: None,
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

    // ── MarkerLabelUnknown vocabulary (ST 2067-3:2013 Table 19,
    //    ST 2067-3:2016 Table 21) ────────────────────────────────────────────

    fn cpl_with_marker(ns: &str, scope: Option<&str>, label: &str) -> CompositionPlaylist {
        let scope_attr = scope
            .map(|s| format!(r#" scope="{s}""#))
            .unwrap_or_default();
        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist xmlns="{ns}">
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
                <Label{scope_attr}>{label}</Label>
                <Offset>0</Offset>
              </Marker>
            </Resource>
          </ResourceList>
        </MarkerSequence>
      </SequenceList>
    </Segment>
  </SegmentList>
</CompositionPlaylist>"#
        );
        crate::cpl::parse_cpl(&xml).expect("parse failed")
    }

    const NS_2013: &str = "http://www.smpte-ra.org/schemas/2067-3/2013";
    const NS_2016: &str = "http://www.smpte-ra.org/schemas/2067-3/2016";
    const SCOPE_2013: &str = "http://www.smpte-ra.org/schemas/2067-3/2013#standard-markers";
    const SCOPE_2016: &str = "http://www.smpte-ra.org/schemas/2067-3/2016#standard-markers";

    fn has_marker_label_unknown(cpl: &CompositionPlaylist) -> bool {
        validate_cpl(cpl)
            .iter()
            .any(|i| i.code.contains("MarkerLabelUnknown"))
    }

    #[test]
    fn marker_labels_from_spec_table_are_accepted() {
        for ns in [NS_2013, NS_2016] {
            for label in [
                "FFOC", "LFOC", "FFEC", "LFEC", "FFTC", "FPCI", "FTXM", "LTXM", "FFOA", "LFOA",
                "FFCO", "LFCO",
            ] {
                for scope in [None, Some(SCOPE_2013)] {
                    let cpl = cpl_with_marker(ns, scope, label);
                    assert!(
                        !has_marker_label_unknown(&cpl),
                        "{label} (ns {ns}, scope {scope:?}) is in the spec table and must not be flagged",
                    );
                }
            }
        }
    }

    #[test]
    fn marker_labels_not_in_spec_table_are_flagged() {
        for ns in [NS_2013, NS_2016] {
            for label in ["FFAC", "LFAC", "FFHC", "LFHC", "FFDC"] {
                let cpl = cpl_with_marker(ns, None, label);
                assert!(
                    has_marker_label_unknown(&cpl),
                    "{label} (ns {ns}) is not in the default-scope table and must be flagged",
                );
            }
        }
    }

    #[test]
    fn marker_labels_2016_scope_accepts_ffdc_lfdc() {
        for label in ["FFDC", "LFDC"] {
            let cpl = cpl_with_marker(NS_2016, Some(SCOPE_2016), label);
            assert!(
                !has_marker_label_unknown(&cpl),
                "{label} under the 2016 scope must not be flagged",
            );
        }
        let cpl = cpl_with_marker(NS_2016, Some(SCOPE_2016), "FFOC");
        assert!(
            has_marker_label_unknown(&cpl),
            "FFOC is not listed under the 2016 scope",
        );
    }

    /// ST 2067-3:2013 §6.14.2: a non-default scope is outside the scope of
    /// the 2013 edition, so the 2016 scope is not checked on 2013 CPLs.
    #[test]
    fn marker_labels_2016_scope_not_checked_on_2013_cpl() {
        let cpl = cpl_with_marker(NS_2013, Some(SCOPE_2016), "UNKNOWN_LABEL");
        assert!(!has_marker_label_unknown(&cpl));
    }

    // ── ContentKindUnknown vocabulary (ST 2067-3 §6.1.8 Tables 3 and 4) ─────

    /// Parse a CPL from XML so the raw ContentKind text is available, as in
    /// real packages.
    fn cpl_with_content_kind(
        ns: CplNamespace,
        scope: Option<&str>,
        value: &str,
    ) -> CompositionPlaylist {
        let ns_uri = match ns {
            CplNamespace::Smpte2067_3_2013 => "http://www.smpte-ra.org/schemas/2067-3/2013",
            CplNamespace::Smpte2067_3_2016 => "http://www.smpte-ra.org/schemas/2067-3/2016",
            other => panic!("unsupported namespace in test helper: {other:?}"),
        };
        let scope_attr = scope
            .map(|s| format!(r#" scope="{s}""#))
            .unwrap_or_default();
        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist xmlns="{ns_uri}">
  <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
  <IssueDate>2024-01-01T00:00:00Z</IssueDate>
  <ContentTitle>Test</ContentTitle>
  <ContentKind{scope_attr}>{value}</ContentKind>
  <EditRate>24 1</EditRate>
  <SegmentList>
    <Segment>
      <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
      <SequenceList/>
    </Segment>
  </SegmentList>
</CompositionPlaylist>"#
        );
        let cpl = crate::cpl::parse_cpl(&xml).expect("parse failed");
        assert_eq!(cpl.namespace, ns);
        cpl
    }

    fn has_content_kind_unknown(cpl: &CompositionPlaylist) -> bool {
        validate_cpl(cpl)
            .iter()
            .any(|i| i.code.contains("ContentKindUnknown"))
    }

    const CK_SCOPE_2016: &str = "http://www.smpte-ra.org/schemas/2067-3/2016#content-kind";

    #[test]
    fn content_kind_table_3_values_are_accepted() {
        for ns in [
            CplNamespace::Smpte2067_3_2013,
            CplNamespace::Smpte2067_3_2016,
        ] {
            for value in [
                "advertisement",
                "feature",
                "psa",
                "rating",
                "short",
                "teaser",
                "test",
                "trailer",
                "transitional",
                "episode",
                "highlights",
                "event",
            ] {
                let cpl = cpl_with_content_kind(ns.clone(), None, value);
                assert!(
                    !has_content_kind_unknown(&cpl),
                    "'{value}' ({ns:?}) is in Table 3 and must not be flagged",
                );
            }
        }
    }

    #[test]
    fn content_kind_values_not_in_table_3_are_flagged() {
        for ns in [
            CplNamespace::Smpte2067_3_2013,
            CplNamespace::Smpte2067_3_2016,
        ] {
            for value in ["promo", "rating-bump", "commercial", "supplemental"] {
                let cpl = cpl_with_content_kind(ns.clone(), None, value);
                assert!(
                    has_content_kind_unknown(&cpl),
                    "'{value}' ({ns:?}) is not in Table 3 and must be flagged",
                );
            }
        }
    }

    /// ContentKindType is an extension of xs:string (ST 2067-3:2013 and 2016
    /// schema), so whitespace is part of the value and " feature " does not
    /// "match" the Table 3 value "feature".
    #[test]
    fn content_kind_with_surrounding_whitespace_is_flagged() {
        for ns in [
            CplNamespace::Smpte2067_3_2013,
            CplNamespace::Smpte2067_3_2016,
        ] {
            for value in [" feature ", "feature ", " feature", "\nfeature\n"] {
                let cpl = cpl_with_content_kind(ns.clone(), None, value);
                assert!(
                    has_content_kind_unknown(&cpl),
                    "{value:?} ({ns:?}) is not exactly a Table 3 value and must be flagged",
                );
            }
        }
        let cpl = cpl_with_content_kind(
            CplNamespace::Smpte2067_3_2016,
            Some(CK_SCOPE_2016),
            " documentary",
        );
        assert!(has_content_kind_unknown(&cpl));
    }

    /// ST 2067-3 §6.1.8: the value "shall match" a Table 3 value (2013 p11,
    /// 2016 p10). Matching is exact, so case variants and synonyms that the
    /// parser used to fold are not Table 3 values.
    #[test]
    fn content_kind_requires_exact_table_3_token() {
        for ns in [
            CplNamespace::Smpte2067_3_2013,
            CplNamespace::Smpte2067_3_2016,
        ] {
            for value in [
                "Feature",
                "RATING",
                "Episode",
                "PSA",
                "public-service-announcement",
                "PublicServiceAnnouncement",
                "RatingBump",
            ] {
                let cpl = cpl_with_content_kind(ns.clone(), None, value);
                assert!(
                    has_content_kind_unknown(&cpl),
                    "'{value}' ({ns:?}) is not an exact Table 3 value and must be flagged",
                );
            }
        }
        let cpl = cpl_with_content_kind(
            CplNamespace::Smpte2067_3_2016,
            Some(CK_SCOPE_2016),
            "Documentary",
        );
        assert!(
            has_content_kind_unknown(&cpl),
            "'Documentary' is not a Table 4 value"
        );
    }

    /// ContentKind is optional (minOccurs="0" in the ST 2067-3:2013 and
    /// 2016 CompositionPlaylistType schema), so an absent element is not
    /// an unknown value.
    #[test]
    fn content_kind_absent_is_not_flagged() {
        for ns_uri in [
            "http://www.smpte-ra.org/schemas/2067-3/2013",
            "http://www.smpte-ra.org/schemas/2067-3/2016",
        ] {
            let xml = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist xmlns="{ns_uri}">
  <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
  <IssueDate>2024-01-01T00:00:00Z</IssueDate>
  <ContentTitle>Test</ContentTitle>
  <EditRate>24 1</EditRate>
  <SegmentList>
    <Segment>
      <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
      <SequenceList/>
    </Segment>
  </SegmentList>
</CompositionPlaylist>"#
            );
            let cpl = crate::cpl::parse_cpl(&xml).expect("parse failed");
            assert!(
                !has_content_kind_unknown(&cpl),
                "absent ContentKind ({ns_uri}) must not be flagged",
            );
        }

        // Same for an in-memory CPL left at the parser's default.
        let mut cpl = minimal_cpl();
        cpl.content_kind = crate::cpl::ContentKindElement {
            kind: ContentKind::Other("unknown".to_string()),
            scope: None,
        };
        assert!(!has_content_kind_unknown(&cpl));
    }

    #[test]
    fn content_kind_2016_scope_uses_table_4() {
        for value in ["supplemental", "documentary"] {
            let cpl =
                cpl_with_content_kind(CplNamespace::Smpte2067_3_2016, Some(CK_SCOPE_2016), value);
            assert!(!has_content_kind_unknown(&cpl), "'{value}' is in Table 4");
        }
        let cpl = cpl_with_content_kind(
            CplNamespace::Smpte2067_3_2016,
            Some(CK_SCOPE_2016),
            "feature",
        );
        assert!(
            has_content_kind_unknown(&cpl),
            "'feature' is not in Table 4 (2016 scope)",
        );
        // The 2016 scope does not exist in the 2013 edition: out of scope.
        let cpl = cpl_with_content_kind(
            CplNamespace::Smpte2067_3_2013,
            Some(CK_SCOPE_2016),
            "anything",
        );
        assert!(!has_content_kind_unknown(&cpl));
    }

    // ── MarkerOffsetOutOfRange (ST 2067-3 §6.13) ─────────────────────────────

    fn cpl_with_marker_offsets(
        intrinsic_duration: u64,
        entry_point: Option<u64>,
        source_duration: Option<u64>,
        offsets: &[u64],
    ) -> CompositionPlaylist {
        use crate::cpl::{MarkerInfo, MarkerLabel, MarkerLabelElement, MarkerSequence};
        let mut resource = make_resource(intrinsic_duration, source_duration, None);
        resource.entry_point = entry_point;
        resource.markers = offsets
            .iter()
            .map(|&offset| MarkerInfo {
                annotation: None,
                label: MarkerLabelElement::from(MarkerLabel::Ffoc),
                offset,
            })
            .collect();
        let mut sl = empty_sequence_list();
        sl.marker_sequences.push(MarkerSequence {
            id: dummy_uuid(),
            track_id: dummy_uuid(),
            resource_list: ResourceList {
                resources: vec![resource],
            },
        });
        let mut cpl = minimal_cpl();
        cpl.segment_list = SegmentList {
            segments: vec![Segment {
                id: dummy_uuid(),
                sequence_list: sl,
            }],
        };
        cpl
    }

    fn has_marker_offset_out_of_range(cpl: &CompositionPlaylist) -> bool {
        validate_cpl(cpl)
            .iter()
            .any(|i| i.code.contains("MarkerOffsetOutOfRange"))
    }

    /// §6.13: IntrinsicDuration shall be equal or larger than the largest
    /// Offset, so Offset == IntrinsicDuration is allowed.
    #[test]
    fn marker_offset_equal_to_intrinsic_duration_ok() {
        let cpl = cpl_with_marker_offsets(100, None, None, &[0, 100]);
        assert!(!has_marker_offset_out_of_range(&cpl));
    }

    /// §6.13 bounds Offset by IntrinsicDuration only; markers outside the
    /// EntryPoint/SourceDuration playable region are not flagged.
    #[test]
    fn marker_offset_outside_playable_region_ok() {
        let cpl = cpl_with_marker_offsets(100, Some(10), Some(20), &[0, 5, 50, 99]);
        assert!(!has_marker_offset_out_of_range(&cpl));
    }

    #[test]
    fn marker_offset_beyond_intrinsic_duration_flagged() {
        let cpl = cpl_with_marker_offsets(100, None, None, &[101]);
        assert!(has_marker_offset_out_of_range(&cpl));
    }
}
