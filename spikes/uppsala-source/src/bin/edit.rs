//! Criterion 3: edit by range on every CPL in the corpus.
//!
//! For each CPL (2013/2016 namespaces, which have vendored XSDs) apply, one
//! at a time, by splicing text at uppsala ranges:
//!   (a) replace the text of <ContentTitle> with a two-line value containing
//!       an entity reference (shifts every later line by one);
//!   (b) replace the value of the first attribute in the document with an
//!       invalid xs:language-ish value;
//!   (c) delete the <IssueDate> element (a required element).
//! Then: reparse succeeds; the full range invariants hold on the new text;
//! the byte diff is exactly the edit window; every XSD finding of the old
//! text reappears at its old position mapped through the edit; every new
//! finding lands on a node (and we record which node).
//!
//! Usage: cargo run --release --bin edit

use std::collections::BTreeMap;
use std::ops::Range;

use uppsala::{Document, NodeId, NodeKind, ValidationError, XsdValidator};
use uppsala_source_spike::corpus::{imf_corpus, rel, root_local_name};
use uppsala_source_spike::invariants::{check, start_tag_end, Options};
use uppsala_source_spike::lines::LineIndex;
use uppsala_source_spike::xsd::{cpl_validator, node_at};
use uppsala_source_spike::{changed_window, splice};

fn first_element(doc: &Document<'_>, local: &str) -> Option<NodeId> {
    doc.descendants(doc.root())
        .into_iter()
        .find(|&n| doc.element(n).is_some_and(|e| e.name.local_name == local))
}

/// Range to replace for an element's text content: the existing text, or
/// the empty insertion point after the start tag.
fn text_range(doc: &Document<'_>, src: &str, el: NodeId) -> Range<usize> {
    let kids: Vec<_> = doc.children_iter(el).collect();
    if let (Some(&f), Some(&l)) = (kids.first(), kids.last()) {
        if kids
            .iter()
            .all(|&k| matches!(doc.node_kind(k), Some(NodeKind::Text(_))))
        {
            return doc.node_range(f).unwrap().start..doc.node_range(l).unwrap().end;
        }
    }
    let r = doc.node_range(el).unwrap();
    let (e, _) = start_tag_end(src, r.start).unwrap();
    e..e
}

struct Finding {
    offset: usize,
    line: usize,
    col: usize,
    message: String,
}

fn findings(v: &XsdValidator, doc: &Document<'_>, lines: &LineIndex) -> Vec<Finding> {
    v.validate(doc)
        .into_iter()
        .map(|e: ValidationError| {
            let (line, col) = (e.line.unwrap_or(0), e.column.unwrap_or(0));
            Finding {
                offset: lines.offset(line, col).unwrap_or(usize::MAX),
                line,
                col,
                message: e.message,
            }
        })
        .collect()
}

#[derive(Default)]
struct Outcome {
    applied: usize,
    reparse_ok: usize,
    invariants_ok: usize,
    diff_exact: usize,
    old_findings: usize,
    old_findings_relocated: usize,
    new_findings: usize,
    new_findings_on_node: usize,
    new_finding_nodes: BTreeMap<String, usize>,
    failures: Vec<String>,
}

