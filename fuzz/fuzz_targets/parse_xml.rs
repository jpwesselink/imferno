//! Every public XML parser must return `Ok` or `Err` for any input — never
//! panic. Release builds use `panic = "abort"`, so a parser panic takes down
//! the host process (CLI, Node via NAPI, or the browser's WASM module).
//!
//! Each input is fed to all parsers; the seed corpus holds real ASSETMAP,
//! PKL, CPL, OPL, VOLINDEX and SCM documents, so mutations of one document
//! type also probe how the other parsers handle near-miss XML.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(xml) = std::str::from_utf8(data) else {
        return;
    };
    let _ = imferno_core::assetmap::parse_assetmap(xml);
    let _ = imferno_core::assetmap::parse_pkl(xml);
    let _ = imferno_core::assetmap::parse_opl(xml);
    let _ = imferno_core::assetmap::volindex::parse_volindex(xml);
    let _ = imferno_core::scm::parse_scm(xml);
    if let Ok(cpl) = imferno_core::cpl::parse_cpl(xml) {
        // Semantic + schema validation of whatever parsed.
        let _ = imferno_core::validation::validate_cpl(&cpl);
        let _ = imferno_core::xsd::validate_parsed_cpl(&cpl);
    }
});
