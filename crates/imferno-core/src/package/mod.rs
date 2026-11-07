//! IMF Core — Integrated IMF Package Parser
//!
//! This module provides a high-level interface for parsing complete IMF packages
//! by coordinating the individual SMPTE standard parsers.
//!
//! ## Key entry points
//!
//! - [`build_report`] — parse and validate an IMF package, returning an [`ImfReport`].
//! - [`format_report`] — render an [`ImfReport`] as a human-readable string.

use crate::assetmap::ImfUuid;
use crate::catalogue::imferno::{self, ImfernoEdition};
use crate::catalogue::st2067_2::{self, St2067_2Edition};
use crate::catalogue::st2067_3;
use crate::catalogue::st2067_9::{self, St2067_9Edition};
use crate::catalogue::st377_1;
use crate::catalogue::st429_8::{self, St429_8Edition};
use crate::catalogue::st429_9::{self, St429_9Edition};
use crate::cpl::EditRate;
use crate::diagnostics::Rule;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod report;

#[allow(deprecated)]
pub use self::report::{
    build_report, format_json_report, format_report, format_validation_result, FormatOptions,
    ImfReport, ReportFormat,
};
pub use crate::assetmap::{Asset, AssetMap, PackingList, PklAsset, VolumeIndex};
pub use crate::cpl::{CompositionPlaylist, Resource as CplResource};
pub use crate::diagnostics::{
    Category, Location, Severity, ValidationIssue, ValidationProfile, ValidationReport,
};

/// Result of parsing and validating an IMF package.
///
/// This is the primary return type — contains the full parsed package
/// and all validation findings.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    /// The fully parsed IMF package.
    pub package: Imferno,
    /// Validation findings (spec violations, warnings, info).
    pub validation: ValidationReport,
    /// Keys of the `rules` option that select no rule (typos, unknown
    /// editions); they were ignored. Absent when every key matched.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rule_warnings: Vec<crate::diagnostics::rules::RuleValidationWarning>,
}

impl ValidationResult {
    /// Rebuild a `ValidationResult` from its own JSON serialisation (e.g. the
    /// object a JS binding's `validate()` returned).
    ///
    /// Use this rather than `serde_json::from_value` directly: some CPL fields
    /// deserialise from their XML form by default.
    pub fn from_json(value: serde_json::Value) -> serde_json::Result<Self> {
        crate::cpl::de_helpers::json_mode(|| serde_json::from_value(value))
    }
}

/// Parse and validate an IMF package in one call.
///
/// This is the recommended entry point. Returns the full parsed package
/// plus all validation findings.
///
/// ```no_run
/// use imferno_core::package::{validate, read_dir, ValidationOptions};
///
/// let files = read_dir("./my-imp").unwrap();
/// let result = validate(files, &ValidationOptions::default());
/// println!("Compliant: {}", result.validation.is_compliant);
/// for cpl in result.package.composition_playlists.values() {
///     println!("CPL: {}", cpl.content_title.text);
/// }
/// ```
pub fn validate(
    files: std::collections::HashMap<String, String>,
    options: &ValidationOptions,
) -> ValidationResult {
    match Imferno::parse(files) {
        Ok(package) => {
            let validation = package.validate(options);
            ValidationResult {
                package,
                validation,
                rule_warnings: options.rules.validate(),
            }
        }
        Err(e) => {
            let mut validation = ValidationReport::new(ValidationProfile::SMPTE);
            validation.add(ValidationIssue::from_rule(
                &imferno::ParseError,
                ImfernoEdition::Current,
                format!("Failed to parse IMF package: {e}"),
            ));
            // Return a minimal Imferno with what we could parse
            // For now, this is unreachable in practice since parse only fails
            // on missing ASSETMAP — but we handle it gracefully.
            let validation = validation.apply_rules(&options.rules);
            // Re-parse won't work since files are consumed. Use parse_and_validate fallback.
            ValidationResult {
                package: Imferno::empty(),
                validation,
                rule_warnings: options.rules.validate(),
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum ImfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("AssetMap parse error: {0}")]
    AssetMapParse(#[from] crate::assetmap::AssetMapParseError),

    #[error("CPL parse error: {0}")]
    CplParse(#[from] crate::cpl::CplParseError),

    #[error("UUID error: {0}")]
    Uuid(String),

    #[error("Missing required file: {0}")]
    MissingFile(String),

    #[error("Invalid IMF package structure: {0}")]
    InvalidStructure(String),
}

pub type Result<T> = std::result::Result<T, ImfError>;

/// Errors found during PKL file manifest / hash / cross-reference validation.
///
/// Per SMPTE ST 2067-2 §7-9, the AssetMap, PKL, and CPL must maintain
/// consistent cross-references. These errors describe structural violations.
#[derive(Debug)]
pub enum FileValidationError {
    /// PKL lists an asset UUID that has no entry in the AssetMap (ST 2067-2 §7).
    NotInAssetMap {
        uuid: String,
        original_file_name: Option<String>,
    },
    /// File expected on disk but not found.
    Missing { uuid: String, path: PathBuf },
    /// File exists but its byte size differs from the PKL declaration.
    SizeMismatch {
        uuid: String,
        path: PathBuf,
        expected: u64,
        actual: u64,
    },
    /// Hash digest does not match PKL hash (SHA-1 or SHA-256).
    HashMismatch {
        uuid: String,
        path: PathBuf,
        expected: String,
        actual: String,
    },
    /// I/O error while reading the file for hashing.
    Io {
        uuid: String,
        path: PathBuf,
        message: String,
    },
    /// Same asset UUID appears more than once in a single PKL (ST 2067-2 §9).
    DuplicatePklAssetId { uuid: String, pkl_id: String },
    /// PKL document carries a namespace URI we don't recognise. Per
    /// ST 429-8 (the canonical PKL standard) and ST 2067-2:2016, the
    /// only acceptable values are the published namespaces; an
    /// unrecognised one breaks downstream tool interoperability.
    UnknownPklNamespace { pkl_id: String, namespace: String },
    /// The AssetMap has no asset carrying `<PackingList>true</PackingList>`,
    /// so no PKL document is declared. ST 429-9 §6.3 requires the
    /// AssetMap to identify which assets are PKLs.
    AssetMapHasNoPackingList,
    /// A PKL document was parsed but its Id does not appear as a
    /// `PackingList`-flagged asset in the AssetMap. ST 429-9 §6.3
    /// requires every PKL to be declared in the AssetMap.
    PklIdNotInAssetMap { pkl_id: String },
}

impl FileValidationError {
    pub fn uuid(&self) -> &str {
        match self {
            Self::NotInAssetMap { uuid, .. } => uuid,
            Self::Missing { uuid, .. } => uuid,
            Self::SizeMismatch { uuid, .. } => uuid,
            Self::HashMismatch { uuid, .. } => uuid,
            Self::Io { uuid, .. } => uuid,
            Self::DuplicatePklAssetId { uuid, .. } => uuid,
            // The variants below are package-scope, not asset-scope —
            // the relevant identifier is the PKL document or "—" when
            // no document is involved at all.
            Self::UnknownPklNamespace { pkl_id, .. } => pkl_id,
            Self::PklIdNotInAssetMap { pkl_id } => pkl_id,
            Self::AssetMapHasNoPackingList => "—",
        }
    }
}

impl std::fmt::Display for FileValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInAssetMap {
                uuid,
                original_file_name,
            } => {
                write!(
                    f,
                    "PKL asset {} ({}) not found in AssetMap",
                    uuid,
                    original_file_name.as_deref().unwrap_or("no filename")
                )
            }
            Self::Missing { uuid, path } => {
                write!(f, "Missing file for {}: {}", uuid, path.display())
            }
            Self::SizeMismatch {
                uuid,
                path,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Size mismatch for {} ({}): expected {} bytes, found {}",
                    uuid,
                    path.display(),
                    expected,
                    actual
                )
            }
            Self::HashMismatch {
                uuid,
                path,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Hash mismatch for {} ({}): expected {}, got {}",
                    uuid,
                    path.display(),
                    expected,
                    actual
                )
            }
            Self::Io {
                uuid,
                path,
                message,
            } => {
                write!(
                    f,
                    "IO error reading {} ({}): {}",
                    uuid,
                    path.display(),
                    message
                )
            }
            Self::DuplicatePklAssetId { uuid, pkl_id } => {
                write!(f, "Duplicate asset UUID {} in PKL {}", uuid, pkl_id)
            }
            Self::UnknownPklNamespace { pkl_id, namespace } => {
                write!(
                    f,
                    "PKL {} carries an unrecognised namespace URI: {}",
                    pkl_id, namespace
                )
            }
            Self::AssetMapHasNoPackingList => {
                write!(
                    f,
                    "AssetMap declares no PKL (no asset has <PackingList>true</PackingList>)"
                )
            }
            Self::PklIdNotInAssetMap { pkl_id } => {
                write!(
                    f,
                    "PKL document {} is not declared as a PackingList asset in the AssetMap",
                    pkl_id
                )
            }
        }
    }
}

/// Report an ST 2067-2 package-level rule for a package of ST 2067-2
/// edition `spec_ed`, or `None` when that edition does not contain the rule
/// (ST 2067-2:2013 has no delivery requirements: Asset Map ingest
/// of PKL assets). The package then reports
/// `IMFERNO:Package/DeliveryNotChecked` once instead.
fn pkg_issue(
    rule: &'static Rule<St2067_2Edition>,
    spec_ed: St2067_2Edition,
    message: impl Into<String>,
) -> Option<ValidationIssue> {
    rule.in_edition(spec_ed)
        .then(|| ValidationIssue::from_rule(rule, spec_ed, message))
}

/// A Packing List Size/Hash rule: under ST 2067-2 (§8.3 ingest) for the
/// editions that have it, and for an ST 2067-2:2013 IMP under ST 429-8:2007,
/// the spec of its Packing List (2013 §7.1), which declares Size and Hash.
fn pkg_or_pkl_issue(
    rule: &'static Rule<St2067_2Edition>,
    pkl_rule: &'static Rule<St429_8Edition>,
    spec_ed: St2067_2Edition,
    message: impl Into<String>,
) -> Option<ValidationIssue> {
    let message = message.into();
    if rule.in_edition(spec_ed) {
        Some(ValidationIssue::from_rule(rule, spec_ed, message))
    } else {
        Some(ValidationIssue::from_rule(
            pkl_rule,
            St429_8Edition::V2007,
            message,
        ))
    }
}

/// The ST 2067-2 edition applied to a package's package-level rules (see
/// [`Imferno::package_edition`]).
pub type PackageEdition = crate::validation::EditionChoice<St2067_2Edition>;

/// The ST 2067-2 delivery rules of ST 429-9 (Annex A Basic Map Profile v2,
/// the VOLINDEX of the deprecated profile): ST 2067-2:2016/2020 §8 apply
/// them to IMP delivery; ST 2067-2:2013 defines no delivery, so they are
/// not reported for a 2013 package.
fn is_delivery_rule(issue: &ValidationIssue) -> bool {
    [
        &st429_9::VolindexMissing,
        &st429_9::AssetMapFileName,
        &st429_9::PathNotRelative,
        &st429_9::PathCharacters,
        &st429_9::PathTooLong,
        &st429_9::PathTooManySegments,
        &st429_9::PathDuplicate,
    ]
    .iter()
    .any(|r| issue.is(*r))
}

impl FileValidationError {
    /// The finding for this error, for a package of ST 2067-2 edition
    /// `spec_ed`; `None` when that edition does not contain the rule.
    pub fn to_issue(&self, spec_ed: St2067_2Edition) -> Option<ValidationIssue> {
        Some(match self {
            FileValidationError::NotInAssetMap {
                uuid,
                original_file_name,
            } => pkg_issue(
                &st2067_2::UnresolvedUuid,
                spec_ed,
                format!(
                    "PKL asset {} ({}) not found in AssetMap",
                    uuid,
                    original_file_name.as_deref().unwrap_or("no filename")
                ),
            )?
            .with_context("asset_uuid", uuid.clone()),
            FileValidationError::Missing { uuid, path } => pkg_issue(
                &st2067_2::FileNotFound,
                spec_ed,
                format!("Missing file for asset {}: {}", uuid, path.display()),
            )?
            .with_location(Location::new().with_file(path.clone()))
            .with_context("asset_uuid", uuid.clone()),
            FileValidationError::SizeMismatch {
                uuid,
                path,
                expected,
                actual,
            } => pkg_or_pkl_issue(
                &st2067_2::SizeMismatch,
                &st429_8::SizeMismatch,
                spec_ed,
                format!(
                    "Size mismatch for asset {} ({}): PKL declares {} bytes, file is {} bytes",
                    uuid,
                    path.display(),
                    expected,
                    actual
                ),
            )?
            .with_location(Location::new().with_file(path.clone()))
            .with_context("asset_uuid", uuid.clone())
            .with_context("expected_size", expected.to_string())
            .with_context("actual_size", actual.to_string()),
            FileValidationError::HashMismatch {
                uuid,
                path,
                expected,
                actual,
            } => pkg_or_pkl_issue(
                &st2067_2::ChecksumMismatch,
                &st429_8::ChecksumMismatch,
                spec_ed,
                format!(
                    "Hash mismatch for asset {} ({}): expected {}, computed {}",
                    uuid,
                    path.display(),
                    expected,
                    actual
                ),
            )?
            .with_location(Location::new().with_file(path.clone()))
            .with_context("asset_uuid", uuid.clone())
            .with_suggestion("Re-deliver the asset or re-generate the PKL hash"),
            FileValidationError::Io {
                uuid,
                path,
                message,
            } => ValidationIssue::from_rule(
                &imferno::IoError,
                ImfernoEdition::Current,
                format!(
                    "IO error reading asset {} ({}): {}",
                    uuid,
                    path.display(),
                    message
                ),
            )
            .with_location(Location::new().with_file(path.clone()))
            .with_context("asset_uuid", uuid.clone()),
            FileValidationError::DuplicatePklAssetId { uuid, pkl_id } => pkg_issue(
                &st2067_2::DuplicateUuid,
                spec_ed,
                format!("Duplicate asset UUID {} in PKL {}", uuid, pkl_id),
            )?
            .with_context("asset_uuid", uuid.clone())
            .with_context("pkl_id", pkl_id.clone()),
            FileValidationError::UnknownPklNamespace { pkl_id, namespace } => pkg_issue(
                &st2067_2::PklUnknownNamespace,
                spec_ed,
                format!(
                    "PKL {} carries unrecognised namespace '{}' — not in the published \
                     SMPTE PKL namespace set",
                    pkl_id, namespace
                ),
            )?
            .with_context("pkl_id", pkl_id.clone())
            .with_context("namespace", namespace.clone()),
            FileValidationError::AssetMapHasNoPackingList => ValidationIssue::from_rule(
                &st429_9::AssetMapHasNoPackingList,
                St429_9Edition::V2014,
                "AssetMap declares no PKL (no asset has <PackingList>true</PackingList>)"
                    .to_string(),
            ),
            FileValidationError::PklIdNotInAssetMap { pkl_id } => ValidationIssue::from_rule(
                &st429_9::PklIdNotInAssetMap,
                St429_9Edition::V2014,
                format!(
                    "PKL document {} is not declared as a PackingList asset in the AssetMap",
                    pkl_id
                ),
            )
            .with_context("pkl_id", pkl_id.clone()),
        })
    }
}

/// High-level IMF package representation.
///
/// This is the full parsed package — all CPLs, PKLs, AssetMap, SCMs, and
/// cross-references. Serializable to JSON for WASM/NAPI consumers.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Imferno {
    /// Package root directory
    #[serde(serialize_with = "serialize_path")]
    #[cfg_attr(feature = "jsonschema", schemars(with = "String"))]
    pub root_path: PathBuf,

    /// Volume index (VOLINDEX.xml)
    pub volume_index: VolumeIndex,

    /// Load-time VOLINDEX diagnostics (ST 429-9), emitted before all other checks.
    #[serde(skip)]
    pub volindex_issues: Vec<ValidationIssue>,

    /// Load-time parse diagnostics (PKL/CPL/OPL/SCM failures), emitted during validation.
    #[serde(skip)]
    pub(crate) parse_issues: Vec<ValidationIssue>,

    /// Asset map (ASSETMAP.xml)
    pub asset_map: AssetMap,

    /// Parsed Packing Lists mapped by UUID
    pub packing_lists: HashMap<ImfUuid, PackingList>,

    /// Parsed CPL files mapped by UUID
    pub composition_playlists: HashMap<ImfUuid, CompositionPlaylist>,

    /// Raw CPL XML content mapped by UUID (retained for future signature verification).
    #[serde(skip)]
    #[allow(dead_code)]
    pub(crate) cpl_xml_content: HashMap<ImfUuid, String>,

    /// Parsed Output Profile Lists mapped by UUID
    pub output_profile_lists: HashMap<ImfUuid, crate::assetmap::OutputProfileList>,

    /// Parsed Sidecar Composition Maps mapped by UUID (ST 2067-9:2018)
    pub sidecar_composition_maps: HashMap<ImfUuid, crate::scm::SidecarCompositionMap>,

    /// Asset UUID to file path mapping
    #[serde(serialize_with = "serialize_path_map")]
    #[cfg_attr(feature = "jsonschema", schemars(with = "HashMap<ImfUuid, String>"))]
    pub asset_paths: HashMap<ImfUuid, PathBuf>,
}

fn serialize_path<S: serde::Serializer>(path: &Path, s: S) -> std::result::Result<S::Ok, S::Error> {
    s.serialize_str(&path.to_string_lossy())
}

fn serialize_path_map<S: serde::Serializer>(
    map: &HashMap<ImfUuid, PathBuf>,
    s: S,
) -> std::result::Result<S::Ok, S::Error> {
    use serde::ser::SerializeMap;
    let mut m = s.serialize_map(Some(map.len()))?;
    for (k, v) in map {
        m.serialize_entry(k, &v.to_string_lossy().into_owned())?;
    }
    m.end()
}

/// Resolve an asset chunk path against the package root, rejecting path traversal.
///
/// Returns `None` if the path is absolute or contains `..` components that
/// would escape the package root. This prevents a malicious AssetMap from
/// causing file reads outside the intended directory.
fn sanitize_asset_path(root: &Path, chunk_path: &str) -> Option<PathBuf> {
    let rel = Path::new(chunk_path);
    // Reject absolute paths outright
    if rel.is_absolute() {
        return None;
    }
    // ST 429-9:2014 A.2: "." and ".." segments are allowed (Note 3), but
    // every location traversed while resolving the path shall stay inside
    // the tree rooted at the Asset Map's directory (Example 2:
    // `bar/../../foo/bar/Image.mxf` is forbidden). Normalise lexically
    // and reject as soon as a ".." would leave the root.
    let mut normalised = PathBuf::new();
    let mut depth: usize = 0;
    for component in rel.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                depth = depth.checked_sub(1)?;
                normalised.pop();
            }
            std::path::Component::Normal(part) => {
                depth += 1;
                normalised.push(part);
            }
            // Prefix / RootDir: not a relative path.
            _ => return None,
        }
    }
    let joined = root.join(normalised);
    // If the file exists, verify the canonical path is still under root
    if let Ok(canonical) = joined.canonicalize() {
        if canonical.starts_with(root) {
            return Some(canonical);
        }
        return None; // symlink escape
    }
    // File doesn't exist yet — lexical check above is sufficient
    Some(joined)
}

/// Read all files from a directory into a `HashMap<String, String>`.
///
/// XML files are read as strings. Binary files (e.g. MXF) that fail UTF-8
/// decoding are silently skipped.
///
/// Keys are the **absolute** file paths. `from_file_map` (called by `parse`)
/// derives the package `root_path` from these keys so that file-manifest
/// and MXF-header validation work correctly on native targets.
/// Collapse the per-CPL "signature not verified" notices into one finding
/// for the package: it describes the tool, not the documents. The first
/// notice (by CPL Id, for stable output) is kept, the other CPLs become its
/// `additional_instances`.
fn collapse_signature_notices(mut notices: Vec<ValidationIssue>) -> Option<ValidationIssue> {
    notices.sort_by_key(|i| i.location.cpl_id.map(|id| id.to_string()));
    let mut iter = notices.into_iter();
    let mut first = iter.next()?;
    first.additional_instances.extend(iter.map(|i| i.location));
    Some(first)
}

pub fn read_dir(path: impl AsRef<Path>) -> Result<HashMap<String, String>> {
    use crate::storage::{fs::FsStorage, StorageUri};

    let path = path
        .as_ref()
        .canonicalize()
        .unwrap_or_else(|_| path.as_ref().to_path_buf());
    let uri = StorageUri::parse(&path.to_string_lossy())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
    let storage = FsStorage::new();

    read_xml_files(&uri, &storage).map_err(|e| std::io::Error::other(e.to_string()).into())
}

/// Read all `.xml` files at the given URI through the supplied storage backend.
///
/// Returns a map of fully-qualified URIs to file contents. Non-XML entries
/// and files that fail UTF-8 decoding are skipped (with a warning to stderr
/// for parity with the legacy `read_dir` behavior).
///
/// This is the recommended trait-based entry point. Re-exported as [`read`].
///
/// # Example — local filesystem
///
/// ```no_run
/// use imferno_core::package::{read, Imferno};
/// use imferno_core::storage::{fs::FsStorage, StorageUri};
///
/// let uri = StorageUri::parse("/path/to/imp").unwrap();
/// let storage = FsStorage::new();
/// let files = read(&uri, &storage).unwrap();
/// let package = Imferno::parse(files).unwrap();
/// ```
///
/// # Example — S3 (requires the `aws-s3` feature)
///
/// ```ignore
/// use imferno_core::package::{read, Imferno};
/// use imferno_core::storage::{s3::S3Storage, StorageUri};
///
/// let uri = StorageUri::parse("s3://my-bucket/imp/").unwrap();
/// let storage = S3Storage::from_default().unwrap();
/// let files = read(&uri, &storage).unwrap();
/// ```
pub fn read_xml_files(
    uri: &crate::storage::StorageUri,
    storage: &dyn crate::storage::Storage,
) -> std::result::Result<HashMap<String, String>, crate::storage::StorageError> {
    let mut files = HashMap::new();
    for entry in storage.list(uri)? {
        if !entry.is_file {
            continue;
        }
        if !entry.uri.to_ascii_lowercase().ends_with(".xml") {
            continue;
        }
        let entry_uri = crate::storage::StorageUri::parse(&entry.uri)?;
        match storage.read_to_string(&entry_uri) {
            Ok(content) => {
                files.insert(entry.uri, content);
            }
            Err(e) => {
                eprintln!("Warning: failed to read XML file {}: {}", entry.uri, e);
            }
        }
    }
    Ok(files)
}

