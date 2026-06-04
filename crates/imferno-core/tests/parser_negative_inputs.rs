//! FIX-10: parser robustness against hostile/malformed input.
//!
//! Each public parser entry point gets the same three probes:
//! - **empty** — `""` input
//! - **malformed** — XML that doesn't close its root tag
//! - **wrong root** — a well-formed XML doc with the wrong root element
//!
//! The contract is the same across all parsers: produce a structured
//! error rather than panicking. We don't pin the exact error variant
//! because the error enums differ per parser, but `Result::is_err()`
//! is universal.

use imferno_core::assetmap::{parse_assetmap, parse_opl, parse_pkl};
use imferno_core::assetmap::volindex::parse_volindex;
use imferno_core::cpl::parse_cpl;
use imferno_core::scm::parse_scm;

// ── CPL ──────────────────────────────────────────────────────────────────────

#[test]
fn cpl_empty_input_errors() {
    assert!(parse_cpl("").is_err());
}

#[test]
fn cpl_malformed_xml_errors() {
    // Unclosed root tag.
    assert!(parse_cpl("<CompositionPlaylist>").is_err());
}

#[test]
fn cpl_wrong_root_errors() {
    // Well-formed XML, but the root isn't a CompositionPlaylist.
    let xml = r#"<?xml version="1.0"?><NotACpl><Id>x</Id></NotACpl>"#;
    assert!(parse_cpl(xml).is_err());
}

// ── AssetMap ─────────────────────────────────────────────────────────────────

#[test]
fn assetmap_empty_input_errors() {
    assert!(parse_assetmap("").is_err());
}

#[test]
fn assetmap_malformed_xml_errors() {
    assert!(parse_assetmap("<AssetMap>").is_err());
}

#[test]
fn assetmap_wrong_root_errors() {
    let xml = r#"<?xml version="1.0"?><PackingList><Id>x</Id></PackingList>"#;
    assert!(parse_assetmap(xml).is_err());
}

// ── PKL ──────────────────────────────────────────────────────────────────────

#[test]
fn pkl_empty_input_errors() {
    assert!(parse_pkl("").is_err());
}

#[test]
fn pkl_malformed_xml_errors() {
    assert!(parse_pkl("<PackingList>").is_err());
}

#[test]
fn pkl_wrong_root_errors() {
    let xml = r#"<?xml version="1.0"?><AssetMap><Id>x</Id></AssetMap>"#;
    assert!(parse_pkl(xml).is_err());
}

// ── OPL ──────────────────────────────────────────────────────────────────────

#[test]
fn opl_empty_input_errors() {
    assert!(parse_opl("").is_err());
}

#[test]
fn opl_malformed_xml_errors() {
    assert!(parse_opl("<OutputProfileList>").is_err());
}

#[test]
fn opl_wrong_root_errors() {
    let xml = r#"<?xml version="1.0"?><CompositionPlaylist><Id>x</Id></CompositionPlaylist>"#;
    assert!(parse_opl(xml).is_err());
}

// ── SCM ──────────────────────────────────────────────────────────────────────

#[test]
fn scm_empty_input_errors() {
    assert!(parse_scm("").is_err());
}

#[test]
fn scm_malformed_xml_errors() {
    assert!(parse_scm("<SidecarCompositionMap>").is_err());
}

#[test]
fn scm_wrong_root_errors() {
    let xml = r#"<?xml version="1.0"?><PackingList><Id>x</Id></PackingList>"#;
    assert!(parse_scm(xml).is_err());
}

// ── VolumeIndex ──────────────────────────────────────────────────────────────

#[test]
fn volindex_empty_input_errors() {
    assert!(parse_volindex("").is_err());
}

#[test]
fn volindex_malformed_xml_errors() {
    assert!(parse_volindex("<VolumeIndex>").is_err());
}

#[test]
fn volindex_wrong_root_errors() {
    let xml = r#"<?xml version="1.0"?><CompositionPlaylist/>"#;
    assert!(parse_volindex(xml).is_err());
}
