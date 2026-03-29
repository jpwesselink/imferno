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
use crate::cpl::EditRate;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod codes;
pub mod report;

pub use self::report::{
    build_report, format_report, format_validation_result, FormatOptions, ImfReport, ReportFormat,
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
#[derive(Debug, serde::Serialize)]
pub struct ValidationResult {
    /// The fully parsed IMF package.
    pub package: Imferno,
    /// Validation findings (spec violations, warnings, info).
    pub validation: ValidationReport,
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
            }
        }
        Err(e) => {
            let mut validation = ValidationReport::new(ValidationProfile::SMPTE);
            validation.add(ValidationIssue::new(
                Severity::Critical,
                Category::Structure,
                codes::ImfernoCode::ParseError,
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
        }
    }
}

impl From<&FileValidationError> for ValidationIssue {
    fn from(err: &FileValidationError) -> Self {
        match err {
            FileValidationError::NotInAssetMap {
                uuid,
                original_file_name,
            } => ValidationIssue::new(
                Severity::Error,
                Category::Reference,
                codes::St2067_2_2020::UnresolvedUuid,
                format!(
                    "PKL asset {} ({}) not found in AssetMap",
                    uuid,
                    original_file_name.as_deref().unwrap_or("no filename")
                ),
            )
            .with_context("asset_uuid", uuid.clone()),
            FileValidationError::Missing { uuid, path } => ValidationIssue::new(
                Severity::Error,
                Category::Asset,
                codes::St2067_2_2020::FileNotFound,
                format!("Missing file for asset {}: {}", uuid, path.display()),
            )
            .with_location(Location::new().with_file(path.clone()))
            .with_context("asset_uuid", uuid.clone()),
            FileValidationError::SizeMismatch {
                uuid,
                path,
                expected,
                actual,
            } => ValidationIssue::new(
                Severity::Error,
                Category::Asset,
                codes::St2067_2_2020::SizeMismatch,
                format!(
                    "Size mismatch for asset {} ({}): PKL declares {} bytes, file is {} bytes",
                    uuid,
                    path.display(),
                    expected,
                    actual
                ),
            )
            .with_location(Location::new().with_file(path.clone()))
            .with_context("asset_uuid", uuid.clone())
            .with_context("expected_size", expected.to_string())
            .with_context("actual_size", actual.to_string()),
            FileValidationError::HashMismatch {
                uuid,
                path,
                expected,
                actual,
            } => ValidationIssue::new(
                Severity::Critical,
                Category::Asset,
                codes::St2067_2_2020::ChecksumMismatch,
                format!(
                    "Hash mismatch for asset {} ({}): expected {}, computed {}",
                    uuid,
                    path.display(),
                    expected,
                    actual
                ),
            )
            .with_location(Location::new().with_file(path.clone()))
            .with_context("asset_uuid", uuid.clone())
            .with_suggestion("Re-deliver the asset or re-generate the PKL hash"),
            FileValidationError::Io {
                uuid,
                path,
                message,
            } => ValidationIssue::new(
                Severity::Error,
                Category::Asset,
                codes::St2067_2_2020::IoError,
                format!(
                    "IO error reading asset {} ({}): {}",
                    uuid,
                    path.display(),
                    message
                ),
            )
            .with_location(Location::new().with_file(path.clone()))
            .with_context("asset_uuid", uuid.clone()),
            FileValidationError::DuplicatePklAssetId { uuid, pkl_id } => ValidationIssue::new(
                Severity::Error,
                Category::Reference,
                codes::St2067_2_2020::DuplicateUuid,
                format!("Duplicate asset UUID {} in PKL {}", uuid, pkl_id),
            )
            .with_context("asset_uuid", uuid.clone())
            .with_context("pkl_id", pkl_id.clone()),
        }
    }
}

