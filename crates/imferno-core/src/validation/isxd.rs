//! SMPTE ST 2067-202 ISXD Plug-in validator.
//!
//! Implements the ISXD (Isochronous Stream of XML Documents) plug-in
//! constraints for SMPTE ST 2067-202:2023.
//!
//! Edition note (AUDIT-2): the publication is ST 2067-202:**2023**; the XML
//! namespace year is frozen at `/2022/` (§6 Table 1), so [`URI_2022`] is
//! correct and unchanged.
//!
//! The plug-in validator runs App2E base validation (ST 2067-21) internally,
//! then applies ISXD-specific descriptor and sequence constraints.

// codes live in isxd_codes.rs (declared from validation/mod.rs)

use std::collections::{HashMap, HashSet};

use crate::cpl::CompositionPlaylist;
use crate::diagnostics::{Category, Severity, ValidationIssue};
use crate::validation::{App2E2021, ConstraintsValidator};

use crate::validation::isxd_codes::{self as isxd_codes, IsxdCode};

// ── Public API ───────────────────────────────────────────────────────────────

/// ST 2067-202:2023 ISXD Plug-in validator.
///
/// Runs App2E base validation plus ST 2067-202:2023-specific ISXD constraints.
pub struct AppIsxdPlugin2023;

impl ConstraintsValidator for AppIsxdPlugin2023 {
    fn spec_id(&self) -> &str {
        "ST 2067-202:2023 (ISXD Plug-in)"
    }

    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        App2E2021.validate_all(cpl, true, &mut issues);
        validate_isxd_descriptors(cpl, isxd_codes::St2067_202_2023::for_code, &mut issues);
        validate_isxd_sequences(cpl, isxd_codes::St2067_202_2023::for_code, &mut issues);
        issues
    }
}

// ── Namespace URIs ────────────────────────────────────────────────────────────

pub const URI_2022: &str = "http://www.smpte-ra.org/ns/2067-202/2022";

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Validate ISXDDataEssenceDescriptor-level constraints.
///
/// For every EssenceDescriptor that carries an ISXDDataEssenceDescriptor:
/// - `NamespaceURI` shall be present (§9.2 Table 5, Req) → `NamespaceUriMissing`
///
/// AUDIT-19: the former `SubDescriptorMissing` rule (required a
/// `ContainerConstraintsSubDescriptor`) was deleted — "ContainerConstraints"
/// appears nowhere in ST 2067-202 prose; §9.2 says implementations "may
/// extend" the descriptor via a SubDescriptor and "shall ignore unrecognized
/// SubDescriptors". The requirement belongs to the ST 2127 lineage
/// (ST 2067-203), not to -202.
fn validate_isxd_descriptors(
    cpl: &CompositionPlaylist,
    code: fn(IsxdCode) -> &'static str,
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
            issues.push(ValidationIssue::new(
                // §9.2 Table 5 marks NamespaceURI "Req" — SHALL, so Error
                // (AUDIT-20; previously mis-emitted as Warning).
                Severity::Error,
                Category::Data,
                code(IsxdCode::NamespaceUriMissing),
                format!(
                    "ISXDDataEssenceDescriptor (EssenceDescriptor Id={}) is missing NamespaceURI \
                     (ST 2067-202 §9.2 Table 5, Req).",
                    ed.id
                ),
            ));
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
fn validate_isxd_sequences(
    cpl: &CompositionPlaylist,
    code: fn(IsxdCode) -> &'static str,
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
                issues.push(ValidationIssue::new(
                    Severity::Error,
                    Category::Data,
                    code(IsxdCode::ISXDSequenceNoResources),
                    format!("ISXDSequence (Id={}) contains no Resources.", isxd_seq.id),
                ));
                continue;
            }

            for resource in resources {
                if let (Some(main_er), Some(isxd_er)) =
                    (main_image_edit_rate, resource.edit_rate.or(cpl.edit_rate))
                {
                    if isxd_er != main_er {
                        issues.push(ValidationIssue::new(
                            Severity::Error,
                            Category::Data,
                            code(IsxdCode::EditRateMismatch),
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
                        issues.push(ValidationIssue::new(
                            Severity::Error,
                            Category::Data,
                            code(IsxdCode::ISXDSequenceSourceEncodingInvalid),
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

    let mut track_ids: Vec<_> = track_namespace_uris.keys().cloned().collect();
    track_ids.sort();
    for track_id in track_ids {
        let uris_set = &track_namespace_uris[&track_id];
        if uris_set.len() > 1 {
            let mut uris: Vec<_> = uris_set.iter().cloned().collect();
            uris.sort();
            issues.push(ValidationIssue::new(
                Severity::Error,
                Category::Data,
                code(IsxdCode::NamespaceUriMismatch),
                format!(
                    "ISXD Virtual Track (TrackId={track_id}) references Track Files with \
                     inconsistent NamespaceURI values: {uris:?} — ST 2067-202 §6 requires an \
                     identical NamespaceURI across the Virtual Track.",
                ),
            ));
        }
    }
}
