//! Rule table: SMPTE ST 2067-2 (Core Constraints).
//!
//! Generated from the pre-C0 per-edition code enums; edit by hand from now on.

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::{absent, row, AnyRule, Rule, Spec};
use crate::diagnostics::{Category, Severity};

crate::diagnostics::rule::editions! {
    /// Editions of SMPTE ST 2067-2 (Core Constraints) imferno knows.
    pub enum St2067_2Edition for Spec::St2067_2 {
        V2013 => "2013",
        V2016 => "2016",
        V2020 => "2020",
    }
}

type E = St2067_2Edition;

/// AssetMap document is invalid or cannot be parsed.
pub static AssetMap: Rule<E> = Rule {
    name: "AssetMap",
    category: Category::Structure,
    description: "AssetMap document is invalid or cannot be parsed.",
    example: Some("ASSETMAP.xml exists but parse_assetmap returns Err (e.g. wrong root element)"),
    editions: &[
        absent(E::V2013),
        row(E::V2016, "8.1", Severity::Critical)
            .reason("Prevents the package or file from being processed further (pre-C0 catalogue severity)."),
        row(E::V2020, "7", Severity::Critical)
            .reason("Prevents the package or file from being processed further (pre-C0 catalogue severity)."),
    ],
};

/// The ASSETMAP.xml document is not well-formed XML.
pub static AssetMapMalformedXml: Rule<E> = Rule {
    name: "AssetMapMalformedXml",
    category: Category::Structure,
    description: "The ASSETMAP.xml document is not well-formed XML.",
    example: Some("ASSETMAP.xml truncated mid-element: `<AssetMap><Id>urn:uuid:…</Id>`"),
    editions: &[
        absent(E::V2013),
        row(E::V2016, "8.1", Severity::Error),
        row(E::V2020, "7", Severity::Error),
    ],
};

/// A Packing List document is not well-formed XML.
pub static PklMalformedXml: Rule<E> = Rule {
    name: "PklMalformedXml",
    category: Category::Structure,
    description: "A Packing List document is not well-formed XML.",
    example: Some("PKL.xml with mismatched tag: `<PackingList>…</packinglist>`"),
    editions: &[
        row(E::V2013, "7.1", Severity::Error),
        row(E::V2016, "K", Severity::Error),
        row(E::V2020, "9", Severity::Error).unverified(),
    ],
};

/// No CPL assets found in the AssetMap.
pub static NoCpls: Rule<E> = Rule {
    name: "NoCpls",
    category: Category::Structure,
    description: "No CPL assets found in the AssetMap.",
    example: Some(
        "ASSETMAP.xml with all assets flagged `<PackingList>true</PackingList>` and zero CPLs",
    ),
    editions: &[
        absent(E::V2013),
        row(E::V2016, "7.1", Severity::Info),
        row(E::V2020, "7", Severity::Info),
    ],
};

/// Declared file size does not match the on-disk size.
pub static SizeMismatch: Rule<E> = Rule {
    name: "SizeMismatch",
    category: Category::Asset,
    description: "Declared file size does not match the on-disk size.",
    example: Some("PKL `<Size>1024</Size>` but the actual file is 2048 bytes on disk"),
    editions: &[
        absent(E::V2013),
        row(E::V2016, "8.3", Severity::Error),
        row(E::V2020, "8.3", Severity::Error),
    ],
};

/// A referenced asset file is not present at the declared path.
pub static FileNotFound: Rule<E> = Rule {
    name: "FileNotFound",
    category: Category::Asset,
    description: "A referenced asset file is not present at the declared path.",
    example: Some(
        "AssetMap `<Path>missing-track.mxf</Path>` but no such file exists in the package",
    ),
    editions: &[
        row(E::V2013, "7.1", Severity::Error),
        row(E::V2016, "8.3", Severity::Error),
        row(E::V2020, "8.3", Severity::Error),
    ],
};

/// File hash does not match the declared SHA-1/SHA-256 checksum.
pub static ChecksumMismatch: Rule<E> = Rule {
    name: "ChecksumMismatch",
    category: Category::Asset,
    description: "File hash does not match the declared SHA-1/SHA-256 checksum.",
    example: Some("PKL `<Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>` (SHA-1 of empty string) on a non-empty file"),
    editions: &[
        absent(E::V2013),
        row(E::V2016, "8.3", Severity::Error),
        row(E::V2020, "8.3", Severity::Error),
    ],
};

/// An asset listed in the Packing List is not in the AssetMap.
pub static UnresolvedUuid: Rule<E> = Rule {
    name: "UnresolvedUuid",
    category: Category::Reference,
    description: "An asset listed in the Packing List is not in the AssetMap.",
    example: Some("PKL `<Asset><Id>urn:uuid:aaa…</Id>` with no AssetMap asset of that Id"),
    editions: &[
        absent(E::V2013),
        row(E::V2016, "8.3", Severity::Error),
        row(E::V2020, "7", Severity::Error),
    ],
};

/// A CPL track file is not in this package; an IMP need not contain complete compositions (e.g. a supplemental package referring to the original version).
pub static TrackFileNotInPackage: Rule<E> = Rule {
    name: "TrackFileNotInPackage",
    category: Category::Reference,
    description: "A CPL track file is not in this package; an IMP need not contain complete compositions (e.g. a supplemental package referring to the original version).",
    example: Some("Supplemental package whose CPL TrackFileId urn:uuid:aaa… is a track file of the original version package"),
    editions: &[
        row(E::V2013, "7.6", Severity::Info),
        row(E::V2016, "7.1", Severity::Info),
        row(E::V2020, "7.1", Severity::Info),
    ],
};

