//! Rule table: SMPTE ST 377-1 (MXF File Format).
//!
//! Every row checked against the text of its edition (C1/C4; 2019 rows C17).

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::Strength::Shall;
use crate::diagnostics::rule::{checked, AnyRule, Rule, Spec};
use crate::diagnostics::{Category, Severity};

crate::diagnostics::rule::editions! {
    /// Editions of SMPTE ST 377-1 (MXF File Format) imferno knows.
    pub enum St377_1Edition for Spec::St377_1 {
        V2011 => "2011",
        V2019 => "2019",
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
        checked(E::V2011, "6.1.1", Shall),
        checked(E::V2019, "6.1.1", Shall),
    ],
};

/// MXF file contains no essence containers.
pub static NoEssenceContainers: Rule<E> = Rule {
    name: "NoEssenceContainers",
    category: Category::Encoding,
    description: "MXF file contains no essence containers.",
    example: Some("An MXF file whose header metadata declares no EssenceContainer ULs — the body has no decodable essence."),
    editions: &[
        checked(E::V2011, "7.1", Shall).severity(Severity::Warning).reason("§7.1 relaxes completeness to should while the Partition is Open, and the check does not test the partition status or whether the file carries internal essence."),
        checked(E::V2019, "7.1", Shall).severity(Severity::Warning).reason("§7.1 relaxes completeness to should while the Partition is Open, and the check does not test the partition status or whether the file carries internal essence."),
    ],
};

/// The first partition in an MXF file must be the Header partition.
pub static NonHeaderFirstPartition: Rule<E> = Rule {
    name: "NonHeaderFirstPartition",
    category: Category::Container,
    description: "The first partition in an MXF file must be the Header partition.",
    example: Some("An authoring tool wrote a Body partition before the Header partition, violating ST 377-1 ordering."),
    editions: &[
        checked(E::V2011, "6.2.5", Shall),
        checked(E::V2019, "6.2.5", Shall),
    ],
};

/// The header partition is Open and no Closed footer partition carries header metadata, so the MXF file is Open.
pub static HeaderPartitionOpen: Rule<E> = Rule {
    name: "HeaderPartitionOpen",
    category: Category::Container,
    description: "The header partition is Open and no Closed footer partition carries header metadata, so the MXF file is Open.",
    example: Some("A live-streaming export shipped an OpenIncomplete Header partition and no Closed Footer partition with header metadata."),
    editions: &[
        checked(E::V2011, "6.2.4", Shall).severity(Severity::Warning).reason("§6.2.4 lets a file be Open while it is being created or received, which a validator cannot tell from the file.").interpret("The header metadata is read from a Closed footer partition when the header is Open; only a file with neither is Open.", Some("https://github.com/cinecert/asdcplib/issues/158")),
        checked(E::V2019, "6.2.4", Shall).severity(Severity::Warning).reason("§6.2.4 lets a file be Open while it is being created or received, which a validator cannot tell from the file.").interpret("The header metadata is read from a Closed footer partition when the header is Open; only a file with neither is Open.", Some("https://github.com/cinecert/asdcplib/issues/158")),
    ],
};

/// The header partition must carry header metadata (HeaderByteCount > 0).
pub static MissingHeaderMetadata: Rule<E> = Rule {
    name: "MissingHeaderMetadata",
    category: Category::Container,
    description: "The header partition must carry header metadata (HeaderByteCount > 0).",
    example: Some("An MXF file whose Header partition advertises HeaderByteCount = 0 — no metadata sets follow the partition pack."),
    editions: &[
        checked(E::V2011, "6.1.1", Shall),
        checked(E::V2019, "6.1.1", Shall),
    ],
};

/// A Structural Component on a Timeline Track has no Duration; it is a Best Effort property there and shall be encoded (-1 when unknown).
pub static StructuralComponentDurationMissing: Rule<E> = Rule {
    name: "StructuralComponentDurationMissing",
    category: Category::Container,
    description: "A Structural Component on a Timeline Track has no Duration; it is a Best Effort property there and shall be encoded (-1 when unknown).",
    example: Some("A timed-text track file whose Source Clip on the Timeline Track carries no Duration item."),
    editions: &[
        checked(E::V2011, "B.8", Shall).interpret("Duration is a Best Effort property on a Timeline Track (encoders shall encode it, -1 when unknown); the editorial issue on its Opt marking is open.", Some("https://github.com/SMPTE/ST377-1-full-revision/issues/18")),
        checked(E::V2019, "B.8", Shall).interpret("Duration is a Best Effort property on a Timeline Track (encoders shall encode it, -1 when unknown); the editorial issue on its Opt marking is open.", Some("https://github.com/SMPTE/ST377-1-full-revision/issues/18")),
    ],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[
    &NotMxf,
    &NoEssenceContainers,
    &NonHeaderFirstPartition,
    &HeaderPartitionOpen,
    &MissingHeaderMetadata,
    &StructuralComponentDurationMissing,
];