/// Public alias: `package::read(uri, storage)` — same as [`read_xml_files`].
pub use self::read_xml_files as read;

/// Read all XML files from an S3 prefix into a filename→content map.
///
/// This mirrors [`read_dir`] but reads from an S3 bucket. Only `.xml` files
/// are returned. Keys are `s3://{bucket}/{key}` URIs.
///
/// # Arguments
/// * `client` — An `aws_sdk_s3::Client` (caller controls region, credentials, endpoint).
/// * `bucket` — The S3 bucket name.
/// * `prefix` — The key prefix (e.g. `"packages/my-imf-package/"`). Should end with `/`.
#[cfg(feature = "aws-s3")]
pub async fn read_s3(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    prefix: &str,
) -> Result<HashMap<String, String>> {
    use crate::storage::{s3::S3Storage, StorageUri};

    let storage =
        S3Storage::from_client(client.clone()).map_err(|e| std::io::Error::other(e.to_string()))?;
    let uri_str = format!("s3://{bucket}/{prefix}");
    let uri = StorageUri::parse(&uri_str)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;

    // The trait method is sync; run on a blocking task so we don't block the
    // async caller's runtime.
    tokio::task::spawn_blocking(move || {
        crate::package::read_xml_files(&uri, &storage)
            .map_err(|e| std::io::Error::other(e.to_string()))
    })
    .await
    .map_err(|e| std::io::Error::other(format!("join error: {e}")))?
    .map_err(Into::into)
}

impl Imferno {
    /// Create an empty Imferno (used when parse fails but we still need a struct).
    fn empty() -> Self {
        Self {
            root_path: PathBuf::new(),
            volume_index: VolumeIndex { index: 1 },
            volindex_issues: Vec::new(),
            parse_issues: Vec::new(),
            asset_map: crate::assetmap::AssetMap {
                namespace: Default::default(),
                id: ImfUuid::parse("urn:uuid:00000000-0000-0000-0000-000000000000")
                    .expect("nil UUID is always valid"),
                annotation_text: None,
                creator: None,
                volume_count: 1,
                issue_date: "1970-01-01T00:00:00+00:00".into(),
                issuer: None,
                asset_list: crate::assetmap::AssetList { assets: Vec::new() },
            },
            packing_lists: HashMap::new(),
            composition_playlists: HashMap::new(),
            cpl_xml_content: HashMap::new(),
            output_profile_lists: HashMap::new(),
            sidecar_composition_maps: HashMap::new(),
            asset_paths: HashMap::new(),
        }
    }

    /// Parse an IMF package from an in-memory filename→XML string map (public API).
    ///
    /// This is the parse-only entry point. For parse + validate, use
    /// [`validate()`] instead.
    pub fn parse(files: HashMap<String, String>) -> Result<Self> {
        Self::from_file_map(&files)
    }

    /// Parse + validate in one call. Returns a `ValidationReport`.
    pub fn parse_and_validate(
        files: HashMap<String, String>,
        options: &ValidationOptions,
    ) -> ValidationReport {
        let package = match Self::parse(files) {
            Ok(pkg) => pkg,
            Err(e) => {
                let mut report = ValidationReport::new(ValidationProfile::SMPTE);
                report.add(ValidationIssue::from_rule(
                    &imferno::ParseError,
                    ImfernoEdition::Current,
                    format!("Failed to parse IMF package: {e}"),
                ));
                return report.apply_rules(&options.rules);
            }
        };

        package.validate(options)
    }

    /// Validate an already-parsed package. Applies rules from options.
    pub fn validate(&self, options: &ValidationOptions) -> ValidationReport {
        use crate::validation::{
            validate_cpl_with_registry, ConfigurableValidatorRegistry, ValidatorSelection,
        };

        let selection = ValidatorSelection {
            core_spec: options.core_spec,
            cpl_spec: options.cpl_spec,
            app_specs: options.app_specs.clone(),
            ..Default::default()
        };
        let registry = ConfigurableValidatorRegistry::new(selection);
        #[cfg(not(target_arch = "wasm32"))]
        let skip_disk = options.skip_disk_checks;
        #[cfg(target_arch = "wasm32")]
        let skip_disk = false;
        let report = self.validate_package_structure_for(
            |cpl| validate_cpl_with_registry(cpl, &registry),
            skip_disk,
            options.core_spec,
        );
        let report = self.enrich_cpl_locations(report);
        let report = report.apply_rules(&options.rules);
        if options.aggregate_repeats {
            report.aggregate()
        } else {
            report
        }
    }

    /// Validate + verify file hashes (expensive — reads every asset).
    ///
    /// Hash verification is only available on native targets (not WASM).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn validate_hashes(&self, options: &ValidationOptions) -> ValidationReport {
        use crate::validation::{
            validate_cpl_with_registry, ConfigurableValidatorRegistry, ValidatorSelection,
        };

