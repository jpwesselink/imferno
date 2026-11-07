//! Criterion 1/2/8: the DOM is an exact index over the source text.
//!
//! For a parsed document this checks, for every node:
//!
//! * `&src[node_range]` is that node's source: elements start with `<` and
//!   their written name and end with their end tag (or `/>`); text decodes to
//!   the DOM text; comments / CDATA / PIs are byte-exact;
//! * children tile their parent: the first child starts where the start tag
//!   ends, siblings are contiguous, the last child ends where the end tag
//!   starts (so ranges are nested, ordered and non-overlapping, and the only
//!   bytes of an element not covered by a child are its own tags);
//! * the bytes outside the root's children are only BOM, XML declaration,
//!   DOCTYPE and whitespace;
//! * every attribute and namespace declaration has a source span inside the
//!   start tag; decoding the raw value gives the DOM value; the start tag
//!   holds nothing but the name, the spans, whitespace and `>`/`/>`;
//! * `node_line`/`node_column` agree with an independent line index.
//!
//! The tag scanner here is written independently of uppsala's.

use std::ops::Range;

use uppsala::{Document, NodeId, NodeKind};

use crate::lines::LineIndex;

#[derive(Debug, Clone)]
pub struct Violation {
    pub node: Option<NodeId>,
    pub range: Range<usize>,
    pub what: String,
}

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub nodes: usize,
    pub elements: usize,
    pub texts: usize,
    pub comments: usize,
    pub cdata: usize,
    pub pis: usize,
    pub attributes: usize,
    pub namespace_decls: usize,
    /// Text/attribute values containing a non-predefined entity reference,
    /// so the decoded comparison could not be made (range still checked).
    pub unverifiable_values: usize,
    /// Bytes inside leaf nodes (text, CDATA, comment, PI).
    pub leaf_bytes: usize,
    /// Bytes inside start/end tags.
    pub tag_bytes: usize,
    /// Prolog/epilog bytes not covered by a node (whitespace, decl, DOCTYPE).
    pub outside_bytes: usize,
}

pub struct Options {
    /// Also compare uppsala's `node_line`/`node_column` (O(n) per call) with
    /// the line index for every node.
    pub check_line_col: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            check_line_col: true,
        }
    }
}

fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

/// End of the start tag beginning at `start` (offset after `>`), and whether
/// it is self-closing. Quote-aware, independent of uppsala.
pub fn start_tag_end(src: &str, start: usize) -> Option<(usize, bool)> {
    let b = src.as_bytes();
    if b.get(start) != Some(&b'<') {
        return None;
    }
    let mut i = start + 1;
    let mut quote: Option<u8> = None;
    while i < b.len() {
        let c = b[i];
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == b'"' || c == b'\'' => quote = Some(c),
            None if c == b'>' => return Some((i + 1, b[i - 1] == b'/')),
            None => {}
        }
        i += 1;
    }
    None
}

/// Decode a text or attribute value as written: expand predefined entities
/// and character references. `None` if it contains another entity reference.
pub fn decode(raw: &str) -> Option<String> {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp + 1..];
        let semi = after.find(';')?;
        let name = &after[..semi];
        match name {
            "lt" => out.push('<'),
            "gt" => out.push('>'),
            "amp" => out.push('&'),
            "apos" => out.push('\''),
            "quot" => out.push('"'),
            _ if name.starts_with("#x") => {
                out.push(char::from_u32(u32::from_str_radix(&name[2..], 16).ok()?)?)
            }
            _ if name.starts_with('#') => out.push(char::from_u32(name[1..].parse().ok()?)?),
            _ => return None,
        }
        rest = &after[semi + 1..];
    }
    out.push_str(rest);
    Some(out)
}

/// XML 1.0 section 2.11 line-end normalization.
pub fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

struct Checker<'s, 'd> {
    src: &'s str,
    doc: &'d Document<'s>,
    lines: LineIndex,
    opts: &'d Options,
    stats: Stats,
    out: Vec<Violation>,
}