/// Two or more assets within the package share the same UUID.
pub static DuplicateUuid: Rule<E> = Rule {
    name: "DuplicateUuid",
    category: Category::Reference,
    description: "Two or more assets within the package share the same UUID.",
    example: Some("Two AssetMap `<Asset><Id>urn:uuid:00…01</Id></Asset>` entries with the same Id"),
    editions: &[
        row(E::V2013, "7.2", Severity::Error),
        row(E::V2016, "7.3.1", Severity::Error),
        row(E::V2020, "7", Severity::Error),
    ],
};

/// PKL document namespace is not one of the published SMPTE PKL namespaces.
pub static PklUnknownNamespace: Rule<E> = Rule {
    name: "PklUnknownNamespace",
    category: Category::Structure,
    description: "PKL document namespace is not one of the published SMPTE PKL namespaces.",
    example: Some("PKL with `xmlns=\"http://example.org/not-a-pkl-namespace\"`"),
    editions: &[
        row(E::V2013, "7.1", Severity::Error),
        row(E::V2016, "K", Severity::Error),
        row(E::V2020, "9", Severity::Error).unverified(),
    ],
};

/// A Sequence has an empty ResourceList.
pub static ResourceListEmpty: Rule<E> = Rule {
    name: "ResourceListEmpty",
    category: Category::Structure,
    description: "A Sequence has an empty ResourceList.",
    example: Some("<Sequence><ResourceList></ResourceList></Sequence>"),
    editions: &[
        row(E::V2013, "XSD", Severity::Error).unverified(),
        row(E::V2016, "XSD", Severity::Error).unverified(),
        row(E::V2020, "XSD", Severity::Error).unverified(),
    ],
};

/// ContentTitle shall not be empty.
pub static ContentTitle: Rule<E> = Rule {
    name: "ContentTitle",
    category: Category::Metadata,
    description: "ContentTitle shall not be empty.",
    example: Some("<ContentTitle></ContentTitle>"),
    editions: &[
        row(E::V2013, "XSD", Severity::Error).unverified(),
        row(E::V2016, "XSD", Severity::Error).unverified(),
        row(E::V2020, "XSD", Severity::Error).unverified(),
    ],
};

/// TotalRunningTime does not match required format HH:MM:SS.
pub static TotalRunningTimeFormat: Rule<E> = Rule {
    name: "TotalRunningTimeFormat",
    category: Category::Structure,
    description: "TotalRunningTime does not match required format HH:MM:SS.",
    example: Some("<TotalRunningTime>1h23m</TotalRunningTime>"),
    editions: &[
        row(E::V2013, "XSD", Severity::Error).unverified(),
        row(E::V2016, "XSD", Severity::Error).unverified(),
        row(E::V2020, "XSD", Severity::Error).unverified(),
    ],
};

/// SegmentList shall contain at least one Segment.
pub static SegmentList: Rule<E> = Rule {
    name: "SegmentList",
    category: Category::Structure,
    description: "SegmentList shall contain at least one Segment.",
    example: Some("<SegmentList></SegmentList>"),
    editions: &[
        row(E::V2013, "XSD", Severity::Critical).unverified()
            .reason("Prevents the package or file from being processed further (pre-C0 catalogue severity)."),
        row(E::V2016, "XSD", Severity::Critical).unverified()
            .reason("Prevents the package or file from being processed further (pre-C0 catalogue severity)."),
        row(E::V2020, "XSD", Severity::Critical).unverified()
            .reason("Prevents the package or file from being processed further (pre-C0 catalogue severity)."),
    ],
};

/// A Segment contains no sequences.
pub static Segment: Rule<E> = Rule {
    name: "Segment",
    category: Category::Structure,
    description: "A Segment contains no sequences.",
    example: Some("<Segment><SequenceList></SequenceList></Segment>"),
    editions: &[
        row(E::V2013, "XSD", Severity::Error).unverified(),
        row(E::V2016, "XSD", Severity::Error).unverified(),
        row(E::V2020, "XSD", Severity::Error).unverified(),
    ],
};

/// CPL EditRate is required (XSD schema §88).
pub static EditRate: Rule<E> = Rule {
    name: "EditRate",
    category: Category::Structure,
    description: "CPL EditRate is required (XSD schema §88).",
    example: Some("<CompositionPlaylist>…</CompositionPlaylist>  <!-- no <EditRate> element -->"),
    editions: &[
        row(E::V2013, "XSD", Severity::Error).unverified(),
        row(E::V2016, "XSD", Severity::Error).unverified(),
        row(E::V2020, "XSD", Severity::Error).unverified(),
    ],
};

/// IssueDate shall not be empty.
pub static IssueDate: Rule<E> = Rule {
    name: "IssueDate",
    category: Category::Metadata,
    description: "IssueDate shall not be empty.",
    example: Some("<IssueDate></IssueDate>"),
    editions: &[
        row(E::V2013, "XSD", Severity::Error).unverified(),
        row(E::V2016, "XSD", Severity::Error).unverified(),
        row(E::V2020, "XSD", Severity::Error).unverified(),
    ],
};

/// IssueDate is not a valid xs:dateTime format.
pub static IssueDateFormat: Rule<E> = Rule {
    name: "IssueDateFormat",
    category: Category::Metadata,
    description: "IssueDate is not a valid xs:dateTime format.",
    example: Some("<IssueDate>2024/01/01</IssueDate>  <!-- not xs:dateTime -->"),
    editions: &[
        row(E::V2013, "XSD", Severity::Warning).unverified(),
        row(E::V2016, "XSD", Severity::Warning).unverified(),
        row(E::V2020, "XSD", Severity::Warning).unverified(),
    ],
};

/// CompositionTimecode.TimecodeDropFrame is required when CompositionTimecode is present.
pub static CompositionTimecodeDropFrame: Rule<E> = Rule {
    name: "CompositionTimecodeDropFrame",
    category: Category::Timing,
    description: "CompositionTimecode.TimecodeDropFrame is required when CompositionTimecode is present.",
    example: Some("<CompositionTimecode><TimecodeRate>24</TimecodeRate>…</CompositionTimecode>  <!-- no <TimecodeDropFrame> -->"),
    editions: &[
        row(E::V2013, "XSD", Severity::Error).unverified(),
        row(E::V2016, "XSD", Severity::Error).unverified(),
        row(E::V2020, "XSD", Severity::Error).unverified(),
    ],
};