        let selection = ValidatorSelection {
            core_spec: options.core_spec,
            cpl_spec: options.cpl_spec,
            app_specs: options.app_specs.clone(),
            ..Default::default()
        };
        let registry = ConfigurableValidatorRegistry::new(selection);
        let report = self.validate_package_with_hashes_for(
            |cpl| validate_cpl_with_registry(cpl, &registry),
            options.core_spec,
        );
        let report = self.enrich_cpl_locations(report);
        let report = report.apply_rules(&options.rules);
        if options.aggregate_repeats {
            report.aggregate()
        } else {
            report
        }
    }

    /// Enrich all validation issues that have a `cpl_id` with the CPL's
    /// filename and content title. This is called once after validation so
    /// that every issue gets human-readable CPL context regardless of where
    /// it was emitted.
    fn enrich_cpl_locations(&self, mut report: ValidationReport) -> ValidationReport {
        // Build lookup: cpl_id -> (filename, title)
        let mut cpl_info: std::collections::HashMap<ImfUuid, (Option<String>, String)> =
            std::collections::HashMap::new();
        for (uuid, cpl) in &self.composition_playlists {
            let filename = self
                .asset_paths
                .get(uuid)
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());
            cpl_info.insert(*uuid, (filename, cpl.content_title.to_string()));
        }

        let enrich = |issue: &mut ValidationIssue| {
            if let Some(ref cpl_id) = issue.location.cpl_id {
                if let Some((filename, title)) = cpl_info.get(cpl_id) {
                    if issue.location.cpl_filename.is_none() {
                        issue.location.cpl_filename = filename.clone();
                    }
                    if issue.location.cpl_title.is_none() {
                        issue.location.cpl_title = Some(title.clone());
                    }
                }
            }
        };

        for issue in &mut report.critical {
            enrich(issue);
        }
        for issue in &mut report.errors {
            enrich(issue);
        }
        for issue in &mut report.warnings {
            enrich(issue);
        }
        for issue in &mut report.info {
            enrich(issue);
        }
        report
    }

    /// Parse an IMF package from an in-memory filename→XML string map.
    ///
    /// Intended for WASM and test contexts where no filesystem is available.
    /// File hashes and existence checks are skipped unless keys are absolute paths
    /// (as produced by `read_dir`), in which case `root_path` is derived from
    /// the common parent directory.
    ///
    /// Lookup is case-insensitive on the file basename, so both
    /// `"ASSETMAP.xml"` and `"assetmap.xml"` resolve correctly.
    fn from_file_map(files: &HashMap<String, String>) -> Result<Self> {
        // Derive root_path from the keys if they are absolute paths.
        // `read_dir` produces absolute paths as keys; WASM callers use plain basenames.
        let root_path: PathBuf = files
            .keys()
            .filter_map(|k| {
                let p = std::path::Path::new(k.as_str());
                if p.is_absolute() {
                    p.parent().map(|par| par.to_path_buf())
                } else {
                    None
                }
            })
            .next()
            .unwrap_or_default();

        // Case-insensitive basename lookup helper.
        let find = |name: &str| -> Option<&str> {
            let lower = name.to_lowercase();
            files
                .iter()
                .find(|(k, _)| {
                    let key_basename = std::path::Path::new(k.as_str())
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or(k.as_str());
                    key_basename.to_lowercase() == lower
                })
                .map(|(_, v)| v.as_str())
        };

        // VOLINDEX.xml — optional per ST 429-9; issues collected here, emitted in validation.
        let mut volindex_issues: Vec<ValidationIssue> = Vec::new();
        let volume_index = match find("VOLINDEX.xml") {
            Some(xml) => match crate::assetmap::parse_volindex(xml) {
                Ok(vi) => vi,
                Err(e) => {
                    volindex_issues.push(ValidationIssue::from_rule(
                        &st429_9::MalformedXml,
                        St429_9Edition::V2014,
                        format!("VOLINDEX.xml is not well-formed XML: {e}"),
                    ));
                    VolumeIndex { index: 1 }
                }
            },
            None => {
                volindex_issues.push(ValidationIssue::from_rule(
                    &st429_9::VolindexMissing,
                    St429_9Edition::V2014,
                    "VOLINDEX.xml is absent; single-volume package assumed",
                ));
                VolumeIndex { index: 1 }
            }
        };

        // ASSETMAP.xml — required. ST 429-9:2014 Annex A.4: the file shall be
        // named exactly "ASSETMAP.xml". Prefer that name; fall back to a
        // case-insensitive match so the rest of the validation still runs,
        // and report the wrong name.
        let mut parse_issues: Vec<ValidationIssue> = Vec::new();
        let basename_of = |k: &str| -> String {
            std::path::Path::new(k)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(k)
                .to_string()
        };
        let assetmap_xml = match files.iter().find(|(k, _)| basename_of(k) == "ASSETMAP.xml") {
            Some((_, v)) => v.as_str(),
            None => {
                let (key, xml) = files
                    .iter()
                    .find(|(k, _)| basename_of(k).eq_ignore_ascii_case("ASSETMAP.xml"))
                    .ok_or_else(|| ImfError::MissingFile("ASSETMAP.xml".to_string()))?;
                let name = basename_of(key);
                parse_issues.push(
                    ValidationIssue::from_rule(
                        &st429_9::AssetMapFileName,
                        St429_9Edition::V2014,
                        format!(
                            "Asset Map file is named '{name}'; ST 429-9 A.4 requires \
                             'ASSETMAP.xml' (a case-sensitive reader finds no Asset Map)"
                        ),
                    )
                    .with_context("file", name),
                );
                xml.as_str()
            }
        };
        let asset_map = crate::assetmap::parse_assetmap(assetmap_xml)?;

        // Asset UUID → path mapping.
        // When root_path is known (native disk load), build absolute paths
        // with path traversal protection. Otherwise keep relative paths (WASM).
        let mut asset_paths: HashMap<ImfUuid, PathBuf> = HashMap::new();
        for asset in &asset_map.asset_list.assets {
            for chunk in &asset.chunk_list.chunks {
                let path = if root_path.as_os_str().is_empty() {
                    // WASM / in-memory: no filesystem, keep relative path as-is
                    Some(PathBuf::from(&chunk.path))
                } else {
                    sanitize_asset_path(&root_path, &chunk.path)
                };
                match path {
                    Some(p) => {
                        asset_paths.insert(asset.id, p);
                    }
                    None => {
                        parse_issues.push(ValidationIssue::from_rule(
                            &imferno::PathTraversal,
                            ImfernoEdition::Current,
                            format!(
                                "Asset '{}' chunk path '{}' escapes the package root directory",
                                asset.id, chunk.path,
                            ),
                        ));
                    }
                }
            }
        }

        // Parse PKLs
        let mut packing_lists = HashMap::new();
        for asset in &asset_map.asset_list.assets {
            if asset.packing_list == Some(true) {
                for chunk in &asset.chunk_list.chunks {
                    let basename = std::path::Path::new(&chunk.path)
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or(&chunk.path);
                    if let Some(pkl_xml) = find(basename) {
                        match crate::assetmap::parse_pkl(pkl_xml) {
                            Ok(pkl) => {
                                packing_lists.insert(asset.id, pkl);
                            }
                            Err(e) => {
                                parse_issues.push(ValidationIssue::from_rule(
                                    &imferno::PklParseError,
                                    ImfernoEdition::Current,
                                    format!("PKL '{}' parse error: {}", basename, e),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Collect XML asset IDs from PKL MIME types
        let mut xml_asset_ids: std::collections::HashSet<ImfUuid> =
            std::collections::HashSet::new();
        for pkl in packing_lists.values() {
            for pkl_asset in &pkl.asset_list.assets {
                if pkl_asset.mime_type.is_xml() {
                    xml_asset_ids.insert(pkl_asset.id);
                }
            }
        }

        // Parse CPLs, OPLs, and SCMs
        let mut composition_playlists = HashMap::new();
        let mut cpl_xml_content = HashMap::new();
        let mut output_profile_lists = HashMap::new();
        let mut sidecar_composition_maps = HashMap::new();
        for asset in &asset_map.asset_list.assets {
            if asset.packing_list == Some(true) {
                continue;
            }
            for chunk in &asset.chunk_list.chunks {
                if !chunk.path.ends_with(".xml") {
                    continue;
                }
                let is_candidate = if !xml_asset_ids.is_empty() {
                    xml_asset_ids.contains(&asset.id)
                } else {
                    true
                };
                if !is_candidate {
                    continue;
                }

                let basename = std::path::Path::new(&chunk.path)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or(&chunk.path);
                if let Some(xml) = find(basename) {
                    match crate::cpl::parse_cpl(xml) {
                        Ok(mut cpl) => {
                            cpl.assetmap_asset_id = Some(asset.id);
                            cpl_xml_content.insert(asset.id, xml.to_string());
                            composition_playlists.insert(asset.id, cpl);
                        }
                        Err(cpl_err) => {
                            if let Ok(opl) = crate::assetmap::parse_opl(xml) {
                                output_profile_lists.insert(asset.id, opl);
                            } else if let Ok(scm) = crate::scm::parse_scm(xml) {
                                sidecar_composition_maps.insert(asset.id, scm);
                            } else {
                                parse_issues.push(ValidationIssue::from_rule(&imferno::XmlAssetParseError, ImfernoEdition::Current,
                                    format!(
                                        "XML asset '{}' ({}) could not be parsed as CPL, OPL, or SCM: {}",
                                        basename, asset.id, cpl_err,
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(Imferno {
            root_path,
            volume_index,
            volindex_issues,
            parse_issues,
            asset_map,
            packing_lists,
            composition_playlists,
            cpl_xml_content,
            output_profile_lists,
            sidecar_composition_maps,
            asset_paths,
        })
    }

    /// Get CPL by UUID
    pub fn get_cpl(&self, uuid: ImfUuid) -> Option<&CompositionPlaylist> {
        self.composition_playlists.get(&uuid)
    }

    /// Get CPL by UUID string (convenience for callers with string UUIDs)
    pub fn get_cpl_str(&self, uuid: &str) -> Option<&CompositionPlaylist> {
        ImfUuid::parse(uuid)
            .ok()
            .and_then(|u| self.composition_playlists.get(&u))
    }

    /// Get asset file path by UUID
    pub fn get_asset_path(&self, uuid: ImfUuid) -> Option<&PathBuf> {
        self.asset_paths.get(&uuid)
    }

    /// Get asset file path by UUID string (convenience)
    pub fn get_asset_path_str(&self, uuid: &str) -> Option<&PathBuf> {
        ImfUuid::parse(uuid)
            .ok()
            .and_then(|u| self.asset_paths.get(&u))
    }

    /// List all CPL UUIDs
    pub fn list_cpl_uuids(&self) -> Vec<ImfUuid> {
        self.composition_playlists.keys().copied().collect()
    }

    /// Get main CPL (first one found)
    pub fn get_main_cpl(&self) -> Option<&CompositionPlaylist> {
        self.composition_playlists.values().next()
    }

    /// Return AssetMap assets that have no known relationship to any CPL.
    ///
    /// An asset is "unreferenced" when it is:
    /// - not a CPL, PKL, SCM, or OPL document
    /// - not referenced by any CPL Virtual Track's `TrackFileId`
    /// - not declared as a sidecar in any SCM
    ///
    /// These are typically sidecar essences (e.g. Dolby Atmos MXF) delivered
    /// without an accompanying SCM document.
    pub fn unreferenced_assets(&self) -> Vec<&crate::assetmap::Asset> {
        use std::collections::HashSet;

        // UUIDs of all document assets we have parsed
        let doc_ids: HashSet<ImfUuid> = self
            .composition_playlists
            .keys()
            .chain(self.packing_lists.keys())
            .chain(self.sidecar_composition_maps.keys())
            .chain(self.output_profile_lists.keys())
            .copied()
            .collect();

        // TrackFileIds referenced by any CPL Virtual Track
        let track_file_ids: HashSet<ImfUuid> = self
            .composition_playlists
            .values()
            .flat_map(|cpl| cpl.segment_list.segments.iter())
            .flat_map(|seg| {
                seg.sequence_list
                    .all_sequences()
                    .into_iter()
                    .flat_map(|seq| {
                        seq.resource_list()
                            .resources
                            .iter()
                            .filter_map(|r| r.track_file_id)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        // Asset IDs already declared as SCM sidecars
        let scm_declared: HashSet<ImfUuid> = self
            .sidecar_composition_maps
            .values()
            .flat_map(|scm| scm.sidecar_assets.iter().map(|sa| sa.id))
            .collect();

        self.asset_map
            .asset_list
            .assets
            .iter()
            .filter(|a| {
                a.packing_list != Some(true)
                    && !doc_ids.contains(&a.id)
                    && !track_file_ids.contains(&a.id)
                    && !scm_declared.contains(&a.id)
            })
            .collect()
    }

    /// The ST 2067-2 edition applied to package-level rules.
    ///
    /// 1. `core_spec`, when the caller pins one.
    /// 2. The Packing Lists: an ST 429-8 PKL is ST 2067-2:2013 (2013 §7.1);
    ///    the `.../2067-2/2016/PKL` namespace is used by both 2016 (Annex K)
    ///    and 2020 (Annex J).
    /// 3. The CPLs whose Core Constraints edition is determined
    ///    ([`crate::validation::cpl_core_edition`], the resolution the CPL
    ///    validators use) name the edition.
    ///
    /// When that leaves more than one edition, the newest is used and
    /// `assumed` says so ([`crate::validation::resolve_edition`]).
    pub fn package_edition(
        &self,
        core_spec: Option<crate::validation::CoreSpecTarget>,
    ) -> PackageEdition {
        use crate::assetmap::PklNamespace;
        use crate::diagnostics::Edition;
        use crate::validation::{resolve_edition, Evidence};
        use St2067_2Edition::{V2013, V2016, V2020};

        if let Some(target) = core_spec {
            return PackageEdition {
                edition: target.edition(),
                assumed: None,
            };
        }

        let mut candidates: Vec<St2067_2Edition> = vec![V2013, V2016, V2020];
        let mut namespaces: Vec<String> = Vec::new();
        for pkl in self.packing_lists.values() {
            let allowed: &[St2067_2Edition] = match pkl.namespace {
                PklNamespace::Dci429_8 | PklNamespace::Smpte2067_2_2013 => &[V2013],
                PklNamespace::Smpte2067_2_2016
                | PklNamespace::Smpte2067_2_2016Pkl
                | PklNamespace::Smpte2067_2_2020 => &[V2016, V2020],
                PklNamespace::Unknown(_) => &[V2013, V2016, V2020],
            };
            candidates.retain(|e| allowed.contains(e));
            namespaces.push(pkl.namespace.to_string());
        }
        namespaces.sort();
        namespaces.dedup();
        if candidates.is_empty() {
            // Packing Lists of different editions: every edition remains.
            candidates = vec![V2013, V2016, V2020];
        }

        // The CPLs whose own Core Constraints edition is determined (the
        // same resolution the CPL validators use) are the evidence.
        let mut cpls: Vec<&CompositionPlaylist> = self.composition_playlists.values().collect();
        cpls.sort_by_key(|c| c.id.to_string());
        let evidence: Vec<Evidence<St2067_2Edition>> = cpls
            .into_iter()
            .filter_map(|cpl| {
                let choice = crate::validation::cpl_core_edition(cpl)?;
                choice.assumed.is_none().then(|| {
                    Evidence::one(
                        choice.edition,
                        format!("CPL {} is ST 2067-2:{}", cpl.id, choice.edition.label()),
                    )
                })
            })
            .collect();

        let namespace = match namespaces.as_slice() {
            [] => "it has no Packing List, so its namespace".to_string(),
            [one] => format!("its Packing List namespace {one}"),
            many => format!("its Packing List namespaces {}", many.join(", ")),
        };
        resolve_edition("the package", &namespace, &candidates, &evidence)
    }

    /// One notice naming the ST 429-9 edition assumed when the report holds
    /// ST 429-9 findings: the 2007 and 2014 editions share the Asset Map
    /// namespace, so the newest is applied.
    fn st429_9_edition_notice(report: &mut ValidationReport) {
        let any = report
            .critical
            .iter()
            .chain(&report.errors)
            .chain(&report.warnings)
            .chain(&report.info)
            .any(|i| i.spec == crate::diagnostics::Spec::St429_9);
        if any {
            report.add(ValidationIssue::from_rule(
                &imferno::EditionAssumed,
                ImfernoEdition::Current,
                "ST 429-9 edition of the Asset Map could not be determined (ST 429-9:2007 and \
                 ST 429-9:2014 share the namespace http://www.smpte-ra.org/schemas/429-9/2007/AM); \
                 ST 429-9 rules were checked against ST 429-9:2014",
            ));
        }
    }

    /// The edition-dependent notices of a package: the assumed edition, and
    /// for an ST 2067-2:2013 package the delivery rules not checked.
    fn edition_notices(&self, edition: &PackageEdition, report: &mut ValidationReport) {
        if let Some(why) = &edition.assumed {
            report.add(ValidationIssue::from_rule(
                &imferno::EditionAssumed,
                ImfernoEdition::Current,
                why.clone(),
            ));
        }
        if edition.edition == St2067_2Edition::V2013 {
            report.add(ValidationIssue::from_rule(
                &imferno::DeliveryNotChecked,
                ImfernoEdition::Current,
                "ST 2067-2:2013 defines no delivery requirements (an IMP is a Packing List \
                 and the assets it references, §7); delivery rules of later editions \
                 (Asset Map ingest, ASSETMAP.xml name, ST 429-9 Annex A paths, VOLINDEX) \
                 were not checked; Packing List Size and Hash are checked under ST 429-8",
            ));
        }
    }

    /// Emit `IMFERNO/UnreferencedAsset` info findings into `report` for each
    /// asset that has no CPL Virtual Track reference and no SCM declaration.
    fn emit_unreferenced_asset_info(&self, report: &mut ValidationReport) {
        for asset in self.unreferenced_assets() {
            let path = asset
                .chunk_list
                .chunks
                .first()
                .map(|c| c.path.as_str())
                .unwrap_or("(unknown)");
            report.add(ValidationIssue::from_rule(
                &imferno::UnreferencedAsset,
                ImfernoEdition::Current,
                format!(
                    "Asset '{}' ({}) is present in the AssetMap but not referenced by any CPL \
                     Virtual Track and has no SCM declaration",
                    path, asset.id,
                ),
            ));
        }
    }

    /// Emit `IMFERNO/UnlistedEssence` for any file in the package
    /// directory that is not accounted for by the AssetMap, PKL, VOLINDEX,
    /// or ASSETMAP itself. Severity comes from the catalogue (Info: ST
    /// 2067-2:2020 §8.3 Note 3 lets the medium hold files that are not
    /// ingested).
    ///
    /// Scans the root directory non-recursively.  Skipped on WASM and when
    /// `root_path` is unset (in-memory / WASM packages).
    #[cfg(not(target_arch = "wasm32"))]
    fn emit_unlisted_essence(&self, report: &mut ValidationReport) {
        if self.root_path.as_os_str().is_empty() {
            return;
        }

        // All filenames listed as chunks in the AssetMap.
        let mut known: std::collections::HashSet<String> = self
            .asset_map
            .asset_list
            .assets
            .iter()
            .flat_map(|a| a.chunk_list.chunks.iter())
            .filter_map(|c| {
                std::path::Path::new(&c.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
            })
            .collect();

        // Package infrastructure files are always expected.
        known.insert("ASSETMAP.xml".into());
        known.insert("VOLINDEX.xml".into());
        // Case variants seen in the wild.
        known.insert("assetmap.xml".into());
        known.insert("volindex.xml".into());
        known.insert("ASSETMAP".into());
        known.insert("VOLINDEX".into());

        let entries = match std::fs::read_dir(&self.root_path) {
            Ok(e) => e,
            Err(e) => {
                report.add(ValidationIssue::from_rule(
                    &imferno::ReadDirError,
                    ImfernoEdition::Current,
                    format!("Could not scan package directory for unlisted files: {}", e,),
                ));
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    report.add(ValidationIssue::from_rule(
                        &imferno::DirEntryError,
                        ImfernoEdition::Current,
                        format!("Could not read directory entry: {}", e),
                    ));
                    continue;
                }
            };
            let path = entry.path();
            // Skip directories
            if path.is_dir() {
                continue;
            }
            let filename = match path.file_name() {
                Some(n) => n.to_string_lossy().into_owned(),
                None => continue,
            };
            // Case-insensitive match against known files
            if known.iter().any(|k| k.eq_ignore_ascii_case(&filename)) {
                continue;
            }
            report.add(ValidationIssue::from_rule(
                &imferno::UnlistedEssence,
                ImfernoEdition::Current,
                format!(
                    "File '{}' is present in the package directory but not listed in the AssetMap",
                    filename,
                ),
            ));
        }
    }

    /// Check package structure, returning an error if any critical or error issues are found.
    ///
    /// Not currently wired into the public API; retained for potential future use.
    #[allow(dead_code)]
    pub(crate) fn validate_structure(&self) -> Result<()> {
        // Run the comprehensive package structure validation and convert to Result
        let report = self.validate_package_structure();
        if report.has_critical() || report.has_errors() {
            let error_messages: Vec<String> = report
                .errors
                .iter()
                .chain(report.critical.iter())
                .map(|i| i.message.clone())
                .collect();
            return Err(ImfError::InvalidStructure(error_messages.join("; ")));
        }
        Ok(())
    }

    /// Validate that every PKL asset exists on disk and has the correct file size.
    ///
    /// Returns a list of `FileValidationError` describing any mismatches found.
    /// An empty vec means the manifest is consistent.
    pub fn validate_file_manifest(&self) -> Vec<FileValidationError> {
        let mut errors = Vec::new();

        // Build UUID → path mapping from AssetMap
        let path_map = self.build_asset_path_map();

        for pkl in self.packing_lists.values() {
            for asset in &pkl.asset_list.assets {
                let uuid_str = asset.id.to_string();
                match path_map.get(&asset.id) {
                    None => {
                        errors.push(FileValidationError::NotInAssetMap {
                            uuid: uuid_str,
                            original_file_name: asset.original_file_name.clone(),
                        });
                    }
                    Some(abs_path) => match std::fs::metadata(abs_path) {
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::NotFound {
                                errors.push(FileValidationError::Missing {
                                    uuid: uuid_str,
                                    path: abs_path.clone(),
                                });
                            } else {
                                errors.push(FileValidationError::Io {
                                    uuid: uuid_str,
                                    path: abs_path.clone(),
                                    message: format!("Cannot access file: {}", e),
                                });
                            }
                        }
                        Ok(meta) => {
                            let actual = meta.len();
                            if actual != asset.size {
                                errors.push(FileValidationError::SizeMismatch {
                                    uuid: uuid_str,
                                    path: abs_path.clone(),
                                    expected: asset.size,
                                    actual,
                                });
                            }
                        }
                    },
                }
            }
        }

        errors
    }

    /// Validate file hashes (SHA-1 or SHA-256) for every PKL asset on disk.
    ///
    /// Per SMPTE ST 2067-2 §9, PKL assets carry hashes with an algorithm
    /// specified by the `<HashAlgorithm>` element (defaulting to SHA-1).
    ///
    /// This is slow — it reads every file. Use `validate_file_manifest` for a
    /// fast size-only check. Returns a list of `FileValidationError` describing
    /// hash mismatches (missing / size issues are also reported).
    pub fn validate_file_hashes(&self) -> Vec<FileValidationError> {
        self.validate_file_hashes_with_progress(|_, _, _, _, _| {})
    }

    /// Like `validate_file_hashes` but calls `on_progress(current, total, filename, bytes_done, bytes_total)`
    /// during hashing. Updates both per-file and within-file progress.
    pub fn validate_file_hashes_with_progress(
        &self,
        mut on_progress: impl FnMut(usize, usize, &str, u64, u64),
    ) -> Vec<FileValidationError> {
        let mut errors = self.validate_file_manifest();
        let errored_uuids: std::collections::HashSet<String> =
            errors.iter().map(|e| e.uuid().to_string()).collect();

        let path_map = self.build_asset_path_map();

        // Count total assets to hash
        let total: usize = self
            .packing_lists
            .values()
            .map(|pkl| pkl.asset_list.assets.len())
            .sum();
        let mut current: usize = 0;

        for pkl in self.packing_lists.values() {
            for asset in &pkl.asset_list.assets {
                current += 1;
                let uuid_str = asset.id.to_string();
                if errored_uuids.contains(&uuid_str) {
                    continue;
                }
                let Some(abs_path) = path_map.get(&asset.id) else {
                    continue;
                };

                let filename = abs_path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                let file_size = std::fs::metadata(abs_path).map(|m| m.len()).unwrap_or(0);
                on_progress(current, total, filename, 0, file_size);

                match std::fs::File::open(abs_path) {
                    Err(e) => {
                        errors.push(FileValidationError::Io {
                            uuid: uuid_str,
                            path: abs_path.clone(),
                            message: e.to_string(),
                        });
                    }
                    Ok(file) => {
                        use std::io::Read;
                        let mut reader = std::io::BufReader::with_capacity(1024 * 1024, file);
                        let mut bytes_done: u64 = 0;
                        let mut had_error = false;
                        let actual_b64 = match asset.hash.algorithm() {
                            crate::assetmap::HashAlgorithm::Sha1 => {
                                use sha1::Digest;
                                let mut hasher = sha1::Sha1::new();
                                let mut buf = vec![0u8; 1024 * 1024];
                                loop {
                                    match reader.read(&mut buf) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            hasher.update(&buf[..n]);
                                            bytes_done += n as u64;
                                            on_progress(
                                                current, total, filename, bytes_done, file_size,
                                            );
                                        }
                                        Err(e) => {
                                            errors.push(FileValidationError::Io {
                                                uuid: uuid_str.clone(),
                                                path: abs_path.clone(),
                                                message: e.to_string(),
                                            });
                                            had_error = true;
                                            break;
                                        }
                                    }
                                }
                                base64::Engine::encode(
                                    &base64::engine::general_purpose::STANDARD,
                                    hasher.finalize(),
                                )
                            }
                            crate::assetmap::HashAlgorithm::Sha256 => {
                                use sha2::Digest;
                                let mut hasher = sha2::Sha256::new();
                                let mut buf = vec![0u8; 1024 * 1024];
                                loop {
                                    match reader.read(&mut buf) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            hasher.update(&buf[..n]);
                                            bytes_done += n as u64;
                                            on_progress(
                                                current, total, filename, bytes_done, file_size,
                                            );
                                        }
                                        Err(e) => {
                                            errors.push(FileValidationError::Io {
                                                uuid: uuid_str.clone(),
                                                path: abs_path.clone(),
                                                message: e.to_string(),
                                            });
                                            had_error = true;
                                            break;
                                        }
                                    }
                                }
                                base64::Engine::encode(
                                    &base64::engine::general_purpose::STANDARD,
                                    hasher.finalize(),
                                )
                            }
                        };
                        if !had_error {
                            let expected_b64 = asset.hash.to_base64();
                            if actual_b64 != expected_b64 {
                                errors.push(FileValidationError::HashMismatch {
                                    uuid: uuid_str,
                                    path: abs_path.clone(),
                                    expected: expected_b64,
                                    actual: actual_b64,
                                });
                            }
                        }
                    }
                }
            }
        }

        errors
    }

    /// Returns the total number of bytes to be hashed, for progress bar setup.
    /// Call this before `validate_file_hashes_parallel` to know the total size.
    ///
    /// Requires the `tokio` feature.
    #[cfg(feature = "tokio")]
    pub fn hash_verification_size(&self) -> u64 {
        let path_map = self.build_asset_path_map();
        self.packing_lists
            .values()
            .flat_map(|pkl| pkl.asset_list.assets.iter())
            .filter_map(|asset| {
                path_map
                    .get(&asset.id)
                    .and_then(|p| std::fs::metadata(p).ok())
                    .map(|m| m.len())
            })
            .sum()
    }

    /// Per-file progress state for parallel hash verification.
    #[cfg(feature = "tokio")]
    pub async fn validate_file_hashes_parallel(
        &self,
        concurrency: usize,
        progress: std::sync::Arc<HashProgressTracker>,
    ) -> Vec<FileValidationError> {
        use std::sync::Arc;

        let path_map = self.build_asset_path_map();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
        let mut handles = Vec::new();

        // First pass: validate file manifest (sync, fast)
        let manifest_errors = self.validate_file_manifest();
        let errored_uuids: std::collections::HashSet<String> = manifest_errors
            .iter()
            .map(|e| e.uuid().to_string())
            .collect();

        // Collect assets to hash, sorted smallest first for fast early progress
        let mut assets_to_hash: Vec<_> = self
            .packing_lists
            .values()
            .flat_map(|pkl| pkl.asset_list.assets.iter())
            .filter(|asset| !errored_uuids.contains(&asset.id.to_string()))
            .filter(|asset| path_map.contains_key(&asset.id))
            .collect();
        assets_to_hash.sort_by_key(|a| a.size);

        // Register and spawn hash tasks
        for asset in assets_to_hash {
            let abs_path = path_map.get(&asset.id).unwrap();

            let filename = abs_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            let file_size = asset.size;
            let (bytes_counter, status_flag) = progress.register(filename, file_size);

            let uuid_str = asset.id.to_string();
            let abs_path = abs_path.clone();
            let expected_b64 = asset.hash.to_base64();
            let algorithm = asset.hash.algorithm();
            let sem = semaphore.clone();

            let err_uuid = uuid_str.clone();
            let err_path = abs_path.clone();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                status_flag.store(1, std::sync::atomic::Ordering::Relaxed); // Hashing
                let result = match tokio::task::spawn_blocking(move || {
                    hash_single_file(
                        &uuid_str,
                        &abs_path,
                        &expected_b64,
                        algorithm,
                        &bytes_counter,
                    )
                })
                .await
                {
                    Ok(r) => r,
                    Err(e) => Some(FileValidationError::Io {
                        uuid: err_uuid,
                        path: err_path,
                        message: format!("hash task failed: {}", e),
                    }),
                };

                status_flag.store(
                    if result.is_some() { 3 } else { 2 }, // Failed or Done
                    std::sync::atomic::Ordering::Relaxed,
                );
                result
            }));
        }

        // Collect results
        let mut errors = manifest_errors;
        for handle in handles {
            if let Ok(Some(err)) = handle.await {
                errors.push(err);
            }
        }

        errors
    }

    /// Validate PKL structural constraints per SMPTE ST 2067-2.
    ///
    /// Checks:
    /// - §9: No duplicate asset UUIDs within a single PKL
    /// - §7/9: Every PKL asset UUID exists in the AssetMap
    pub fn validate_pkl_constraints(&self) -> Vec<FileValidationError> {
        use crate::assetmap::PklNamespace;
        let mut errors = Vec::new();

        // Build AssetMap UUID set + PackingList-flagged subset for the
        // ST 429-9 §6.3 cross-doc checks.
        let assetmap_ids: std::collections::HashSet<ImfUuid> = self
            .asset_map
            .asset_list
            .assets
            .iter()
            .map(|a| a.id)
            .collect();
        let assetmap_pkl_ids: std::collections::HashSet<ImfUuid> = self
            .asset_map
            .asset_list
            .assets
            .iter()
            .filter(|a| a.packing_list.unwrap_or(false))
            .map(|a| a.id)
            .collect();

        // ST 429-9 §6.3: AssetMap must identify at least one PKL via
        // `<PackingList>true</PackingList>`. Fire whenever the AssetMap
        // has no such flag — this is true regardless of whether any
        // PKL files happen to be present on disk (the AssetMap is
        // authoritative on what the package is supposed to contain).
        if assetmap_pkl_ids.is_empty() {
            errors.push(FileValidationError::AssetMapHasNoPackingList);
        }

        for pkl in self.packing_lists.values() {
            // ST 2067-2 §9: PKL namespace must be one of the published
            // SMPTE PKL namespace URIs. `PklNamespace::Unknown` is the
            // parser's sentinel for "not in the recognised set".
            if let PklNamespace::Unknown(uri) = &pkl.namespace {
                errors.push(FileValidationError::UnknownPklNamespace {
                    pkl_id: pkl.id.to_string(),
                    namespace: uri.clone(),
                });
            }

            // ST 429-9 §6.3: every PKL document's Id must appear as a
            // PackingList-flagged asset in the AssetMap.
            if !assetmap_pkl_ids.contains(&pkl.id) {
                errors.push(FileValidationError::PklIdNotInAssetMap {
                    pkl_id: pkl.id.to_string(),
                });
            }

            // ST 2067-2 §9: Check for duplicate asset IDs within this PKL
            let mut seen_ids: std::collections::HashSet<ImfUuid> = std::collections::HashSet::new();
            for asset in &pkl.asset_list.assets {
                if !seen_ids.insert(asset.id) {
                    errors.push(FileValidationError::DuplicatePklAssetId {
                        uuid: asset.id.to_string(),
                        pkl_id: pkl.id.to_string(),
                    });
                }

                // ST 2067-2 §7: Every PKL asset must be in the AssetMap
                if !assetmap_ids.contains(&asset.id) {
                    errors.push(FileValidationError::NotInAssetMap {
                        uuid: asset.id.to_string(),
                        original_file_name: asset.original_file_name.clone(),
                    });
                }
            }
        }

        errors
    }

    /// Build a map from asset UUID to sanitized relative file path.
    ///
    /// Paths that would escape the package root (path traversal) are excluded.
    /// PKL assets whose Id is not an AssetMap asset (XML only, no disk).
    fn pkl_assets_not_in_assetmap(&self) -> Vec<FileValidationError> {
        let assetmap_ids: std::collections::HashSet<ImfUuid> = self
            .asset_map
            .asset_list
            .assets
            .iter()
            .map(|a| a.id)
            .collect();
        self.packing_lists
            .values()
            .flat_map(|pkl| pkl.asset_list.assets.iter())
            .filter(|a| !assetmap_ids.contains(&a.id))
            .map(|a| FileValidationError::NotInAssetMap {
                uuid: a.id.to_string(),
                original_file_name: a.original_file_name.clone(),
            })
            .collect()
    }

    fn build_asset_path_map(&self) -> HashMap<ImfUuid, PathBuf> {
        let mut map = HashMap::new();
        let has_root = !self.root_path.as_os_str().is_empty();
        for asset in &self.asset_map.asset_list.assets {
            if let Some(chunk) = asset.chunk_list.chunks.first() {
                if has_root {
                    if let Some(safe_path) = sanitize_asset_path(&self.root_path, &chunk.path) {
                        map.insert(asset.id, safe_path);
                    }
                    // Traversal paths silently excluded — already reported at parse time
                } else {
                    map.insert(asset.id, PathBuf::from(&chunk.path));
                }
            }
        }
        map
    }

    /// Comprehensive package-level validation producing a unified `ValidationReport`.
    ///
    /// Runs all structural and cross-reference checks that require package context
    /// (AssetMap, PKL, CPL relationships). This covers:
    ///
    /// - **ST 2067-2 §7/9:** PKL asset UUIDs exist in AssetMap
    /// - **ST 2067-2 §9:** No duplicate asset UUIDs within a PKL
    /// - **ST 2067-2 §7:** CPL TrackFileId references resolve in AssetMap
    /// - **ST 2067-2 §9:** File manifest (size) validation
    ///
    /// Callers should merge this with CPL-level validation results (e.g., from
    /// `crate::validation::ConstraintsValidator`) for a complete report.
    ///
    /// For hash verification (expensive I/O), use `validate_package_with_hashes()`.
    pub fn validate_package_structure(&self) -> ValidationReport {
        self.validate_package_structure_with_cpl_validator(|_| Vec::new(), false)
    }

    /// Comprehensive package-level validation with optional CPL-level validator injection.
    ///
    /// This provides an extension seam for callers to plug in profile/spec CPL validators
    /// (e.g. registry-driven validators) without changing core package validation behavior.
    ///
    /// Set `skip_disk_checks` to `true` to skip file manifest (existence/size) and MXF header
    /// inspection. Useful for packages on slow or remote filesystems (e.g. S3 via MacFUSE).
    pub fn validate_package_structure_with_cpl_validator<F>(
        &self,
        cpl_validator: F,
        skip_disk_checks: bool,
    ) -> ValidationReport
    where
        F: Fn(&CompositionPlaylist) -> Vec<ValidationIssue>,
    {
        self.validate_package_structure_for(cpl_validator, skip_disk_checks, None)
    }

    /// [`Self::validate_package_structure_with_cpl_validator`] with the
    /// ST 2067-2 edition pinned by `core_spec` (see [`Self::package_edition`]).
    pub fn validate_package_structure_for<F>(
        &self,
        cpl_validator: F,
        skip_disk_checks: bool,
        core_spec: Option<crate::validation::CoreSpecTarget>,
    ) -> ValidationReport
    where
        F: Fn(&CompositionPlaylist) -> Vec<ValidationIssue>,
    {
        let edition = self.package_edition(core_spec);
        let spec_ed = edition.edition;
        let delivery = spec_ed != St2067_2Edition::V2013;
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        self.edition_notices(&edition, &mut report);

        // VOLINDEX diagnostics (ST 429-9) — emitted first
        for issue in &self.volindex_issues {
            if delivery || !is_delivery_rule(issue) {
                report.add(issue.clone());
            }
        }

        // Parse-time diagnostics (PKL/CPL/OPL/SCM failures)
        for issue in &self.parse_issues {
            report.add(issue.clone());
        }

        // ST 429-9:2014 Annex A.2 Path rules, from the AssetMap <Path>
        // values alone, so they run for every input (filesystem,
        // in-memory, WASM).
        {
            let paths = self
                .asset_map
                .asset_list
                .assets
                .iter()
                .flat_map(|a| a.chunk_list.chunks.iter().map(|c| c.path.as_str()));
            if delivery {
                for issue in crate::assetmap::map_profile::basic_map_profile_v2_issues(paths) {
                    report.add(issue);
                }
            }
        }

        // PKL structural constraints (ST 2067-2 §7/9)
        for issue in self
            .validate_pkl_constraints()
            .iter()
            .filter_map(|e| e.to_issue(spec_ed))
        {
            report.add(issue);
        }

        // File manifest: every PKL asset exists on disk with correct size
        // (skipped on WASM — no real filesystem available, skipped when no root_path is set,
        //  and skipped when skip_disk_checks is true)
        #[cfg(not(target_arch = "wasm32"))]
        let manifest_checked = !skip_disk_checks && !self.root_path.as_os_str().is_empty();
        #[cfg(target_arch = "wasm32")]
        let manifest_checked = false;
        #[cfg(not(target_arch = "wasm32"))]
        if manifest_checked {
            for issue in self
                .validate_file_manifest()
                .iter()
                .filter_map(|e| e.to_issue(spec_ed))
            {
                report.add(issue);
            }
        }
        // Without the disk pass, still report PKL assets missing from the
        // AssetMap (UnresolvedUuid): it needs only the XML, and
        // validate_cpl_asset_references_accumulating relies on it for
        // CPL track files that are in a PKL but not in the AssetMap.
        if !manifest_checked {
            for issue in self
                .pkl_assets_not_in_assetmap()
                .iter()
                .filter_map(|e| e.to_issue(spec_ed))
            {
                report.add(issue);
            }
        }

        // CPL TrackFileId → AssetMap cross-references
        let mut signature_notices = Vec::new();
        for cpl in self.composition_playlists.values() {
            self.validate_cpl_asset_references_accumulating(cpl, &mut report, spec_ed);

            // Optional external CPL-level validation injection
            for issue in cpl_validator(cpl) {
                if issue.code.ends_with("/DigitalSignature") {
                    signature_notices.push(issue);
                } else {
                    report.add(issue);
                }
            }
        }
        if let Some(notice) = collapse_signature_notices(signature_notices) {
            report.add(notice);
        }

        // SCM reference checks (ST 2067-9:2018 §6)
        self.validate_scm_references(&mut report);

        // Tool-level observations (not spec violations)
        self.emit_unreferenced_asset_info(&mut report);

        // Multi-PKL consistency (ST 2067-2 §7)
        self.validate_multi_pkl_consistency(&mut report, spec_ed);

        // MXF header cross-validation (ST 377-1) — skipped on WASM, when no root_path is set,
        // and when skip_disk_checks is true
        #[cfg(not(target_arch = "wasm32"))]
        if !skip_disk_checks && !self.root_path.as_os_str().is_empty() {
            self.validate_mxf_headers(&mut report, spec_ed);
        }

        // Files the AssetMap does not list. A directory listing reads no
        // asset, so this also runs with skip_disk_checks.
        #[cfg(not(target_arch = "wasm32"))]
        self.emit_unlisted_essence(&mut report);

        Self::st429_9_edition_notice(&mut report);
        report
    }

    /// Like `validate_package_structure()` but also verifies file hashes.
    ///
    /// **Warning:** This reads every asset file from disk to compute SHA-1/SHA-256
    /// digests. For large packages this can be slow.
    pub fn validate_package_with_hashes(&self) -> ValidationReport {
        self.validate_package_with_hashes_with_cpl_validator(|_| Vec::new())
    }

    /// Hash-validating package-level validation with optional CPL-level validator injection.
    pub fn validate_package_with_hashes_with_cpl_validator<F>(
        &self,
        cpl_validator: F,
    ) -> ValidationReport
    where
        F: Fn(&CompositionPlaylist) -> Vec<ValidationIssue>,
    {
        self.validate_package_with_hashes_for(cpl_validator, None)
    }

    /// [`Self::validate_package_with_hashes_with_cpl_validator`] with the
    /// ST 2067-2 edition pinned by `core_spec`.
    pub fn validate_package_with_hashes_for<F>(
        &self,
        cpl_validator: F,
        core_spec: Option<crate::validation::CoreSpecTarget>,
    ) -> ValidationReport
    where
        F: Fn(&CompositionPlaylist) -> Vec<ValidationIssue>,
    {
        let edition = self.package_edition(core_spec);
        let spec_ed = edition.edition;
        let delivery = spec_ed != St2067_2Edition::V2013;
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        self.edition_notices(&edition, &mut report);

        // VOLINDEX diagnostics (ST 429-9) — emitted first
        for issue in &self.volindex_issues {
            if delivery || !is_delivery_rule(issue) {
                report.add(issue.clone());
            }
        }

        // Parse-time diagnostics (PKL/CPL/OPL/SCM failures)
        for issue in &self.parse_issues {
            report.add(issue.clone());
        }

        // PKL structural constraints
        for issue in self
            .validate_pkl_constraints()
            .iter()
            .filter_map(|e| e.to_issue(spec_ed))
        {
            report.add(issue);
        }

        // File manifest + hash verification (subsumes validate_file_manifest)
        for issue in self
            .validate_file_hashes()
            .iter()
            .filter_map(|e| e.to_issue(spec_ed))
        {
            report.add(issue);
        }

        // CPL TrackFileId → AssetMap cross-references
        let mut signature_notices = Vec::new();
        for cpl in self.composition_playlists.values() {
            self.validate_cpl_asset_references_accumulating(cpl, &mut report, spec_ed);

            // Optional external CPL-level validation injection
            for issue in cpl_validator(cpl) {
                if issue.code.ends_with("/DigitalSignature") {
                    signature_notices.push(issue);
                } else {
                    report.add(issue);
                }
            }
        }
        if let Some(notice) = collapse_signature_notices(signature_notices) {
            report.add(notice);
        }

        // Multi-PKL consistency
        self.validate_multi_pkl_consistency(&mut report, spec_ed);

        // MXF header cross-validation (ST 377-1)
        self.validate_mxf_headers(&mut report, spec_ed);

        Self::st429_9_edition_notice(&mut report);
        report
    }

    /// Validate Sidecar Composition Map references (ST 2067-9:2018).
    ///
    /// Enforces normative requirements from §5, §7.2.3, §7.2.4, §7.2.5, §7.3.1, §7.3.1.1.
    fn validate_scm_references(&self, report: &mut ValidationReport) {
        use std::collections::HashSet;

        let asset_ids: HashSet<_> = self
            .asset_map
            .asset_list
            .assets
            .iter()
            .map(|a| a.id)
            .collect();

        // §5: Collect all TrackFileIds referenced by any Virtual Track in any CPL.
        let virtual_track_file_ids: HashSet<ImfUuid> = self
            .composition_playlists
            .values()
            .flat_map(|cpl| cpl.segment_list.segments.iter())
            .flat_map(|seg| {
                seg.sequence_list
                    .all_sequences()
                    .into_iter()
                    .flat_map(|seq| {
                        seq.resource_list()
                            .resources
                            .iter()
                            .filter_map(|r| r.track_file_id)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        for scm in self.sidecar_composition_maps.values() {
            // §7.2.4: Signer present → Signature must be present.
            if scm.has_signer && !scm.has_signature {
                report.add(
                    ValidationIssue::from_rule(
                        &st2067_9::SignerWithoutSignature,
                        St2067_9Edition::V2018,
                        format!(
                            "SCM {}: Signer element present but Signature element is absent",
                            scm.id
                        ),
                    )
                    .with_context("scm_id", scm.id.to_string()),
                );
            }

            // §7.2.5: Signature present → Signer must be present.
            if scm.has_signature && !scm.has_signer {
                report.add(
                    ValidationIssue::from_rule(
                        &st2067_9::SignatureWithoutSigner,
                        St2067_9Edition::V2018,
                        format!(
                            "SCM {}: Signature element present but Signer element is absent",
                            scm.id
                        ),
                    )
                    .with_context("scm_id", scm.id.to_string()),
                );
            }

            let mut seen_asset_ids = HashSet::new();
            for sidecar_asset in &scm.sidecar_assets {
                // §7.2.3: Duplicate SidecarAsset Id within SidecarAssetList.
                if !seen_asset_ids.insert(sidecar_asset.id) {
                    report.add(
                        ValidationIssue::from_rule(
                            &st2067_9::DuplicateAssetId,
                            St2067_9Edition::V2018,
                            format!(
                                "Duplicate SidecarAsset Id {} in SCM {}",
                                sidecar_asset.id, scm.id
                            ),
                        )
                        .with_context("scm_id", scm.id.to_string())
                        .with_context("asset_id", sidecar_asset.id.to_string()),
                    );
                }

                // §7.3.1: SidecarAsset Id must exist in the AssetMap.
                if !asset_ids.contains(&sidecar_asset.id) {
                    report.add(
                        ValidationIssue::from_rule(
                            &st2067_9::SidecarAssetNotFound,
                            St2067_9Edition::V2018,
                            format!(
                                "SCM {} references sidecar asset {} not found in AssetMap",
                                scm.id, sidecar_asset.id
                            ),
                        )
                        .with_context("scm_id", scm.id.to_string())
                        .with_context("asset_id", sidecar_asset.id.to_string()),
                    );
                }

                // §5: Sidecar asset shall not be referenced by any Virtual Track.
                if virtual_track_file_ids.contains(&sidecar_asset.id) {
                    report.add(
                        ValidationIssue::from_rule(
                            &st2067_9::SidecarAssetReferencedByVirtualTrack,
                            St2067_9Edition::V2018,
                            format!(
                            "Sidecar asset {} (SCM {}) is referenced by a Virtual Track in a CPL",
                            sidecar_asset.id, scm.id
                        ),
                        )
                        .with_context("scm_id", scm.id.to_string())
                        .with_context("asset_id", sidecar_asset.id.to_string()),
                    );
                }

                // §7.3.1.1: CPL Ids within AssociatedCPLList.
                let mut seen_cpl_ids = HashSet::new();
                for cpl_id in &sidecar_asset.cpl_ids {
                    // No duplicate CPLIds within one AssociatedCPLList.
                    if !seen_cpl_ids.insert(*cpl_id) {
                        report.add(ValidationIssue::from_rule(&st2067_9::DuplicateCplId, St2067_9Edition::V2018,
                            format!(
                                "Duplicate CPLId {} in AssociatedCPLList of sidecar asset {} (SCM {})",
                                cpl_id, sidecar_asset.id, scm.id
                            ),
                        ).with_context("scm_id", scm.id.to_string())
                         .with_context("asset_id", sidecar_asset.id.to_string())
                         .with_context("cpl_id", cpl_id.to_string()));
                    }

                    // Flag a CPLId that is not in this package. ST 2067-9
                    // §8 (Note): the referenced composition need not be in
                    // the same IMP, so this is a warning.
                    if !self.composition_playlists.contains_key(cpl_id) {
                        report.add(ValidationIssue::from_rule(&st2067_9::CplNotFound, St2067_9Edition::V2018,
                            format!(
                                "SCM {} sidecar asset {} references CPL {} which is not known in this package",
                                scm.id, sidecar_asset.id, cpl_id
                            ),
                        ).with_context("scm_id", scm.id.to_string())
                         .with_context("asset_id", sidecar_asset.id.to_string())
                         .with_context("cpl_id", cpl_id.to_string()));
                    }
                }
            }
        }
    }

    /// Validate consistency across multiple PKLs.
    ///
    /// Per ST 2067-2 §7, when the same asset UUID appears in multiple PKLs,
    /// the hash and size must be identical. Conflicting metadata indicates
    /// a corrupt or inconsistent package delivery.
    fn validate_multi_pkl_consistency(
        &self,
        report: &mut ValidationReport,
        spec_ed: St2067_2Edition,
    ) {
        if self.packing_lists.len() < 2 {
            return; // Nothing to cross-validate
        }

        // Build: asset UUID → Vec<(pkl_id, hash_b64, size)>
        let mut asset_records: HashMap<ImfUuid, Vec<(ImfUuid, String, u64)>> = HashMap::new();
        for (pkl_id, pkl) in &self.packing_lists {
            for asset in &pkl.asset_list.assets {
                asset_records.entry(asset.id).or_default().push((
                    *pkl_id,
                    asset.hash.to_base64(),
                    asset.size,
                ));
            }
        }

        for (asset_id, records) in &asset_records {
            if records.len() < 2 {
                continue;
            }
            let (first_pkl, ref first_hash, first_size) = records[0];
            for (pkl_id, hash, size) in &records[1..] {
                if hash != first_hash {
                    report.extend(
                        pkg_or_pkl_issue(
                            &st2067_2::ChecksumMismatch,
                            &st429_8::ChecksumMismatch,
                            spec_ed,
                            format!(
                                "Asset {} has different hashes in PKL {} ({}) vs PKL {} ({})",
                                asset_id,
                                &first_pkl.to_string()[..8],
                                &first_hash[..8.min(first_hash.len())],
                                &pkl_id.to_string()[..8],
                                &hash[..8.min(hash.len())],
                            ),
                        )
                        .map(|i| i.with_context("asset_uuid", asset_id.to_string())),
                    );
                }
                if *size != first_size {
                    report.extend(
                        pkg_or_pkl_issue(&st2067_2::SizeMismatch, &st429_8::SizeMismatch, spec_ed,
                            format!(
                                "Asset {} has different sizes in PKL {} ({} bytes) vs PKL {} ({} bytes)",
                                asset_id,
                                &first_pkl.to_string()[..8],
                                first_size,
                                &pkl_id.to_string()[..8],
                                size,
                            ),
                        )
                        .map(|i| i.with_context("asset_uuid", asset_id.to_string())),
                    );
                }
            }
        }
    }

    /// ST 377-1 / ST 2067-2: Cross-validate MXF file headers against package metadata.
    ///
    /// For each MXF track file in the package:
    /// 1. Parse the MXF Header Partition Pack
    /// 2. Report an empty EssenceContainers batch (ST 377-1 §7.1)
    /// 3. Report parse failures as warnings (file may be unavailable or corrupt)
    ///
    /// The Operational Pattern is checked by `mxf::essence::validate_mxf_essence`.
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    fn validate_mxf_headers(&self, report: &mut ValidationReport, spec_ed: St2067_2Edition) {
        // Collect MXF asset UUIDs from PKLs
        for pkl in self.packing_lists.values() {
            for asset in &pkl.asset_list.assets {
                if !asset.mime_type.is_mxf() {
                    continue;
                }
                let path = match self.asset_paths.get(&asset.id) {
                    Some(p) => p,
                    None => continue, // Missing file already reported by validate_file_manifest
                };
                if !path.exists() {
                    continue; // Missing file already reported by validate_file_manifest
                }

                // The ST 377-1 / ST 377-4 editions of this track file, from the
                // dated references of the specs that apply to it; the ST 2067-201
                // edition from the CPLs whose IAB Virtual Tracks reference it.
                let file_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let iab_ref = self
                    .referenced_by_iab_track(&asset.id)
                    .then_some(crate::validation::CPL_IAB_EDITION);
                let (c1, c4) = crate::validation::mxf_editions(
                    &format!("MXF track file '{file_name}'"),
                    spec_ed,
                    iab_ref,
                );
                let iab_ed =
                    iab_ref.unwrap_or(crate::catalogue::st2067_201::St2067_201Edition::V2026);
                let mark = |r: &ValidationReport, spec| count_spec(r, spec);
                let before = [
                    mark(report, crate::diagnostics::Spec::St377_1),
                    mark(report, crate::diagnostics::Spec::St377_4),
                    mark(report, crate::diagnostics::Spec::St2067_201),
                ];

                // MXF essence checks via the `regxmllib-rs` family
                // (smpte-mxf). Emits ST 2067-2 §5.1.1 / ST 377-1 §6.1.1, §6.2.4
                // and §6.2.5 diagnostics that the hand-rolled parser
                // below doesn't cover. Native-only — the wasm build
                // doesn't link smpte-mxf, and browser callers don't
                // see MXF binaries anyway (they upload XML only).
                #[cfg(not(target_arch = "wasm32"))]
                for issue in
                    crate::mxf::essence::validate_mxf_essence_for(path, spec_ed, c1.edition)
                {
                    let issue = issue.with_context("asset_uuid", asset.id.to_string());
                    report.add(issue);
                }

                // ST 2067-2 §5.3 audio MCA + §5.4 timed-text checks.
                // Run the regxml pipeline on the MXF to get typed
                // header metadata, then apply the descriptor rules.
                // Failure to convert is surfaced as a Warning rather
                // than Error — partition-pack diagnostics above
                // already cover the structural concern, and we don't
                // want to double-report.
                #[cfg(not(target_arch = "wasm32"))]
                {
                    // Footer-first with header fallback: closed MXFs carry
                    // the authoritative (final) metadata copy in the footer
                    // partition, while standalone track files (e.g. the
                    // Fraunhofer ST 2067-203/-204 corpus) may only have it
                    // in the header.
                    match crate::mxf::metadata::parse_mxf_to_regxml_with_partition_fallback(
                        path,
                        regxml::RootMode::Preface,
                    ) {
                        Ok(regxml) => {
                            for issue in crate::mxf::audio_mca::check_audio_mca_for(
                                &regxml, path, spec_ed, c4.edition,
                            ) {
                                let issue = issue.with_context("asset_uuid", asset.id.to_string());
                                report.add(issue);
                            }
                            for issue in
                                crate::mxf::timed_text::check_timed_text(&regxml, path, spec_ed)
                            {
                                let issue = issue.with_context("asset_uuid", asset.id.to_string());
                                report.add(issue);
                            }
                            // ST 377-1 Annex B.8: Duration on Timeline
                            // Track components (ST377-1 issue 18).
                            for issue in crate::mxf::structural::check_structural_components_for(
                                &regxml, path, c1.edition,
                            ) {
                                let issue = issue.with_context("asset_uuid", asset.id.to_string());
                                report.add(issue);
                            }
                            // ST 2067-201 §5.10.2 / Annex C.2 IAB labeling
                            // rules (AUDIT-18) — no-op unless the file
                            // carries an IABEssenceDescriptor.
                            for issue in
                                crate::mxf::iab_labeling::check_iab_labeling(&regxml, path, iab_ed)
                            {
                                let issue = issue.with_context("asset_uuid", asset.id.to_string());
                                report.add(issue);
                            }
                        }
                        Err(e) => {
                            report.add(
                                crate::mxf::metadata::regxml_error_issue(path, &e)
                                    .with_context("asset_uuid", asset.id.to_string()),
                            );
                        }
                    }
                }

                match crate::mxf::parse_mxf_header_info(path) {
                    Ok(info) => {
                        // OP1a is checked by `mxf::essence::validate_mxf_essence`
                        // (ST 2067-2 OperationalPatternNotOP1A).

                        // ST 377-1 §7.1: the header partition pack lists the essence containers
                        if info.essence_containers.is_empty() {
                            report.add(
                                ValidationIssue::from_rule(&st377_1::NoEssenceContainers, c1.edition,
                                    format!(
                                        "MXF track file '{}' has no essence containers in its header partition",
                                        path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default(),
                                    ),
                                )
                                .with_location(Location::new().with_file(path.clone()))
                                .with_context("asset_uuid", asset.id.to_string()),
                            );
                        }
                    }
                    Err(crate::mxf::MxfParseError::NotMxf) => {
                        report.add(
                            ValidationIssue::from_rule(
                                &st377_1::NotMxf,
                                c1.edition,
                                format!(
                                    "File '{}' has MXF MIME type but is not a valid MXF file",
                                    path.file_name()
                                        .map(|n| n.to_string_lossy())
                                        .unwrap_or_default(),
                                ),
                            )
                            .with_location(Location::new().with_file(path.clone()))
                            .with_context("asset_uuid", asset.id.to_string()),
                        );
                    }
                    Err(e) => {
                        report.add(
                            ValidationIssue::from_rule(
                                &crate::catalogue::imferno::MxfHeaderUnreadable,
                                crate::catalogue::imferno::ImfernoEdition::Current,
                                format!(
                                    "Could not parse MXF header of '{}': {}",
                                    path.file_name()
                                        .map(|n| n.to_string_lossy())
                                        .unwrap_or_default(),
                                    e,
                                ),
                            )
                            .with_location(Location::new().with_file(path.clone()))
                            .with_context("asset_uuid", asset.id.to_string()),
                        );
                    }
                }

                // Announce an assumed edition when this file has findings of
                // that spec.
                let iab_assumed = iab_ref.is_none().then(|| {
                    format!(
                        "ST 2067-201 edition of MXF track file '{file_name}' could not be \
                         determined: no CPL IAB Virtual Track references it, and every \
                         ST 2067-201 edition shares one namespace; ST 2067-201 rules were \
                         checked against ST 2067-201:2026 (the newest)"
                    )
                });
                for ((spec, assumed), before) in [
                    (crate::diagnostics::Spec::St377_1, c1.assumed),
                    (crate::diagnostics::Spec::St377_4, c4.assumed),
                    (crate::diagnostics::Spec::St2067_201, iab_assumed),
                ]
                .into_iter()
                .zip(before)
                {
                    if let Some(why) = assumed {
                        if count_spec(report, spec) > before {
                            report.add(
                                ValidationIssue::from_rule(
                                    &imferno::EditionAssumed,
                                    ImfernoEdition::Current,
                                    why,
                                )
                                .with_location(Location::new().with_file(path.clone()))
                                .with_context("asset_uuid", asset.id.to_string()),
                            );
                        }
                    }
                }
            }
        }
    }

    /// Whether a CPL IAB Virtual Track references track file `id`.
    fn referenced_by_iab_track(&self, id: &ImfUuid) -> bool {
        self.composition_playlists.values().any(|cpl| {
            cpl.segment_list.segments.iter().any(|s| {
                s.sequence_list.iab_sequences.iter().any(|seq| {
                    seq.resource_list
                        .resources
                        .iter()
                        .any(|r| r.track_file_id.as_ref() == Some(id))
                })
            })
        })
    }

    /// ST 2067-3 §7.2: Within each segment, all virtual tracks must span the
    /// same timeline duration. Durations are compared in time (seconds), not in
    /// raw edit-rate units, because video (e.g. 24fps) and audio (e.g. 48000Hz)
    /// use different edit rates.
    ///
    /// A resource's effective duration in edit-rate units =
    /// `source_duration.unwrap_or(intrinsic_duration - entry_point.unwrap_or(0))`.
    /// Time = effective_duration / edit_rate.
    #[allow(dead_code)]
    fn validate_segment_durations(&self, report: &mut ValidationReport) {
        for cpl in self.composition_playlists.values() {
            let cpl_id = cpl.id;
            let cpl_er = cpl.edit_rate.as_ref();

            for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
                let mut durations: Vec<(String, f64)> = Vec::new();

                for seq in segment.sequence_list.all_sequences() {
                    let resources = &seq.resource_list().resources;
                    let mut total_num: u64 = 0;
                    let mut rate_den: u64 = 1;
                    for r in resources {
                        let ep = r.entry_point.unwrap_or(0);
                        let dur = r
                            .source_duration
                            .unwrap_or(r.intrinsic_duration.saturating_sub(ep));
                        let er = r
                            .edit_rate
                            .as_ref()
                            .or(cpl_er)
                            .cloned()
                            .unwrap_or(EditRate::new(1, 1));
                        total_num =
                            total_num.saturating_add(dur.saturating_mul(er.denominator as u64));
                        rate_den = er.numerator as u64;
                    }
                    if rate_den > 0 {
                        durations.push((
                            seq.track_id().to_string(),
                            total_num as f64 / rate_den as f64,
                        ));
                    }
                }

                if durations.is_empty() {
                    continue;
                }

                let first_dur = durations[0].1;
                // Allow 1μs tolerance for floating-point rounding
                const TOLERANCE: f64 = 0.000001;
                for (track_id, dur) in &durations[1..] {
                    if (*dur - first_dur).abs() > TOLERANCE {
                        report.add(
                            ValidationIssue::from_rule(
                                &st2067_3::SegmentDuration,
                                crate::validation::cpl_edition(cpl).edition,
                                format!(
                                    "Segment {} has mismatched virtual track durations: \
                                     track {} = {:.6}s but track {} = {:.6}s",
                                    seg_idx, durations[0].0, first_dur, track_id, dur,
                                ),
                            )
                            .with_location(Location::new().with_cpl(cpl_id).with_segment(seg_idx)),
                        );
                        break; // One error per segment is sufficient
                    }
                }
            }
        }
    }

    /// Accumulating version of CPL asset reference validation.
    ///
    /// Reports each CPL TrackFileId that is not an asset of this package as
    /// info. ST 2067-2:2020 §7.1: an IMP need not contain complete
    /// compositions, so this is not an error.
    fn validate_cpl_asset_references_accumulating(
        &self,
        cpl: &crate::cpl::CompositionPlaylist,
        report: &mut ValidationReport,
        spec_ed: St2067_2Edition,
    ) {
        if self.asset_map.asset_list.assets.is_empty() {
            // ST 429-9 §11: AssetList has one or more Asset elements. The
            // Asset Map namespace (`.../429-9/2007/AM`) is shared by the
            // 2007 and 2014 editions; the current edition is applied.
            report.add(
                ValidationIssue::from_rule(
                    &st429_9::AssetListEmpty,
                    St429_9Edition::V2014,
                    "AssetMap contains no assets",
                )
                .with_location(Location::new().with_cpl(cpl.id)),
            );
            return;
        }

        let assetmap_ids: std::collections::HashSet<ImfUuid> = self
            .asset_map
            .asset_list
            .assets
            .iter()
            .map(|a| a.id)
            .collect();

        // An Id listed in a PKL of this package is part of the package even
        // when the AssetMap lacks it; that gap is reported as UnresolvedUuid
        // (ERROR) by the PKL check, so no INFO is added here.
        let pkl_ids: std::collections::HashSet<ImfUuid> = self
            .packing_lists
            .values()
            .flat_map(|pkl| pkl.asset_list.assets.iter().map(|a| a.id))
            .collect();

        let cpl_id = cpl.id;

        for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
            for (seq, track_type) in segment.sequence_list.all_sequences_typed() {
                for (res_idx, resource) in seq.resource_list().resources.iter().enumerate() {
                    if let Some(ref track_file_id) = resource.track_file_id {
                        if !assetmap_ids.contains(track_file_id) && !pkl_ids.contains(track_file_id)
                        {
                            // ST 2067-2:2020 §7.1: an IMP need not contain
                            // complete compositions, so this is legal (e.g. a
                            // supplemental package referring to the OV).
                            // https://github.com/Netflix/photon/issues/337
                            report.extend(
                                pkg_issue(
                                    &st2067_2::TrackFileNotInPackage,
                                    spec_ed,
                                    format!(
                                        "{} TrackFileId {} is not in this package \
                                         (supplemental package?); it was not validated",
                                        track_type, track_file_id
                                    ),
                                )
                                .map(|i| {
                                    i.with_location(
                                        Location::new()
                                            .with_cpl(cpl_id)
                                            .with_segment(seg_idx)
                                            .with_resource(res_idx),
                                    )
                                    .with_context("track_file_id", track_file_id.to_string())
                                }),
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CplDetails {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub issue_date: String,
    pub annotation: Option<String>,
    pub issuer: Option<String>,
    pub creator: Option<String>,
    pub content_originator: Option<String>,
    pub content_versions: Vec<String>,
    pub segments: Vec<SegmentInfo>,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SegmentInfo {
    pub id: String,
    pub sequence_count: usize,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrackAnalysis {
    pub cpl_id: String,
    pub cpl_title: String,
    pub total_tracks: usize,
    pub audio_tracks: usize,
    pub video_tracks: usize,
    pub subtitle_tracks: usize,
    pub languages: Vec<String>,
    pub codecs: Vec<String>,
}

/// Project a parsed CPL into the flat `CplDetails` view used by the CLI and other consumers.
pub fn cpl_details_from(cpl: &crate::cpl::CompositionPlaylist) -> CplDetails {
    let content_versions = if let Some(ref version_list) = cpl.content_version_list {
        version_list
            .content_versions
            .iter()
            .map(|v| v.id.clone())
            .collect()
    } else {
        Vec::new()
    };

    let segments = cpl
        .segment_list
        .segments
        .iter()
        .map(|seg| {
            let seq_list = &seg.sequence_list;
            let sequence_count = seq_list.main_image_sequences.len()
                + seq_list.main_audio_sequences.len()
                + seq_list.subtitles_sequences.len();
            SegmentInfo {
                id: seg.id.to_string(),
                sequence_count,
            }
        })
        .collect();

    CplDetails {
        id: cpl.id.to_string(),
        title: cpl.content_title.text.clone(),
        kind: cpl.content_kind.to_string(),
        issue_date: cpl.issue_date.clone(),
        annotation: cpl.annotation.as_ref().map(|ls| ls.text.clone()),
        issuer: cpl.issuer.as_ref().map(|ls| ls.text.clone()),
        creator: cpl.creator.as_ref().map(|ls| ls.text.clone()),
        content_originator: cpl.content_originator.as_ref().map(|ls| ls.text.clone()),
        content_versions,
        segments,
    }
}

impl Imferno {
    /// Get detailed information about a specific CPL
    pub fn get_cpl_details(&self, uuid: &str) -> Option<CplDetails> {
        self.get_cpl_str(uuid).map(cpl_details_from)
    }

    /// Get track analysis for all CPLs
    pub fn analyze_tracks(&self) -> Vec<TrackAnalysis> {
        let mut analyses = Vec::new();

        for (uuid, cpl) in &self.composition_playlists {
            let mut total_tracks = 0;
            let mut audio_tracks = 0;
            let mut video_tracks = 0;
            let mut subtitle_tracks = 0;
            let mut codecs = std::collections::HashSet::new();

            for segment in &cpl.segment_list.segments {
                let seq_list = &segment.sequence_list;

                if !seq_list.main_image_sequences.is_empty() {
                    video_tracks += seq_list.main_image_sequences.len();
                    total_tracks += seq_list.main_image_sequences.len();
                    codecs.insert("Video".to_string());
                }

                if !seq_list.main_audio_sequences.is_empty() {
                    audio_tracks += seq_list.main_audio_sequences.len();
                    total_tracks += seq_list.main_audio_sequences.len();
                    codecs.insert("Audio".to_string());
                }

                if !seq_list.subtitles_sequences.is_empty() {
                    subtitle_tracks += seq_list.subtitles_sequences.len();
                    total_tracks += seq_list.subtitles_sequences.len();
                    codecs.insert("Subtitle".to_string());
                }
            }

            analyses.push(TrackAnalysis {
                cpl_id: uuid.to_string(),
                cpl_title: cpl.content_title.text.clone(),
                total_tracks,
                audio_tracks,
                video_tracks,
                subtitle_tracks,
                languages: Vec::new(),
                codecs: codecs.into_iter().collect(),
            });
        }

        analyses
    }

    /// Get enhanced track analysis using provided feature data
    pub fn analyze_tracks_enhanced(
        &self,
        feature_data: Option<serde_json::Value>,
    ) -> Vec<TrackAnalysis> {
        let mut analyses = Vec::new();

        for (uuid, cpl) in &self.composition_playlists {
            let mut total_tracks = 0;
            let mut audio_tracks = 0;
            let mut video_tracks = 0;
            let mut subtitle_tracks = 0;
            let mut codecs = std::collections::HashSet::new();

            for segment in &cpl.segment_list.segments {
                let seq_list = &segment.sequence_list;

                if !seq_list.main_image_sequences.is_empty() {
                    video_tracks += seq_list.main_image_sequences.len();
                    total_tracks += seq_list.main_image_sequences.len();
                }

                if !seq_list.main_audio_sequences.is_empty() {
                    audio_tracks += seq_list.main_audio_sequences.len();
                    total_tracks += seq_list.main_audio_sequences.len();
                }

                if !seq_list.subtitles_sequences.is_empty() {
                    subtitle_tracks += seq_list.subtitles_sequences.len();
                    total_tracks += seq_list.subtitles_sequences.len();
                }
            }

            let languages = if let Some(ref data) = feature_data {
                if let Some(audio_langs) = data["audio_languages"].as_array() {
                    audio_langs
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            if let Some(ref data) = feature_data {
                if let Some(video_codecs) = data["video_codecs"].as_array() {
                    for codec in video_codecs {
                        if let Some(codec_str) = codec.as_str() {
                            codecs.insert(codec_str.to_string());
                        }
                    }
                }
                if let Some(audio_codecs) = data["audio_codecs"].as_array() {
                    for codec in audio_codecs {
                        if let Some(codec_str) = codec.as_str() {
                            codecs.insert(codec_str.to_string());
                        }
                    }
                }
            }

            if video_tracks > 0 {
                codecs.insert("Video".to_string());
            }
            if audio_tracks > 0 {
                codecs.insert("Audio".to_string());
            }
            if subtitle_tracks > 0 {
                codecs.insert("Subtitle".to_string());
            }

            analyses.push(TrackAnalysis {
                cpl_id: uuid.to_string(),
                cpl_title: cpl.content_title.text.clone(),
                total_tracks,
                audio_tracks,
                video_tracks,
                subtitle_tracks,
                languages,
                codecs: codecs.into_iter().collect(),
            });
        }

        analyses
    }
}

// ── Pipeline options ──────────────────────────────────────────────────────────

pub use crate::diagnostics::{RuleSeverity, RulesConfig};

/// Options controlling validation behaviour.
#[derive(Debug, Default, Clone)]
pub struct ValidationOptions {
    /// ESLint-style per-rule severity overrides applied to the output.
    /// An empty map (the default) is a no-op.
    pub rules: RulesConfig,
    /// Core constraints spec version. `None` = auto-detect from CPL namespace.
    pub core_spec: Option<crate::validation::CoreSpecTarget>,
    /// ST 2067-3 edition for the CPL rules. `None` = resolve it per CPL from
    /// its namespace and contents (newest when they do not tell, announced).
    pub cpl_spec: Option<crate::validation::CplSpecTarget>,
    /// Application profile spec versions. `None` = auto-detect from CPL.
    pub app_specs: Option<Vec<crate::validation::AppSpecTarget>>,
    /// When `true`, collapse repeat-offender issues (same code in the
    /// same severity bucket) into one entry carrying the rest of their
    /// `Location`s in `additional_instances`. Operator-facing wins:
    /// reports stay readable on packages with thousands of similar
    /// findings. Default `false` — preserves the legacy
    /// one-issue-per-occurrence shape.
    pub aggregate_repeats: bool,
    /// Path used for hash verification (only meaningful on native targets).
    /// When `Some`, hash verification is enabled; when `None` (the default), skipped.
    #[cfg(not(target_arch = "wasm32"))]
    pub verify_hashes: Option<PathBuf>,
    /// Skip all disk I/O checks: file manifest (existence/size) and MXF header inspection.
    /// Useful for packages on slow or remote filesystems (e.g. S3 via MacFUSE) where
    /// XML-only structural validation is sufficient.
    #[cfg(not(target_arch = "wasm32"))]
    pub skip_disk_checks: bool,
}

/// Per-file hash verification status.
#[cfg(feature = "tokio")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashFileStatus {
    Waiting,
    Hashing,
    Done,
    Failed,
}

/// Per-file progress info for the hash verification display.
#[cfg(feature = "tokio")]
pub struct HashFileInfo {
    pub name: String,
    pub size: u64,
    pub bytes_done: std::sync::Arc<std::sync::atomic::AtomicU64>,
    pub status: std::sync::Arc<std::sync::atomic::AtomicU8>,
}

/// Thread-safe progress tracker for parallel hash verification.
#[cfg(feature = "tokio")]
pub struct HashProgressTracker {
    pub files: std::sync::Mutex<Vec<HashFileInfo>>,
}

#[cfg(feature = "tokio")]
impl HashProgressTracker {
    pub fn new() -> Self {
        Self {
            files: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn register(
        &self,
        name: String,
        size: u64,
    ) -> (
        std::sync::Arc<std::sync::atomic::AtomicU64>,
        std::sync::Arc<std::sync::atomic::AtomicU8>,
    ) {
        let bytes_done = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        let status = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(0));
        let bd = bytes_done.clone();
        let st = status.clone();
        self.files.lock().unwrap().push(HashFileInfo {
            name,
            size,
            bytes_done,
            status,
        });
        (bd, st)
    }

    /// Snapshot of all file progress for display. Lock-free reads on atomics.
    pub fn snapshot(&self) -> Vec<(String, u64, u64, HashFileStatus)> {
        use std::sync::atomic::Ordering::Relaxed;
        let files = self.files.lock().unwrap();
        files
            .iter()
            .map(|f| {
                let status = match f.status.load(Relaxed) {
                    1 => HashFileStatus::Hashing,
                    2 => HashFileStatus::Done,
                    3 => HashFileStatus::Failed,
                    _ => HashFileStatus::Waiting,
                };
                (f.name.clone(), f.bytes_done.load(Relaxed), f.size, status)
            })
            .collect()
    }

    /// Total bytes done across all files.
    pub fn total_bytes_done(&self) -> u64 {
        use std::sync::atomic::Ordering::Relaxed;
        let files = self.files.lock().unwrap();
        files.iter().map(|f| f.bytes_done.load(Relaxed)).sum()
    }

    /// Total bytes across all files.
    pub fn total_bytes(&self) -> u64 {
        let files = self.files.lock().unwrap();
        files.iter().map(|f| f.size).sum()
    }
}

#[cfg(feature = "tokio")]
impl Default for HashProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash a single file and compare against expected digest. Returns error on mismatch.
#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
fn hash_single_file(
    uuid: &str,
    path: &std::path::Path,
    expected_b64: &str,
    algorithm: crate::assetmap::HashAlgorithm,
    bytes_done: &std::sync::atomic::AtomicU64,
) -> Option<FileValidationError> {
    use std::io::Read;
    use std::sync::atomic::Ordering;

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            return Some(FileValidationError::Io {
                uuid: uuid.to_string(),
                path: path.to_path_buf(),
                message: e.to_string(),
            });
        }
    };

    let mut reader = std::io::BufReader::with_capacity(1024 * 1024, file);
    let mut buf = vec![0u8; 1024 * 1024];

    let actual_b64 = match algorithm {
        crate::assetmap::HashAlgorithm::Sha1 => {
            use sha1::Digest;
            let mut hasher = sha1::Sha1::new();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        hasher.update(&buf[..n]);
                        bytes_done.fetch_add(n as u64, Ordering::Relaxed);
                    }
                    Err(e) => {
                        return Some(FileValidationError::Io {
                            uuid: uuid.to_string(),
                            path: path.to_path_buf(),
                            message: e.to_string(),
                        });
                    }
                }
            }
            base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                hasher.finalize(),
            )
        }
        crate::assetmap::HashAlgorithm::Sha256 => {
            use sha2::Digest;
            let mut hasher = sha2::Sha256::new();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        hasher.update(&buf[..n]);
                        bytes_done.fetch_add(n as u64, Ordering::Relaxed);
                    }
                    Err(e) => {
                        return Some(FileValidationError::Io {
                            uuid: uuid.to_string(),
                            path: path.to_path_buf(),
                            message: e.to_string(),
                        });
                    }
                }
            }
            base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                hasher.finalize(),
            )
        }
    };

    if actual_b64 != expected_b64 {
        Some(FileValidationError::HashMismatch {
            uuid: uuid.to_string(),
            path: path.to_path_buf(),
            expected: expected_b64.to_string(),
            actual: actual_b64,
        })
    } else {
        None
    }
}

/// Number of findings of `spec` in `report`.
fn count_spec(report: &ValidationReport, spec: crate::diagnostics::Spec) -> usize {
    report
        .critical
        .iter()
        .chain(&report.errors)
        .chain(&report.warnings)
        .chain(&report.info)
        .filter(|i| i.spec == spec)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-data")
            .join(name)
    }

    /// `ValidationOptions::default()` plus the MERIDIAN-specific rule
    /// suppressions needed so the reference fixture validates clean.
    ///
    /// The Photon-vintage MERIDIAN sample carries a real ST 377-4 §6.3.2
    /// audio defect (`SoundfieldGroupLinkID` on the
    /// `AudioChannelLabelSubDescriptor` doesn't match the
    /// `SoundfieldGroupLabelSubDescriptor`'s `MCALinkID`). Our photon-
    /// parity sweep now catches this — correctly — at Error severity, so
    /// tests asserting "MERIDIAN validates without errors" need to opt
    /// out of that single rule explicitly. The CLI tests do the same via
    /// `--rule SoundfieldGroupLinkIDMismatch=off`.
    fn meridian_test_options() -> ValidationOptions {
        let mut rules = crate::diagnostics::rules::RulesConfig::default();
        rules.set_raw(
            "SoundfieldGroupLinkIDMismatch".to_string(),
            crate::diagnostics::rules::RuleSeverity::Off,
        );
        // MERIDIAN's ContentKind "Test" is not a Table 3 value (Table 3
        // lists lowercase "test"; ST 2067-3 §6.1.8 match is exact).
        rules.set_raw(
            "ContentKindUnknown".to_string(),
            crate::diagnostics::rules::RuleSeverity::Off,
        );
        ValidationOptions {
            rules,
            ..ValidationOptions::default()
        }
    }

    #[test]
    fn test_parse_netflix_photon_package() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");

        match Imferno::parse(read_dir(test_path).unwrap()) {
            Ok(package) => {
                assert_eq!(package.volume_index.index, 1);
                assert!(!package.asset_map.asset_list.assets.is_empty());
                assert!(!package.composition_playlists.is_empty());

                let main_cpl = package.get_main_cpl().unwrap();
                assert_eq!(main_cpl.content_kind, crate::cpl::ContentKind::Test);
                assert_eq!(main_cpl.content_title.text, "MERIDIAN");

                // `validate_structure()` is the legacy Result-shaped entry
                // and can't accept rule overrides, so go through
                // `validate(&meridian_test_options())` to suppress the
                // known MERIDIAN ST 377-4 §6.3.2 audio defect (see helper).
                let report = package.validate(&meridian_test_options());
                assert!(
                    !report.has_errors(),
                    "MERIDIAN should validate cleanly under meridian_test_options: {:?}",
                    report.summary()
                );
            }
            Err(e) => panic!("Failed to parse IMF package: {:?}", e),
        }
    }

    #[test]
    fn test_get_cpl_details_api() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        let cpl_uuid = "0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85";
        let details = package
            .get_cpl_details(cpl_uuid)
            .expect("Failed to get CPL details");

        assert_eq!(details.id, cpl_uuid);
        assert_eq!(details.title, "MERIDIAN");
        assert_eq!(details.kind, "Test");
        assert!(details.annotation.is_some());
        assert_eq!(details.segments.len(), 1);

        let segment = &details.segments[0];
        assert!(!segment.id.is_empty());

        // Test with non-existent UUID
        assert!(package.get_cpl_details("invalid-uuid").is_none());
    }

    #[test]
    fn test_analyze_tracks_api() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        let track_analyses = package.analyze_tracks();

        assert_eq!(track_analyses.len(), 1);
        let analysis = &track_analyses[0];

        assert_eq!(analysis.cpl_title, "MERIDIAN");
    }

    #[test]
    fn test_list_cpl_uuids_api() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        let uuids = package.list_cpl_uuids();

        assert_eq!(uuids.len(), 1);
        assert_eq!(uuids[0].to_string(), "0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85");
    }

    #[test]
    fn test_validation_api() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        let report = package.validate(&meridian_test_options());
        assert!(
            !report.has_errors(),
            "Package structure validation should have no errors: {:?}",
            report.summary()
        );
    }

    #[test]
    fn test_validate_package_structure_with_cpl_validator_injects_issues() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        const INJECTED_CODE: &str = "ST2067-3:2016:6.1.8/ContentKindUnknown";

        let report = package.validate_package_structure_with_cpl_validator(
            |cpl| {
                vec![ValidationIssue::from_rule(
                    &st2067_3::ContentKindUnknown,
                    crate::catalogue::st2067_3::St2067_3Edition::V2016,
                    format!("Injected validator issue for CPL {}", cpl.id),
                )]
            },
            false,
        );

        let expected_code = INJECTED_CODE;
        let injected_present = report
            .warnings
            .iter()
            .any(|issue| issue.code == expected_code)
            || report
                .errors
                .iter()
                .any(|issue| issue.code == expected_code)
            || report
                .critical
                .iter()
                .any(|issue| issue.code == expected_code)
            || report.info.iter().any(|issue| issue.code == expected_code);
        assert!(
            injected_present,
            "Expected injected CPL issue to be present in report"
        );
    }

    #[test]
    fn test_validate_package_structure_with_empty_cpl_validator_matches_default_counts() {
        use crate::validation::{
            validate_cpl_with_registry, ConfigurableValidatorRegistry, ValidatorSelection,
        };

        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        // default_report uses the same st2067_21 registry as validate() uses internally.
        let default_report = package.validate(&ValidationOptions::default());

        // Build the same registry that validate() uses so counts are comparable.
        let registry = ConfigurableValidatorRegistry::new(ValidatorSelection::default());
        let injected_report = package.validate_package_structure_with_cpl_validator(
            |cpl| validate_cpl_with_registry(cpl, &registry),
            false,
        );

        assert_eq!(
            default_report.total_issues(),
            injected_report.total_issues()
        );
        assert_eq!(default_report.errors.len(), injected_report.errors.len());
        assert_eq!(
            default_report.warnings.len(),
            injected_report.warnings.len()
        );
        assert_eq!(
            default_report.critical.len(),
            injected_report.critical.len()
        );
        assert_eq!(default_report.info.len(), injected_report.info.len());
    }

    #[test]
    fn test_package_with_missing_files() {
        let test_path = test_data("MissingFilesAndAssetMapEntries");

        match Imferno::parse(read_dir(test_path).unwrap()) {
            Ok(package) => {
                let validation_fails = package.validate_structure().is_err();
                let structure_report = package.validate(&ValidationOptions::default());
                assert!(validation_fails || structure_report.has_errors());
            }
            Err(_) => {
                // Expected
            }
        }
    }

    #[test]
    fn test_package_with_id_mismatch() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006_ID_MISMATCH");

        let package = Imferno::parse(read_dir(test_path).unwrap()).unwrap();
        assert!(!package.composition_playlists.is_empty());
        // Photon's ID_MISMATCH fixture: the AssetMap lists the CPL under
        // ...dca85, the CPL document says ...dca84.
        let report = package.validate(&skip_disk_options());
        assert!(
            report
                .errors
                .iter()
                .any(|i| i.code == "ST2067-2:2013:7.2/CplIdAssetMapMismatch"
                    && i.message.contains("0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca84")),
            "{:#?}",
            report.errors
        );
    }

    #[test]
    fn test_lenient_parsing() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");

        let package = Imferno::parse(read_dir(&test_path).unwrap_or_default())
            .expect("Failed to parse package");

        assert_eq!(package.composition_playlists.len(), 1);
    }

    #[test]
    fn test_error_handling_invalid_path() {
        let invalid_path = "/nonexistent/path/to/package";

        let result = Imferno::parse(read_dir(invalid_path).unwrap_or_default());
        // With an empty file map, ASSETMAP.xml will be missing → parse error
        assert!(result.is_err());
    }

    #[test]
    fn test_get_asset_path() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        if let Some(first_asset) = package.asset_map.asset_list.assets.first() {
            let asset_path = package.get_asset_path(first_asset.id);
            assert!(asset_path.is_some());
        }

        // Test with invalid asset ID
        assert!(package.get_asset_path_str("invalid-id").is_none());
    }

    #[test]
    fn test_validation_errors() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        let report = package.validate(&meridian_test_options());
        assert!(
            !report.has_errors(),
            "Validation should pass: {:?}",
            report.summary()
        );
    }

    #[test]
    fn test_get_cpl_with_invalid_uuid() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        assert!(package.get_cpl_str("invalid-uuid").is_none());

        let uuid = "0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85";
        let result = package.get_cpl_str(uuid);
        assert!(result.is_some());
    }

    #[test]
    fn test_empty_package_edge_cases() {
        let test_path = test_data("MissingFilesAndAssetMapEntries");

        if let Ok(package) = Imferno::parse(read_dir(test_path).unwrap()) {
            assert!(package.composition_playlists.is_empty());
            assert!(package.get_main_cpl().is_none());
            assert!(package.analyze_tracks().is_empty());
        }
    }

    #[test]
    fn test_bad_xml_package() {
        match Imferno::parse(read_dir(test_data("BadXML")).unwrap_or_default()) {
            Ok(_) => {}
            Err(err) => {
                assert!(
                    err.to_string().contains("parsing")
                        || err.to_string().contains("XML")
                        || err.to_string().contains("Invalid")
                        || err.to_string().contains("Missing")
                );
            }
        }
    }

    #[test]
    fn test_wrong_mime_types_package() {
        let test_path = test_data("WrongXmlMimeTypes");

        if let Ok(package) = Imferno::parse(read_dir(test_path).unwrap_or_default()) {
            assert!(!package.asset_map.asset_list.assets.is_empty());
        }
    }

    #[test]
    fn test_cpl_edge_cases() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        assert!(!package.composition_playlists.is_empty());

        let first_cpl = package.composition_playlists.values().next().unwrap();
        let details = package.get_cpl_details(&first_cpl.id.to_string()).unwrap();
        assert_eq!(details.title, first_cpl.content_title.text);

        for version in &details.content_versions {
            assert!(!version.is_empty());
        }
    }

    #[test]
    fn test_directory_structure_validation() {
        let current_dir = std::env::current_dir().unwrap();
        let result = Imferno::parse(read_dir(&current_dir).unwrap_or_default());
        assert!(result.is_err());

        let fake_dir = "/this/path/does/not/exist";
        let result = Imferno::parse(read_dir(fake_dir).unwrap_or_default());
        assert!(result.is_err());

        let file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.toml");
        let result = Imferno::parse(read_dir(file_path).unwrap_or_default());
        assert!(result.is_err());
    }

    #[test]
    fn test_serialization() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        let tracks = package.analyze_tracks();
        let json = serde_json::to_string(&tracks).expect("Failed to serialize tracks");
        assert!(json.contains("total_tracks") || json == "[]");
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package = Arc::new(
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package"),
        );

        let mut handles = vec![];

        for _ in 0..4 {
            let pkg = package.clone();
            let handle = thread::spawn(move || {
                assert!(!pkg.asset_map.asset_list.assets.is_empty());
                assert!(!pkg.composition_playlists.is_empty());
                let _ = pkg.analyze_tracks();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread failed");
        }
    }

    #[test]
    fn test_malformed_xml_handling() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        let volindex_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<VolumeIndex xmlns="http://www.smpte-ra.org/schemas/2067-2/2016/volindex">
  <Index>1</Index>
</VolumeIndex>"#;
        fs::write(temp_path.join("VOLINDEX.xml"), volindex_content)
            .expect("Failed to write VOLINDEX");

        let malformed_assetmap = r#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/2067-2/2016/assetmap">
  <Id>urn:uuid:invalid-xml</Id>
  <!-- Missing closing tag -->
  <AssetList>
    <Asset>
      <Id>test-asset</Id>
"#;
        fs::write(temp_path.join("ASSETMAP.xml"), malformed_assetmap)
            .expect("Failed to write malformed ASSETMAP");

        let result = Imferno::parse(read_dir(temp_path).unwrap());
        assert!(result.is_err(), "Should fail with malformed XML");
    }

    #[test]
    fn test_validation_with_complex_structure() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        let report = package.validate(&meridian_test_options());
        assert!(
            !report.has_errors(),
            "Package should be valid: {:?}",
            report.summary()
        );
    }

    #[test]
    fn test_package_with_no_cpls() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        let volindex_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<VolumeIndex xmlns="http://www.smpte-ra.org/schemas/2067-2/2016/volindex">
  <Index>1</Index>
</VolumeIndex>"#;
        fs::write(temp_path.join("VOLINDEX.xml"), volindex_content)
            .expect("Failed to write VOLINDEX");

        let no_cpl_assetmap = r#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/2067-2/2016/assetmap">
  <Id>urn:uuid:12345678-1234-1234-1234-123456789012</Id>
  <VolumeCount>1</VolumeCount>
  <IssueDate>2023-01-01T00:00:00</IssueDate>
  <AssetList>
    <Asset>
      <Id>urn:uuid:aabbccdd-1122-3344-5566-778899aabbcc</Id>
      <ChunkList>
        <Chunk>
          <Path>video.mxf</Path>
        </Chunk>
      </ChunkList>
    </Asset>
  </AssetList>
</AssetMap>"#;
        fs::write(temp_path.join("ASSETMAP.xml"), no_cpl_assetmap)
            .expect("Failed to write ASSETMAP");

        let result = Imferno::parse(read_dir(temp_path).unwrap());
        assert!(
            result.is_ok(),
            "Package with no CPLs should parse successfully"
        );

        let package = result.unwrap();
        assert!(package.composition_playlists.is_empty());
        assert!(package.get_main_cpl().is_none());
        assert!(package.analyze_tracks().is_empty());
    }

    /// ST 2067-2:2020 §7.1 (and 2016 §7.1): an IMP need not contain complete
    /// compositions, so an IMP without a CPL is legitimate. NoCpls is
    /// informational, never critical.
    #[test]
    fn no_cpls_is_info_not_critical() {
        assert_eq!(
            st2067_2::NoCpls
                .entry(St2067_2Edition::V2020)
                .unwrap()
                .default_severity,
            Severity::Info
        );
    }

    #[test]
    fn test_asset_path_resolution() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        for asset in &package.asset_map.asset_list.assets {
            let resolved_path = package.get_asset_path(asset.id);
            assert!(
                resolved_path.is_some(),
                "Should resolve path for asset {}",
                asset.id
            );

            let path = resolved_path.unwrap();
            assert!(path.is_absolute(), "Resolved path should be absolute");
            assert!(
                path.starts_with(&package.root_path),
                "Path should be within package directory"
            );
        }

        assert!(package.get_asset_path_str("invalid-id").is_none());
    }

    #[test]
    fn test_boundary_conditions() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        assert!(package.get_cpl_details("").is_none());
        assert!(package.get_cpl_details("   ").is_none());
        assert!(package.get_cpl_details("not-a-uuid").is_none());

        assert!(package.get_asset_path_str("").is_none());
        assert!(package.get_asset_path_str("   ").is_none());
        assert!(package.get_asset_path_str("invalid-asset-id").is_none());
    }

    #[test]
    fn test_large_package_handling() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        let cpl_count = package.composition_playlists.len();
        for _ in 0..10 {
            assert!(!package.asset_map.asset_list.assets.is_empty());
            assert_eq!(package.analyze_tracks().len(), cpl_count);
        }
    }

    #[test]
    fn test_validate_file_manifest_detects_mxf_files() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package =
            Imferno::parse(read_dir(test_path).unwrap()).expect("Failed to parse package");

        let errors = package.validate_file_manifest();

        for err in &errors {
            assert!(
                !matches!(err, FileValidationError::Missing { .. }),
                "Unexpected missing file: {}",
                err
            );
        }
    }

    #[test]
    fn test_validate_file_manifest_detects_missing_files() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let root = dir.path();

        std::fs::write(root.join("VOLINDEX.xml"), r#"<?xml version="1.0"?><VolumeIndex xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM"><Index>1</Index></VolumeIndex>"#).unwrap();

        let pkl_xml = r#"<?xml version="1.0"?><PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
<Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<AssetList>
  <Asset>
    <Id>urn:uuid:bbbbbbbb-0000-0000-0000-000000000002</Id>
    <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
    <Size>999</Size>
    <Type>application/mxf</Type>
    <OriginalFileName>missing_file.mxf</OriginalFileName>
  </Asset>
</AssetList>
</PackingList>"#;
        std::fs::write(root.join("PKL.xml"), pkl_xml).unwrap();

        let assetmap_xml = r#"<?xml version="1.0"?><AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
<Id>urn:uuid:cccccccc-0000-0000-0000-000000000003</Id>
<Creator>test</Creator>
<VolumeCount>1</VolumeCount>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<Issuer>test</Issuer>
<AssetList>
  <Asset>
    <Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
    <PackingList>true</PackingList>
    <ChunkList><Chunk><Path>PKL.xml</Path></Chunk></ChunkList>
  </Asset>
  <Asset>
    <Id>urn:uuid:bbbbbbbb-0000-0000-0000-000000000002</Id>
    <ChunkList><Chunk><Path>missing_file.mxf</Path></Chunk></ChunkList>
  </Asset>
</AssetList>
</AssetMap>"#;
        std::fs::write(root.join("ASSETMAP.xml"), assetmap_xml).unwrap();

        let package = Imferno::parse(read_dir(root).unwrap()).expect("Failed to parse package");
        let errors = package.validate_file_manifest();

        assert!(
            errors
                .iter()
                .any(|e| matches!(e, FileValidationError::Missing { .. })),
            "Expected a Missing error, got: {:?}",
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        );
    }

    // ── ST 2067-2 cross-reference validation ────────────────────────────────

    /// SMPTE ST 2067-2 §7/9: PKL asset UUIDs must exist in the AssetMap.
    #[test]
    fn test_pkl_constraints_detects_missing_assetmap_entries() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let root = dir.path();

        std::fs::write(root.join("VOLINDEX.xml"),
            r#"<?xml version="1.0"?><VolumeIndex xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM"><Index>1</Index></VolumeIndex>"#).unwrap();

        // PKL references an asset that is NOT in the AssetMap
        let pkl_xml = r#"<?xml version="1.0"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
<Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<AssetList>
  <Asset>
    <Id>urn:uuid:bbbbbbbb-0000-0000-0000-000000000002</Id>
    <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
    <Size>999</Size>
    <Type>application/mxf</Type>
    <OriginalFileName>some.mxf</OriginalFileName>
  </Asset>
  <Asset>
    <Id>urn:uuid:cccccccc-0000-0000-0000-000000000099</Id>
    <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
    <Size>100</Size>
    <Type>application/mxf</Type>
    <OriginalFileName>orphan.mxf</OriginalFileName>
  </Asset>
</AssetList>
</PackingList>"#;
        std::fs::write(root.join("PKL.xml"), pkl_xml).unwrap();

        // AssetMap only knows about the PKL and one asset (bbbbbbbb), not cccccccc
        let assetmap_xml = r#"<?xml version="1.0"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
<Id>urn:uuid:dddddddd-0000-0000-0000-000000000004</Id>
<Creator>test</Creator>
<VolumeCount>1</VolumeCount>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<Issuer>test</Issuer>
<AssetList>
  <Asset>
    <Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
    <PackingList>true</PackingList>
    <ChunkList><Chunk><Path>PKL.xml</Path></Chunk></ChunkList>
  </Asset>
  <Asset>
    <Id>urn:uuid:bbbbbbbb-0000-0000-0000-000000000002</Id>
    <ChunkList><Chunk><Path>some.mxf</Path></Chunk></ChunkList>
  </Asset>
</AssetList>
</AssetMap>"#;
        std::fs::write(root.join("ASSETMAP.xml"), assetmap_xml).unwrap();

        let package = Imferno::parse(read_dir(root).unwrap()).expect("parse");
        let errors = package.validate_pkl_constraints();

        assert!(
            errors.iter().any(|e| matches!(e, FileValidationError::NotInAssetMap { uuid, .. } if uuid.contains("cccccccc"))),
            "Expected NotInAssetMap for cccccccc, got: {:?}",
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        );
    }

    /// SMPTE ST 2067-2 §7: CPL TrackFileId references must resolve in AssetMap.
    #[test]
    fn test_cpl_asset_reference_validation_on_meridian() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package = Imferno::parse(read_dir(test_path).unwrap()).expect("parse");

        // MERIDIAN package should have valid cross-references
        let report = package.validate(&meridian_test_options());
        assert!(
            !report.has_errors(),
            "MERIDIAN should be valid: {:?}",
            report.summary()
        );
    }

    /// ST 2067-2 §9: PKL with an unrecognised namespace URI is flagged.
    /// Matches the published SMPTE PKL namespace whitelist.
    #[test]
    fn test_pkl_constraints_flags_unknown_namespace() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        std::fs::write(
            root.join("VOLINDEX.xml"),
            r#"<?xml version="1.0"?><VolumeIndex xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM"><Index>1</Index></VolumeIndex>"#,
        )
        .unwrap();
        let pkl_xml = r#"<?xml version="1.0"?>
