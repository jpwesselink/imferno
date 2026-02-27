//! Snapshot test for validation code stability.
//!
//! These codes form a public API contract — downstream consumers match on them.
//! If a code changes, this test fails, forcing a conscious review.

use imferno_core::assetmap::codes::{
    St2067_2_2013_Core, St2067_2_2016_Core, St2067_2_2020, St2067_2_2020_Core,
};
use imferno_core::assetmap::volindex_codes::St429_9_2014;
use imferno_core::cpl::codes::{St2067_3_2013, St2067_3_2016, St2067_3_2020};
use imferno_core::diagnostics::codes::ValidationCode;
use imferno_core::mxf::codes::St377_1_2011;
use imferno_core::package::codes::ImfernoCode;
use imferno_core::scm::codes::St2067_9_2018;
use imferno_core::validation::codes::{St2067_21_2020, St2067_21_2023, St2067_21_2025};
use imferno_core::validation::iab_codes::{St2067_201_2019, St2067_201_2021};
use imferno_core::validation::isxd_codes::St2067_202_2022;

fn collect_all_codes() -> Vec<String> {
    let mut codes: Vec<String> = Vec::new();

    for v in St429_9_2014::ALL {
        codes.push(v.code().to_string());
    }
    for v in St377_1_2011::ALL {
        codes.push(v.code().to_string());
    }
    for v in St2067_2_2020::ALL {
        codes.push(v.code().to_string());
    }
    for v in St2067_2_2013_Core::ALL {
        codes.push(v.code().to_string());
    }
    for v in St2067_2_2016_Core::ALL {
        codes.push(v.code().to_string());
    }
    for v in St2067_2_2020_Core::ALL {
        codes.push(v.code().to_string());
    }
    for v in St2067_3_2013::ALL {
        codes.push(v.code().to_string());
    }
    for v in St2067_3_2016::ALL {
        codes.push(v.code().to_string());
    }
    for v in St2067_3_2020::ALL {
        codes.push(v.code().to_string());
    }
    for v in St2067_9_2018::ALL {
        codes.push(v.code().to_string());
    }
    for v in St2067_21_2020::ALL {
        codes.push(v.code().to_string());
    }
    for v in St2067_21_2023::ALL {
        codes.push(v.code().to_string());
    }
    for v in St2067_21_2025::ALL {
        codes.push(v.code().to_string());
    }
    for v in St2067_201_2019::ALL {
        codes.push(v.code().to_string());
    }
    for v in St2067_201_2021::ALL {
        codes.push(v.code().to_string());
    }
    for v in St2067_202_2022::ALL {
        codes.push(v.code().to_string());
    }
    for v in ImfernoCode::ALL {
        codes.push(v.code().to_string());
    }

    codes.sort();
    codes
}

#[test]
fn validation_codes_are_stable() {
    let codes = collect_all_codes();
    let snapshot = codes.join("\n");

    let snapshot_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots/validation-codes.txt");

    if let Ok(expected) = std::fs::read_to_string(&snapshot_path) {
        if snapshot != expected.trim_end() {
            // Show a clear diff
            let expected_lines: Vec<&str> = expected.trim_end().lines().collect();
            let actual_lines: Vec<&str> = snapshot.lines().collect();

            let mut added = Vec::new();
            let mut removed = Vec::new();

            for line in &actual_lines {
                if !expected_lines.contains(line) {
                    added.push(*line);
                }
            }
            for line in &expected_lines {
                if !actual_lines.contains(line) {
                    removed.push(*line);
                }
            }

            let mut msg = String::from("Validation codes changed!\n\n");
            if !added.is_empty() {
                msg.push_str("Added:\n");
                for line in &added {
                    msg.push_str(&format!("  + {line}\n"));
                }
            }
            if !removed.is_empty() {
                msg.push_str("Removed:\n");
                for line in &removed {
                    msg.push_str(&format!("  - {line}\n"));
                }
            }
            msg.push_str("\nIf this is intentional, update the snapshot:\n");
            msg.push_str("  cargo test -p imferno-core validation_codes_are_stable -- --ignored\n");
            panic!("{msg}");
        }
    } else {
        // No snapshot yet — create it
        std::fs::create_dir_all(snapshot_path.parent().unwrap()).unwrap();
        std::fs::write(&snapshot_path, &snapshot).unwrap();
        println!("Created snapshot at {}", snapshot_path.display());
        println!("{} validation codes captured", codes.len());
    }
}

#[test]
#[ignore]
fn update_validation_code_snapshot() {
    let codes = collect_all_codes();
    let snapshot = codes.join("\n");

    let snapshot_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots/validation-codes.txt");

    std::fs::create_dir_all(snapshot_path.parent().unwrap()).unwrap();
    std::fs::write(&snapshot_path, &snapshot).unwrap();
    println!("Updated snapshot at {}", snapshot_path.display());
    println!("{} validation codes captured", codes.len());
}

#[test]
fn no_unexpected_duplicate_validation_codes() {
    // Known duplicates: codes that intentionally appear in multiple enums
    // (e.g., same code in both package-level and core-constraints validators)
    let known_duplicates: std::collections::HashSet<&str> = [
        "ST2067-2:2020:6.4.2/EssenceDescriptorList",
    ]
    .into_iter()
    .collect();

    let codes = collect_all_codes();
    let mut seen = std::collections::HashSet::new();
    let mut dupes = Vec::new();

    for code in &codes {
        if !seen.insert(code.as_str()) && !known_duplicates.contains(code.as_str()) {
            dupes.push(code.as_str());
        }
    }

    assert!(
        dupes.is_empty(),
        "Unexpected duplicate validation codes found: {dupes:?}"
    );
}

#[test]
fn validation_codes_follow_format() {
    let codes = collect_all_codes();
    let pattern = regex::Regex::new(
        r"^(ST\d+-\d+(:\d{4})?(:[A-Za-z0-9._/-]+)|IMF(ERNO)?:[A-Za-z/_-]+)$"
    ).unwrap();

    let mut bad = Vec::new();
    for code in &codes {
        if !pattern.is_match(code) {
            bad.push(code.as_str());
        }
    }

    assert!(
        bad.is_empty(),
        "Codes not matching expected format: {bad:?}"
    );
}
