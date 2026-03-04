//! IMF WASM — WebAssembly bindings for imferno.
//!
//! ## v2 API
//!
//! The WASM module exposes two main functions:
//!
//! - [`build_report`] — parse and validate an IMF package from in-memory files.
//! - [`format_report`] — render an [`ImfReport`] as a human-readable string.
//!
//! Individual parsers (`parseCplTyped`, `parseAssetmapTyped`, etc.) were removed in v2.

use imferno_core::package::{
    build_report, format_report, ImfReport, Imferno, RulesConfig, ValidationOptions,
};
use imferno_core::validation::{AppSpecTarget, CoreSpecTarget};
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
/// - `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"`
/// - `app2eSpec`: `"auto"` | `"none"` | `"v2020"` | `"v2021"` | `"v2023"`
/// - `rules`: ESLint-style rules configuration object
#[wasm_bindgen(js_name = "buildReport")]
pub fn build_report_js(
    #[wasm_bindgen(js_name = "files")] files_js: JsValue,
    #[wasm_bindgen(js_name = "options")] options_js: JsValue,
) -> Result<JsValue, JsValue> {
    let files: std::collections::HashMap<String, String> = serde_wasm_bindgen::from_value(files_js)
        .map_err(|e| JsValue::from_str(&format!("Invalid files argument: {}", e)))?;

    let (rules, core_spec, app_specs) = parse_options(&options_js)?;

    let options = ValidationOptions {
        rules,
        core_spec,
        app_specs,
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

/// Format a previously built `ImfReport` as a human-readable string.
///
/// Pass the object returned by `buildReport()` (or any valid `ImfReport` JSON).
/// Returns the same output as `imferno report` on the CLI.
#[wasm_bindgen(js_name = "formatReport")]
pub fn format_report_js(
    #[wasm_bindgen(js_name = "report")] report_js: JsValue,
) -> Result<String, JsValue> {
    let imf_report: ImfReport = serde_wasm_bindgen::from_value(report_js)
        .map_err(|e| JsValue::from_str(&format!("Invalid ImfReport: {}", e)))?;

    Ok(format_report(&imf_report, false))
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
    Option<Vec<AppSpecTarget>>,
);

fn parse_options(options_js: &JsValue) -> Result<ParsedOptions, JsValue> {
    if options_js.is_null() || options_js.is_undefined() {
        return Ok((Default::default(), None, None));
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
        None | Some("auto") => None,
        Some("v2013") => Some(CoreSpecTarget::St2067_2_2013),
        Some("v2016") => Some(CoreSpecTarget::St2067_2_2016),
        Some("v2020") => Some(CoreSpecTarget::St2067_2_2020),
        Some(other) => {
            return Err(JsValue::from_str(&format!(
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
            return Err(JsValue::from_str(&format!(
                "Unsupported app2eSpec '{}'. Use auto|none|v2020|v2021|v2023",
                other
            )));
        }
    };

    Ok((rules, core_spec, app_specs))
}
