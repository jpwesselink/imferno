//! Spike: can uppsala serve as imferno's lossless source layer?
//!
//! See `REPORT.md` for the criteria and results. Nothing here is production
//! code; it measures.

pub mod assetmap;
pub mod corpus;
pub mod invariants;
pub mod lines;
pub mod xsd;

use std::ops::Range;

/// Replace `range` of `src` with `with`. The editor primitive: everything
/// outside `range` is copied unchanged.
pub fn splice(src: &str, range: Range<usize>, with: &str) -> String {
    let mut out = String::with_capacity(src.len() - range.len() + with.len());
    out.push_str(&src[..range.start]);
    out.push_str(with);
    out.push_str(&src[range.end..]);
    out
}

/// Byte-level diff summary of two texts: the single changed window
/// (common prefix / suffix stripped). `None` if identical.
pub fn changed_window(a: &str, b: &str) -> Option<(Range<usize>, Range<usize>)> {
    let (ab, bb) = (a.as_bytes(), b.as_bytes());
    let prefix = ab.iter().zip(bb).take_while(|(x, y)| x == y).count();
    if prefix == ab.len() && prefix == bb.len() {
        return None;
    }
    let max_suffix = ab.len().min(bb.len()) - prefix;
    let suffix = ab
        .iter()
        .rev()
        .zip(bb.iter().rev())
        .take(max_suffix)
        .take_while(|(x, y)| x == y)
        .count();
    Some((prefix..ab.len() - suffix, prefix..bb.len() - suffix))
}

#[cfg(feature = "wasm")]
mod wasm {
    use wasm_bindgen::prelude::*;

    /// Parse `xml` `iters` times with uppsala; returns total node count so
    /// the work cannot be optimised away.
    #[wasm_bindgen]
    pub fn bench_uppsala(xml: &str, iters: u32) -> u32 {
        let mut n = 0;
        for _ in 0..iters {
            let doc = uppsala::parse(xml).expect("parse");
            n += doc.descendants(doc.root()).len() as u32;
        }
        n
    }

    /// uppsala parse + line index (the source layer's index).
    #[wasm_bindgen]
    pub fn bench_uppsala_indexed(xml: &str, iters: u32) -> u32 {
        let mut n = 0;
        for _ in 0..iters {
            let doc = uppsala::parse(xml).expect("parse");
            let idx = crate::lines::LineIndex::new(xml);
            n += doc.descendants(doc.root()).len() as u32 + idx.line_count() as u32;
        }
        n
    }

    /// imferno's current CPL parse (quick-xml + serde).
    #[wasm_bindgen]
    pub fn bench_imferno_cpl(xml: &str, iters: u32) -> u32 {
        let mut n = 0;
        for _ in 0..iters {
            let cpl = imferno_core::cpl::parse_cpl(xml).expect("parse_cpl");
            n += cpl.segment_list.segments.len() as u32;
        }
        n
    }

    /// Linear-memory size in bytes. Wasm memory never shrinks, so after a
    /// call this is the high-water mark of the whole instance.
    #[wasm_bindgen]
    pub fn wasm_memory_bytes() -> f64 {
        (core::arch::wasm32::memory_size(0) * 65536) as f64
    }

    /// quick-xml raw event loop (no serde), as a floor.
    #[wasm_bindgen]
    pub fn bench_quick_xml_events(xml: &str, iters: u32) -> u32 {
        let mut n = 0;
        for _ in 0..iters {
            let mut r = quick_xml::Reader::from_str(xml);
            loop {
                match r.read_event().expect("event") {
                    quick_xml::events::Event::Eof => break,
                    _ => n += 1,
                }
            }
        }
        n
    }
}
