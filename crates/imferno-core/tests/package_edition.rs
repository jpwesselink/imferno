//! The ST 2067-2 edition applied to package-level rules (Packing List,
//! Asset Map, ingest), and the delivery rules of a 2013 package.

use std::path::PathBuf;

use imferno_core::catalogue::{imferno, st2067_2::St2067_2Edition};
use imferno_core::package::{read_dir, Imferno, ValidationOptions};
use imferno_core::validation::CoreSpecTarget;
use imferno_core::ValidationIssue;

fn package(name: &str) -> Imferno {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-data")
        .join(name);
    Imferno::parse(read_dir(&dir).unwrap()).expect("package parses")
}

fn all(report: &imferno_core::ValidationReport) -> Vec<&ValidationIssue> {
    report
        .critical
        .iter()
        .chain(&report.errors)
        .chain(&report.warnings)
        .chain(&report.info)
        .collect()
}

/// 2016/PKL namespace (2016 and 2020) + CPLs declaring a Core Constraints
/// namespace → that edition.
#[test]
fn shared_pkl_namespace_is_resolved_by_the_cpl_core_namespace() {
    for (name, edition) in [
        ("ISXD/CompleteIMP", St2067_2Edition::V2020),
        ("IAB/CompleteIMP", St2067_2Edition::V2016),
    ] {
        let e = package(name).package_edition(None);
        assert_eq!(e.edition, edition, "{name}");
        assert_eq!(e.assumed, None, "{name}");
    }
}

/// The 2020 Core Constraints namespace of this package is only in a CPL the
/// Asset Map does not list (so it is not ingested): 2016/PKL alone leaves
/// 2016 and 2020, so 2020 is used and the guess is reported.
#[test]
fn missing_files_package_is_checked_against_2020_with_a_notice() {
    let pkg = package("MissingFilesAndAssetMapEntries");
    let e = pkg.package_edition(None);
    assert_eq!(e.edition, St2067_2Edition::V2020);
    assert!(e.assumed.is_some());
    let report = pkg.validate(&ValidationOptions::default());
    let missing: Vec<_> = all(&report)
        .into_iter()
        .filter(|i| i.rule.0 == "ST2067-2/FileNotFound")
        .collect();
    assert!(!missing.is_empty());
    for i in missing {
        assert_eq!(i.code, "ST2067-2:2020:7.2/FileNotFound");
    }
    assert_eq!(
        all(&report)
            .iter()
            .filter(|i| i.is(&imferno::EditionAssumed))
            .count(),
        1
    );
}

/// An ST 429-8 PKL with ST 2067-2:2013 CPLs is an ST 2067-2:2013 IMP: no
/// delivery rules, one DeliveryNotChecked notice.
#[test]
fn st429_8_pkl_is_an_st2067_2_2013_package() {
    let pkg = package("NYCbCrLT_3840x2160x23.98x10min");
    let e = pkg.package_edition(None);
    assert_eq!(e.edition, St2067_2Edition::V2013);
    assert_eq!(e.assumed, None);
    let report = pkg.validate(&ValidationOptions {
        skip_disk_checks: true,
        ..Default::default()
    });
    let issues = all(&report);
    assert_eq!(
        issues
            .iter()
            .filter(|i| i.is(&imferno::DeliveryNotChecked))
            .count(),
        1
    );
    for i in &issues {
        assert!(
            !(i.spec == imferno_core::Spec::St2067_2
                && (i.edition == "2016" || i.edition == "2020")),
            "package-level finding of another edition: {}",
            i.code
        );
        assert!(
            ![
                "ST429-9/PathCharacters",
                "ST429-9/AssetMapFileName",
                "ST429-9/VolindexMissing"
            ]
            .contains(&i.rule.0.as_str()),
            "delivery rule reported for a 2013 package: {}",
            i.code
        );
    }
}

#[test]
fn core_spec_pins_the_package_edition() {
    let pkg = package("NYCbCrLT_3840x2160x23.98x10min");
    let e = pkg.package_edition(Some(CoreSpecTarget::St2067_2_2016));
    assert_eq!(e.edition, St2067_2Edition::V2016);
    let report = pkg.validate(&ValidationOptions {
        core_spec: Some(CoreSpecTarget::St2067_2_2016),
        skip_disk_checks: true,
        ..Default::default()
    });
    assert!(!all(&report)
        .iter()
        .any(|i| i.is(&imferno::DeliveryNotChecked)));
}