impl<'s, 'd> Checker<'s, 'd> {
    fn fail(&mut self, node: Option<NodeId>, range: Range<usize>, what: impl Into<String>) {
        self.out.push(Violation {
            node,
            range,
            what: what.into(),
        });
    }

    fn compare_value(&mut self, id: NodeId, range: Range<usize>, dom: &str, what: &str) {
        let raw = &self.src[range.clone()];
        match decode(raw) {
            None => self.stats.unverifiable_values += 1,
            Some(d) if d == dom => {}
            Some(d) if normalize_newlines(&d) == dom => {}
            Some(d) => self.fail(
                Some(id),
                range,
                format!("{what}: decoded source {d:?} != DOM {dom:?}"),
            ),
        }
    }

    fn node(&mut self, id: NodeId, parent_range: &Range<usize>) {
        self.stats.nodes += 1;
        let Some(range) = self.doc.node_range(id) else {
            self.fail(Some(id), 0..0, "parsed node has no range");
            return;
        };
        if range.start > range.end
            || range.start < parent_range.start
            || range.end > parent_range.end
        {
            self.fail(Some(id), range, "range not inside parent range");
            return;
        }
        if !self.src.is_char_boundary(range.start) || !self.src.is_char_boundary(range.end) {
            self.fail(Some(id), range, "range not on a char boundary");
            return;
        }
        if self.opts.check_line_col {
            let ours = self.lines.line_col(range.start);
            let theirs = (self.doc.node_line(id), self.doc.node_column(id));
            if ours != theirs {
                self.fail(
                    Some(id),
                    range.clone(),
                    format!("line/col {theirs:?} != line index {ours:?}"),
                );
            }
        }
        let slice = &self.src[range.clone()];
        match self.doc.node_kind(id).cloned() {
            Some(NodeKind::Element(elem)) => self.element(id, range, &elem),
            Some(NodeKind::Text(t)) => {
                self.stats.texts += 1;
                self.stats.leaf_bytes += range.len();
                if slice.contains('<') || slice.is_empty() {
                    self.fail(Some(id), range.clone(), "text slice empty or contains '<'");
                }
                self.compare_value(id, range, &t, "text");
            }
            Some(NodeKind::CData(t)) => {
                self.stats.cdata += 1;
                self.stats.leaf_bytes += range.len();
                if slice != format!("<![CDATA[{t}]]>") {
                    self.fail(Some(id), range, "CDATA slice mismatch");
                }
            }
            Some(NodeKind::Comment(t)) => {
                self.stats.comments += 1;
                self.stats.leaf_bytes += range.len();
                if slice != format!("<!--{t}-->") {
                    self.fail(Some(id), range, "comment slice mismatch");
                }
            }
            Some(NodeKind::ProcessingInstruction(pi)) => {
                self.stats.pis += 1;
                self.stats.leaf_bytes += range.len();
                let ok = slice.starts_with(&format!("<?{}", pi.target))
                    && slice.ends_with("?>")
                    && pi.data.as_deref().is_none_or(|d| slice.contains(d));
                if !ok {
                    self.fail(Some(id), range, "PI slice mismatch");
                }
            }
            other => self.fail(Some(id), range, format!("unexpected node kind {other:?}")),
        }
    }

