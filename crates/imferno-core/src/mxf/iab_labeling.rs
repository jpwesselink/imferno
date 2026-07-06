//! ST 2067-201 §5.10 / Annex C IAB labeling rules, evaluated against the
//! RegXML produced by `mxf::metadata::parse_mxf_to_regxml` (AUDIT-18).
//!
//! Three rules, all scoped to MXF files whose header metadata carries an
//! `IABEssenceDescriptor` (audio_mca.rs deliberately skips these files, so
//! this module is the IAB counterpart of its §5.3.6 checks):
//!
//! - **§5.10.2** — "An IAB Track File shall not contain instances of
//!   AudioChannelLabelSubDescriptor, SoundfieldGroupLabelSubDescriptor, or
//!   GroupOfSoundfieldGroupsLabelSubDescriptor."
//! - **§5.10.2** — "An IAB Track File shall contain exactly one instance of
//!   an IAB Soundfield Label SubDescriptor" — both the missing and the
//!   duplicate direction (the CPL-level validator can only see the missing
//!   direction: the EssenceDescriptorList model holds an `Option`).
//! - **Annex C.2** — "MCA Channel ID shall not be present in the IAB
//!   Soundfield Label SubDescriptor." Since §5.10.2 already bans every
//!   sub-descriptor type that could legitimately carry an MCAChannelID, any
//!   occurrence in an IAB file violates Annex C Table C.1.
//!
//! Codes are emitted with the ST 2067-201:2021 prefix: the 2019 and 2021
//! catalogues are bit-identical (verified snapshot diff), and at MXF level
//! there is no CPL ApplicationIdentification to select an edition from.
//!
//! Native-only (same constraint as other `mxf::*` essence modules).

use std::path::Path;

use crate::diagnostics::{Category, Location, Severity, ValidationIssue};
use crate::mxf::audio_mca::count_elements;
use crate::validation::iab_codes::{IabCode, St2067_201_2021};

