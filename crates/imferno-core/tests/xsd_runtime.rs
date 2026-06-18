//! Integration tests for the runtime-XSD validation path.
//!
//! Exercises `imferno_core::xsd::validate_against_schema` against real
//! SMPTE CPL fixtures + synthetic broken CPLs.
//!
//! Run with:
//!
//!     cargo test -p imferno-core --test xsd_runtime

use std::fs;
use std::path::{Path, PathBuf};

use imferno_core::cpl::parse_cpl;
use imferno_core::validation::validate_cpl;
use imferno_core::xsd::{
    validate_against_composite_schema, validate_against_schema, validate_cpl_xml,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .unwrap()
}

fn specs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("specs")
}

fn read_xsd(name: &str) -> String {
    fs::read_to_string(specs_dir().join(name)).unwrap_or_else(|e| panic!("read xsd {name}: {e}"))
}

fn list_fixtures(dir: &Path, prefix: &str) -> Vec<PathBuf> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(prefix) && n.ends_with(".xml"))
                .unwrap_or(false)
        })
        .collect();
    entries.sort();
    entries
}

/// All 20 real fixtures should report XSD-valid (any failures here
/// signal a regression in the runtime-XSD pipeline against real CPLs).
///
/// The fixtures all use CPL-typed elements whose types come from the
/// unresolved `dcml` import — uppsala lax-validates these, which is
/// the documented limitation (see `specs/comparisons/imf-cpl.md`).
#[test]
fn real_fixtures_validate_as_xsd_clean() {
    let schema_xml = read_xsd("imf-cpl.xsd");
    let fixture_dir = repo_root().join("test-data/Application2Extended");
    let fixtures = list_fixtures(&fixture_dir, "CPL_");
    assert!(!fixtures.is_empty(), "no CPL fixtures found");

    for path in &fixtures {
        let name = path.file_name().unwrap().to_string_lossy();
        let cpl_xml = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {name}: {e}"));
        let issues = validate_against_schema(&cpl_xml, &schema_xml, None);
        assert!(
            issues.is_empty(),
            "expected XSD-clean for {name}, got {} issues:\n{:#?}",
            issues.len(),
            issues
        );
    }
}

/// Synthetic deliberately-broken CPLs that exercise the translator's
/// classification logic against the production path.
#[test]
fn synthetic_broken_cpls_fire_expected_codes() {
    let schema_xml = read_xsd("imf-cpl.xsd");

    let missing_issue_date = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <ContentTitle>X</ContentTitle>
        <EditRate>24 1</EditRate>
        <SegmentList><Segment>
            <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
            <SequenceList/>
        </Segment></SegmentList>
    </CompositionPlaylist>"#;
    let issues = validate_against_schema(missing_issue_date, &schema_xml, None);
    assert!(
        !issues.is_empty(),
        "missing IssueDate must fire at least one issue"
    );
    assert!(
        issues
            .iter()
            .any(|i| i.code.starts_with("XSD/ElementMissing")),
        "missing IssueDate must classify as XSD/ElementMissing: {issues:#?}"
    );

    let bad_date = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <IssueDate>not-a-date</IssueDate>
        <ContentTitle>X</ContentTitle>
        <EditRate>24 1</EditRate>
        <SegmentList><Segment>
            <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
            <SequenceList/>
        </Segment></SegmentList>
    </CompositionPlaylist>"#;
    let issues = validate_against_schema(bad_date, &schema_xml, None);
    assert!(
        issues.iter().any(|i| i.code.starts_with("XSD/TypeInvalid")),
        "bad xs:dateTime must classify as XSD/TypeInvalid: {issues:#?}"
    );

    let out_of_order = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <EditRate>24 1</EditRate>
        <IssueDate>2025-01-01T00:00:00Z</IssueDate>
        <ContentTitle>X</ContentTitle>
        <SegmentList><Segment>
            <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
            <SequenceList/>
        </Segment></SegmentList>
    </CompositionPlaylist>"#;
    let issues = validate_against_schema(out_of_order, &schema_xml, None);
    assert!(
        !issues.is_empty(),
        "out-of-order elements must fire at least one issue"
    );
    // Out-of-order produces a mix of ElementMissing (expected here but found
    // elsewhere) and possibly UnexpectedElement — assert the bag is non-empty
    // and includes at least one of those two.
    assert!(
        issues.iter().any(|i| {
            i.code.starts_with("XSD/ElementMissing") || i.code.starts_with("XSD/UnexpectedElement")
        }),
        "out-of-order must classify as ElementMissing or UnexpectedElement: {issues:#?}"
    );
}

