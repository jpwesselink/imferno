//! Rule table: SMPTE ST 2067-3 (Composition Playlist).
//!
//! Every row checked against the text of its edition (C1/C4).

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::Strength::Shall;
use crate::diagnostics::rule::{checked, AnyRule, Rule, Spec};
use crate::diagnostics::{Category, Severity};

/// Kept below the strength's Error: see the C1 report.
const CONTENT_KIND_REASON: &str = "Kept at Warning: widely exchanged packages, including the Netflix Meridian reference IMP, carry capitalised values such as \"Test\"; an Error would fail them on a display-only field.";

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
    example: Some("<ContentKind>Banana</ContentKind>  <!-- default scope: Table 3 values only -->"),
    editions: &[
        checked(E::V2013, "6.1.8", Shall).severity(Severity::Warning).reason(CONTENT_KIND_REASON).interpret("Exact, case-sensitive match against Table 3 under the default scope; other scopes are out of scope.", Some("https://github.com/Netflix/photon/issues/361")),
        checked(E::V2016, "6.1.8", Shall).severity(Severity::Warning).reason(CONTENT_KIND_REASON).interpret("Exact, case-sensitive match against Table 3 (default scope) or Table 4 (2016#content-kind scope); other scopes are out of scope.", Some("https://github.com/Netflix/photon/issues/361")),
    ],
};

/// SourceEncoding present but EssenceDescriptorList absent.
pub static SourceEncodingNoEssenceDescriptorList: Rule<E> = Rule {
    name: "SourceEncodingNoEssenceDescriptorList",
    category: Category::Reference,
    description: "SourceEncoding present but EssenceDescriptorList absent.",
    example: Some("<Resource><SourceEncoding>urn:uuid:…</SourceEncoding></Resource>  <!-- but the CPL has no <EssenceDescriptorList> -->"),
    editions: &[
        checked(E::V2013, "6.12.1", Shall),
        checked(E::V2016, "6.12.1", Shall),
    ],
};

/// SourceEncoding does not match any EssenceDescriptor Id.
pub static SourceEncodingUnresolved: Rule<E> = Rule {
    name: "SourceEncodingUnresolved",
    category: Category::Reference,
    description: "SourceEncoding does not match any EssenceDescriptor Id.",
    example: Some("<SourceEncoding>urn:uuid:aaaa…</SourceEncoding>  <!-- no EssenceDescriptor with that Id -->"),
    editions: &[
        checked(E::V2013, "6.12.1", Shall),
        checked(E::V2016, "6.12.1", Shall),
    ],
};

/// EssenceDescriptorList present but contains no descriptors.
pub static EssenceDescriptorListEmpty: Rule<E> = Rule {
    name: "EssenceDescriptorListEmpty",
    category: Category::Structure,
    description: "EssenceDescriptorList present but contains no descriptors.",
    example: Some("<EssenceDescriptorList></EssenceDescriptorList>"),
    editions: &[
        checked(E::V2013, "6.1", Shall),
        checked(E::V2016, "6.1", Shall),
    ],
};

/// ContentVersionList present but empty.
pub static ContentVersionListEmpty: Rule<E> = Rule {
    name: "ContentVersionListEmpty",
    category: Category::Structure,
    description: "ContentVersionList present but empty.",
    example: Some("<ContentVersionList></ContentVersionList>"),
    editions: &[
        checked(E::V2013, "6.1", Shall),
        checked(E::V2016, "6.1", Shall),
    ],
};

/// ContentVersion/Id is empty (shall be a URI).
pub static ContentVersionIdInvalid: Rule<E> = Rule {
    name: "ContentVersionIdInvalid",
    category: Category::Metadata,
    description: "ContentVersion/Id is empty (shall be a URI).",
    example: Some("<ContentVersion><Id></Id>…</ContentVersion>"),
    editions: &[
        checked(E::V2013, "6.8.1", Shall),
        checked(E::V2016, "6.8.1", Shall),
    ],
};

/// ContentVersion/LabelText is absent.
pub static ContentVersionLabelTextMissing: Rule<E> = Rule {
    name: "ContentVersionLabelTextMissing",
    category: Category::Metadata,
    description: "ContentVersion/LabelText is absent.",
    example: Some("<ContentVersion><Id>urn:uuid:…</Id></ContentVersion>  <!-- no <LabelText> -->"),
    editions: &[
        checked(E::V2013, "6.8", Shall),
        checked(E::V2016, "6.8", Shall),
    ],
};

/// Locale language tag does not conform to RFC 5646.
pub static LocaleLanguageTagInvalid: Rule<E> = Rule {
    name: "LocaleLanguageTagInvalid",
    category: Category::Metadata,
    description: "Locale language tag does not conform to RFC 5646.",
    example: Some("<Locale><LanguageList><Language>english</Language></LanguageList></Locale>  <!-- not RFC 5646 -->"),
    editions: &[
        checked(E::V2013, "6.5.2", Shall),
        checked(E::V2016, "6.5.2", Shall),
    ],
};

