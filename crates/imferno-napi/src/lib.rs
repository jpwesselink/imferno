//! Native Node.js bindings for imferno — SMPTE ST 2067 IMF validator.
//!
//! Provides `buildReport` / `buildReportFromPath` / `buildReportFromUri`,
//! `validate` / `validatePath` / `validateUri`, `formatReport`, and `getVersion`.
//!
//! Platform binaries are published to npm under the `@imferno/node-<platform>`
//! scope (e.g. `@imferno/node-darwin-arm64`); the wrapper at `@imferno/node`
//! resolves the right one at runtime via `optionalDependencies`. The CLI's
//! prebuilt platform packages live under `@imferno/<platform>` (no `node-`
//! prefix) — separate namespace, no collision.

use std::collections::HashMap;
use std::path::PathBuf;

use imferno_core::diagnostics::codes::ValidationCode;
#[allow(deprecated)]
use imferno_core::package::{
    build_report, format_report, validate as validate_package, ImfReport, Imferno, RulesConfig,
    ValidationOptions,
};
use imferno_core::validation::{
    parse_app_spec_targets, parse_core_spec_target, AppSpecTarget, CoreSpecTarget,
};
use napi_derive::napi;
use serde_json::json;
use strum::IntoEnumIterator;

// =============================================================================
// Version
// =============================================================================

#[napi(js_name = "getVersion")]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// =============================================================================
// listRules — enumerate every configurable validation code
// =============================================================================

/// Returns the engine's full catalogue of validation rule codes, one entry per
/// per-spec enum variant. Useful for building settings UIs that render a
/// configurable severity dropdown per rule. The shape is stable JSON, so
/// downstream callers can store severity overrides keyed by `code`.
#[napi(js_name = "listRules")]
pub fn list_rules_js() -> serde_json::Value {
    fn collect<C: ValidationCode + IntoEnumIterator>(spec: &str, out: &mut Vec<serde_json::Value>) {
        for c in C::iter() {
            let mut entry = json!({
                "code": c.code(),
                "spec": spec,
                "description": c.description(),
                "defaultSeverity": format!("{:?}", c.default_severity()).to_lowercase(),
                "category": format!("{:?}", c.category()),
            });
            // Cross-edition annotation: when the enum's code set is
            // bit-for-bit identical to its predecessor (e.g. ST 2067-3:2016 → :2013),
            // expose the predecessor's prefix so UIs can group / hide
            // the duplicate block. Skipped when None to keep the
            // on-wire shape backwards-compatible.
            if let Some(prev) = c.previous_identical_edition() {
                entry["sameAsPreviousEdition"] = json!(prev);
            }
            out.push(entry);
        }
    }

    let mut out: Vec<serde_json::Value> = Vec::new();

    // Core (ASSETMAP / PKL)
    collect::<imferno_core::assetmap::codes::St2067_2_2020>("core", &mut out);

    // CPL — 2013 + 2016. ST 2067-3:2020 reuses the 2016 namespace and is
    // covered by the 2016 rule set (canonical XSD is byte-identical).
    collect::<imferno_core::cpl::codes::St2067_3_2013>("cpl", &mut out);
    collect::<imferno_core::cpl::codes::St2067_3_2016>("cpl", &mut out);

    // App 2E — all three editions
    collect::<imferno_core::validation::codes::St2067_21_2020>("app2e", &mut out);
    collect::<imferno_core::validation::codes::St2067_21_2023>("app2e", &mut out);
    collect::<imferno_core::validation::codes::St2067_21_2025>("app2e", &mut out);

    // Volume Index, MXF, Sidecar Composition Map, ISXD, IAB
    collect::<imferno_core::assetmap::volindex_codes::St429_9_2014>("volindex", &mut out);
    collect::<imferno_core::mxf::codes::St377_1_2011>("mxf", &mut out);
    collect::<imferno_core::scm::codes::St2067_9_2018>("scm", &mut out);
    collect::<imferno_core::validation::isxd_codes::St2067_202_2022>("isxd", &mut out);
    collect::<imferno_core::validation::iab_codes::St2067_201_2019>("iab", &mut out);
    // 2021 catalogue is bit-identical to 2019; the previous_identical_edition
    // annotation lets downstream UIs group / hide the duplicate block.
    collect::<imferno_core::validation::iab_codes::St2067_201_2021>("iab", &mut out);
    // 2026 adds exactly one Annex E recommendation on top of the 2021
    // catalogue; the delta enum surfaces only the new rule.
    collect::<imferno_core::validation::iab_codes::St2067_201_2026Delta>("iab", &mut out);

    // Imferno's own rule namespace (cross-cutting checks beyond pure SMPTE).
    collect::<imferno_core::package::codes::ImfernoCode>("imferno", &mut out);

    serde_json::Value::Array(out)
}

// =============================================================================
// Build report (string-based) — DEPRECATED, use validate() instead
// =============================================================================

