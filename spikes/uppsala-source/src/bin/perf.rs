//! Criterion 6 (native): parse + index time and peak heap, uppsala vs
//! imferno's current quick-xml/serde parsers.
//!
//! Usage: cargo run --release --bin perf
//! Writes the synthetic 5 MB CPL to `target/synthetic-5mb-cpl.xml` (reused by
//! the wasm bench).

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::{Duration, Instant};

use uppsala_source_spike::assetmap::{self, ST429_9_2014};
use uppsala_source_spike::corpus::{imf_corpus, rel, repo_root, root_local_name};
use uppsala_source_spike::lines::LineIndex;

struct Counting;
static CUR: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = System.alloc(l);
        if !p.is_null() {
            let now = CUR.fetch_add(l.size(), Relaxed) + l.size();
            PEAK.fetch_max(now, Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l);
        CUR.fetch_sub(l.size(), Relaxed);
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        let q = System.realloc(p, l, new);
        if !q.is_null() {
            if new > l.size() {
                let now = CUR.fetch_add(new - l.size(), Relaxed) + new - l.size();
                PEAK.fetch_max(now, Relaxed);
            } else {
                CUR.fetch_sub(l.size() - new, Relaxed);
            }
        }
        q
    }
}

#[global_allocator]
static A: Counting = Counting;

/// Median time per call and peak extra heap of one call.
fn measure<R>(mut f: impl FnMut() -> R) -> (Duration, usize) {
    // Peak heap: one call, measured above the live baseline.
    let base = CUR.load(Relaxed);
    PEAK.store(base, Relaxed);
    black_box(f());
    let peak = PEAK.load(Relaxed) - base;
    // Time: run for >= 300 ms (at least 5 runs), take the median.
    let mut samples = Vec::new();
    let start = Instant::now();
    while samples.len() < 5
        || (start.elapsed() < Duration::from_millis(300) && samples.len() < 10_000)
    {
        let t = Instant::now();
        black_box(f());
        samples.push(t.elapsed());
    }
    samples.sort();
    (samples[samples.len() / 2], peak)
}

fn fmt_d(d: Duration) -> String {
    let us = d.as_secs_f64() * 1e6;
    if us >= 1000.0 {
        format!("{:.2} ms", us / 1000.0)
    } else {
        format!("{us:.1} µs")
    }
}

fn kib(b: usize) -> String {
    format!("{:.0} KiB", b as f64 / 1024.0)
}

fn largest(root: &str) -> PathBuf {
    imf_corpus()
        .into_iter()
        .filter(|p| {
            let s = std::fs::read_to_string(p).unwrap();
            root_local_name(&s).as_deref() == Some(root)
        })
        .max_by_key(|p| std::fs::metadata(p).unwrap().len())
        .unwrap()
}

/// The largest CPL with its <Segment> repeated until the file is >= 5 MB.
pub fn synthetic_cpl() -> String {
    let base = std::fs::read_to_string(largest("CompositionPlaylist")).unwrap();
    let s = base.find("<Segment>").unwrap();
    let e = base.find("</Segment>").unwrap() + "</Segment>".len();
    let seg = &base[s..e];
    let mut out = String::from(&base[..e]);
    while out.len() + (base.len() - e) < 5 * 1024 * 1024 {
        out.push('\n');
        out.push_str(seg);
    }
    out.push_str(&base[e..]);
    out
}

fn quick_xml_events(src: &str) -> usize {
    let mut r = quick_xml::Reader::from_str(src);
    let mut n = 0;
    loop {
        match r.read_event().unwrap() {
            quick_xml::events::Event::Eof => return n,
            _ => n += 1,
        }
    }
}

fn row(label: &str, f: &mut dyn FnMut() -> usize) {
    let (t, peak) = measure(f);
    println!("| {label} | {} | {} |", fmt_d(t), kib(peak));
}

