//! Spike: prove that xmloxide's pure-Rust XSD validator handles a real
//! SMPTE CPL against the vendored ST 2067-3:2013 schema.
//!
//! Run from the imferno repo root:
//!
//!     cargo run --example xsd_validate_spike -p imferno-core
//!
//! What this validates about the architecture:
//! - xmloxide's `parse_xsd` accepts a real SMPTE XSD (not just toy schemas)
//! - `validate_xsd` produces a structured ValidationResult against a real CPL
//! - Diagnostics include enough metadata (element path, expected/found,
//!   line/col) to drive a translator layer
//! - The dcml import (an `xs:import` with no `schemaLocation`) doesn't
//!   blow up parsing — we either get lax validation past the boundary
//!   or a clear "unresolved namespace" diagnostic

use std::fs;
use std::path::PathBuf;

use xmloxide::Document;
use xmloxide::validation::xsd::{
    parse_xsd, parse_xsd_with_options, validate_xsd, SchemaResolver, XsdParseOptions,
};

/// Minimal synthetic stub for SMPTE ST 433 dcml types — provides the three
/// types every IMF schema imports (`UUIDType`, `UserTextType`, `RationalType`)
/// matching the shapes the PKL XSD declares locally (`SMPTE-429-8-PKL-2007.xsd`).
/// Not a full ST 433 dcml-types schema; sufficient to satisfy CPL/PKL imports
/// for spike-level XSD validation.
const DCML_STUB_XSD: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
           xmlns:dcml="http://www.smpte-ra.org/schemas/433/2008/dcmlTypes/"
           targetNamespace="http://www.smpte-ra.org/schemas/433/2008/dcmlTypes/"
           elementFormDefault="qualified">
  <xs:simpleType name="UUIDType">
    <xs:restriction base="xs:anyURI">
      <xs:pattern value="urn:uuid:[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"/>
    </xs:restriction>
  </xs:simpleType>
  <xs:complexType name="UserTextType">
    <xs:simpleContent>
      <xs:extension base="xs:string">
        <xs:attribute name="language" type="xs:language" use="optional" default="en"/>
      </xs:extension>
    </xs:simpleContent>
  </xs:complexType>
  <xs:simpleType name="RationalType">
    <xs:restriction>
      <xs:simpleType>
        <xs:list itemType="xs:integer"/>
      </xs:simpleType>
      <xs:length value="2"/>
    </xs:restriction>
  </xs:simpleType>
</xs:schema>"#;

/// Stub resolver that hands xmloxide the synthetic dcml types when it asks
/// for the ST 433 namespace; returns None for anything else.
struct StubResolver;
impl SchemaResolver for StubResolver {
    fn resolve(&self, uri: &str, _base: Option<&str>) -> Option<String> {
        eprintln!("[resolver] xmloxide requested: {uri}");
        if uri.contains("433/2008/dcmlTypes") {
            Some(DCML_STUB_XSD.to_string())
        } else {
            None
        }
    }
}

fn repo_root() -> PathBuf {
    // crates/imferno-core/examples/xsd_validate_spike.rs → repo root is 3 up
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("..").join("..").canonicalize().unwrap()
}

fn main() {
    let root = repo_root();
    let xsd_path = root.join("specs/imf-cpl.xsd");
    let fixture_dir = root.join("test-data/Application2Extended");

    println!("== xmloxide XSD validation spike ==");
    println!("XSD: {}", xsd_path.display());

    let xsd_src = fs::read_to_string(&xsd_path)
        .unwrap_or_else(|e| panic!("read xsd: {e}"));

    let resolver = StubResolver;
    let opts = XsdParseOptions {
        resolver: Some(&resolver),
        base_uri: None,
    };
    let schema = parse_xsd_with_options(&xsd_src, &opts)
        .unwrap_or_else(|e| panic!("parse_xsd_with_options: {e:?}"));
    println!("schema parsed ok (with stub dcml resolver)\n");

    // Validate every CPL_*.xml in the fixture dir
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
        let doc = match Document::parse_str(&src) {
            Ok(d) => d,
            Err(e) => {
                println!("[xml-parse-error] {name} -- {e:?}");
                continue;
            }
        };
        let result = validate_xsd(&doc, &schema);
        if result.is_valid {
            tally_valid += 1;
            println!(
                "[xsd-valid]       {name}  ({}E/{}W)",
                result.errors.len(),
                result.warnings.len()
            );
        } else {
            tally_invalid += 1;
            println!(
                "[xsd-INVALID]     {name}  ({}E/{}W)",
                result.errors.len(),
                result.warnings.len()
            );
            if !first_failure_dumped {
                println!("\n  -- first failure full Debug dump --");
                println!("  {:#?}", result);
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
        // Missing required IssueDate (between Annotation and Issuer)
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
        // IssueDate present but not a valid xs:dateTime
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
        // Out-of-order elements (EditRate before IssueDate)
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
        match Document::parse_str(xml) {
            Ok(doc) => {
                let result = validate_xsd(&doc, &schema);
                println!("  is_valid: {}", result.is_valid);
                println!("  errors  : {}", result.errors.len());
                println!("  warnings: {}", result.warnings.len());
                if !result.errors.is_empty() {
                    println!("  first error: {:#?}", result.errors[0]);
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
    let mini_schema = parse_xsd(mini_xsd).unwrap();
    let probes: &[(&str, &str)] = &[
        ("valid",                "<thing><name>x</name><count>5</count></thing>"),
        ("missing-required",     "<thing><name>x</name></thing>"),
        ("wrong-element-order",  "<thing><count>5</count><name>x</name></thing>"),
        ("invalid-type",         "<thing><name>x</name><count>not-a-number</count></thing>"),
        ("negative-positive",    "<thing><name>x</name><count>-1</count></thing>"),
        ("unknown-element",      "<thing><name>x</name><count>5</count><unknown/></thing>"),
    ];
    for (label, xml) in probes {
        let doc = Document::parse_str(xml).unwrap();
        let r = validate_xsd(&doc, &mini_schema);
        println!(
            "  {:<22} is_valid={}  ({}E/{}W)",
            label,
            r.is_valid,
            r.errors.len(),
            r.warnings.len()
        );
        if !r.errors.is_empty() {
            println!("      → {:?}", r.errors[0]);
        }
    }
}