/// CompositionTimecode.TimecodeRate is required when CompositionTimecode is present.
pub static CompositionTimecodeRate: Rule<E> = Rule {
    name: "CompositionTimecodeRate",
    category: Category::Timing,
    description: "CompositionTimecode.TimecodeRate is required when CompositionTimecode is present.",
    example: Some("<CompositionTimecode><TimecodeDropFrame>false</TimecodeDropFrame>…</CompositionTimecode>  <!-- no <TimecodeRate> -->"),
    editions: &[
        row(E::V2013, "XSD", Severity::Error).unverified(),
        row(E::V2016, "XSD", Severity::Error).unverified(),
        row(E::V2020, "XSD", Severity::Error).unverified(),
    ],
};

/// CompositionTimecode.TimecodeStartAddress is required when CompositionTimecode is present.
pub static CompositionTimecodeStartAddress: Rule<E> = Rule {
    name: "CompositionTimecodeStartAddress",
    category: Category::Timing,
    description: "CompositionTimecode.TimecodeStartAddress is required when CompositionTimecode is present.",
    example: Some("<CompositionTimecode><TimecodeRate>24</TimecodeRate><TimecodeDropFrame>false</TimecodeDropFrame></CompositionTimecode>  <!-- no <TimecodeStartAddress> -->"),
    editions: &[
        row(E::V2013, "XSD", Severity::Error).unverified(),
        row(E::V2016, "XSD", Severity::Error).unverified(),
        row(E::V2020, "XSD", Severity::Error).unverified(),
    ],
};

/// CompositionTimecode.TimecodeRate shall be a positive integer.
pub static CompositionTimecodeRateZero: Rule<E> = Rule {
    name: "CompositionTimecodeRateZero",
    category: Category::Timing,
    description: "CompositionTimecode.TimecodeRate shall be a positive integer.",
    example: Some("<TimecodeRate>0</TimecodeRate>"),
    editions: &[
        row(E::V2013, "XSD", Severity::Error).unverified(),
        row(E::V2016, "XSD", Severity::Error).unverified(),
        row(E::V2020, "XSD", Severity::Error).unverified(),
    ],
};

/// TimecodeStartAddress does not match SMPTE timecode format HH:MM:SS:FF.
pub static CompositionTimecodeStartAddressFormat: Rule<E> = Rule {
    name: "CompositionTimecodeStartAddressFormat",
    category: Category::Timing,
    description: "TimecodeStartAddress does not match SMPTE timecode format HH:MM:SS:FF.",
    example: Some(
        "<TimecodeStartAddress>1:2:3</TimecodeStartAddress>  <!-- expected HH:MM:SS:FF -->",
    ),
    editions: &[
        row(E::V2013, "XSD", Severity::Error).unverified(),
        row(E::V2016, "XSD", Severity::Error).unverified(),
        row(E::V2020, "XSD", Severity::Error).unverified(),
    ],
};

/// CompositionTimecode.TimecodeRate does not match the CPL EditRate.
pub static CompositionTimecodeRateMismatch: Rule<E> = Rule {
    name: "CompositionTimecodeRateMismatch",
    category: Category::Metadata,
    description: "CompositionTimecode.TimecodeRate does not match the CPL EditRate.",
    example: Some("CPL <EditRate>24 1</EditRate> but <TimecodeRate>25</TimecodeRate>"),
    editions: &[
        row(E::V2013, "XSD", Severity::Warning).unverified(),
        row(E::V2016, "XSD", Severity::Warning).unverified(),
        row(E::V2020, "XSD", Severity::Warning).unverified(),
    ],
};

/// LocaleList shall contain at least one Locale.
pub static LocaleListNonEmpty: Rule<E> = Rule {
    name: "LocaleListNonEmpty",
    category: Category::Structure,
    description: "LocaleList shall contain at least one Locale.",
    example: Some("<LocaleList></LocaleList>"),
    editions: &[
        row(E::V2013, "XSD", Severity::Error).unverified(),
        row(E::V2016, "XSD", Severity::Error).unverified(),
        row(E::V2020, "XSD", Severity::Error).unverified(),
    ],
};

/// Duplicate Segment Id within the CPL.
pub static UniqueSegmentId: Rule<E> = Rule {
    name: "UniqueSegmentId",
    category: Category::Structure,
    description: "Duplicate Segment Id within the CPL.",
    example: Some("Two <Segment><Id>urn:uuid:abc…</Id></Segment> with the same Id"),
    editions: &[
        row(E::V2013, "6.1", Severity::Error).unverified(),
        row(E::V2016, "6.1", Severity::Error).unverified(),
        row(E::V2020, "6.1", Severity::Error).unverified(),
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
        row(E::V2013, "6.1", Severity::Error).unverified(),
        row(E::V2016, "6.1", Severity::Error).unverified(),
        row(E::V2020, "6.1", Severity::Error).unverified(),
    ],
};

/// Duplicate Resource Id within the CPL.
pub static UniqueResourceId: Rule<E> = Rule {
    name: "UniqueResourceId",
    category: Category::Structure,
    description: "Duplicate Resource Id within the CPL.",
    example: Some("Two <Resource><Id>urn:uuid:abc…</Id></Resource> with the same Id"),
    editions: &[
        row(E::V2013, "6.1", Severity::Error).unverified(),
        row(E::V2016, "6.1", Severity::Error).unverified(),
        row(E::V2020, "6.1", Severity::Error).unverified(),
    ],
};

