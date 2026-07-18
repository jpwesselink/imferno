//! SMPTE ST 2067-203 (S-ADM Audio) and ST 2067-204 (ADM Audio) plug-in
//! CPL-level validators.
//!
//! The two specs are structural mirrors — same three CPL-level concepts
//! (a `*SignalSequence` / `*AudioSequence` in the sequence list, a
//! matching `*VirtualTrackParameterSet` under `ExtensionProperties`,
//! zero-or-more `*SoundfieldGroupSelector` children on the parameter set).
//! One shared implementation walks both by parameterizing over the
//! sequence + parameter-set accessors.
//!
//! Runs App2E base validation internally (the plug-in composes with an
//! App2E CPL), then applies the three cross-reference / structural
//! constraints defined in each spec's §5.4.

use crate::cpl::CompositionPlaylist;
use crate::diagnostics::{Severity, ValidationIssue};
use crate::validation::sadm_codes::{self as sadm_codes, SadmCode};
use crate::validation::{App2E2021, ConstraintsValidator};

// ── Public API ───────────────────────────────────────────────────────────────

/// ST 2067-203:2023 S-ADM Audio Plug-in validator.
pub struct AppSadmPlugin2023;

impl ConstraintsValidator for AppSadmPlugin2023 {
    fn spec_id(&self) -> &str {
        "ST 2067-203:2023 (S-ADM Audio Plug-in)"
    }

    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        App2E2021.validate_all(cpl, true, &mut issues);
        validate_sadm_cpl(cpl, &mut issues);
        issues
    }
}

/// ST 2067-204:2023 ADM Audio Plug-in validator.
pub struct AppAdmPlugin2023;

impl ConstraintsValidator for AppAdmPlugin2023 {
    fn spec_id(&self) -> &str {
        "ST 2067-204:2023 (ADM Audio Plug-in)"
    }

    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        App2E2021.validate_all(cpl, true, &mut issues);
        validate_adm_cpl(cpl, &mut issues);
        issues
    }
}

// ── Namespace URIs (frozen at /2022/ per each spec's §5) ─────────────────────

pub const URI_2067_203_2022: &str = "http://www.smpte-ra.org/ns/2067-203/2022";
pub const URI_2067_204_2022: &str = "http://www.smpte-ra.org/ns/2067-204/2022";

// ── Internal helpers ─────────────────────────────────────────────────────────

/// Collect (Signal-sequence TrackIds, Resource IDs by TrackId) across
/// every segment for the S-ADM specs. Structurally identical to the ADM
/// helper below; they differ only in which SequenceList field they walk.
fn sadm_signal_view(
    cpl: &CompositionPlaylist,
) -> (
    std::collections::HashSet<String>,
    std::collections::HashMap<String, std::collections::HashSet<String>>,
) {
    let mut track_ids = std::collections::HashSet::new();
    let mut resources_by_track_id: std::collections::HashMap<
        String,
        std::collections::HashSet<String>,
    > = std::collections::HashMap::new();
    for seg in &cpl.segment_list.segments {
        for seq in &seg.sequence_list.mga_sadm_signal_sequences {
            let tid = seq.track_id.to_string();
            track_ids.insert(tid.clone());
            let entry = resources_by_track_id.entry(tid).or_default();
            for r in &seq.resource_list.resources {
                entry.insert(r.id.to_string());
            }
        }
    }
    (track_ids, resources_by_track_id)
}

fn adm_signal_view(
    cpl: &CompositionPlaylist,
) -> (
    std::collections::HashSet<String>,
    std::collections::HashMap<String, std::collections::HashSet<String>>,
) {
    let mut track_ids = std::collections::HashSet::new();
    let mut resources_by_track_id: std::collections::HashMap<
        String,
        std::collections::HashSet<String>,
    > = std::collections::HashMap::new();
    for seg in &cpl.segment_list.segments {
        for seq in &seg.sequence_list.adm_audio_sequences {
            let tid = seq.track_id.to_string();
            track_ids.insert(tid.clone());
            let entry = resources_by_track_id.entry(tid).or_default();
            for r in &seq.resource_list.resources {
                entry.insert(r.id.to_string());
            }
        }
    }
    (track_ids, resources_by_track_id)
}

