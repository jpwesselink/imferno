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

#[allow(deprecated)]
use imferno_core::package::{
    build_report, format_json_report, validate as validate_package, Imferno, RulesConfig,
    ValidationOptions,
};
use imferno_core::validation::{
    parse_app_spec_targets, parse_core_spec_target, parse_cpl_spec_target, AppSpecTarget,
    CoreSpecTarget, CplSpecTarget,
};
use napi_derive::napi;

// =============================================================================
// Version
// =============================================================================

/// Returns the imferno library version string.
#[napi(js_name = "getVersion")]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// =============================================================================
// listRules — enumerate every configurable validation code
// =============================================================================

/// Returns the engine's rule catalogue: one entry per rule, each with its
/// table of editions (clause, strength, default severity, display code).
/// Useful for building settings UIs; `id`, `name`, `SPEC:EDITION/Name` and
/// the display codes are all usable as `rules` keys.
#[napi(js_name = "listRules", ts_return_type = "RuleInfo[]")]
pub fn list_rules_js() -> serde_json::Value {
    serde_json::to_value(imferno_core::catalogue::rule_infos()).unwrap_or_default()
}

// =============================================================================
// Build report (string-based) — DEPRECATED, use validate() instead
// =============================================================================

/// Validate an IMF package from in-memory XML strings, returning the legacy
/// `ImfReport` summary. Prefer `validate` for new code.
///
/// Disk checks are always skipped (there is no package on disk).
///
/// @param files - A map of filename to XML content (e.g. `{ "ASSETMAP.xml": "..." }`)
/// @param options - Optional validation options (`coreSpec`, `cplSpec`, `app2eSpec`, `rules`, `aggregateRepeats`)
/// @returns An `ImfReport` with `package`, `cpls`, and `validation` fields.
/// @throws If the package cannot be parsed or the options are invalid.
#[allow(deprecated)]
#[napi(
    js_name = "buildReport",
    ts_args_type = "files: Record<string, string>, options?: ValidateOptions | undefined | null",
    ts_return_type = "ImfReport"
)]
pub fn build_report_js(
    files: HashMap<String, String>,
    options: Option<serde_json::Value>,
) -> napi::Result<serde_json::Value> {
    let opts = parse_options(options.as_ref())?;
    let validation_options = ValidationOptions {
        rules: opts.rules,
        core_spec: opts.core_spec,
        cpl_spec: opts.cpl_spec,
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

/// Validate an IMF package directory on disk, returning the legacy
/// `ImfReport` summary. Prefer `validatePath` for new code.
///
/// @param path - Path to the IMP directory
/// @param options - Optional validation options (`coreSpec`, `cplSpec`, `app2eSpec`, `skipDiskChecks`, `rules`, `aggregateRepeats`)
/// @returns An `ImfReport` with `package`, `cpls`, and `validation` fields.
/// @throws If the directory cannot be read, the package cannot be parsed, or the options are invalid.
#[allow(deprecated)]
#[napi(
    js_name = "buildReportFromPath",
    ts_args_type = "path: string, options?: PathValidateOptions | undefined | null",
    ts_return_type = "ImfReport"
)]
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
        cpl_spec: opts.cpl_spec,
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

/// Pretty-print a validation result as a human-readable string.
///
/// @param report - The object returned by `validate*`, or (legacy) by `buildReport*`
/// @returns A formatted plain-text report.
/// @throws If `report` is neither a `ValidationResult` nor an `ImfReport`.
#[napi(
    js_name = "formatReport",
    ts_args_type = "report: ValidationResult | ImfReport"
)]
pub fn format_report_js(report: serde_json::Value) -> napi::Result<String> {
    format_json_report(report).map_err(napi::Error::from_reason)
}

// =============================================================================
// SARIF — validation findings as a SARIF 2.1.0 log
// =============================================================================

/// Convert a validation result to a SARIF 2.1.0 log.
///
/// Rules are reported by rule id (`ST2067-3/SegmentDuration`); each finding
/// points at the applied edition's clause in that edition's taxonomy
/// (`ST2067-3:2016`).
///
/// @param report - The object returned by `validate*` (or its `validation`)
/// @returns A SARIF 2.1.0 log object.
/// @throws If `report` is not a validation result or report.
#[napi(
    js_name = "toSarif",
    ts_args_type = "report: ValidationResult | ValidationReport",
    ts_return_type = "SarifLog"
)]
pub fn to_sarif_js(report: serde_json::Value) -> napi::Result<serde_json::Value> {
    imferno_core::diagnostics::sarif_from_json(report).map_err(napi::Error::from_reason)
}

// =============================================================================
// Parse package — full serialized Imferno struct
// =============================================================================

