//! SARIF 2.1.0 output validates against the official SARIF 2.1.0 JSON
//! schema (vendored from serde-sarif, `tests/fixtures/`), for a valid and
//! an invalid package.

use std::path::PathBuf;

use imferno_core::diagnostics::{to_sarif_with_root, RuleSeverity, RulesConfig};
use imferno_core::package::{read_dir, Imferno, ValidationOptions};

fn schema() -> jsonschema::Validator {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sarif-schema-2.1.0.json");
    // The official schema's run.language pattern has an unescaped `]`
    // (`...{2}]?$`), a literal in ECMA-262 regexes but a syntax error for
    // the Rust regex engine the validator uses to check the schema itself;
    // escape it (same meaning) before compiling.
    let text = std::fs::read_to_string(path)
        .unwrap()
        .replace(r"[a-zA-Z]{2}]?$", r"[a-zA-Z]{2}\\]?$");
    let schema: serde_json::Value = serde_json::from_str(&text).unwrap();
    jsonschema::validator_for(&schema).expect("SARIF schema compiles")
}

fn sarif_for(package: &str, options: &ValidationOptions) -> serde_json::Value {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-data")
        .join(package);
    let pkg = Imferno::parse(read_dir(&dir).unwrap()).expect("package parses");
    serde_json::to_value(to_sarif_with_root(
        &pkg.validate(options),
        Some(&pkg.root_path),
    ))
    .unwrap()
}

fn assert_valid(log: &serde_json::Value) {
    let validator = schema();
    let errors: Vec<String> = validator
        .iter_errors(log)
        .map(|e| format!("{} at {}", e, e.instance_path()))
        .collect();
    assert!(
        errors.is_empty(),
        "SARIF schema violations:\n{}",
        errors.join("\n")
    );
}

fn levels(log: &serde_json::Value) -> Vec<String> {
    log["runs"][0]["results"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r.get("suppressions").is_none())
        .map(|r| r["level"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn valid_package_sarif_validates_against_the_schema() {
    // The Photon MERIDIAN package is valid apart from one known MXF MCA
    // link mismatch, switched off here (and reported as suppressed).
    let mut rules = RulesConfig::default();
    rules.set_raw("SoundfieldGroupLinkIDMismatch".into(), RuleSeverity::Off);
    let log = sarif_for(
        "MERIDIAN_Netflix_Photon_161006",
        &ValidationOptions {
            rules,
            ..ValidationOptions::default()
        },
    );
    assert_valid(&log);
    assert!(!levels(&log).iter().any(|l| l == "error"), "{log:#}");
    assert_eq!(log["version"], "2.1.0");
    assert_eq!(log["runs"][0]["properties"]["isCompliant"], true);
    let suppressed = log["runs"][0]["results"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r.get("suppressions").is_some())
        .expect("suppressed finding is reported with a suppression");
    assert_eq!(
        suppressed["ruleId"],
        "ST377-4/SoundfieldGroupLinkIDMismatch"
    );
    // Reported at the level it had before the rules config switched it off.
    assert_eq!(suppressed["level"], "error");
    assert_eq!(suppressed["properties"]["originalSeverity"], "Error");

    // Artifact URIs are relative to the package root.
    let root = log["runs"][0]["originalUriBaseIds"]["PKGROOT"]["uri"]
        .as_str()
        .unwrap();
    assert!(
        root.starts_with("file:///") && root.ends_with('/'),
        "{root}"
    );
    let art = &suppressed["locations"][0]["physicalLocation"]["artifactLocation"];
    assert_eq!(art["uriBaseId"], "PKGROOT");
    assert!(!art["uri"].as_str().unwrap().starts_with('/'), "{art}");
}

#[test]
fn invalid_package_sarif_validates_against_the_schema() {
    let log = sarif_for(
        "MERIDIAN_Netflix_Photon_161006_ID_MISMATCH",
        &ValidationOptions::default(),
    );
    assert_valid(&log);
    assert!(levels(&log).iter().any(|l| l == "error"), "{log:#}");
    assert_eq!(log["runs"][0]["properties"]["isCompliant"], false);

    // Every result points at a driver rule and at the applied edition's
    // clause in that edition's taxonomy.
    let run = &log["runs"][0];
    let rules = run["tool"]["driver"]["rules"].as_array().unwrap();
    let taxonomies = run["taxonomies"].as_array().unwrap();
    for r in run["results"].as_array().unwrap() {
        let idx = r["ruleIndex"].as_u64().unwrap() as usize;
        assert_eq!(rules[idx]["id"], r["ruleId"]);
        let taxon = &r["taxa"][0];
        let t = taxon["toolComponent"]["index"].as_u64().unwrap() as usize;
        assert_eq!(taxonomies[t]["name"], taxon["toolComponent"]["name"]);
        let code = r["properties"]["code"].as_str().unwrap();
        let name = taxon["toolComponent"]["name"].as_str().unwrap();
        assert!(code.starts_with(name), "{code} vs taxonomy {name}");
    }
}

#[test]
fn a_schema_violation_is_detected() {
    // Guard against a validator that accepts anything.
    let bad = serde_json::json!({ "version": "2.1.0", "runs": [{ "results": [] }] });
    assert!(!schema().is_valid(&bad));
}
