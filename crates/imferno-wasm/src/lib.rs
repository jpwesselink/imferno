//! IMF WASM Parser - Clean TypeScript API
//!
//! This module provides a clean, type-safe JavaScript API for parsing IMF (Interoperable Master Format) files.
//! All functions return properly typed objects instead of JSON strings, providing full IntelliSense support.

use wasm_bindgen::prelude::*;
use imferno_core::cpl::CompositionPlaylist;
use imferno_core::assetmap::VolumeIndex;
use imferno_core::validation::{
    validate_cpl_with_registry, AppSpecTarget, ConfigurableValidatorRegistry, CoreSpecTarget,
    ValidatorSelection,
};
use imferno_core::{
    Category, Severity, ValidationIssue, ValidationProfile, ValidationReport,
};
use imferno_core::package::{Imferno, RulesConfig, ValidationOptions};

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
pub fn parse_volindex_typed(#[wasm_bindgen(js_name = "xmlContent")] xml_content: &str) -> Result<VolumeIndex, JsValue> {
    match imferno_core::assetmap::parse_volindex(xml_content) {
        Ok(volindex) => Ok(volindex),
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {}", e))),
    }
}

/// Parse ASSETMAP.xml and return a typed AssetMap object
#[wasm_bindgen(js_name = "parseAssetmapTyped")]
pub fn parse_assetmap_typed(#[wasm_bindgen(js_name = "xmlContent")] xml_content: &str) -> Result<JsValue, JsValue> {
    let assetmap = imferno_core::assetmap::parse_assetmap(xml_content)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    serde_wasm_bindgen::to_value(&assetmap)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Parse PKL XML and return a typed PackingList object
#[wasm_bindgen(js_name = "parsePklTyped")]
pub fn parse_pkl_typed(#[wasm_bindgen(js_name = "xmlContent")] xml_content: &str) -> Result<JsValue, JsValue> {
    let pkl = imferno_core::assetmap::parse_pkl(xml_content)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    serde_wasm_bindgen::to_value(&pkl)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Parse CPL XML and return a typed CompositionPlaylist object
#[wasm_bindgen(js_name = "parseCplTyped")]
pub fn parse_cpl_typed(#[wasm_bindgen(js_name = "xmlContent")] xml_content: &str) -> Result<CompositionPlaylist, JsValue> {
    match imferno_core::cpl::parse_cpl(xml_content) {
        Ok(cpl) => Ok(cpl),
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {}", e))),
    }
}

// =============================================================================
// SOURCE ASSET EXTRACTION
// =============================================================================

/// Extract a SourceAsset from CPL XML
#[wasm_bindgen(js_name = "extractSourceAsset")]
pub fn extract_source_asset(#[wasm_bindgen(js_name = "cplXml")] cpl_xml: &str) -> Result<JsValue, JsValue> {
    let cpl = imferno_core::cpl::parse_cpl(cpl_xml)
        .map_err(|e| JsValue::from_str(&format!("CPL parse error: {}", e)))?;

    let source_asset = imferno_core::package::extract_source_asset(&cpl)
        .map_err(|e| JsValue::from_str(&format!("Extraction error: {}", e)))?;

    serde_wasm_bindgen::to_value(&source_asset)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Compare a SourceAsset against a delivery spec
#[wasm_bindgen(js_name = "compareDelivery")]
pub fn compare_delivery(
    #[wasm_bindgen(js_name = "sourceAssetJson")] source_asset_json: JsValue,
    #[wasm_bindgen(js_name = "deliverySpecJson")] delivery_spec_json: JsValue,
) -> Result<JsValue, JsValue> {
    let source: imferno_core::package::SourceAsset = serde_wasm_bindgen::from_value(source_asset_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid source asset: {}", e)))?;

    let spec: imferno_core::package::DeliveryRequest = serde_wasm_bindgen::from_value(delivery_spec_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid delivery spec: {}", e)))?;

    let comparison = imferno_core::package::compare_delivery(&source, &spec);

    serde_wasm_bindgen::to_value(&comparison)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

// =============================================================================
// VALIDATION — returns ValidationReport (rich Rust struct, serialized to JS)
// =============================================================================

/// Validate a CPL with configurable built-in spec selection (ST 2067-2/App2E).
///
/// `coreSpec`: "auto" | "v2013" | "v2016" | "v2020"
/// `app2eSpec`: "auto" | "none" | "v2020" | "v2021" | "v2023"
#[wasm_bindgen(js_name = "validateCplWithSpecSelection")]
pub fn validate_cpl_with_spec_selection(
    #[wasm_bindgen(js_name = "cplXml")] cpl_xml: &str,
    #[wasm_bindgen(js_name = "coreSpec")] core_spec: Option<String>,
    #[wasm_bindgen(js_name = "app2eSpec")] app2e_spec: Option<String>,
) -> Result<JsValue, JsValue> {
    let cpl = match imferno_core::cpl::parse_cpl(cpl_xml) {
        Ok(cpl) => cpl,
        Err(e) => {
            let report = make_error_report("PARSE-CPL-FAILED", &format!("Failed to parse CPL: {}", e));
            return serde_wasm_bindgen::to_value(&report)
                .map_err(|se| JsValue::from_str(&format!("Serialization error: {}", se)));
        }
    };

    let core_spec_target = match core_spec.as_deref().unwrap_or("auto") {
        "auto" => None,
        "v2013" => Some(CoreSpecTarget::St2067_2_2013),
        "v2016" => Some(CoreSpecTarget::St2067_2_2016),
        "v2020" => Some(CoreSpecTarget::St2067_2_2020),
        other => {
            let report = make_error_report(
                "INVALID-CORE-SPEC",
                &format!("Unsupported coreSpec '{}'. Use auto|v2013|v2016|v2020", other),
            );
            return serde_wasm_bindgen::to_value(&report)
                .map_err(|se| JsValue::from_str(&format!("Serialization error: {}", se)));
        }
    };

    let app_spec_targets = match app2e_spec.as_deref().unwrap_or("auto") {
        "auto" => None,
        "none" => Some(vec![]),
        "v2020" => Some(vec![AppSpecTarget::St2067_21_2020]),
        "v2021" => Some(vec![AppSpecTarget::St2067_21_2021]),
        "v2023" => Some(vec![AppSpecTarget::St2067_21_2023]),
        other => {
            let report = make_error_report(
                "INVALID-APP2E-SPEC",
                &format!("Unsupported app2eSpec '{}'. Use auto|none|v2020|v2021|v2023", other),
            );
            return serde_wasm_bindgen::to_value(&report)
                .map_err(|se| JsValue::from_str(&format!("Serialization error: {}", se)));
        }
    };

    let registry = ConfigurableValidatorRegistry::new(ValidatorSelection {
        core_spec: core_spec_target,
        app_specs: app_spec_targets,
        ..Default::default()
    });

    let issues = validate_cpl_with_registry(&cpl, &registry);
    let mut report = ValidationReport::new(ValidationProfile::SMPTE);
    for issue in issues {
        report.add(issue);
    }

    serde_wasm_bindgen::to_value(&report)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Validate a full IMF package from an in-memory map of filename → XML string.
///
/// Pass all XML files from the package as a plain JS object where each key is
/// the filename and each value is the file's text content. ASSETMAP.xml is
/// required; VOLINDEX.xml, PKL files, and CPL files are resolved automatically
/// from the AssetMap.
///
/// Returns a `ValidationReport` serialized to JS.
#[wasm_bindgen(js_name = "validatePackage")]
pub fn validate_package(
    #[wasm_bindgen(js_name = "files")] files_js: JsValue,
    #[wasm_bindgen(js_name = "rules")] rules_js: JsValue,
) -> Result<JsValue, JsValue> {
    let files: std::collections::HashMap<String, String> =
        serde_wasm_bindgen::from_value(files_js)
            .map_err(|e| JsValue::from_str(&format!("Invalid files argument: {}", e)))?;

    let rules: RulesConfig = if rules_js.is_null() || rules_js.is_undefined() {
        Default::default()
    } else {
        serde_wasm_bindgen::from_value(rules_js)
            .map_err(|e| JsValue::from_str(&format!("Invalid rules argument: {}", e)))?
    };

    let options = ValidationOptions { rules, ..Default::default() };
    let report = Imferno::parse_and_validate(files, &options);

    serde_wasm_bindgen::to_value(&report)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Inspect an IMF package and return structural metadata including unreferenced assets.
///
/// Returns `{ cplCount, scmCount, declaredSidecars, unreferencedAssets }` where
/// `unreferencedAssets` are assets in the AssetMap with no CPL Virtual Track reference
/// and no SCM declaration — likely sidecar essences delivered without an SCM.
#[wasm_bindgen(js_name = "inspectPackage")]
pub fn inspect_package(
    #[wasm_bindgen(js_name = "files")] files_js: JsValue,
) -> Result<JsValue, JsValue> {
    let files: std::collections::HashMap<String, String> =
        serde_wasm_bindgen::from_value(files_js)
            .map_err(|e| JsValue::from_str(&format!("Invalid files argument: {}", e)))?;

    let package = match Imferno::parse(files) {
        Ok(p) => p,
        Err(e) => return Err(JsValue::from_str(&format!("Parse error: {}", e))),
    };

    let declared_sidecars: Vec<serde_json::Value> = package.sidecar_composition_maps.values()
        .flat_map(|scm| scm.sidecar_assets.iter().map(|sa| {
            serde_json::json!({
                "id": sa.id.to_string(),
                "cplIds": sa.cpl_ids.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
            })
        }))
        .collect();

    let unreferenced: Vec<serde_json::Value> = package.unreferenced_assets().iter()
        .map(|a| {
            let path = a.chunk_list.chunks.first().map(|c| c.path.as_str()).unwrap_or("");
            serde_json::json!({
                "id": a.id.to_string(),
                "path": path,
            })
        })
        .collect();

    let result = serde_json::json!({
        "cplCount": package.composition_playlists.len(),
        "scmCount": package.sidecar_composition_maps.len(),
        "declaredSidecars": declared_sidecars,
        "unreferencedAssets": unreferenced,
    });

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

// =============================================================================
// Internal helpers
// =============================================================================

/// Build a ValidationReport with a single critical error.
fn make_error_report(code: &str, message: &str) -> ValidationReport {
    let mut report = ValidationReport::new(ValidationProfile::SMPTE);
    report.add(ValidationIssue::new(
        Severity::Critical,
        Category::Structure,
        code,
        message,
    ));
    report
}