#[allow(deprecated)]
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
        aggregate_repeats: opts.aggregate_repeats,
        // Hash verification not yet exposed via NAPI; skip disk checks for in-memory files.
        verify_hashes: None,
        skip_disk_checks: true,
    };

    let package = Imferno::parse(files)
        .map_err(|e| napi::Error::from_reason(format!("Failed to parse IMF package: {}", e)))?;

    let report =
        build_report(&package, &validation_options, None).map_err(napi::Error::from_reason)?;

    serde_json::to_value(&report)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

// =============================================================================
// Build report (path-based) — DEPRECATED, use validatePath() instead
// =============================================================================

#[allow(deprecated)]
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
        aggregate_repeats: opts.aggregate_repeats,
        // Hash verification not yet exposed via NAPI options.
        verify_hashes: None,
        skip_disk_checks: opts.skip_disk_checks,
    };

    let package = Imferno::parse(files)
        .map_err(|e| napi::Error::from_reason(format!("Failed to parse IMF package: {}", e)))?;

    let report =
        build_report(&package, &validation_options, None).map_err(napi::Error::from_reason)?;

    serde_json::to_value(&report)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

// =============================================================================
// Format report — pretty-print an ImfReport as a human-readable string
// =============================================================================

#[allow(deprecated)]
#[napi(js_name = "formatReport")]
pub fn format_report_js(report: serde_json::Value) -> napi::Result<String> {
    let imf_report: ImfReport = serde_json::from_value(report)
        .map_err(|e| napi::Error::from_reason(format!("Invalid ImfReport JSON: {}", e)))?;

    Ok(format_report(&imf_report, false))
}

// =============================================================================
// Parse package — full serialized Imferno struct
// =============================================================================

