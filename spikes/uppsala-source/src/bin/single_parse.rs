//! Criterion 5: one uppsala parse feeds both the XSD layer and the typed
//! model, and XSD findings land on the same nodes.
//!
//! * CPL (2013/2016): validate the shared `Document` with uppsala's
//!   `XsdValidator`, and check the findings are identical (message, line,
//!   column) to what imferno's own `xsd::validate_parsed_cpl` reports today
//!   after its separate parse.
//! * ASSETMAP: validate the shared `Document` against the ST 429-9:2014
//!   schema (read from the spec text at runtime), build the typed model from
//!   the same `Document`, and map every XSD finding to a node and to the
//!   typed-model field at that node.
//!
//! Usage: cargo run --release --bin single_parse

use std::collections::{BTreeMap, HashSet};

use uppsala::NodeKind;
use uppsala_source_spike::assetmap::{self, Value, ST429_9_2014};
use uppsala_source_spike::corpus::{imf_corpus, rel, root_local_name};
use uppsala_source_spike::lines::LineIndex;
use uppsala_source_spike::xsd::{am_2014_validator, cpl_validator, node_at};

fn root_ns(doc: &uppsala::Document<'_>) -> String {
    let r = doc.document_element().unwrap();
    doc.element(r)
        .and_then(|e| e.name.namespace_uri.as_deref().map(str::to_string))
        .unwrap_or_default()
}

/// Start offsets of every element / value the typed model points at.
fn model_starts(am: &assetmap::AssetMap<'_>) -> HashSet<usize> {
    let mut s = HashSet::new();
    let val = |s: &mut HashSet<usize>, v: &Value<'_>| match v {
        Value::Absent => {}
        Value::Empty { at, .. } | Value::Present { at, .. } => {
            s.insert(at.range.start);
        }
    };
    s.insert(am.at.range.start);
    for v in [&am.id, &am.volume_count, &am.issue_date] {
        val(&mut s, v);
    }
    for u in [&am.annotation_text, &am.creator, &am.issuer]
        .into_iter()
        .flatten()
    {
        s.insert(u.at.range.start);
    }
    for a in am.asset_list.iter().flatten() {
        s.insert(a.at.range.start);
        val(&mut s, &a.id);
        val(&mut s, &a.packing_list);
        if let Some(u) = &a.annotation_text {
            s.insert(u.at.range.start);
        }
        for c in a.chunk_list.iter().flatten() {
            s.insert(c.at.range.start);
            for v in [&c.path, &c.volume_index, &c.offset, &c.length] {
                val(&mut s, v);
            }
        }
    }
    s
}

