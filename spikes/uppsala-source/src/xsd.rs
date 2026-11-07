//! XSD validators built straight from uppsala, so the caller controls the
//! instance `Document` (one parse shared with the typed model).
//!
//! * CPL 2013 / 2016: imferno's vendored schemas, with the ST 433 dcml types
//!   inlined the same way `imferno_core::xsd` does it (that helper is
//!   private, so the logic is repeated here).
//! * AM (ST 429-9:2014): imferno vendors no Asset Map schema. It is read at
//!   runtime from the local SMPTE spec text (`$SMPTE_SPECS_TXT` or
//!   `/Users/jp/projects/smpte-specs/txt`) and cleaned of page furniture;
//!   nothing from the spec is stored in this repo.

use std::path::PathBuf;

use uppsala::{Document, NodeId, ValidationError, XsdValidator};

use crate::corpus::repo_root;
use crate::lines::LineIndex;

const CPL_2013: &str = "crates/imferno-core/specs/imf-cpl.xsd";
const CPL_2016: &str = "crates/imferno-core/specs/st2067-3a-2016.xsd";

/// Copy of `imferno_core::xsd::inline_dcml_types` (private there).
fn inline_dcml_types(xsd_src: &str) -> String {
    const TYPES: [&str; 3] = ["UUIDType", "UserTextType", "RationalType"];
    const DEFINITIONS: &str = r#"
  <xs:simpleType name="ImfernoDcmlUUIDType">
    <xs:restriction base="xs:anyURI">
      <xs:pattern value="urn:uuid:[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"/>
    </xs:restriction>
  </xs:simpleType>
  <xs:complexType name="ImfernoDcmlUserTextType">
    <xs:simpleContent>
      <xs:extension base="xs:string">
        <xs:attribute name="language" type="xs:language" use="optional" default="en"/>
      </xs:extension>
    </xs:simpleContent>
  </xs:complexType>
  <xs:simpleType name="ImfernoDcmlRationalType">
    <xs:restriction>
      <xs:simpleType>
        <xs:list itemType="xs:integer"/>
      </xs:simpleType>
      <xs:length value="2"/>
    </xs:restriction>
  </xs:simpleType>
"#;
    let root_start = xsd_src.find("<xs:schema").expect("xs:schema");
    let root_end = root_start + xsd_src[root_start..].find('>').expect(">");
    let tns = xsd_src[root_start..root_end]
        .split_once("targetNamespace=\"")
        .and_then(|(_, r)| r.split_once('"'))
        .map(|(ns, _)| ns)
        .expect("targetNamespace");
    let close = xsd_src.rfind("</xs:schema>").expect("close");
    let mut out = String::new();
    out.push_str(&xsd_src[..root_end]);
    out.push_str(&format!(" xmlns:imfdcml=\"{tns}\""));
    out.push_str(&xsd_src[root_end..close]);
    out.push_str(DEFINITIONS);
    out.push_str(&xsd_src[close..]);
    for t in TYPES {
        out = out.replace(
            &format!("\"dcml:{t}\""),
            &format!("\"imfdcml:ImfernoDcml{t}\""),
        );
    }
    out
}

fn build(schema_src: &str) -> XsdValidator {
    let doc = uppsala::parse(schema_src).expect("schema parses");
    XsdValidator::from_schema(&doc).expect("schema builds")
}

pub fn cpl_validator(namespace: &str) -> Option<XsdValidator> {
    let path = match namespace {
        "http://www.smpte-ra.org/schemas/2067-3/2013" => CPL_2013,
        "http://www.smpte-ra.org/schemas/2067-3/2016" => CPL_2016,
        _ => return None,
    };
    let src = std::fs::read_to_string(repo_root().join(path)).expect("read xsd");
    Some(build(&inline_dcml_types(&src)))
}

pub fn specs_txt_dir() -> PathBuf {
    std::env::var_os("SMPTE_SPECS_TXT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/jp/projects/smpte-specs/txt"))
}

/// The Annex A schema of ST 429-9:2014, extracted from the spec text.
pub fn am_2014_schema_text() -> std::io::Result<String> {
    let txt = std::fs::read_to_string(specs_txt_dir().join("st429-9-2014.txt"))?;
    let start = txt.find("<xs:schema").expect("schema start");
    let end = txt[start..].find("</xs:schema>").expect("schema end") + start;
    let mut out = String::new();
    let mut lines = txt[start..end].lines().peekable();
    while let Some(line) = lines.next() {
        let t = line.trim();
        if !(t.starts_with('<') || t.starts_with("xmlns") || t.starts_with("elementFormDefault")) {
            continue; // page headers/footers, blank lines
        }
        let mut joined = t.to_string();
        // A pattern value wrapped across two lines.
        if joined.ends_with('-') {
            if let Some(next) = lines.next() {
                joined.push_str(next.trim());
            }
        }
        out.push_str(&joined);
        out.push('\n');
    }
    out.push_str("</xs:schema>\n");
    Ok(out)
}

pub fn am_2014_validator() -> std::io::Result<XsdValidator> {
    Ok(build(&am_2014_schema_text()?))
}

/// Element/text start offset -> node, for mapping (line, column) findings.
pub fn node_at(doc: &Document<'_>, lines: &LineIndex, err: &ValidationError) -> Option<NodeId> {
    let off = lines.offset(err.line?, err.column?)?;
    doc.descendants(doc.root())
        .into_iter()
        .find(|&n| n != doc.root() && doc.node_range(n).is_some_and(|r| r.start == off))
}