fn main() {
    let mut validators: BTreeMap<String, Option<XsdValidator>> = BTreeMap::new();
    let mut outcomes: BTreeMap<&str, Outcome> = BTreeMap::new();
    let mut files = 0;

    for path in imf_corpus() {
        let src = std::fs::read_to_string(&path).unwrap();
        if root_local_name(&src).as_deref() != Some("CompositionPlaylist") {
            continue;
        }
        let doc = uppsala::parse(&src).unwrap();
        let ns = doc
            .element(doc.document_element().unwrap())
            .and_then(|e| e.name.namespace_uri.as_deref().map(str::to_string))
            .unwrap_or_default();
        let Some(v) = validators
            .entry(ns.clone())
            .or_insert_with(|| cpl_validator(&ns))
        else {
            continue;
        };
        files += 1;
        let lines = LineIndex::new(&src);
        let before = findings(v, &doc, &lines);

        // Build the three edits as (label, range, replacement, edited node).
        let mut edits: Vec<(&str, Range<usize>, String, NodeId)> = Vec::new();
        if let Some(el) = first_element(&doc, "ContentTitle") {
            edits.push((
                "a: element text",
                text_range(&doc, &src, el),
                "Edited &amp;\ntitle".into(),
                el,
            ));
        }
        let first_attr = doc.descendants(doc.root()).into_iter().find_map(|n| {
            let e = doc.element(n)?;
            let a = e.attributes.first()?;
            Some((n, doc.attribute_range(n, &a.name)?))
        });
        if let Some((n, r)) = first_attr {
            edits.push(("b: attribute value", r, "not a language!".into(), n));
        }
        if let Some(el) = first_element(&doc, "IssueDate") {
            edits.push((
                "c: delete element",
                doc.node_range(el).unwrap(),
                String::new(),
                el,
            ));
        }

        for (label, range, with, node) in edits {
            let o = outcomes.entry(label).or_default();
            o.applied += 1;
            let new = splice(&src, range.clone(), &with);
            let fail = |o: &mut Outcome, m: String| o.failures.push(format!("{}: {m}", rel(&path)));

            // Diff: exactly the edit window changed.
            let expect = (range.clone(), range.start..range.start + with.len());
            match changed_window(&src, &new) {
                // The window may be narrower if replacement shares a prefix/suffix with the old bytes.
                Some((a, b))
                    if a.start >= expect.0.start
                        && a.end <= expect.0.end
                        && b.start >= expect.1.start
                        && b.end <= expect.1.end =>
                {
                    o.diff_exact += 1
                }
                w => fail(o, format!("{label}: diff window {w:?} != {expect:?}")),
            }
            let new_doc = match uppsala::parse(&new) {
                Ok(d) => d,
                Err(e) => {
                    fail(o, format!("{label}: reparse failed: {e}"));
                    continue;
                }
            };
            o.reparse_ok += 1;
            let (_, viol) = check(
                &new,
                &new_doc,
                &Options {
                    check_line_col: false,
                },
            );
            if viol.is_empty() {
                o.invariants_ok += 1;
            } else {
                fail(o, format!("{label}: {} invariant violations", viol.len()));
            }

            // Findings: old ones relocate through the edit; new ones land on nodes.
            let new_lines = LineIndex::new(&new);
            let mut after = findings(v, &new_doc, &new_lines);
            let delta = with.len() as isize - range.len() as isize;
            for f in &before {
                // Findings inside the edited span are not expected to survive.
                if f.offset >= range.start && f.offset < range.end {
                    continue;
                }
                o.old_findings += 1;
                let moved = if f.offset >= range.end {
                    (f.offset as isize + delta) as usize
                } else {
                    f.offset
                };
                let (l, c) = new_lines.line_col(moved);
                if let Some(i) = after
                    .iter()
                    .position(|g| g.line == l && g.col == c && g.message == f.message)
                {
                    after.remove(i);
                    o.old_findings_relocated += 1;
                } else {
                    fail(
                        o,
                        format!(
                            "{label}: finding {}:{} {:?} not found at {l}:{c}",
                            f.line, f.col, f.message
                        ),
                    );
                }
            }
            let edited_start = doc.node_range(node).unwrap().start;
            for g in after {
                o.new_findings += 1;
                let err = ValidationError {
                    message: g.message.clone(),
                    line: Some(g.line),
                    column: Some(g.col),
                    element_path: None,
                };
                match node_at(&new_doc, &new_lines, &err) {
                    Some(n) => {
                        o.new_findings_on_node += 1;
                        let name = new_doc
                            .element(n)
                            .map(|e| e.name.local_name.to_string())
                            .unwrap_or("#text".into());
                        let start = new_doc.node_range(n).unwrap().start;
                        let rel_pos = if start == edited_start {
                            "edited node"
                        } else if start == range.start + with.len()
                            || (with.is_empty() && start >= range.start)
                        {
                            "node after deletion"
                        } else {
                            "other node"
                        };
                        let key = format!(
                            "<{name}> ({rel_pos}): {}",
                            g.message.chars().take(60).collect::<String>()
                        );
                        *o.new_finding_nodes.entry(key).or_default() += 1;
                    }
                    None => fail(
                        o,
                        format!("{label}: new finding {}:{} maps to no node", g.line, g.col),
                    ),
                }
            }
        }
    }

    println!("CPLs edited: {files}");
    for (label, o) in &outcomes {
        println!(
            "[{label}] applied {}  reparse ok {}  invariants ok {}  diff == edit window {}",
            o.applied, o.reparse_ok, o.invariants_ok, o.diff_exact
        );
        println!(
            "  pre-existing findings outside the edit: {}, found at the mapped position after the edit: {}",
            o.old_findings, o.old_findings_relocated
        );
        println!(
            "  new findings: {}, on a node: {}",
            o.new_findings, o.new_findings_on_node
        );
        for (k, n) in &o.new_finding_nodes {
            println!("    {n} x {k}");
        }
        for f in o.failures.iter().take(10) {
            println!("  FAIL {f}");
        }
        if o.failures.len() > 10 {
            println!("  ... {} failures", o.failures.len());
        }
    }
}
