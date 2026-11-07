//! `ValidationResult` must survive a JSON round trip losslessly: the JS
//! bindings' `formatReport` deserialises the object `validate()` returned.
//! Checked over every IMP in `test-data/` (valid and invalid).

use imferno_core::package::{
    format_validation_result, read_dir, validate, FormatOptions, ValidationOptions,
    ValidationResult,
};
use std::path::{Path, PathBuf};

fn find_packages(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut has_assetmap = false;
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            find_packages(&p, out);
        } else if p
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.to_ascii_lowercase().starts_with("assetmap"))
        {
            has_assetmap = true;
        }
    }
    if has_assetmap {
        out.push(dir.to_path_buf());
    }
}

#[test]
fn validation_result_json_round_trips() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data");
    let mut packages = Vec::new();
    find_packages(&root, &mut packages);
    assert!(packages.len() > 20, "expected the test-data IMPs");

    for dir in packages {
        let files = read_dir(&dir).expect("read package");
        let result = validate(files, &ValidationOptions::default());
        let json = serde_json::to_value(&result).expect("serialize");
        let back = ValidationResult::from_json(json.clone())
            .unwrap_or_else(|e| panic!("{}: deserialize failed: {e}", dir.display()));
        let again = serde_json::to_value(&back).expect("re-serialize");
        assert_eq!(
            json,
            again,
            "{}: round trip changed the value",
            dir.display()
        );
        // CPLs live in a HashMap, so their print order is arbitrary; compare
        // the formatted lines as a multiset.
        let lines = |r: &ValidationResult| {
            let mut v: Vec<String> = format_validation_result(r, &FormatOptions::default())
                .lines()
                .map(str::to_owned)
                .collect();
            v.sort();
            v
        };
        assert_eq!(
            lines(&result),
            lines(&back),
            "{}: formatted output differs after round trip",
            dir.display()
        );
    }
}
