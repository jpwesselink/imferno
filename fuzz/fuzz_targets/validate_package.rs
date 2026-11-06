//! End-to-end package validation over an in-memory file set, as the WASM
//! playground and NAPI bindings call it. Exercises cross-file logic (asset
//! map ↔ packing list ↔ CPL references) and the XSD layer.
//!
//! Input format: documents separated by a line containing only `\x1e`
//! (ASCII record separator). Each document is named after its root element
//! so the package loader classifies it like a real file. File names are
//! bare (no directory part) so `validate` never derives a filesystem root.
#![no_main]

use std::collections::HashMap;

use imferno_core::package::{validate, ValidationOptions};
use libfuzzer_sys::fuzz_target;

const SEPARATOR: &str = "\n\u{1e}\n";
const MAX_FILES: usize = 8;

fn file_name(doc: &str, index: usize) -> String {
    // Classify on the first ~512 bytes, backing off to a char boundary.
    let mut end = doc.len().min(512);
    while !doc.is_char_boundary(end) {
        end -= 1;
    }
    let head = &doc[..end];
    if head.contains("AssetMap") {
        "ASSETMAP.xml".to_string()
    } else if head.contains("VolumeIndex") {
        "VOLINDEX.xml".to_string()
    } else if head.contains("PackingList") {
        format!("PKL_{index}.xml")
    } else if head.contains("OutputProfileList") {
        format!("OPL_{index}.xml")
    } else if head.contains("SidecarCompositionMap") {
        format!("SCM_{index}.xml")
    } else {
        format!("CPL_{index}.xml")
    }
}

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let mut files = HashMap::new();
    for (i, doc) in text.split(SEPARATOR).take(MAX_FILES).enumerate() {
        files.insert(file_name(doc, i), doc.to_string());
    }
    let _ = validate(files, &ValidationOptions::default());
});
