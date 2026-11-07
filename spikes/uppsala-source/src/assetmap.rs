//! Criterion 4: a typed ("Parsed") ST 429-9:2014 Asset Map built from the
//! uppsala DOM.
//!
//! * Only what the 2014 schema knows becomes data. The content models below
//!   are a hand transcription of the element names, cardinalities and
//!   attributes of the ST 429-9:2014 Annex A schema (no spec text is copied).
//! * Values are kept exactly as written: decoded (entities expanded) but never
//!   trimmed or case-folded. `Value` keeps absent / empty / present apart.
//! * Every field carries the source position of its element (and of its raw
//!   content or attribute value, for editing).
//! * Structure the schema does not know becomes a `Finding` with its position:
//!   unknown elements in the AM namespace, foreign-namespace elements where the
//!   content model has no `xs:any`, duplicates of single elements, text in
//!   element-only content, elements in simple content, unknown attributes.
//!   Foreign elements where the content model *does* have `xs:any
//!   namespace="##other"` are kept as `Extension`s. The 2014 schema has no
//!   `xs:any`, so that path is only exercised by the unit test with a
//!   modified content model.
//! * Cardinality (missing required elements) and ordering are left to the XSD
//!   layer, which already reports them with positions (criterion 5).

use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Range;

use uppsala::{Document, NodeId, NodeKind};

use crate::lines::LineIndex;

pub const AM_NS: &str = "http://www.smpte-ra.org/schemas/429-9/2007/AM";
const XSI_NS: &str = "http://www.w3.org/2001/XMLSchema-instance";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pos {
    pub range: Range<usize>,
    pub line: usize,
    pub col: usize,
}

/// A simple-content element or attribute value, exactly as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value<'a> {
    Absent,
    /// Present with no characters: `<X/>`, `<X></X>` or `a=""`.
    /// `raw` is the (empty) insertion point.
    Empty {
        at: Pos,
        raw: Range<usize>,
    },
    /// Present with characters. `text` is decoded (entities expanded) but
    /// otherwise untouched; `raw` is the source range of the content.
    Present {
        text: Cow<'a, str>,
        at: Pos,
        raw: Range<usize>,
    },
}

impl<'a> Value<'a> {
    pub fn text(&self) -> Option<&str> {
        match self {
            Value::Present { text, .. } => Some(text),
            Value::Empty { .. } => Some(""),
            Value::Absent => None,
        }
    }
    pub fn is_absent(&self) -> bool {
        matches!(self, Value::Absent)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserText<'a> {
    pub at: Pos,
    pub text: Value<'a>,
    pub language: Value<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk<'a> {
    pub at: Pos,
    pub path: Value<'a>,
    pub volume_index: Value<'a>,
    pub offset: Value<'a>,
    pub length: Value<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset<'a> {
    pub at: Pos,
    pub id: Value<'a>,
    pub annotation_text: Option<UserText<'a>>,
    pub packing_list: Value<'a>,
    /// `None` if the ChunkList element is absent.
    pub chunk_list: Option<Vec<Chunk<'a>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetMap<'a> {
    pub at: Pos,
    pub id: Value<'a>,
    pub annotation_text: Option<UserText<'a>>,
    pub creator: Option<UserText<'a>>,
    pub volume_count: Value<'a>,
    pub issue_date: Value<'a>,
    pub issuer: Option<UserText<'a>>,
    /// `None` if the AssetList element is absent.
    pub asset_list: Option<Vec<Asset<'a>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub code: &'static str,
    pub message: String,
    pub at: Pos,
    pub node: NodeId,
}

/// A foreign-namespace element accepted by an `xs:any`. Not data: a pointer
/// into the source, whose text is the payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extension {
    pub namespace: Option<String>,
    pub local_name: String,
    pub at: Pos,
    pub node: NodeId,
}

#[derive(Debug, Default)]
pub struct Parsed<'a> {
    pub asset_map: Option<AssetMap<'a>>,
    pub findings: Vec<Finding>,
    pub extensions: Vec<Extension>,
}

// ── Content models (ST 429-9:2014 Annex A) ──────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Model {
    /// Child elements in the AM namespace: (local name, maxOccurs > 1).
    pub children: &'static [(&'static str, bool)],
    /// `xs:any namespace="##other"` at the end of the sequence.
    pub any_other: bool,
    /// Unqualified attributes the type declares.
    pub attributes: &'static [&'static str],
    /// Simple content (text) rather than element-only content.
    pub simple: bool,
}

const fn simple(attributes: &'static [&'static str]) -> Model {
    Model {
        children: &[],
        any_other: false,
        attributes,
        simple: true,
    }
}

