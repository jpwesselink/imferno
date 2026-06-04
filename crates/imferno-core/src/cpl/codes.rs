//! Typed validation-code catalogue for SMPTE ST 2067-3 (Composition Playlist).

use crate::diagnostics::codes::ValidationCode;
use crate::diagnostics::{Category, Severity};

// ─── Spec-agnostic reason enum ────────────────────────────────────────────────

/// Reason codes for ST 2067-3 validators, independent of spec edition year.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum St2067_3Code {
    /// ST 2067-3 §5.5.1.2: ContentKind uses an unrecognized value under the SMPTE scope.
    ContentKindUnknown,
    /// ST 2067-3 §6.4.2: SourceEncoding present but EssenceDescriptorList absent.
    SourceEncodingNoEssenceDescriptorList,
    /// ST 2067-3 §6.4.2: SourceEncoding does not match any EssenceDescriptor Id.
    SourceEncodingUnresolved,
    /// ST 2067-3 §6.4.2: EssenceDescriptorList present but contains no descriptors.
    EssenceDescriptorListEmpty,
    /// ST 2067-3 §6.11: ContentVersionList present but empty.
    ContentVersionListEmpty,
    /// ST 2067-3 §6.11: ContentVersion/Id is empty (shall be a URI).
    ContentVersionIdInvalid,
    /// ST 2067-3 §6.11: ContentVersion/LabelText is absent.
    ContentVersionLabelTextMissing,
    /// ST 2067-3 §6.12: Locale language tag does not conform to RFC 5646.
    LocaleLanguageTagInvalid,
    /// ST 2067-3 §7.3: TrackId is not unique within a segment.
    TrackIdNotUnique,
    /// ST 2067-3 §7.4: Marker offset exceeds resource effective duration.
    MarkerOffsetOutOfRange,
    /// ST 2067-3 §7.4: Marker label is not a recognized SMPTE standard value.
    MarkerLabelUnknown,
    /// ST 2067-3 §7.2.2: All virtual tracks in a segment must span the same edit units.
    SegmentDuration,
    /// ST 2067-3 §6.1.9: Two ContentVersion elements share the same Id value.
    ContentVersionIdDuplicate,
    /// ST 2067-3 §7.3: Sequence duration is not an integer number of Composition Edit Units.
    SegmentDurationIntegerEditUnits,
}

// ─── Per-edition enums ────────────────────────────────────────────────────────