fn main() {
    let synthetic = synthetic_cpl();
    let out = repo_root().join("spikes/uppsala-source/target/synthetic-5mb-cpl.xml");
    std::fs::write(&out, &synthetic).unwrap();

    let mut inputs: Vec<(String, &str, String)> = Vec::new();
    for root in [
        "CompositionPlaylist",
        "AssetMap",
        "PackingList",
        "OutputProfileList",
    ] {
        let p = largest(root);
        inputs.push((rel(&p), root, std::fs::read_to_string(&p).unwrap()));
    }
    inputs.push((
        "synthetic 5 MB CPL".into(),
        "CompositionPlaylist",
        synthetic,
    ));
    let reg = repo_root().join("crates/imferno-core/resources/registers/Elements.xml");
    inputs.push((
        rel(&reg),
        "ElementsRegister",
        std::fs::read_to_string(&reg).unwrap(),
    ));

    for (name, root, src) in &inputs {
        let src = src.as_str();
        let nodes = uppsala::parse(src)
            .map(|d| d.descendants(d.root()).len())
            .unwrap_or(0);
        println!("\n### {name} ({} bytes, {nodes} nodes)\n", src.len());
        println!("| operation | median time | peak heap |");
        println!("|---|---|---|");
        row("uppsala::parse", &mut || {
            uppsala::parse(src)
                .unwrap()
                .children_count(uppsala::NodeId::new(0))
        });
        row("uppsala::parse + LineIndex", &mut || {
            let d = uppsala::parse(src).unwrap();
            let l = LineIndex::new(src);
            d.children_count(d.root()) + l.line_count()
        });
        row("LineIndex alone", &mut || LineIndex::new(src).line_count());
        if *root == "AssetMap" {
            row("uppsala::parse + LineIndex + typed AssetMap", &mut || {
                let d = uppsala::parse(src).unwrap();
                let l = LineIndex::new(src);
                let p = assetmap::from_document(&d, &l, &ST429_9_2014);
                p.asset_map
                    .map(|a| a.asset_list.map_or(0, |v| v.len()))
                    .unwrap_or(0)
            });
        }
        let imf: Option<(&str, Box<dyn Fn(&str) -> usize>)> = match *root {
            "CompositionPlaylist" => Some((
                "imferno parse_cpl (quick-xml serde)",
                Box::new(|s| {
                    imferno_core::cpl::parse_cpl(s)
                        .unwrap()
                        .segment_list
                        .segments
                        .len()
                }),
            )),
            "AssetMap" => Some((
                "imferno parse_assetmap (quick-xml serde)",
                Box::new(|s| {
                    imferno_core::assetmap::parse_assetmap(s)
                        .unwrap()
                        .asset_list
                        .assets
                        .len()
                }),
            )),
            "PackingList" => Some((
                "imferno parse_pkl (quick-xml serde)",
                Box::new(|s| {
                    imferno_core::assetmap::parse_pkl(s)
                        .unwrap()
                        .asset_list
                        .assets
                        .len()
                }),
            )),
            "OutputProfileList" => Some((
                "imferno parse_opl (quick-xml serde)",
                Box::new(|s| imferno_core::assetmap::parse_opl(s).unwrap().macros.len()),
            )),
            _ => None,
        };
        if let Some((label, f)) = imf {
            row(label, &mut || f(src));
        }
        row("quick-xml Reader, events only", &mut || {
            quick_xml_events(src)
        });
    }
    // XSD on the shared Document, and the cost of uppsala's O(input) per-call
    // node_line/node_column when a large file has many findings.
    let synth = &inputs
        .iter()
        .find(|i| i.0 == "synthetic 5 MB CPL")
        .unwrap()
        .2;
    let ns = "http://www.smpte-ra.org/schemas/2067-3/2016";
    let ns_in = if synth.contains(ns) {
        ns
    } else {
        "http://www.smpte-ra.org/schemas/2067-3/2013"
    };
    let v = uppsala_source_spike::xsd::cpl_validator(ns_in).unwrap();
    let bad = synth.replace("<Id>urn:uuid:", "<Id>bad:uuid:");
    println!("\n### XSD validation of the synthetic 5 MB CPL ({ns_in})\n");
    println!("| operation | median time | peak heap |");
    println!("|---|---|---|");
    for (label, text) in [
        ("clean", synth.as_str()),
        ("every <Id> invalid", bad.as_str()),
    ] {
        let doc = uppsala::parse(text).unwrap();
        let n = v.validate(&doc).len();
        row(
            &format!("validate(&doc), {label}: {n} findings"),
            &mut || v.validate(&doc).len(),
        );
    }
    println!("\nsynthetic CPL written to {}", out.display());
}