/// TrackId is not unique within a segment.
pub static TrackIdNotUnique: Rule<E> = Rule {
    name: "TrackIdNotUnique",
    category: Category::Structure,
    description: "TrackId is not unique within a segment.",
    example: Some("<Sequence><TrackId>urn:uuid:abc…</TrackId>…</Sequence><Sequence><TrackId>urn:uuid:abc…</TrackId>…</Sequence>  <!-- same TrackId in two sequences of one segment -->"),
    editions: &[
        checked(E::V2013, "6.9.3", Shall),
        checked(E::V2016, "6.9.3", Shall),
    ],
};

/// Marker offset exceeds the resource IntrinsicDuration.
pub static MarkerOffsetOutOfRange: Rule<E> = Rule {
    name: "MarkerOffsetOutOfRange",
    category: Category::Timing,
    description: "Marker offset exceeds the resource IntrinsicDuration.",
    example: Some("<Marker><Offset>120</Offset></Marker>  <!-- resource IntrinsicDuration=100 -->"),
    editions: &[
        checked(E::V2013, "6.13", Shall).interpret("The bound is IntrinsicDuration, inclusive (Offset == IntrinsicDuration passes); EntryPoint/SourceDuration do not constrain Offset.", Some("https://github.com/Netflix/photon/issues/273")),
        checked(E::V2016, "6.13", Shall).interpret("The bound is IntrinsicDuration, inclusive (Offset == IntrinsicDuration passes); EntryPoint/SourceDuration do not constrain Offset.", Some("https://github.com/Netflix/photon/issues/273")),
    ],
};

/// Marker label is not a recognized SMPTE standard value.
pub static MarkerLabelUnknown: Rule<E> = Rule {
    name: "MarkerLabelUnknown",
    category: Category::Metadata,
    description: "Marker label is not a recognized SMPTE standard value.",
    example: Some("<Marker><Label>NotARealMarker</Label></Marker>"),
    editions: &[
        checked(E::V2013, "6.14.2", Shall).interpret("Labels under the default scope must be in Table 19; other scopes are out of scope.", Some("https://github.com/Netflix/photon/issues/394")),
        checked(E::V2016, "6.14.1.2", Shall).interpret("Labels under the 2013 and 2016 standard-marker scopes must be in Table 21; other scopes are out of scope.", Some("https://github.com/Netflix/photon/issues/394")),
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
        checked(E::V2013, "7.2", Shall),
        checked(E::V2016, "7.2", Shall),
    ],
};

/// No two ContentVersion elements shall have identical Id values.
pub static ContentVersionIdDuplicate: Rule<E> = Rule {
    name: "ContentVersionIdDuplicate",
    category: Category::Structure,
    description: "No two ContentVersion elements shall have identical Id values.",
    example: Some("<ContentVersion><Id>urn:uuid:a…</Id></ContentVersion><ContentVersion><Id>urn:uuid:a…</Id></ContentVersion>"),
    editions: &[
        checked(E::V2013, "6.1.9", Shall),
        checked(E::V2016, "6.1.9", Shall),
    ],
};

/// Sequence duration shall be an integer number of Composition Edit Units.
pub static SegmentDurationIntegerEditUnits: Rule<E> = Rule {
    name: "SegmentDurationIntegerEditUnits",
    category: Category::Timing,
    description: "Sequence duration shall be an integer number of Composition Edit Units.",
    example: Some("Sequence duration = 12.5 edit units (must be an integer multiple of the Composition Edit Unit)"),
    editions: &[
        checked(E::V2013, "7.3", Shall),
        checked(E::V2016, "7.3", Shall),
    ],
};

/// Duplicate Segment Id within the CPL.
pub static UniqueSegmentId: Rule<E> = Rule {
    name: "UniqueSegmentId",
    category: Category::Structure,
    description: "Duplicate Segment Id within the CPL.",
    example: Some("Two <Segment><Id>urn:uuid:abc…</Id></Segment> with the same Id"),
    editions: &[
        checked(E::V2013, "6.9.1", Shall),
        checked(E::V2016, "6.9.1", Shall),
    ],
};

/// Duplicate EssenceDescriptor Id within the CPL.
pub static UniqueEssenceDescriptorId: Rule<E> = Rule {
    name: "UniqueEssenceDescriptorId",
    category: Category::Structure,
    description: "Duplicate EssenceDescriptor Id within the CPL.",
    example: Some(
        "Two <EssenceDescriptor><Id>urn:uuid:abc…</Id></EssenceDescriptor> with the same Id",
    ),
    editions: &[
        checked(E::V2013, "6.7.1", Shall),
        checked(E::V2016, "6.7.1", Shall),
    ],
};

/// Duplicate Resource Id within the CPL.
pub static UniqueResourceId: Rule<E> = Rule {
    name: "UniqueResourceId",
    category: Category::Structure,
    description: "Duplicate Resource Id within the CPL.",
    example: Some("Two <Resource><Id>urn:uuid:abc…</Id></Resource> with the same Id"),
    editions: &[
        checked(E::V2013, "6.11.1", Shall),
        checked(E::V2016, "6.11.1", Shall),
    ],
};