fn main() {
    let mut cpl_validators: BTreeMap<String, Option<uppsala::XsdValidator>> = BTreeMap::new();
    let am_validator = am_2014_validator().expect("ST 429-9:2014 spec text available");

    let (mut cpl_files, mut cpl_identical, mut cpl_errors, mut cpl_mapped) = (0, 0, 0, 0);
    let mut cpl_mismatch = Vec::new();
    let mut cpl_via_raw = 0;
    let mut cpl_skipped: BTreeMap<String, usize> = BTreeMap::new();
    let (mut am_files, mut am_errors, mut am_mapped, mut am_on_model) = (0, 0, 0, 0);
    let mut am_on_finding = 0;
    let mut am_examples = Vec::new();
    let mut am_by_msg: BTreeMap<String, usize> = BTreeMap::new();

    for path in imf_corpus() {
        let src = std::fs::read_to_string(&path).unwrap();
        let root = root_local_name(&src).unwrap_or_default();
        if root != "CompositionPlaylist" && root != "AssetMap" {
            continue;
        }
        // The one parse.
        let doc = uppsala::parse(&src).unwrap();
        let lines = LineIndex::new(&src);
        let ns = root_ns(&doc);

        if root == "CompositionPlaylist" {
            let v = cpl_validators
                .entry(ns.clone())
                .or_insert_with(|| cpl_validator(&ns));
            let Some(v) = v else {
                *cpl_skipped.entry(ns).or_default() += 1;
                continue;
            };
            cpl_files += 1;
            let errs = v.validate(&doc);
            cpl_errors += errs.len();
            cpl_mapped += errs
                .iter()
                .filter(|e| node_at(&doc, &lines, e).is_some())
                .count();
            // imferno today: parse_cpl (quick-xml) + a second uppsala parse
            // inside validate_parsed_cpl.
            let ours: Vec<String> = errs
                .iter()
                .map(|e| match (e.line, e.column) {
                    (Some(l), Some(c)) => format!("{} (at line {l}, column {c})", e.message),
                    _ => e.message.clone(),
                })
                .collect();
            let theirs: Vec<String> = match imferno_core::cpl::parse_cpl(&src) {
                Ok(cpl) => imferno_core::xsd::validate_parsed_cpl(&cpl)
                    .into_iter()
                    .map(|i| i.message)
                    .collect(),
                // parse_cpl rejects the file, so imferno never reaches the
                // XSD pass; compare with its raw-XML entry point instead.
                Err(_) => {
                    cpl_via_raw += 1;
                    let xsd_path = match ns.as_str() {
                        "http://www.smpte-ra.org/schemas/2067-3/2013" => "imf-cpl.xsd",
                        _ => "st2067-3a-2016.xsd",
                    };
                    let xsd = std::fs::read_to_string(
                        uppsala_source_spike::corpus::repo_root()
                            .join("crates/imferno-core/specs")
                            .join(xsd_path),
                    )
                    .unwrap();
                    imferno_core::xsd::validate_against_schema(&src, &xsd, None)
                        .into_iter()
                        .map(|i| i.message)
                        .collect()
                }
            };
            if ours == theirs {
                cpl_identical += 1;
            } else {
                cpl_mismatch.push(format!(
                    "{}: ours {} vs imferno {} findings",
                    rel(&path),
                    ours.len(),
                    theirs.len()
                ));
            }
        } else {
            am_files += 1;
            let errs = am_validator.validate(&doc);
            let model = assetmap::from_document(&doc, &lines, &ST429_9_2014);
            let starts = model
                .asset_map
                .as_ref()
                .map(model_starts)
                .unwrap_or_default();
            let finding_starts: HashSet<usize> =
                model.findings.iter().map(|f| f.at.range.start).collect();
            for e in &errs {
                am_errors += 1;
                let key: String = e.message.chars().take(70).collect();
                *am_by_msg.entry(key).or_default() += 1;
                let Some(n) = node_at(&doc, &lines, e) else {
                    continue;
                };
                am_mapped += 1;
                // The XSD node, or its element if it is a text node.
                let el = match doc.node_kind(n) {
                    Some(NodeKind::Element(_)) => n,
                    _ => doc.parent(n).unwrap_or(n),
                };
                let start = doc.node_range(el).unwrap().start;
                let on_model = starts.contains(&start);
                if on_model {
                    am_on_model += 1;
                }
                if finding_starts.contains(&start) {
                    am_on_finding += 1;
                }
                if am_examples.len() < 8 {
                    let name = doc
                        .element(el)
                        .map(|x| x.name.local_name.to_string())
                        .unwrap_or_default();
                    am_examples.push(format!(
                        "{}:{}:{} <{name}> on typed field: {on_model} | {}",
                        rel(&path),
                        e.line.unwrap_or(0),
                        e.column.unwrap_or(0),
                        e.message
                    ));
                }
            }
        }
    }

    println!("CPL: {cpl_files} validated from the shared Document; skipped (no vendored XSD): {cpl_skipped:?}");
    println!("  findings identical to imferno's XSD pass: {cpl_identical}/{cpl_files} ({cpl_via_raw} via validate_against_schema because parse_cpl rejects them)");
    for m in &cpl_mismatch {
        println!("    {m}");
    }
    println!("  XSD findings: {cpl_errors}, mapped to a node by (line, col): {cpl_mapped}");
    println!("ASSETMAP: {am_files} validated against ST 429-9:2014 from the shared Document");
    println!("  XSD findings: {am_errors}, mapped to a node: {am_mapped}, node is a typed-model field: {am_on_model}, node carries a typed-model finding: {am_on_finding}");
    for (m, n) in &am_by_msg {
        println!("    {n} x {m}");
    }
    for e in &am_examples {
        println!("    {e}");
    }
}