pub const PLAIN: Model = simple(&[]);
pub const USER_TEXT: Model = simple(&["language"]);
pub const ASSET_MAP: Model = Model {
    children: &[
        ("Id", false),
        ("AnnotationText", false),
        ("Creator", false),
        ("VolumeCount", false),
        ("IssueDate", false),
        ("Issuer", false),
        ("AssetList", false),
    ],
    any_other: false,
    attributes: &[],
    simple: false,
};
pub const ASSET_LIST: Model = Model {
    children: &[("Asset", true)],
    any_other: false,
    attributes: &[],
    simple: false,
};
pub const ASSET: Model = Model {
    children: &[
        ("Id", false),
        ("AnnotationText", false),
        ("PackingList", false),
        ("ChunkList", false),
    ],
    any_other: false,
    attributes: &[],
    simple: false,
};
pub const CHUNK_LIST: Model = Model {
    children: &[("Chunk", true)],
    any_other: false,
    attributes: &[],
    simple: false,
};
pub const CHUNK: Model = Model {
    children: &[
        ("Path", false),
        ("VolumeIndex", false),
        ("Offset", false),
        ("Length", false),
    ],
    any_other: false,
    attributes: &[],
    simple: false,
};

/// The models used for each element; swappable so tests can exercise the
/// `xs:any` path the 2014 schema does not have.
#[derive(Debug, Clone, Copy)]
pub struct Schema {
    pub asset_map: Model,
    pub asset_list: Model,
    pub asset: Model,
    pub chunk_list: Model,
    pub chunk: Model,
}

pub const ST429_9_2014: Schema = Schema {
    asset_map: ASSET_MAP,
    asset_list: ASSET_LIST,
    asset: ASSET,
    chunk_list: CHUNK_LIST,
    chunk: CHUNK,
};

// ── Builder ─────────────────────────────────────────────────────────────────

struct Builder<'d, 'a> {
    doc: &'d Document<'a>,
    lines: &'d LineIndex,
    findings: Vec<Finding>,
    extensions: Vec<Extension>,
}

type Kids = HashMap<&'static str, Vec<NodeId>>;

impl<'d, 'a> Builder<'d, 'a> {
    fn pos(&self, range: Range<usize>) -> Pos {
        let (line, col) = self.lines.line_col(range.start);
        Pos { range, line, col }
    }

    fn node_pos(&self, id: NodeId) -> Pos {
        self.pos(self.doc.node_range(id).unwrap_or(0..0))
    }

    fn finding(&mut self, code: &'static str, node: NodeId, at: Pos, message: String) {
        self.findings.push(Finding {
            code,
            message,
            at,
            node,
        });
    }

    fn check_attributes(&mut self, id: NodeId, model: &Model) {
        let Some(elem) = self.doc.element(id) else {
            return;
        };
        for attr in &elem.attributes {
            let ns = attr.name.namespace_uri.as_deref();
            let known = ns.is_none() && model.attributes.contains(&&*attr.name.local_name);
            if known || ns == Some(XSI_NS) {
                continue;
            }
            let range = self
                .doc
                .attribute_span(id, &attr.name)
                .map(|s| s.full())
                .unwrap_or_else(|| self.doc.node_range(id).unwrap_or(0..0));
            let at = self.pos(range);
            self.finding(
                "AM/UnknownAttribute",
                id,
                at,
                format!(
                    "attribute '{}' is not declared on <{}>",
                    attr.name.prefixed_name(),
                    elem.name.local_name
                ),
            );
        }
    }

    /// Sort the children of an element-only element into the model's slots.
    fn group(&mut self, id: NodeId, model: &Model) -> Kids {
        self.check_attributes(id, model);
        let parent = self
            .doc
            .element(id)
            .map(|e| e.name.local_name.to_string())
            .unwrap_or_default();
        let mut kids: Kids = HashMap::new();
        for c in self.doc.children_iter(id) {
            match self.doc.node_kind(c) {
                Some(NodeKind::Element(e)) => {
                    let ns = e.name.namespace_uri.as_deref();
                    let local = &*e.name.local_name;
                    if ns == Some(AM_NS) {
                        match model.children.iter().find(|(n, _)| *n == local) {
                            Some((name, many)) => {
                                let slot = kids.entry(name).or_default();
                                if !many && !slot.is_empty() {
                                    let at = self.node_pos(c);
                                    self.finding(
                                        "AM/DuplicateElement",
                                        c,
                                        at,
                                        format!("<{local}> may occur once in <{parent}>; extra occurrence ignored"),
                                    );
                                } else {
                                    slot.push(c);
                                }
                            }
                            None => {
                                let at = self.node_pos(c);
                                self.finding(
                                    "AM/UnknownElement",
                                    c,
                                    at,
                                    format!("<{local}> is not part of <{parent}> in ST 429-9:2014"),
                                );
                            }
                        }
                    } else if model.any_other {
                        let at = self.node_pos(c);
                        self.extensions.push(Extension {
                            namespace: ns.map(str::to_string),
                            local_name: local.to_string(),
                            at,
                            node: c,
                        });
                    } else {
                        let at = self.node_pos(c);
                        self.finding(
                            "AM/ForeignElement",
                            c,
                            at,
                            format!(
                                "<{local}> in namespace {} is not allowed in <{parent}> (no xs:any)",
                                ns.unwrap_or("(none)")
                            ),
                        );
                    }
                }
                Some(NodeKind::Text(t)) | Some(NodeKind::CData(t))
                    if !t.chars().all(|c| matches!(c, ' ' | '\t' | '\n' | '\r')) =>
                {
                    let at = self.node_pos(c);
                    self.finding(
                        "AM/UnexpectedText",
                        c,
                        at,
                        format!("character data in element-only <{parent}>"),
                    );
                }
                _ => {}
            }
        }
        kids
    }