/// EntryPoint shall be less than IntrinsicDuration.
pub static EntryPoint: Rule<E> = Rule {
    name: "EntryPoint",
    category: Category::Timing,
    description: "EntryPoint shall be less than IntrinsicDuration.",
    example: Some("<IntrinsicDuration>100</IntrinsicDuration><EntryPoint>150</EntryPoint>"),
    editions: &[
        checked(E::V2013, "6.11.6", Shall).interpret("Flags EntryPoint >= IntrinsicDuration. The text bounds SourceDuration (0 to IntrinsicDuration - EntryPoint), so EntryPoint == IntrinsicDuration with a zero SourceDuration is arguably allowed; open for decision.", None),
        checked(E::V2016, "6.11.6", Shall).interpret("Flags EntryPoint >= IntrinsicDuration. The text bounds SourceDuration (0 to IntrinsicDuration - EntryPoint), so EntryPoint == IntrinsicDuration with a zero SourceDuration is arguably allowed; open for decision.", None),
    ],
};

/// EntryPoint + SourceDuration exceeds IntrinsicDuration.
pub static ResourceDuration: Rule<E> = Rule {
    name: "ResourceDuration",
    category: Category::Timing,
    description: "EntryPoint + SourceDuration exceeds IntrinsicDuration.",
    example: Some("<IntrinsicDuration>100</IntrinsicDuration><EntryPoint>50</EntryPoint><SourceDuration>80</SourceDuration>  <!-- 50+80 > 100 -->"),
    editions: &[
        checked(E::V2013, "6.11.6", Shall),
        checked(E::V2016, "6.11.6", Shall),
    ],
};

/// RepeatCount shall be a positive integer.
pub static RepeatCount: Rule<E> = Rule {
    name: "RepeatCount",
    category: Category::Timing,
    description: "RepeatCount shall be a positive integer.",
    example: Some("<RepeatCount>0</RepeatCount>"),
    editions: &[
        checked(E::V2013, "6.11", Shall),
        checked(E::V2016, "6.11", Shall),
    ],
};

/// A non-marker resource is missing a TrackFileId.
pub static TrackFileId: Rule<E> = Rule {
    name: "TrackFileId",
    category: Category::Reference,
    description: "A non-marker resource is missing a TrackFileId.",
    example: Some("<Resource xsi:type=\"TrackFileResourceType\">…</Resource>  <!-- no <TrackFileId> child -->"),
    editions: &[
        checked(E::V2013, "6.12", Shall),
        checked(E::V2016, "6.12", Shall),
    ],
};

/// A virtual track is missing from one or more segments.
pub static VirtualTrackContinuity: Rule<E> = Rule {
    name: "VirtualTrackContinuity",
    category: Category::Structure,
    description: "A virtual track is missing from one or more segments.",
    example: Some(
        "Two <Segment> entries, MainAudio track present in segment 0 but absent from segment 1",
    ),
    editions: &[
        checked(E::V2013, "6.9.3", Shall),
        checked(E::V2016, "6.9.3", Shall),
    ],
};

/// EssenceDescriptor present in EssenceDescriptorList but not referenced by any Resource.
pub static DanglingEssenceDescriptor: Rule<E> = Rule {
    name: "DanglingEssenceDescriptor",
    category: Category::Reference,
    description: "EssenceDescriptor present in EssenceDescriptorList but not referenced by any Resource.",
    example: Some("<EssenceDescriptor><Id>urn:uuid:abc…</Id></EssenceDescriptor>  <!-- no Resource references this Id -->"),
    editions: &[
        checked(E::V2013, "6.1.10.1", Shall),
        checked(E::V2016, "6.1.10.1", Shall),
    ],
};

/// ContentTitle is empty.
pub static ContentTitleEmpty: Rule<E> = Rule {
    name: "ContentTitleEmpty",
    category: Category::Metadata,
    description: "ContentTitle is empty; it shall contain a human-readable title.",
    example: Some("<ContentTitle></ContentTitle>"),
    editions: &[
        checked(E::V2013, "6.1.7", Shall),
        checked(E::V2016, "6.1.7", Shall),
    ],
};

/// A Sequence has an empty ResourceList.
pub static ResourceListEmpty: Rule<E> = Rule {
    name: "ResourceListEmpty",
    category: Category::Structure,
    description: "A Sequence has an empty ResourceList; the SequenceType schema requires one or more Resource elements.",
    example: Some("<MainAudioSequence>…<ResourceList></ResourceList></MainAudioSequence>"),
    editions: &[
        checked(E::V2013, "6.10", Shall).interpret("Sequences sit in an xs:any lax slot, so the runtime XSD layer does not check Resource minOccurs; imferno does.", None),
        checked(E::V2016, "6.10", Shall).interpret("Sequences sit in an xs:any lax slot, so the runtime XSD layer does not check Resource minOccurs; imferno does.", None),
    ],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[
    &ContentTitleEmpty,
    &ResourceListEmpty,
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
    &UniqueSegmentId,
    &UniqueEssenceDescriptorId,
    &UniqueResourceId,
    &EntryPoint,
    &ResourceDuration,
    &RepeatCount,
    &TrackFileId,
    &VirtualTrackContinuity,
    &DanglingEssenceDescriptor,
];
