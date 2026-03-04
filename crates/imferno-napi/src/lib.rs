//! Native Node.js bindings for imferno — SMPTE ST 2067 IMF validator.
//!
//! Provides `buildReport` / `buildReportFromPath` / `formatReport` plus `getVersion`.

use std::collections::HashMap;
use std::path::PathBuf;

use imferno_core::package::{
    build_report, format_report, ImfReport, Imferno, RulesConfig, ValidationOptions,
};
use imferno_core::validation::{AppSpecTarget, CoreSpecTarget};
use napi_derive::napi;

// =============================================================================
// Version
// =============================================================================

#[napi(js_name = "getVersion")]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// =============================================================================
// Build report (string-based) — same API as @imferno/wasm
// =============================================================================

#[napi(js_name = "buildReport")]
pub fn build_report_js(
    files: HashMap<String, String>,
    options: Option<serde_json::Value>,
) -> napi::Result<serde_json::Value> {
    let opts = parse_options(options.as_ref())?;
    let validation_options = ValidationOptions {
        rules: opts.rules,
        core_spec: opts.core_spec,
        app_specs: opts.app_specs,
        verify_hashes: None,
        skip_disk_checks: true,
    };

    let package = Imferno::parse(files)
        .map_err(|e| napi::Error::from_reason(format!("Failed to parse IMF package: {}", e)))?;

    let report = build_report(&package, &validation_options, None)
        .map_err(napi::Error::from_reason)?;

    serde_json::to_value(&report)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

// =============================================================================
// Build report (path-based) — NAPI-only
// =============================================================================

#[napi(js_name = "buildReportFromPath")]
pub fn build_report_from_path(
    path: String,
    options: Option<serde_json::Value>,
) -> napi::Result<serde_json::Value> {
    let opts = parse_options(options.as_ref())?;
    let pkg_path = PathBuf::from(&path);

    let files = imferno_core::package::read_dir(&pkg_path)
        .map_err(|e| napi::Error::from_reason(format!("Failed to read directory: {}", e)))?;

    let validation_options = ValidationOptions {
        rules: opts.rules,
        core_spec: opts.core_spec,
        app_specs: opts.app_specs,
        verify_hashes: None,
        skip_disk_checks: opts.skip_disk_checks,
    };

    let package = Imferno::parse(files)
        .map_err(|e| napi::Error::from_reason(format!("Failed to parse IMF package: {}", e)))?;

    let report = build_report(&package, &validation_options, None)
        .map_err(napi::Error::from_reason)?;

    serde_json::to_value(&report)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

// =============================================================================
// Format report — pretty-print an ImfReport as a human-readable string
// =============================================================================

#[napi(js_name = "formatReport")]
pub fn format_report_js(report: serde_json::Value) -> napi::Result<String> {
    let imf_report: ImfReport = serde_json::from_value(report)
        .map_err(|e| napi::Error::from_reason(format!("Invalid ImfReport JSON: {}", e)))?;

    Ok(format_report(&imf_report, false))
}

// =============================================================================
// Internal helpers
// =============================================================================

struct ParsedOptions {
    rules: RulesConfig,
    core_spec: Option<CoreSpecTarget>,
    app_specs: Option<Vec<AppSpecTarget>>,
    skip_disk_checks: bool,
}

fn parse_options(options: Option<&serde_json::Value>) -> napi::Result<ParsedOptions> {
    let Some(opts) = options else {
        return Ok(ParsedOptions {
            rules: Default::default(),
            core_spec: None,
            app_specs: None,
            skip_disk_checks: false,
        });
    };

    // Rules
    let rules: RulesConfig = if let Some(rules_val) = opts.get("rules") {
        serde_json::from_value(rules_val.clone())
            .map_err(|e| napi::Error::from_reason(format!("Invalid rules: {}", e)))?
    } else {
        Default::default()
    };

    // Core spec
    let core_spec = match opts.get("coreSpec").and_then(|v| v.as_str()) {
        None | Some("auto") => None,
        Some("v2013") => Some(CoreSpecTarget::St2067_2_2013),
        Some("v2016") => Some(CoreSpecTarget::St2067_2_2016),
        Some("v2020") => Some(CoreSpecTarget::St2067_2_2020),
        Some(other) => {
            return Err(napi::Error::from_reason(format!(
                "Unsupported coreSpec '{}'. Use auto|v2013|v2016|v2020",
                other
            )));
        }
    };

    // App2e spec
    let app_specs = match opts.get("app2eSpec").and_then(|v| v.as_str()) {
        None | Some("auto") => None,
        Some("none") => Some(vec![]),
        Some("v2020") => Some(vec![AppSpecTarget::St2067_21_2020]),
        Some("v2021") => Some(vec![AppSpecTarget::St2067_21_2021]),
        Some("v2023") => Some(vec![AppSpecTarget::St2067_21_2023]),
        Some(other) => {
            return Err(napi::Error::from_reason(format!(
                "Unsupported app2eSpec '{}'. Use auto|none|v2020|v2021|v2023",
                other
            )));
        }
    };

    let skip_disk_checks = opts
        .get("skipDiskChecks")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    Ok(ParsedOptions {
        rules,
        core_spec,
        app_specs,
        skip_disk_checks,
    })
}