    /// Simple-content value of element `id`.
    fn value(&mut self, id: NodeId, model: &Model) -> Value<'a> {
        self.check_attributes(id, model);
        let at = self.node_pos(id);
        let mut text: Option<Cow<'a, str>> = None;
        let mut raw: Option<Range<usize>> = None;
        for c in self.doc.children_iter(id) {
            let piece = match self.doc.node_kind(c) {
                Some(NodeKind::Text(t)) | Some(NodeKind::CData(t)) => t.clone(),
                Some(NodeKind::Element(e)) => {
                    let at = self.node_pos(c);
                    let msg = format!("element <{}> inside simple content", e.name.local_name);
                    self.finding("AM/ElementInSimpleContent", c, at, msg);
                    continue;
                }
                _ => continue, // comments, PIs: not character data
            };
            let r = self.doc.node_range(c).unwrap_or(0..0);
            raw = Some(match raw {
                None => r,
                Some(prev) => prev.start..r.end,
            });
            text = Some(match text {
                None => piece,
                Some(prev) => Cow::Owned(format!("{prev}{piece}")),
            });
        }
        match (text, raw) {
            (Some(text), Some(raw)) if !text.is_empty() => Value::Present { text, at, raw },
            _ => {
                // Insertion point: right after the start tag.
                let src = self.doc.input_text();
                let ins = crate::invariants::start_tag_end(src, at.range.start)
                    .map(|(e, _)| e)
                    .unwrap_or(at.range.end);
                Value::Empty { raw: ins..ins, at }
            }
        }
    }

    fn opt_value(&mut self, kids: &Kids, name: &str) -> Value<'a> {
        match kids.get(name).and_then(|v| v.first()) {
            Some(&id) => self.value(id, &PLAIN),
            None => Value::Absent,
        }
    }

    fn attr(&self, id: NodeId, local: &str) -> Value<'a> {
        let q = uppsala::QName::local(local);
        let Some(elem) = self.doc.element(id) else {
            return Value::Absent;
        };
        let Some(attr) = elem.attributes.iter().find(|a| a.name.matches(None, local)) else {
            return Value::Absent;
        };
        let raw = self.doc.attribute_range(id, &q).unwrap_or(0..0);
        let at = self.pos(raw.clone());
        if attr.value.is_empty() {
            Value::Empty { at, raw }
        } else {
            Value::Present {
                text: attr.value.clone(),
                at,
                raw,
            }
        }
    }

    fn user_text(&mut self, kids: &Kids, name: &str) -> Option<UserText<'a>> {
        let &id = kids.get(name)?.first()?;
        Some(UserText {
            at: self.node_pos(id),
            text: self.value(id, &USER_TEXT),
            language: self.attr(id, "language"),
        })
    }

    fn chunk(&mut self, id: NodeId, s: &Schema) -> Chunk<'a> {
        let k = self.group(id, &s.chunk);
        Chunk {
            at: self.node_pos(id),
            path: self.opt_value(&k, "Path"),
            volume_index: self.opt_value(&k, "VolumeIndex"),
            offset: self.opt_value(&k, "Offset"),
            length: self.opt_value(&k, "Length"),
        }
    }

    fn asset(&mut self, id: NodeId, s: &Schema) -> Asset<'a> {
        let k = self.group(id, &s.asset);
        let chunk_list = k.get("ChunkList").and_then(|v| v.first()).map(|&cl| {
            let ck = self.group(cl, &s.chunk_list);
            ck.get("Chunk")
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|c| self.chunk(c, s))
                .collect()
        });
        Asset {
            at: self.node_pos(id),
            id: self.opt_value(&k, "Id"),
            annotation_text: self.user_text(&k, "AnnotationText"),
            packing_list: self.opt_value(&k, "PackingList"),
            chunk_list,
        }
    }

    fn asset_map(&mut self, id: NodeId, s: &Schema) -> AssetMap<'a> {
        let k = self.group(id, &s.asset_map);
        let asset_list = k.get("AssetList").and_then(|v| v.first()).map(|&al| {
            let ak = self.group(al, &s.asset_list);
            ak.get("Asset")
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|a| self.asset(a, s))
                .collect()
        });
        AssetMap {
            at: self.node_pos(id),
            id: self.opt_value(&k, "Id"),
            annotation_text: self.user_text(&k, "AnnotationText"),
            creator: self.user_text(&k, "Creator"),
            volume_count: self.opt_value(&k, "VolumeCount"),
            issue_date: self.opt_value(&k, "IssueDate"),
            issuer: self.user_text(&k, "Issuer"),
            asset_list,
        }
    }
}