/// Apply the §5.10.2 / Annex C.2 IAB labeling rules. Returns an empty Vec
/// when no `IABEssenceDescriptor` is present (the file is not IAB essence).
pub fn check_iab_labeling(regxml: &str, path: &Path) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if !regxml.contains("IABEssenceDescriptor") {
        return issues;
    }

    let code = St2067_201_2021::for_code;
    let loc = || Location::new().with_file(path.to_path_buf());

    // §5.10.2 — plain MCA label sub-descriptors are prohibited. The three
    // element names do not collide as substrings of each other or of
    // IABSoundfieldLabelSubDescriptor, and count_elements matches on
    // `:<LocalName>` boundaries.
    for forbidden in [
        "AudioChannelLabelSubDescriptor",
        "SoundfieldGroupLabelSubDescriptor",
        "GroupOfSoundfieldGroupsLabelSubDescriptor",
    ] {
        let n = count_elements(regxml, forbidden);
        if n > 0 {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Audio,
                    code(IabCode::ForbiddenMCASubDescriptor),
                    format!(
                        "MXF {} IAB Track File carries {n} {forbidden} instance(s) — \
                         ST 2067-201 §5.10.2 prohibits AudioChannelLabel, SoundfieldGroupLabel \
                         and GroupOfSoundfieldGroupsLabel SubDescriptors in IAB Track Files.",
                        path.display(),
                    ),
                )
                .with_location(loc()),
            );
        }
    }

    // §5.10.2 — exactly one IABSoundfieldLabelSubDescriptor.
    let sfl_count = count_elements(regxml, "IABSoundfieldLabelSubDescriptor");
    match sfl_count {
        0 => {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Audio,
                    code(IabCode::SubDescriptorMissing),
                    format!(
                        "MXF {} IAB Track File carries no IABSoundfieldLabelSubDescriptor — \
                         ST 2067-201 §5.10.2 requires exactly one instance.",
                        path.display(),
                    ),
                )
                .with_location(loc()),
            );
        }
        1 => {}
        n => {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Audio,
                    code(IabCode::SubDescriptorDuplicate),
                    format!(
                        "MXF {} IAB Track File carries {n} IABSoundfieldLabelSubDescriptor \
                         instances — ST 2067-201 §5.10.2 requires exactly one.",
                        path.display(),
                    ),
                )
                .with_location(loc()),
            );
        }
    }

    // Annex C.2 — MCAChannelID shall not be present in the IAB Soundfield
    // Label SubDescriptor (Table C.1 excludes the item). With §5.10.2
    // banning every other MCA sub-descriptor type, any MCAChannelID in an
    // IAB file is a violation.
    let mca_channel_ids = count_elements(regxml, "MCAChannelID");
    if mca_channel_ids > 0 {
        issues.push(
            ValidationIssue::new(
                Severity::Error,
                Category::Audio,
                code(IabCode::MCAChannelIDForbidden),
                format!(
                    "MXF {} IAB Track File carries {mca_channel_ids} MCAChannelID item(s) — \
                     ST 2067-201 Annex C.2 excludes MCAChannelID from the IAB Soundfield \
                     Label SubDescriptor.",
                    path.display(),
                ),
            )
            .with_location(loc()),
        );
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_non_iab_mxf() {
        let xml = r#"<ns1:Preface><ns1:WAVEPCMDescriptor>
            <ns2:MCAChannelID>1</ns2:MCAChannelID>
        </ns1:WAVEPCMDescriptor></ns1:Preface>"#;
        let issues = check_iab_labeling(xml, Path::new("/synth.mxf"));
        assert!(
            issues.is_empty(),
            "IAB labeling rules must be silent on non-IAB MXF, got: {issues:#?}"
        );
    }

    #[test]
    fn clean_iab_file_passes() {
        let xml = r#"<ns1:Preface><ns1:IABEssenceDescriptor>
            <ns2:SubDescriptors>
                <ns1:IABSoundfieldLabelSubDescriptor>
                    <ns2:MCATagSymbol>IAB</ns2:MCATagSymbol>
                </ns1:IABSoundfieldLabelSubDescriptor>
            </ns2:SubDescriptors>
        </ns1:IABEssenceDescriptor></ns1:Preface>"#;
        let issues = check_iab_labeling(xml, Path::new("/iab.mxf"));
        assert!(
            issues.is_empty(),
            "conformant IAB labeling must produce no issues, got: {issues:#?}"
        );
    }

    #[test]
    fn flags_forbidden_plain_mca_subdescriptors() {
        let xml = r#"<ns1:Preface><ns1:IABEssenceDescriptor>
            <ns2:SubDescriptors>
                <ns1:IABSoundfieldLabelSubDescriptor/>
                <ns1:AudioChannelLabelSubDescriptor/>
                <ns1:SoundfieldGroupLabelSubDescriptor/>
                <ns1:GroupOfSoundfieldGroupsLabelSubDescriptor/>
            </ns2:SubDescriptors>
        </ns1:IABEssenceDescriptor></ns1:Preface>"#;
        let issues = check_iab_labeling(xml, Path::new("/iab.mxf"));
        let hits = issues
            .iter()
            .filter(|i| i.code.contains("5.10.2/ForbiddenMCASubDescriptor"))
            .count();
        assert_eq!(
            hits, 3,
            "each of the three prohibited sub-descriptor types must fire: {issues:#?}"
        );
    }

    #[test]
    fn flags_missing_and_duplicate_soundfield_label() {
        let missing = r#"<ns1:Preface><ns1:IABEssenceDescriptor>
            <ns2:SubDescriptors/>
        </ns1:IABEssenceDescriptor></ns1:Preface>"#;
        let issues = check_iab_labeling(missing, Path::new("/iab.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("5.10.2/SubDescriptorMissing")),
            "zero IABSoundfieldLabelSubDescriptor must fire SubDescriptorMissing: {issues:#?}"
        );

        let duplicate = r#"<ns1:Preface><ns1:IABEssenceDescriptor>
            <ns2:SubDescriptors>
                <ns1:IABSoundfieldLabelSubDescriptor/>
                <ns1:IABSoundfieldLabelSubDescriptor/>
            </ns2:SubDescriptors>
        </ns1:IABEssenceDescriptor></ns1:Preface>"#;
        let issues = check_iab_labeling(duplicate, Path::new("/iab.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("5.10.2/SubDescriptorDuplicate")),
            "two IABSoundfieldLabelSubDescriptors must fire SubDescriptorDuplicate: {issues:#?}"
        );
    }

    #[test]
    fn flags_mca_channel_id_in_iab_soundfield_label() {
        let xml = r#"<ns1:Preface><ns1:IABEssenceDescriptor>
            <ns2:SubDescriptors>
                <ns1:IABSoundfieldLabelSubDescriptor>
                    <ns2:MCAChannelID>1</ns2:MCAChannelID>
                </ns1:IABSoundfieldLabelSubDescriptor>
            </ns2:SubDescriptors>
        </ns1:IABEssenceDescriptor></ns1:Preface>"#;
        let issues = check_iab_labeling(xml, Path::new("/iab.mxf"));
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("C.2/MCAChannelIDForbidden")),
            "MCAChannelID inside the IAB SFL sub-descriptor must fire Annex C.2: {issues:#?}"
        );
    }
}