/// IntrinsicDuration shall be greater than 0.
pub static IntrinsicDuration: Rule<E> = Rule {
    name: "IntrinsicDuration",
    category: Category::Timing,
    description: "IntrinsicDuration shall be greater than 0.",
    example: Some("<IntrinsicDuration>0</IntrinsicDuration>"),
    editions: &[
        row(E::V2013, "6.10", Severity::Error).unverified(),
        row(E::V2016, "6.10", Severity::Error).unverified(),
        row(E::V2020, "6.10", Severity::Error).unverified(),
    ],
};

/// EntryPoint shall be less than IntrinsicDuration.
pub static EntryPoint: Rule<E> = Rule {
    name: "EntryPoint",
    category: Category::Timing,
    description: "EntryPoint shall be less than IntrinsicDuration.",
    example: Some("<IntrinsicDuration>100</IntrinsicDuration><EntryPoint>150</EntryPoint>"),
    editions: &[
        row(E::V2013, "6.10", Severity::Error).unverified(),
        row(E::V2016, "6.10", Severity::Error).unverified(),
        row(E::V2020, "6.10", Severity::Error).unverified(),
    ],
};

/// EntryPoint + SourceDuration exceeds IntrinsicDuration.
pub static ResourceDuration: Rule<E> = Rule {
    name: "ResourceDuration",
    category: Category::Timing,
    description: "EntryPoint + SourceDuration exceeds IntrinsicDuration.",
    example: Some("<IntrinsicDuration>100</IntrinsicDuration><EntryPoint>50</EntryPoint><SourceDuration>80</SourceDuration>  <!-- 50+80 > 100 -->"),
    editions: &[
        row(E::V2013, "6.10", Severity::Error).unverified(),
        row(E::V2016, "6.10", Severity::Error).unverified(),
        row(E::V2020, "6.10", Severity::Error).unverified(),
    ],
};

/// RepeatCount shall be a positive integer.
pub static RepeatCount: Rule<E> = Rule {
    name: "RepeatCount",
    category: Category::Timing,
    description: "RepeatCount shall be a positive integer.",
    example: Some("<RepeatCount>0</RepeatCount>"),
    editions: &[
        row(E::V2013, "6.10", Severity::Error).unverified(),
        row(E::V2016, "6.10", Severity::Error).unverified(),
        row(E::V2020, "6.10", Severity::Error).unverified(),
    ],
};

/// A non-marker resource is missing a TrackFileId.
pub static TrackFileId: Rule<E> = Rule {
    name: "TrackFileId",
    category: Category::Reference,
    description: "A non-marker resource is missing a TrackFileId.",
    example: Some("<Resource xsi:type=\"TrackFileResourceType\">…</Resource>  <!-- no <TrackFileId> child -->"),
    editions: &[
        row(E::V2013, "6.10", Severity::Error).unverified(),
        row(E::V2016, "6.10", Severity::Error).unverified(),
        row(E::V2020, "6.10", Severity::Error).unverified(),
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
        row(E::V2013, "6.9", Severity::Error).unverified(),
        row(E::V2016, "6.9", Severity::Error).unverified(),
        row(E::V2020, "6.9", Severity::Error).unverified(),
    ],
};

/// All resources in a virtual track shall have the same edit rate.
pub static VirtualTrackEditRate: Rule<E> = Rule {
    name: "VirtualTrackEditRate",
    category: Category::Timing,
    description: "All resources in a virtual track shall have the same edit rate.",
    example: Some("Track resources mix <EditRate>24 1</EditRate> and <EditRate>25 1</EditRate>"),
    editions: &[
        row(E::V2013, "6.9.3", Severity::Error).unverified(),
        row(E::V2016, "6.9.3", Severity::Error).unverified(),
        row(E::V2020, "6.9.3", Severity::Error).unverified(),
    ],
};

/// DCTimedTextDescriptor SampleRate is missing.
pub static TimedTextSampleRate: Rule<E> = Rule {
    name: "TimedTextSampleRate",
    category: Category::Subtitle,
    description: "DCTimedTextDescriptor SampleRate is missing.",
    example: Some("<DCTimedTextDescriptor>…</DCTimedTextDescriptor>  <!-- no SampleRate -->"),
    editions: &[
        row(E::V2013, "10", Severity::Warning).unverified(),
        row(E::V2016, "10", Severity::Warning).unverified(),
        row(E::V2020, "10", Severity::Warning).unverified(),
    ],
};

/// Empty language tag in RFC5646LanguageTagList.
pub static TimedTextEmptyLanguageTag: Rule<E> = Rule {
    name: "TimedTextEmptyLanguageTag",
    category: Category::Subtitle,
    description: "Empty language tag in RFC5646LanguageTagList.",
    example: Some("<RFC5646LanguageTagList><RFC5646LanguageTag></RFC5646LanguageTag></RFC5646LanguageTagList>"),
    editions: &[
        row(E::V2013, "10", Severity::Warning).unverified(),
        row(E::V2016, "10", Severity::Warning).unverified(),
        row(E::V2020, "10", Severity::Warning).unverified(),
    ],
};

/// Language tag does not start with an ASCII letter (RFC 5646 primary subtag).
pub static TimedTextMalformedLanguageTag: Rule<E> = Rule {
    name: "TimedTextMalformedLanguageTag",
    category: Category::Subtitle,
    description: "Language tag does not start with an ASCII letter (RFC 5646 primary subtag).",
    example: Some("<RFC5646LanguageTag>123-en</RFC5646LanguageTag>  <!-- primary subtag must start with a letter -->"),
    editions: &[
        row(E::V2013, "10", Severity::Warning).unverified(),
        row(E::V2016, "10", Severity::Warning).unverified(),
        row(E::V2020, "10", Severity::Warning).unverified(),
    ],
};

/// WAVEPCMDescriptor has no AudioSampleRate or SampleRate.
pub static AudioSampleRate: Rule<E> = Rule {
    name: "AudioSampleRate",
    category: Category::Audio,
    description: "WAVEPCMDescriptor has no AudioSampleRate or SampleRate.",
    example: Some("<WAVEPCMDescriptor>…</WAVEPCMDescriptor>  <!-- no AudioSampleRate -->"),
    editions: &[
        row(E::V2013, "ST377-4", Severity::Warning).unverified(),
        row(E::V2016, "ST377-4", Severity::Warning).unverified(),
        row(E::V2020, "ST377-4", Severity::Warning).unverified(),
    ],
};