/// Build the typed Asset Map view over an already-parsed document.
pub fn from_document<'a>(doc: &Document<'a>, lines: &LineIndex, schema: &Schema) -> Parsed<'a> {
    let mut b = Builder {
        doc,
        lines,
        findings: Vec::new(),
        extensions: Vec::new(),
    };
    let mut asset_map = None;
    if let Some(root) = doc.document_element() {
        let e = doc.element(root).expect("document element");
        if e.name.matches(Some(AM_NS), "AssetMap") {
            asset_map = Some(b.asset_map(root, schema));
        } else {
            let at = b.node_pos(root);
            b.finding(
                "AM/NotAnAssetMap",
                root,
                at,
                format!(
                    "root is {{{}}}{}, not an ST 429-9 AssetMap",
                    e.name.namespace_uri.as_deref().unwrap_or(""),
                    e.name.local_name
                ),
            );
        }
    }
    Parsed {
        asset_map,
        findings: b.findings,
        extensions: b.extensions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = r#"<?xml version="1.0"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM" xmlns:x="urn:x">
  <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
  <AnnotationText language="en"> padded </AnnotationText>
  <Creator/>
  <VolumeCount>1</VolumeCount>
  <IssueDate>2016-01-01T00:00:00Z</IssueDate>
  <Issuer language="">me &amp; you</Issuer>
  <Bogus>1</Bogus>
  <x:Vendor>v</x:Vendor>
  <AssetList>
    <Asset>
      <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
      <ChunkList><Chunk><Path>a.mxf</Path><Offset>0</Offset></Chunk></ChunkList>
    </Asset>
  </AssetList>
</AssetMap>"#;

    #[test]
    fn values_findings_and_positions() {
        let doc = uppsala::parse(DOC).unwrap();
        let lines = LineIndex::new(DOC);
        let p = from_document(&doc, &lines, &ST429_9_2014);
        let am = p.asset_map.unwrap();
        assert_eq!(
            am.annotation_text.as_ref().unwrap().text.text(),
            Some(" padded ")
        );
        assert!(matches!(
            am.creator.as_ref().unwrap().text,
            Value::Empty { .. }
        ));
        assert!(am.creator.as_ref().unwrap().language.is_absent());
        let issuer = am.issuer.as_ref().unwrap();
        assert_eq!(issuer.text.text(), Some("me & you"));
        let Value::Present { raw, .. } = &issuer.text else {
            panic!()
        };
        assert_eq!(&DOC[raw.clone()], "me &amp; you");
        assert!(matches!(issuer.language, Value::Empty { .. }));
        let chunk = &am.asset_list.as_ref().unwrap()[0]
            .chunk_list
            .as_ref()
            .unwrap()[0];
        assert!(chunk.volume_index.is_absent());
        assert_eq!(chunk.offset.text(), Some("0"));
        let codes: Vec<_> = p
            .findings
            .iter()
            .map(|f| (f.code, f.at.line, f.at.col))
            .collect();
        assert_eq!(
            codes,
            [("AM/UnknownElement", 9, 3), ("AM/ForeignElement", 10, 3)]
        );
        assert!(p.extensions.is_empty());
    }

    #[test]
    fn xs_any_keeps_foreign_elements_as_extensions() {
        let mut schema = ST429_9_2014;
        schema.asset_map.any_other = true;
        let doc = uppsala::parse(DOC).unwrap();
        let lines = LineIndex::new(DOC);
        let p = from_document(&doc, &lines, &schema);
        assert_eq!(p.findings.len(), 1); // Bogus (AM namespace) is still unknown
        assert_eq!(p.extensions.len(), 1);
        let ext = &p.extensions[0];
        assert_eq!(ext.namespace.as_deref(), Some("urn:x"));
        assert_eq!(&DOC[ext.at.range.clone()], "<x:Vendor>v</x:Vendor>");
    }
}