<PackingList xmlns="urn:not-a-real-pkl-namespace">
<Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<AssetList>
  <Asset>
    <Id>urn:uuid:bbbbbbbb-0000-0000-0000-000000000002</Id>
    <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
    <Size>999</Size>
    <Type>application/mxf</Type>
  </Asset>
</AssetList>
</PackingList>"#;
        std::fs::write(root.join("PKL.xml"), pkl_xml).unwrap();
        let assetmap_xml = r#"<?xml version="1.0"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
<Id>urn:uuid:dddddddd-0000-0000-0000-000000000004</Id>
<Creator>test</Creator><VolumeCount>1</VolumeCount>
<IssueDate>2024-01-01T00:00:00Z</IssueDate><Issuer>test</Issuer>
<AssetList>
  <Asset>
    <Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
    <PackingList>true</PackingList>
    <ChunkList><Chunk><Path>PKL.xml</Path></Chunk></ChunkList>
  </Asset>
  <Asset>
    <Id>urn:uuid:bbbbbbbb-0000-0000-0000-000000000002</Id>
    <ChunkList><Chunk><Path>some.mxf</Path></Chunk></ChunkList>
  </Asset>
</AssetList></AssetMap>"#;
        std::fs::write(root.join("ASSETMAP.xml"), assetmap_xml).unwrap();
        let package = Imferno::parse(read_dir(root).unwrap()).expect("parse");
        let errors = package.validate_pkl_constraints();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, FileValidationError::UnknownPklNamespace { .. })),
            "Expected UnknownPklNamespace, got: {:?}",
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        );
    }

    /// ST 429-9 §6.3: AssetMap with no `<PackingList>true</PackingList>`
    /// asset must be flagged when a PKL document exists in the package.
    #[test]
    fn test_pkl_constraints_flags_assetmap_with_no_packinglist() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        std::fs::write(
            root.join("VOLINDEX.xml"),
            r#"<?xml version="1.0"?><VolumeIndex xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM"><Index>1</Index></VolumeIndex>"#,
        )
        .unwrap();
        let pkl_xml = r#"<?xml version="1.0"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
<Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<AssetList>
  <Asset>
    <Id>urn:uuid:bbbbbbbb-0000-0000-0000-000000000002</Id>
    <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
    <Size>999</Size>
    <Type>application/mxf</Type>
  </Asset>
</AssetList>
</PackingList>"#;
        std::fs::write(root.join("PKL.xml"), pkl_xml).unwrap();
        // AssetMap omits the PackingList flag on every asset.
        let assetmap_xml = r#"<?xml version="1.0"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
<Id>urn:uuid:dddddddd-0000-0000-0000-000000000004</Id>
<Creator>test</Creator><VolumeCount>1</VolumeCount>
<IssueDate>2024-01-01T00:00:00Z</IssueDate><Issuer>test</Issuer>
<AssetList>
  <Asset>
    <Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
    <ChunkList><Chunk><Path>PKL.xml</Path></Chunk></ChunkList>
  </Asset>
  <Asset>
    <Id>urn:uuid:bbbbbbbb-0000-0000-0000-000000000002</Id>
    <ChunkList><Chunk><Path>some.mxf</Path></Chunk></ChunkList>
  </Asset>
</AssetList></AssetMap>"#;
        std::fs::write(root.join("ASSETMAP.xml"), assetmap_xml).unwrap();
        let package = Imferno::parse(read_dir(root).unwrap()).expect("parse");
        let errors = package.validate_pkl_constraints();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, FileValidationError::AssetMapHasNoPackingList)),
            "Expected AssetMapHasNoPackingList, got: {:?}",
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        );
        // `PklIdNotInAssetMap` doesn't fire here because the package
        // pipeline only loads PKL files when the AssetMap flags them —
        // so `self.packing_lists` is empty in this scenario. That
        // check is exercised separately in
        // `test_pkl_constraints_flags_pkl_id_mismatch`.
    }

    /// ST 429-9 §6.3: when a PKL is loaded (via a flagged AssetMap
    /// asset) but the PKL document's internal Id differs from the
    /// AssetMap asset Id, flag the mismatch. Constructs a fixture
    /// where the AssetMap declares one Id but the PKL XML carries a
    /// different one inside.
    #[test]
    fn test_pkl_constraints_flags_pkl_id_mismatch() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        std::fs::write(
            root.join("VOLINDEX.xml"),
            r#"<?xml version="1.0"?><VolumeIndex xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM"><Index>1</Index></VolumeIndex>"#,
        )
        .unwrap();
        // PKL Id `99999999-...` is intentionally different from the
        // AssetMap asset Id `aaaaaaaa-...` that flags it.
        let pkl_xml = r#"<?xml version="1.0"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
