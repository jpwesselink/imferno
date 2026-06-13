//! FIX-11 regression: pin `IssueSource::from_code` against every typed
//! code enum's `ALL` const so no future code-prefix change silently
//! reroutes a rule to the wrong inferred source.
//!
//! Inference contract (from `diagnostics::mod.rs`):
//! - `XSD/...`               → `XsdLayer`
//! - `IMFERNO:...` / `IMFERNO/...` → `EngineInternal`
//! - everything else         → `ProseRule`
//!
//! Each typed code module is enumerated below. Adding a new code-enum
//! to the workspace should mean adding a new block here.

use imferno_core::assetmap::codes::{
    St2067_2_2013_Core, St2067_2_2016_Core, St2067_2_2020, St2067_2_2020_Core,
};
use imferno_core::assetmap::volindex_codes::St429_9_2014;
use imferno_core::cpl::codes::{St2067_3_2013, St2067_3_2016};
use imferno_core::diagnostics::codes::ValidationCode;
use imferno_core::diagnostics::IssueSource;
use imferno_core::mxf::codes::{
    ImfernoMxf, St2067_2_2016 as Mxf_St2067_2_2016, St377_1_2011, St377_4_2012,
};
use imferno_core::package::codes::ImfernoCode;
use imferno_core::scm::codes::St2067_9_2018;
use imferno_core::validation::codes::{St2067_21_2020, St2067_21_2023, St2067_21_2025};
use imferno_core::validation::iab_codes::{St2067_201_2019, St2067_201_2021, St2067_201_2026Delta};
use imferno_core::validation::isxd_codes::St2067_202_2022;
use imferno_core::xsd::codes::XsdConstraintCode;

fn check_all<C: ValidationCode>(codes: &[C], expected: IssueSource) {
    for c in codes {
        let code_str = c.code();
        let inferred = IssueSource::from_code(code_str);
        assert_eq!(
            inferred, expected,
            "code `{code_str}` inferred to {inferred:?}, expected {expected:?}"
        );
    }
}

/// Every code in `codes` returns `Some(...)` from `example()` — catches
/// new variants added without a populated example. Catalogue item 6.
fn check_all_have_examples<C: ValidationCode>(codes: &[C]) {
    let missing: Vec<&'static str> = codes
        .iter()
        .filter(|c| c.example().is_none())
        .map(|c| c.code())
        .collect();
    assert!(
        missing.is_empty(),
        "the following codes have no example() — populate before merging: {missing:#?}"
    );
}

#[test]
fn xsd_codes_infer_as_xsd_layer() {
    check_all(XsdConstraintCode::ALL, IssueSource::XsdLayer);
}

#[test]
fn imferno_internal_codes_infer_as_engine_internal() {
    check_all(ImfernoCode::ALL, IssueSource::EngineInternal);
    check_all(ImfernoMxf::ALL, IssueSource::EngineInternal);
}

/// Cross-edition annotation: when an enum's code set is bit-for-bit
/// identical to its predecessor (verified by snapshot diff), it
/// returns the predecessor's prefix from `previous_identical_edition`.
/// Currently three pairs: ST 2067-2:2013↔2016, ST 2067-3:2013↔2016,
/// ST 2067-201:2019↔2021. All other enums default to `None`.
#[test]
fn previous_identical_edition_is_set_for_known_identical_pairs() {
    let c1 = St2067_2_2016_Core::ALL[0];
    assert_eq!(c1.previous_identical_edition(), Some("ST2067-2:2013"));

    let c2 = St2067_3_2016::ALL[0];
    assert_eq!(c2.previous_identical_edition(), Some("ST2067-3:2013"));

    let c3 = St2067_201_2021::ALL[0];
    assert_eq!(c3.previous_identical_edition(), Some("ST2067-201:2019"));

    // Sanity: the older editions don't claim a predecessor.
    let c4 = St2067_2_2013_Core::ALL[0];
    assert_eq!(c4.previous_identical_edition(), None);
    let c5 = St2067_3_2013::ALL[0];
    assert_eq!(c5.previous_identical_edition(), None);
    let c6 = St2067_201_2019::ALL[0];
    assert_eq!(c6.previous_identical_edition(), None);
}

/// Catalogue item 6: every code in every enum populates `example()`.
/// Defaults-to-None would be silently dropped by the catalogue
/// renderer; this pins the coverage so a new variant without an
/// example trips the build.
#[test]
fn every_code_has_a_populated_example() {
    check_all_have_examples(XsdConstraintCode::ALL);
    check_all_have_examples(ImfernoCode::ALL);
    check_all_have_examples(ImfernoMxf::ALL);
    check_all_have_examples(St429_9_2014::ALL);
    check_all_have_examples(St377_1_2011::ALL);
    check_all_have_examples(St377_4_2012::ALL);
    check_all_have_examples(Mxf_St2067_2_2016::ALL);
    check_all_have_examples(St2067_2_2020::ALL);
    check_all_have_examples(St2067_2_2013_Core::ALL);
    check_all_have_examples(St2067_2_2016_Core::ALL);
    check_all_have_examples(St2067_2_2020_Core::ALL);
    check_all_have_examples(St2067_3_2013::ALL);
    check_all_have_examples(St2067_3_2016::ALL);
    check_all_have_examples(St2067_9_2018::ALL);
    check_all_have_examples(St2067_21_2020::ALL);
    check_all_have_examples(St2067_21_2023::ALL);
    check_all_have_examples(St2067_21_2025::ALL);
    check_all_have_examples(St2067_201_2019::ALL);
    check_all_have_examples(St2067_201_2021::ALL);
    check_all_have_examples(St2067_201_2026Delta::ALL);
    check_all_have_examples(St2067_202_2022::ALL);
}

#[test]
fn spec_codes_infer_as_prose_rule() {
    check_all(St429_9_2014::ALL, IssueSource::ProseRule);
    check_all(St377_1_2011::ALL, IssueSource::ProseRule);
    check_all(St377_4_2012::ALL, IssueSource::ProseRule);
    check_all(Mxf_St2067_2_2016::ALL, IssueSource::ProseRule);
    check_all(St2067_2_2020::ALL, IssueSource::ProseRule);
    check_all(St2067_2_2013_Core::ALL, IssueSource::ProseRule);
    check_all(St2067_2_2016_Core::ALL, IssueSource::ProseRule);
    check_all(St2067_2_2020_Core::ALL, IssueSource::ProseRule);
    check_all(St2067_3_2013::ALL, IssueSource::ProseRule);
    check_all(St2067_3_2016::ALL, IssueSource::ProseRule);
    check_all(St2067_9_2018::ALL, IssueSource::ProseRule);
    check_all(St2067_21_2020::ALL, IssueSource::ProseRule);
    check_all(St2067_21_2023::ALL, IssueSource::ProseRule);
    check_all(St2067_21_2025::ALL, IssueSource::ProseRule);
    check_all(St2067_201_2019::ALL, IssueSource::ProseRule);
    check_all(St2067_201_2021::ALL, IssueSource::ProseRule);
    check_all(St2067_201_2026Delta::ALL, IssueSource::ProseRule);
    check_all(St2067_202_2022::ALL, IssueSource::ProseRule);
}
