//! Rule table: SMPTE ST 429-9 (Asset Mapping and File Segmentation).
//!
//! Generated from the pre-C0 per-edition code enums; edit by hand from now on.

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::{row, AnyRule, Rule, Spec};
use crate::diagnostics::{Category, Severity};

crate::diagnostics::rule::editions! {
    /// Editions of SMPTE ST 429-9 (Asset Mapping and File Segmentation) imferno knows.
    pub enum St429_9Edition for Spec::St429_9 {
        V2014 => "2014",
    }
}

type E = St429_9Edition;

/// The AssetList of the Asset Map holds no Asset (the schema requires at
/// least one).
pub static AssetListEmpty: Rule<E> = Rule {
    name: "AssetListEmpty",
    category: Category::Structure,
    description:
        "The Asset Map's AssetList contains no Asset element (the §11 schema requires one or more).",
    example: Some("<AssetMap>…<AssetList></AssetList></AssetMap>"),
    editions: &[row(E::V2014, "11", Severity::Critical)
        .reason("A package whose Asset Map lists no asset cannot be ingested at all.")
        .verified()],
};

/// No volume-index document found in the package root.
pub static VolindexMissing: Rule<E> = Rule {
    name: "VolindexMissing",
    category: Category::Structure,
    description: "No volume-index document found in the package root.",
    example: Some("Package root contains ASSETMAP.xml and CPL/PKL XMLs but no VOLINDEX.xml."),
    editions: &[row(E::V2014, "7", Severity::Info).suspect()],
};

/// The VOLINDEX.xml document is not well-formed XML.
pub static MalformedXml: Rule<E> = Rule {
    name: "MalformedXml",
    category: Category::Structure,
    description: "The VOLINDEX.xml document is not well-formed XML.",
    example: Some("<VolumeIndex>1<!-- truncated --> — VOLINDEX.xml ends mid-element."),
    editions: &[row(E::V2014, "7", Severity::Error)],
};

/// Asset Map Path is not a relative-path reference without query or fragment (Basic Map Profile v2).
pub static PathNotRelative: Rule<E> = Rule {
    name: "PathNotRelative",
    category: Category::Structure,
    description: "Asset Map Path is not a relative-path reference without query or fragment (Basic Map Profile v2).",
    example: Some("<Path>/mnt/imp/video.mxf</Path>"),
    editions: &[
        row(E::V2014, "A.2", Severity::Error),
    ],
};

/// Asset Map Path segment uses characters other than a-z, A-Z, 0-9, dash, underscore and period (Basic Map Profile v2).
pub static PathCharacters: Rule<E> = Rule {
    name: "PathCharacters",
    category: Category::Structure,
    description: "Asset Map Path segment uses characters other than a-z, A-Z, 0-9, dash, underscore and period (Basic Map Profile v2).",
    example: Some("<Path>my video.mxf</Path>  <!-- space -->"),
    editions: &[
        row(E::V2014, "A.2", Severity::Error),
    ],
};

/// Asset Map Path or one of its segments is longer than 100 characters (Basic Map Profile v2).
pub static PathTooLong: Rule<E> = Rule {
    name: "PathTooLong",
    category: Category::Structure,
    description: "Asset Map Path or one of its segments is longer than 100 characters (Basic Map Profile v2).",
    example: Some("<Path>…111-character file name….mxf</Path>"),
    editions: &[
        row(E::V2014, "A.2", Severity::Error),
    ],
};

/// Asset Map Path has more than 10 segments (Basic Map Profile v2).
pub static PathTooManySegments: Rule<E> = Rule {
    name: "PathTooManySegments",
    category: Category::Structure,
    description: "Asset Map Path has more than 10 segments (Basic Map Profile v2).",
    example: Some("<Path>a/b/c/d/e/f/g/h/i/j/k.mxf</Path>"),
    editions: &[row(E::V2014, "A.2", Severity::Error)],
};

/// Two Asset Map Paths are identical regardless of case (Basic Map Profile v2).
pub static PathDuplicate: Rule<E> = Rule {
    name: "PathDuplicate",
    category: Category::Structure,
    description: "Two Asset Map Paths are identical regardless of case (Basic Map Profile v2).",
    example: Some("<Path>Video.mxf</Path> and <Path>video.mxf</Path>"),
    editions: &[row(E::V2014, "A.2", Severity::Error)],
};

/// The Asset Map file is not named exactly ASSETMAP.xml (Basic Map Profile v2).
pub static AssetMapFileName: Rule<E> = Rule {
    name: "AssetMapFileName",
    category: Category::Structure,
    description: "The Asset Map file is not named exactly ASSETMAP.xml (Basic Map Profile v2).",
    example: Some("Package root holds assetmap.xml instead of ASSETMAP.xml"),
    editions: &[row(E::V2014, "A.4", Severity::Error)],
};

/// AssetMap declares no asset as a PackingList (ST 429-9 §6.3).
pub static AssetMapHasNoPackingList: Rule<E> = Rule {
    name: "AssetMapHasNoPackingList",
    category: Category::Structure,
    description: "AssetMap declares no asset as a PackingList (ST 429-9 §6.3).",
    example: Some("AssetMap whose `<Asset>` entries all omit `<PackingList>true</PackingList>`"),
    editions: &[row(E::V2014, "6.3", Severity::Critical).reason(
        "Prevents the package or file from being processed further (pre-C0 catalogue severity).",
    )],
};

/// PKL document Id is not declared as a PackingList asset in the AssetMap.
pub static PklIdNotInAssetMap: Rule<E> = Rule {
    name: "PklIdNotInAssetMap",
    category: Category::Reference,
    description: "PKL document Id is not declared as a PackingList asset in the AssetMap.",
    example: Some(
        "PKL `<Id>urn:uuid:abc…</Id>` not present in the AssetMap as a packing-list asset",
    ),
    editions: &[row(E::V2014, "6.3", Severity::Error)],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[
    &AssetListEmpty,
    &VolindexMissing,
    &MalformedXml,
    &PathNotRelative,
    &PathCharacters,
    &PathTooLong,
    &PathTooManySegments,
    &PathDuplicate,
    &AssetMapFileName,
    &AssetMapHasNoPackingList,
    &PklIdNotInAssetMap,
];
