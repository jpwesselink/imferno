//! Typed validation-code catalogue for codes emitted by the package module.
//!
//! Re-exports per-spec enums from their home modules within imferno-core.

pub use crate::diagnostics::codes::ValidationCode;

pub use crate::assetmap::codes::St2067_2_2020;
pub use crate::assetmap::volindex_codes::St429_9_2014;
pub use crate::cpl::codes::St2067_3_2020;
pub use crate::mxf::codes::St377_1_2011;
pub use crate::scm::codes::St2067_9_2018;

use crate::diagnostics::{Category, Severity};

// ─────────────────────────────────────────────────────────────────────────────
// imferno tool-level codes  (not derived from any SMPTE spec)
// ─────────────────────────────────────────────────────────────────────────────

/// Tool-level observation codes emitted by imferno itself.
///
/// These are not normative violations — they reflect structural observations
/// that the tool surfaces as informational findings.  Code strings use the
/// `IMFERNO:` namespace prefix to distinguish them from spec codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum ImfernoCode {
    /// An asset is present in the AssetMap but has no CPL Virtual Track
    /// reference and no SCM declaration.  Likely a sidecar essence (e.g.
    /// Dolby Atmos MXF) delivered without an accompanying SCM document.
    UnreferencedAsset,
    /// A file is present in the package directory but not listed as a
    /// chunk in any AssetMap entry.  The file is completely outside the
    /// package manifest and will be ignored by any conforming IMF reader.
    UnlistedEssence,
    /// IMF package failed to parse (top-level structural failure).
    ParseError,
    /// A PKL referenced by the AssetMap could not be parsed.
    PklParseError,
    /// An XML asset could not be parsed as CPL, OPL, or SCM.
    XmlAssetParseError,
    /// An XML file could not be read from disk.
    XmlReadError,
    /// Could not scan the package directory.
    ReadDirError,
    /// Could not read a directory entry while scanning for unlisted essences.
    DirEntryError,
    /// An asset chunk path attempts to escape the package root directory.
    PathTraversal,
}

impl ValidationCode for ImfernoCode {
    fn code(&self) -> &'static str {
        match self {
            Self::UnreferencedAsset => "IMFERNO:Package/UnreferencedAsset",
            Self::UnlistedEssence => "IMFERNO:Package/UnlistedEssence",
            Self::ParseError => "IMFERNO:Package/ParseError",
            Self::PklParseError => "IMFERNO:Package/PklParseError",
            Self::XmlAssetParseError => "IMFERNO:Package/XmlAssetParseError",
            Self::XmlReadError => "IMFERNO:Package/XmlReadError",
            Self::ReadDirError => "IMFERNO:Package/ReadDirError",
            Self::DirEntryError => "IMFERNO:Package/DirEntryError",
            Self::PathTraversal => "IMFERNO:Package/PathTraversal",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::UnreferencedAsset =>
                "Asset is present in the AssetMap but not referenced by any CPL Virtual Track and has no SCM declaration. Likely a sidecar essence without an SCM.",
            Self::UnlistedEssence =>
                "File is present in the package directory but not listed in the AssetMap. The file is invisible to any conforming IMF reader.",
            Self::ParseError =>
                "IMF package failed to parse due to a structural error.",
            Self::PklParseError =>
                "A Packing List referenced by the AssetMap could not be parsed.",
            Self::XmlAssetParseError =>
                "An XML asset could not be parsed as CPL, OPL, or SCM.",
            Self::XmlReadError =>
                "An XML file could not be read from disk.",
            Self::ReadDirError =>
                "Could not scan the package directory.",
            Self::DirEntryError =>
                "Could not read a directory entry while scanning for unlisted essences.",
            Self::PathTraversal =>
                "An asset chunk path attempts to escape the package root directory (path traversal).",
        }
    }

    fn default_severity(&self) -> Severity {
        match self {
            Self::UnreferencedAsset => Severity::Info,
            Self::UnlistedEssence => Severity::Warning,
            Self::ParseError => Severity::Critical,
            Self::PklParseError => Severity::Error,
            Self::XmlAssetParseError => Severity::Warning,
            Self::XmlReadError => Severity::Warning,
            Self::ReadDirError => Severity::Info,
            Self::DirEntryError => Severity::Info,
            Self::PathTraversal => Severity::Error,
        }
    }

    fn category(&self) -> Category {
        Category::Structure
    }
}

impl ImfernoCode {
    pub const ALL: &'static [Self] = &[
        Self::UnreferencedAsset,
        Self::UnlistedEssence,
        Self::ParseError,
        Self::PklParseError,
        Self::XmlAssetParseError,
        Self::XmlReadError,
        Self::ReadDirError,
        Self::DirEntryError,
        Self::PathTraversal,
    ];
}

impl From<ImfernoCode> for String {
    fn from(c: ImfernoCode) -> String {
        c.code().to_string()
    }
}
