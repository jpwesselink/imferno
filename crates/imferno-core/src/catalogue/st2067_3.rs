//! Rule table: SMPTE ST 2067-3 (Composition Playlist).
//!
//! Generated from the pre-C0 per-edition code enums; edit by hand from now on.

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::{row, AnyRule, Rule, Spec};
use crate::diagnostics::{Category, Severity};

crate::diagnostics::rule::editions! {
    /// Editions of SMPTE ST 2067-3 (Composition Playlist) imferno knows.
    pub enum St2067_3Edition for Spec::St2067_3 {
        V2013 => "2013",
        V2016 => "2016",
    }
}

type E = St2067_3Edition;

/// ContentKind uses an unrecognized value under the SMPTE scope.
pub static ContentKindUnknown: Rule<E> = Rule {
    name: "ContentKindUnknown",
    category: Category::Metadata,
    description: "ContentKind uses an unrecognized value under the SMPTE scope.",
    example: Some("<ContentKind scope=\"http://www.smpte-ra.org/schemas/2067-3/2013/content-kind\">Banana</ContentKind>"),
    editions: &[
        row(E::V2013, "5.5.1.2", Severity::Warning).suspect(),
        row(E::V2016, "5.5.1.2", Severity::Warning).suspect(),
    ],
};

/// SourceEncoding present but EssenceDescriptorList absent.
pub static SourceEncodingNoEssenceDescriptorList: Rule<E> = Rule {
    name: "SourceEncodingNoEssenceDescriptorList",
    category: Category::Reference,
    description: "SourceEncoding present but EssenceDescriptorList absent.",
    example: Some("<Resource><SourceEncoding>urn:uuid:…</SourceEncoding></Resource>  <!-- but the CPL has no <EssenceDescriptorList> -->"),
    editions: &[
        row(E::V2013, "6.4.2", Severity::Error),
        row(E::V2016, "6.4.2", Severity::Error),
    ],
};

/// SourceEncoding does not match any EssenceDescriptor Id.
pub static SourceEncodingUnresolved: Rule<E> = Rule {
    name: "SourceEncodingUnresolved",
    category: Category::Reference,
    description: "SourceEncoding does not match any EssenceDescriptor Id.",
    example: Some("<SourceEncoding>urn:uuid:aaaa…</SourceEncoding>  <!-- no EssenceDescriptor with that Id -->"),
    editions: &[
        row(E::V2013, "6.4.2", Severity::Error),
        row(E::V2016, "6.4.2", Severity::Error),
    ],
};

/// EssenceDescriptorList present but contains no descriptors.
pub static EssenceDescriptorListEmpty: Rule<E> = Rule {
    name: "EssenceDescriptorListEmpty",
    category: Category::Structure,
    description: "EssenceDescriptorList present but contains no descriptors.",
    example: Some("<EssenceDescriptorList></EssenceDescriptorList>"),
    editions: &[
        row(E::V2013, "6.4.2", Severity::Error),
        row(E::V2016, "6.4.2", Severity::Error),
    ],
};

/// ContentVersionList present but empty.
pub static ContentVersionListEmpty: Rule<E> = Rule {
    name: "ContentVersionListEmpty",
    category: Category::Structure,
    description: "ContentVersionList present but empty.",
    example: Some("<ContentVersionList></ContentVersionList>"),
    editions: &[
        row(E::V2013, "6.11", Severity::Error),
        row(E::V2016, "6.11", Severity::Error),
    ],
};

/// ContentVersion/Id is empty (shall be a URI).
pub static ContentVersionIdInvalid: Rule<E> = Rule {
    name: "ContentVersionIdInvalid",
    category: Category::Metadata,
    description: "ContentVersion/Id is empty (shall be a URI).",
    example: Some("<ContentVersion><Id></Id>…</ContentVersion>"),
    editions: &[
        row(E::V2013, "6.11", Severity::Error),
        row(E::V2016, "6.11", Severity::Error),
    ],
};

/// ContentVersion/LabelText is absent.
pub static ContentVersionLabelTextMissing: Rule<E> = Rule {
    name: "ContentVersionLabelTextMissing",
    category: Category::Metadata,
    description: "ContentVersion/LabelText is absent.",
    example: Some("<ContentVersion><Id>urn:uuid:…</Id></ContentVersion>  <!-- no <LabelText> -->"),
    editions: &[
        row(E::V2013, "6.11", Severity::Warning),
        row(E::V2016, "6.11", Severity::Warning),
    ],
};