/// WAVEPCMDescriptor ChannelCount is zero or missing.
pub static ChannelCount: Rule<E> = Rule {
    name: "ChannelCount",
    category: Category::Audio,
    description: "WAVEPCMDescriptor ChannelCount is zero or missing.",
    example: Some("<WAVEPCMDescriptor><ChannelCount>0</ChannelCount></WAVEPCMDescriptor>"),
    editions: &[
        row(E::V2013, "ST377-4", Severity::Warning).unverified(),
        row(E::V2016, "ST377-4", Severity::Warning).unverified(),
        row(E::V2020, "ST377-4", Severity::Warning).unverified(),
    ],
};

/// WAVEPCMDescriptor has no MCA SubDescriptors.
pub static MCASubDescriptors: Rule<E> = Rule {
    name: "MCASubDescriptors",
    category: Category::Audio,
    description: "WAVEPCMDescriptor has no MCA SubDescriptors.",
    example: Some(
        "<WAVEPCMDescriptor>…</WAVEPCMDescriptor>  <!-- no SubDescriptors strong-ref list -->",
    ),
    editions: &[
        row(E::V2013, "ST377-4", Severity::Warning).unverified(),
        row(E::V2016, "ST377-4", Severity::Warning).unverified(),
        row(E::V2020, "ST377-4", Severity::Warning).unverified(),
    ],
};

/// WAVEPCMDescriptor SubDescriptors missing SoundfieldGroupLabelSubDescriptor.
pub static SoundfieldGroup: Rule<E> = Rule {
    name: "SoundfieldGroup",
    category: Category::Audio,
    description: "WAVEPCMDescriptor SubDescriptors missing SoundfieldGroupLabelSubDescriptor.",
    example: Some("<WAVEPCMDescriptor><SubDescriptors>…</SubDescriptors></WAVEPCMDescriptor>  <!-- but no SoundfieldGroupLabelSubDescriptor inside -->"),
    editions: &[
        row(E::V2013, "ST377-4", Severity::Warning).unverified(),
        row(E::V2016, "ST377-4", Severity::Warning).unverified(),
        row(E::V2020, "ST377-4", Severity::Warning).unverified(),
    ],
};

/// SoundfieldGroupLabelSubDescriptor is missing MCATagSymbol.
pub static MCATagSymbol: Rule<E> = Rule {
    name: "MCATagSymbol",
    category: Category::Audio,
    description: "SoundfieldGroupLabelSubDescriptor is missing MCATagSymbol.",
    example: Some("<SoundfieldGroupLabelSubDescriptor>…</SoundfieldGroupLabelSubDescriptor>  <!-- no <MCATagSymbol> -->"),
    editions: &[
        row(E::V2013, "ST377-4", Severity::Warning).unverified(),
        row(E::V2016, "ST377-4", Severity::Warning).unverified(),
        row(E::V2020, "ST377-4", Severity::Warning).unverified(),
    ],
};

/// Soundfield group channel count is inconsistent with WAVEPCMDescriptor.ChannelCount.
pub static SoundfieldChannelCount: Rule<E> = Rule {
    name: "SoundfieldChannelCount",
    category: Category::Audio,
    description: "Soundfield group channel count is inconsistent with WAVEPCMDescriptor.ChannelCount.",
    example: Some("<WAVEPCMDescriptor><ChannelCount>2</ChannelCount>…</WAVEPCMDescriptor> but soundfield group declares 6 channels"),
    editions: &[
        row(E::V2013, "ST377-4", Severity::Error).unverified(),
        row(E::V2016, "ST377-4", Severity::Error).unverified(),
        row(E::V2020, "ST377-4", Severity::Error).unverified(),
    ],
};

/// The CPL carries a Signature (ST 2067-2 §6.9); signature verification is not performed.
pub static DigitalSignature: Rule<E> = Rule {
    name: "DigitalSignature",
    category: Category::Security,
    description:
        "The CPL carries a Signature (ST 2067-2 §6.9); signature verification is not performed.",
    example: Some(
        "CPL declares Signer/Signature but the engine does not yet verify them (info notice)",
    ),
    editions: &[
        row(E::V2013, "6.9", Severity::Info).unverified(),
        row(E::V2016, "6.9", Severity::Info).unverified(),
        row(E::V2020, "6.9", Severity::Info).unverified(),
    ],
};

/// EssenceDescriptor present in EssenceDescriptorList but not referenced by any Resource.
pub static DanglingEssenceDescriptor: Rule<E> = Rule {
    name: "DanglingEssenceDescriptor",
    category: Category::Reference,
    description: "EssenceDescriptor present in EssenceDescriptorList but not referenced by any Resource.",
    example: Some("<EssenceDescriptor><Id>urn:uuid:abc…</Id></EssenceDescriptor>  <!-- no Resource references this Id -->"),
    editions: &[
        row(E::V2013, "6.4.2", Severity::Error).unverified(),
        row(E::V2016, "6.4.2", Severity::Error).unverified(),
        row(E::V2020, "6.4.2", Severity::Error).unverified(),
    ],
};

/// EssenceDescriptorList is required per ST 2067-2 §6.4.2.
pub static EssenceDescriptorList: Rule<E> = Rule {
    name: "EssenceDescriptorList",
    category: Category::Structure,
    description: "EssenceDescriptorList is required per ST 2067-2 §6.4.2.",
    example: Some(
        "CPL with `<SourceEncoding>` references but no top-level `<EssenceDescriptorList>` element",
    ),
    editions: &[
        row(E::V2013, "6.4.2", Severity::Error).unverified(),
        row(E::V2016, "6.4.2", Severity::Error).unverified(),
        row(E::V2020, "6.4.2", Severity::Error).unverified(),
    ],
};

