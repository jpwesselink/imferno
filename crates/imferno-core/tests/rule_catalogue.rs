//! Contract tests for the per-edition rule catalogue (C0).
//!
//! - The catalogue snapshot (`tests/snapshots/validation-codes.txt`) pins
//!   every rule × edition row: a change to a clause, a severity or an
//!   edition's coverage fails here and has to be reviewed.
//! - Every rule carries an example and the source its spec implies.
//! - No validation code sets a severity or builds a finding by hand: the
//!   only constructor is `ValidationIssue::from_rule`.
//! - Every validator of every edition runs over every CPL fixture; reporting
//!   a rule for an edition that does not contain it panics in debug builds,
//!   so this exercises the emission sites against all editions.

use std::path::{Path, PathBuf};

use imferno_core::catalogue;
use imferno_core::diagnostics::{IssueSource, Spec};
use imferno_core::validation::{
    app2e_validator, App2EEdition, AppIabPlugin2019, AppIabPlugin2021, AppIabPlugin2026,
    AppIsxdPlugin2023, ConstraintsValidator, CoreConstraints2013, CoreConstraints2016,
    CoreConstraints2020,
};

fn snapshot() -> String {
    let mut lines = Vec::new();
    for rule in catalogue::all() {
        for row in rule.edition_rows() {
            let edition = if row.edition.is_empty() {
                "-".to_string()
            } else {
                row.edition.clone()
            };
            let line = match &row.code {
                Some(code) => format!(
                    "{} | {} | {} | {:?} | {} | {}",
                    rule.id(),
                    edition,
                    code,
                    row.default_severity,
                    row.strength,
                    if row.verified {
                        "verified"
                    } else {
                        "unverified"
                    }
                ),
                None => format!("{} | {} | absent", rule.id(), edition),
            };
            lines.push(line);
        }
    }
    lines.join("\n")
}

fn snapshot_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/validation-codes.txt")
}

#[test]
fn rule_catalogue_is_stable() {
    let actual = snapshot();
    let expected = std::fs::read_to_string(snapshot_path()).unwrap_or_default();
    if actual != expected.trim_end() {
        let exp: Vec<&str> = expected.trim_end().lines().collect();
        let act: Vec<&str> = actual.lines().collect();
        let mut msg = String::from("Rule catalogue changed!\n\n");
        for l in act.iter().filter(|l| !exp.contains(l)) {
            msg.push_str(&format!("  + {l}\n"));
        }
        for l in exp.iter().filter(|l| !act.contains(l)) {
            msg.push_str(&format!("  - {l}\n"));
        }
        msg.push_str(
            "\nIf intentional: cargo test -p imferno-core --test rule_catalogue -- --ignored\n",
        );
        panic!("{msg}");
    }
}

#[test]
#[ignore]
fn update_rule_catalogue_snapshot() {
    std::fs::write(snapshot_path(), snapshot() + "\n").unwrap();
}

#[test]
fn every_rule_has_an_example_and_the_source_of_its_spec() {
    for rule in catalogue::all() {
        assert!(rule.example().is_some(), "{} has no example", rule.id());
        let expected = match rule.spec() {
            Spec::Xsd => IssueSource::XsdLayer,
            Spec::Imferno => IssueSource::EngineInternal,
            _ => IssueSource::ProseRule,
        };
        assert_eq!(rule.spec().source(), expected);
    }
}

#[test]
fn rule_info_round_trips_through_json() {
    let infos = catalogue::rule_infos();
    let json = serde_json::to_string(&infos).unwrap();
    let back: Vec<imferno_core::diagnostics::rule::RuleInfo> = serde_json::from_str(&json).unwrap();
    assert_eq!(back, infos);
    let seg = infos
        .iter()
        .find(|r| r.id.0 == "ST2067-3/SegmentDuration")
        .unwrap();
    assert_eq!(seg.editions.len(), 2);
    assert_eq!(
        seg.editions[0].code.as_deref(),
        Some("ST2067-3:2013:7.2.2/SegmentDuration")
    );
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let p = entry.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

/// C3: severity comes only from the rule tables. Validation code may not
/// assign a severity or build a `ValidationIssue` literal; the only places
/// allowed to are the issue constructor and the rules-config remapping.
#[test]
fn no_validation_code_sets_its_own_severity() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let allowed = ["diagnostics/mod.rs", "diagnostics/rules.rs"];
    let assign = regex::Regex::new(r"\.severity\s*=[^=]").unwrap();
    let literal = regex::Regex::new(r"ValidationIssue\s*\{").unwrap();
    let mut offenders = Vec::new();
    for f in files {
        let rel = f
            .strip_prefix(&src)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if allowed.contains(&rel.as_str()) {
            continue;
        }
        let text = std::fs::read_to_string(&f).unwrap();
        for (n, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            let builds_literal =
                literal.is_match(code) && !code.contains("->") && !code.contains("impl ");
            if assign.is_match(code) || builds_literal {
                offenders.push(format!("{rel}:{}: {}", n + 1, line.trim()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "findings must take their severity from the rule tables:\n{}",
        offenders.join("\n")
    );
}

fn xml_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            xml_files(&p, out);
        } else if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("xml")) {
            out.push(p);
        }
    }
}

/// Every validator of every edition over every CPL fixture. A rule reported
/// for an edition whose table row lacks a clause panics (debug_assert in
/// `ValidationIssue::from_rule`), so a pass means every emission site only
/// reports rules its edition contains — for all editions.
#[test]
fn every_edition_validator_reports_only_rules_of_its_edition() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-data");
    let mut files = Vec::new();
    xml_files(&root, &mut files);
    let mut validators: Vec<Box<dyn ConstraintsValidator>> = vec![
        Box::new(CoreConstraints2013),
        Box::new(CoreConstraints2016),
        Box::new(CoreConstraints2020),
        Box::new(AppIabPlugin2019),
        Box::new(AppIabPlugin2021),
        Box::new(AppIabPlugin2026),
        Box::new(AppIsxdPlugin2023),
    ];
    for e in App2EEdition::ALL {
        validators.push(app2e_validator(*e));
    }
    let mut cpls = 0;
    for f in files {
        let Ok(xml) = std::fs::read_to_string(&f) else {
            continue;
        };
        if !xml.contains("CompositionPlaylist") {
            continue;
        }
        let Ok(cpl) = imferno_core::cpl::parse_cpl(&xml) else {
            continue;
        };
        cpls += 1;
        for v in &validators {
            for issue in v.validate_cpl(&cpl) {
                assert!(
                    !issue.code.contains("RuleNotInEdition"),
                    "{}: {}",
                    v.spec_id(),
                    issue.message
                );
            }
        }
        for issue in imferno_core::cpl::validate_cpl_constraints(&cpl) {
            assert!(!issue.code.contains("RuleNotInEdition"));
        }
    }
    assert!(cpls > 0, "no CPL fixtures found under {}", root.display());
}
