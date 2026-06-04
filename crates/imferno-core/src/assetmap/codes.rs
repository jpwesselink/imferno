//! Typed validation-code catalogue for SMPTE ST 2067-2.
//!
//! Two groups of codes live here:
//!
//! 1. **Package-level codes** (`St2067_2_2020`) — AssetMap / PKL / checksum checks
//!    emitted by the package validator.
//!
//! 2. **Core Constraints CPL codes** (`St2067_2_2013_Core` / `St2067_2_2016_Core` /
//!    `St2067_2_2020_Core`) — CPL structure rules (XSD, §6.1, §6.9, §6.10, §8, §10,
//!    ST 377-4) shared across all three spec editions, emitted by the CoreConstraints
//!    validators.  A `macro_rules!` generates the three enums from a
//!    single source-of-truth variant list.  Each enum also exposes `for_code` so that
//!    shared helper functions can produce the right `&'static str` without any runtime
//!    string building.
//!
//! Also re-exports [`St429_9_2014`] from the volindex module.

// Re-export St429_9_2014 from the volindex module for convenience.
pub use crate::assetmap::volindex_codes::St429_9_2014;

use crate::diagnostics::codes::ValidationCode;
use crate::diagnostics::{Category, Severity};

// ─────────────────────────────────────────────────────────────────────────────
// ST 2067-2:2020 — Package-level codes
// ─────────────────────────────────────────────────────────────────────────────

/// Validation codes defined by SMPTE ST 2067-2:2020 for package-level checks
/// (AssetMap, PKL, file integrity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum St2067_2_2020 {
    /// AssetMap document is invalid or cannot be parsed.
    AssetMap,
    /// The ASSETMAP.xml document is not well-formed XML (ST 2067-2:2020 §7).
    AssetMapMalformedXml,
    /// A Packing List document is not well-formed XML (ST 2067-2:2020 §9).
    PklMalformedXml,
    /// No CPL assets found in the AssetMap.
    NoCpls,
    /// Declared file size does not match the on-disk size.
    SizeMismatch,
    /// A referenced asset file is not present at the declared path.
    FileNotFound,
    /// File hash does not match the declared SHA-1/SHA-256 checksum.
    ChecksumMismatch,
    /// UUID referenced in the CPL does not resolve to a known asset.
    UnresolvedUuid,
    /// Two or more assets within the package share the same UUID.
    DuplicateUuid,
    /// An I/O error prevented the asset from being read.
    IoError,
    /// EssenceDescriptorList element is required per ST 2067-2:2020 §6.4.2.
    EssenceDescriptorList,
    /// PKL document namespace is not one of the published SMPTE PKL
    /// namespace URIs — breaks interop with conformant validators.
    PklUnknownNamespace,
    /// AssetMap declares no asset as a PackingList — ST 429-9 §6.3
    /// requires every package to have at least one PKL identified
    /// by `<PackingList>true</PackingList>`.
    AssetMapHasNoPackingList,
    /// A PKL document was parsed but its Id is not declared in the
    /// AssetMap as a `PackingList`-flagged asset (ST 429-9 §6.3).
    PklIdNotInAssetMap,
}

