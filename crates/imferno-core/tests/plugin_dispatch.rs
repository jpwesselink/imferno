//! Regression tests for IAB / ISXD plug-in dispatch.
//!
//! Before these plugins were registered, `AppIabPlugin*` and
//! `AppIsxdPlugin2022` were only reachable by direct construction —
//! `validate_cpl` / `imferno validate` never ran them. Real-world CPLs
//! signal plug-in usage two ways:
//!
//!   1. an `ApplicationIdentification` URI (`.../ns/2067-201/2019`,
//!      `.../ns/2067-202/2022`), and/or
//!   2. the presence of `iab:IABSequence` / `isxd:ISXDSequence` elements
//!      in segment sequence lists (the corpus fixtures use ONLY this
//!      form — their ApplicationIdentification names just the App2E
//!      profile).
//!
//! Both paths must resolve the plug-in.

use std::path::PathBuf;

use imferno_core::cpl::parse_cpl;
use imferno_core::validation::{
    get_validator, get_validators_for_cpl, ConfigurableValidatorRegistry, ValidatorRegistry,
    ValidatorSelection, URI_2019, URI_2019_SCHEMAS, URI_2022,
};

fn read_fixture(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// ── URI-based resolution ─────────────────────────────────────────────────────

#[test]
fn iab_namespace_uris_resolve_to_iab_plugin() {
    for uri in [URI_2019, URI_2019_SCHEMAS] {
        let v = get_validator(uri)
            .unwrap_or_else(|| panic!("IAB namespace {uri} must resolve to a validator"));
        assert!(
            v.spec_id().contains("2067-201"),
            "expected an ST 2067-201 validator for {uri}, got {}",
            v.spec_id()
        );
    }
}

#[test]
fn isxd_namespace_uri_resolves_to_isxd_plugin() {
    let v = get_validator(URI_2022).expect("ISXD namespace must resolve to a validator");
    assert!(
        v.spec_id().contains("2067-202"),
        "expected an ST 2067-202 validator, got {}",
        v.spec_id()
    );
}

// ── Sequence-presence dispatch (the corpus-fixture reality) ─────────────────

#[test]
fn iab_complete_imp_cpl_dispatches_iab_plugin() {
    // This CPL's ApplicationIdentification names ONLY App2E
    // (`.../schemas/2067-21/2016`); IAB usage is signaled by the
    // iab:IABSequence elements. Dispatch must still select the IAB
    // plug-in.
    let xml =
        read_fixture("test-data/IAB/CompleteIMP/CPL_e0265fda-cb35-4e35-a4e4-4f44d82d2a52.xml");
    let cpl = parse_cpl(&xml).expect("IAB CompleteIMP CPL should parse");
    let validators = get_validators_for_cpl(&cpl);
    let ids: Vec<&str> = validators.iter().map(|v| v.spec_id()).collect();
    assert!(
        ids.iter().any(|id| id.contains("2067-201")),
        "IAB CPL must dispatch the ST 2067-201 plug-in; got validators: {ids:?}"
    );
    // The App2E profile named in ApplicationIdentification must ALSO
    // still be selected — the plug-in runs on top of it, not instead.
    assert!(
        ids.iter().any(|id| id.contains("2067-21")),
        "IAB CPL must still dispatch App2E; got validators: {ids:?}"
    );
}

#[test]
fn isxd_complete_imp_cpl_dispatches_isxd_plugin() {
    let xml = read_fixture("test-data/ISXD/CompleteIMP/CPL_ISXD_TEST_1.xml");
    let cpl = parse_cpl(&xml).expect("ISXD CompleteIMP CPL should parse");
    let validators = get_validators_for_cpl(&cpl);
    let ids: Vec<&str> = validators.iter().map(|v| v.spec_id()).collect();
    assert!(
        ids.iter().any(|id| id.contains("2067-202")),
        "ISXD CPL must dispatch the ST 2067-202 plug-in; got validators: {ids:?}"
    );
}

#[test]
fn plain_app2e_cpl_does_not_dispatch_plugins() {
    // Negative control: a CPL with no IAB/ISXD sequences must not get
    // plug-in validators.
    let xml = read_fixture(
        "test-data/MERIDIAN_Netflix_Photon_161006/CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml",
    );
    let cpl = parse_cpl(&xml).expect("MERIDIAN CPL should parse");
    let validators = get_validators_for_cpl(&cpl);
    let ids: Vec<&str> = validators.iter().map(|v| v.spec_id()).collect();
    assert!(
        !ids.iter()
            .any(|id| id.contains("2067-201") || id.contains("2067-202")),
        "plain CPL must not dispatch IAB/ISXD plug-ins; got validators: {ids:?}"
    );
}

#[test]
fn uri_and_sequence_presence_dedupe_to_one_iab_validator() {
    // A CPL that BOTH declares the IAB URI in ApplicationIdentification
    // AND carries IABSequence elements gets exactly one IAB validator.
    let xml = read_fixture("test-data/IAB/CompleteIMP/CPL_e0265fda-cb35-4e35-a4e4-4f44d82d2a52.xml")
        .replace(
            "http://www.smpte-ra.org/schemas/2067-21/2016</cc:ApplicationIdentification>",
            "http://www.smpte-ra.org/schemas/2067-21/2016 http://www.smpte-ra.org/ns/2067-201/2019</cc:ApplicationIdentification>",
        );
    let cpl = parse_cpl(&xml).expect("modified IAB CPL should parse");
    let validators = get_validators_for_cpl(&cpl);
    let iab_count = validators
        .iter()
        .filter(|v| v.spec_id().contains("2067-201"))
        .count();
    assert_eq!(
        iab_count, 1,
        "URI + sequence-presence must dedupe to exactly one IAB validator; got {iab_count}"
    );
}

// ── Pinned-selection semantics ───────────────────────────────────────────────

#[test]
fn pinned_app_uris_suppress_sequence_presence_auto_add() {
    // When the caller explicitly pins application URIs, auto plug-in
    // dispatch must respect the pin — even if the CPL carries IAB
    // sequences.
    let xml =
        read_fixture("test-data/IAB/CompleteIMP/CPL_e0265fda-cb35-4e35-a4e4-4f44d82d2a52.xml");
    let cpl = parse_cpl(&xml).expect("IAB CompleteIMP CPL should parse");
    let registry = ConfigurableValidatorRegistry::new(ValidatorSelection {
        application_identification_uris: Some(vec![
            "http://www.smpte-ra.org/schemas/2067-21/2016".to_string()
        ]),
        ..Default::default()
    });
    let validators = registry.resolve_for_cpl(&cpl);
    let ids: Vec<&str> = validators.iter().map(|v| v.spec_id()).collect();
    assert!(
        !ids.iter().any(|id| id.contains("2067-201")),
        "pinned app selection must suppress auto plug-in dispatch; got: {ids:?}"
    );
}