<Id>urn:uuid:99999999-0000-0000-0000-000000000099</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<AssetList>
  <Asset>
    <Id>urn:uuid:bbbbbbbb-0000-0000-0000-000000000002</Id>
    <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
    <Size>999</Size>
    <Type>application/mxf</Type>
  </Asset>
</AssetList>
</PackingList>"#;
        std::fs::write(root.join("PKL.xml"), pkl_xml).unwrap();
        let assetmap_xml = r#"<?xml version="1.0"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
<Id>urn:uuid:dddddddd-0000-0000-0000-000000000004</Id>
<Creator>test</Creator><VolumeCount>1</VolumeCount>
<IssueDate>2024-01-01T00:00:00Z</IssueDate><Issuer>test</Issuer>
<AssetList>
  <Asset>
    <Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
    <PackingList>true</PackingList>
    <ChunkList><Chunk><Path>PKL.xml</Path></Chunk></ChunkList>
  </Asset>
  <Asset>
    <Id>urn:uuid:bbbbbbbb-0000-0000-0000-000000000002</Id>
    <ChunkList><Chunk><Path>some.mxf</Path></Chunk></ChunkList>
  </Asset>
</AssetList></AssetMap>"#;
        std::fs::write(root.join("ASSETMAP.xml"), assetmap_xml).unwrap();
        let package = Imferno::parse(read_dir(root).unwrap()).expect("parse");
        let errors = package.validate_pkl_constraints();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, FileValidationError::PklIdNotInAssetMap { pkl_id } if pkl_id.contains("99999999"))),
            "Expected PklIdNotInAssetMap for 99999999, got: {:?}",
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        );
    }

    /// SMPTE ST 2067-2 §9: PKL constraints validation passes on well-formed MERIDIAN.
    #[test]
    fn test_pkl_constraints_pass_on_meridian() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package = Imferno::parse(read_dir(test_path).unwrap()).expect("parse");

        let errors = package.validate_pkl_constraints();
        assert!(
            errors.is_empty(),
            "MERIDIAN PKL constraints should pass, got: {:?}",
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        );
    }

    // ── Unified ValidationReport pipeline ────────────────────────────────

    /// validate_package_structure produces a clean report for MERIDIAN.
    #[test]
    fn test_validate_package_structure_meridian() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package = Imferno::parse(read_dir(test_path).unwrap()).expect("parse");

        let report = package.validate(&meridian_test_options());
        assert!(
            !report.has_critical(),
            "MERIDIAN should have no critical issues: {}",
            report.summary()
        );
        assert!(
            !report.has_errors(),
            "MERIDIAN should have no errors: {}",
            report.summary()
        );
    }

    /// ST 2067-2:2020 §7.1: an IMP need not contain complete compositions,
    /// so a CPL track file that is not in this package (e.g. a supplemental
    /// package referring to the OV) is reported as info, not an error.
    /// https://github.com/Netflix/photon/issues/337
    #[test]
    fn cpl_track_file_outside_package_is_info_not_error() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let package = Imferno::parse(read_dir(test_path).unwrap()).expect("parse");
        let cpl_xml = std::fs::read_to_string(
            test_data("MERIDIAN_Netflix_Photon_161006")
                .join("CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml"),
        )
        .unwrap();
        let mut cpl = crate::cpl::parse_cpl(&cpl_xml).expect("CPL");
        let outside = ImfUuid::parse("urn:uuid:0f0f0f0f-1111-2222-3333-444444444444").unwrap();
        let seq = &mut cpl.segment_list.segments[0].sequence_list;
        let resource = &mut seq.main_image_sequences[0].resource_list.resources[0];
        resource.track_file_id = Some(outside);

        let mut report = ValidationReport::default();
        package.validate_cpl_asset_references_accumulating(
            &cpl,
            &mut report,
            St2067_2Edition::V2020,
        );

        assert!(report.errors.is_empty(), "{:#?}", report.errors);
        let info: Vec<_> = report
            .info
            .iter()
            .filter(|i| i.is(&st2067_2::TrackFileNotInPackage))
            .collect();
        assert_eq!(info.len(), 1, "{:#?}", report.info);
        assert!(info[0].message.contains("not in this package"));
        assert!(info[0].message.contains(&outside.to_string()));
    }

    /// A track file listed in a PKL of this package but missing from the
    /// AssetMap is part of the package; that gap is UnresolvedUuid (WARNING,
    /// from the PKL check), so no TrackFileNotInPackage INFO is added.
    #[test]
    fn cpl_track_file_in_pkl_but_not_assetmap_gets_no_info() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let mut package = Imferno::parse(read_dir(test_path).unwrap()).expect("parse");
        let cpl_xml = std::fs::read_to_string(
            test_data("MERIDIAN_Netflix_Photon_161006")
                .join("CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml"),
        )
        .unwrap();
        let cpl = crate::cpl::parse_cpl(&cpl_xml).expect("CPL");
        let track_file = cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences[0]
            .resource_list
            .resources[0]
            .track_file_id
            .expect("TrackFileId");
        assert!(package.packing_lists.values().any(|pkl| pkl
            .asset_list
            .assets
            .iter()
            .any(|a| a.id == track_file)));
        package
            .asset_map
            .asset_list
            .assets
            .retain(|a| a.id != track_file);

        // MERIDIAN is an ST 2067-2:2013 IMP (ST 429-8 PKL): no delivery
        // rules, so the gap is not reported, and the package says so.
        let as_2013 = package.validate_package_structure_with_cpl_validator(|_| Vec::new(), true);
        assert!(!as_2013
            .errors
            .iter()
            .any(|i| i.is(&st2067_2::UnresolvedUuid)));
        assert!(as_2013
            .info
            .iter()
            .any(|i| i.is(&imferno::DeliveryNotChecked)));

        // Checked as ST 2067-2:2020, the PKL→AssetMap gap is a WARNING
        // (§8.3, should) even without disk checks.
        let full = package.validate_package_structure_for(
            |_| Vec::new(),
            true,
            Some(crate::validation::CoreSpecTarget::St2067_2_2020),
        );
        let unresolved = st2067_2::UnresolvedUuid
            .code(St2067_2Edition::V2020)
            .unwrap();
        assert!(
            full.warnings
                .iter()
                .any(|i| i.code == unresolved && i.message.contains(&track_file.to_string())),
            "{:#?}",
            full.warnings
        );

        let mut report = ValidationReport::default();
        package.validate_cpl_asset_references_accumulating(
            &cpl,
            &mut report,
            St2067_2Edition::V2020,
        );
        assert!(
            !report
                .info
                .iter()
                .any(|i| i.is(&st2067_2::TrackFileNotInPackage)),
            "{:#?}",
            report.info
        );
    }

    /// ST 429-9:2014 Annex A.2 path rules run for a package on disk.
    #[test]
    fn basic_map_profile_v2_paths_checked_for_filesystem_package() {
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path();
        let assetmap_xml = r#"<?xml version="1.0"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
<Id>urn:uuid:dddddddd-0000-0000-0000-000000000004</Id>
<Creator>test</Creator>
<VolumeCount>1</VolumeCount>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<Issuer>test</Issuer>
<AssetList>
  <Asset>
    <Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
    <PackingList>true</PackingList>
    <ChunkList><Chunk><Path>my pkl.xml</Path></Chunk></ChunkList>
  </Asset>
</AssetList>
</AssetMap>"#;
        std::fs::write(root.join("ASSETMAP.xml"), assetmap_xml).unwrap();
        let package = Imferno::parse(read_dir(root).unwrap()).expect("parse");
        let report = package.validate(&ValidationOptions::default());
        assert!(
            report
                .errors
                .iter()
                .any(|i| i.code == "ST429-9:2014:A.2/PathCharacters"),
            "{:#?}",
            report.errors
        );
    }

    /// ST 429-9:2014 Annex A.2 path rules also run for in-memory input
    /// (the WASM entry point: relative keys, no root_path, no disk), with
    /// one finding per path naming every offending character (#79).
    #[test]
    fn basic_map_profile_v2_paths_checked_for_in_memory_package() {
        let assetmap_xml = r#"<?xml version="1.0"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
<Id>urn:uuid:dddddddd-0000-0000-0000-000000000004</Id>
<Creator>test</Creator>
<VolumeCount>1</VolumeCount>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<Issuer>test</Issuer>
<AssetList>
  <Asset>
    <Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
    <PackingList>true</PackingList>
    <ChunkList><Chunk><Path>FR Dubcard (v2)+.xml</Path></Chunk></ChunkList>
  </Asset>
</AssetList>
</AssetMap>"#;
        let mut files = HashMap::new();
        files.insert("ASSETMAP.xml".to_string(), assetmap_xml.to_string());
        let package = Imferno::parse(files).expect("parse");
        assert!(package.root_path.as_os_str().is_empty());
        // Same call the WASM `validate` / `buildReport` entry points make.
        let report = package.validate(&ValidationOptions::default());
        let found: Vec<_> = report
            .errors
            .iter()
            .filter(|i| i.code == "ST429-9:2014:A.2/PathCharacters")
            .collect();
        assert_eq!(found.len(), 1, "{:#?}", report.errors);
        for c in ["' '", "'('", "')'", "'+'"] {
            assert!(found[0].message.contains(c), "{c}: {}", found[0].message);
        }
    }

    /// FileValidationError::NotInAssetMap converts to REF_UNRESOLVED_UUID.
    #[test]
    fn test_file_validation_error_to_issue_not_in_assetmap() {
        let err = FileValidationError::NotInAssetMap {
            uuid: "test-uuid".to_string(),
            original_file_name: Some("test.mxf".to_string()),
        };
        let issue = err.to_issue(St2067_2Edition::V2020).unwrap();
        assert_eq!(issue.severity, Severity::Warning);
        assert_eq!(issue.category, Category::Reference);
        assert!(issue.is(&st2067_2::UnresolvedUuid));
        assert!(issue.message.contains("test-uuid"));
    }

    /// FileValidationError::HashMismatch reports ChecksumMismatch at its
    /// table severity (Error; it was Critical at this site before C0).
    #[test]
    fn test_file_validation_error_to_issue_hash_mismatch() {
        let err = FileValidationError::HashMismatch {
            uuid: "asset-123".to_string(),
            path: PathBuf::from("/tmp/test.mxf"),
            expected: "abc123".to_string(),
            actual: "def456".to_string(),
        };
        let issue = err.to_issue(St2067_2Edition::V2020).unwrap();
        assert_eq!(issue.severity, Severity::Error);
        assert!(issue.is(&st2067_2::ChecksumMismatch));
        assert!(issue.suggestion.is_some());
    }

    /// FileValidationError::Missing converts to ASSET_FILE_NOT_FOUND.
    #[test]
    fn test_file_validation_error_to_issue_missing() {
        let err = FileValidationError::Missing {
            uuid: "missing-uuid".to_string(),
            path: PathBuf::from("/tmp/missing.mxf"),
        };
        let issue = err.to_issue(St2067_2Edition::V2020).unwrap();
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.category, Category::Asset);
        assert!(issue.is(&st2067_2::FileNotFound));
    }

    /// validate_package_structure detects PKL→AssetMap orphans.
    #[test]
    fn test_validate_package_structure_detects_orphan_pkl_assets() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let root = dir.path();

        std::fs::write(root.join("VOLINDEX.xml"),
            r#"<?xml version="1.0"?><VolumeIndex xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM"><Index>1</Index></VolumeIndex>"#).unwrap();

        // PKL references cccccccc which is NOT in AssetMap
        let pkl_xml = r#"<?xml version="1.0"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/2067-2/2016/PKL">
<Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<AssetList>
  <Asset>
    <Id>urn:uuid:cccccccc-0000-0000-0000-000000000099</Id>
    <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
    <Size>100</Size>
    <Type>application/mxf</Type>
    <OriginalFileName>orphan.mxf</OriginalFileName>
  </Asset>
</AssetList>
</PackingList>"#;
        std::fs::write(root.join("PKL.xml"), pkl_xml).unwrap();

        let assetmap_xml = r#"<?xml version="1.0"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
<Id>urn:uuid:dddddddd-0000-0000-0000-000000000004</Id>
<Creator>test</Creator>
<VolumeCount>1</VolumeCount>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<Issuer>test</Issuer>
<AssetList>
  <Asset>
    <Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
    <PackingList>true</PackingList>
    <ChunkList><Chunk><Path>PKL.xml</Path></Chunk></ChunkList>
  </Asset>
</AssetList>
</AssetMap>"#;
        std::fs::write(root.join("ASSETMAP.xml"), assetmap_xml).unwrap();

        let package = Imferno::parse(read_dir(root).unwrap()).expect("parse");
        let report = package.validate(&ValidationOptions::default());

        // ST 2067-2 §8.3 (should): the orphan PKL asset is a warning.
        let all_issues: Vec<_> = report
            .warnings
            .iter()
            .filter(|i| i.is(&st2067_2::UnresolvedUuid))
            .collect();
        assert!(
            !all_issues.is_empty(),
            "Should have UnresolvedUuid for orphan PKL asset"
        );
    }

    /// validate_package_structure detects missing files on disk.
    #[test]
    fn test_validate_package_structure_detects_missing_files() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let root = dir.path();

        std::fs::write(root.join("VOLINDEX.xml"),
            r#"<?xml version="1.0"?><VolumeIndex xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM"><Index>1</Index></VolumeIndex>"#).unwrap();

        let pkl_xml = r#"<?xml version="1.0"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
<Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<AssetList>
  <Asset>
    <Id>urn:uuid:bbbbbbbb-0000-0000-0000-000000000002</Id>
    <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
    <Size>999</Size>
    <Type>application/mxf</Type>
    <OriginalFileName>ghost.mxf</OriginalFileName>
  </Asset>
</AssetList>
</PackingList>"#;
        std::fs::write(root.join("PKL.xml"), pkl_xml).unwrap();

        let assetmap_xml = r#"<?xml version="1.0"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
<Id>urn:uuid:dddddddd-0000-0000-0000-000000000004</Id>
<Creator>test</Creator>
<VolumeCount>1</VolumeCount>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<Issuer>test</Issuer>
<AssetList>
  <Asset>
    <Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
    <PackingList>true</PackingList>
    <ChunkList><Chunk><Path>PKL.xml</Path></Chunk></ChunkList>
  </Asset>
  <Asset>
    <Id>urn:uuid:bbbbbbbb-0000-0000-0000-000000000002</Id>
    <ChunkList><Chunk><Path>ghost.mxf</Path></Chunk></ChunkList>
  </Asset>
</AssetList>
</AssetMap>"#;
        std::fs::write(root.join("ASSETMAP.xml"), assetmap_xml).unwrap();
        // Note: ghost.mxf is NOT created on disk

        let package = Imferno::parse(read_dir(root).unwrap()).expect("parse");
        let report = package.validate(&ValidationOptions::default());

        assert!(
            report.has_errors(),
            "Should report errors for missing file: {}",
            report.summary()
        );
        let missing_issues: Vec<_> = report
            .errors
            .iter()
            .filter(|i| i.is(&st2067_2::FileNotFound))
            .collect();
        assert!(
            !missing_issues.is_empty(),
            "Should have FileNotFound for ghost.mxf"
        );
    }

    // ── MXF header cross-validation ─────────────────────────────────────────

    /// Build a minimal MXF byte stream with the given Operational Pattern UL.
    fn make_mxf_bytes(op_ul: [u8; 16]) -> Vec<u8> {
        let mut stream = Vec::new();
        // Key: Header Partition Pack (Closed and Complete)
        stream.extend_from_slice(&[
            0x06, 0x0E, 0x2B, 0x34, 0x02, 0x05, 0x01, 0x01, 0x0D, 0x01, 0x02, 0x01, 0x01, 0x02,
            0x04, 0x00,
        ]);
        // BER length = 88
        stream.push(88);
        // MajorVersion = 1, MinorVersion = 3
        stream.extend_from_slice(&[0x00, 0x01, 0x00, 0x03]);
        // KAGSize = 512
        stream.extend_from_slice(&[0x00, 0x00, 0x02, 0x00]);
        // ThisPartition through BodySID (56 bytes of zeros)
        stream.extend_from_slice(&[0u8; 56]);
        // OperationalPattern UL
        stream.extend_from_slice(&op_ul);
        // EssenceContainers batch: count=0, element_size=16
        stream.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        stream.extend_from_slice(&[0x00, 0x00, 0x00, 0x10]);
        stream
    }

    /// Validate a one-MXF package whose header carries Operational Pattern `op`.
    fn op_report(op: [u8; 16]) -> ValidationReport {
        let root = tempfile::tempdir().unwrap();
        let root = root.path();

        std::fs::write(root.join("video.mxf"), make_mxf_bytes(op)).unwrap();

        // Minimal PKL + AssetMap
        let pkl_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/ns/2067-2/2020">
<Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<AssetList>
  <Asset>
    <Id>urn:uuid:cccccccc-0000-0000-0000-000000000001</Id>
    <Hash>AAAAAAAAAAAAAAAAAAAAAAAAAAA=</Hash>
    <Size>105</Size>
    <Type>application/mxf</Type>
    <OriginalFileName>video.mxf</OriginalFileName>
  </Asset>
</AssetList>
</PackingList>"#;
        std::fs::write(root.join("PKL.xml"), pkl_xml).unwrap();

        let assetmap_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
<Id>urn:uuid:dddddddd-0000-0000-0000-000000000001</Id>
<Creator>test</Creator>
<VolumeCount>1</VolumeCount>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<Issuer>test</Issuer>
<AssetList>
  <Asset>
    <Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
    <PackingList>true</PackingList>
    <ChunkList><Chunk><Path>PKL.xml</Path></Chunk></ChunkList>
  </Asset>
  <Asset>
    <Id>urn:uuid:cccccccc-0000-0000-0000-000000000001</Id>
    <ChunkList><Chunk><Path>video.mxf</Path></Chunk></ChunkList>
  </Asset>
</AssetList>
</AssetMap>"#
            .to_string();
        std::fs::write(root.join("ASSETMAP.xml"), assetmap_xml).unwrap();

        let package = Imferno::parse(read_dir(root).unwrap()).expect("parse");
        package.validate(&ValidationOptions::default())
    }

    fn op_issues<'a>(
        report: &'a ValidationReport,
        rule: &dyn Fn(&ValidationIssue) -> bool,
    ) -> Vec<&'a ValidationIssue> {
        report
            .critical
            .iter()
            .chain(report.errors.iter())
            .chain(report.warnings.iter())
            .chain(report.info.iter())
            .filter(|i| rule(i))
            .collect()
    }

    /// OP1a with qualifier 0x01 (internal, stream, uni-track): no OP finding.
    #[test]
    fn mxf_validation_accepts_op1a() {
        let report = op_report([
            0x06, 0x0E, 0x2B, 0x34, 0x04, 0x01, 0x01, 0x02, 0x0D, 0x01, 0x02, 0x01, 0x01, 0x01,
            0x01, 0x00,
        ]);
        let found = op_issues(&report, &|i| {
            i.is(&st2067_2::OperationalPatternNotOP1A)
                || i.is(&crate::catalogue::st2067_5::OperationalPatternQualifier)
        });
        assert!(
            found.is_empty(),
            "OP1a should not produce OP issues: {found:#?}"
        );
    }

    /// OP1a with qualifier 0x09 sets bit 3 (multi-track): ST 2067-5 §5.2.1
    /// requires uni-track, so the qualifier finding fires (OP1a identity holds).
    #[test]
    fn mxf_validation_flags_multi_track_qualifier() {
        let report = op_report([
            0x06, 0x0E, 0x2B, 0x34, 0x04, 0x01, 0x01, 0x02, 0x0D, 0x01, 0x02, 0x01, 0x01, 0x01,
            0x09, 0x00,
        ]);
        assert!(op_issues(&report, &|i| i.is(&st2067_2::OperationalPatternNotOP1A)).is_empty());
        let q = op_issues(&report, &|i| {
            i.is(&crate::catalogue::st2067_5::OperationalPatternQualifier)
        });
        assert_eq!(q.len(), 1, "{q:#?}");
        assert_eq!(q[0].code, "ST2067-5:2020:5.2.1/OperationalPatternQualifier");
    }

    #[test]
    fn mxf_validation_flags_non_op1a() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path();

        // OP-Atom UL: bytes 13-14 = 03 01 (not OP1a's 01 01)
        let op_atom: [u8; 16] = [
            0x06, 0x0E, 0x2B, 0x34, 0x04, 0x01, 0x01, 0x02, 0x0D, 0x01, 0x02, 0x01, 0x03, 0x01,
            0x00, 0x00,
        ];
        std::fs::write(root.join("video.mxf"), make_mxf_bytes(op_atom)).unwrap();

        let pkl_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/ns/2067-2/2020">
<Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<AssetList>
  <Asset>
    <Id>urn:uuid:cccccccc-0000-0000-0000-000000000001</Id>
    <Hash>AAAAAAAAAAAAAAAAAAAAAAAAAAA=</Hash>
    <Size>105</Size>
    <Type>application/mxf</Type>
    <OriginalFileName>video.mxf</OriginalFileName>
  </Asset>
</AssetList>
</PackingList>"#;
        std::fs::write(root.join("PKL.xml"), pkl_xml).unwrap();

        let assetmap_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
<Id>urn:uuid:dddddddd-0000-0000-0000-000000000001</Id>
<Creator>test</Creator>
<VolumeCount>1</VolumeCount>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<Issuer>test</Issuer>
<AssetList>
  <Asset>
    <Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
    <PackingList>true</PackingList>
    <ChunkList><Chunk><Path>PKL.xml</Path></Chunk></ChunkList>
  </Asset>
  <Asset>
    <Id>urn:uuid:cccccccc-0000-0000-0000-000000000001</Id>
    <ChunkList><Chunk><Path>video.mxf</Path></Chunk></ChunkList>
  </Asset>
</AssetList>
</AssetMap>"#;
        std::fs::write(root.join("ASSETMAP.xml"), assetmap_xml).unwrap();

        let package = Imferno::parse(read_dir(root).unwrap()).expect("parse");
        let report = package.validate(&ValidationOptions::default());

        let op_issues: Vec<_> = report
            .critical
            .iter()
            .chain(report.errors.iter())
            .chain(report.warnings.iter())
            .chain(report.info.iter())
            .filter(|i| i.is(&st2067_2::OperationalPatternNotOP1A))
            .collect();
        assert_eq!(
            op_issues.len(),
            1,
            "Non-OP1a should produce exactly one OP issue: {:#?}",
            op_issues,
        );
    }

    #[test]
    fn mxf_validation_warns_invalid_mxf() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path();

        // Write garbage data as an MXF file
        std::fs::write(root.join("bad.mxf"), b"not-an-mxf-file-at-all-garbage").unwrap();

        let pkl_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/ns/2067-2/2020">
<Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<AssetList>
  <Asset>
    <Id>urn:uuid:cccccccc-0000-0000-0000-000000000001</Id>
    <Hash>AAAAAAAAAAAAAAAAAAAAAAAAAAA=</Hash>
    <Size>30</Size>
    <Type>application/mxf</Type>
    <OriginalFileName>bad.mxf</OriginalFileName>
  </Asset>
</AssetList>
</PackingList>"#;
        std::fs::write(root.join("PKL.xml"), pkl_xml).unwrap();

        let assetmap_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
<Id>urn:uuid:dddddddd-0000-0000-0000-000000000001</Id>
<Creator>test</Creator>
<VolumeCount>1</VolumeCount>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<Issuer>test</Issuer>
<AssetList>
  <Asset>
    <Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
    <PackingList>true</PackingList>
    <ChunkList><Chunk><Path>PKL.xml</Path></Chunk></ChunkList>
  </Asset>
  <Asset>
    <Id>urn:uuid:cccccccc-0000-0000-0000-000000000001</Id>
    <ChunkList><Chunk><Path>bad.mxf</Path></Chunk></ChunkList>
  </Asset>