impl ValidationCode for St2067_2_2020 {
    fn code(&self) -> &'static str {
        match self {
            Self::AssetMap => "ST2067-2:2020:7/AssetMap",
            Self::AssetMapMalformedXml => "ST2067-2:2020:7/MalformedXml",
            Self::PklMalformedXml => "ST2067-2:2020:9/MalformedXml",
            Self::NoCpls => "ST2067-2:2020:7/NoCpls",
            Self::SizeMismatch => "ST2067-2:2020:8.3/SizeMismatch",
            Self::FileNotFound => "ST2067-2:2020:8.3/FileNotFound",
            Self::ChecksumMismatch => "ST2067-2:2020:8.3/ChecksumMismatch",
            Self::UnresolvedUuid => "ST2067-2:2020:7/UnresolvedUuid",
            Self::DuplicateUuid => "ST2067-2:2020:7/DuplicateUuid",
            Self::IoError => "IMF:General/IoError",
            Self::EssenceDescriptorList => "ST2067-2:2020:6.4.2/EssenceDescriptorList",
            Self::PklUnknownNamespace => "ST2067-2:2020:9/PklUnknownNamespace",
            Self::AssetMapHasNoPackingList => "ST429-9:2014:6.3/AssetMapHasNoPackingList",
            Self::PklIdNotInAssetMap => "ST429-9:2014:6.3/PklIdNotInAssetMap",
        }
    }
    fn description(&self) -> &'static str {
        match self {
            Self::AssetMap => "AssetMap document is invalid or cannot be parsed.",
            Self::AssetMapMalformedXml => "The ASSETMAP.xml document is not well-formed XML.",
            Self::PklMalformedXml => "A Packing List document is not well-formed XML.",
            Self::NoCpls => "No CPL assets found in the AssetMap.",
            Self::SizeMismatch => "Declared file size does not match the on-disk size.",
            Self::FileNotFound => "A referenced asset file is not present at the declared path.",
            Self::ChecksumMismatch => {
                "File hash does not match the declared SHA-1/SHA-256 checksum."
            }
            Self::UnresolvedUuid => "UUID referenced in the CPL does not resolve to a known asset.",
            Self::DuplicateUuid => "Two or more assets within the package share the same UUID.",
            Self::IoError => "An I/O error prevented the asset from being read.",
            Self::EssenceDescriptorList => {
                "EssenceDescriptorList element is required per ST 2067-2:2020 §6.4.2."
            }
            Self::PklUnknownNamespace => {
                "PKL document namespace is not one of the published SMPTE PKL namespaces."
            }
            Self::AssetMapHasNoPackingList => {
                "AssetMap declares no asset as a PackingList (ST 429-9 §6.3)."
            }
            Self::PklIdNotInAssetMap => {
                "PKL document Id is not declared as a PackingList asset in the AssetMap."
            }
        }
    }
    fn default_severity(&self) -> Severity {
        match self {
            Self::AssetMap | Self::NoCpls | Self::AssetMapHasNoPackingList => Severity::Critical,
            _ => Severity::Error,
        }
    }
    fn category(&self) -> Category {
        match self {
            Self::AssetMap
            | Self::AssetMapMalformedXml
            | Self::PklMalformedXml
            | Self::NoCpls
            | Self::EssenceDescriptorList
            | Self::PklUnknownNamespace
            | Self::AssetMapHasNoPackingList => Category::Structure,
            Self::SizeMismatch | Self::FileNotFound | Self::IoError | Self::ChecksumMismatch => {
                Category::Asset
            }
            Self::UnresolvedUuid | Self::DuplicateUuid | Self::PklIdNotInAssetMap => {
                Category::Reference
            }
        }
    }
    fn example(&self) -> Option<&'static str> {
        Some(match self {
            Self::AssetMap => "ASSETMAP.xml exists but parse_assetmap returns Err (e.g. wrong root element)",
            Self::AssetMapMalformedXml => "ASSETMAP.xml truncated mid-element: `<AssetMap><Id>urn:uuid:…</Id>`",
            Self::PklMalformedXml => "PKL.xml with mismatched tag: `<PackingList>…</packinglist>`",
            Self::NoCpls => "ASSETMAP.xml with all assets flagged `<PackingList>true</PackingList>` and zero CPLs",
            Self::SizeMismatch => "PKL `<Size>1024</Size>` but the actual file is 2048 bytes on disk",
            Self::FileNotFound => "AssetMap `<Path>missing-track.mxf</Path>` but no such file exists in the package",
            Self::ChecksumMismatch => "PKL `<Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>` (SHA-1 of empty string) on a non-empty file",
            Self::UnresolvedUuid => "CPL TrackFileId references urn:uuid:aaa… but no PKL/AssetMap asset has that Id",
            Self::DuplicateUuid => "Two AssetMap `<Asset><Id>urn:uuid:00…01</Id></Asset>` entries with the same Id",
            Self::IoError => "Underlying filesystem refused the read (permissions, device error, etc.)",
            Self::EssenceDescriptorList => "CPL with `<SourceEncoding>` references but no top-level `<EssenceDescriptorList>` element",
            Self::PklUnknownNamespace => "PKL with `xmlns=\"http://example.org/not-a-pkl-namespace\"`",
            Self::AssetMapHasNoPackingList => "AssetMap whose `<Asset>` entries all omit `<PackingList>true</PackingList>`",
            Self::PklIdNotInAssetMap => "PKL `<Id>urn:uuid:abc…</Id>` not present in the AssetMap as a packing-list asset",
        })
    }
}

impl St2067_2_2020 {
    pub const ALL: &'static [Self] = &[
        Self::AssetMap,
        Self::AssetMapMalformedXml,
        Self::PklMalformedXml,
        Self::NoCpls,
        Self::SizeMismatch,
        Self::FileNotFound,
        Self::ChecksumMismatch,
        Self::UnresolvedUuid,
        Self::DuplicateUuid,
        Self::IoError,
        Self::EssenceDescriptorList,
        Self::PklUnknownNamespace,
        Self::AssetMapHasNoPackingList,
        Self::PklIdNotInAssetMap,
    ];
}