/// Parse an IMF package from in-memory files, returning the full parsed package.
#[napi(js_name = "parsePackage")]
pub fn parse_package(files: HashMap<String, String>) -> napi::Result<serde_json::Value> {
    let package = Imferno::parse(files)
        .map_err(|e| napi::Error::from_reason(format!("Failed to parse IMF package: {}", e)))?;

    serde_json::to_value(&package)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

/// Parse an IMF package from a directory path, returning the full parsed package.
#[napi(js_name = "parsePackageFromPath")]
pub fn parse_package_from_path(path: String) -> napi::Result<serde_json::Value> {
    let pkg_path = PathBuf::from(&path);
    let files = imferno_core::package::read_dir(&pkg_path)
        .map_err(|e| napi::Error::from_reason(format!("Failed to read directory: {}", e)))?;

    let package = Imferno::parse(files)
        .map_err(|e| napi::Error::from_reason(format!("Failed to parse IMF package: {}", e)))?;

    serde_json::to_value(&package)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

// =============================================================================
// VALIDATE — parse + validate, returns { package, validation }
// =============================================================================

/// Parse and validate an IMF package from in-memory files.
///
/// Returns `{ package, validation }` — the full parsed package and all findings.
#[napi(js_name = "validate")]
pub fn validate_js(
    files: HashMap<String, String>,
    options: Option<serde_json::Value>,
) -> napi::Result<serde_json::Value> {
    let opts = parse_options(options.as_ref())?;
    let validation_options = ValidationOptions {
        rules: opts.rules,
        core_spec: opts.core_spec,
        app_specs: opts.app_specs,
        aggregate_repeats: opts.aggregate_repeats,
        verify_hashes: None,
        skip_disk_checks: true,
    };

    let result = validate_package(files, &validation_options);
    serde_json::to_value(&result)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

/// Parse and validate an IMF package from a directory path.
///
/// Returns `{ package, validation }` — the full parsed package and all findings.
#[napi(js_name = "validatePath")]
pub fn validate_path_js(
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
        aggregate_repeats: opts.aggregate_repeats,
        // Hash verification not yet exposed via NAPI options.
        verify_hashes: None,
        skip_disk_checks: opts.skip_disk_checks,
    };

    let result = validate_package(files, &validation_options);
    serde_json::to_value(&result)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

// =============================================================================
// URI-based variants (file://, s3://, bare paths)
// =============================================================================

#[allow(deprecated)]
#[napi(js_name = "buildReportFromUri")]
pub fn build_report_from_uri(
    uri: String,
    options: Option<serde_json::Value>,
) -> napi::Result<serde_json::Value> {
    let opts = parse_options(options.as_ref())?;
    let files = read_uri(&uri, opts.credentials.as_ref())?;

    let validation_options = ValidationOptions {
        rules: opts.rules,
        core_spec: opts.core_spec,
        app_specs: opts.app_specs,
        aggregate_repeats: opts.aggregate_repeats,
        verify_hashes: None,
        skip_disk_checks: opts.skip_disk_checks,
    };

    let package = Imferno::parse(files)
        .map_err(|e| napi::Error::from_reason(format!("Failed to parse IMF package: {}", e)))?;

    let report =
        build_report(&package, &validation_options, None).map_err(napi::Error::from_reason)?;

    serde_json::to_value(&report)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

#[napi(js_name = "validateUri")]
pub fn validate_uri_js(
    uri: String,
    options: Option<serde_json::Value>,
) -> napi::Result<serde_json::Value> {
    let opts = parse_options(options.as_ref())?;
    let files = read_uri(&uri, opts.credentials.as_ref())?;

    let validation_options = ValidationOptions {
        rules: opts.rules,
        core_spec: opts.core_spec,
        app_specs: opts.app_specs,
        aggregate_repeats: opts.aggregate_repeats,
        verify_hashes: None,
        skip_disk_checks: opts.skip_disk_checks,
    };

    let result = validate_package(files, &validation_options);
    serde_json::to_value(&result)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

fn read_uri(
    uri: &str,
    credentials: Option<&S3CredentialsInput>,
) -> napi::Result<HashMap<String, String>> {
    use imferno_core::package::read_xml_files;
    use imferno_core::storage::fs::FsStorage;
    use imferno_core::storage::{Scheme, StorageUri};

    let parsed = StorageUri::parse(uri)
        .map_err(|e| napi::Error::from_reason(format!("Invalid URI: {}", e)))?;

    match parsed.scheme {
        Scheme::File => {
            // credentials are silently ignored for fs:// URIs.
            let _ = credentials;
            let storage = FsStorage::new();
            read_xml_files(&parsed, &storage)
                .map_err(|e| napi::Error::from_reason(format!("Failed to read URI: {}", e)))
        }
        Scheme::S3 => {
            #[cfg(feature = "aws-s3")]
            {
                let storage = match credentials {
                    Some(c) => imferno_core::storage::s3::S3Storage::from_explicit_creds(
                        c.access_key_id.clone(),
                        c.secret_access_key.clone(),
                        c.session_token.clone(),
                        c.region.clone(),
                        c.endpoint.clone(),
                    )
                    .map_err(|e| napi::Error::from_reason(format!("S3 init: {}", e)))?,
                    None => imferno_core::storage::s3::S3Storage::from_default()
                        .map_err(|e| napi::Error::from_reason(format!("S3 init: {}", e)))?,
                };
                read_xml_files(&parsed, &storage)
                    .map_err(|e| napi::Error::from_reason(format!("Failed to read S3 URI: {}", e)))
            }
            #[cfg(not(feature = "aws-s3"))]
            {
                let _ = credentials;
                Err(napi::Error::from_reason(
                    "s3:// URIs require building imferno-napi with the aws-s3 feature".to_string(),
                ))
            }
        }
    }
}

// =============================================================================
// Internal helpers
// =============================================================================

struct ParsedOptions {
    rules: RulesConfig,
    core_spec: Option<CoreSpecTarget>,
    app_specs: Option<Vec<AppSpecTarget>>,
    aggregate_repeats: bool,
    skip_disk_checks: bool,
    credentials: Option<S3CredentialsInput>,
}

#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "aws-s3"), allow(dead_code))]
struct S3CredentialsInput {
    access_key_id: String,
    secret_access_key: String,
    session_token: Option<String>,
    region: Option<String>,
    endpoint: Option<String>,
}

fn parse_options(options: Option<&serde_json::Value>) -> napi::Result<ParsedOptions> {
    let Some(opts) = options else {
        return Ok(ParsedOptions {
            rules: Default::default(),
            core_spec: None,
            app_specs: None,
            aggregate_repeats: false,
            skip_disk_checks: false,
            credentials: None,
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
        None => None,
        Some(s) => parse_core_spec_target(s).map_err(napi::Error::from_reason)?,
    };

    // App2e spec
    let app_specs = match opts.get("app2eSpec").and_then(|v| v.as_str()) {
        None => None,
        Some(s) => parse_app_spec_targets(s).map_err(napi::Error::from_reason)?,
    };

    let skip_disk_checks = opts
        .get("skipDiskChecks")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let aggregate_repeats = opts
        .get("aggregateRepeats")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Optional explicit S3 credentials (skipped here for non-s3 URIs;
    // read_uri ignores the field unless the scheme is s3).
    let credentials = match opts.get("credentials") {
        None => None,
        Some(v) if v.is_null() => None,
        Some(v) => Some(parse_s3_credentials(v)?),
    };

    Ok(ParsedOptions {
        rules,
        core_spec,
        app_specs,
        aggregate_repeats,
        skip_disk_checks,
        credentials,
    })
}

fn parse_s3_credentials(value: &serde_json::Value) -> napi::Result<S3CredentialsInput> {
    let access_key_id = value
        .get("accessKeyId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| napi::Error::from_reason("credentials.accessKeyId is required".to_string()))?
        .to_string();
    let secret_access_key = value
        .get("secretAccessKey")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            napi::Error::from_reason("credentials.secretAccessKey is required".to_string())
        })?
        .to_string();
    let session_token = value
        .get("sessionToken")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let region = value
        .get("region")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let endpoint = value
        .get("endpoint")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    Ok(S3CredentialsInput {
        access_key_id,
        secret_access_key,
        session_token,
        region,
        endpoint,
    })
}
