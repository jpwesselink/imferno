//! Rule table: SMPTE ST 2067-9 (Sidecar Composition Map).
//!
//! Every row checked against the text of its edition (C1/C4).

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::Strength::Shall;
use crate::diagnostics::rule::{checked, AnyRule, Rule, Spec};
use crate::diagnostics::{Category, Severity};

crate::diagnostics::rule::editions! {
    /// Editions of SMPTE ST 2067-9 (Sidecar Composition Map) imferno knows.
    pub enum St2067_9Edition for Spec::St2067_9 {
        V2018 => "2018",
    }
}

type E = St2067_9Edition;

/// SidecarCompositionMap document is not well-formed XML (§6.1).
pub static MalformedXml: Rule<E> = Rule {
    name: "MalformedXml",
    category: Category::Reference,
    description: "SidecarCompositionMap document is not well-formed XML (§6.1).",
    example: Some("SCM_<uuid>.xml ends mid-element or has mismatched tags."),
    editions: &[checked(E::V2018, "6.1", Shall)
        .severity(Severity::Critical)
        .reason("A Sidecar Composition Map that is not XML cannot be processed further.")],
};

/// A sidecar asset shall not be referenced by any Virtual Track in a CPL (§5).
pub static SidecarAssetReferencedByVirtualTrack: Rule<E> = Rule {
    name: "SidecarAssetReferencedByVirtualTrack",
    category: Category::Reference,
    description: "A sidecar asset shall not be referenced by any Virtual Track in a CPL (§5).",
    example: Some(
        "A CPL <Resource><TrackFileId> points at a sidecar asset declared in an SCM document.",
    ),
    editions: &[checked(E::V2018, "5", Shall)],
};

/// Duplicate SidecarAsset Id within SidecarAssetList (§7.2.3).
pub static DuplicateAssetId: Rule<E> = Rule {
    name: "DuplicateAssetId",
    category: Category::Reference,
    description: "Duplicate SidecarAsset Id within SidecarAssetList (§7.2.3).",
    example: Some("Two <SidecarAsset> entries in one SCM share the same <Id>urn:uuid:abc…</Id>."),
    editions: &[checked(E::V2018, "7.2.3", Shall)],
};

/// Signer element is present but the required Signature element is absent (§7.2.4).
pub static SignerWithoutSignature: Rule<E> = Rule {
    name: "SignerWithoutSignature",
    category: Category::Reference,
    description: "Signer element is present but the required Signature element is absent (§7.2.4).",
    example: Some("<Signer>…</Signer> appears but no sibling <Signature> follows."),
    editions: &[checked(E::V2018, "7.2.4", Shall)],
};

/// Signature element is present but the required Signer element is absent (§7.2.5).
pub static SignatureWithoutSigner: Rule<E> = Rule {
    name: "SignatureWithoutSigner",
    category: Category::Reference,
    description: "Signature element is present but the required Signer element is absent (§7.2.5).",
    example: Some("<Signature>…</Signature> appears without a preceding <Signer>."),
    editions: &[checked(E::V2018, "7.2.5", Shall)],
};

/// SidecarAsset Id is not present in the package AssetMap (§7.3.1).
pub static SidecarAssetNotFound: Rule<E> = Rule {
    name: "SidecarAssetNotFound",
    category: Category::Reference,
    description: "SidecarAsset Id is not present in the package AssetMap (§7.3.1).",
    example: Some("<SidecarAsset><Id>urn:uuid:abc…</Id> references a UUID that is not present in ASSETMAP.xml."),
    editions: &[
        checked(E::V2018, "7.3.1", Shall),
    ],
};

/// CPLId in AssociatedCPLList does not reference a known CPL in this package (§7.3.1.1).
pub static CplNotFound: Rule<E> = Rule {
    name: "CplNotFound",
    category: Category::Reference,
    description: "CPLId in AssociatedCPLList does not reference a known CPL in this package (§7.3.1.1).",
    example: Some("<AssociatedCPLList><CPLId>urn:uuid:xyz…</CPLId> points at a CPL that isn't in the package."),
    editions: &[
        checked(E::V2018, "7.3.1.1", Shall).severity(Severity::Warning).reason("§7.3.1.1 requires the CPLId to be the Id of an associated Composition, not of one delivered in the same package; a CPL absent from this package may live in another."),
    ],
};

/// Duplicate CPLId within a single AssociatedCPLList (§7.3.1.1).
pub static DuplicateCplId: Rule<E> = Rule {
    name: "DuplicateCplId",
    category: Category::Reference,
    description: "Duplicate CPLId within a single AssociatedCPLList (§7.3.1.1).",
    example: Some("An <AssociatedCPLList> contains the same <CPLId>urn:uuid:xyz…</CPLId> twice."),
    editions: &[checked(E::V2018, "7.3.1.1", Shall)],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[
    &MalformedXml,
    &SidecarAssetReferencedByVirtualTrack,
    &DuplicateAssetId,
    &SignerWithoutSignature,
    &SignatureWithoutSigner,
    &SidecarAssetNotFound,
    &CplNotFound,
    &DuplicateCplId,
];