impl From<St2067_2_2020> for String {
    fn from(c: St2067_2_2020) -> String {
        c.code().to_string()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Core Constraints CPL codes  (emitted by CoreConstraints validators in st2067-21)
// ─────────────────────────────────────────────────────────────────────────────

/// Spec-agnostic reason codes for Core Constraints CPL validation.
///
/// Passed to each edition's `for_code` dispatch function to get the full
/// `&'static str` code without any runtime string building.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreConstraintsCode {
    ResourceListEmpty,
    ContentTitle,
    TotalRunningTimeFormat,
    SegmentList,
    Segment,
    EditRate,
    IssueDate,
    IssueDateFormat,
    CompositionTimecodeDropFrame,
    CompositionTimecodeRate,
    CompositionTimecodeStartAddress,
    CompositionTimecodeRateZero,
    CompositionTimecodeStartAddressFormat,
    CompositionTimecodeRateMismatch,
    LocaleListNonEmpty,
    UniqueSegmentId,
    UniqueEssenceDescriptorId,
    UniqueResourceId,
    IntrinsicDuration,
    EntryPoint,
    SourceDuration,
    ResourceDuration,
    RepeatCount,
    TrackFileId,
    VirtualTrackContinuity,
    VirtualTrackEditRate,
    TimedTextSampleRate,
    TimedTextEmptyLanguageTag,
    TimedTextMalformedLanguageTag,
    AudioSampleRate,
    ChannelCount,
    MCASubDescriptors,
    SoundfieldGroup,
    MCATagSymbol,
    SoundfieldChannelCount,
    DigitalSignature,
    DanglingEssenceDescriptor,
    EssenceDescriptorList,
}

macro_rules! define_core_constraints_enum {
    // No identical predecessor — trait default `None`.
    ($name:ident, $prefix:literal) => {
        define_core_constraints_enum!(@inner $name, $prefix, None);
    };
    // Identical to a prior edition (verified via snapshot diff,
    // docs/catalogue-todos.md Item 2).
    ($name:ident, $prefix:literal, $previous:literal) => {
        define_core_constraints_enum!(@inner $name, $prefix, Some($previous));
    };
    (@inner $name:ident, $prefix:literal, $previous:expr) => {
        /// Validation codes for Core Constraints CPL checks, edition
        #[doc = $prefix]
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            /// A Sequence has an empty ResourceList.
            ResourceListEmpty,
            /// ContentTitle shall not be empty.
            ContentTitle,
            /// TotalRunningTime does not match format HH:MM:SS.
            TotalRunningTimeFormat,
            /// SegmentList shall contain at least one Segment.
            SegmentList,
            /// A Segment contains no sequences.
            Segment,
            /// CPL EditRate is required (XSD schema §88).
            EditRate,
            /// IssueDate shall not be empty.
            IssueDate,
            /// IssueDate is not a valid xs:dateTime format.
            IssueDateFormat,
            /// CompositionTimecode.TimecodeDropFrame is required when CompositionTimecode is present.
            CompositionTimecodeDropFrame,
            /// CompositionTimecode.TimecodeRate is required when CompositionTimecode is present.
            CompositionTimecodeRate,
            /// CompositionTimecode.TimecodeStartAddress is required when CompositionTimecode is present.
            CompositionTimecodeStartAddress,
            /// CompositionTimecode.TimecodeRate shall be a positive integer.
            CompositionTimecodeRateZero,
            /// TimecodeStartAddress does not match SMPTE timecode format HH:MM:SS:FF.
            CompositionTimecodeStartAddressFormat,
            /// CompositionTimecode.TimecodeRate does not match the CPL EditRate.
            CompositionTimecodeRateMismatch,
            /// LocaleList shall contain at least one Locale.
            LocaleListNonEmpty,
            /// Duplicate Segment Id within the CPL.
            UniqueSegmentId,
            /// Duplicate EssenceDescriptor Id within the CPL.
            UniqueEssenceDescriptorId,
            /// Duplicate Resource Id within the CPL.
            UniqueResourceId,
            /// IntrinsicDuration shall be greater than 0.
            IntrinsicDuration,
            /// EntryPoint shall be less than IntrinsicDuration.
            EntryPoint,
            /// EntryPoint + SourceDuration exceeds IntrinsicDuration.
            SourceDuration,
            /// SourceDuration shall be a positive integer.
            ResourceDuration,
            /// RepeatCount shall be a positive integer.
            RepeatCount,
            /// A non-marker resource is missing a TrackFileId.
            TrackFileId,
            /// A virtual track is missing from one or more segments.
            VirtualTrackContinuity,
            /// All resources in a virtual track shall have the same edit rate.
            VirtualTrackEditRate,
            /// DCTimedTextDescriptor SampleRate is missing.
            TimedTextSampleRate,
            /// Empty language tag in RFC5646LanguageTagList.
            TimedTextEmptyLanguageTag,
            /// Language tag does not start with an ASCII letter (RFC 5646 primary subtag).
            TimedTextMalformedLanguageTag,
            /// WAVEPCMDescriptor has no AudioSampleRate or SampleRate.
            AudioSampleRate,
            /// WAVEPCMDescriptor ChannelCount is zero or missing.
            ChannelCount,
            /// WAVEPCMDescriptor has no MCA SubDescriptors.
            MCASubDescriptors,
            /// WAVEPCMDescriptor SubDescriptors missing SoundfieldGroupLabelSubDescriptor.
            SoundfieldGroup,
            /// SoundfieldGroupLabelSubDescriptor is missing MCATagSymbol.
            MCATagSymbol,
            /// Soundfield group channel count is inconsistent with WAVEPCMDescriptor.ChannelCount.
            SoundfieldChannelCount,
            /// Digital signature validation (ST 2067-2 §8) is not currently performed.
            DigitalSignature,
            /// EssenceDescriptor present in EssenceDescriptorList but not referenced by any Resource.
            DanglingEssenceDescriptor,
            /// EssenceDescriptorList is required per ST 2067-2 §6.4.2.
            EssenceDescriptorList,
        }

        impl ValidationCode for $name {
            fn code(&self) -> &'static str {
                match self {
                    Self::ResourceListEmpty                    => concat!($prefix, ":XSD/ResourceList-Empty"),
                    Self::ContentTitle                         => concat!($prefix, ":XSD/ContentTitle"),
                    Self::TotalRunningTimeFormat               => concat!($prefix, ":XSD/TotalRunningTime-Format"),
                    Self::SegmentList                          => concat!($prefix, ":XSD/SegmentList"),
                    Self::Segment                              => concat!($prefix, ":XSD/Segment"),
                    Self::EditRate                             => concat!($prefix, ":XSD/CompositionPlaylist/EditRate"),
                    Self::IssueDate                            => concat!($prefix, ":XSD/CompositionPlaylist/IssueDate"),
                    Self::IssueDateFormat                      => concat!($prefix, ":XSD/CompositionPlaylist/IssueDate-Format"),
                    Self::CompositionTimecodeDropFrame         => concat!($prefix, ":XSD/CompositionTimecode/DropFrame"),
                    Self::CompositionTimecodeRate              => concat!($prefix, ":XSD/CompositionTimecode/Rate"),
                    Self::CompositionTimecodeStartAddress      => concat!($prefix, ":XSD/CompositionTimecode/StartAddress"),
                    Self::CompositionTimecodeRateZero          => concat!($prefix, ":XSD/CompositionTimecode/Rate-Zero"),
                    Self::CompositionTimecodeStartAddressFormat => concat!($prefix, ":XSD/CompositionTimecode/StartAddress-Format"),
                    Self::CompositionTimecodeRateMismatch      => concat!($prefix, ":XSD/CompositionTimecode/Rate-Mismatch"),
                    Self::LocaleListNonEmpty                   => concat!($prefix, ":XSD/LocaleList-NonEmpty"),
                    Self::UniqueSegmentId                      => concat!($prefix, ":6.1/UniqueSegmentId"),
                    Self::UniqueEssenceDescriptorId            => concat!($prefix, ":6.1/UniqueEssenceDescriptorId"),
                    Self::UniqueResourceId                     => concat!($prefix, ":6.1/UniqueResourceId"),
                    Self::IntrinsicDuration                    => concat!($prefix, ":6.10/IntrinsicDuration"),
                    Self::EntryPoint                           => concat!($prefix, ":6.10/EntryPoint"),
                    Self::SourceDuration                       => concat!($prefix, ":6.10/SourceDuration"),
                    Self::ResourceDuration                     => concat!($prefix, ":6.10/ResourceDuration"),
                    Self::RepeatCount                          => concat!($prefix, ":6.10/RepeatCount"),
                    Self::TrackFileId                          => concat!($prefix, ":6.10/TrackFileId"),
                    Self::VirtualTrackContinuity               => concat!($prefix, ":6.9/VirtualTrackContinuity"),
                    Self::VirtualTrackEditRate                 => concat!($prefix, ":6.9.3/VirtualTrackEditRate"),
                    Self::TimedTextSampleRate                  => concat!($prefix, ":10/TimedText-SampleRate"),
                    Self::TimedTextEmptyLanguageTag            => concat!($prefix, ":10/TimedText-EmptyLanguageTag"),
                    Self::TimedTextMalformedLanguageTag        => concat!($prefix, ":10/TimedText-MalformedLanguageTag"),
                    Self::AudioSampleRate                      => concat!($prefix, ":ST377-4/AudioSampleRate"),
                    Self::ChannelCount                         => concat!($prefix, ":ST377-4/ChannelCount"),
                    Self::MCASubDescriptors                    => concat!($prefix, ":ST377-4/MCASubDescriptors"),
                    Self::SoundfieldGroup                      => concat!($prefix, ":ST377-4/SoundfieldGroup"),
                    Self::MCATagSymbol                         => concat!($prefix, ":ST377-4/MCATagSymbol"),
                    Self::SoundfieldChannelCount               => concat!($prefix, ":ST377-4/SoundfieldChannelCount"),
                    Self::DigitalSignature                     => concat!($prefix, ":8/DigitalSignature"),
                    Self::DanglingEssenceDescriptor            => concat!($prefix, ":6.4.2/DanglingEssenceDescriptor"),
                    Self::EssenceDescriptorList                => concat!($prefix, ":6.4.2/EssenceDescriptorList"),
                }
            }
            fn description(&self) -> &'static str {
                match self {
                    Self::ResourceListEmpty                    => "A Sequence has an empty ResourceList.",
                    Self::ContentTitle                         => "ContentTitle shall not be empty.",
                    Self::TotalRunningTimeFormat               => "TotalRunningTime does not match required format HH:MM:SS.",
                    Self::SegmentList                          => "SegmentList shall contain at least one Segment.",
                    Self::Segment                              => "A Segment contains no sequences.",
                    Self::EditRate                             => "CPL EditRate is required (XSD schema §88).",
                    Self::IssueDate                            => "IssueDate shall not be empty.",
                    Self::IssueDateFormat                      => "IssueDate is not a valid xs:dateTime format.",
                    Self::CompositionTimecodeDropFrame         => "CompositionTimecode.TimecodeDropFrame is required when CompositionTimecode is present.",
                    Self::CompositionTimecodeRate              => "CompositionTimecode.TimecodeRate is required when CompositionTimecode is present.",
                    Self::CompositionTimecodeStartAddress      => "CompositionTimecode.TimecodeStartAddress is required when CompositionTimecode is present.",
                    Self::CompositionTimecodeRateZero          => "CompositionTimecode.TimecodeRate shall be a positive integer.",
                    Self::CompositionTimecodeStartAddressFormat => "TimecodeStartAddress does not match SMPTE timecode format HH:MM:SS:FF.",
                    Self::CompositionTimecodeRateMismatch      => "CompositionTimecode.TimecodeRate does not match the CPL EditRate.",
                    Self::LocaleListNonEmpty                   => "LocaleList shall contain at least one Locale.",
                    Self::UniqueSegmentId                      => "Duplicate Segment Id within the CPL.",
                    Self::UniqueEssenceDescriptorId            => "Duplicate EssenceDescriptor Id within the CPL.",
                    Self::UniqueResourceId                     => "Duplicate Resource Id within the CPL.",
                    Self::IntrinsicDuration                    => "IntrinsicDuration shall be greater than 0.",
                    Self::EntryPoint                           => "EntryPoint shall be less than IntrinsicDuration.",
                    Self::SourceDuration                       => "EntryPoint + SourceDuration exceeds IntrinsicDuration.",
                    Self::ResourceDuration                     => "SourceDuration shall be a positive integer.",
                    Self::RepeatCount                          => "RepeatCount shall be a positive integer.",
                    Self::TrackFileId                          => "A non-marker resource is missing a TrackFileId.",
                    Self::VirtualTrackContinuity               => "A virtual track is missing from one or more segments.",
                    Self::VirtualTrackEditRate                 => "All resources in a virtual track shall have the same edit rate.",
                    Self::TimedTextSampleRate                  => "DCTimedTextDescriptor SampleRate is missing.",
                    Self::TimedTextEmptyLanguageTag            => "Empty language tag in RFC5646LanguageTagList.",
                    Self::TimedTextMalformedLanguageTag        => "Language tag does not start with an ASCII letter (RFC 5646 primary subtag).",
                    Self::AudioSampleRate                      => "WAVEPCMDescriptor has no AudioSampleRate or SampleRate.",
                    Self::ChannelCount                         => "WAVEPCMDescriptor ChannelCount is zero or missing.",
                    Self::MCASubDescriptors                    => "WAVEPCMDescriptor has no MCA SubDescriptors.",
                    Self::SoundfieldGroup                      => "WAVEPCMDescriptor SubDescriptors missing SoundfieldGroupLabelSubDescriptor.",
                    Self::MCATagSymbol                         => "SoundfieldGroupLabelSubDescriptor is missing MCATagSymbol.",
                    Self::SoundfieldChannelCount               => "Soundfield group channel count is inconsistent with WAVEPCMDescriptor.ChannelCount.",
                    Self::DigitalSignature                     => "Digital signature validation (ST 2067-2 §8) is not currently performed.",
                    Self::DanglingEssenceDescriptor            => "EssenceDescriptor present in EssenceDescriptorList but not referenced by any Resource.",
                    Self::EssenceDescriptorList                => "EssenceDescriptorList is required per ST 2067-2 §6.4.2.",
                }
            }
            fn default_severity(&self) -> Severity {
                match self {
                    Self::SegmentList => Severity::Critical,
                    Self::IssueDateFormat
                    | Self::CompositionTimecodeRateMismatch
                    | Self::TimedTextSampleRate
                    | Self::TimedTextEmptyLanguageTag
                    | Self::TimedTextMalformedLanguageTag
                    | Self::AudioSampleRate
                    | Self::ChannelCount
                    | Self::MCASubDescriptors
                    | Self::SoundfieldGroup
                    | Self::MCATagSymbol => Severity::Warning,
                    Self::DigitalSignature => Severity::Info,
                    _ => Severity::Error,
                }
            }
            fn category(&self) -> Category {
                match self {
                    Self::ContentTitle
                    | Self::IssueDate
                    | Self::IssueDateFormat
                    | Self::CompositionTimecodeRateMismatch => Category::Metadata,

                    Self::CompositionTimecodeDropFrame
                    | Self::CompositionTimecodeRate
                    | Self::CompositionTimecodeStartAddress
                    | Self::CompositionTimecodeRateZero
                    | Self::CompositionTimecodeStartAddressFormat
                    | Self::IntrinsicDuration
                    | Self::EntryPoint
                    | Self::SourceDuration
                    | Self::ResourceDuration
                    | Self::RepeatCount
                    | Self::VirtualTrackEditRate => Category::Timing,

                    Self::TimedTextSampleRate
                    | Self::TimedTextEmptyLanguageTag
                    | Self::TimedTextMalformedLanguageTag => Category::Subtitle,

                    Self::AudioSampleRate
                    | Self::ChannelCount
                    | Self::MCASubDescriptors
                    | Self::SoundfieldGroup
                    | Self::MCATagSymbol
                    | Self::SoundfieldChannelCount => Category::Audio,

                    Self::DanglingEssenceDescriptor | Self::TrackFileId => Category::Reference,

                    Self::DigitalSignature => Category::Security,

                    _ => Category::Structure,
                }
            }
            fn previous_identical_edition(&self) -> Option<&'static str> {
                $previous
            }
            fn example(&self) -> Option<&'static str> {
                Some(match self {
                    Self::ResourceListEmpty                    => "<Sequence><ResourceList></ResourceList></Sequence>",
                    Self::ContentTitle                         => "<ContentTitle></ContentTitle>",
                    Self::TotalRunningTimeFormat               => "<TotalRunningTime>1h23m</TotalRunningTime>",
                    Self::SegmentList                          => "<SegmentList></SegmentList>",
                    Self::Segment                              => "<Segment><SequenceList></SequenceList></Segment>",
                    Self::EditRate                             => "<CompositionPlaylist>…</CompositionPlaylist>  <!-- no <EditRate> element -->",
                    Self::IssueDate                            => "<IssueDate></IssueDate>",
                    Self::IssueDateFormat                      => "<IssueDate>2024/01/01</IssueDate>  <!-- not xs:dateTime -->",
                    Self::CompositionTimecodeDropFrame         => "<CompositionTimecode><TimecodeRate>24</TimecodeRate>…</CompositionTimecode>  <!-- no <TimecodeDropFrame> -->",
                    Self::CompositionTimecodeRate              => "<CompositionTimecode><TimecodeDropFrame>false</TimecodeDropFrame>…</CompositionTimecode>  <!-- no <TimecodeRate> -->",
                    Self::CompositionTimecodeStartAddress      => "<CompositionTimecode><TimecodeRate>24</TimecodeRate><TimecodeDropFrame>false</TimecodeDropFrame></CompositionTimecode>  <!-- no <TimecodeStartAddress> -->",
                    Self::CompositionTimecodeRateZero          => "<TimecodeRate>0</TimecodeRate>",
                    Self::CompositionTimecodeStartAddressFormat => "<TimecodeStartAddress>1:2:3</TimecodeStartAddress>  <!-- expected HH:MM:SS:FF -->",
                    Self::CompositionTimecodeRateMismatch      => "CPL <EditRate>24 1</EditRate> but <TimecodeRate>25</TimecodeRate>",
                    Self::LocaleListNonEmpty                   => "<LocaleList></LocaleList>",
                    Self::UniqueSegmentId                      => "Two <Segment><Id>urn:uuid:abc…</Id></Segment> with the same Id",
                    Self::UniqueEssenceDescriptorId            => "Two <EssenceDescriptor><Id>urn:uuid:abc…</Id></EssenceDescriptor> with the same Id",
                    Self::UniqueResourceId                     => "Two <Resource><Id>urn:uuid:abc…</Id></Resource> with the same Id",
                    Self::IntrinsicDuration                    => "<IntrinsicDuration>0</IntrinsicDuration>",
                    Self::EntryPoint                           => "<IntrinsicDuration>100</IntrinsicDuration><EntryPoint>150</EntryPoint>",
                    Self::SourceDuration                       => "<IntrinsicDuration>100</IntrinsicDuration><EntryPoint>50</EntryPoint><SourceDuration>80</SourceDuration>  <!-- 50+80 > 100 -->",
                    Self::ResourceDuration                     => "<SourceDuration>-5</SourceDuration>",
                    Self::RepeatCount                          => "<RepeatCount>0</RepeatCount>",
                    Self::TrackFileId                          => "<Resource xsi:type=\"TrackFileResourceType\">…</Resource>  <!-- no <TrackFileId> child -->",
                    Self::VirtualTrackContinuity               => "Two <Segment> entries, MainAudio track present in segment 0 but absent from segment 1",
                    Self::VirtualTrackEditRate                 => "Track resources mix <EditRate>24 1</EditRate> and <EditRate>25 1</EditRate>",
                    Self::TimedTextSampleRate                  => "<DCTimedTextDescriptor>…</DCTimedTextDescriptor>  <!-- no SampleRate -->",
                    Self::TimedTextEmptyLanguageTag            => "<RFC5646LanguageTagList><RFC5646LanguageTag></RFC5646LanguageTag></RFC5646LanguageTagList>",
                    Self::TimedTextMalformedLanguageTag        => "<RFC5646LanguageTag>123-en</RFC5646LanguageTag>  <!-- primary subtag must start with a letter -->",
                    Self::AudioSampleRate                      => "<WAVEPCMDescriptor>…</WAVEPCMDescriptor>  <!-- no AudioSampleRate -->",
                    Self::ChannelCount                         => "<WAVEPCMDescriptor><ChannelCount>0</ChannelCount></WAVEPCMDescriptor>",
                    Self::MCASubDescriptors                    => "<WAVEPCMDescriptor>…</WAVEPCMDescriptor>  <!-- no SubDescriptors strong-ref list -->",
                    Self::SoundfieldGroup                      => "<WAVEPCMDescriptor><SubDescriptors>…</SubDescriptors></WAVEPCMDescriptor>  <!-- but no SoundfieldGroupLabelSubDescriptor inside -->",
                    Self::MCATagSymbol                         => "<SoundfieldGroupLabelSubDescriptor>…</SoundfieldGroupLabelSubDescriptor>  <!-- no <MCATagSymbol> -->",
                    Self::SoundfieldChannelCount               => "<WAVEPCMDescriptor><ChannelCount>2</ChannelCount>…</WAVEPCMDescriptor> but soundfield group declares 6 channels",
                    Self::DigitalSignature                     => "CPL declares Signer/Signature but the engine does not yet verify them (info notice)",
                    Self::DanglingEssenceDescriptor            => "<EssenceDescriptor><Id>urn:uuid:abc…</Id></EssenceDescriptor>  <!-- no Resource references this Id -->",
                    Self::EssenceDescriptorList                => "CPL with `<SourceEncoding>` references but no top-level `<EssenceDescriptorList>` element",
                })
            }
        }

        impl $name {
            pub const ALL: &'static [Self] = &[
                Self::ResourceListEmpty,
                Self::ContentTitle,
                Self::TotalRunningTimeFormat,
                Self::SegmentList,
                Self::Segment,
                Self::EditRate,
                Self::IssueDate,
                Self::IssueDateFormat,
                Self::CompositionTimecodeDropFrame,
                Self::CompositionTimecodeRate,
                Self::CompositionTimecodeStartAddress,
                Self::CompositionTimecodeRateZero,
                Self::CompositionTimecodeStartAddressFormat,
                Self::CompositionTimecodeRateMismatch,
                Self::LocaleListNonEmpty,
                Self::UniqueSegmentId,
                Self::UniqueEssenceDescriptorId,
                Self::UniqueResourceId,
                Self::IntrinsicDuration,
                Self::EntryPoint,
                Self::SourceDuration,
                Self::ResourceDuration,
                Self::RepeatCount,
                Self::TrackFileId,
                Self::VirtualTrackContinuity,
                Self::VirtualTrackEditRate,
                Self::TimedTextSampleRate,
                Self::TimedTextEmptyLanguageTag,
                Self::TimedTextMalformedLanguageTag,
                Self::AudioSampleRate,
                Self::ChannelCount,
                Self::MCASubDescriptors,
                Self::SoundfieldGroup,
                Self::MCATagSymbol,
                Self::SoundfieldChannelCount,
                Self::DigitalSignature,
                Self::DanglingEssenceDescriptor,
                Self::EssenceDescriptorList,
            ];

            /// Dispatch from the spec-agnostic [`CoreConstraintsCode`] to this
            /// edition's static code string.  Used by the shared validator helpers.
            pub fn for_code(r: CoreConstraintsCode) -> &'static str {
                match r {
                    CoreConstraintsCode::ResourceListEmpty                    => concat!($prefix, ":XSD/ResourceList-Empty"),
                    CoreConstraintsCode::ContentTitle                         => concat!($prefix, ":XSD/ContentTitle"),
                    CoreConstraintsCode::TotalRunningTimeFormat               => concat!($prefix, ":XSD/TotalRunningTime-Format"),
                    CoreConstraintsCode::SegmentList                          => concat!($prefix, ":XSD/SegmentList"),
                    CoreConstraintsCode::Segment                              => concat!($prefix, ":XSD/Segment"),
                    CoreConstraintsCode::EditRate                             => concat!($prefix, ":XSD/CompositionPlaylist/EditRate"),
                    CoreConstraintsCode::IssueDate                            => concat!($prefix, ":XSD/CompositionPlaylist/IssueDate"),
                    CoreConstraintsCode::IssueDateFormat                      => concat!($prefix, ":XSD/CompositionPlaylist/IssueDate-Format"),
                    CoreConstraintsCode::CompositionTimecodeDropFrame         => concat!($prefix, ":XSD/CompositionTimecode/DropFrame"),
                    CoreConstraintsCode::CompositionTimecodeRate              => concat!($prefix, ":XSD/CompositionTimecode/Rate"),
                    CoreConstraintsCode::CompositionTimecodeStartAddress      => concat!($prefix, ":XSD/CompositionTimecode/StartAddress"),
                    CoreConstraintsCode::CompositionTimecodeRateZero          => concat!($prefix, ":XSD/CompositionTimecode/Rate-Zero"),
                    CoreConstraintsCode::CompositionTimecodeStartAddressFormat => concat!($prefix, ":XSD/CompositionTimecode/StartAddress-Format"),
                    CoreConstraintsCode::CompositionTimecodeRateMismatch      => concat!($prefix, ":XSD/CompositionTimecode/Rate-Mismatch"),
                    CoreConstraintsCode::LocaleListNonEmpty                   => concat!($prefix, ":XSD/LocaleList-NonEmpty"),
                    CoreConstraintsCode::UniqueSegmentId                      => concat!($prefix, ":6.1/UniqueSegmentId"),
                    CoreConstraintsCode::UniqueEssenceDescriptorId            => concat!($prefix, ":6.1/UniqueEssenceDescriptorId"),
                    CoreConstraintsCode::UniqueResourceId                     => concat!($prefix, ":6.1/UniqueResourceId"),
                    CoreConstraintsCode::IntrinsicDuration                    => concat!($prefix, ":6.10/IntrinsicDuration"),
                    CoreConstraintsCode::EntryPoint                           => concat!($prefix, ":6.10/EntryPoint"),
                    CoreConstraintsCode::SourceDuration                       => concat!($prefix, ":6.10/SourceDuration"),
                    CoreConstraintsCode::ResourceDuration                     => concat!($prefix, ":6.10/ResourceDuration"),
                    CoreConstraintsCode::RepeatCount                          => concat!($prefix, ":6.10/RepeatCount"),
                    CoreConstraintsCode::TrackFileId                          => concat!($prefix, ":6.10/TrackFileId"),
                    CoreConstraintsCode::VirtualTrackContinuity               => concat!($prefix, ":6.9/VirtualTrackContinuity"),
                    CoreConstraintsCode::VirtualTrackEditRate                 => concat!($prefix, ":6.9.3/VirtualTrackEditRate"),
                    CoreConstraintsCode::TimedTextSampleRate                  => concat!($prefix, ":10/TimedText-SampleRate"),
                    CoreConstraintsCode::TimedTextEmptyLanguageTag            => concat!($prefix, ":10/TimedText-EmptyLanguageTag"),
                    CoreConstraintsCode::TimedTextMalformedLanguageTag        => concat!($prefix, ":10/TimedText-MalformedLanguageTag"),
                    CoreConstraintsCode::AudioSampleRate                      => concat!($prefix, ":ST377-4/AudioSampleRate"),
                    CoreConstraintsCode::ChannelCount                         => concat!($prefix, ":ST377-4/ChannelCount"),
                    CoreConstraintsCode::MCASubDescriptors                    => concat!($prefix, ":ST377-4/MCASubDescriptors"),
                    CoreConstraintsCode::SoundfieldGroup                      => concat!($prefix, ":ST377-4/SoundfieldGroup"),
                    CoreConstraintsCode::MCATagSymbol                         => concat!($prefix, ":ST377-4/MCATagSymbol"),
                    CoreConstraintsCode::SoundfieldChannelCount               => concat!($prefix, ":ST377-4/SoundfieldChannelCount"),
                    CoreConstraintsCode::DigitalSignature                     => concat!($prefix, ":8/DigitalSignature"),
                    CoreConstraintsCode::DanglingEssenceDescriptor            => concat!($prefix, ":6.4.2/DanglingEssenceDescriptor"),
                    CoreConstraintsCode::EssenceDescriptorList                => concat!($prefix, ":6.4.2/EssenceDescriptorList"),
                }
            }
        }

        impl From<$name> for String {
            fn from(c: $name) -> String {
                c.code().to_string()
            }
        }
    };
}

define_core_constraints_enum!(St2067_2_2013_Core, "ST2067-2:2013");
// 2016 catalogue is bit-for-bit identical to 2013 (audit, 2026-06-04).
define_core_constraints_enum!(St2067_2_2016_Core, "ST2067-2:2016", "ST2067-2:2013");
define_core_constraints_enum!(St2067_2_2020_Core, "ST2067-2:2020");