/// ST 2067-203 §5.3/5.4 CPL-level constraints.
fn validate_sadm_cpl(cpl: &CompositionPlaylist, issues: &mut Vec<ValidationIssue>) {
    let code = sadm_codes::St2067_203_2023::for_code;
    let (signal_track_ids, resources_by_track_id) = sadm_signal_view(cpl);

    // §5.3 / ST 2067-3 §7.2 — every SignalSequence needs at least one Resource.
    for seg in &cpl.segment_list.segments {
        for seq in &seg.sequence_list.mga_sadm_signal_sequences {
            if seq.resource_list.resources.is_empty() {
                issues.push(ValidationIssue::new(
                    Severity::Error,
                    crate::diagnostics::Category::Audio,
                    code(SadmCode::SignalSequenceNoResources),
                    format!(
                        "MGASADMSignalSequence (Id={}) has an empty ResourceList — \
                         ST 2067-203 §5.3 / ST 2067-3 §7.2 require at least one Resource.",
                        seq.id
                    ),
                ));
            }
        }
    }

    let (param_set_track_ids, param_sets) = if let Some(ext) = &cpl.extension_properties {
        let ids: std::collections::HashSet<String> = ext
            .mga_sadm_virtual_track_parameter_sets
            .iter()
            .map(|p| p.track_id.to_string())
            .collect();
        (ids, &ext.mga_sadm_virtual_track_parameter_sets[..])
    } else {
        (std::collections::HashSet::new(), &[][..])
    };

    // §5.4 — every SignalSequence's TrackId must be referenced by a
    // matching VirtualTrackParameterSet.
    for tid in &signal_track_ids {
        if !param_set_track_ids.contains(tid) {
            issues.push(ValidationIssue::new(
                Severity::Error,
                crate::diagnostics::Category::Audio,
                code(SadmCode::VirtualTrackParameterSetMissing),
                format!(
                    "MGASADMSignalSequence (TrackId={tid}) has no matching \
                     MGASADMVirtualTrackParameterSet under ExtensionProperties — \
                     ST 2067-203 §5.4 requires one per S-ADM VirtualTrack."
                ),
            ));
        }
    }

    // §5.4 — every VirtualTrackParameterSet.TrackId must match a
    // SignalSequence in the CPL (no orphaned parameter sets).
    for ps in param_sets {
        let tid = ps.track_id.to_string();
        if !signal_track_ids.contains(&tid) {
            issues.push(ValidationIssue::new(
                Severity::Error,
                crate::diagnostics::Category::Audio,
                code(SadmCode::VirtualTrackParameterSetOrphaned),
                format!(
                    "MGASADMVirtualTrackParameterSet (Id={}) references TrackId={tid} \
                     but no MGASADMSignalSequence in the CPL uses that TrackId — \
                     ST 2067-203 §5.4 orphan.",
                    ps.id
                ),
            ));
        }

        if ps.mga_sadm_operational_mode.trim().is_empty() {
            issues.push(ValidationIssue::new(
                Severity::Error,
                crate::diagnostics::Category::Audio,
                code(SadmCode::OperationalModeEmpty),
                format!(
                    "MGASADMVirtualTrackParameterSet (Id={}) has empty \
                     MGASADMOperationalMode — ST 2067-203 §5.4 requires a URI.",
                    ps.id
                ),
            ));
        }

        // §5.4 — SoundfieldGroupSelector.ResourceId must point at a
        // Resource in the referenced SignalSequence.
        if let Some(known_resources) = resources_by_track_id.get(&tid) {
            for sel in &ps.mga_sadm_soundfield_group_selector {
                let rid = sel.resource_id.to_string();
                if !known_resources.contains(&rid) {
                    issues.push(ValidationIssue::new(
                        Severity::Error,
                        crate::diagnostics::Category::Audio,
                        code(SadmCode::SoundfieldGroupSelectorResourceIdOrphaned),
                        format!(
                            "MGASADMSoundfieldGroupSelector.ResourceId={rid} \
                             (in ParameterSet Id={}) doesn't match any Resource in the \
                             MGASADMSignalSequence with TrackId={tid} — ST 2067-203 §5.4.",
                            ps.id
                        ),
                    ));
                }
            }
        }
    }
}