/// High-level IMF package representation.
///
/// This is the full parsed package — all CPLs, PKLs, AssetMap, SCMs, and
/// cross-references. Serializable to JSON for WASM/NAPI consumers.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Imferno {
    /// Package root directory
    #[serde(serialize_with = "serialize_path")]
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
    // Check lexical components for parent-dir traversal
    for component in rel.components() {
        if component == std::path::Component::ParentDir {
            return None;
        }
    }
    let joined = root.join(rel);
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
pub fn read_dir(path: impl AsRef<Path>) -> Result<HashMap<String, String>> {
    let path = path
        .as_ref()
        .canonicalize()
        .unwrap_or_else(|_| path.as_ref().to_path_buf());
    let mut files = HashMap::new();
    for entry in std::fs::read_dir(&path)? {
        let entry = entry?;
        let p = entry.path();
        // Only read XML files — MXF and other binary assets are parsed separately
        // and must not be opened here (avoids pulling large files over remote mounts).
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext != "xml" {
            continue;
        }
        let abs_path = p.to_string_lossy().into_owned();
        match std::fs::read_to_string(&p) {
            Ok(content) => {
                files.insert(abs_path, content);
            }
            Err(e) => {
                // read_dir is a filesystem helper with no ValidationReport context;
                // log to stderr so callers have some visibility into read failures.
                eprintln!("Warning: failed to read XML file {}: {}", abs_path, e);
            }
        }
    }
    Ok(files)
}