/// 2016/PKL namespace and no Core Constraints namespace in the CPLs: the
/// newest candidate, with one EditionAssumed notice naming it.
#[test]
fn ambiguous_edition_uses_the_newest_and_says_so() {
    let mut pkg = package("MissingFilesAndAssetMapEntries");
    for cpl in pkg.composition_playlists.values_mut() {
        cpl.source_xml = None;
    }
    let e = pkg.package_edition(None);
    assert_eq!(e.edition, St2067_2Edition::V2020);
    let why = e.assumed.expect("assumed");
    assert!(why.contains("ST 2067-2:2020"), "{why}");
    let report = pkg.validate(&ValidationOptions::default());
    let notices: Vec<_> = all(&report)
        .into_iter()
        .filter(|i| i.is(&imferno::EditionAssumed))
        .collect();
    assert_eq!(notices.len(), 1);
}

/// The package edition and the CPL Core Constraints edition come from one
/// resolution: an App #2E 2020 ApplicationIdentification (which references
/// ST 2067-2:2020) in a CPL declaring the 2016 Core Constraints namespace
/// leaves both at the newest edition, each announced, instead of the CPL
/// at 2020 and the package at 2016.
#[test]
fn package_and_cpl_core_editions_agree() {
    let mut pkg = package("IAB/CompleteIMP");
    for cpl in pkg.composition_playlists.values_mut() {
        cpl.extension_properties
            .as_mut()
            .expect("ExtensionProperties")
            .application_identification = Some("http://www.smpte-ra.org/ns/2067-21/2020".into());
    }
    let e = pkg.package_edition(None);
    assert_eq!(e.edition, St2067_2Edition::V2020);
    assert!(e.assumed.is_some());
    for cpl in pkg.composition_playlists.values() {
        let c = imferno_core::validation::cpl_core_edition(cpl).expect("SMPTE CPL");
        assert_eq!(c.edition, e.edition);
        let why = c.assumed.expect("conflicting evidence is announced");
        assert!(why.contains("ST 2067-21:2020"), "{why}");
        assert!(
            why.contains("ST 2067-2:2016 Core Constraints namespace"),
            "{why}"
        );
    }
    let report = pkg.validate(&ValidationOptions {
        skip_disk_checks: true,
        ..Default::default()
    });
    let notices: Vec<_> = all(&report)
        .into_iter()
        .filter(|i| i.is(&imferno::EditionAssumed))
        .collect();
    // ST 2067-2: one for the package, one for its CPL; ST 2067-3: one for
    // the CPL (2016 and 2020 share its namespace); ST 2067-201: one for the
    // CPL (every edition shares its namespace).
    assert_eq!(notices.len(), 4, "{notices:#?}");
    assert_eq!(
        notices
            .iter()
            .filter(|n| n.message.starts_with("ST 2067-2 edition of"))
            .count(),
        2
    );
    assert!(notices.iter().any(|n| n.location.cpl_id.is_some()));
}

/// Only real Packing List namespaces (ST 429-8 → 2013; `.../2067-2/2016/PKL`
/// → 2016 or 2020, Annex K / Annex J) say anything about the edition. A
/// Core Constraints namespace on a PKL root is not a PKL namespace and
/// names no edition; the CPLs decide.
#[test]
fn core_constraints_namespace_on_a_pkl_names_no_edition() {
    use imferno_core::assetmap::PklNamespace;
    let mut pkg = package("IAB/CompleteIMP");
    for pkl in pkg.packing_lists.values_mut() {
        pkl.namespace = PklNamespace::Smpte2067_2_2013;
    }
    let e = pkg.package_edition(None);
    assert_eq!(e.edition, St2067_2Edition::V2016, "{:?}", e.assumed);
    assert_eq!(e.assumed, None);
    for pkl in pkg.packing_lists.values_mut() {
        pkl.namespace = PklNamespace::Smpte2067_2_2020;
    }
    let e = pkg.package_edition(None);
    assert_eq!((e.edition, e.assumed), (St2067_2Edition::V2016, None));
}

