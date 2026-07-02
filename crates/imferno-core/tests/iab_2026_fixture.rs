//! Integration test for the ST 2067-201:2026 Annex E recommendation
//! using fixture files under `tests/fixtures/iab/`.
//!
//! The fixtures are documented in `tests/fixtures/iab/README.md` —
//! they are deliberately synthetic (no canonical SMPTE-published 2026
//! IAB CPL example exists yet, since the publication is from
//! 2026-03-25 with no bundled XML examples).
//!
//! When a real-world / SMPTE-RA-published 2026 IAB CPL becomes
//! available, swap the fixtures for the canonical source and keep
//! this test pinning the same contracts.

use imferno_core::cpl::parse_cpl;
use imferno_core::validation::{
    AppIabPlugin2019, AppIabPlugin2021, AppIabPlugin2026, ConstraintsValidator,
};

const CPL_CONFORMANT: &str = include_str!("fixtures/iab/cpl-iab-2026-conformant.xml");
const CPL_NON_CONFORMANT: &str =
    include_str!("fixtures/iab/cpl-iab-2026-missing-channel-subdescriptors.xml");

const ANNEX_E_CODE: &str = "ST2067-201:2026:5.10.2/IabChannelSubDescriptorRecommended";

#[test]
fn fixture_conformant_passes_under_2026_plugin() {
    let cpl = parse_cpl(CPL_CONFORMANT).expect("conformant fixture should parse");
    let issues = AppIabPlugin2026.validate_cpl(&cpl);
    assert!(
        !issues.iter().any(|i| i.code == ANNEX_E_CODE),
        "AppIabPlugin2026 should not flag Annex E on the conformant fixture, got: {issues:#?}"
    );
}

#[test]
fn fixture_non_conformant_fires_annex_e_warning_under_2026_plugin() {
    let cpl = parse_cpl(CPL_NON_CONFORMANT).expect("non-conformant fixture should parse");
    let issues = AppIabPlugin2026.validate_cpl(&cpl);
    assert!(
        issues.iter().any(|i| i.code == ANNEX_E_CODE),
        "AppIabPlugin2026 should flag Annex E on the non-conformant fixture, got: {issues:#?}"
    );
}

#[test]
fn fixture_non_conformant_silent_under_2021_plugin() {
    // The 2021 plugin must NOT emit the Annex E code — the rule
    // didn't exist before 2026.
    let cpl = parse_cpl(CPL_NON_CONFORMANT).expect("non-conformant fixture should parse");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        !issues.iter().any(|i| i.code == ANNEX_E_CODE),
        "AppIabPlugin2021 must not emit the 2026-only Annex E code"
    );
}

#[test]
fn fixture_non_conformant_silent_under_2019_plugin() {
    // Belt-and-braces: also verify the 2019 plugin doesn't fire.
    let cpl = parse_cpl(CPL_NON_CONFORMANT).expect("non-conformant fixture should parse");
    let issues = AppIabPlugin2019.validate_cpl(&cpl);
    assert!(
        !issues.iter().any(|i| i.code == ANNEX_E_CODE),
        "AppIabPlugin2019 must not emit the 2026-only Annex E code"
    );
}
