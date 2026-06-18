//! Spike: uppsala XSD validation, parallel to `xsd_validate_spike.rs`.
//!
//! Same shape (20 real fixtures + 3 synthetic broken CPLs + no-import
//! boundary probe) so the two crates can be compared head-to-head.
//!
//! Run from the imferno repo root:
//!
//!     cargo run --example xsd_validate_spike_uppsala -p imferno-core
//!
//! What this validates about the architecture:
//! - uppsala's XSD 1.1 validator handles a real SMPTE XSD without crashing
//! - Diagnostic shape is workable for a translator layer
//! - How uppsala treats `xs:import` without `schemaLocation` (same gap
//!   as xmloxide per source inspection — expected outcome: silently
//!   skips the import, lax-validates the referencing namespaces)
//! - Whether XSD 1.1 features (e.g. xs:assertion) are functional

use std::fs;
use std::path::PathBuf;

use uppsala::{parse, XsdValidator};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .unwrap()
}

fn main() {
    let root = repo_root();
    let xsd_path = root.join("crates/imferno-core/specs/imf-cpl.xsd");
    let fixture_dir = root.join("test-data/Application2Extended");

    println!("== uppsala XSD validation spike ==");
    println!("XSD: {}", xsd_path.display());

    let xsd_src = fs::read_to_string(&xsd_path).unwrap_or_else(|e| panic!("read xsd: {e}"));

    let schema_doc = parse(&xsd_src).unwrap_or_else(|e| panic!("parse schema xml: {e:?}"));
    let validator = XsdValidator::from_schema(&schema_doc)
        .unwrap_or_else(|e| panic!("XsdValidator::from_schema: {e:?}"));
    println!("schema parsed + validator built ok\n");

    let mut entries: Vec<_> = fs::read_dir(&fixture_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("CPL_") && n.ends_with(".xml"))
                .unwrap_or(false)
        })
        .collect();
    entries.sort();

    let mut tally_valid = 0;
    let mut tally_invalid = 0;
    let mut first_failure_dumped = false;

    for path in &entries {
        let name = path.file_name().unwrap().to_string_lossy();
        let src = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                println!("[read-error]      {name} -- {e}");
                continue;
            }
        };
        let instance_doc = match parse(&src) {
            Ok(d) => d,
            Err(e) => {
                println!("[xml-parse-error] {name} -- {e:?}");
                continue;
            }
        };
        let errors = validator.validate(&instance_doc);
        if errors.is_empty() {
            tally_valid += 1;
            println!("[xsd-valid]       {name}  (0 errors)");
        } else {
            tally_invalid += 1;
            println!("[xsd-INVALID]     {name}  ({} errors)", errors.len());
            if !first_failure_dumped {
                println!("\n  -- first failure errors (up to 5) --");
                for e in errors.iter().take(5) {
                    println!("    {:#?}", e);
                }
                println!("  -- end dump --\n");
                first_failure_dumped = true;
            }
        }
    }

    println!(
        "\ntotal: {} fixtures  ({} XSD-valid, {} XSD-invalid)",
        entries.len(),
        tally_valid,
        tally_invalid
    );

    println!("\n== synthetic deliberately-invalid CPLs (exercise diagnostic shape) ==");

    let cases: &[(&str, &str)] = &[
        (
            "missing-IssueDate",
            r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
                <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
                <ContentTitle>X</ContentTitle>
                <EditRate>24 1</EditRate>
                <SegmentList><Segment>
                    <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
                    <SequenceList/>
                </Segment></SegmentList>
            </CompositionPlaylist>"#,
        ),
        (
            "bad-IssueDate-format",
            r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
                <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
                <IssueDate>not-a-date</IssueDate>
                <ContentTitle>X</ContentTitle>
                <EditRate>24 1</EditRate>
                <SegmentList><Segment>
                    <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
                    <SequenceList/>
                </Segment></SegmentList>
            </CompositionPlaylist>"#,
        ),
        (
            "out-of-order-elements",
            r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
                <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
                <EditRate>24 1</EditRate>
                <IssueDate>2025-01-01T00:00:00Z</IssueDate>
                <ContentTitle>X</ContentTitle>
                <SegmentList><Segment>
                    <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
                    <SequenceList/>
                </Segment></SegmentList>
            </CompositionPlaylist>"#,
        ),
    ];

    for (label, xml) in cases {
        println!("\n--- {label} ---");
        match parse(xml) {
            Ok(doc) => {
                let errors = validator.validate(&doc);
                println!("  errors: {}", errors.len());
                if !errors.is_empty() {
                    println!("  first: {:#?}", errors[0]);
                }
            }
            Err(e) => println!("  XML parse failed: {e:?}"),
        }
    }

    println!("\n== boundary probe with self-contained no-import schema ==");
    let mini_xsd = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
      <xs:element name="thing">
        <xs:complexType>
          <xs:sequence>
            <xs:element name="name" type="xs:string"/>
            <xs:element name="count" type="xs:positiveInteger"/>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
    </xs:schema>"#;
    let mini_schema_doc = parse(mini_xsd).unwrap();
    let mini_validator = XsdValidator::from_schema(&mini_schema_doc).unwrap();
    let probes: &[(&str, &str)] = &[
        ("valid", "<thing><name>x</name><count>5</count></thing>"),
        ("missing-required", "<thing><name>x</name></thing>"),
        (
            "wrong-element-order",
            "<thing><count>5</count><name>x</name></thing>",
        ),
        (
            "invalid-type",
            "<thing><name>x</name><count>not-a-number</count></thing>",
        ),
        (
            "negative-positive",
            "<thing><name>x</name><count>-1</count></thing>",
        ),
        (
            "unknown-element",
            "<thing><name>x</name><count>5</count><unknown/></thing>",
        ),
    ];
    for (label, xml) in probes {
        let doc = parse(xml).unwrap();
        let errors = mini_validator.validate(&doc);
        println!("  {:<22} {} error(s)", label, errors.len());
        if !errors.is_empty() {
            println!("      → {:?}", errors[0]);
        }
    }
}
