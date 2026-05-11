//! Typed validation-code catalogue for SMPTE ST 2067-9:2018 (Sidecar Composition Map).

use crate::diagnostics::codes::ValidationCode;
use crate::diagnostics::{Category, Severity};

/// Validation codes for ST 2067-9:2018 Sidecar Composition Map constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum St2067_9_2018 {
    /// SCM XML document is not well-formed or cannot be parsed (§6.1).
    MalformedXml,
    /// A sidecar asset is referenced by a Virtual Track in a CPL, violating §5.
    SidecarAssetReferencedByVirtualTrack,
    /// The same SidecarAsset Id appears more than once within an SCM document (§7.2.3).
    DuplicateAssetId,
    /// A `Signer` element is present but no `Signature` element (§7.2.4).
    SignerWithoutSignature,
    /// A `Signature` element is present but no `Signer` element (§7.2.5).
    SignatureWithoutSigner,
    /// A SidecarAsset Id is not present in the package AssetMap (§7.3.1).
    SidecarAssetNotFound,
    /// A CPLId within an AssociatedCPLList does not reference a known CPL (§7.3.1.1).
    CplNotFound,
    /// The same CPLId appears more than once within a single AssociatedCPLList (§7.3.1.1).
    DuplicateCplId,
}

impl ValidationCode for St2067_9_2018 {
    fn code(&self) -> &'static str {
        match self {
            Self::MalformedXml => "ST2067-9:2018:6.1/MalformedXml",
            Self::SidecarAssetReferencedByVirtualTrack => {
                "ST2067-9:2018:5/SidecarAssetReferencedByVirtualTrack"
            }
            Self::DuplicateAssetId => "ST2067-9:2018:7.2.3/DuplicateAssetId",
            Self::SignerWithoutSignature => "ST2067-9:2018:7.2.4/SignerWithoutSignature",
            Self::SignatureWithoutSigner => "ST2067-9:2018:7.2.5/SignatureWithoutSigner",
            Self::SidecarAssetNotFound => "ST2067-9:2018:7.3.1/SidecarAssetNotFound",
            Self::CplNotFound => "ST2067-9:2018:7.3.1.1/CplNotFound",
            Self::DuplicateCplId => "ST2067-9:2018:7.3.1.1/DuplicateCplId",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::MalformedXml =>
                "SidecarCompositionMap document is not well-formed XML (§6.1).",
            Self::SidecarAssetReferencedByVirtualTrack =>
                "A sidecar asset shall not be referenced by any Virtual Track in a CPL (§5).",
            Self::DuplicateAssetId =>
                "Duplicate SidecarAsset Id within SidecarAssetList (§7.2.3).",
            Self::SignerWithoutSignature =>
                "Signer element is present but the required Signature element is absent (§7.2.4).",
            Self::SignatureWithoutSigner =>
                "Signature element is present but the required Signer element is absent (§7.2.5).",
            Self::SidecarAssetNotFound =>
                "SidecarAsset Id is not present in the package AssetMap (§7.3.1).",
            Self::CplNotFound =>
                "CPLId in AssociatedCPLList does not reference a known CPL in this package (§7.3.1.1).",
            Self::DuplicateCplId =>
                "Duplicate CPLId within a single AssociatedCPLList (§7.3.1.1).",
        }
    }

    fn default_severity(&self) -> Severity {
        match self {
            Self::MalformedXml => Severity::Critical,
            _ => Severity::Error,
        }
    }

    fn category(&self) -> Category {
        Category::Reference
    }
}

impl St2067_9_2018 {
    pub const ALL: &'static [Self] = &[
        Self::MalformedXml,
        Self::SidecarAssetReferencedByVirtualTrack,
        Self::DuplicateAssetId,
        Self::SignerWithoutSignature,
        Self::SignatureWithoutSigner,
        Self::SidecarAssetNotFound,
        Self::CplNotFound,
        Self::DuplicateCplId,
    ];
}

impl From<St2067_9_2018> for String {
    fn from(c: St2067_9_2018) -> String {
        c.code().to_string()
    }
}
