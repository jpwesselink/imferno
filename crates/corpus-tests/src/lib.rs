//! Shared helpers for corpus-based regression tests.
//!
//! All tests in this crate read real IMF package files from the workspace
//! `test-data/` directory and run them through the public parser + validator
//! APIs. They are ported from the Netflix Photon Java test suite.

use std::path::PathBuf;

/// Absolute path to the workspace `test-data/` directory.
pub fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap() // crates/
        .parent()
        .unwrap() // workspace root
        .join("test-data")
}

/// Read and parse a CPL from a path relative to `test-data/`.
///
/// Panics with a clear message if the file cannot be read or parsed.
pub fn read_cpl(rel: &str) -> imferno_core::cpl::CompositionPlaylist {
    let path = corpus_dir().join(rel);
    let xml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    imferno_core::cpl::parse_cpl(&xml)
        .unwrap_or_else(|e| panic!("cannot parse {}: {e:?}", path.display()))
}

/// Parse and validate an IMF package from an in-memory file map.
///
/// Keys are plain basenames (no absolute paths), so `root_path` is empty
/// and file-manifest / MXF-header checks are skipped. Only structural and
/// reference validation runs — ideal for SCM and other XML-only tests.
pub fn validate_package(files: std::collections::HashMap<String, String>) -> imferno_core::ValidationReport {
    imferno_core::package::Imferno::parse_and_validate(files, &imferno_core::package::ValidationOptions::default())
}

/// Filter issues down to errors and criticals.
pub fn errors(issues: &[imferno_core::ValidationIssue]) -> Vec<&imferno_core::ValidationIssue> {
    issues
        .iter()
        .filter(|i| {
            i.severity == imferno_core::Severity::Error
                || i.severity == imferno_core::Severity::Critical
        })
        .collect()
}
