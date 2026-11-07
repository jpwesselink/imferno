//! Rule table: imferno engine.
//!
//! Generated from the pre-C0 per-edition code enums; edit by hand from now on.

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::{row, AnyRule, Rule, Spec};
use crate::diagnostics::{Category, Severity};

crate::diagnostics::rule::editions! {
    /// Editions of imferno engine imferno knows.
    pub enum ImfernoEdition for Spec::Imferno {
        Current => "",
    }
}

type E = ImfernoEdition;

/// MXF file could not be opened for reading.
pub static OpenFailed: Rule<E> = Rule {
    name: "OpenFailed",
    category: Category::Container,
    description: "MXF file could not be opened for reading.",
    example: Some("fs::File::open(\"audio1.mxf\") returned Err (file missing or unreadable)"),
    editions: &[row(E::Current, "Mxf", Severity::Critical).reason(
        "Prevents the package or file from being processed further (pre-C0 catalogue severity).",
    )],
};

/// MXF header partition pack failed to parse.
pub static PartitionPackParseFailed: Rule<E> = Rule {
    name: "PartitionPackParseFailed",
    category: Category::Container,
    description: "MXF header partition pack failed to parse.",
    example: Some("Header partition pack KLV is malformed (e.g. BER length undershoots the minimum 88-byte body)"),
    editions: &[
        row(E::Current, "Mxf", Severity::Critical)
            .reason("Prevents the package or file from being processed further (pre-C0 catalogue severity)."),
    ],
};

/// MXF header metadata could not be converted to RegXML.
pub static RegXmlConversionFailed: Rule<E> = Rule {
    name: "RegXmlConversionFailed",
    category: Category::Container,
    description: "MXF header metadata could not be converted to RegXML.",
    example: Some(
        "smpte-mxf could not convert header metadata to RegXML (e.g. unknown KLV set in header)",
    ),
    editions: &[row(E::Current, "Mxf", Severity::Warning)],
};

/// Informational trace of how many essence containers the MXF declares.
pub static EssenceContainersDetected: Rule<E> = Rule {
    name: "EssenceContainersDetected",
    category: Category::Container,
    description: "Informational trace of how many essence containers the MXF declares.",
    example: Some("Header partition pack declares N essence container ULs — info notice surfaces the count for traceability"),
    editions: &[
        row(E::Current, "Mxf", Severity::Info),
    ],
};

/// An I/O error prevented the asset from being read.
pub static IoError: Rule<E> = Rule {
    name: "IoError",
    category: Category::Asset,
    description: "An I/O error prevented the asset from being read.",
    example: Some("Underlying filesystem refused the read (permissions, device error, etc.)"),
    editions: &[row(E::Current, "General", Severity::Error)],
};

/// Asset is present in the AssetMap but not referenced by any CPL Virtual Track and has no SCM declaration. Likely a sidecar essence without an SCM.
pub static UnreferencedAsset: Rule<E> = Rule {
    name: "UnreferencedAsset",
    category: Category::Structure,
    description: "Asset is present in the AssetMap but not referenced by any CPL Virtual Track and has no SCM declaration. Likely a sidecar essence without an SCM.",
    example: Some("ASSETMAP lists urn:uuid:abc… as an Asset, but no CPL Virtual Track resource and no SCM Asset entry references it."),
    editions: &[
        row(E::Current, "Package", Severity::Info),
    ],
};

/// File is present in the package directory but not listed in the AssetMap. The file is invisible to any conforming IMF reader.
pub static UnlistedEssence: Rule<E> = Rule {
    name: "UnlistedEssence",
    category: Category::Structure,
    description: "File is present in the package directory but not listed in the AssetMap. The file is invisible to any conforming IMF reader.",
    example: Some("BONUS_AUDIO.mxf sits next to ASSETMAP.xml on disk but does not appear in any <Asset> entry of ASSETMAP.xml."),
    editions: &[
        row(E::Current, "Package", Severity::Info),
    ],
};

/// IMF package failed to parse due to a structural error.
pub static ParseError: Rule<E> = Rule {
    name: "ParseError",
    category: Category::Structure,
    description: "IMF package failed to parse due to a structural error.",
    example: Some("Top-level error parsing ASSETMAP.xml — e.g. missing root <AssetMap> element or truncated stream."),
    editions: &[
        row(E::Current, "Package", Severity::Critical)
            .reason("Prevents the package or file from being processed further (pre-C0 catalogue severity)."),
    ],
};

/// A Packing List referenced by the AssetMap could not be parsed.
pub static PklParseError: Rule<E> = Rule {
    name: "PklParseError",
    category: Category::Structure,
    description: "A Packing List referenced by the AssetMap could not be parsed.",
    example: Some("PKL_<uuid>.xml is referenced from ASSETMAP but is malformed XML or violates the PKL schema."),
    editions: &[
        row(E::Current, "Package", Severity::Error),
    ],
};