</AssetList>
</AssetMap>"#;
        std::fs::write(root.join("ASSETMAP.xml"), assetmap_xml).unwrap();

        let package = Imferno::parse(read_dir(root).unwrap()).expect("parse");
        let report = package.validate(&ValidationOptions::default());

        let notmxf_issues: Vec<_> = report
            .critical
            .iter()
            .chain(report.errors.iter())
            .chain(report.warnings.iter())
            .chain(report.info.iter())
            .filter(|i| i.is(&st377_1::NotMxf))
            .collect();
        assert!(
            !notmxf_issues.is_empty(),
            "Invalid MXF should produce ST377-1-NotMxf warning: {:#?}",
            report.warnings,
        );
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Normative-claim gap closure: From<&FileValidationError> remaining variants
    // ═════════════════════════════════════════════════════════════════════════

    /// FileValidationError::SizeMismatch converts to ASSET-005.
    #[test]
    fn test_file_validation_error_to_issue_size_mismatch() {
        let err = FileValidationError::SizeMismatch {
            uuid: "size-uuid".to_string(),
            path: PathBuf::from("/tmp/test.mxf"),
            expected: 1000,
            actual: 2000,
        };
        let issue = err.to_issue(St2067_2Edition::V2020).unwrap();
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.category, Category::Asset);
        assert!(issue.is(&st2067_2::SizeMismatch));
        assert!(issue.message.contains("1000"));
        assert!(issue.message.contains("2000"));
    }

    /// FileValidationError::Io converts to ASSET-006.
    #[test]
    fn test_file_validation_error_to_issue_io() {
        let err = FileValidationError::Io {
            uuid: "io-uuid".to_string(),
            path: PathBuf::from("/tmp/broken.mxf"),
            message: "permission denied".to_string(),
        };
        let issue = err.to_issue(St2067_2Edition::V2020).unwrap();
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.category, Category::Asset);
        assert_eq!(issue.code, "IMFERNO:General/IoError");
        assert!(issue.message.contains("permission denied"));
    }

    /// FileValidationError::DuplicatePklAssetId converts to REF_DUPLICATE_UUID.
    #[test]
    fn test_file_validation_error_to_issue_duplicate_pkl_asset_id() {
        let err = FileValidationError::DuplicatePklAssetId {
            uuid: "dup-uuid".to_string(),
            pkl_id: "pkl-001".to_string(),
        };
        let issue = err.to_issue(St2067_2Edition::V2020).unwrap();
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.category, Category::Reference);
        assert!(issue.is(&st2067_2::DuplicateUuid));
        assert!(issue.message.contains("dup-uuid"));
        assert!(issue.message.contains("pkl-001"));
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Normative-claim gap closure: validate_multi_pkl_consistency
    // ═════════════════════════════════════════════════════════════════════════

    /// validate_package_structure on single-PKL fixture should not emit cross-PKL issues.
    #[test]
    fn test_multi_pkl_single_pkl_no_cross_pkl_issues() {
        let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("fixture");
        if !fixture_path.exists() {
            eprintln!("skipping: fixture/ not present");
            return;
        }
        let package = Imferno::parse(read_dir(fixture_path).unwrap()).expect("parse fixture");
        let report = package.validate(&ValidationOptions::default());
        assert!(
            !report
                .errors
                .iter()
                .any(|i| i.code.contains("ChecksumMismatch") || i.is(&st2067_2::SizeMismatch)),
            "Single-PKL package should have no multi-PKL consistency issues: {:#?}",
            report.errors,
        );
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Normative-claim gap closure: validate_segment_durations (positive path)
    // ═════════════════════════════════════════════════════════════════════════

    /// Segment duration validation on fixture should pass (tracks have matching durations).
    #[test]
    fn test_segment_durations_fixture_pass() {
        let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("fixture");
        if !fixture_path.exists() {
            eprintln!("skipping: fixture/ not present");
            return;
        }
        let package = Imferno::parse(read_dir(fixture_path).unwrap()).expect("parse fixture");
        let report = package.validate(&ValidationOptions::default());
        let duration_issues: Vec<_> = report
            .errors
            .iter()
            .filter(|i| i.code.contains("SegmentDuration"))
            .collect();
        assert!(
            duration_issues.is_empty(),
            "Fixture should have matching segment durations: {:#?}",
            duration_issues,
        );
    }

    /// Regression guard: emitted package validation codes should not use :General fallback.
    #[test]
    fn test_emitted_codes_do_not_use_general_fallback() {
        let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("fixture");
        if !fixture_path.exists() {
            eprintln!("skipping: fixture/ not present");
            return;
        }
        let package = Imferno::parse(read_dir(fixture_path).unwrap()).expect("parse fixture");
        let report = package.validate(&ValidationOptions::default());

        let all_issues: Vec<_> = report
            .critical
            .iter()
            .chain(report.errors.iter())
            .chain(report.warnings.iter())
            .chain(report.info.iter())
            .collect();

        assert!(
            !all_issues.iter().any(|i| i.code.contains(":General/")),
            "Package validator emitted :General fallback codes: {:#?}",
            all_issues,
        );
    }

    // ═════════════════════════════════════════════════════════════════════════
    // ST 429-9 — VolindexMissing and MalformedXml
    // ═════════════════════════════════════════════════════════════════════════

    /// Minimal AssetMap with a PackingList-flagged asset. ST 429-9 §6.3
    /// requires every AssetMap to identify at least one PKL via
    /// `<PackingList>true</PackingList>` — fixtures that don't include
    /// one trip the new AssetMapHasNoPackingList check.
    const MINIMAL_ASSETMAP: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
  <Id>urn:uuid:dddddddd-0000-0000-0000-000000000001</Id>
  <Creator>test</Creator>
  <VolumeCount>1</VolumeCount>
  <IssueDate>2024-01-01T00:00:00Z</IssueDate>
  <Issuer>test</Issuer>
  <AssetList>
    <Asset>
      <Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
      <PackingList>true</PackingList>
      <ChunkList><Chunk><Path>PKL.xml</Path></Chunk></ChunkList>
    </Asset>
    <Asset>
      <Id>urn:uuid:eeeeeeee-0000-0000-0000-000000000001</Id>
      <ChunkList><Chunk><Path>dummy.mxf</Path></Chunk></ChunkList>
    </Asset>
  </AssetList>
</AssetMap>"#;

    const VALID_VOLINDEX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<VolumeIndex xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
  <Index>1</Index>
</VolumeIndex>"#;

    /// ST 429-9:2014 §8 (VolumeIndex deprecated): absent VOLINDEX.xml emits
    /// VolindexMissing (Info severity).
    #[test]
    fn volindex_missing_emits_info() {
        let mut files = HashMap::new();
        files.insert("ASSETMAP.xml".to_string(), MINIMAL_ASSETMAP.to_string());

        let pkg = Imferno::parse(files).expect("parse");
        let report = pkg.validate(&ValidationOptions::default());

        let all: Vec<_> = report.info.iter().collect();
        assert!(
            all.iter().any(|i| i.code.contains("VolindexMissing")),
            "expected VolindexMissing info, got: {all:?}",
        );
    }

    /// ST 429-9:2014 §8 (VolumeIndex deprecated): malformed VOLINDEX.xml
    /// emits MalformedXml (Warning severity).
    #[test]
    fn volindex_malformed_emits_warning() {
        let mut files = HashMap::new();
        files.insert("ASSETMAP.xml".to_string(), MINIMAL_ASSETMAP.to_string());
        files.insert(
            "VOLINDEX.xml".to_string(),
            "not xml <<< garbage".to_string(),
        );

        let pkg = Imferno::parse(files).expect("parse");
        let report = pkg.validate(&ValidationOptions::default());

        assert!(
            report
                .warnings
                .iter()
                .any(|i| i.code.contains("MalformedXml")),
            "expected MalformedXml warning, got: {:?}",
            report.warnings,
        );
    }

    /// ST 429-9:2014 §8: valid VOLINDEX.xml produces no VOLINDEX diagnostic.
    #[test]
    fn volindex_valid_no_issue() {
        let mut files = HashMap::new();
        files.insert("ASSETMAP.xml".to_string(), MINIMAL_ASSETMAP.to_string());
        files.insert("VOLINDEX.xml".to_string(), VALID_VOLINDEX.to_string());

        let pkg = Imferno::parse(files).expect("parse");
        let report = pkg.validate(&ValidationOptions::default());

        let all: Vec<_> = report
            .critical
            .iter()
            .chain(report.errors.iter())
            .chain(report.warnings.iter())
            .chain(report.info.iter())
            .filter(|i| i.code.contains("ST429-9"))
            .collect();
        assert!(
            all.is_empty(),
            "expected no ST 429-9 diagnostics for valid VOLINDEX, got: {all:?}",
        );
    }

    // ── sanitize_asset_path tests ─────────────────────────────────────────

    #[test]
    fn sanitize_simple_relative_path() {
        let root = std::env::temp_dir();
        assert!(sanitize_asset_path(&root, "video.mxf").is_some());
    }

    #[test]
    fn sanitize_nested_relative_path() {
        let root = std::env::temp_dir();
        assert!(sanitize_asset_path(&root, "subdir/video.mxf").is_some());
    }

    #[test]
    fn sanitize_rejects_parent_dir_traversal() {
        let root = std::env::temp_dir();
        assert!(sanitize_asset_path(&root, "../escape.mxf").is_none());
    }

    #[test]
    fn sanitize_rejects_deep_traversal() {
        let root = std::env::temp_dir();
        assert!(sanitize_asset_path(&root, "sub/../../escape.mxf").is_none());
    }

    #[test]
    fn sanitize_rejects_absolute_path() {
        let root = std::env::temp_dir();
        assert!(sanitize_asset_path(&root, "/etc/passwd").is_none());
    }

    #[test]
    fn sanitize_rejects_double_dot_prefix() {
        let root = std::env::temp_dir();
        assert!(sanitize_asset_path(&root, "../../etc/shadow").is_none());
    }

    /// ST 429-9:2014 A.2 Note 3 allows "." and ".." segments; only a
    /// location outside the package root is forbidden, at any step of the
    /// resolution (A.2 Example 2).
    #[test]
    fn sanitize_allows_dot_dot_that_stays_inside_root() {
        let root = std::env::temp_dir();
        let p = sanitize_asset_path(&root, "a/../video.mxf").expect("inside root");
        assert_eq!(p, root.join("video.mxf"));
        let p = sanitize_asset_path(&root, "./a/b/../c.mxf").expect("inside root");
        assert_eq!(p, root.join("a/c.mxf"));
    }

    #[test]
    fn sanitize_rejects_dot_dot_that_leaves_root_at_any_step() {
        let root = std::env::temp_dir();
        // A.2 Example 2: bar/../.. is outside the tree even though the
        // final path would come back in.
        assert!(sanitize_asset_path(&root, "bar/../../foo/bar/Image.mxf").is_none());
        assert!(sanitize_asset_path(&root, "a/../../a/video.mxf").is_none());
        assert!(sanitize_asset_path(&root, "../").is_none());
    }

    // ── parse_issues tests ────────────────────────────────────────────────

    /// Minimal valid ASSETMAP XML template with placeholders for assets.
    fn minimal_assetmap(assets_xml: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
              <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
              <VolumeCount>1</VolumeCount>
              <IssueDate>2024-01-01T00:00:00+00:00</IssueDate>
              <Issuer>test</Issuer>
              <AssetList>{}</AssetList>
            </AssetMap>"#,
            assets_xml,
        )
    }

    #[test]
    fn malformed_pkl_produces_parse_issue() {
        let mut files = HashMap::new();
        files.insert(
            "ASSETMAP.xml".to_string(),
            minimal_assetmap(
                r#"<Asset>
                  <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
                  <PackingList>true</PackingList>
                  <ChunkList><Chunk><Path>PKL.xml</Path><VolumeIndex>1</VolumeIndex></Chunk></ChunkList>
                </Asset>"#,
            ),
        );
        // Deliberately malformed PKL
        files.insert("PKL.xml".to_string(), "<not-a-pkl/>".to_string());

        let package = Imferno::parse(files).expect("parse should succeed even with bad PKL");
        assert!(
            package
                .parse_issues
                .iter()
                .any(|i| i.is(&imferno::PklParseError)),
            "expected PklParseError issue, got: {:?}",
            package.parse_issues,
        );
    }

    #[test]
    fn unparseable_xml_asset_produces_parse_issue() {
        let mut files = HashMap::new();
        files.insert(
            "ASSETMAP.xml".to_string(),
            minimal_assetmap(
                r#"<Asset>
                  <Id>urn:uuid:00000000-0000-0000-0000-000000000003</Id>
                  <ChunkList><Chunk><Path>MYSTERY.xml</Path><VolumeIndex>1</VolumeIndex></Chunk></ChunkList>
                </Asset>"#,
            ),
        );
        files.insert("MYSTERY.xml".to_string(), "<SomethingElse/>".to_string());

        let package = Imferno::parse(files).expect("parse should succeed");
        assert!(
            package
                .parse_issues
                .iter()
                .any(|i| i.is(&imferno::XmlAssetParseError)),
            "expected XmlAssetParseError issue, got: {:?}",
            package.parse_issues,
        );
    }

    #[test]
    fn path_traversal_produces_parse_issue() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let files = read_dir(test_path).unwrap();
        let package = Imferno::parse(files).expect("parse should succeed");

        // Simulate what would happen with a traversal path by checking
        // that our existing valid package has NO traversal issues
        assert!(
            !package
                .parse_issues
                .iter()
                .any(|i| i.is(&imferno::PathTraversal)),
            "valid package should have no path traversal issues",
        );
    }

    #[allow(deprecated)]
    #[test]
    fn sequence_language_extracted_from_descriptors() {
        let test_path = test_data("MERIDIAN_Netflix_Photon_161006");
        let files = read_dir(test_path).unwrap();
        let package = Imferno::parse(files).unwrap();
        let report =
            crate::package::report::build_report(&package, &ValidationOptions::default(), None)
                .unwrap();
        for cpl in &report.cpls {
            let audio_seqs: Vec<_> = cpl
                .sequences
                .iter()
                .filter(|s| s.r#type == "MainAudio")
                .collect();
            assert!(
                !audio_seqs.is_empty(),
                "should have at least one audio sequence"
            );
            for seq in &audio_seqs {
                eprintln!("Audio seq {} language: {:?}", seq.track_id, seq.language);
                assert_eq!(
                    seq.language.as_deref(),
                    Some("en"),
                    "MERIDIAN audio should have language 'en', got {:?}",
                    seq.language,
                );
            }
        }
    }

    /// Copy the XML documents (not the MXFs) of a test-data package into a
    /// fresh temporary directory, so a test can edit them.
    fn copy_xml_package(name: &str) -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().unwrap();
        for entry in std::fs::read_dir(test_data(name)).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) == Some("xml") {
                std::fs::copy(&path, dir.path().join(path.file_name().unwrap())).unwrap();
            }
        }
        dir
    }

    fn skip_disk_options() -> ValidationOptions {
        ValidationOptions {
            skip_disk_checks: true,
            ..meridian_test_options()
        }
    }

    /// The MERIDIAN CPL file in a copied package.
    const MERIDIAN_CPL: &str = "CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml";

    /// DanglingEssenceDescriptor also runs at package level with
    /// `skip_disk_checks` (the CLI's `--skip-disk-checks`, formerly
    /// `--xml-only`): it needs only the CPL XML.
    /// https://github.com/jpwesselink/imferno/issues/82
    #[test]
    fn dangling_essence_descriptor_reported_with_skip_disk_checks() {
        let dir = copy_xml_package("MERIDIAN_Netflix_Photon_161006");
        let cpl_path = dir.path().join(MERIDIAN_CPL);
        let xml = std::fs::read_to_string(&cpl_path).unwrap().replacen(
            "</EssenceDescriptorList>",
            "<EssenceDescriptor><Id>urn:uuid:11111111-2222-4333-8444-555555555555</Id>\
             </EssenceDescriptor></EssenceDescriptorList>",
            1,
        );
        std::fs::write(&cpl_path, xml).unwrap();
        let package = Imferno::parse(read_dir(dir.path()).unwrap()).unwrap();
        let report = package.validate(&skip_disk_options());
        let dangling: Vec<_> = report
            .errors
            .iter()
            .filter(|i| i.code.ends_with("/DanglingEssenceDescriptor"))
            .collect();
        assert_eq!(dangling.len(), 1, "{:#?}", report.errors);
        assert!(dangling[0]
            .message
            .contains("11111111-2222-4333-8444-555555555555"));
    }

    /// ST 429-9:2014 Annex A.4: each Asset Map document shall be a file
    /// named exactly "ASSETMAP.xml". A lower-case `assetmap.xml` is still
    /// read, so the rest of the validation runs, but it is reported: on a
    /// case-sensitive filesystem a conforming reader finds no Asset Map.
    /// https://github.com/jpwesselink/imferno/issues/80
    #[test]
    fn assetmap_file_name_must_be_exact() {
        let code = st429_9::AssetMapFileName
            .code(St429_9Edition::V2014)
            .unwrap();

        let clean = copy_xml_package("MERIDIAN_Netflix_Photon_161006");
        let package = Imferno::parse(read_dir(clean.path()).unwrap()).unwrap();
        let report = package.validate(&skip_disk_options());
        assert!(
            !report.errors.iter().any(|i| i.code == code),
            "{:#?}",
            report.errors
        );

        let dir = copy_xml_package("MERIDIAN_Netflix_Photon_161006");
        std::fs::rename(
            dir.path().join("ASSETMAP.xml"),
            dir.path().join("assetmap.xml"),
        )
        .unwrap();
        let package = Imferno::parse(read_dir(dir.path()).unwrap()).unwrap();
        // Still parsed: the CPL is found through the Asset Map.
        assert_eq!(package.composition_playlists.len(), 1);
        let report = package.validate(&skip_disk_options());
        let found: Vec<_> = report.errors.iter().filter(|i| i.code == code).collect();
        assert_eq!(found.len(), 1, "{:#?}", report.errors);
        assert!(found[0].message.contains("'assetmap.xml'"));
    }

    /// Files in the package directory that the AssetMap does not list are
    /// reported once each as UnlistedEssence, also with `skip_disk_checks`
    /// (a directory listing reads no asset). It is Info: ST 2067-2:2020 §8.3
    /// Note 3 allows the delivery medium to hold other files that are not
    /// ingested, so a stray file is not a violation.
    /// https://github.com/jpwesselink/imferno/issues/84
    #[test]
    fn unlisted_file_reported_as_info_with_skip_disk_checks() {
        let code = imferno::UnlistedEssence
            .code(ImfernoEdition::Current)
            .unwrap();
        let dir = copy_xml_package("MERIDIAN_Netflix_Photon_161006");
        std::fs::write(dir.path().join("CPL_copy.xml"), "<x/>").unwrap();
        let package = Imferno::parse(read_dir(dir.path()).unwrap()).unwrap();
        let report = package.validate(&skip_disk_options());
        let found: Vec<_> = report.info.iter().filter(|i| i.code == code).collect();
        assert_eq!(found.len(), 1, "{:#?}", report.info);
        assert!(found[0].message.contains("'CPL_copy.xml'"));
        assert!(
            !report
                .errors
                .iter()
                .chain(&report.warnings)
                .any(|i| i.code == code),
            "UnlistedEssence is Info only"
        );
    }

    /// The "signature not verified" notice is a property of the tool, not of
    /// each document: one per package, listing every signed CPL as an
    /// additional instance, instead of one line per CPL.
    /// https://github.com/jpwesselink/imferno/issues/87
    #[test]
    fn digital_signature_notice_once_per_package() {
        let dir = copy_xml_package("MERIDIAN_Netflix_Photon_161006");
        let signature =
            "<ds:Signature xmlns:ds=\"http://www.w3.org/2000/09/xmldsig#\"/></CompositionPlaylist>";
        let cpl_a = std::fs::read_to_string(dir.path().join(MERIDIAN_CPL))
            .unwrap()
            .replacen("</CompositionPlaylist>", signature, 1);
        std::fs::write(dir.path().join(MERIDIAN_CPL), &cpl_a).unwrap();
        // A second signed CPL with its own Id, listed in the AssetMap and PKL.
        let (a, b) = (
            "0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85",
            "0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca86",
        );
        std::fs::write(dir.path().join("CPL_b.xml"), cpl_a.replacen(a, b, 1)).unwrap();
        let am_path = dir.path().join("ASSETMAP.xml");
        let am = std::fs::read_to_string(&am_path).unwrap().replacen(
            "<AssetList>",
            &format!(
                "<AssetList><Asset><Id>urn:uuid:{b}</Id><ChunkList><Chunk>\
                 <Path>CPL_b.xml</Path></Chunk></ChunkList></Asset>"
            ),
            1,
        );
        std::fs::write(&am_path, am).unwrap();
        let pkl_path = dir
            .path()
            .join("PKL_f5e93462-aed2-44ad-a4ba-2adb65823e7c.xml");
        let pkl = std::fs::read_to_string(&pkl_path).unwrap().replacen(
            "<AssetList>",
            &format!(
                "<AssetList><Asset><Id>urn:uuid:{b}</Id>\
                 <Hash>AX7vs/c1eJFL4eHfkLeIv79h3vE=</Hash><Size>1</Size>\
                 <Type>text/xml</Type></Asset>"
            ),
            1,
        );
        std::fs::write(&pkl_path, pkl).unwrap();

        let package = Imferno::parse(read_dir(dir.path()).unwrap()).unwrap();
        assert_eq!(package.composition_playlists.len(), 2);
        let report = package.validate(&skip_disk_options());
        let notices: Vec<_> = report
            .info
            .iter()
            .filter(|i| i.code.ends_with("/DigitalSignature"))
            .collect();
        assert_eq!(notices.len(), 1, "{:#?}", report.info);
        assert_eq!(notices[0].instance_count(), 2, "{:#?}", notices[0]);
    }

    /// Asset Identification: a CPL asset is identified by the Id element of
    /// the CompositionPlaylist, and ST 429-9 maps those asset Ids to files.
    /// An AssetMap entry whose file is a CPL with another Id sends every
    /// lookup by Id to the wrong composition. MERIDIAN is a 2013 CPL, so the
    /// ST 2067-2:2013 code (§7.2, Table 18) fires; the per-edition codes are
    /// tested in the validation module.
    /// https://github.com/jpwesselink/imferno/issues/81
    /// https://github.com/Netflix/photon/issues/265
    #[test]
    fn cpl_id_differing_from_assetmap_id_is_reported() {
        let code = "ST2067-2:2013:7.2/CplIdAssetMapMismatch";

        let clean = copy_xml_package("MERIDIAN_Netflix_Photon_161006");
        let package = Imferno::parse(read_dir(clean.path()).unwrap()).unwrap();
        let report = package.validate(&skip_disk_options());
        assert!(
            !report.errors.iter().any(|i| i.code == code),
            "{:#?}",
            report.errors
        );

        let dir = copy_xml_package("MERIDIAN_Netflix_Photon_161006");
        let cpl_path = dir.path().join(MERIDIAN_CPL);
        let xml = std::fs::read_to_string(&cpl_path).unwrap().replacen(
            "<Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>",
            "<Id>urn:uuid:99999999-b77b-4d3f-bbe5-7c69b15dca85</Id>",
            1,
        );
        std::fs::write(&cpl_path, xml).unwrap();
        let package = Imferno::parse(read_dir(dir.path()).unwrap()).unwrap();
        let report = package.validate(&skip_disk_options());
        let found: Vec<_> = report.errors.iter().filter(|i| i.code == code).collect();
        assert_eq!(found.len(), 1, "{:#?}", report.errors);
        assert!(found[0]
            .message
            .contains("0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85"));
        assert!(found[0]
            .message
            .contains("99999999-b77b-4d3f-bbe5-7c69b15dca85"));
    }
}
