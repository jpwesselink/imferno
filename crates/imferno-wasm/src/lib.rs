//! IMF WASM Parser - Clean TypeScript API
//!
//! This module provides a clean, type-safe JavaScript API for parsing IMF (Interoperable Master Format) files.
//! All functions return properly typed objects instead of JSON strings, providing full IntelliSense support.

use imferno_core::assetmap::VolumeIndex;
use imferno_core::cpl::CompositionPlaylist;
use imferno_core::package::{Imferno, RulesConfig, ValidationOptions};
use imferno_core::validation::{AppSpecTarget, CoreSpecTarget};
use imferno_core::{Category, Severity, ValidationIssue, ValidationProfile, ValidationReport};
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
// CORE PARSING FUNCTIONS
// =============================================================================

/// Parse VOLINDEX.xml and return a typed VolumeIndex object
#[wasm_bindgen(js_name = "parseVolindexTyped")]
pub fn parse_volindex_typed(
    #[wasm_bindgen(js_name = "xmlContent")] xml_content: &str,
) -> Result<VolumeIndex, JsValue> {
    match imferno_core::assetmap::parse_volindex(xml_content) {
        Ok(volindex) => Ok(volindex),
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {}", e))),
    }
}

/// Parse ASSETMAP.xml and return a typed AssetMap object
#[wasm_bindgen(js_name = "parseAssetmapTyped")]
pub fn parse_assetmap_typed(
    #[wasm_bindgen(js_name = "xmlContent")] xml_content: &str,
) -> Result<JsValue, JsValue> {
    let assetmap = imferno_core::assetmap::parse_assetmap(xml_content)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    serde_wasm_bindgen::to_value(&assetmap)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Parse PKL XML and return a typed PackingList object
#[wasm_bindgen(js_name = "parsePklTyped")]
pub fn parse_pkl_typed(
    #[wasm_bindgen(js_name = "xmlContent")] xml_content: &str,
) -> Result<JsValue, JsValue> {
    let pkl = imferno_core::assetmap::parse_pkl(xml_content)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    serde_wasm_bindgen::to_value(&pkl)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Parse CPL XML and return a typed CompositionPlaylist object
#[wasm_bindgen(js_name = "parseCplTyped")]
pub fn parse_cpl_typed(
    #[wasm_bindgen(js_name = "xmlContent")] xml_content: &str,
) -> Result<CompositionPlaylist, JsValue> {
    match imferno_core::cpl::parse_cpl(xml_content) {
        Ok(cpl) => Ok(cpl),
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {}", e))),
    }
}

// =============================================================================
// VALIDATE — the unified function
// =============================================================================

/// Validate a full IMF package and return both the validation report and parsed data.
///
/// Pass all XML files from the package as a plain JS object where each key is
/// the filename and each value is the file's text content. ASSETMAP.xml is
/// required; VOLINDEX.xml, PKL files, and CPL files are resolved automatically
/// from the AssetMap.
///
/// Options (all optional):
/// - `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"` — core constraints version
/// - `app2eSpec`: `"auto"` | `"none"` | `"v2020"` | `"v2021"` | `"v2023"` — app profile version
/// - `rules`: ESLint-style rules configuration object
///
/// Returns `{ report, cpls, assetMap, packingLists, volumeIndex, unreferencedAssets, declaredSidecars }`
#[wasm_bindgen(js_name = "validate")]
pub fn validate(
    #[wasm_bindgen(js_name = "files")] files_js: JsValue,
    #[wasm_bindgen(js_name = "options")] options_js: JsValue,
) -> Result<JsValue, JsValue> {
    let files: std::collections::HashMap<String, String> = serde_wasm_bindgen::from_value(files_js)
        .map_err(|e| JsValue::from_str(&format!("Invalid files argument: {}", e)))?;

    // Parse options
    let (rules, core_spec, app_specs) = parse_validate_options(&options_js)?;

    let options = ValidationOptions {
        rules,
        core_spec,
        app_specs,
        ..Default::default()
    };

    // Try to parse the package
    let package = match Imferno::parse(files) {
        Ok(p) => p,
        Err(e) => {
            // Parse failed — return error report with empty package data
            let mut report = ValidationReport::new(ValidationProfile::SMPTE);
            report.add(ValidationIssue::new(
                Severity::Critical,
                Category::Structure,
                "IMF/ParseError",
                format!("Failed to parse IMF package: {}", e),
            ));
            let result = serde_json::json!({
                "report": report,
                "cpls": [],
                "assetMap": null,
                "packingLists": [],
                "volumeIndex": null,
                "unreferencedAssets": [],
                "declaredSidecars": [],
            });
            return to_js_value(&result);
        }
    };

    // Validate
    let report = package.validate(&options);

    // Collect parsed data
    let cpls: Vec<&imferno_core::cpl::CompositionPlaylist> =
        package.composition_playlists.values().collect();

    let packing_lists: Vec<&imferno_core::assetmap::PackingList> =
        package.packing_lists.values().collect();

    let declared_sidecars: Vec<serde_json::Value> = package
        .sidecar_composition_maps
        .values()
        .flat_map(|scm| {
            scm.sidecar_assets.iter().map(|sa| {
                serde_json::json!({
                    "id": sa.id.to_string(),
                    "cplIds": sa.cpl_ids.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
                })
            })
        })
        .collect();

    let unreferenced: Vec<serde_json::Value> = package
        .unreferenced_assets()
        .iter()
        .map(|a| {
            let path = a
                .chunk_list
                .chunks
                .first()
                .map(|c| c.path.as_str())
                .unwrap_or("");
            serde_json::json!({
                "id": a.id.to_string(),
                "path": path,
            })
        })
        .collect();

    let result = serde_json::json!({
        "report": report,
        "cpls": cpls,
        "assetMap": package.asset_map,
        "packingLists": packing_lists,
        "volumeIndex": package.volume_index,
        "unreferencedAssets": unreferenced,
        "declaredSidecars": declared_sidecars,
    });

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
    Option<Vec<AppSpecTarget>>,
);

/// Parse the optional validate options JS object into Rust types.
fn parse_validate_options(options_js: &JsValue) -> Result<ParsedOptions, JsValue> {
    if options_js.is_null() || options_js.is_undefined() {
        return Ok((Default::default(), None, None));
    }

    // Deserialize as a generic JSON value to extract fields
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