/// An XML asset could not be parsed as CPL, OPL, or SCM.
pub static XmlAssetParseError: Rule<E> = Rule {
    name: "XmlAssetParseError",
    category: Category::Structure,
    description: "An XML asset could not be parsed as CPL, OPL, or SCM.",
    example: Some("An asset listed with a .xml suffix doesn't parse as CPL, OPL, or SCM (likely a misnamed file or a sidecar XML)."),
    editions: &[
        row(E::Current, "Package", Severity::Warning),
    ],
};

/// An XML file could not be read from disk.
pub static XmlReadError: Rule<E> = Rule {
    name: "XmlReadError",
    category: Category::Structure,
    description: "An XML file could not be read from disk.",
    example: Some("ASSETMAP.xml exists in the manifest but couldn't be opened (permission denied, network read failure, etc.)."),
    editions: &[
        row(E::Current, "Package", Severity::Warning),
    ],
};

/// Could not scan the package directory.
pub static ReadDirError: Rule<E> = Rule {
    name: "ReadDirError",
    category: Category::Structure,
    description: "Could not scan the package directory.",
    example: Some(
        "The package root directory could not be enumerated (permission denied or path missing).",
    ),
    editions: &[row(E::Current, "Package", Severity::Info)],
};

/// Could not read a directory entry while scanning for unlisted essences.
pub static DirEntryError: Rule<E> = Rule {
    name: "DirEntryError",
    category: Category::Structure,
    description: "Could not read a directory entry while scanning for unlisted essences.",
    example: Some(
        "A specific directory entry returned an error during scanning (often a broken symlink).",
    ),
    editions: &[row(E::Current, "Package", Severity::Info)],
};

/// An asset chunk path attempts to escape the package root directory (path traversal).
pub static PathTraversal: Rule<E> = Rule {
    name: "PathTraversal",
    category: Category::Structure,
    description: "An asset chunk path attempts to escape the package root directory (path traversal).",
    example: Some("<Chunk><Path>../../etc/passwd</Path></Chunk> — the asset path resolves outside the package root."),
    editions: &[
        row(E::Current, "Package", Severity::Error),
    ],
};

/// Application-profile validation was not performed for the CPL: ApplicationIdentification is absent or lists URIs imferno has no validator for.
pub static ApplicationProfileNotValidated: Rule<E> = Rule {
    name: "ApplicationProfileNotValidated",
    category: Category::Structure,
    description: "Application-profile validation was not performed for the CPL: ApplicationIdentification is absent or lists URIs imferno has no validator for.",
    example: Some("<ExtensionProperties> without <ApplicationIdentification>, or listing only http://example.org/unknown-app"),
    editions: &[
        row(E::Current, "Package", Severity::Warning),
    ],
};

/// Internal: a check reported a rule against an edition whose table row says the edition does not contain it. The finding was dropped; please report this as an imferno bug.
pub static RuleNotInEdition: Rule<E> = Rule {
    name: "RuleNotInEdition",
    category: Category::Structure,
    description: "Internal: a check reported a rule against an edition whose table row says the edition does not contain it. The finding was dropped; please report this as an imferno bug.",
    example: Some("IMFERNO:Engine/RuleNotInEdition — ST2067-21/HtIrvParameterBMax was reported for ST2067-21 2014, which does not contain it"),
    editions: &[
        row(E::Current, "Engine", Severity::Warning),
    ],
};

/// The ST 2067-2 edition of a package could not be determined from its
/// Packing Lists and CPLs; package-level rules were checked against the
/// edition named in the message.
pub static EditionAssumed: Rule<E> = Rule {
    name: "EditionAssumed",
    category: Category::Structure,
    description: "The package's ST 2067-2 edition could not be determined from its Packing List and CPL namespaces; package-level rules were checked against the edition named in the message (the newest candidate).",
    example: Some("Packing List in the ST 2067-2:2016 PKL namespace (used by 2016 and 2020) and no CPL declaring a Core Constraints namespace"),
    editions: &[row(E::Current, "Package", Severity::Info)],
};

/// The package is an ST 2067-2:2013 IMP: that edition defines no delivery
/// requirements, so delivery rules of later editions were not checked.
pub static DeliveryNotChecked: Rule<E> = Rule {
    name: "DeliveryNotChecked",
    category: Category::Structure,
    description: "ST 2067-2:2013 defines no delivery requirements (an IMP is a Packing List and its assets, §7); the delivery rules of later editions (Asset Map ingest, ASSETMAP.xml name, Annex A paths, VOLINDEX, asset size and hash at ingest) were not checked.",
    example: Some("ST 429-8 Packing List with CPLs in the ST 2067-3:2013 namespace"),
    editions: &[row(E::Current, "Package", Severity::Info)],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[
    &EditionAssumed,
    &DeliveryNotChecked,
    &OpenFailed,
    &PartitionPackParseFailed,
    &RegXmlConversionFailed,
    &EssenceContainersDetected,
    &IoError,
    &UnreferencedAsset,
    &UnlistedEssence,
    &ParseError,
    &PklParseError,
    &XmlAssetParseError,
    &XmlReadError,
    &ReadDirError,
    &DirEntryError,
    &PathTraversal,
    &ApplicationProfileNotValidated,
    &RuleNotInEdition,
];
