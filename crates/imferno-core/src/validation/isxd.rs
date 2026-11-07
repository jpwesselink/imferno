//! SMPTE ST 2067-202 ISXD Plug-in validator.
//!
//! Implements the ISXD (Isochronous Stream of XML Documents) plug-in
//! constraints for SMPTE ST 2067-202:2023.
//!
//! Edition note (AUDIT-2): the publication is ST 2067-202:**2023**; the XML
//! namespace year is frozen at `/2022/` (§6 Table 1), so [`URI_2022`] is
//! correct and unchanged.
//!
//! The plug-in validator applies only ISXD constraints. ST 2067-202 does not
//! reference ST 2067-21; App #2E rules come from the App #2E validator the
//! CPL's ApplicationIdentification selects, in that edition.

use std::collections::{HashMap, HashSet};

use crate::assetmap::SmpteUl;
use crate::catalogue::st2067_202::{self, St2067_202Edition};
use crate::cpl::CompositionPlaylist;
use crate::diagnostics::ValidationIssue;
use crate::validation::ConstraintsValidator;

// ── Public API ───────────────────────────────────────────────────────────────

/// ST 2067-202:2023 ISXD Plug-in validator.
///
/// Applies the ST 2067-202:2023 ISXD constraints.
pub struct AppIsxdPlugin2023;

impl ConstraintsValidator for AppIsxdPlugin2023 {
    fn spec_id(&self) -> &str {
        "ST 2067-202:2023 (ISXD Plug-in)"
    }

    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        validate_isxd_descriptors(cpl, St2067_202Edition::V2023, &mut issues);
        validate_isxd_sequences(cpl, St2067_202Edition::V2023, &mut issues);
        issues
    }
}

// ── Namespace URIs ────────────────────────────────────────────────────────────

pub const URI_2022: &str = "http://www.smpte-ra.org/ns/2067-202/2022";

/// ST 2067-202:2023 §9.3 Table 6 — UTF-8 Text Data Essence Coding UL.
const UTF8_TEXT_DATA_ESSENCE_CODING: &str = "urn:smpte:ul:060e2b34.04010105.0e090606.00000000";

// ── Internal helpers ──────────────────────────────────────────────────────────

/// UL comparison ignoring byte 8, the version byte (ST 377-1 §6.3,
/// ST 336 §5.2.3), as in the CPL enum and IAB checks. Agreed for this
/// UL at the IMF UG plugfest: https://github.com/Netflix/photon/issues/408
fn is_utf8_text_data_essence_coding(ul: &str) -> bool {
    // Only the version byte is ignored. The value must still be the
    // canonical URN form (`urn:smpte:ul:` and four dot-separated groups of
    // eight hex digits); `SmpteUl::parse` alone is more lenient.
    let ul = ul.to_ascii_lowercase();
    let canonical = ul.strip_prefix("urn:smpte:ul:").is_some_and(|hex| {
        let groups: Vec<&str> = hex.split('.').collect();
        groups.len() == 4
            && groups
                .iter()
                .all(|g| g.len() == 8 && g.bytes().all(|b| b.is_ascii_hexdigit()))
    });
    if !canonical {
        return false;
    }
    match (
        SmpteUl::parse(&ul),
        SmpteUl::parse(UTF8_TEXT_DATA_ESSENCE_CODING),
    ) {
        (Ok(a), Ok(b)) => a.matches_ignoring_version(&b),
        _ => false,
    }
}

