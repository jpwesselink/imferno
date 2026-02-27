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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImfernoCode {
    /// An asset is present in the AssetMap but has no CPL Virtual Track
    /// reference and no SCM declaration.  Likely a sidecar essence (e.g.
    /// Dolby Atmos MXF) delivered without an accompanying SCM document.
    UnreferencedAsset,
    /// An MXF file is present in the package directory but not listed as a
    /// chunk in any AssetMap entry.  The file is completely outside the
    /// package manifest and will be ignored by any conforming IMF reader.
    UnlistedEssence,
}

impl ValidationCode for ImfernoCode {
    fn code(&self) -> &'static str {
        match self {
            Self::UnreferencedAsset => "IMFERNO:Package/UnreferencedAsset",
            Self::UnlistedEssence => "IMFERNO:Package/UnlistedEssence",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::UnreferencedAsset =>
                "Asset is present in the AssetMap but not referenced by any CPL Virtual Track and has no SCM declaration. Likely a sidecar essence without an SCM.",
            Self::UnlistedEssence =>
                "MXF file is present in the package directory but not listed in the AssetMap. The file is invisible to any conforming IMF reader.",
        }
    }

    fn default_severity(&self) -> Severity {
        match self {
            Self::UnreferencedAsset => Severity::Info,
            Self::UnlistedEssence => Severity::Warning,
        }
    }

    fn category(&self) -> Category {
        Category::Structure
    }
}

impl ImfernoCode {
    pub const ALL: &'static [Self] = &[Self::UnreferencedAsset, Self::UnlistedEssence];
}

impl From<ImfernoCode> for String {
    fn from(c: ImfernoCode) -> String {
        c.code().to_string()
    }
}
