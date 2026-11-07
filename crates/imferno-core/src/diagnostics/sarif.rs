//! SARIF 2.1.0 output for validation reports.
//!
//! - `tool.driver.rules[]`: every rule of the catalogue; `id` is the rule id
//!   (`ST2067-3/SegmentDuration`), with `shortDescription` and `properties`
//!   (spec, category, per-edition table).
//! - `runs[0].taxonomies[]`: one taxonomy per spec edition
//!   (`ST2067-3:2016`), whose `taxa` are the clauses of that edition the
//!   catalogue cites.
//! - `results[]`: one per finding; `ruleId`/`ruleIndex`, `level`
//!   (Critical/Error → error, Warning → warning, Info → note), `taxa`
//!   pointing at the applied edition's clause, `message`, `locations`
//!   (artifact URI relative to the package root, `uriBaseId` `PKGROOT`;
//!   regions follow with D1). Findings suppressed by a rules config are
//!   included with an external suppression, at their original level
//!   (property `originalSeverity`).

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde_json::json;
use serde_sarif::sarif::{
    ArtifactLocation, Location as SarifLocation, LogicalLocation, Message,
    MultiformatMessageString, PhysicalLocation, PropertyBag, ReportingDescriptor,
    ReportingDescriptorReference, Result as SarifResult, ResultLevel, Run, Sarif, Suppression,
    Tool, ToolComponent, ToolComponentReference,
};

use super::{Location, Severity, ValidationIssue, ValidationReport};
use crate::catalogue;

/// The `uriBaseId` artifact URIs are relative to: the package root.
pub const PACKAGE_ROOT_BASE_ID: &str = "PKGROOT";

/// SARIF 2.1.0 schema URI written into `$schema`.
pub const SARIF_SCHEMA: &str = "https://json.schemastore.org/sarif-2.1.0.json";

/// The taxonomy name of a spec edition (`ST2067-3:2016`, `IMFERNO`).
fn taxonomy_name(spec: &str, edition: &str) -> String {
    if edition.is_empty() {
        spec.to_string()
    } else {
        format!("{spec}:{edition}")
    }
}

/// The id of a clause within its taxonomy (`-` for pseudo-specs without
/// clauses, as SARIF ids must not be empty).
fn taxon_id(clause: &str) -> String {
    if clause.is_empty() {
        "-".to_string()
    } else {
        clause.to_string()
    }
}

fn level(severity: Severity) -> ResultLevel {
    match severity {
        Severity::Critical | Severity::Error => ResultLevel::Error,
        Severity::Warning => ResultLevel::Warning,
        Severity::Info => ResultLevel::Note,
    }
}