/// Validate ISXDDataEssenceDescriptor-level constraints.
///
/// For every EssenceDescriptor that carries an ISXDDataEssenceDescriptor:
/// - `NamespaceURI` shall be present (§9.2 Table 5, Req) → `NamespaceUriMissing`
/// - `DataEssenceCoding` shall be present and shall be the UTF-8 Text Data
///   Essence Coding UL (§9.3 Table 6) → `DataEssenceCodingMissing`/`Invalid`
///   (AUDIT-21)
///
/// AUDIT-19: the former `SubDescriptorMissing` rule (required a
/// `ContainerConstraintsSubDescriptor`) was deleted — "ContainerConstraints"
/// appears nowhere in ST 2067-202 prose; §9.2 says implementations "may
/// extend" the descriptor via a SubDescriptor and "shall ignore unrecognized
/// SubDescriptors". The requirement belongs to the ST 2127 lineage
/// (ST 2067-203), not to -202.
fn validate_isxd_descriptors(
    cpl: &CompositionPlaylist,
    spec_ed: St2067_202Edition,
    issues: &mut Vec<ValidationIssue>,
) {
    let eds = match cpl.essence_descriptor_list.as_ref() {
        Some(list) => &list.essence_descriptors,
        None => return,
    };

    for ed in eds {
        let isxd = match ed.isxd_data_essence_descriptor.as_ref() {
            Some(d) => d,
            None => continue,
        };

        if isxd.namespace_uri.is_none() {
            issues.push(ValidationIssue::from_rule(
                // §9.2 Table 5 marks NamespaceURI "Req" — SHALL, so Error
                // (AUDIT-20; previously mis-emitted as Warning).
                &st2067_202::NamespaceUriMissing,
                spec_ed,
                format!(
                    "ISXDDataEssenceDescriptor (EssenceDescriptor Id={}) is missing NamespaceURI \
                     (ST 2067-202 §9.2 Table 5, Req).",
                    ed.id
                ),
            ));
        }

        // §9.3: "The DataEssenceCoding item shall be present in the
        // ISXDDataEssenceDescriptor. The value ... shall be as defined in
        // Table 6": urn:smpte:ul:060e2b34.04010105.0e090606.00000000
        // (UTF-8 Text Data Essence Coding).
        match isxd.data_essence_coding.as_deref().map(str::trim) {
            None | Some("") => {
                issues.push(ValidationIssue::from_rule(
                    &st2067_202::DataEssenceCodingMissing,
                    spec_ed,
                    format!(
                        "ISXDDataEssenceDescriptor (EssenceDescriptor Id={}) is missing \
                         DataEssenceCoding (ST 2067-202 §9.3).",
                        ed.id
                    ),
                ));
            }
            Some(ul) if !is_utf8_text_data_essence_coding(ul) => {
                issues.push(ValidationIssue::from_rule(
                    &st2067_202::DataEssenceCodingInvalid,
                    spec_ed,
                    format!(
                        "ISXDDataEssenceDescriptor (EssenceDescriptor Id={}) DataEssenceCoding \
                         `{ul}` is not the UTF-8 Text Data Essence Coding UL \
                         {UTF8_TEXT_DATA_ESSENCE_CODING} (ST 2067-202 §9.3 Table 6).",
                        ed.id
                    ),
                ));
            }
            _ => {}
        }
    }
}