/// A 2013-namespace CPL listing App #2E 2016 (which cites ST 2067-2:2016):
/// the namespace decides, and the contradiction is EditionConflict (Warning),
/// naming both editions and the evidence; EditionAssumed is not used for it.
#[test]
fn namespace_vs_evidence_is_an_edition_conflict_warning() {
    let pkg = package("MERIDIAN_Netflix_Photon_161006");
    let report = pkg.validate(&ValidationOptions {
        skip_disk_checks: true,
        ..Default::default()
    });
    let conflicts: Vec<_> = all(&report)
        .into_iter()
        .filter(|i| i.is(&imferno::EditionConflict))
        .collect();
    assert_eq!(conflicts.len(), 1, "{conflicts:#?}");
    let c = conflicts[0];
    assert_eq!(c.severity, imferno_core::Severity::Warning);
    assert!(c.message.contains("ST 2067-2:2013"), "{}", c.message);
    assert!(c.message.contains("ST 2067-2:2016"), "{}", c.message);
    assert!(c.message.contains("ST 2067-21:2016"), "{}", c.message);
    assert!(!all(&report)
        .iter()
        .any(|i| i.is(&imferno::EditionAssumed) && i.message.contains("contradicts")));
}

/// Rules-config keys that select nothing come back with the result.
#[test]
fn unknown_rules_keys_are_reported_with_the_result() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-data/MERIDIAN_Netflix_Photon_161006");
    let mut rules = imferno_core::diagnostics::RulesConfig::default();
    rules.set_raw(
        "BogusKey".into(),
        imferno_core::diagnostics::RuleSeverity::Off,
    );
    let result = imferno_core::package::validate(
        read_dir(&dir).unwrap(),
        &ValidationOptions {
            rules,
            ..Default::default()
        },
    );
    assert_eq!(result.rule_warnings.len(), 1);
    assert_eq!(result.rule_warnings[0].key, "BogusKey");
}

/// One pin for everything: pinning the ST 2067-2 edition pins the editions
/// that follow from it through its dated references (ST 2067-3 here), over
/// the CPL's namespace, unless they are pinned separately. MERIDIAN has
/// 2013 namespaces and lists App #2E 2016.
#[test]
fn core_pin_carries_the_editions_that_follow_from_it() {
    use imferno_core::validation::CplSpecTarget;
    let pkg = package("MERIDIAN_Netflix_Photon_161006");
    let report = pkg.validate(&ValidationOptions {
        core_spec: Some(CoreSpecTarget::St2067_2_2016),
        skip_disk_checks: true,
        ..Default::default()
    });
    let issues = all(&report);
    assert!(
        !issues.iter().any(|i| i.is(&imferno::EditionConflict)),
        "{issues:#?}"
    );
    let st2067_3: Vec<_> = issues
        .iter()
        .filter(|i| i.spec == imferno_core::diagnostics::Spec::St2067_3)
        .collect();
    assert!(
        !st2067_3.is_empty(),
        "ContentKindUnknown is an ST 2067-3 rule"
    );
    for i in &st2067_3 {
        assert_eq!(i.edition, "2016", "{}", i.code);
    }
    for i in issues
        .iter()
        .filter(|i| i.spec == imferno_core::diagnostics::Spec::St2067_2)
    {
        assert_eq!(i.edition, "2016", "{}", i.code);
    }

    // A separate ST 2067-3 pin wins over the one that follows from the core.
    let report = pkg.validate(&ValidationOptions {
        core_spec: Some(CoreSpecTarget::St2067_2_2016),
        cpl_spec: Some(CplSpecTarget::St2067_3_2013),
        skip_disk_checks: true,
        ..Default::default()
    });
    let issues = all(&report);
    assert!(!issues.iter().any(|i| i.is(&imferno::EditionConflict)));
    let st2067_3: Vec<_> = issues
        .iter()
        .filter(|i| i.spec == imferno_core::diagnostics::Spec::St2067_3)
        .collect();
    assert!(!st2067_3.is_empty());
    for i in st2067_3 {
        assert_eq!(i.edition, "2013", "{}", i.code);
    }
}