/// A file path as a URI reference: `/` separators, percent-encoding for
/// characters a URI reference cannot carry.
fn path_uri(path: &std::path::Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        let c = b as char;
        if c.is_ascii_alphanumeric() || "-._~/:@!$&'()*+,;=".contains(c) {
            out.push(c);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// An artifact location for `path`: relative to the package root (with
/// `uriBaseId` [`PACKAGE_ROOT_BASE_ID`]) when it lies under `root` or is
/// relative already, else an absolute `file://` URI.
fn artifact(path: &std::path::Path, root: Option<&std::path::Path>) -> ArtifactLocation {
    let relative = match root {
        Some(r) if !r.as_os_str().is_empty() => path.strip_prefix(r).ok(),
        _ => None,
    }
    .or_else(|| path.is_relative().then_some(path));
    match relative {
        Some(rel) => ArtifactLocation::builder()
            .uri(path_uri(rel))
            .uri_base_id(PACKAGE_ROOT_BASE_ID)
            .build(),
        None => {
            let abs = path_uri(path);
            let uri = if abs.starts_with('/') {
                format!("file://{abs}")
            } else {
                format!("file:///{abs}")
            };
            ArtifactLocation::builder().uri(uri).build()
        }
    }
}

/// The `file://` URI of the package root, ending in `/`.
fn root_uri(root: &std::path::Path) -> String {
    let mut abs = path_uri(root);
    if !abs.ends_with('/') {
        abs.push('/');
    }
    if abs.starts_with('/') {
        format!("file://{abs}")
    } else {
        format!("file:///{abs}")
    }
}

fn sarif_location(loc: &Location, root: Option<&std::path::Path>) -> Option<SarifLocation> {
    let uri = loc.file.as_deref().map(|f| artifact(f, root)).or_else(|| {
        loc.cpl_filename
            .as_deref()
            .map(|f| artifact(std::path::Path::new(f), root))
    });
    let mut logical = Vec::new();
    if let Some(cpl) = loc.cpl_id {
        logical.push(
            LogicalLocation::builder()
                .kind("cpl")
                .fully_qualified_name(cpl.to_string())
                .build(),
        );
    }
    if let Some(path) = &loc.path {
        logical.push(
            LogicalLocation::builder()
                .kind("element")
                .fully_qualified_name(path.clone())
                .build(),
        );
    }
    if uri.is_none() && logical.is_empty() {
        return None;
    }
    let mut props = BTreeMap::new();
    if let Some(s) = loc.segment {
        props.insert("segment".to_string(), json!(s));
    }
    if let Some(s) = &loc.segment_id {
        props.insert("segmentId".to_string(), json!(s));
    }
    if let Some(s) = &loc.sequence_id {
        props.insert("sequenceId".to_string(), json!(s));
    }
    if let Some(s) = &loc.resource_id {
        props.insert("resourceId".to_string(), json!(s));
    }
    let mut out = SarifLocation::default();
    if let Some(artifact) = uri {
        out.physical_location = Some(
            PhysicalLocation::builder()
                .artifact_location(artifact)
                .build(),
        );
    }
    if !logical.is_empty() {
        out.logical_locations = Some(logical);
    }
    if !props.is_empty() {
        out.properties = Some(PropertyBag::builder().additional_properties(props).build());
    }
    Some(out)
}

/// Build the SARIF log for `report`, with artifact URIs relative to the
/// package root when it is not known (see [`to_sarif_with_root`]).
pub fn to_sarif(report: &ValidationReport) -> Sarif {
    to_sarif_with_root(report, None)
}

/// Build the SARIF log for `report`; `root` is the package root directory,
/// recorded as `originalUriBaseIds.PKGROOT` and used to make artifact URIs
/// relative.
pub fn to_sarif_with_root(report: &ValidationReport, root: Option<&std::path::Path>) -> Sarif {
    // Driver rules: the whole catalogue, in catalogue order.
    let mut rule_index: HashMap<String, i64> = HashMap::new();
    let mut rules = Vec::new();
    // spec edition → clauses cited by the catalogue
    let mut clauses: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (i, r) in catalogue::all().enumerate() {
        let info = r.info();
        rule_index.insert(info.id.0.clone(), i as i64);
        for e in &info.editions {
            if let Some(c) = &e.clause {
                clauses
                    .entry(taxonomy_name(info.spec.id(), &e.edition))
                    .or_default()
                    .insert(taxon_id(c));
            }
        }
        let mut props = BTreeMap::new();
        props.insert("spec".to_string(), json!(info.spec));
        props.insert("category".to_string(), json!(info.category));
        props.insert("editions".to_string(), json!(info.editions));
        let mut d = ReportingDescriptor::builder()
            .id(info.id.0.clone())
            .name(info.name.clone())
            .short_description(
                MultiformatMessageString::builder()
                    .text(info.description)
                    .build(),
            )
            .properties(PropertyBag::builder().additional_properties(props).build())
            .build();
        if let Some(ex) = info.example {
            d.full_description = Some(
                MultiformatMessageString::builder()
                    .text(format!("Example: {ex}"))
                    .build(),
            );
        }
        rules.push(d);
    }

    // Taxonomies, one per spec edition; findings reference them by index.
    let mut taxonomy_index: HashMap<String, (i64, HashMap<String, i64>)> = HashMap::new();
    let mut taxonomies = Vec::new();
    for (i, (name, cls)) in clauses.into_iter().enumerate() {
        let mut idx = HashMap::new();
        let taxa: Vec<ReportingDescriptor> = cls
            .into_iter()
            .enumerate()
            .map(|(j, c)| {
                idx.insert(c.clone(), j as i64);
                ReportingDescriptor::builder().id(c).build()
            })
            .collect();
        taxonomy_index.insert(name.clone(), (i as i64, idx));
        taxonomies.push(ToolComponent::builder().name(name).taxa(taxa).build());
    }

    let result = |issue: &ValidationIssue, suppressed: bool| -> SarifResult {
        let taxonomy = taxonomy_name(issue.spec.id(), &issue.edition);
        let taxon = taxon_id(&issue.clause);
        let (tc_index, taxon_index) = taxonomy_index
            .get(&taxonomy)
            .map(|(i, idx)| (Some(*i), idx.get(&taxon).copied()))
            .unwrap_or((None, None));
        let mut tc = ToolComponentReference::builder().name(taxonomy).build();
        tc.index = tc_index;
        let mut taxon_ref = ReportingDescriptorReference::builder()
            .id(taxon)
            .tool_component(tc)
            .build();
        taxon_ref.index = taxon_index;

        let mut props = BTreeMap::new();
        props.insert("code".to_string(), json!(issue.code));
        props.insert("spec".to_string(), json!(issue.spec));
        props.insert("edition".to_string(), json!(issue.edition));
        props.insert("clause".to_string(), json!(issue.clause));
        props.insert("strength".to_string(), json!(issue.strength));
        props.insert("severity".to_string(), json!(issue.severity));
        props.insert("category".to_string(), json!(issue.category));
        if let Some(s) = &issue.suggestion {
            props.insert("suggestion".to_string(), json!(s));
        }
        if !issue.context.is_empty() {
            let ctx: BTreeMap<_, _> = issue.context.iter().collect();
            props.insert("context".to_string(), json!(ctx));
        }

        let locations: Vec<SarifLocation> = std::iter::once(&issue.location)
            .chain(issue.additional_instances.iter())
            .filter_map(|l| sarif_location(l, root))
            .collect();

        // A suppressed finding is reported at the severity it had before
        // the rules config switched it off.
        let original = issue
            .context
            .get("original_severity")
            .and_then(|s| serde_json::from_value::<Severity>(json!(s)).ok());
        let shown = if suppressed {
            original.unwrap_or(issue.severity)
        } else {
            issue.severity
        };
        if suppressed {
            props.insert("originalSeverity".to_string(), json!(shown));
        }
        let mut r = SarifResult::builder()
            .message(Message::builder().text(issue.message.clone()).build())
            .rule_id(issue.rule.0.clone())
            .level(level(shown))
            .taxa(vec![taxon_ref])
            .properties(PropertyBag::builder().additional_properties(props).build())
            .build();
        r.rule_index = rule_index.get(&issue.rule.0).copied();
        if !locations.is_empty() {
            r.locations = Some(locations);
        }
        if issue.instance_count() > 1 {
            r.occurrence_count = Some(issue.instance_count() as i64);
        }
        if suppressed {
            let mut s = Suppression::builder().kind(json!("external")).build();
            s.justification = issue
                .context
                .get("suppressed_by")
                .map(|k| format!("rules config key `{k}`"));
            r.suppressions = Some(vec![s]);
        }
        r
    };

    let results: Vec<SarifResult> = report
        .critical
        .iter()
        .chain(&report.errors)
        .chain(&report.warnings)
        .chain(&report.info)
        .map(|i| result(i, false))
        .chain(report.suppressed.iter().map(|i| result(i, true)))
        .collect();

    let driver = ToolComponent::builder()
        .name("imferno")
        .version(env!("CARGO_PKG_VERSION"))
        .semantic_version(env!("CARGO_PKG_VERSION"))
        .information_uri("https://github.com/jpwesselink/imferno")
        .rules(rules)
        .supported_taxonomies(
            taxonomies
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let mut r = ToolComponentReference::builder()
                        .name(t.name.clone())
                        .build();
                    r.index = Some(i as i64);
                    r
                })
                .collect::<Vec<_>>(),
        )
        .build();

    let mut run_props = BTreeMap::new();
    run_props.insert("isCompliant".to_string(), json!(report.is_compliant));
    run_props.insert("isPlayable".to_string(), json!(report.is_playable));
    run_props.insert("profile".to_string(), json!(report.profile));
    run_props.insert("timestamp".to_string(), json!(report.timestamp));

    let mut base = ArtifactLocation::builder()
        .description(
            Message::builder()
                .text("The IMF package root directory")
                .build(),
        )
        .build();
    if let Some(r) = root.filter(|r| !r.as_os_str().is_empty()) {
        base.uri = Some(root_uri(r));
    }
    let mut bases = BTreeMap::new();
    bases.insert(PACKAGE_ROOT_BASE_ID.to_string(), base);

    let run = Run::builder()
        .tool(Tool::builder().driver(driver).build())
        .original_uri_base_ids(bases)
        .taxonomies(taxonomies)
        .results(results)
        .properties(
            PropertyBag::builder()
                .additional_properties(run_props)
                .build(),
        )
        .build();

    Sarif::builder()
        .schema(SARIF_SCHEMA)
        .version(json!("2.1.0"))
        .runs(vec![run])
        .build()
}

/// SARIF log for a serialized `ValidationReport`, or for a serialized
/// validation result carrying one under `validation` (what the NAPI/WASM
/// `validate` functions return).
pub fn sarif_from_json(value: serde_json::Value) -> Result<serde_json::Value, String> {
    let root = value
        .get("package")
        .and_then(|p| p.get("rootPath").or_else(|| p.get("root_path")))
        .and_then(|r| r.as_str())
        .map(std::path::PathBuf::from);
    let report = match value.get("validation") {
        Some(v) if v.is_object() => v.clone(),
        _ => value,
    };
    let report: ValidationReport =
        serde_json::from_value(report).map_err(|e| format!("Invalid validation report: {e}"))?;
    serde_json::to_value(to_sarif_with_root(&report, root.as_deref()))
        .map_err(|e| format!("SARIF serialisation failed: {e}"))
}

/// [`to_sarif`] as a JSON value.
pub fn to_sarif_json(report: &ValidationReport) -> serde_json::Value {
    serde_json::to_value(to_sarif(report)).expect("SARIF log serialises")
}