/// An AssetMap asset points at a CPL file whose Id differs from the asset Id.
pub static CplIdAssetMapMismatch: Rule<E> = Rule {
    name: "CplIdAssetMapMismatch",
    category: Category::Reference,
    description: "An AssetMap asset points at a CPL file whose Id differs from the asset Id.",
    example: Some("AssetMap `<Id>urn:uuid:aaa…</Id>` with `<Path>CPL.xml</Path>`, but CPL.xml carries `<Id>urn:uuid:bbb…</Id>`"),
    editions: &[
        row(E::V2013, "7.2", Severity::Error).unverified(),
        row(E::V2016, "7.3.1", Severity::Error).unverified(),
        row(E::V2020, "7.3.1", Severity::Error).unverified(),
    ],
};

/// MXF essence Operational Pattern UL is not OP1a (ST 378:2004).
pub static OperationalPatternNotOP1A: Rule<E> = Rule {
    name: "OperationalPatternNotOP1A",
    category: Category::Container,
    description: "MXF essence Operational Pattern UL is not OP1a (ST 378:2004).",
    example: Some(
        "Header partition pack OP UL bytes 13..14 = 02 03 (OP2a) instead of OP1a (01 01)",
    ),
    editions: &[
        row(E::V2013, "5.1.1", Severity::Error),
        row(E::V2016, "5.2", Severity::Error).unverified(),
        row(E::V2020, "5.1.1", Severity::Error),
    ],
};

/// Audio essence descriptor must be WAVEPCMDescriptor.
pub static SoundDescriptorNotWAVEPCM: Rule<E> = Rule {
    name: "SoundDescriptorNotWAVEPCM",
    category: Category::Audio,
    description: "Audio essence descriptor must be WAVEPCMDescriptor.",
    example: Some("Audio essence descriptor is AES3PCMDescriptor instead of WAVEPCMDescriptor"),
    editions: &[
        row(E::V2013, "5.3.4.1", Severity::Error),
        row(E::V2016, "5.3.4.1", Severity::Error),
        row(E::V2020, "5.3.4.1", Severity::Error),
    ],
};

/// AudioSampleRate must be 48000 Hz or 96000 Hz.
pub static AudioSampleRateUnsupported: Rule<E> = Rule {
    name: "AudioSampleRateUnsupported",
    category: Category::Audio,
    description: "AudioSampleRate must be 48000 Hz or 96000 Hz.",
    example: Some(
        "<AudioSampleRate>44100/1</AudioSampleRate>  <!-- only 48000 or 96000 allowed -->",
    ),
    editions: &[
        row(E::V2013, "5.3.2.2", Severity::Error),
        row(E::V2016, "5.3.2.2", Severity::Error),
        row(E::V2020, "5.3.2.2", Severity::Error),
    ],
};

/// QuantizationBits must be 24.
pub static QuantizationBitsNot24: Rule<E> = Rule {
    name: "QuantizationBitsNot24",
    category: Category::Audio,
    description: "QuantizationBits must be 24.",
    example: Some("<QuantizationBits>16</QuantizationBits>"),
    editions: &[
        row(E::V2013, "5.3.2.3", Severity::Error),
        row(E::V2016, "5.3.2.3", Severity::Error),
        row(E::V2020, "5.3.2.3", Severity::Error),
    ],
};

/// Number of AudioChannelLabelSubDescriptors must equal ChannelCount.
pub static ChannelLabelCountMismatch: Rule<E> = Rule {
    name: "ChannelLabelCountMismatch",
    category: Category::Audio,
    description: "Number of AudioChannelLabelSubDescriptors must equal ChannelCount.",
    example: Some(
        "<ChannelCount>6</ChannelCount> but only 2 AudioChannelLabelSubDescriptor entries present",
    ),
    editions: &[
        row(E::V2013, "5.3.6.2", Severity::Error),
        row(E::V2016, "5.3.6.2", Severity::Error),
        row(E::V2020, "5.3.6.2", Severity::Error),
    ],
};

/// MCAChannelID shall be present per channel (channel 1 may omit it) — §5.3.6.5 Table 7.
pub static MCAChannelIDMissing: Rule<E> = Rule {
    name: "MCAChannelIDMissing",
    category: Category::Audio,
    description: "MCAChannelID shall be present per channel (channel 1 may omit it) — §5.3.6.5 Table 7.",
    example: Some("ChannelCount=6 but no AudioChannelLabelSubDescriptor carries MCAChannelID=4 (channels 2..N must be identified)"),
    editions: &[
        absent(E::V2013),
        row(E::V2016, "5.3.6.5", Severity::Error),
        row(E::V2020, "5.3.6.5", Severity::Error),
    ],
};

/// Exactly one SoundfieldGroupLabelSubDescriptor is required per WAVEPCMDescriptor.
pub static SoundFieldGroupLabelCount: Rule<E> = Rule {
    name: "SoundFieldGroupLabelCount",
    category: Category::Audio,
    description: "Exactly one SoundfieldGroupLabelSubDescriptor is required per WAVEPCMDescriptor.",
    example: Some(
        "WAVEPCMDescriptor with 2 SoundfieldGroupLabelSubDescriptor children (must be exactly 1)",
    ),
    editions: &[
        row(E::V2013, "5.3.6.3", Severity::Error),
        row(E::V2016, "5.3.6.3", Severity::Error),
        row(E::V2020, "5.3.6.3", Severity::Error),
    ],
};