macro_rules! define_st2067_3_enum {
    // Edition with no identical predecessor — defaults to the
    // trait-provided `previous_identical_edition` = None.
    ($name:ident, $prefix:literal) => {
        define_st2067_3_enum!(@inner $name, $prefix, None);
    };
    // Edition whose code set matches a prior edition byte-for-byte
    // (per the snapshot diff in docs/catalogue-todos.md Item 2).
    ($name:ident, $prefix:literal, $previous:literal) => {
        define_st2067_3_enum!(@inner $name, $prefix, Some($previous));
    };
    (@inner $name:ident, $prefix:literal, $previous:expr) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
        pub enum $name {
            ContentKindUnknown,
            SourceEncodingNoEssenceDescriptorList,
            SourceEncodingUnresolved,
            EssenceDescriptorListEmpty,
            ContentVersionListEmpty,
            ContentVersionIdInvalid,
            ContentVersionLabelTextMissing,
            LocaleLanguageTagInvalid,
            TrackIdNotUnique,
            MarkerOffsetOutOfRange,
            MarkerLabelUnknown,
            SegmentDuration,
            ContentVersionIdDuplicate,
            SegmentDurationIntegerEditUnits,
        }

        impl $name {
            pub fn for_code(r: St2067_3Code) -> &'static str {
                match r {
                    St2067_3Code::ContentKindUnknown => {
                        concat!($prefix, ":5.5.1.2/ContentKindUnknown")
                    }
                    St2067_3Code::SourceEncodingNoEssenceDescriptorList => {
                        concat!($prefix, ":6.4.2/SourceEncodingNoEssenceDescriptorList")
                    }
                    St2067_3Code::SourceEncodingUnresolved => {
                        concat!($prefix, ":6.4.2/SourceEncodingUnresolved")
                    }
                    St2067_3Code::EssenceDescriptorListEmpty => {
                        concat!($prefix, ":6.4.2/EssenceDescriptorListEmpty")
                    }
                    St2067_3Code::ContentVersionListEmpty => {
                        concat!($prefix, ":6.11/ContentVersionListEmpty")
                    }
                    St2067_3Code::ContentVersionIdInvalid => {
                        concat!($prefix, ":6.11/ContentVersionIdInvalid")
                    }
                    St2067_3Code::ContentVersionLabelTextMissing => {
                        concat!($prefix, ":6.11/ContentVersionLabelTextMissing")
                    }
                    St2067_3Code::LocaleLanguageTagInvalid => {
                        concat!($prefix, ":6.12/LocaleLanguageTagInvalid")
                    }
                    St2067_3Code::TrackIdNotUnique => concat!($prefix, ":7.3/TrackIdNotUnique"),
                    St2067_3Code::MarkerOffsetOutOfRange => {
                        concat!($prefix, ":7.4/MarkerOffsetOutOfRange")
                    }
                    St2067_3Code::MarkerLabelUnknown => concat!($prefix, ":7.4/MarkerLabelUnknown"),
                    St2067_3Code::SegmentDuration => concat!($prefix, ":7.2.2/SegmentDuration"),
                    St2067_3Code::ContentVersionIdDuplicate => {
                        concat!($prefix, ":6.1.9/ContentVersionIdDuplicate")
                    }
                    St2067_3Code::SegmentDurationIntegerEditUnits => {
                        concat!($prefix, ":7.3/SegmentDurationIntegerEditUnits")
                    }
                }
            }

            pub const ALL: &'static [Self] = &[
                Self::ContentKindUnknown,
                Self::SourceEncodingNoEssenceDescriptorList,
                Self::SourceEncodingUnresolved,
                Self::EssenceDescriptorListEmpty,
                Self::ContentVersionListEmpty,
                Self::ContentVersionIdInvalid,
                Self::ContentVersionLabelTextMissing,
                Self::LocaleLanguageTagInvalid,
                Self::TrackIdNotUnique,
                Self::MarkerOffsetOutOfRange,
                Self::MarkerLabelUnknown,
                Self::SegmentDuration,
                Self::ContentVersionIdDuplicate,
                Self::SegmentDurationIntegerEditUnits,
            ];
        }

        impl ValidationCode for $name {
            fn code(&self) -> &'static str {
                match self {
                    Self::ContentKindUnknown => concat!($prefix, ":5.5.1.2/ContentKindUnknown"),
                    Self::SourceEncodingNoEssenceDescriptorList => {
                        concat!($prefix, ":6.4.2/SourceEncodingNoEssenceDescriptorList")
                    }
                    Self::SourceEncodingUnresolved => {
                        concat!($prefix, ":6.4.2/SourceEncodingUnresolved")
                    }
                    Self::EssenceDescriptorListEmpty => {
                        concat!($prefix, ":6.4.2/EssenceDescriptorListEmpty")
                    }
                    Self::ContentVersionListEmpty => {
                        concat!($prefix, ":6.11/ContentVersionListEmpty")
                    }
                    Self::ContentVersionIdInvalid => {
                        concat!($prefix, ":6.11/ContentVersionIdInvalid")
                    }
                    Self::ContentVersionLabelTextMissing => {
                        concat!($prefix, ":6.11/ContentVersionLabelTextMissing")
                    }
                    Self::LocaleLanguageTagInvalid => {
                        concat!($prefix, ":6.12/LocaleLanguageTagInvalid")
                    }
                    Self::TrackIdNotUnique => concat!($prefix, ":7.3/TrackIdNotUnique"),
                    Self::MarkerOffsetOutOfRange => concat!($prefix, ":7.4/MarkerOffsetOutOfRange"),
                    Self::MarkerLabelUnknown => concat!($prefix, ":7.4/MarkerLabelUnknown"),
                    Self::SegmentDuration => concat!($prefix, ":7.2.2/SegmentDuration"),
                    Self::ContentVersionIdDuplicate => {
                        concat!($prefix, ":6.1.9/ContentVersionIdDuplicate")
                    }
                    Self::SegmentDurationIntegerEditUnits => {
                        concat!($prefix, ":7.3/SegmentDurationIntegerEditUnits")
                    }
                }
            }
            fn description(&self) -> &'static str {
                match self {
                    Self::ContentKindUnknown => {
                        "ContentKind uses an unrecognized value under the SMPTE scope."
                    }
                    Self::SourceEncodingNoEssenceDescriptorList => {
                        "SourceEncoding present but EssenceDescriptorList absent."
                    }
                    Self::SourceEncodingUnresolved => {
                        "SourceEncoding does not match any EssenceDescriptor Id."
                    }
                    Self::EssenceDescriptorListEmpty => {
                        "EssenceDescriptorList present but contains no descriptors."
                    }
                    Self::ContentVersionListEmpty => "ContentVersionList present but empty.",
                    Self::ContentVersionIdInvalid => "ContentVersion/Id is empty (shall be a URI).",
                    Self::ContentVersionLabelTextMissing => "ContentVersion/LabelText is absent.",
                    Self::LocaleLanguageTagInvalid => {
                        "Locale language tag does not conform to RFC 5646."
                    }
                    Self::TrackIdNotUnique => "TrackId is not unique within a segment.",
                    Self::MarkerOffsetOutOfRange => {
                        "Marker offset exceeds resource effective duration."
                    }
                    Self::MarkerLabelUnknown => {
                        "Marker label is not a recognized SMPTE standard value."
                    }
                    Self::SegmentDuration => {
                        "All virtual tracks in a segment must span the same number of edit units."
                    }
                    Self::ContentVersionIdDuplicate => {
                        "No two ContentVersion elements shall have identical Id values."
                    }
                    Self::SegmentDurationIntegerEditUnits => {
                        "Sequence duration shall be an integer number of Composition Edit Units."
                    }
                }
            }
            fn default_severity(&self) -> Severity {
                match self {
                    Self::ContentKindUnknown => Severity::Warning,
                    Self::SourceEncodingNoEssenceDescriptorList => Severity::Error,
                    Self::SourceEncodingUnresolved => Severity::Error,
                    Self::EssenceDescriptorListEmpty => Severity::Error,
                    Self::ContentVersionListEmpty => Severity::Error,
                    Self::ContentVersionIdInvalid => Severity::Error,
                    Self::ContentVersionLabelTextMissing => Severity::Warning,
                    Self::LocaleLanguageTagInvalid => Severity::Warning,
                    Self::TrackIdNotUnique => Severity::Error,
                    Self::MarkerOffsetOutOfRange => Severity::Error,
                    Self::MarkerLabelUnknown => Severity::Warning,
                    Self::SegmentDuration => Severity::Error,
                    Self::ContentVersionIdDuplicate => Severity::Error,
                    Self::SegmentDurationIntegerEditUnits => Severity::Error,
                }
            }
            fn category(&self) -> Category {
                match self {
                    Self::ContentKindUnknown => Category::Metadata,
                    Self::SourceEncodingNoEssenceDescriptorList => Category::Reference,
                    Self::SourceEncodingUnresolved => Category::Reference,
                    Self::EssenceDescriptorListEmpty => Category::Structure,
                    Self::ContentVersionListEmpty => Category::Structure,
                    Self::ContentVersionIdInvalid => Category::Metadata,
                    Self::ContentVersionLabelTextMissing => Category::Metadata,
                    Self::LocaleLanguageTagInvalid => Category::Metadata,
                    Self::TrackIdNotUnique => Category::Structure,
                    Self::MarkerOffsetOutOfRange => Category::Timing,
                    Self::MarkerLabelUnknown => Category::Metadata,
                    Self::SegmentDuration => Category::Timing,
                    Self::ContentVersionIdDuplicate => Category::Structure,
                    Self::SegmentDurationIntegerEditUnits => Category::Timing,
                }
            }
            fn previous_identical_edition(&self) -> Option<&'static str> {
                $previous
            }
            fn example(&self) -> Option<&'static str> {
                Some(match self {
                    Self::ContentKindUnknown =>
                        "<ContentKind scope=\"http://www.smpte-ra.org/schemas/2067-3/2013/content-kind\">Banana</ContentKind>",
                    Self::SourceEncodingNoEssenceDescriptorList =>
                        "<Resource><SourceEncoding>urn:uuid:…</SourceEncoding></Resource>  <!-- but the CPL has no <EssenceDescriptorList> -->",
                    Self::SourceEncodingUnresolved =>
                        "<SourceEncoding>urn:uuid:aaaa…</SourceEncoding>  <!-- no EssenceDescriptor with that Id -->",
                    Self::EssenceDescriptorListEmpty =>
                        "<EssenceDescriptorList></EssenceDescriptorList>",
                    Self::ContentVersionListEmpty =>
                        "<ContentVersionList></ContentVersionList>",
                    Self::ContentVersionIdInvalid =>
                        "<ContentVersion><Id></Id>…</ContentVersion>",
                    Self::ContentVersionLabelTextMissing =>
                        "<ContentVersion><Id>urn:uuid:…</Id></ContentVersion>  <!-- no <LabelText> -->",
                    Self::LocaleLanguageTagInvalid =>
                        "<Locale><LanguageList><Language>english</Language></LanguageList></Locale>  <!-- not RFC 5646 -->",
                    Self::TrackIdNotUnique =>
                        "<Sequence><TrackId>urn:uuid:abc…</TrackId>…</Sequence><Sequence><TrackId>urn:uuid:abc…</TrackId>…</Sequence>  <!-- same TrackId in two sequences of one segment -->",
                    Self::MarkerOffsetOutOfRange =>
                        "<Marker><Offset>120</Offset></Marker>  <!-- resource IntrinsicDuration=100 -->",
                    Self::MarkerLabelUnknown =>
                        "<Marker><Label>NotARealMarker</Label></Marker>",
                    Self::SegmentDuration =>
                        "Segment with MainImageSequence duration = 24f and MainAudioSequence duration = 25f",
                    Self::ContentVersionIdDuplicate =>
                        "<ContentVersion><Id>urn:uuid:a…</Id></ContentVersion><ContentVersion><Id>urn:uuid:a…</Id></ContentVersion>",
                    Self::SegmentDurationIntegerEditUnits =>
                        "Sequence duration = 12.5 edit units (must be an integer multiple of the Composition Edit Unit)",
                })
            }
        }

        impl From<$name> for String {
            fn from(c: $name) -> String {
                c.code().to_string()
            }
        }
    };
}

define_st2067_3_enum!(St2067_3_2013, "ST2067-3:2013");
// ST 2067-3:2020 reuses the 2016 namespace and the canonical XSD is byte-identical
// to 2016, so 2016 + 2020 share this rule set. Also bit-for-bit identical to the
// 2013 catalogue (audit snapshot diff, 2026-06-04), recorded via the second
// macro arm.
define_st2067_3_enum!(St2067_3_2016, "ST2067-3:2016", "ST2067-3:2013");