/// Validate ISXDSequence-level constraints.
///
/// - ISXDSequence shall contain at least one Resource → `ISXDSequenceNoResources`
/// - Each resource's SourceEncoding shall reference an ISXDDataEssenceDescriptor →
///   `ISXDSequenceSourceEncodingInvalid`
/// - §6: "All ISXD Track Files referenced by an ISXD Virtual Track shall have
///   an identical value for the NamespaceURI item" → `NamespaceUriMismatch`,
///   scoped per Virtual Track (sequences sharing a TrackId across segments),
///   not per sequence (AUDIT-20).
/// - §6: "The Edit Rate of an ISXD Virtual Track shall be equal to the Edit
///   Rate of the Main Image Virtual Track" → `EditRateMismatch` (AUDIT-21).
/// - §6: "A Composition ... that references an ISXD Track File, shall contain
///   one or more ISXD Virtual Tracks" → `ISXDVirtualTrackMissing` (AUDIT-21).
fn validate_isxd_sequences(
    cpl: &CompositionPlaylist,
    spec_ed: St2067_202Edition,
    issues: &mut Vec<ValidationIssue>,
) {
    // Build lookup: EssenceDescriptor UUID string → namespace_uri Option<String>
    let isxd_descriptor_map: HashMap<String, Option<String>> = cpl
        .essence_descriptor_list
        .as_ref()
        .map(|edl| {
            edl.essence_descriptors
                .iter()
                .filter_map(|ed| {
                    ed.isxd_data_essence_descriptor
                        .as_ref()
                        .map(|d| (ed.id.to_string(), d.namespace_uri.clone()))
                })
                .collect()
        })
        .unwrap_or_default();

    // §6 scopes NamespaceURI homogeneity to the Virtual Track: collect the
    // namespace URIs seen per TrackId across every segment.
    let mut track_namespace_uris: HashMap<String, HashSet<String>> = HashMap::new();

    // §6: the ISXD Virtual Track edit rate shall equal the Main Image
    // Virtual Track edit rate. Resolve the Main Image rate from the first
    // Main Image resource (absent resource EditRate inherits the CPL edit
    // rate per ST 2067-3 §6.9.3).
    let main_image_edit_rate = cpl
        .segment_list
        .segments
        .iter()
        .flat_map(|seg| &seg.sequence_list.main_image_sequences)
        .flat_map(|seq| &seq.resource_list.resources)
        .find_map(|r| r.edit_rate.or(cpl.edit_rate));

    for segment in &cpl.segment_list.segments {
        let sl = &segment.sequence_list;

        for isxd_seq in &sl.isxd_sequences {
            let resources = &isxd_seq.resource_list.resources;

            if resources.is_empty() {
                issues.push(ValidationIssue::from_rule(
                    &st2067_202::ISXDSequenceNoResources,
                    spec_ed,
                    format!("ISXDSequence (Id={}) contains no Resources.", isxd_seq.id),
                ));
                continue;
            }

            for resource in resources {
                if let (Some(main_er), Some(isxd_er)) =
                    (main_image_edit_rate, resource.edit_rate.or(cpl.edit_rate))
                {
                    if isxd_er != main_er {
                        issues.push(ValidationIssue::from_rule(
                            &st2067_202::EditRateMismatch,
                            spec_ed,
                            format!(
                                "ISXDSequence (Id={}) Resource (Id={}) EditRate {}/{} does not \
                                 equal the Main Image Virtual Track Edit Rate {}/{} — \
                                 ST 2067-202 §6.",
                                isxd_seq.id,
                                resource.id,
                                isxd_er.numerator,
                                isxd_er.denominator,
                                main_er.numerator,
                                main_er.denominator,
                            ),
                        ));
                    }
                }

                let se_uuid = match resource.source_encoding {
                    Some(ref uuid) => uuid.to_string(),
                    None => continue,
                };

                match isxd_descriptor_map.get(&se_uuid) {
                    Some(ns_uri) => {
                        if let Some(uri) = ns_uri {
                            track_namespace_uris
                                .entry(isxd_seq.track_id.to_string())
                                .or_default()
                                .insert(uri.clone());
                        }
                    }
                    None => {
                        issues.push(ValidationIssue::from_rule(
                            &st2067_202::ISXDSequenceSourceEncodingInvalid,
                            spec_ed,
                            format!(
                                "ISXDSequence (Id={}) Resource (Id={}) SourceEncoding={} does not \
                                 reference an ISXDDataEssenceDescriptor.",
                                isxd_seq.id, resource.id, se_uuid
                            ),
                        ));
                    }
                }
            }
        }
    }

    // §6: "A Composition, as defined in SMPTE ST 2067-3, that references an
    // ISXD Track File, shall contain one or more ISXD Virtual Tracks." If any
    // resource (in any sequence type) resolves to an ISXDDataEssenceDescriptor
    // but the CPL has no ISXDSequence, the ISXD Track File is being referenced
    // outside an ISXD Virtual Track.
    let has_isxd_sequence = cpl
        .segment_list
        .segments
        .iter()
        .any(|seg| !seg.sequence_list.isxd_sequences.is_empty());
    if !has_isxd_sequence && !isxd_descriptor_map.is_empty() {
        let references_isxd = cpl.segment_list.segments.iter().any(|seg| {
            seg.sequence_list.all_sequences().iter().any(|seq| {
                seq.resource_list().resources.iter().any(|r| {
                    r.source_encoding
                        .as_ref()
                        .is_some_and(|se| isxd_descriptor_map.contains_key(&se.to_string()))
                })
            })
        });
        if references_isxd {
            issues.push(ValidationIssue::from_rule(
                &st2067_202::ISXDVirtualTrackMissing,
                spec_ed,
                "Composition references an ISXD Track File but contains no ISXD Virtual \
                 Track — ST 2067-202 §6 requires one or more ISXD Virtual Tracks."
                    .to_string(),
            ));
        }
    }

    let mut track_ids: Vec<_> = track_namespace_uris.keys().cloned().collect();
    track_ids.sort();
    for track_id in track_ids {
        let uris_set = &track_namespace_uris[&track_id];
        if uris_set.len() > 1 {
            let mut uris: Vec<_> = uris_set.iter().cloned().collect();
            uris.sort();
            issues.push(ValidationIssue::from_rule(
                &st2067_202::NamespaceUriMismatch,
                spec_ed,
                format!(
                    "ISXD Virtual Track (TrackId={track_id}) references Track Files with \
                     inconsistent NamespaceURI values: {uris:?} — ST 2067-202 §6 requires an \
                     identical NamespaceURI across the Virtual Track.",
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpl::parse_cpl;

    const CPL: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-data/ISXD/CPL_ISXD_TEST_1.xml"
    ));
    const CANONICAL: &str = "urn:smpte:ul:060e2b34.04010105.0e090606.00000000";

    fn coding_issues(coding: &str) -> Vec<ValidationIssue> {
        let xml = CPL.replace(CANONICAL, coding);
        let cpl = parse_cpl(&xml).expect("ISXD fixture parses");
        AppIsxdPlugin2023
            .validate_cpl(&cpl)
            .into_iter()
            .filter(|i| i.code.contains("DataEssenceCoding"))
            .collect()
    }

    #[test]
    fn canonical_ul_passes() {
        assert!(CPL.contains(CANONICAL));
        assert!(coding_issues(CANONICAL).is_empty());
    }

    /// https://github.com/Netflix/photon/issues/408: a different version
    /// byte (byte 8) is the same UL.
    #[test]
    fn version_byte_variant_passes() {
        for ul in [
            "urn:smpte:ul:060e2b34.04010101.0e090606.00000000",
            "URN:SMPTE:UL:060E2B34.0401010D.0E090606.00000000",
        ] {
            let issues = coding_issues(ul);
            assert!(issues.is_empty(), "{ul}: {issues:#?}");
        }
    }

    #[test]
    fn different_ul_is_invalid() {
        let issues = coding_issues("urn:smpte:ul:060e2b34.04010105.0e090607.01010103");
        assert_eq!(issues.len(), 1, "{issues:#?}");
        assert!(issues[0].code.ends_with("9.3/DataEssenceCodingInvalid"));
    }

    /// Only the version byte is ignored: a value that is not a canonical
    /// `urn:smpte:ul:` URN (prefix, four dot-separated groups of eight hex
    /// digits) is still invalid.
    #[test]
    fn malformed_urn_is_invalid() {
        for ul in [
            "060e2b34.04010105.0e090606.00000000",
            "urn:smpte:ul:060e2b3404010105.0e090606.00000000",
            "urn:smpte:ul:060e2b34.0401.0105.0e090606.00000000",
            "urn:smpte:ul:060e2b34040101050e09060600000000",
            "urn:smpte:ul:060e2b34.04010105.0e090606.0000000g",
        ] {
            let issues = coding_issues(ul);
            assert_eq!(issues.len(), 1, "{ul}: {issues:#?}");
            assert!(issues[0].code.ends_with("9.3/DataEssenceCodingInvalid"));
        }
    }
}