/// Every AudioChannelLabelSubDescriptor shall carry a SoundfieldGroupLinkID (§5.3.6.5 Table 7).
pub static SoundfieldGroupLinkIDMissing: Rule<E> = Rule {
    name: "SoundfieldGroupLinkIDMissing",
    category: Category::Audio,
    description: "Every AudioChannelLabelSubDescriptor shall carry a SoundfieldGroupLinkID (§5.3.6.5 Table 7).",
    example: Some("<AudioChannelLabelSubDescriptor>…</AudioChannelLabelSubDescriptor>  <!-- no <SoundfieldGroupLinkID> child -->"),
    editions: &[
        row(E::V2013, "5.3.6.5", Severity::Error),
        row(E::V2016, "5.3.6.5", Severity::Error),
        row(E::V2020, "5.3.6.5", Severity::Error),
    ],
};

/// Audio essence must be Wave Clip-Wrapped (ST 382 §10).
pub static AudioNotClipWrapped: Rule<E> = Rule {
    name: "AudioNotClipWrapped",
    category: Category::Container,
    description: "Audio essence must be Wave Clip-Wrapped (ST 382 §10).",
    example: Some(
        "ContainerFormat UL byte 14 = 0x02 (frame-wrapped) instead of 0x06 (clip-wrapped)",
    ),
    editions: &[
        row(E::V2013, "5.3.3", Severity::Error),
        row(E::V2016, "5.3.3", Severity::Error),
        row(E::V2020, "5.3.3", Severity::Error),
    ],
};

/// Audio descriptor should carry an RFC-5646 spoken language tag.
pub static RFC5646SpokenLanguageMissing: Rule<E> = Rule {
    name: "RFC5646SpokenLanguageMissing",
    category: Category::Audio,
    description: "Audio descriptor should carry an RFC-5646 spoken language tag.",
    example: Some(
        "No RFC5646SpokenLanguage tag on any AudioChannelLabel/SoundfieldGroupLabel sub-descriptor",
    ),
    editions: &[
        row(E::V2013, "5.3.6.5", Severity::Warning),
        row(E::V2016, "5.3", Severity::Warning),
        row(E::V2020, "5.3.6.5", Severity::Warning),
    ],
};

/// ChannelAssignment shall be present on the audio essence descriptor (§5.3.4.2).
pub static ChannelAssignmentMissing: Rule<E> = Rule {
    name: "ChannelAssignmentMissing",
    category: Category::Audio,
    description: "ChannelAssignment shall be present on the audio essence descriptor (§5.3.4.2).",
    example: Some("WAVEPCMDescriptor with no <ChannelAssignment> item"),
    editions: &[
        row(E::V2013, "5.3.4.2", Severity::Error),
        row(E::V2016, "5.3.4.2", Severity::Error),
        row(E::V2020, "5.3.4.2", Severity::Error),
    ],
};

/// ChannelAssignment shall equal the ST 2067-2 Table 5 channel-assignment label UL.
pub static ChannelAssignmentNotMCA: Rule<E> = Rule {
    name: "ChannelAssignmentNotMCA",
    category: Category::Audio,
    description: "ChannelAssignment shall equal the ST 2067-2 Table 5 channel-assignment label UL.",
    example: Some("ChannelAssignment UL with byte 13 = 05h (ADM labeling) instead of the Table 5 value 04h on a plain IMF audio file"),
    editions: &[
        row(E::V2013, "5.3.4.2", Severity::Error),
        row(E::V2016, "5.3.4.2", Severity::Error),
        row(E::V2020, "5.3.4.2", Severity::Error),
    ],
};

/// SoundfieldGroupLabelSubDescriptor shall carry MCATitle (§5.3.6.5 Table 7).
pub static SoundfieldGroupMissingMCATitle: Rule<E> = Rule {
    name: "SoundfieldGroupMissingMCATitle",
    category: Category::Audio,
    description: "SoundfieldGroupLabelSubDescriptor shall carry MCATitle (§5.3.6.5 Table 7).",
    example: Some("SoundfieldGroupLabelSubDescriptor with no <MCATitle> item"),
    editions: &[
        row(E::V2013, "5.3.6.5", Severity::Error),
        row(E::V2016, "5.3.6.5", Severity::Error),
        row(E::V2020, "5.3.6.5", Severity::Error),
    ],
};

/// SoundfieldGroupLabelSubDescriptor shall carry MCATitleVersion (§5.3.6.5 Table 7).
pub static SoundfieldGroupMissingMCATitleVersion: Rule<E> = Rule {
    name: "SoundfieldGroupMissingMCATitleVersion",
    category: Category::Audio,
    description:
        "SoundfieldGroupLabelSubDescriptor shall carry MCATitleVersion (§5.3.6.5 Table 7).",
    example: Some("SoundfieldGroupLabelSubDescriptor with no <MCATitleVersion> item"),
    editions: &[
        row(E::V2013, "5.3.6.5", Severity::Error),
        row(E::V2016, "5.3.6.5", Severity::Error),
        row(E::V2020, "5.3.6.5", Severity::Error),
    ],
};

/// SoundfieldGroupLabelSubDescriptor shall carry MCAAudioContentKind (§5.3.6.5 Table 7).
pub static SoundfieldGroupMissingMCAAudioContentKind: Rule<E> = Rule {
    name: "SoundfieldGroupMissingMCAAudioContentKind",
    category: Category::Audio,
    description:
        "SoundfieldGroupLabelSubDescriptor shall carry MCAAudioContentKind (§5.3.6.5 Table 7).",
    example: Some("SoundfieldGroupLabelSubDescriptor with no <MCAAudioContentKind> item"),
    editions: &[
        row(E::V2013, "5.3.6.5", Severity::Error),
        row(E::V2016, "5.3.6.5", Severity::Error),
        row(E::V2020, "5.3.6.5", Severity::Error),
    ],
};

