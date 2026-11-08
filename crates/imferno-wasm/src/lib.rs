//! IMF WASM — WebAssembly bindings for imferno.
//!
//! ## v2 API
//!
//! The WASM module exposes two main functions:
//!
//! - [`build_report`] — parse and validate an IMF package from in-memory files.
//! - [`format_report_js`] — render a `ValidationResult` (or legacy `ImfReport`) as text.

#[allow(deprecated)]
use imferno_core::package::{
    build_report, format_json_report, validate as validate_package, Imferno, RulesConfig,
    ValidationOptions,
};
use imferno_core::validation::{
    parse_app_spec_targets, parse_core_spec_target, parse_cpl_spec_target, parse_iab_spec_target,
    AppSpecTarget, CoreSpecTarget, CplSpecTarget, IabSpecTarget,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// Initialize the WASM module
#[wasm_bindgen(js_name = "init")]
pub fn init() {
    console_log!("IMF WASM parser initialized");
}

/// Get library version
#[wasm_bindgen(js_name = "getVersion")]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// =============================================================================
// BUILD REPORT — structured ImfReport (package + validation + CPL analysis)
// =============================================================================

/// Build a structured report from an IMF package.
///
/// Pass all XML files as a plain JS object where each key is the filename
/// and each value is the file's text content.
///
/// Returns an `ImfReport` containing package metadata, CPL analysis, and
/// validation results. This is the same JSON that the CLI `export` command produces.
///
/// Options (all optional):
/// - `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"` (ST 2067-2; a
///   pin also pins the editions it cites: ST 2067-3, ST 2067-201, ST 377-1/-4)
/// - `cplSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"` (ST 2067-3; wins
///   over the edition `coreSpec` implies)
/// - `iabSpec`: `"auto"` | `"v2019"` | `"v2021"` | `"v2026"` (ST 2067-201; wins
///   over the edition `coreSpec` implies)
/// - `app2eSpec`: `"auto"` | `"none"` | `"v2014"` | `"v2016"` | `"v2020"` | `"v2023"` | `"v2026"`
/// - `rules`: ESLint-style rules configuration object
#[allow(deprecated)]
#[wasm_bindgen(js_name = "buildReport")]
pub fn build_report_js(
    #[wasm_bindgen(js_name = "files")] files_js: JsValue,
    #[wasm_bindgen(js_name = "options")] options_js: JsValue,
) -> Result<JsValue, JsValue> {
    let files: std::collections::HashMap<String, String> = serde_wasm_bindgen::from_value(files_js)
        .map_err(|e| JsValue::from_str(&format!("Invalid files argument: {}", e)))?;

    let (rules, core_spec, cpl_spec, iab_spec, app_specs, aggregate_repeats) =
        parse_options(&options_js)?;

    let options = ValidationOptions {
        rules,
        core_spec,
        cpl_spec,
        iab_spec,
        app_specs,
        aggregate_repeats,
        ..Default::default()
    };

    let package = Imferno::parse(files)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse IMF package: {}", e)))?;

    let report = build_report(&package, &options, None).map_err(|e| JsValue::from_str(&e))?;

    to_js_value(&report)
}

// =============================================================================
// FORMAT REPORT — pretty-print an ImfReport as a human-readable string
// =============================================================================

/// Format a validation result as a human-readable string.
///
/// Pass the object returned by `validate()`, or (legacy) by `buildReport()`.
#[wasm_bindgen(js_name = "formatReport")]
pub fn format_report_js(
    #[wasm_bindgen(js_name = "report")] report_js: JsValue,
) -> Result<String, JsValue> {
    let value: serde_json::Value = serde_wasm_bindgen::from_value(report_js)
        .map_err(|e| JsValue::from_str(&format!("Invalid report: {}", e)))?;
    format_json_report(value).map_err(|e| JsValue::from_str(&e))
}

// =============================================================================
// SARIF + rule catalogue
// =============================================================================

/// Convert a validation result (or its `validation` report) to a SARIF 2.1.0 log.
#[wasm_bindgen(js_name = "toSarif")]
pub fn to_sarif_js(
    #[wasm_bindgen(js_name = "report")] report_js: JsValue,
) -> Result<JsValue, JsValue> {
    let value: serde_json::Value = serde_wasm_bindgen::from_value(report_js)
        .map_err(|e| JsValue::from_str(&format!("Invalid report: {}", e)))?;
    let log =
        imferno_core::diagnostics::sarif_from_json(value).map_err(|e| JsValue::from_str(&e))?;
    to_js_value(&log)
}

/// The rule catalogue: one entry per rule with its per-edition table.
#[wasm_bindgen(js_name = "listRules")]
pub fn list_rules_js() -> Result<JsValue, JsValue> {
    to_js_value(&imferno_core::catalogue::rule_infos())
}

// =============================================================================
// PARSE PACKAGE — full serialized Imferno struct
// =============================================================================

/// Parse an IMF package from in-memory files, returning the full parsed package.
///
/// Unlike `buildReport` which returns a summary, this returns the complete
/// `Imferno` struct with all essence descriptors, locales, content versions, etc.
#[wasm_bindgen(js_name = "parsePackage")]
pub fn parse_package_js(
    #[wasm_bindgen(js_name = "files")] files_js: JsValue,
) -> Result<JsValue, JsValue> {
    let files: std::collections::HashMap<String, String> = serde_wasm_bindgen::from_value(files_js)
        .map_err(|e| JsValue::from_str(&format!("Invalid files argument: {}", e)))?;

    let package = Imferno::parse(files)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse IMF package: {}", e)))?;

    to_js_value(&package)
}

// =============================================================================
// VALIDATE — parse + validate, returns { package, validation }
// =============================================================================

/// Parse and validate an IMF package in one call.
///
/// Returns `{ package, validation }` where `package` is the full `Imferno`
/// struct and `validation` is the `ValidationReport` with all findings.
///
/// This is the recommended entry point.
#[wasm_bindgen(js_name = "validate")]
pub fn validate_js(
    #[wasm_bindgen(js_name = "files")] files_js: JsValue,
    #[wasm_bindgen(js_name = "options")] options_js: JsValue,
) -> Result<JsValue, JsValue> {
    let files: std::collections::HashMap<String, String> = serde_wasm_bindgen::from_value(files_js)
        .map_err(|e| JsValue::from_str(&format!("Invalid files argument: {}", e)))?;

    let (rules, core_spec, cpl_spec, iab_spec, app_specs, aggregate_repeats) =
        parse_options(&options_js)?;

    let options = ValidationOptions {
        rules,
        core_spec,
        cpl_spec,
        iab_spec,
        app_specs,
        aggregate_repeats,
        ..Default::default()
    };

    let result = validate_package(files, &options);
    to_js_value(&result)
}

// =============================================================================
// Internal helpers
// =============================================================================

/// Serialize a value to a plain JS object (not a Map).
fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    value
        .serialize(&serializer)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

type ParsedOptions = (
    RulesConfig,
    Option<CoreSpecTarget>,
    Option<CplSpecTarget>,
    Option<IabSpecTarget>,
    Option<Vec<AppSpecTarget>>,
    bool, // aggregate_repeats
);

fn parse_options(options_js: &JsValue) -> Result<ParsedOptions, JsValue> {
    if options_js.is_null() || options_js.is_undefined() {
        return Ok((Default::default(), None, None, None, None, false));
    }

    let opts: serde_json::Value = serde_wasm_bindgen::from_value(options_js.clone())
        .map_err(|e| JsValue::from_str(&format!("Invalid options argument: {}", e)))?;

    // Rules
    let rules: RulesConfig = if let Some(rules_val) = opts.get("rules") {
        serde_json::from_value(rules_val.clone())
            .map_err(|e| JsValue::from_str(&format!("Invalid rules: {}", e)))?
    } else {
        Default::default()
    };

    // Core spec
    let core_spec = match opts.get("coreSpec").and_then(|v| v.as_str()) {
        None => None,
        Some(s) => parse_core_spec_target(s).map_err(|e| JsValue::from_str(&e))?,
    };

    // CPL (ST 2067-3) spec
    let cpl_spec = match opts.get("cplSpec").and_then(|v| v.as_str()) {
        None => None,
        Some(s) => parse_cpl_spec_target(s).map_err(|e| JsValue::from_str(&e))?,
    };

    // IAB (ST 2067-201) spec
    let iab_spec = match opts.get("iabSpec").and_then(|v| v.as_str()) {
        None => None,
        Some(s) => parse_iab_spec_target(s).map_err(|e| JsValue::from_str(&e))?,
    };

    // App2e spec
    let app_specs = match opts.get("app2eSpec").and_then(|v| v.as_str()) {
        None => None,
        Some(s) => parse_app_spec_targets(s).map_err(|e| JsValue::from_str(&e))?,
    };

    let aggregate_repeats = opts
        .get("aggregateRepeats")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    Ok((
        rules,
        core_spec,
        cpl_spec,
        iab_spec,
        app_specs,
        aggregate_repeats,
    ))
}