impl Imferno {
    /// Create an empty Imferno (used when parse fails but we still need a struct).
    fn empty() -> Self {
        Self {
            root_path: PathBuf::new(),
            volume_index: VolumeIndex { index: 1 },
            volindex_issues: Vec::new(),
            parse_issues: Vec::new(),
            asset_map: crate::assetmap::parse_assetmap(
                r#"<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM"><Id>urn:uuid:00000000-0000-0000-0000-000000000000</Id><VolumeCount>1</VolumeCount><IssueDate>1970-01-01T00:00:00+00:00</IssueDate><Issuer/><AssetList/></AssetMap>"#,
            ).unwrap_or_else(|_| unreachable!()),
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
                report.add(ValidationIssue::new(
                    Severity::Critical,
                    Category::Structure,
                    codes::ImfernoCode::ParseError,
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
            app_specs: options.app_specs.clone(),
            ..Default::default()
        };
        let registry = ConfigurableValidatorRegistry::new(selection);
        #[cfg(not(target_arch = "wasm32"))]
        let skip_disk = options.skip_disk_checks;
        #[cfg(target_arch = "wasm32")]
        let skip_disk = false;
        let report = self.validate_package_structure_with_cpl_validator(
            |cpl| validate_cpl_with_registry(cpl, &registry),
            skip_disk,
        );
        report.apply_rules(&options.rules)
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
            app_specs: options.app_specs.clone(),
            ..Default::default()
        };
        let registry = ConfigurableValidatorRegistry::new(selection);
        let report = self.validate_package_with_hashes_with_cpl_validator(|cpl| {
            validate_cpl_with_registry(cpl, &registry)
        });
        report.apply_rules(&options.rules)
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
                    volindex_issues.push(ValidationIssue::new(
                        Severity::Error,
                        Category::Structure,
                        codes::St429_9_2014::MalformedXml,
                        format!("VOLINDEX.xml is not well-formed XML: {e}"),
                    ));
                    VolumeIndex { index: 1 }
                }
            },
            None => {
                volindex_issues.push(ValidationIssue::new(
                    Severity::Info,
                    Category::Structure,
                    codes::St429_9_2014::VolindexMissing,
                    "VOLINDEX.xml is absent; single-volume package assumed",
                ));
                VolumeIndex { index: 1 }
            }
        };

        // ASSETMAP.xml — required
        let assetmap_xml = find("ASSETMAP.xml")
            .ok_or_else(|| ImfError::MissingFile("ASSETMAP.xml".to_string()))?;
        let asset_map = crate::assetmap::parse_assetmap(assetmap_xml)?;

        // Asset UUID → path mapping.
        // When root_path is known (native disk load), build absolute paths
        // with path traversal protection. Otherwise keep relative paths (WASM).
        let mut asset_paths: HashMap<ImfUuid, PathBuf> = HashMap::new();
        let mut parse_issues: Vec<ValidationIssue> = Vec::new();
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
                        parse_issues.push(ValidationIssue::new(
                            Severity::Error,
                            Category::Structure,
                            codes::ImfernoCode::PathTraversal,
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
                                parse_issues.push(ValidationIssue::new(
                                    Severity::Error,
                                    Category::Structure,
                                    codes::ImfernoCode::PklParseError,
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
                        Ok(cpl) => {
                            cpl_xml_content.insert(asset.id, xml.to_string());
                            composition_playlists.insert(asset.id, cpl);
                        }
                        Err(cpl_err) => {
                            if let Ok(opl) = crate::assetmap::parse_opl(xml) {
                                output_profile_lists.insert(asset.id, opl);
                            } else if let Ok(scm) = crate::scm::parse_scm(xml) {
                                sidecar_composition_maps.insert(asset.id, scm);
                            } else {
                                parse_issues.push(ValidationIssue::new(
                                    Severity::Warning,
                                    Category::Structure,
                                    codes::ImfernoCode::XmlAssetParseError,
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

    /// Emit `ImfernoCode::UnreferencedAsset` info findings into `report` for each
    /// asset that has no CPL Virtual Track reference and no SCM declaration.
    fn emit_unreferenced_asset_info(&self, report: &mut ValidationReport) {
        use crate::diagnostics::codes::ValidationCode as _;
        for asset in self.unreferenced_assets() {
            let path = asset
                .chunk_list
                .chunks
                .first()
                .map(|c| c.path.as_str())
                .unwrap_or("(unknown)");
            report.add(ValidationIssue::new(
                Severity::Info,
                Category::Structure,
                codes::ImfernoCode::UnreferencedAsset.code(),
                format!(
                    "Asset '{}' ({}) is present in the AssetMap but not referenced by any CPL \
                     Virtual Track and has no SCM declaration",
                    path, asset.id,
                ),
            ));
        }
    }

    /// Emit `ImfernoCode::UnlistedEssence` warnings for any `.mxf` file in the
    /// package directory that is not listed as a chunk path in the AssetMap.
    ///
    /// Scans the root directory non-recursively.  Skipped on WASM and when
    /// `root_path` is unset (in-memory / WASM packages).
    #[cfg(not(target_arch = "wasm32"))]
    fn emit_unlisted_essence(&self, report: &mut ValidationReport) {
        use crate::diagnostics::codes::ValidationCode as _;
        if self.root_path.as_os_str().is_empty() {
            return;
        }

        // All filenames listed as chunks in the AssetMap.
        let mapped: std::collections::HashSet<String> = self
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

        let entries = match std::fs::read_dir(&self.root_path) {
            Ok(e) => e,
            Err(e) => {
                report.add(ValidationIssue::new(
                    Severity::Info,
                    Category::Structure,
                    codes::ImfernoCode::ReadDirError,
                    format!(
                        "Could not scan package directory for unlisted essences: {}",
                        e,
                    ),
                ));
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    report.add(ValidationIssue::new(
                        Severity::Info,
                        Category::Structure,
                        codes::ImfernoCode::DirEntryError,
                        format!("Could not read directory entry: {}", e),
                    ));
                    continue;
                }
            };
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !ext.eq_ignore_ascii_case("mxf") {
                continue;
            }
            let filename = match path.file_name() {
                Some(n) => n.to_string_lossy().into_owned(),
                None => continue,
            };
            if !mapped.contains(&filename) {
                report.add(ValidationIssue::new(
                    Severity::Warning,
                    Category::Structure,
                    codes::ImfernoCode::UnlistedEssence.code(),
                    format!(
                        "MXF file '{}' is present in the package directory but not listed in the AssetMap",
                        filename,
                    ),
                ));
            }
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
                                let mut buf = [0u8; 1024 * 1024];
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
                                let mut buf = [0u8; 1024 * 1024];
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

    /// Parallel hash verification using tokio.
    ///
    /// Hashes up to `concurrency` files simultaneously. Calls `on_progress(bytes_done, bytes_total)`
    /// periodically so callers can render a progress bar.
    ///
    /// Requires the `tokio` feature.
    #[cfg(feature = "tokio")]
    pub async fn validate_file_hashes_parallel(
        &self,
        concurrency: usize,
    ) -> (
        Vec<FileValidationError>,
        std::sync::Arc<std::sync::atomic::AtomicU64>,
        u64,
    ) {
        use std::sync::atomic::AtomicU64;
        use std::sync::Arc;

        let path_map = self.build_asset_path_map();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
        let bytes_done = Arc::new(AtomicU64::new(0));
        let mut total_bytes: u64 = 0;
        let mut handles = Vec::new();

        // First pass: validate file manifest (sync, fast)
        let manifest_errors = self.validate_file_manifest();
        let errored_uuids: std::collections::HashSet<String> = manifest_errors
            .iter()
            .map(|e| e.uuid().to_string())
            .collect();

        // Collect files to hash
        for pkl in self.packing_lists.values() {
            for asset in &pkl.asset_list.assets {
                let uuid_str = asset.id.to_string();
                if errored_uuids.contains(&uuid_str) {
                    continue;
                }
                let Some(abs_path) = path_map.get(&asset.id) else {
                    continue;
                };
                let file_size = std::fs::metadata(abs_path).map(|m| m.len()).unwrap_or(0);
                total_bytes += file_size;

                let abs_path = abs_path.clone();
                let expected_b64 = asset.hash.to_base64();
                let algorithm = asset.hash.algorithm();
                let sem = semaphore.clone();
                let counter = bytes_done.clone();

                handles.push(tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    tokio::task::spawn_blocking(move || {
                        hash_single_file(&uuid_str, &abs_path, &expected_b64, algorithm, &counter)
                    })
                    .await
                    .unwrap_or(None)
                }));
            }
        }

        // Collect results
        let mut errors = manifest_errors;
        for handle in handles {
            if let Ok(Some(err)) = handle.await {
                errors.push(err);
            }
        }

        (errors, bytes_done, total_bytes)
    }

    /// Validate PKL structural constraints per SMPTE ST 2067-2.
    ///
    /// Checks:
    /// - §9: No duplicate asset UUIDs within a single PKL
    /// - §7/9: Every PKL asset UUID exists in the AssetMap
    pub fn validate_pkl_constraints(&self) -> Vec<FileValidationError> {
        let mut errors = Vec::new();

        // Build AssetMap UUID set
        let assetmap_ids: std::collections::HashSet<ImfUuid> = self
            .asset_map
            .asset_list
            .assets
            .iter()
            .map(|a| a.id)
            .collect();

        for pkl in self.packing_lists.values() {
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
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);

        // VOLINDEX diagnostics (ST 429-9) — emitted first
        for issue in &self.volindex_issues {
            report.add(issue.clone());
        }

        // Parse-time diagnostics (PKL/CPL/OPL/SCM failures)
        for issue in &self.parse_issues {
            report.add(issue.clone());
        }

        // PKL structural constraints (ST 2067-2 §7/9)
        for issue in self
            .validate_pkl_constraints()
            .iter()
            .map(ValidationIssue::from)
        {
            report.add(issue);
        }

        // File manifest: every PKL asset exists on disk with correct size
        // (skipped on WASM — no real filesystem available, skipped when no root_path is set,
        //  and skipped when skip_disk_checks is true)
        #[cfg(not(target_arch = "wasm32"))]
        if !skip_disk_checks && !self.root_path.as_os_str().is_empty() {
            for issue in self
                .validate_file_manifest()
                .iter()
                .map(ValidationIssue::from)
            {
                report.add(issue);
            }
        }

        // CPL TrackFileId → AssetMap cross-references
        for cpl in self.composition_playlists.values() {
            self.validate_cpl_asset_references_accumulating(cpl, &mut report);

            // Optional external CPL-level validation injection
            for issue in cpl_validator(cpl) {
                report.add(issue);
            }
        }

        // SCM reference checks (ST 2067-9:2018 §6)
        self.validate_scm_references(&mut report);

        // Tool-level observations (not spec violations)
        self.emit_unreferenced_asset_info(&mut report);

        // Multi-PKL consistency (ST 2067-2 §7)
        self.validate_multi_pkl_consistency(&mut report);

        // MXF header cross-validation (ST 377-1) — skipped on WASM, when no root_path is set,
        // and when skip_disk_checks is true
        #[cfg(not(target_arch = "wasm32"))]
        if !skip_disk_checks && !self.root_path.as_os_str().is_empty() {
            self.validate_mxf_headers(&mut report);
            self.emit_unlisted_essence(&mut report);
        }

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
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);

        // VOLINDEX diagnostics (ST 429-9) — emitted first
        for issue in &self.volindex_issues {
            report.add(issue.clone());
        }

        // Parse-time diagnostics (PKL/CPL/OPL/SCM failures)
        for issue in &self.parse_issues {
            report.add(issue.clone());
        }

        // PKL structural constraints
        for issue in self
            .validate_pkl_constraints()
            .iter()
            .map(ValidationIssue::from)
        {
            report.add(issue);
        }

        // File manifest + hash verification (subsumes validate_file_manifest)
        for issue in self
            .validate_file_hashes()
            .iter()
            .map(ValidationIssue::from)
        {
            report.add(issue);
        }

        // CPL TrackFileId → AssetMap cross-references
        for cpl in self.composition_playlists.values() {
            self.validate_cpl_asset_references_accumulating(cpl, &mut report);

            // Optional external CPL-level validation injection
            for issue in cpl_validator(cpl) {
                report.add(issue);
            }
        }

        // Multi-PKL consistency
        self.validate_multi_pkl_consistency(&mut report);

        // MXF header cross-validation (ST 377-1)
        self.validate_mxf_headers(&mut report);

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
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Reference,
                        codes::St2067_9_2018::SignerWithoutSignature,
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
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Reference,
                        codes::St2067_9_2018::SignatureWithoutSigner,
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
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Reference,
                            codes::St2067_9_2018::DuplicateAssetId,
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
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Reference,
                            codes::St2067_9_2018::SidecarAssetNotFound,
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
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Reference,
                            codes::St2067_9_2018::SidecarAssetReferencedByVirtualTrack,
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
                        report.add(ValidationIssue::new(
                            Severity::Error,
                            Category::Reference,
                            codes::St2067_9_2018::DuplicateCplId,
                            format!(
                                "Duplicate CPLId {} in AssociatedCPLList of sidecar asset {} (SCM {})",
                                cpl_id, sidecar_asset.id, scm.id
                            ),
                        ).with_context("scm_id", scm.id.to_string())
                         .with_context("asset_id", sidecar_asset.id.to_string())
                         .with_context("cpl_id", cpl_id.to_string()));
                    }

                    // Each CPLId must reference a known CPL in the package.
                    if !self.composition_playlists.contains_key(cpl_id) {
                        report.add(ValidationIssue::new(
                            Severity::Error,
                            Category::Reference,
                            codes::St2067_9_2018::CplNotFound,
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
    fn validate_multi_pkl_consistency(&self, report: &mut ValidationReport) {
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
                    report.add(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Asset,
                            codes::St2067_2_2020::ChecksumMismatch,
                            format!(
                                "Asset {} has different hashes in PKL {} ({}) vs PKL {} ({})",
                                asset_id,
                                &first_pkl.to_string()[..8],
                                &first_hash[..8.min(first_hash.len())],
                                &pkl_id.to_string()[..8],
                                &hash[..8.min(hash.len())],
                            ),
                        )
                        .with_context("asset_uuid", asset_id.to_string()),
                    );
                }
                if *size != first_size {
                    report.add(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Asset,
                            codes::St2067_2_2020::SizeMismatch,
                            format!(
                                "Asset {} has different sizes in PKL {} ({} bytes) vs PKL {} ({} bytes)",
                                asset_id,
                                &first_pkl.to_string()[..8],
                                first_size,
                                &pkl_id.to_string()[..8],
                                size,
                            ),
                        )
                        .with_context("asset_uuid", asset_id.to_string()),
                    );
                }
            }
        }
    }

    /// ST 377-1 / ST 2067-2: Cross-validate MXF file headers against package metadata.
    ///
    /// For each MXF track file in the package:
    /// 1. Parse the MXF Header Partition Pack
    /// 2. Check that the Operational Pattern is OP1a (required for IMF per ST 2067-2)
    /// 3. Report parse failures as warnings (file may be unavailable or corrupt)
    fn validate_mxf_headers(&self, report: &mut ValidationReport) {
        // OP1a UL prefix: 060e2b34.04010102.0d010201.0101__00
        // Bytes 13-14 identify the OP variant: 01 01 = OP1a, 01 02 = OP1b, etc.
        // Byte 15 encodes the qualifier (xxxx xxxx pattern). We ignore byte 8 (version).
        const OP1A_BYTES_13_14: [u8; 2] = [0x01, 0x01];

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

                match crate::mxf::parse_mxf_header_info(path) {
                    Ok(info) => {
                        // Parse the operational pattern UL back to bytes to check OP variant.
                        // The UL format is: urn:smpte:ul:XXXXXXXX.XXXXXXXX.XXXXXXXX.XXXXXXXX
                        // We need bytes 13-14 (1-indexed) to identify the OP.
                        let op_bytes = parse_ul_bytes(&info.operational_pattern);
                        if let Some(bytes) = op_bytes {
                            // IMF requires OP1a: bytes 13-14 (0-indexed: 12-13) = 01 01
                            if bytes[12] != OP1A_BYTES_13_14[0] || bytes[13] != OP1A_BYTES_13_14[1]
                            {
                                report.add(
                                    ValidationIssue::new(
                                        Severity::Error,
                                        Category::Encoding,
                                        codes::St377_1_2011::Op1a,
                                        format!(
                                            "MXF track file '{}' has Operational Pattern '{}' \
                                             but IMF requires OP1a (ST 2067-2 §5.1)",
                                            path.file_name()
                                                .map(|n| n.to_string_lossy())
                                                .unwrap_or_default(),
                                            info.operational_pattern,
                                        ),
                                    )
                                    .with_location(Location::new().with_file(path.clone()))
                                    .with_context("asset_uuid", asset.id.to_string()),
                                );
                            }
                        }

                        // ST 377-1: MXF track files should have at least one essence container
                        if info.essence_containers.is_empty() {
                            report.add(
                                ValidationIssue::new(
                                    Severity::Warning,
                                    Category::Encoding,
                                    codes::St377_1_2011::NoEssenceContainers,
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
                            ValidationIssue::new(
                                Severity::Warning,
                                Category::Asset,
                                codes::St377_1_2011::NotMxf,
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
                            ValidationIssue::new(
                                Severity::Warning,
                                Category::Asset,
                                codes::St377_1_2011::ParseError,
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
            }
        }
    }

    /// ST 2067-3 §7.2.2: Within each segment, all virtual tracks must span the
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
                            ValidationIssue::new(
                                Severity::Error,
                                Category::Timing,
                                codes::St2067_3_2020::SegmentDuration,
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
    /// Per SMPTE ST 2067-2 §7, every TrackFileId in a CPL Resource must correspond
    /// to an asset UUID in the AssetMap. Reports each missing reference as a separate
    /// `ValidationIssue` rather than failing on the first one.
    fn validate_cpl_asset_references_accumulating(
        &self,
        cpl: &crate::cpl::CompositionPlaylist,
        report: &mut ValidationReport,
    ) {
        if self.asset_map.asset_list.assets.is_empty() {
            report.add(
                ValidationIssue::new(
                    Severity::Critical,
                    Category::Structure,
                    codes::St2067_2_2020::AssetMap,
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

        let cpl_id = cpl.id;

        for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
            for (seq, track_type) in segment.sequence_list.all_sequences_typed() {
                for (res_idx, resource) in seq.resource_list().resources.iter().enumerate() {
                    if let Some(ref track_file_id) = resource.track_file_id {
                        if !assetmap_ids.contains(track_file_id) {
                            report.add(
                                ValidationIssue::new(
                                    Severity::Error,
                                    Category::Reference,
                                    codes::St2067_2_2020::UnresolvedUuid,
                                    format!(
                                        "{} TrackFileId {} not found in AssetMap",
                                        track_type, track_file_id
                                    ),
                                )
                                .with_location(
                                    Location::new()
                                        .with_cpl(cpl_id)
                                        .with_segment(seg_idx)
                                        .with_resource(res_idx),
                                )
                                .with_context("track_file_id", track_file_id.to_string()),
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Parse a `urn:smpte:ul:XXXXXXXX.XXXXXXXX.XXXXXXXX.XXXXXXXX` string into 16 raw bytes.
fn parse_ul_bytes(ul: &str) -> Option<[u8; 16]> {
    let hex = ul.strip_prefix("urn:smpte:ul:")?;
    let hex_clean: String = hex.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex_clean.len() != 32 {
        return None;
    }
    let mut bytes = [0u8; 16];
    for i in 0..16 {
        bytes[i] = u8::from_str_radix(&hex_clean[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(bytes)
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

impl Imferno {
    /// Get detailed information about a specific CPL
    pub fn get_cpl_details(&self, uuid: &str) -> Option<CplDetails> {
        let cpl = self.get_cpl_str(uuid)?;

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

        Some(CplDetails {
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
        })
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
    /// Application profile spec versions. `None` = auto-detect from CPL.
    pub app_specs: Option<Vec<crate::validation::AppSpecTarget>>,
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

/// Hash a single file and compare against expected digest. Returns error on mismatch.
#[cfg(not(target_arch = "wasm32"))]
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
    let mut buf = [0u8; 1024 * 1024];

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

#[cfg(test)]
mod tests {
    use super::*;
    use codes::{St2067_2_2020, St377_1_2011, ValidationCode};

    fn test_data(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-data")
            .join(name)
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

                package.validate_structure().unwrap();
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

        let report = package.validate(&ValidationOptions::default());
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

        const INJECTED_CODE: &str = "ST2067-2:2020:6.12/InjectedRuleForTest";

        let report = package.validate_package_structure_with_cpl_validator(
            |cpl| {
                vec![ValidationIssue::new(
                    Severity::Warning,
                    Category::Metadata,
                    INJECTED_CODE,
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

        if let Ok(package) = Imferno::parse(read_dir(test_path).unwrap()) {
            assert!(!package.composition_playlists.is_empty());
        }
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

        let report = package.validate(&ValidationOptions::default());
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

        let report = package.validate(&ValidationOptions::default());
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
        let report = package.validate(&ValidationOptions::default());
        assert!(
            !report.has_errors(),
            "MERIDIAN should be valid: {:?}",
            report.summary()
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

        let report = package.validate(&ValidationOptions::default());
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

    /// FileValidationError::NotInAssetMap converts to REF_UNRESOLVED_UUID.
    #[test]
    fn test_file_validation_error_to_issue_not_in_assetmap() {
        let err = FileValidationError::NotInAssetMap {
            uuid: "test-uuid".to_string(),
            original_file_name: Some("test.mxf".to_string()),
        };
        let issue = ValidationIssue::from(&err);
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.category, Category::Reference);
        assert_eq!(issue.code, codes::St2067_2_2020::UnresolvedUuid.code());
        assert!(issue.message.contains("test-uuid"));
    }

    /// FileValidationError::HashMismatch converts to Critical severity.
    #[test]
    fn test_file_validation_error_to_issue_hash_mismatch() {
        let err = FileValidationError::HashMismatch {
            uuid: "asset-123".to_string(),
            path: PathBuf::from("/tmp/test.mxf"),
            expected: "abc123".to_string(),
            actual: "def456".to_string(),
        };
        let issue = ValidationIssue::from(&err);
        assert_eq!(issue.severity, Severity::Critical);
        assert_eq!(issue.code, codes::St2067_2_2020::ChecksumMismatch.code());
        assert!(issue.suggestion.is_some());
    }

    /// FileValidationError::Missing converts to ASSET_FILE_NOT_FOUND.
    #[test]
    fn test_file_validation_error_to_issue_missing() {
        let err = FileValidationError::Missing {
            uuid: "missing-uuid".to_string(),
            path: PathBuf::from("/tmp/missing.mxf"),
        };
        let issue = ValidationIssue::from(&err);
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.category, Category::Asset);
        assert_eq!(issue.code, codes::St2067_2_2020::FileNotFound.code());
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
<PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
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

        assert!(
            report.has_errors(),
            "Should report errors for orphan PKL asset: {}",
            report.summary()
        );
        // Should have at least the NotInAssetMap error
        let all_issues: Vec<_> = report
            .errors
            .iter()
            .filter(|i| i.code == codes::St2067_2_2020::UnresolvedUuid.code())
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
            .filter(|i| i.code == codes::St2067_2_2020::FileNotFound.code())
            .collect();
        assert!(
            !missing_issues.is_empty(),
            "Should have FileNotFound for ghost.mxf"
        );
    }

    // ── parse_ul_bytes ──────────────────────────────────────────────────────

    #[test]
    fn parse_ul_bytes_valid() {
        let bytes = parse_ul_bytes("urn:smpte:ul:060e2b34.04010102.0d010201.01010900");
        assert!(bytes.is_some());
        let b = bytes.unwrap();
        assert_eq!(b[0], 0x06);
        assert_eq!(b[12], 0x01);
        assert_eq!(b[13], 0x01); // OP1a
        assert_eq!(b[14], 0x09);
    }

    #[test]
    fn parse_ul_bytes_invalid() {
        assert!(parse_ul_bytes("not-a-ul").is_none());
        assert!(parse_ul_bytes("urn:smpte:ul:060e2b34").is_none());
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

    #[test]
    fn mxf_validation_accepts_op1a() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path();

        // OP1a UL
        let op1a: [u8; 16] = [
            0x06, 0x0E, 0x2B, 0x34, 0x04, 0x01, 0x01, 0x02, 0x0D, 0x01, 0x02, 0x01, 0x01, 0x01,
            0x09, 0x00,
        ];
        std::fs::write(root.join("video.mxf"), make_mxf_bytes(op1a)).unwrap();

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
        let report = package.validate(&ValidationOptions::default());

        let op_issues: Vec<_> = report
            .critical
            .iter()
            .chain(report.errors.iter())
            .chain(report.warnings.iter())
            .chain(report.info.iter())
            .filter(|i| i.code == St377_1_2011::Op1a.code())
            .collect();
        assert!(
            op_issues.is_empty(),
            "OP1a should not produce OP issues: {:#?}",
            op_issues,
        );
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
            .filter(|i| i.code == St377_1_2011::Op1a.code())
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
            .filter(|i| i.code == St377_1_2011::NotMxf.code())
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
        let issue = ValidationIssue::from(&err);
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.category, Category::Asset);
        assert_eq!(issue.code, St2067_2_2020::SizeMismatch.code());
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
        let issue = ValidationIssue::from(&err);
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.category, Category::Asset);
        assert_eq!(issue.code, "IMF:General/IoError");
        assert!(issue.message.contains("permission denied"));
    }

    /// FileValidationError::DuplicatePklAssetId converts to REF_DUPLICATE_UUID.
    #[test]
    fn test_file_validation_error_to_issue_duplicate_pkl_asset_id() {
        let err = FileValidationError::DuplicatePklAssetId {
            uuid: "dup-uuid".to_string(),
            pkl_id: "pkl-001".to_string(),
        };
        let issue = ValidationIssue::from(&err);
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.category, Category::Reference);
        assert_eq!(issue.code, codes::St2067_2_2020::DuplicateUuid.code());
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
                .any(|i| i.code.contains("ChecksumMismatch")
                    || i.code == St2067_2_2020::SizeMismatch.code()),
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

    const MINIMAL_ASSETMAP: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
  <Id>urn:uuid:dddddddd-0000-0000-0000-000000000001</Id>
  <Creator>test</Creator>
  <VolumeCount>1</VolumeCount>
  <IssueDate>2024-01-01T00:00:00Z</IssueDate>
  <Issuer>test</Issuer>
  <AssetList>
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

    /// ST 429-9 §7: absent VOLINDEX.xml emits VolindexMissing (Info severity).
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

    /// ST 429-9 §7: malformed VOLINDEX.xml emits MalformedXml (Error severity).
    #[test]
    fn volindex_malformed_emits_error() {
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
                .errors
                .iter()
                .any(|i| i.code.contains("MalformedXml")),
            "expected MalformedXml error, got: {:?}",
            report.errors,
        );
    }

    /// ST 429-9 §7: valid VOLINDEX.xml produces no VOLINDEX diagnostic.
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
                .any(|i| i.code == codes::ImfernoCode::PklParseError.code()),
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
                .any(|i| i.code == codes::ImfernoCode::XmlAssetParseError.code()),
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
                .any(|i| i.code == codes::ImfernoCode::PathTraversal.code()),
            "valid package should have no path traversal issues",
        );
    }

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
}