/// SoundfieldGroupLabelSubDescriptor shall carry MCAAudioElementKind (§5.3.6.5 Table 7).
pub static SoundfieldGroupMissingMCAAudioElementKind: Rule<E> = Rule {
    name: "SoundfieldGroupMissingMCAAudioElementKind",
    category: Category::Audio,
    description:
        "SoundfieldGroupLabelSubDescriptor shall carry MCAAudioElementKind (§5.3.6.5 Table 7).",
    example: Some("SoundfieldGroupLabelSubDescriptor with no <MCAAudioElementKind> item"),
    editions: &[
        row(E::V2013, "5.3.6.5", Severity::Error),
        row(E::V2016, "5.3.6.5", Severity::Error),
        row(E::V2020, "5.3.6.5", Severity::Error),
    ],
};

/// TimedTextDescriptor UCSEncoding must be UTF-8.
pub static TimedTextUCSEncodingNotUTF8: Rule<E> = Rule {
    name: "TimedTextUCSEncodingNotUTF8",
    category: Category::Subtitle,
    description: "TimedTextDescriptor UCSEncoding must be UTF-8.",
    example: Some("<UCSEncoding>ISO-8859-1</UCSEncoding>"),
    editions: &[
        row(E::V2013, "5.4.2", Severity::Error),
        row(E::V2016, "5.4", Severity::Error),
        row(E::V2020, "4.3", Severity::Error),
    ],
};

/// TimedTextDescriptor NamespaceURI must be one of the IMSC1 profile namespaces.
pub static TimedTextNamespaceNotIMSC: Rule<E> = Rule {
    name: "TimedTextNamespaceNotIMSC",
    category: Category::Subtitle,
    description: "TimedTextDescriptor NamespaceURI must be one of the IMSC1 profile namespaces.",
    example: Some(
        "<NamespaceURI>http://www.w3.org/ns/ttml</NamespaceURI>  <!-- not an IMSC1 profile URI -->",
    ),
    editions: &[
        absent(E::V2013),
        row(E::V2016, "5.4", Severity::Error),
        row(E::V2020, "5.4.2", Severity::Error),
    ],
};

/// TimeTextResourceSubDescriptor MIMEType must be image/png, font/otf (2020) or application/x-font-opentype.
pub static TimedTextResourceMIMETypeUnsupported: Rule<E> = Rule {
    name: "TimedTextResourceMIMETypeUnsupported",
    category: Category::Subtitle,
    description: "TimeTextResourceSubDescriptor MIMEType must be image/png, font/otf (2020) or application/x-font-opentype.",
    example: Some("<TimeTextResourceSubDescriptor><MIMEType>application/json</MIMEType></TimeTextResourceSubDescriptor>"),
    editions: &[
        row(E::V2013, "5.4.5", Severity::Error),
        row(E::V2016, "5.4.5", Severity::Error),
        row(E::V2020, "5.4.5", Severity::Error),
    ],
};

/// Timed-text ContainerFormat UL Mapping Kind byte must be 0x13 (IMSC).
pub static TimedTextMappingKindNot0x13: Rule<E> = Rule {
    name: "TimedTextMappingKindNot0x13",
    category: Category::Container,
    description: "Timed-text ContainerFormat UL Mapping Kind byte must be 0x13 (IMSC).",
    example: Some(
        "Timed-text ContainerFormat UL byte 14 (Mapping Kind) = 0x12 instead of 0x13 (IMSC)",
    ),
    editions: &[
        row(E::V2013, "5.4.2", Severity::Error).unverified(),
        row(E::V2016, "5.4", Severity::Error),
        row(E::V2020, "5.4.2", Severity::Error),
    ],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[
    &AssetMap,
    &AssetMapMalformedXml,
    &PklMalformedXml,
    &NoCpls,
    &SizeMismatch,
    &FileNotFound,
    &ChecksumMismatch,
    &UnresolvedUuid,
    &TrackFileNotInPackage,
    &DuplicateUuid,
    &PklUnknownNamespace,
    &ResourceListEmpty,
    &ContentTitle,
    &TotalRunningTimeFormat,
    &SegmentList,
    &Segment,
    &EditRate,
    &IssueDate,
    &IssueDateFormat,
    &CompositionTimecodeDropFrame,
    &CompositionTimecodeRate,
    &CompositionTimecodeStartAddress,
    &CompositionTimecodeRateZero,
    &CompositionTimecodeStartAddressFormat,
    &CompositionTimecodeRateMismatch,
    &LocaleListNonEmpty,
    &UniqueSegmentId,
    &UniqueEssenceDescriptorId,
    &UniqueResourceId,
    &IntrinsicDuration,
    &EntryPoint,
    &ResourceDuration,
    &RepeatCount,
    &TrackFileId,
    &VirtualTrackContinuity,
    &VirtualTrackEditRate,
    &TimedTextSampleRate,
    &TimedTextEmptyLanguageTag,
    &TimedTextMalformedLanguageTag,
    &AudioSampleRate,
    &ChannelCount,
    &MCASubDescriptors,
    &SoundfieldGroup,
    &MCATagSymbol,
    &SoundfieldChannelCount,
    &DigitalSignature,
    &DanglingEssenceDescriptor,
    &EssenceDescriptorList,
    &CplIdAssetMapMismatch,
    &OperationalPatternNotOP1A,
    &SoundDescriptorNotWAVEPCM,
    &AudioSampleRateUnsupported,
    &QuantizationBitsNot24,
    &ChannelLabelCountMismatch,
    &MCAChannelIDMissing,
    &SoundFieldGroupLabelCount,
    &SoundfieldGroupLinkIDMissing,
    &AudioNotClipWrapped,
    &RFC5646SpokenLanguageMissing,
    &ChannelAssignmentMissing,
    &ChannelAssignmentNotMCA,
    &SoundfieldGroupMissingMCATitle,
    &SoundfieldGroupMissingMCATitleVersion,
    &SoundfieldGroupMissingMCAAudioContentKind,
    &SoundfieldGroupMissingMCAAudioElementKind,
    &TimedTextUCSEncodingNotUTF8,
    &TimedTextNamespaceNotIMSC,
    &TimedTextResourceMIMETypeUnsupported,
    &TimedTextMappingKindNot0x13,
];