/// Every edition the caller pinned is announced (EditionPinned, Info), one
/// per document and spec, naming the edition, the pin it comes from, and
/// what the document itself gives; also when the pin agrees with it.
#[test]
fn every_pinned_edition_is_announced() {
    use imferno_core::validation::CplSpecTarget;
    let pkg = package("MERIDIAN_Netflix_Photon_161006");
    let pinned = |options: ValidationOptions| -> Vec<ValidationIssue> {
        all(&pkg.validate(&options))
            .into_iter()
            .filter(|i| i.is(&imferno::EditionPinned))
            .cloned()
            .collect()
    };
    assert!(pinned(ValidationOptions {
        skip_disk_checks: true,
        ..Default::default()
    })
    .is_empty());

    let notices = pinned(ValidationOptions {
        core_spec: Some(CoreSpecTarget::St2067_2_2016),
        skip_disk_checks: true,
        ..Default::default()
    });
    assert_eq!(notices.len(), 3, "{notices:#?}");
    for n in &notices {
        assert_eq!(n.severity, imferno_core::Severity::Info);
    }
    let package_notice = notices
        .iter()
        .find(|n| n.location.cpl_id.is_none())
        .expect("package notice");
    let m = &package_notice.message;
    assert!(m.starts_with("ST 2067-2:2016 applied"), "{m}");
    assert!(
        m.contains("pinned by the caller (coreSpec / --core-spec)"),
        "{m}"
    );
    assert!(m.contains("ST 2067-2:2013"), "what the package gives: {m}");
    let cpl: Vec<&str> = notices
        .iter()
        .filter(|n| n.location.cpl_id.is_some())
        .map(|n| n.message.as_str())
        .collect();
    let core = cpl
        .iter()
        .find(|m| m.starts_with("ST 2067-2:2016 applied"))
        .expect("CPL ST 2067-2 notice");
    assert!(core.contains("pinned by the caller"), "{core}");
    assert!(core.contains("ST 2067-21:2016"), "the AppId: {core}");
    let st3 = cpl
        .iter()
        .find(|m| m.starts_with("ST 2067-3:2016 applied"))
        .expect("CPL ST 2067-3 notice");
    assert!(
        st3.contains("follows from the ST 2067-2:2016 pin (coreSpec / --core-spec)"),
        "{st3}"
    );
    assert!(
        st3.contains("http://www.smpte-ra.org/schemas/2067-3/2013"),
        "the namespace: {st3}"
    );
    assert!(st3.contains("ST 2067-3:2013"), "what the CPL gives: {st3}");

    // A separate ST 2067-3 pin is named as such.
    let notices = pinned(ValidationOptions {
        core_spec: Some(CoreSpecTarget::St2067_2_2016),
        cpl_spec: Some(CplSpecTarget::St2067_3_2013),
        skip_disk_checks: true,
        ..Default::default()
    });
    assert!(
        notices.iter().any(|n| n
            .message
            .starts_with("ST 2067-3:2013 applied: pinned by the caller (cplSpec / --cpl-spec)")),
        "{notices:#?}"
    );

    // A pin that agrees with the documents is announced too.
    let notices = pinned(ValidationOptions {
        core_spec: Some(CoreSpecTarget::St2067_2_2013),
        skip_disk_checks: true,
        ..Default::default()
    });
    assert_eq!(notices.len(), 3, "{notices:#?}");

    // MXF track files: their ST 377-1 / ST 377-4 editions follow the pin.
    let notices = pinned(ValidationOptions {
        core_spec: Some(CoreSpecTarget::St2067_2_2016),
        ..Default::default()
    });
    let mxf: Vec<_> = notices
        .iter()
        .filter(|n| n.location.file.is_some())
        .collect();
    assert!(!mxf.is_empty(), "{notices:#?}");
    for n in mxf {
        assert!(
            n.message.starts_with(
                "ST 377-1:2011 and ST 377-4:2012 applied: follow from the ST 2067-2:2016 pin"
            ),
            "{}",
            n.message
        );
    }
}
