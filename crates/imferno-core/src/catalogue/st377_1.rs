//! Rule table: SMPTE ST 377-1 (MXF File Format).
//!
//! Generated from the pre-C0 per-edition code enums; edit by hand from now on.

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::{row, AnyRule, Rule, Spec};
use crate::diagnostics::{Category, Severity};

crate::diagnostics::rule::editions! {
    /// Editions of SMPTE ST 377-1 (MXF File Format) imferno knows.
    pub enum St377_1Edition for Spec::St377_1 {
        V2011 => "2011",
    }
}

type E = St377_1Edition;

/// File is not a valid MXF container.
pub static NotMxf: Rule<E> = Rule {
    name: "NotMxf",
    category: Category::Asset,
    description: "File is not a valid MXF container.",
    example: Some("An asset declared in the PKL with MIME `application/mxf` whose header doesn't carry the MXF run-in / partition pack key."),
    editions: &[
        row(E::V2011, "5", Severity::Warning).suspect(),
    ],
};

/// MXF file could not be parsed; it may be truncated or corrupt.
pub static ParseError: Rule<E> = Rule {
    name: "ParseError",
    category: Category::Asset,
    description: "MXF file could not be parsed; it may be truncated or corrupt.",
    example: Some("An MXF file that was truncated mid-transfer; the footer partition is missing or the RIP is unreadable."),
    editions: &[
        row(E::V2011, "5", Severity::Warning).suspect(),
    ],
};

/// MXF file contains no essence containers.
pub static NoEssenceContainers: Rule<E> = Rule {
    name: "NoEssenceContainers",
    category: Category::Encoding,
    description: "MXF file contains no essence containers.",
    example: Some("An MXF file whose header metadata declares no EssenceContainer ULs — the body has no decodable essence."),
    editions: &[
        row(E::V2011, "11", Severity::Warning).suspect(),
    ],
};

/// MXF operational pattern must be OP1a for IMF packages.
pub static Op1a: Rule<E> = Rule {
    name: "Op1a",
    category: Category::Encoding,
    description: "MXF operational pattern must be OP1a for IMF packages.",
    example: Some(
        "An MXF file whose Preface declares OperationalPattern OP-Atom or OP-3a instead of OP-1a.",
    ),
    editions: &[row(E::V2011, "7", Severity::Error).suspect()],
};

/// The first partition in an MXF file must be the Header partition.
pub static NonHeaderFirstPartition: Rule<E> = Rule {
    name: "NonHeaderFirstPartition",
    category: Category::Container,
    description: "The first partition in an MXF file must be the Header partition.",
    example: Some("An authoring tool wrote a Body partition before the Header partition, violating ST 377-1 ordering."),
    editions: &[
        row(E::V2011, "6.4", Severity::Error).suspect(),
    ],
};

/// The header partition is Open and no Closed footer partition carries header metadata, so the MXF file is Open.
pub static HeaderPartitionOpen: Rule<E> = Rule {
    name: "HeaderPartitionOpen",
    category: Category::Container,
    description: "The header partition is Open and no Closed footer partition carries header metadata, so the MXF file is Open.",
    example: Some("A live-streaming export shipped an OpenIncomplete Header partition and no Closed Footer partition with header metadata."),
    editions: &[
        row(E::V2011, "8.3.3", Severity::Warning).suspect(),
    ],
};

/// The header partition must carry header metadata (HeaderByteCount > 0).
pub static MissingHeaderMetadata: Rule<E> = Rule {
    name: "MissingHeaderMetadata",
    category: Category::Container,
    description: "The header partition must carry header metadata (HeaderByteCount > 0).",
    example: Some("An MXF file whose Header partition advertises HeaderByteCount = 0 — no metadata sets follow the partition pack."),
    editions: &[
        row(E::V2011, "8.3.3", Severity::Error).suspect(),
    ],
};

/// A Structural Component on a Timeline Track has no Duration; it is a Best Effort property there and shall be encoded (-1 when unknown).
pub static StructuralComponentDurationMissing: Rule<E> = Rule {
    name: "StructuralComponentDurationMissing",
    category: Category::Container,
    description: "A Structural Component on a Timeline Track has no Duration; it is a Best Effort property there and shall be encoded (-1 when unknown).",
    example: Some("A timed-text track file whose Source Clip on the Timeline Track carries no Duration item."),
    editions: &[
        row(E::V2011, "B.8", Severity::Error).suspect(),
    ],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[
    &NotMxf,
    &ParseError,
    &NoEssenceContainers,
    &Op1a,
    &NonHeaderFirstPartition,
    &HeaderPartitionOpen,
    &MissingHeaderMetadata,
    &StructuralComponentDurationMissing,
];