/// Composite-schema path: imports are resolved against vendored stubs
/// so elements typed against dcml: types actually get validated
/// instead of being silently lax-validated. This is what closes the
/// gap noted in `specs/comparisons/imf-cpl.md` for real CPLs.
#[test]
fn composite_schema_validates_real_fixture_with_dcml_types_bound() {
    let primary = specs_dir().join("imf-cpl.xsd");
    let specs = specs_dir();
    let fixture = repo_root()
        .join("test-data/Application2Extended/CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml");
    let cpl_xml = fs::read_to_string(&fixture).unwrap_or_else(|e| panic!("read fixture: {e}"));
    let issues = validate_against_composite_schema(&cpl_xml, &primary, &specs, None);
    // The well-formed BLACKL fixture should still be XSD-valid even
    // with dcml types bound — if it's not, our stub is mis-spec'd
    // or the fixture has a real bug.
    assert!(
        issues.is_empty(),
        "real fixture should validate under composite schema; got {} issues:\n{:#?}",
        issues.len(),
        issues
    );
}

/// Composite-schema path: built-in type violations still fire under
/// the composite path (built-in types are always validated regardless
/// of import-resolution state).
#[test]
fn composite_schema_still_catches_builtin_type_violations() {
    let primary = specs_dir().join("imf-cpl.xsd");
    let specs = specs_dir();
    let bad_date = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <IssueDate>not-a-date</IssueDate>
        <ContentTitle>X</ContentTitle>
        <EditRate>24 1</EditRate>
        <SegmentList><Segment>
            <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
            <SequenceList/>
        </Segment></SegmentList>
    </CompositionPlaylist>"#;
    let issues = validate_against_composite_schema(bad_date, &primary, &specs, None);
    assert!(
        issues.iter().any(|i| i.code.starts_with("XSD/TypeInvalid")),
        "bad xs:dateTime under composite must classify as TypeInvalid: {issues:#?}"
    );
}

/// **Known uppsala v0.4.0 limitation**: pattern/restriction facets on
/// types imported from another namespace are NOT applied during instance
/// validation, even when the import is resolved via the composite path.
///
/// Concretely: `<Id>not-a-uuid</Id>` against a schema typed
/// `dcml:UUIDType` (which restricts xs:anyURI with a UUID-URN pattern)
/// is silently accepted. The same value violating a built-in type
/// (e.g. xs:dateTime) IS caught.
///
/// This test is marked `#[ignore]` and pinned to the behavior as
/// observed. When uppsala lands imported-type-facet enforcement, this
/// test should be un-ignored and the assertion swapped to expect the
/// violation to fire.
/// Closed by uppsala fork patch 2 (xmlns-aware QName prefix resolution).
/// `dcml:UUIDType`'s URN pattern now fires for imported-namespace types
/// just like built-in types do — confirming SMPTE-spec-compliant
/// validation reaches every constraint expressible in the XSD layer.
#[test]
fn composite_schema_catches_dcml_typed_violations() {
    let primary = specs_dir().join("imf-cpl.xsd");
    let specs = specs_dir();
    let bad_uuid = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
        <Id>not-a-uuid</Id>
        <IssueDate>2025-01-01T00:00:00Z</IssueDate>
        <ContentTitle>X</ContentTitle>
        <EditRate>24 1</EditRate>
        <SegmentList><Segment>
            <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
            <SequenceList/>
        </Segment></SegmentList>
    </CompositionPlaylist>"#;
    let issues = validate_against_composite_schema(bad_uuid, &primary, &specs, None);
    assert!(
        issues.iter().any(|i| {
            i.code.starts_with("XSD/PatternInvalid") || i.code.starts_with("XSD/TypeInvalid")
        }),
        "bad UUIDType must classify as Pattern/TypeInvalid: {issues:#?}"
    );
}

