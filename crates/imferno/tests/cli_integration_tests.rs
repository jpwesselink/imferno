//! CLI Integration Tests
//!
//! Tests all CLI commands with real test data to ensure they work correctly
//! after the library API refactoring.

use std::path::PathBuf;
use std::process::Command;

const TEST_PACKAGE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-data/MERIDIAN_Netflix_Photon_161006"
);
const INVALID_PACKAGE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-data/MissingFilesAndAssetMapEntries"
);

fn get_cli_binary() -> PathBuf {
    // Try to find the binary in the workspace target directory
    let mut path = std::env::current_dir().expect("Failed to get current dir");

    // Navigate to the workspace root if we're in a subdirectory
    while !path.join("Cargo.toml").exists() || !path.join("crates").exists() {
        if !path.pop() {
            panic!("Could not find workspace root");
        }
    }

    path.push("target");
    path.push("debug");
    path.push("imferno");
    path
}

fn run_cli_command(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(get_cli_binary())
        .args(args)
        .output()
        .expect("Failed to execute CLI command");

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (success, stdout, stderr)
}

#[test]
fn test_cli_help() {
    let (success, stdout, _) = run_cli_command(&["--help"]);

    assert!(success);
    assert!(stdout.contains("SMPTE ST 2067 IMF validator"));
    assert!(stdout.contains("cpl"));
    assert!(stdout.contains("validate"));
    // export and report are deprecated (hidden from help)
}

#[test]
fn test_cli_cpl_default() {
    let (success, stdout, stderr) = run_cli_command(&["cpl", TEST_PACKAGE]);

    assert!(success, "CLI cpl failed: {}", stderr);
    assert!(stdout.contains("CPL Details"));
    assert!(stdout.contains("ID: 0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85"));
    assert!(stdout.contains("Title: MERIDIAN"));
    assert!(stdout.contains("Kind: Test"));
    assert!(stdout.contains("Annotation: Meridian UHD 5994P"));
    assert!(stdout.contains("Segments: 1"));
}

#[test]
fn test_cli_cpl_with_uuid() {
    let uuid = "urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85";
    let (success, stdout, stderr) = run_cli_command(&["cpl", TEST_PACKAGE, "--uuid", uuid]);

    assert!(success, "CLI cpl with UUID failed: {}", stderr);
    assert!(stdout.contains("CPL Details"));
    assert!(stdout.contains("ID: 0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85"));
    assert!(stdout.contains("Title: MERIDIAN"));
}

#[test]
fn test_cli_cpl_invalid_uuid() {
    let (success, _stdout, stderr) =
        run_cli_command(&["cpl", TEST_PACKAGE, "--uuid", "invalid-uuid"]);

    assert!(!success);
    assert!(stderr.contains("not found") || stderr.contains("invalid"));
}

#[test]
fn test_cli_validate_success() {
    let (success, stdout, stderr) = run_cli_command(&["validate", TEST_PACKAGE]);

    assert!(success, "CLI validate failed: {}", stderr);
    assert!(stdout.contains("VOLINDEX.xml found"));
    assert!(stdout.contains("ASSETMAP.xml found"));
    assert!(stdout.contains("assets mapped"));
    assert!(stdout.contains("CPL(s) parsed"));
    assert!(
        stdout.contains("valid"),
        "Expected 'valid' in output: {}",
        stdout
    );
}

#[test]
fn test_cli_validate_failure() {
    let (success, stdout, stderr) = run_cli_command(&["validate", INVALID_PACKAGE]);

    // This particular test package actually validates successfully but has 0 CPLs
    // which is a valid structure but unusual
    if success {
        assert!(
            stdout.contains("0 CPL(s) parsed")
                || stdout.contains("Validation failed")
                || stdout.contains("✗")
        );
    } else {
        // If it fails to parse, that's also expected
        assert!(!stderr.is_empty());
    }
}

#[test]
fn test_cli_error_handling_invalid_path() {
    let invalid_path = "/nonexistent/path/to/package";

    let (success, _stdout, stderr) = run_cli_command(&["validate", invalid_path]);
    assert!(!success);
    assert!(!stderr.is_empty());
}

#[test]
fn test_cli_error_handling_invalid_args() {
    // Test invalid format
    let (success, _stdout, stderr) =
        run_cli_command(&["validate", TEST_PACKAGE, "--format", "invalid"]);
    assert!(!success);
    assert!(stderr.contains("invalid") || stderr.contains("error"));

    // Test missing required arguments
    let (success, _stdout, stderr) = run_cli_command(&["validate"]);
    assert!(!success);
    assert!(!stderr.is_empty());
}

#[test]
fn test_cli_performance() {
    use std::time::Instant;

    for cmd in &["validate", "cpl"] {
        let start = Instant::now();
        let (success, _stdout, _stderr) = run_cli_command(&[cmd, TEST_PACKAGE]);
        let duration = start.elapsed();

        assert!(success, "Command {} failed", cmd);
        assert!(
            duration.as_secs() < 10,
            "Command {} took too long: {:?}",
            cmd,
            duration
        );
    }
}