    fn element(&mut self, id: NodeId, range: Range<usize>, elem: &uppsala::Element<'s>) {
        self.stats.elements += 1;
        let name = elem.name.prefixed_name().into_owned();
        let slice = &self.src[range.clone()];
        let open = format!("<{name}");
        let after_name = slice.as_bytes().get(open.len()).copied();
        if !slice.starts_with(&open)
            || !matches!(after_name, Some(b'>') | Some(b'/')) && !after_name.is_some_and(is_ws)
        {
            self.fail(
                Some(id),
                range,
                format!("element slice does not start with {open}"),
            );
            return;
        }
        let Some((tag_end, self_closing)) = start_tag_end(self.src, range.start) else {
            self.fail(Some(id), range, "unterminated start tag");
            return;
        };
        self.attributes(id, range.start + open.len()..tag_end, elem);

        let children: Vec<NodeId> = self.doc.children_iter(id).collect();
        if self_closing {
            self.stats.tag_bytes += tag_end - range.start;
            if tag_end != range.end || !children.is_empty() {
                self.fail(
                    Some(id),
                    range,
                    "self-closing element range/children mismatch",
                );
            }
            return;
        }
        let mut cursor = tag_end;
        for &c in &children {
            let Some(cr) = self.doc.node_range(c) else {
                self.fail(Some(c), 0..0, "child without range");
                return;
            };
            if cr.start != cursor {
                self.fail(
                    Some(c),
                    cursor..cr.start.max(cursor),
                    format!(
                        "gap/overlap before child: expected start {cursor}, got {}",
                        cr.start
                    ),
                );
            }
            self.node(c, &range);
            cursor = cr.end;
        }
        let end_tag = &self.src[cursor.min(range.end)..range.end];
        let ok = end_tag
            .strip_prefix("</")
            .and_then(|r| r.strip_prefix(name.as_str()))
            .and_then(|r| r.strip_suffix('>'))
            .is_some_and(|ws| ws.bytes().all(is_ws));
        if !ok {
            self.fail(
                Some(id),
                cursor..range.end,
                format!("end tag mismatch: {end_tag:?}"),
            );
        }
        self.stats.tag_bytes += (tag_end - range.start) + end_tag.len();
    }

    /// `inner` is the start tag after the element name, through `>`.
    fn attributes(&mut self, id: NodeId, inner: Range<usize>, elem: &uppsala::Element<'s>) {
        let Some(spans) = self.doc.attribute_spans(id) else {
            self.fail(Some(id), inner, "element has no attribute spans");
            return;
        };
        // The start tag is exactly: name, then (ws + span)*, ws*, '>' or '/>'.
        let mut cursor = inner.start;
        for s in &spans {
            let gap = &self.src[cursor..s.name.start.max(cursor)];
            if s.name.start < cursor || gap.is_empty() || !gap.bytes().all(is_ws) {
                self.fail(Some(id), cursor..s.name.start, "bad bytes before attribute");
            }
            let full = &self.src[s.full()];
            let name = &self.src[s.name.clone()];
            let eq = &self.src[s.name.end..s.value.start - 1];
            let q = self.src.as_bytes()[s.value.start - 1];
            let closing = self.src.as_bytes()[s.value.end];
            if q != s.quote
                || closing != s.quote
                || eq.trim_matches(|c: char| c.is_ascii_whitespace()) != "="
                || !full.starts_with(name)
            {
                self.fail(Some(id), s.full(), "malformed attribute span");
            }
            cursor = s.value.end + 1;
        }
        let tail = &self.src[cursor..inner.end];
        let tail = tail.trim_start_matches([' ', '\t', '\n', '\r']);
        if tail != ">" && tail != "/>" {
            self.fail(
                Some(id),
                cursor..inner.end,
                format!("start tag tail {tail:?}"),
            );
        }

        // Every DOM attribute / namespace declaration maps to a span.
        let non_ns = spans
            .iter()
            .filter(|s| !s.is_namespace_declaration(self.src))
            .count();
        if non_ns != elem.attributes.len()
            || spans.len() - non_ns != elem.namespace_declarations.len()
        {
            self.fail(Some(id), inner.clone(), "span count != attribute count");
        }
        for attr in &elem.attributes {
            self.stats.attributes += 1;
            match self.doc.attribute_range(id, &attr.name) {
                Some(r) => {
                    self.compare_value(id, r, &attr.value, &format!("attribute {}", attr.name))
                }
                None => self.fail(
                    Some(id),
                    inner.clone(),
                    format!("no range for {}", attr.name),
                ),
            }
        }
        for (prefix, uri) in &elem.namespace_declarations {
            self.stats.namespace_decls += 1;
            match self.doc.namespace_declaration_span(id, prefix) {
                Some(s) => self.compare_value(id, s.value, uri, "xmlns"),
                None => self.fail(
                    Some(id),
                    inner.clone(),
                    format!("no span for xmlns:{prefix}"),
                ),
            }
        }
    }

