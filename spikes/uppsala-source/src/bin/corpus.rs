//! Criterion 1 (+2): range fidelity over the corpus, and parser acceptance
//! compared with quick-xml and imferno's current parsers.
//!
//! Usage: cargo run --release --bin corpus

use std::collections::BTreeMap;

use uppsala_source_spike::corpus::{extra_corpus, imf_corpus, rel, root_local_name};
use uppsala_source_spike::invariants::{check, Options, Stats};

fn quick_xml_accepts(src: &str) -> Result<(), String> {
    let mut r = quick_xml::Reader::from_str(src);
    loop {
        match r.read_event() {
            Ok(quick_xml::events::Event::Eof) => return Ok(()),
            Ok(_) => {}
            Err(e) => return Err(format!("at byte {}: {e}", r.buffer_position())),
        }
    }
}

/// imferno's current typed parser for this root element, if there is one.
fn imferno_parse(root: &str, src: &str) -> Option<Result<(), String>> {
    use imferno_core::{assetmap, cpl, scm};
    let r = match root {
        "CompositionPlaylist" => cpl::parse_cpl(src).map(|_| ()).map_err(|e| e.to_string()),
        "AssetMap" => assetmap::parse_assetmap(src)
            .map(|_| ())
            .map_err(|e| e.to_string()),
        "PackingList" => assetmap::parse_pkl(src)
            .map(|_| ())
            .map_err(|e| e.to_string()),
        "OutputProfileList" => assetmap::parse_opl(src)
            .map(|_| ())
            .map_err(|e| e.to_string()),
        "VolumeIndex" => assetmap::parse_volindex(src)
            .map(|_| ())
            .map_err(|e| e.to_string()),
        "SidecarCompositionMap" => scm::parse_scm(src).map(|_| ()).map_err(|e| e.to_string()),
        _ => return None,
    };
    Some(r)
}

fn add(total: &mut Stats, s: &Stats) {
    total.nodes += s.nodes;
    total.elements += s.elements;
    total.texts += s.texts;
    total.comments += s.comments;
    total.cdata += s.cdata;
    total.pis += s.pis;
    total.attributes += s.attributes;
    total.namespace_decls += s.namespace_decls;
    total.unverifiable_values += s.unverifiable_values;
    total.leaf_bytes += s.leaf_bytes;
    total.tag_bytes += s.tag_bytes;
    total.outside_bytes += s.outside_bytes;
}

fn main() {
    for (label, files) in [("IMF corpus", imf_corpus()), ("extra XML", extra_corpus())] {
        println!("== {label}: {} files", files.len());
        let mut total = Stats::default();
        let mut bytes = 0usize;
        let mut parsed = 0usize;
        let mut parsed_bytes = 0usize;
        let mut not_utf8 = Vec::new();
        let mut uppsala_rejects = Vec::new();
        let mut violations = 0usize;
        let mut by_root: BTreeMap<String, usize> = BTreeMap::new();
        let mut acceptance: BTreeMap<&str, usize> = BTreeMap::new();
        let mut disagreements = Vec::new();

        for path in &files {
            let raw = std::fs::read(path).expect("read");
            bytes += raw.len();
            let src = match String::from_utf8(raw) {
                Ok(s) => s,
                Err(e) => {
                    not_utf8.push(format!("{}: {e}", rel(path)));
                    continue;
                }
            };
            let root = root_local_name(&src).unwrap_or_default();
            *by_root.entry(root.clone()).or_default() += 1;
            let up = uppsala::parse(&src);
            let qx = quick_xml_accepts(&src);
            let imf = imferno_parse(&root, &src);
            let key = match (&up, &qx) {
                (Ok(_), Ok(_)) => "both accept",
                (Ok(_), Err(_)) => "uppsala only",
                (Err(_), Ok(_)) => "quick-xml only",
                (Err(_), Err(_)) => "both reject",
            };
            *acceptance.entry(key).or_default() += 1;
            if let (Err(e), Some(Ok(()))) = (&up, &imf) {
                disagreements.push(format!(
                    "{}: imferno parse_* accepts, uppsala rejects: {e}",
                    rel(path)
                ));
            }
            if let (Ok(_), Err(e)) = (&up, &qx) {
                disagreements.push(format!("{}: quick-xml rejects: {e}", rel(path)));
            }
            match up {
                Err(e) => uppsala_rejects.push(format!(
                    "{}: {e} | quick-xml: {} | imferno: {}",
                    rel(path),
                    qx.err().unwrap_or_else(|| "accepts".into()),
                    match imf {
                        None => "n/a".into(),
                        Some(Ok(())) => "accepts".into(),
                        Some(Err(e)) => format!("rejects ({e})"),
                    }
                )),
                Ok(doc) => {
                    parsed += 1;
                    parsed_bytes += src.len();
                    let (stats, v) = check(&src, &doc, &Options::default());
                    add(&mut total, &stats);
                    violations += v.len();
                    for x in v.iter().take(5) {
                        let snippet: String = src[x.range.clone()].chars().take(60).collect();
                        println!(
                            "  VIOLATION {}: {} {:?} {snippet:?}",
                            rel(path),
                            x.what,
                            x.range
                        );
                    }
                }
            }
        }
        println!("  bytes: {bytes}");
        println!("  roots: {by_root:?}");
        println!("  not UTF-8: {}", not_utf8.len());
        for x in &not_utf8 {
            println!("    {x}");
        }
        println!(
            "  uppsala parsed: {parsed}, rejected: {}",
            uppsala_rejects.len()
        );
        for x in &uppsala_rejects {
            println!("    {x}");
        }
        println!("  acceptance vs quick-xml reader: {acceptance:?}");
        for x in &disagreements {
            println!("    {x}");
        }
        println!("  invariant violations: {violations}");
        println!("  stats: {total:?}");
        let covered = total.leaf_bytes + total.tag_bytes + total.outside_bytes;
        println!(
            "  byte accounting: leaf {} + tags {} + outside {} = {covered} (parsed files total {parsed_bytes})",
            total.leaf_bytes, total.tag_bytes, total.outside_bytes
        );
    }
}