/// Locale language tag does not conform to RFC 5646.
pub static LocaleLanguageTagInvalid: Rule<E> = Rule {
    name: "LocaleLanguageTagInvalid",
    category: Category::Metadata,
    description: "Locale language tag does not conform to RFC 5646.",
    example: Some("<Locale><LanguageList><Language>english</Language></LanguageList></Locale>  <!-- not RFC 5646 -->"),
    editions: &[
        row(E::V2013, "6.12", Severity::Warning),
        row(E::V2016, "6.12", Severity::Warning),
    ],
};

/// TrackId is not unique within a segment.
pub static TrackIdNotUnique: Rule<E> = Rule {
    name: "TrackIdNotUnique",
    category: Category::Structure,
    description: "TrackId is not unique within a segment.",
    example: Some("<Sequence><TrackId>urn:uuid:abc…</TrackId>…</Sequence><Sequence><TrackId>urn:uuid:abc…</TrackId>…</Sequence>  <!-- same TrackId in two sequences of one segment -->"),
    editions: &[
        row(E::V2013, "7.3", Severity::Error),
        row(E::V2016, "7.3", Severity::Error),
    ],
};

/// Marker offset exceeds the resource IntrinsicDuration.
pub static MarkerOffsetOutOfRange: Rule<E> = Rule {
    name: "MarkerOffsetOutOfRange",
    category: Category::Timing,
    description: "Marker offset exceeds the resource IntrinsicDuration.",
    example: Some("<Marker><Offset>120</Offset></Marker>  <!-- resource IntrinsicDuration=100 -->"),
    editions: &[
        row(E::V2013, "7.4", Severity::Error).suspect(),
        row(E::V2016, "7.4", Severity::Error).suspect(),
    ],
};

/// Marker label is not a recognized SMPTE standard value.
pub static MarkerLabelUnknown: Rule<E> = Rule {
    name: "MarkerLabelUnknown",
    category: Category::Metadata,
    description: "Marker label is not a recognized SMPTE standard value.",
    example: Some("<Marker><Label>NotARealMarker</Label></Marker>"),
    editions: &[
        row(E::V2013, "7.4", Severity::Warning).suspect(),
        row(E::V2016, "7.4", Severity::Warning).suspect(),
    ],
};

/// All virtual tracks in a segment must span the same number of edit units.
pub static SegmentDuration: Rule<E> = Rule {
    name: "SegmentDuration",
    category: Category::Timing,
    description: "All virtual tracks in a segment must span the same number of edit units.",
    example: Some(
        "Segment with MainImageSequence duration = 24f and MainAudioSequence duration = 25f",
    ),
    editions: &[
        row(E::V2013, "7.2.2", Severity::Error),
        row(E::V2016, "7.2.2", Severity::Error).suspect(),
    ],
};

/// No two ContentVersion elements shall have identical Id values.
pub static ContentVersionIdDuplicate: Rule<E> = Rule {
    name: "ContentVersionIdDuplicate",
    category: Category::Structure,
    description: "No two ContentVersion elements shall have identical Id values.",
    example: Some("<ContentVersion><Id>urn:uuid:a…</Id></ContentVersion><ContentVersion><Id>urn:uuid:a…</Id></ContentVersion>"),
    editions: &[
        row(E::V2013, "6.1.9", Severity::Error),
        row(E::V2016, "6.1.9", Severity::Error),
    ],
};

/// Sequence duration shall be an integer number of Composition Edit Units.
pub static SegmentDurationIntegerEditUnits: Rule<E> = Rule {
    name: "SegmentDurationIntegerEditUnits",
    category: Category::Timing,
    description: "Sequence duration shall be an integer number of Composition Edit Units.",
    example: Some("Sequence duration = 12.5 edit units (must be an integer multiple of the Composition Edit Unit)"),
    editions: &[
        row(E::V2013, "7.3", Severity::Error),
        row(E::V2016, "7.3", Severity::Error),
    ],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[
    &ContentKindUnknown,
    &SourceEncodingNoEssenceDescriptorList,
    &SourceEncodingUnresolved,
    &EssenceDescriptorListEmpty,
    &ContentVersionListEmpty,
    &ContentVersionIdInvalid,
    &ContentVersionLabelTextMissing,
    &LocaleLanguageTagInvalid,
    &TrackIdNotUnique,
    &MarkerOffsetOutOfRange,
    &MarkerLabelUnknown,
    &SegmentDuration,
    &ContentVersionIdDuplicate,
    &SegmentDurationIntegerEditUnits,
];