    fn root(&mut self) {
        let root = self.doc.root();
        let whole = 0..self.src.len();
        let mut cursor = 0;
        let mut outside = String::new();
        for c in self.doc.children_iter(root).collect::<Vec<_>>() {
            let Some(cr) = self.doc.node_range(c) else {
                self.fail(Some(c), 0..0, "top-level node without range");
                continue;
            };
            if cr.start < cursor {
                self.fail(Some(c), cr.clone(), "top-level overlap");
            } else {
                outside.push_str(&self.src[cursor..cr.start]);
            }
            self.node(c, &whole);
            cursor = cr.end;
        }
        outside.push_str(&self.src[cursor.min(self.src.len())..]);
        self.stats.outside_bytes = outside.len();
        // Strip what may legitimately sit outside nodes.
        let mut rest = outside.trim_start_matches('\u{feff}').to_string();
        if let Some(decl_end) = rest
            .trim_start()
            .starts_with("<?xml")
            .then(|| rest.find("?>"))
        {
            if let Some(e) = decl_end {
                rest.replace_range(..e + 2, "");
            }
        }
        if let Some(dt) = &self.doc.doctype {
            rest = rest.replacen(dt.as_ref(), "", 1);
        }
        if !rest.bytes().all(is_ws) {
            self.fail(
                None,
                0..0,
                format!("non-whitespace outside nodes: {:?}", truncate(&rest)),
            );
        }
    }
}

fn truncate(s: &str) -> String {
    s.chars().take(80).collect()
}

/// Run every invariant over a parsed document.
pub fn check<'s>(src: &'s str, doc: &Document<'s>, opts: &Options) -> (Stats, Vec<Violation>) {
    let mut c = Checker {
        src,
        doc,
        lines: LineIndex::new(src),
        opts,
        stats: Stats::default(),
        out: Vec::new(),
    };
    c.root();
    (c.stats, c.out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn violations(parsed_from: &str, checked_against: &str) -> Vec<Violation> {
        let doc = uppsala::parse(parsed_from).unwrap();
        // Point the check at different text of the same length: node ranges
        // now index into the wrong bytes, which the checker must notice.
        let (_, v) = check(checked_against, &doc, &Options::default());
        v
    }

    #[test]
    fn clean_document_passes() {
        let xml = "<?xml version=\"1.0\"?>\n<!-- c --><a x='1' xmlns='urn:a'>\r\n t&amp;<b/><![CDATA[<x>]]><?p d?></a>\n";
        assert!(violations(xml, xml).is_empty());
    }

    #[test]
    fn detects_changed_text_tag_and_attribute() {
        let xml = "<a x=\"1\"><b>text</b></a>";
        assert!(!violations(xml, "<a x=\"1\"><b>tExt</b></a>").is_empty());
        assert!(!violations(xml, "<a x=\"1\"><c>text</c></a>").is_empty());
        assert!(!violations(xml, "<a x=\"2\"><b>text</b></a>").is_empty());
        assert!(!violations(xml, "<a x=\"1\"> b>text</b></a>").is_empty());
    }

    #[test]
    fn uppsala_text_line_endings() {
        // Records uppsala's behaviour: CR LF in text content is normalized
        // to LF in the DOM (XML 1.0 section 2.11); the range keeps the CR.
        let xml = "<a>x\r\ny</a>";
        let doc = uppsala::parse(xml).unwrap();
        let a = doc.document_element().unwrap();
        let t = doc.first_child(a).unwrap();
        assert_eq!(doc.text_content(t), Some("x\ny"));
        assert_eq!(doc.node_source(t), Some("x\r\ny"));
    }
}