/// Top-level wiring: `validate_cpl_xml` runs XSD + semantic in order,
/// chains the results. On a clean fixture both layers contribute zero
/// issues (or only known prose-only ones).
#[test]
fn validate_cpl_xml_runs_both_layers_clean_fixture() {
    let primary = specs_dir().join("imf-cpl.xsd");
    let specs = specs_dir();
    let fixture = repo_root()
        .join("test-data/Application2Extended/CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml");
    let cpl_xml = fs::read_to_string(&fixture).unwrap();
    let issues = validate_cpl_xml(&cpl_xml, &primary, &specs);
    // XSD layer should contribute 0 (clean fixture); semantic layer
    // may contribute some prose-only findings depending on the
    // catalogue's coverage of this fixture. Either way, no XSD/* codes.
    let xsd_codes: Vec<_> = issues
        .iter()
        .filter(|i| i.code.starts_with("XSD/"))
        .collect();
    assert!(
        xsd_codes.is_empty(),
        "clean fixture should produce no XSD/* issues; got:\n{xsd_codes:#?}"
    );
}

/// Top-level wiring: a CPL that fails both XSD (bad xs:dateTime) AND
/// semantics (whatever the catalogue catches on top) should produce a
/// concatenated list — XSD findings first, semantic after.
#[test]
fn validate_cpl_xml_chains_xsd_then_semantic() {
    let primary = specs_dir().join("imf-cpl.xsd");
    let specs = specs_dir();
    let bad_xml = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <IssueDate>not-a-date</IssueDate>
        <ContentTitle>X</ContentTitle>
        <EditRate>24 1</EditRate>
        <SegmentList><Segment>
            <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
            <SequenceList/>
        </Segment></SegmentList>
    </CompositionPlaylist>"#;
    let issues = validate_cpl_xml(bad_xml, &primary, &specs);
    assert!(
        issues.iter().any(|i| i.code.starts_with("XSD/TypeInvalid")),
        "should include XSD/TypeInvalid for bad date: {issues:#?}"
    );
    // Ordering: first XSD/*, then anything else (parse error or semantic)
    let first_non_xsd = issues.iter().position(|i| !i.code.starts_with("XSD/"));
    let last_xsd = issues.iter().rposition(|i| i.code.starts_with("XSD/"));
    if let (Some(fns), Some(lx)) = (first_non_xsd, last_xsd) {
        assert!(lx < fns, "XSD issues should precede non-XSD issues");
    }
}

/// Unified-path verification: parse a raw CPL with a deliberate XSD
/// violation, then call the regular `validate_cpl(&parsed)` semantic
/// entry point. The XSD pre-pass in validate_core_structure should
/// fire and emit XSD/* codes, proving callers don't need to use the
/// raw-XML entry point separately.
#[test]
fn parse_then_validate_runs_xsd_prepass_via_source_xml() {
    let bad_xml = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <IssueDate>not-a-date</IssueDate>
        <ContentTitle>X</ContentTitle>
        <EditRate>24 1</EditRate>
        <SegmentList><Segment>
            <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
            <SequenceList/>
        </Segment></SegmentList>
    </CompositionPlaylist>"#;
    let cpl = parse_cpl(bad_xml).expect("should parse");
    assert!(
        cpl.source_xml.is_some(),
        "parse_cpl must populate source_xml"
    );

    let issues = validate_cpl(&cpl);
    let xsd_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.code.starts_with("XSD/"))
        .collect();
    assert!(
        !xsd_issues.is_empty(),
        "validate_cpl should now emit XSD/* codes via the runtime-XSD pre-pass; got: {:#?}",
        issues
    );
    assert!(
        xsd_issues
            .iter()
            .any(|i| i.code.starts_with("XSD/TypeInvalid")),
        "bad xs:dateTime should fire XSD/TypeInvalid: {xsd_issues:#?}"
    );
}

/// The message body should carry uppsala's line/column for any
/// diagnostic that has them — needed so operators can locate the
/// violation without re-running validation.
#[test]
fn diagnostic_messages_include_position_when_available() {
    let schema_xml = read_xsd("imf-cpl.xsd");
    let bad_date = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <IssueDate>not-a-date</IssueDate>
        <ContentTitle>X</ContentTitle>
        <EditRate>24 1</EditRate>
        <SegmentList><Segment>
            <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
            <SequenceList/>
        </Segment></SegmentList>
    </CompositionPlaylist>"#;
    let issues = validate_against_schema(bad_date, &schema_xml, None);
    let type_invalid = issues
        .iter()
        .find(|i| i.code.starts_with("XSD/TypeInvalid"))
        .expect("expected an XSD/TypeInvalid issue");
    assert!(
        type_invalid.message.contains("line") && type_invalid.message.contains("column"),
        "message should include line+column: {}",
        type_invalid.message
    );
}