/// Parse an IMF package from in-memory XML strings, returning the full parsed
/// package without validating it.
///
/// @param files - A map of filename to XML content (e.g. `{ "ASSETMAP.xml": "..." }`)
/// @returns The full parsed `Imferno` package.
/// @throws If the package cannot be parsed.
#[napi(js_name = "parsePackage", ts_return_type = "Imferno")]
pub fn parse_package(files: HashMap<String, String>) -> napi::Result<serde_json::Value> {
    let package = Imferno::parse(files)
        .map_err(|e| napi::Error::from_reason(format!("Failed to parse IMF package: {}", e)))?;

    serde_json::to_value(&package)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

/// Parse an IMF package directory on disk, returning the full parsed package
/// without validating it.
///
/// @param path - Path to the IMP directory
/// @returns The full parsed `Imferno` package.
/// @throws If the directory cannot be read or the package cannot be parsed.
#[napi(js_name = "parsePackageFromPath", ts_return_type = "Imferno")]
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

/// Parse and validate an IMF package from in-memory XML strings. This is the
/// recommended entry point.
///
/// Parse failures are reported as findings in `validation`, not thrown.
/// Disk checks are always skipped (there is no package on disk).
///
/// @param files - A map of filename to XML content
/// @param options - Optional validation options
/// @returns `{ package, validation }` — the full parsed package and all findings.
/// @throws Only if the options are invalid.
#[napi(
    js_name = "validate",
    ts_args_type = "files: Record<string, string>, options?: ValidateOptions | undefined | null",
    ts_return_type = "ValidationResult"
)]
pub fn validate_js(
    files: HashMap<String, String>,
    options: Option<serde_json::Value>,
) -> napi::Result<serde_json::Value> {
    let opts = parse_options(options.as_ref())?;
    let validation_options = ValidationOptions {
        rules: opts.rules,
        core_spec: opts.core_spec,
        cpl_spec: opts.cpl_spec,
        app_specs: opts.app_specs,
        aggregate_repeats: opts.aggregate_repeats,
        verify_hashes: None,
        skip_disk_checks: true,
    };

    let result = validate_package(files, &validation_options);
    serde_json::to_value(&result)
        .map_err(|e| napi::Error::from_reason(format!("Serialization error: {}", e)))
}

/// Parse and validate an IMF package directory on disk.
///
/// @param path - Path to the IMP directory
/// @param options - Optional validation options
/// @returns `{ package, validation }` — the full parsed package and all findings.
/// @throws If the directory cannot be read or the options are invalid.
#[napi(
    js_name = "validatePath",
    ts_args_type = "path: string, options?: PathValidateOptions | undefined | null",
    ts_return_type = "ValidationResult"
)]
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
        cpl_spec: opts.cpl_spec,
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

/// Build the legacy `ImfReport` summary from a URI. Accepts the same URI forms
/// as `validateUri`; prefer `validateUri` for new code.
///
/// @param uri - Path or URI to the IMP
/// @param options - Optional validation options
/// @returns An `ImfReport` with `package`, `cpls`, and `validation` fields.
/// @throws If the URI cannot be read, the package cannot be parsed, or the options are invalid.
#[allow(deprecated)]
#[napi(
    js_name = "buildReportFromUri",
    ts_args_type = "uri: string, options?: UriValidateOptions | undefined | null",
    ts_return_type = "ImfReport"
)]
pub fn build_report_from_uri(
    uri: String,
    options: Option<serde_json::Value>,
) -> napi::Result<serde_json::Value> {
    let opts = parse_options(options.as_ref())?;
    let files = read_uri(&uri, opts.credentials.as_ref())?;

    let validation_options = ValidationOptions {
        rules: opts.rules,
        core_spec: opts.core_spec,
        cpl_spec: opts.cpl_spec,
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

/// Parse and validate an IMF package from a URI.
///
/// Accepts `file://` URIs, bare filesystem paths, and (when built with the
/// `aws-s3` feature, as the prebuilt npm binaries are) `s3://bucket/prefix/`.
///
/// @param uri - Path or URI to the IMP
/// @param options - Optional validation options (plus `credentials` for S3)
/// @returns `{ package, validation }` — the full parsed package and all findings.
/// @throws If the URI cannot be read or the options are invalid.
#[napi(
    js_name = "validateUri",
    ts_args_type = "uri: string, options?: UriValidateOptions | undefined | null",
    ts_return_type = "ValidationResult"
)]
pub fn validate_uri_js(
    uri: String,
    options: Option<serde_json::Value>,
) -> napi::Result<serde_json::Value> {
    let opts = parse_options(options.as_ref())?;
    let files = read_uri(&uri, opts.credentials.as_ref())?;

    let validation_options = ValidationOptions {
        rules: opts.rules,
        core_spec: opts.core_spec,
        cpl_spec: opts.cpl_spec,
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
    cpl_spec: Option<CplSpecTarget>,
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
            cpl_spec: None,
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

    // CPL (ST 2067-3) spec
    let cpl_spec = match opts.get("cplSpec").and_then(|v| v.as_str()) {
        None => None,
        Some(s) => parse_cpl_spec_target(s).map_err(napi::Error::from_reason)?,
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
        cpl_spec,
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
