//! SMPTE ST 2067-202 ISXD Plug-in validator.
//!
//! Implements the ISXD (Immersive Sound XML Data) plug-in constraints for
//! SMPTE ST 2067-202:2022.
//!
//! The plug-in validator runs App2E base validation (ST 2067-21) internally,
//! then applies ISXD-specific descriptor and sequence constraints.

// codes live in isxd_codes.rs (declared from validation/mod.rs)

use std::collections::{HashMap, HashSet};

use crate::diagnostics::{Category, Severity, ValidationIssue};
use crate::validation::{App2E2021, ConstraintsValidator};
use crate::cpl::CompositionPlaylist;

use crate::validation::isxd_codes::{self as isxd_codes, IsxdCode};

// ── Public API ───────────────────────────────────────────────────────────────

/// ST 2067-202:2022 ISXD Plug-in validator.
///
/// Runs App2E base validation plus ST 2067-202:2022-specific ISXD constraints.
pub struct AppIsxdPlugin2022;

impl ConstraintsValidator for AppIsxdPlugin2022 {
    fn spec_id(&self) -> &str {
        "ST 2067-202:2022 (ISXD Plug-in)"
    }

    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        App2E2021.validate_all(cpl, true, &mut issues);
        validate_isxd_descriptors(cpl, isxd_codes::St2067_202_2022::for_code, &mut issues);
        validate_isxd_sequences(cpl, isxd_codes::St2067_202_2022::for_code, &mut issues);
        issues
    }
}

// ── Namespace URIs ────────────────────────────────────────────────────────────

pub const URI_2022: &str = "http://www.smpte-ra.org/ns/2067-202/2022";

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Validate ISXDDataEssenceDescriptor-level constraints.
///
/// For every EssenceDescriptor that carries an ISXDDataEssenceDescriptor:
/// - `ContainerConstraintsSubDescriptor` shall be present → `SubDescriptorMissing`
/// - `NamespaceURI` shall be present → `NamespaceUriMissing` (Warning)
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

        let has_ccsd = isxd
            .sub_descriptors
            .as_ref()
            .and_then(|s| s.container_constraints_sub_descriptor.as_ref())
            .is_some();

        if !has_ccsd {
            issues.push(ValidationIssue::new(
                Severity::Error,
                Category::Audio,
                code(IsxdCode::SubDescriptorMissing),
                format!(
                    "ISXDDataEssenceDescriptor (EssenceDescriptor Id={}) is missing \
                     ContainerConstraintsSubDescriptor.",
                    ed.id
                ),
            ));
        }

        if isxd.namespace_uri.is_none() {
            issues.push(ValidationIssue::new(
                Severity::Warning,
                Category::Audio,
                code(IsxdCode::NamespaceUriMissing),
                format!(
                    "ISXDDataEssenceDescriptor (EssenceDescriptor Id={}) is missing NamespaceURI.",
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
/// - All resolved descriptors within the same ISXDSequence shall have the same
///   NamespaceURI → `NamespaceUriMismatch`
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

    for segment in &cpl.segment_list.segments {
        let sl = &segment.sequence_list;

        for isxd_seq in &sl.isxd_sequences {
            let resources = &isxd_seq.resource_list.resources;

            if resources.is_empty() {
                issues.push(ValidationIssue::new(
                    Severity::Error,
                    Category::Audio,
                    code(IsxdCode::ISXDSequenceNoResources),
                    format!("ISXDSequence (Id={}) contains no Resources.", isxd_seq.id),
                ));
                continue;
            }

            let mut namespace_uris: HashSet<String> = HashSet::new();

            for resource in resources {
                let se_uuid = match resource.source_encoding {
                    Some(ref uuid) => uuid.to_string(),
                    None => continue,
                };

                match isxd_descriptor_map.get(&se_uuid) {
                    Some(ns_uri) => {
                        if let Some(uri) = ns_uri {
                            namespace_uris.insert(uri.clone());
                        }
                    }
                    None => {
                        issues.push(ValidationIssue::new(
                            Severity::Error,
                            Category::Audio,
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

            if namespace_uris.len() > 1 {
                let mut uris: Vec<_> = namespace_uris.into_iter().collect();
                uris.sort();
                issues.push(ValidationIssue::new(
                    Severity::Error,
                    Category::Audio,
                    code(IsxdCode::NamespaceUriMismatch),
                    format!(
                        "ISXDSequence (Id={}) references descriptors with inconsistent \
                         NamespaceURI values: {:?}",
                        isxd_seq.id, uris
                    ),
                ));
            }
        }
    }
}
