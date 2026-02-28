//! Native Node.js bindings for imferno — SMPTE ST 2067 IMF validator.
//!
//! Provides the same API as `@imferno/wasm` plus path-based validation
//! with filesystem access (hash verification, MXF header checks).

use std::collections::HashMap;
use std::path::PathBuf;

use imferno_core::package::{Imferno, RulesConfig, ValidationOptions};
use imferno_core::validation::{AppSpecTarget, CoreSpecTarget};
use imferno_core::{Category, Severity, ValidationIssue, ValidationProfile, ValidationReport};
use napi_derive::napi;

// =============================================================================
// Version
// =============================================================================

#[napi(js_name = "getVersion")]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// =============================================================================
// Individual parsers
// =============================================================================

#[napi(js_name = "parseCpl")]
pub fn parse_cpl(xml: String) -> napi::Result<serde_json::Value> {
    let cpl = imferno_core::cpl::parse_cpl(&xml)
        .map_err(|e| napi::Error::from_reason(format!("Parse error: {}", e)))?;
    serde_json::to_value(&cpl)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

#[napi(js_name = "parseAssetmap")]
pub fn parse_assetmap(xml: String) -> napi::Result<serde_json::Value> {
    let am = imferno_core::assetmap::parse_assetmap(&xml)
        .map_err(|e| napi::Error::from_reason(format!("Parse error: {}", e)))?;
    serde_json::to_value(&am)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

#[napi(js_name = "parsePkl")]
pub fn parse_pkl(xml: String) -> napi::Result<serde_json::Value> {
    let pkl = imferno_core::assetmap::parse_pkl(&xml)
        .map_err(|e| napi::Error::from_reason(format!("Parse error: {}", e)))?;
    serde_json::to_value(&pkl)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

#[napi(js_name = "parseVolindex")]
pub fn parse_volindex(xml: String) -> napi::Result<serde_json::Value> {
    let vi = imferno_core::assetmap::parse_volindex(&xml)
        .map_err(|e| napi::Error::from_reason(format!("Parse error: {}", e)))?;
    serde_json::to_value(&vi)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

// =============================================================================
// Validate (string-based) — same API as @imferno/wasm
// =============================================================================

#[napi(js_name = "validate")]
pub fn validate(
    files: HashMap<String, String>,
    options: Option<serde_json::Value>,
) -> napi::Result<serde_json::Value> {
    let opts = parse_options(options.as_ref())?;

    let validation_options = ValidationOptions {
        rules: opts.rules,
        core_spec: opts.core_spec,
        app_specs: opts.app_specs,
        verify_hashes: None,
        skip_disk_checks: true, // no disk for string-based
    };

    do_validate_package(files, &validation_options)
}

// =============================================================================
// ValidatePath — the NAPI-only path-based function
// =============================================================================

#[napi(js_name = "validatePath")]
pub fn validate_path(
    path: String,
    options: Option<serde_json::Value>,
) -> napi::Result<serde_json::Value> {
    let opts = parse_options(options.as_ref())?;
    let pkg_path = PathBuf::from(&path);

    // Read all XML files from the directory
    let files = imferno_core::package::read_dir(&pkg_path)
        .map_err(|e| napi::Error::from_reason(format!("Failed to read directory: {}", e)))?;

    let verify_hashes = if opts.verify_hashes {
        Some(pkg_path.clone())
    } else {
        None
    };

    let validation_options = ValidationOptions {
        rules: opts.rules,
        core_spec: opts.core_spec,
        app_specs: opts.app_specs,
        verify_hashes,
        skip_disk_checks: opts.skip_disk_checks,
    };

    do_validate_package(files, &validation_options)
}

// =============================================================================
// Internal helpers
// =============================================================================

struct ParsedOptions {
    rules: RulesConfig,
    core_spec: Option<CoreSpecTarget>,
    app_specs: Option<Vec<AppSpecTarget>>,
    verify_hashes: bool,
    skip_disk_checks: bool,
}

fn parse_options(options: Option<&serde_json::Value>) -> napi::Result<ParsedOptions> {
    let Some(opts) = options else {
        return Ok(ParsedOptions {
            rules: Default::default(),
            core_spec: None,
            app_specs: None,
            verify_hashes: false,
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

    let verify_hashes = opts
        .get("verifyHashes")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let skip_disk_checks = opts
        .get("skipDiskChecks")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    Ok(ParsedOptions {
        rules,
        core_spec,
        app_specs,
        verify_hashes,
        skip_disk_checks,
    })
}

fn do_validate_package(
    files: HashMap<String, String>,
    options: &ValidationOptions,
) -> napi::Result<serde_json::Value> {
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
            return Ok(serde_json::json!({
                "report": report,
                "cpls": [],
                "assetMap": null,
                "packingLists": [],
                "volumeIndex": null,
                "unreferencedAssets": [],
                "declaredSidecars": [],
            }));
        }
    };

    // Validate
    let report = package.validate(options);

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

    Ok(serde_json::json!({
        "report": report,
        "cpls": cpls,
        "assetMap": package.asset_map,
        "packingLists": packing_lists,
        "volumeIndex": package.volume_index,
        "unreferencedAssets": unreferenced,
        "declaredSidecars": declared_sidecars,
    }))
}
