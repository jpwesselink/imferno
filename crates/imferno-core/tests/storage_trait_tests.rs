//! Integration tests for the Storage trait against built-in backends.

use imferno_core::storage::{fs::FsStorage, Storage, StorageUri};
use std::fs;
use tempfile::TempDir;

fn make_fixture() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("ASSETMAP.xml"), "<assetmap/>").unwrap();
    fs::write(dir.path().join("CPL_test.xml"), "<cpl/>").unwrap();
    // 0xFF 0xFE 0xFD is not a valid UTF-8 sequence
    fs::write(dir.path().join("data.bin"), [0xFFu8, 0xFE, 0xFD]).unwrap();
    dir
}

#[test]
fn fs_list_returns_all_entries() {
    let dir = make_fixture();
    let storage = FsStorage::new();
    let uri = StorageUri::parse(dir.path().to_str().unwrap()).unwrap();

    let entries = storage.list(&uri).unwrap();
    assert_eq!(entries.len(), 3);
    assert!(entries.iter().all(|e| e.is_file));
}

#[test]
fn fs_read_to_string_returns_xml_content() {
    let dir = make_fixture();
    let storage = FsStorage::new();
    let cpl_uri_str = format!("file://{}", dir.path().join("CPL_test.xml").display());
    let uri = StorageUri::parse(&cpl_uri_str).unwrap();

    let content = storage.read_to_string(&uri).unwrap();
    assert_eq!(content, "<cpl/>");
}

#[test]
fn fs_read_to_string_errors_on_non_utf8() {
    let dir = make_fixture();
    let storage = FsStorage::new();
    let bin_uri_str = format!("file://{}", dir.path().join("data.bin").display());
    let uri = StorageUri::parse(&bin_uri_str).unwrap();

    assert!(storage.read_to_string(&uri).is_err());
}
