//! Rule table: SMPTE ST 2067-202 (ISXD Plug-in).
//!
//! Generated from the pre-C0 per-edition code enums; edit by hand from now on.

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::{row, AnyRule, Rule, Spec};
use crate::diagnostics::{Category, Severity};

crate::diagnostics::rule::editions! {
    /// Editions of SMPTE ST 2067-202 (ISXD Plug-in) imferno knows.
    pub enum St2067_202Edition for Spec::St2067_202 {
        V2023 => "2023",
    }
}

type E = St2067_202Edition;

/// ISXDDataEssenceDescriptor: NamespaceURI shall be present (§9.2 Table 5, Req).
pub static NamespaceUriMissing: Rule<E> = Rule {
    name: "NamespaceUriMissing",
    category: Category::Data,
    description: "ISXDDataEssenceDescriptor: NamespaceURI shall be present (§9.2 Table 5, Req).",
    example: Some("<ISXDDataEssenceDescriptor>  <!-- missing required <NamespaceURI>… --> </ISXDDataEssenceDescriptor>"),
    editions: &[
        row(E::V2023, "9.2", Severity::Error),
    ],
};

/// ISXDSequence shall contain at least one Resource.
pub static ISXDSequenceNoResources: Rule<E> = Rule {
    name: "ISXDSequenceNoResources",
    category: Category::Data,
    description: "ISXDSequence shall contain at least one Resource.",
    example: Some("<ISXDSequence><ResourceList/></ISXDSequence>"),
    editions: &[row(E::V2023, "6", Severity::Error)],
};

/// ISXDSequence Resource.SourceEncoding does not reference an ISXDDataEssenceDescriptor.
pub static ISXDSequenceSourceEncodingInvalid: Rule<E> = Rule {
    name: "ISXDSequenceSourceEncodingInvalid",
    category: Category::Data,
    description: "ISXDSequence Resource.SourceEncoding does not reference an ISXDDataEssenceDescriptor.",
    example: Some("<ISXDSequence>…<SourceEncoding>urn:uuid:…</SourceEncoding></ISXDSequence>  <!-- UUID doesn't resolve to an ISXDDataEssenceDescriptor -->"),
    editions: &[
        row(E::V2023, "6", Severity::Error),
    ],
};

/// All ISXD Track Files referenced by an ISXD Virtual Track shall have an identical NamespaceURI value (§6).
pub static NamespaceUriMismatch: Rule<E> = Rule {
    name: "NamespaceUriMismatch",
    category: Category::Data,
    description: "All ISXD Track Files referenced by an ISXD Virtual Track shall have an identical NamespaceURI value (§6).",
    example: Some("Two ISXDSequences with the same TrackId point at descriptors whose <NamespaceURI> differs."),
    editions: &[
        row(E::V2023, "6", Severity::Error),
    ],
};

/// The Edit Rate of an ISXD Virtual Track shall be equal to the Edit Rate of the Main Image Virtual Track (§6).
pub static EditRateMismatch: Rule<E> = Rule {
    name: "EditRateMismatch",
    category: Category::Data,
    description: "The Edit Rate of an ISXD Virtual Track shall be equal to the Edit Rate of the Main Image Virtual Track (§6).",
    example: Some("ISXDSequence Resource with <EditRate>48 1</EditRate> in a Composition whose Main Image Virtual Track runs at 24000/1001."),
    editions: &[
        row(E::V2023, "6", Severity::Error),
    ],
};

/// A Composition that references an ISXD Track File shall contain one or more ISXD Virtual Tracks (§6).
pub static ISXDVirtualTrackMissing: Rule<E> = Rule {
    name: "ISXDVirtualTrackMissing",
    category: Category::Data,
    description: "A Composition that references an ISXD Track File shall contain one or more ISXD Virtual Tracks (§6).",
    example: Some("A MainImageSequence Resource whose SourceEncoding resolves to an ISXDDataEssenceDescriptor, in a CPL with no ISXDSequence."),
    editions: &[
        row(E::V2023, "6", Severity::Error),
    ],
};

/// ISXDDataEssenceDescriptor: DataEssenceCoding shall be present (§9.3).
pub static DataEssenceCodingMissing: Rule<E> = Rule {
    name: "DataEssenceCodingMissing",
    category: Category::Data,
    description: "ISXDDataEssenceDescriptor: DataEssenceCoding shall be present (§9.3).",
    example: Some("<ISXDDataEssenceDescriptor>  <!-- no <DataEssenceCoding> UL --> </ISXDDataEssenceDescriptor>"),
    editions: &[
        row(E::V2023, "9.3", Severity::Error),
    ],
};

/// ISXDDataEssenceDescriptor: DataEssenceCoding shall be the UTF-8 Text Data Essence Coding UL (§9.3 Table 6).
pub static DataEssenceCodingInvalid: Rule<E> = Rule {
    name: "DataEssenceCodingInvalid",
    category: Category::Data,
    description: "ISXDDataEssenceDescriptor: DataEssenceCoding shall be the UTF-8 Text Data Essence Coding UL (§9.3 Table 6).",
    example: Some("<DataEssenceCoding>urn:smpte:ul:060e2b34.04010105.0e090607.01010103</DataEssenceCoding>  <!-- container UL instead of 0e090606 coding UL -->"),
    editions: &[
        row(E::V2023, "9.3", Severity::Error),
    ],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[
    &NamespaceUriMissing,
    &ISXDSequenceNoResources,
    &ISXDSequenceSourceEncodingInvalid,
    &NamespaceUriMismatch,
    &EditRateMismatch,
    &ISXDVirtualTrackMissing,
    &DataEssenceCodingMissing,
    &DataEssenceCodingInvalid,
];
