//! FIX-19: SCM parser regression against the canonical SMPTE-published
//! `st2067-9b-2018.xml` example bundled with ST 2067-9:2018.
//!
//! All other SCM tests use hand-rolled XML; this fixture catches drift
//! between our parser and the canonical-shape SMPTE document.

use imferno_core::scm::parse_scm;

const CANONICAL_SCM: &str = include_str!("fixtures/scm/st2067-9b-2018.xml");

#[test]
fn canonical_scm_example_parses() {
    let _ = parse_scm(CANONICAL_SCM).expect("canonical SCM example should parse");
}

#[test]
fn canonical_scm_example_has_expected_id_and_one_asset() {
    let scm = parse_scm(CANONICAL_SCM).expect("canonical SCM example should parse");
    assert_eq!(scm.id.to_string(), "144dbc24-62bf-611c-4fcc-a936759e31f7");
    assert_eq!(scm.issue_date, "2018-02-07T12:51:21+00:00");
    assert_eq!(
        scm.annotation.as_deref(),
        Some("Sidecar Composition Map Example")
    );

    assert_eq!(scm.sidecar_assets.len(), 1);
    assert_eq!(
        scm.sidecar_assets[0].id.to_string(),
        "32bae51e-0e70-ebaa-c643-77805f3c90f5"
    );
    assert_eq!(scm.sidecar_assets[0].cpl_ids.len(), 1);
    assert_eq!(
        scm.sidecar_assets[0].cpl_ids[0].to_string(),
        "d7ab9e8a-15b3-2e11-15d9-fc30b26e924e"
    );
}