/// ST 2067-204 §5.3/5.4 CPL-level constraints — mirror of the S-ADM
/// path, differing only in element names.
fn validate_adm_cpl(cpl: &CompositionPlaylist, issues: &mut Vec<ValidationIssue>) {
    let code = sadm_codes::St2067_204_2023::for_code;
    let (signal_track_ids, resources_by_track_id) = adm_signal_view(cpl);

    for seg in &cpl.segment_list.segments {
        for seq in &seg.sequence_list.adm_audio_sequences {
            if seq.resource_list.resources.is_empty() {
                issues.push(ValidationIssue::new(
                    Severity::Error,
                    crate::diagnostics::Category::Audio,
                    code(SadmCode::SignalSequenceNoResources),
                    format!(
                        "ADMAudioSequence (Id={}) has an empty ResourceList — \
                         ST 2067-204 §5.3 / ST 2067-3 §7.2 require at least one Resource.",
                        seq.id
                    ),
                ));
            }
        }
    }

    let (param_set_track_ids, param_sets) = if let Some(ext) = &cpl.extension_properties {
        let ids: std::collections::HashSet<String> = ext
            .adm_audio_virtual_track_parameter_sets
            .iter()
            .map(|p| p.track_id.to_string())
            .collect();
        (ids, &ext.adm_audio_virtual_track_parameter_sets[..])
    } else {
        (std::collections::HashSet::new(), &[][..])
    };

    for tid in &signal_track_ids {
        if !param_set_track_ids.contains(tid) {
            issues.push(ValidationIssue::new(
                Severity::Error,
                crate::diagnostics::Category::Audio,
                code(SadmCode::VirtualTrackParameterSetMissing),
                format!(
                    "ADMAudioSequence (TrackId={tid}) has no matching \
                     ADMAudioVirtualTrackParameterSet under ExtensionProperties — \
                     ST 2067-204 §5.4 requires one per ADM VirtualTrack."
                ),
            ));
        }
    }

    for ps in param_sets {
        let tid = ps.track_id.to_string();
        if !signal_track_ids.contains(&tid) {
            issues.push(ValidationIssue::new(
                Severity::Error,
                crate::diagnostics::Category::Audio,
                code(SadmCode::VirtualTrackParameterSetOrphaned),
                format!(
                    "ADMAudioVirtualTrackParameterSet (Id={}) references TrackId={tid} \
                     but no ADMAudioSequence in the CPL uses that TrackId — \
                     ST 2067-204 §5.4 orphan.",
                    ps.id
                ),
            ));
        }

        if ps.adm_operational_mode.trim().is_empty() {
            issues.push(ValidationIssue::new(
                Severity::Error,
                crate::diagnostics::Category::Audio,
                code(SadmCode::OperationalModeEmpty),
                format!(
                    "ADMAudioVirtualTrackParameterSet (Id={}) has empty \
                     ADMOperationalMode — ST 2067-204 §5.4 requires a URI.",
                    ps.id
                ),
            ));
        }

        if let Some(known_resources) = resources_by_track_id.get(&tid) {
            for sel in &ps.adm_soundfield_group_selector {
                let rid = sel.resource_id.to_string();
                if !known_resources.contains(&rid) {
                    issues.push(ValidationIssue::new(
                        Severity::Error,
                        crate::diagnostics::Category::Audio,
                        code(SadmCode::SoundfieldGroupSelectorResourceIdOrphaned),
                        format!(
                            "ADMSoundfieldGroupSelector.ResourceId={rid} \
                             (in ParameterSet Id={}) doesn't match any Resource in the \
                             ADMAudioSequence with TrackId={tid} — ST 2067-204 §5.4.",
                            ps.id
                        ),
                    ));
                }
            }
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpl::parse_cpl;

    /// Build a minimal CPL body embedding an arbitrary SequenceList and
    /// ExtensionProperties fragment. Keeps the boilerplate out of the
    /// individual tests.
    fn cpl_xml(sequence_list_extras: &str, extension_properties_extras: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2016">
    <Id>urn:uuid:00000000-0000-0000-0000-00000000c203</Id>
    <IssueDate>2026-07-18T12:00:00Z</IssueDate>
    <ContentTitle>SADM/ADM plug-in test CPL</ContentTitle>
    <ContentKind>test</ContentKind>
    <EditRate>48000 1</EditRate>
    <ExtensionProperties>
        {extension_properties_extras}
    </ExtensionProperties>
    <SegmentList>
        <Segment>
            <Id>urn:uuid:00000000-0000-0000-0000-00000000c001</Id>
            <SequenceList>
                {sequence_list_extras}
            </SequenceList>
        </Segment>
    </SegmentList>
</CompositionPlaylist>"#
        )
    }

    // ── Shared shapes ────────────────────────────────────────────────────

    /// A well-formed MGASADMSignalSequence with one resource.
    const SADM_SIGNAL_SEQ: &str = r#"
        <MGASADMSignalSequence>
            <Id>urn:uuid:00000000-0000-0000-0000-00000000aa01</Id>
            <TrackId>urn:uuid:00000000-0000-0000-0000-000000000a01</TrackId>
            <ResourceList>
                <Resource>
                    <Id>urn:uuid:00000000-0000-0000-0000-000000000b01</Id>
                    <IntrinsicDuration>48000</IntrinsicDuration>
                    <SourceEncoding>urn:uuid:00000000-0000-0000-0000-000000000e01</SourceEncoding>
                    <TrackFileId>urn:uuid:00000000-0000-0000-0000-000000000f01</TrackFileId>
                </Resource>
            </ResourceList>
        </MGASADMSignalSequence>"#;

    /// A well-formed MGASADMVirtualTrackParameterSet that references the
    /// TrackId above.
    const SADM_PARAM_SET_OK: &str = r#"
        <MGASADMVirtualTrackParameterSet>
            <Id>urn:uuid:00000000-0000-0000-0000-000000000101</Id>
            <TrackId>urn:uuid:00000000-0000-0000-0000-000000000a01</TrackId>
            <MGASADMOperationalMode>urn:smpte:ul:060e2b34.04010105.0e090607.00000000</MGASADMOperationalMode>
        </MGASADMVirtualTrackParameterSet>"#;

    // ── Positive paths ───────────────────────────────────────────────────

    #[test]
    fn sadm_signal_sequence_with_matching_parameter_set_is_clean() {
        let xml = cpl_xml(SADM_SIGNAL_SEQ, SADM_PARAM_SET_OK);
        let cpl = parse_cpl(&xml).expect("CPL should parse");
        let issues = AppSadmPlugin2023.validate_cpl(&cpl);
        let sadm_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.code.starts_with("ST2067-203"))
            .collect();
        assert!(
            sadm_issues.is_empty(),
            "conformant S-ADM CPL should produce zero ST 2067-203 findings; got: {sadm_issues:#?}"
        );
    }

    // ── Negative paths ───────────────────────────────────────────────────

    #[test]
    fn sadm_signal_sequence_without_parameter_set_fires_missing() {
        // SignalSequence present, no matching ParameterSet.
        let xml = cpl_xml(SADM_SIGNAL_SEQ, "");
        let cpl = parse_cpl(&xml).expect("CPL should parse");
        let issues = AppSadmPlugin2023.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code == "ST2067-203:2023:5.4/VirtualTrackParameterSetMissing"),
            "expected VirtualTrackParameterSetMissing; got: {issues:#?}"
        );
    }

    #[test]
    fn sadm_orphaned_parameter_set_fires_orphaned() {
        // ParameterSet references a TrackId that no SignalSequence uses.
        let orphan = r#"
            <MGASADMVirtualTrackParameterSet>
                <Id>urn:uuid:00000000-0000-0000-0000-000000000102</Id>
                <TrackId>urn:uuid:ffffffff-ffff-ffff-ffff-ffffffffffff</TrackId>
                <MGASADMOperationalMode>urn:smpte:ul:060e2b34.04010105.0e090607.00000000</MGASADMOperationalMode>
            </MGASADMVirtualTrackParameterSet>"#;
        let xml = cpl_xml("", orphan);
        let cpl = parse_cpl(&xml).expect("CPL should parse");
        let issues = AppSadmPlugin2023.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code == "ST2067-203:2023:5.4/VirtualTrackParameterSetOrphaned"),
            "expected VirtualTrackParameterSetOrphaned; got: {issues:#?}"
        );
    }

    #[test]
    fn sadm_empty_operational_mode_fires_error() {
        let param_set_empty_mode = r#"
            <MGASADMVirtualTrackParameterSet>
                <Id>urn:uuid:00000000-0000-0000-0000-000000000103</Id>
                <TrackId>urn:uuid:00000000-0000-0000-0000-000000000a01</TrackId>
                <MGASADMOperationalMode></MGASADMOperationalMode>
            </MGASADMVirtualTrackParameterSet>"#;
        let xml = cpl_xml(SADM_SIGNAL_SEQ, param_set_empty_mode);
        let cpl = parse_cpl(&xml).expect("CPL should parse");
        let issues = AppSadmPlugin2023.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code == "ST2067-203:2023:5.4/OperationalModeEmpty"),
            "expected OperationalModeEmpty; got: {issues:#?}"
        );
    }

    #[test]
    fn sadm_soundfield_selector_with_unknown_resource_id_fires_orphan() {
        let param_set_bad_ref = r#"
            <MGASADMVirtualTrackParameterSet>
                <Id>urn:uuid:00000000-0000-0000-0000-000000000104</Id>
                <TrackId>urn:uuid:00000000-0000-0000-0000-000000000a01</TrackId>
                <MGASADMOperationalMode>urn:smpte:ul:060e2b34.04010105.0e090607.00000000</MGASADMOperationalMode>
                <MGASADMSoundfieldGroupSelector>
                    <ResourceId>urn:uuid:deadbeef-dead-beef-dead-beefdeadbeef</ResourceId>
                    <MGASoundfieldGroupLinkID>urn:uuid:00000000-0000-0000-0000-000000000c01</MGASoundfieldGroupLinkID>
                </MGASADMSoundfieldGroupSelector>
            </MGASADMVirtualTrackParameterSet>"#;
        let xml = cpl_xml(SADM_SIGNAL_SEQ, param_set_bad_ref);
        let cpl = parse_cpl(&xml).expect("CPL should parse");
        let issues = AppSadmPlugin2023.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code == "ST2067-203:2023:5.4/SoundfieldGroupSelectorResourceIdOrphaned"),
            "expected SoundfieldGroupSelectorResourceIdOrphaned; got: {issues:#?}"
        );
    }

    // ── ADM (ST 2067-204) — one positive + one negative to prove the
    // parallel path is wired ────────────────────────────────────────────

    const ADM_SIGNAL_SEQ: &str = r#"
        <ADMAudioSequence>
            <Id>urn:uuid:00000000-0000-0000-0000-000000000a04</Id>
            <TrackId>urn:uuid:00000000-0000-0000-0000-000000000b04</TrackId>
            <ResourceList>
                <Resource>
                    <Id>urn:uuid:00000000-0000-0000-0000-000000000c04</Id>
                    <IntrinsicDuration>48000</IntrinsicDuration>
                    <SourceEncoding>urn:uuid:00000000-0000-0000-0000-000000000d04</SourceEncoding>
                    <TrackFileId>urn:uuid:00000000-0000-0000-0000-000000000e04</TrackFileId>
                </Resource>
            </ResourceList>
        </ADMAudioSequence>"#;

    const ADM_PARAM_SET_OK: &str = r#"
        <ADMAudioVirtualTrackParameterSet>
            <Id>urn:uuid:00000000-0000-0000-0000-000000000f04</Id>
            <TrackId>urn:uuid:00000000-0000-0000-0000-000000000b04</TrackId>
            <ADMOperationalMode>urn:smpte:ul:060e2b34.04010105.0e090607.00000000</ADMOperationalMode>
        </ADMAudioVirtualTrackParameterSet>"#;

    #[test]
    fn adm_conformant_cpl_is_clean() {
        let xml = cpl_xml(ADM_SIGNAL_SEQ, ADM_PARAM_SET_OK);
        let cpl = parse_cpl(&xml).expect("CPL should parse");
        let issues = AppAdmPlugin2023.validate_cpl(&cpl);
        let adm_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.code.starts_with("ST2067-204"))
            .collect();
        assert!(
            adm_issues.is_empty(),
            "conformant ADM CPL should produce zero ST 2067-204 findings; got: {adm_issues:#?}"
        );
    }

    #[test]
    fn adm_missing_parameter_set_fires_missing() {
        let xml = cpl_xml(ADM_SIGNAL_SEQ, "");
        let cpl = parse_cpl(&xml).expect("CPL should parse");
        let issues = AppAdmPlugin2023.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code == "ST2067-204:2023:5.4/VirtualTrackParameterSetMissing"),
            "expected ADM VirtualTrackParameterSetMissing; got: {issues:#?}"
        );
    }

    // ── Dispatch integration ──────────────────────────────────────────────

    #[test]
    fn presence_of_signal_sequence_dispatches_the_plugin() {
        // Validating via `validate_cpl` (not via direct AppSadmPlugin2023
        // construction) should pick up ST 2067-203 findings.
        let xml = cpl_xml(SADM_SIGNAL_SEQ, "");
        let cpl = parse_cpl(&xml).expect("CPL should parse");
        let issues = crate::validation::validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.starts_with("ST2067-203")),
            "sequence-presence dispatch should pick up the S-ADM plug-in; got codes: {:?}",
            issues.iter().map(|i| i.code.as_str()).collect::<Vec<_>>()
        );
    }
}
