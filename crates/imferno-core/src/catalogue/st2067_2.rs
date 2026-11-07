//! Rule table: SMPTE ST 2067-2 (Core Constraints).
//!
//! Every row checked against the text of its edition (C1/C4).

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::Strength::{May, Shall, Should};
use crate::diagnostics::rule::{absent, checked, AnyRule, Rule, Spec};
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

/// A Packing List document is not well-formed XML.
pub static PklMalformedXml: Rule<E> = Rule {
    name: "PklMalformedXml",
    category: Category::Structure,
    description: "A Packing List document is not well-formed XML.",
    example: Some("PKL.xml with mismatched tag: `<PackingList>…</packinglist>`"),
    editions: &[
        checked(E::V2013, "7.1", Shall),
        checked(E::V2016, "K", Shall),
        checked(E::V2020, "J", Shall),
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
        absent(E::V2013).verified(),
        checked(E::V2016, "7.2.1", May),
        checked(E::V2020, "7.2", May),
    ],
};

/// Declared file size does not match the on-disk size.
pub static SizeMismatch: Rule<E> = Rule {
    name: "SizeMismatch",
    category: Category::Asset,
    description: "Declared file size does not match the on-disk size.",
    example: Some("PKL `<Size>1024</Size>` but the actual file is 2048 bytes on disk"),
    editions: &[
        absent(E::V2013).verified(),
        checked(E::V2016, "K", Shall),
        checked(E::V2020, "J", Shall),
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
        checked(E::V2013, "7.1", Shall),
        checked(E::V2016, "7.2.1", Shall).interpret("7.2.1 defines an IMP as the Packing List and all the assets it references; a referenced asset missing at its mapped path leaves the IMP incomplete.", None),
        checked(E::V2020, "7.2", Shall).interpret("7.2 defines an IMP as the Packing List and all the assets it references; a referenced asset missing at its mapped path leaves the IMP incomplete.", None),
    ],
};

/// File hash does not match the declared SHA-1/SHA-256 checksum.
pub static ChecksumMismatch: Rule<E> = Rule {
    name: "ChecksumMismatch",
    category: Category::Asset,
    description: "File hash does not match the declared SHA-1/SHA-256 checksum.",
    example: Some("PKL `<Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>` (SHA-1 of empty string) on a non-empty file"),
    editions: &[
        absent(E::V2013).verified(),
        checked(E::V2016, "K", Shall),
        checked(E::V2020, "J", Shall),
    ],
};

/// An asset listed in the Packing List is not in the AssetMap.
pub static UnresolvedUuid: Rule<E> = Rule {
    name: "UnresolvedUuid",
    category: Category::Reference,
    description: "An asset listed in the Packing List is not in the AssetMap.",
    example: Some("PKL `<Asset><Id>urn:uuid:aaa…</Id>` with no AssetMap asset of that Id"),
    editions: &[
        absent(E::V2013).verified(),
        checked(E::V2016, "8.3", Should).interpret("8.3: assets intended to be ingested should be referenced by both the Packing List and the Asset Map.", None),
        checked(E::V2020, "8.3", Should).interpret("8.3: assets intended to be ingested should be referenced by both the Packing List and the Asset Map (Note 6: otherwise not ingested).", None),
    ],
};

/// A CPL track file is not in this package; an IMP need not contain complete compositions (e.g. a supplemental package referring to the original version).
pub static TrackFileNotInPackage: Rule<E> = Rule {
    name: "TrackFileNotInPackage",
    category: Category::Reference,
    description: "A CPL track file is not in this package; an IMP need not contain complete compositions (e.g. a supplemental package referring to the original version).",
    example: Some("Supplemental package whose CPL TrackFileId urn:uuid:aaa… is a track file of the original version package"),
    editions: &[
        checked(E::V2013, "7.6", May),
        checked(E::V2016, "7.2.3", May),
        checked(E::V2020, "7.2", May),
    ],
};

/// Two or more assets within the package share the same UUID.
pub static DuplicateUuid: Rule<E> = Rule {
    name: "DuplicateUuid",
    category: Category::Reference,
    description: "Two or more assets within the package share the same UUID.",
    example: Some("Two AssetMap `<Asset><Id>urn:uuid:00…01</Id></Asset>` entries with the same Id"),
    editions: &[
        checked(E::V2013, "7.2", Shall),
        checked(E::V2016, "7.3.1", Shall),
        checked(E::V2020, "7.3.1", Shall),
    ],
};

/// PKL document namespace is not one of the published SMPTE PKL namespaces.
pub static PklUnknownNamespace: Rule<E> = Rule {
    name: "PklUnknownNamespace",
    category: Category::Structure,
    description: "PKL document namespace is not one of the published SMPTE PKL namespaces.",
    example: Some("PKL with `xmlns=\"http://example.org/not-a-pkl-namespace\"`"),
    editions: &[
        checked(E::V2013, "7.1", Shall),
        checked(E::V2016, "K", Shall),
        checked(E::V2020, "J", Shall),
    ],
};

/// All resources in a virtual track shall have the same edit rate.
pub static VirtualTrackEditRate: Rule<E> = Rule {
    name: "VirtualTrackEditRate",
    category: Category::Timing,
    description: "All resources in a virtual track shall have the same edit rate.",
    example: Some("Track resources mix <EditRate>24 1</EditRate> and <EditRate>25 1</EditRate>"),
    editions: &[
        checked(E::V2013, "6.3", Shall).interpret("Resource EditRate equals the frame rate of the image essence (6.3.1), which equals the Composition Edit Rate (6.4), and the sampling rate of the audio essence (6.3.2), which is constant within a Composition (6.2.1); so every Resource of an image or audio Virtual Track has one EditRate.", None),
        checked(E::V2016, "6.3", Shall).interpret("Resource EditRate equals the frame rate of the image essence (6.3.1), which equals the Composition Edit Rate (6.4), and the sampling rate of the audio essence (6.3.2), which is constant within a Composition (6.2.1); so every Resource of an image or audio Virtual Track has one EditRate.", None),
        checked(E::V2020, "6.3", Shall).interpret("Resource EditRate equals the frame rate of the image essence (6.3.1), which equals the Composition Edit Rate (6.4), and the sampling rate of the audio essence (6.3.2), which is constant within a Composition (6.2.1); so every Resource of an image or audio Virtual Track has one EditRate.", None),
    ],
};

/// DCTimedTextDescriptor SampleRate is missing.
pub static TimedTextSampleRate: Rule<E> = Rule {
    name: "TimedTextSampleRate",
    category: Category::Subtitle,
    description: "DCTimedTextDescriptor SampleRate is missing.",
    example: Some("<DCTimedTextDescriptor>…</DCTimedTextDescriptor>  <!-- no SampleRate -->"),
    editions: &[
        checked(E::V2013, "5.4.2", Shall),
        checked(E::V2016, "5.4.2", Shall),
        checked(E::V2020, "5.4.2", Shall),
    ],
};

/// Empty language tag in RFC5646LanguageTagList.
pub static TimedTextEmptyLanguageTag: Rule<E> = Rule {
    name: "TimedTextEmptyLanguageTag",
    category: Category::Subtitle,
    description: "Empty language tag in RFC5646LanguageTagList.",
    example: Some("<RFC5646LanguageTagList><RFC5646LanguageTag></RFC5646LanguageTag></RFC5646LanguageTagList>"),
    editions: &[
        checked(E::V2013, "5.4.4", Shall),
        checked(E::V2016, "5.4.4", Shall),
        checked(E::V2020, "5.4.4", Shall),
    ],
};

/// Language tag does not start with an ASCII letter (RFC 5646 primary subtag).
pub static TimedTextMalformedLanguageTag: Rule<E> = Rule {
    name: "TimedTextMalformedLanguageTag",
    category: Category::Subtitle,
    description: "Language tag does not start with an ASCII letter (RFC 5646 primary subtag).",
    example: Some("<RFC5646LanguageTag>123-en</RFC5646LanguageTag>  <!-- primary subtag must start with a letter -->"),
    editions: &[
        checked(E::V2013, "5.4.4", Shall),
        checked(E::V2016, "5.4.4", Shall),
        checked(E::V2020, "5.4.4", Shall),
    ],
};

/// WAVEPCMDescriptor has no AudioSampleRate or SampleRate.
pub static AudioSampleRate: Rule<E> = Rule {
    name: "AudioSampleRate",
    category: Category::Audio,
    description: "WAVEPCMDescriptor has no AudioSampleRate or SampleRate.",
    example: Some("<WAVEPCMDescriptor>…</WAVEPCMDescriptor>  <!-- no AudioSampleRate -->"),
    editions: &[
        checked(E::V2013, "5.3.4.1", Shall),
        checked(E::V2016, "5.3.4.1", Shall),
        checked(E::V2020, "5.3.4.1", Shall),
    ],
};

/// WAVEPCMDescriptor ChannelCount is zero or missing.
pub static ChannelCount: Rule<E> = Rule {
    name: "ChannelCount",
    category: Category::Audio,
    description: "WAVEPCMDescriptor ChannelCount is missing.",
    example: Some("<WAVEPCMDescriptor>…</WAVEPCMDescriptor>  <!-- no <ChannelCount> -->"),
    editions: &[
        checked(E::V2013, "5.3.4.1", Shall),
        checked(E::V2016, "5.3.4.1", Shall),
        checked(E::V2020, "5.3.4.1", Shall),
    ],
};

/// WAVEPCMDescriptor ChannelCount is zero.
///
/// Split from `ChannelCount` in C0: the old site reported a zero count as
/// an error and an absent count as a warning under one code.
pub static ChannelCountZero: Rule<E> = Rule {
    name: "ChannelCountZero",
    category: Category::Audio,
    description: "WAVEPCMDescriptor ChannelCount is zero.",
    example: Some("<WAVEPCMDescriptor><ChannelCount>0</ChannelCount></WAVEPCMDescriptor>"),
    editions: &[
        checked(E::V2013, "5.3.2.1", Shall),
        checked(E::V2016, "5.3.2.1", Shall),
        checked(E::V2020, "5.3.2.1", Shall),
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
        checked(E::V2013, "5.3.6.2", Shall),
        checked(E::V2016, "5.3.6.2", Shall),
        checked(E::V2020, "5.3.6.2", Shall),
    ],
};

/// WAVEPCMDescriptor SubDescriptors missing SoundfieldGroupLabelSubDescriptor.
pub static SoundfieldGroup: Rule<E> = Rule {
    name: "SoundfieldGroup",
    category: Category::Audio,
    description: "WAVEPCMDescriptor SubDescriptors missing SoundfieldGroupLabelSubDescriptor.",
    example: Some("<WAVEPCMDescriptor><SubDescriptors>…</SubDescriptors></WAVEPCMDescriptor>  <!-- but no SoundfieldGroupLabelSubDescriptor inside -->"),
    editions: &[
        checked(E::V2013, "5.3.6.3", Shall),
        checked(E::V2016, "5.3.6.3", Shall),
        checked(E::V2020, "5.3.6.3", Shall),
    ],
};

/// SoundfieldGroupLabelSubDescriptor is missing MCATagSymbol.
pub static MCATagSymbol: Rule<E> = Rule {
    name: "MCATagSymbol",
    category: Category::Audio,
    description: "SoundfieldGroupLabelSubDescriptor is missing MCATagSymbol.",
    example: Some("<SoundfieldGroupLabelSubDescriptor>…</SoundfieldGroupLabelSubDescriptor>  <!-- no <MCATagSymbol> -->"),
    editions: &[
        checked(E::V2013, "5.3.6.1", Shall),
        checked(E::V2016, "5.3.6.1", Shall),
        checked(E::V2020, "5.3.6.1", Shall),
    ],
};

/// Soundfield group channel count is inconsistent with WAVEPCMDescriptor.ChannelCount.
pub static SoundfieldChannelCount: Rule<E> = Rule {
    name: "SoundfieldChannelCount",
    category: Category::Audio,
    description: "WAVEPCMDescriptor.ChannelCount exceeds the number of channels of its soundfield group (a subset is allowed).",
    example: Some("<WAVEPCMDescriptor><ChannelCount>8</ChannelCount>…</WAVEPCMDescriptor> but the SoundfieldGroupLabelSubDescriptor MCATagSymbol is sg51 (6 channels)"),
    editions: &[
        checked(E::V2013, "5.3.6.6", Shall),
        checked(E::V2016, "5.3.6.6", Shall),
        checked(E::V2020, "5.3.6.6", Shall),
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
        checked(E::V2013, "6.9", Shall).severity(Severity::Info).reason("A notice that the §6.9 signature constraints are not verified, not a finding that they are violated."),
        checked(E::V2016, "6.9", Shall).severity(Severity::Info).reason("A notice that the §6.9 signature constraints are not verified, not a finding that they are violated."),
        checked(E::V2020, "6.9", Shall).severity(Severity::Info).reason("A notice that the §6.9 signature constraints are not verified, not a finding that they are violated."),
    ],
};

/// EssenceDescriptorList absent although the CPL references Track Files.
pub static EssenceDescriptorList: Rule<E> = Rule {
    name: "EssenceDescriptorList",
    category: Category::Structure,
    description: "EssenceDescriptorList is absent: the File Descriptor of every Track File the CPL references shall be mapped to an EssenceDescriptor (§6.8).",
    example: Some(
        "CPL with `<SourceEncoding>` references but no top-level `<EssenceDescriptorList>` element",
    ),
    editions: &[
        checked(E::V2013, "6.8", Shall),
        checked(E::V2016, "6.8", Shall),
        checked(E::V2020, "6.8", Shall),
    ],
};

/// An AssetMap asset points at a CPL file whose Id differs from the asset Id.
pub static CplIdAssetMapMismatch: Rule<E> = Rule {
    name: "CplIdAssetMapMismatch",
    category: Category::Reference,
    description: "An AssetMap asset points at a CPL file whose Id differs from the asset Id.",
    example: Some("AssetMap `<Id>urn:uuid:aaa…</Id>` with `<Path>CPL.xml</Path>`, but CPL.xml carries `<Id>urn:uuid:bbb…</Id>`"),
    editions: &[
        checked(E::V2013, "7.2", Shall).interpret("The AssetMap asset Id that locates a CPL must equal the CPL Id (Table 18).", Some("https://github.com/Netflix/photon/issues/265")),
        checked(E::V2016, "7.3.1", Shall).interpret("The AssetMap asset Id that locates a CPL must equal the CPL Id (Table 19).", Some("https://github.com/Netflix/photon/issues/265")),
        checked(E::V2020, "7.3.1", Shall).interpret("The AssetMap asset Id that locates a CPL must equal the CPL Id (Table 19).", Some("https://github.com/Netflix/photon/issues/265")),
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
        checked(E::V2013, "5.1.1", Shall).interpret(
            "5.1.1 requires Track Files to conform to SMPTE ST 2067-5, which mandates OP1a.",
            None,
        ),
        checked(E::V2016, "5.1.1", Shall).interpret(
            "5.1.1 requires Track Files to conform to SMPTE ST 2067-5, which mandates OP1a.",
            None,
        ),
        checked(E::V2020, "5.1.1", Shall).interpret(
            "5.1.1 requires Track Files to conform to SMPTE ST 2067-5, which mandates OP1a.",
            None,
        ),
    ],
};

/// Audio essence descriptor must be WAVEPCMDescriptor.
pub static SoundDescriptorNotWAVEPCM: Rule<E> = Rule {
    name: "SoundDescriptorNotWAVEPCM",
    category: Category::Audio,
    description: "Audio essence descriptor must be WAVEPCMDescriptor.",
    example: Some("Audio essence descriptor is AES3PCMDescriptor instead of WAVEPCMDescriptor"),
    editions: &[
        checked(E::V2013, "5.3.4.1", Shall),
        checked(E::V2016, "5.3.4.1", Shall),
        checked(E::V2020, "5.3.4.1", Shall),
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
        checked(E::V2013, "5.3.2.2", Shall),
        checked(E::V2016, "5.3.2.2", Shall),
        checked(E::V2020, "5.3.2.2", Shall),
    ],
};

/// QuantizationBits must be 24.
pub static QuantizationBitsNot24: Rule<E> = Rule {
    name: "QuantizationBitsNot24",
    category: Category::Audio,
    description: "QuantizationBits must be 24.",
    example: Some("<QuantizationBits>16</QuantizationBits>"),
    editions: &[
        checked(E::V2013, "5.3.2.3", Shall).interpret(
            "24-bit samples are required.",
            Some("https://github.com/Netflix/photon/issues/343"),
        ),
        checked(E::V2016, "5.3.2.3", Shall).interpret(
            "24-bit samples are required.",
            Some("https://github.com/Netflix/photon/issues/343"),
        ),
        checked(E::V2020, "5.3.2.3", Shall).interpret(
            "24-bit samples are required.",
            Some("https://github.com/Netflix/photon/issues/343"),
        ),
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
        checked(E::V2013, "5.3.6.2", Shall),
        checked(E::V2016, "5.3.6.2", Shall),
        checked(E::V2020, "5.3.6.2", Shall),
    ],
};

/// MCAChannelID shall be present per channel (channel 1 may omit it) — §5.3.6.5 Table 7.
pub static MCAChannelIDMissing: Rule<E> = Rule {
    name: "MCAChannelIDMissing",
    category: Category::Audio,
    description: "MCAChannelID shall be present per channel (channel 1 may omit it) — §5.3.6.5 Table 7.",
    example: Some("ChannelCount=6 but no AudioChannelLabelSubDescriptor carries MCAChannelID=4 (channels 2..N must be identified)"),
    editions: &[
        absent(E::V2013).verified(),
        checked(E::V2016, "5.3.6.5", Shall).interpret("MCA Channel ID is decoded by value; ST 377-4 Table 3 and the register disagree on its size.", Some("https://github.com/SMPTE/ST377-4-Revision-and-ST377-41/issues/8")),
        checked(E::V2020, "5.3.6.5", Shall).interpret("MCA Channel ID is decoded by value; ST 377-4 Table 3 and the register disagree on its size.", Some("https://github.com/SMPTE/ST377-4-Revision-and-ST377-41/issues/8")),
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
        checked(E::V2013, "5.3.6.3", Shall),
        checked(E::V2016, "5.3.6.3", Shall),
        checked(E::V2020, "5.3.6.3", Shall),
    ],
};

/// Every AudioChannelLabelSubDescriptor shall carry a SoundfieldGroupLinkID (§5.3.6.5 Table 7).
pub static SoundfieldGroupLinkIDMissing: Rule<E> = Rule {
    name: "SoundfieldGroupLinkIDMissing",
    category: Category::Audio,
    description: "Every AudioChannelLabelSubDescriptor shall carry a SoundfieldGroupLinkID (§5.3.6.5 Table 7).",
    example: Some("<AudioChannelLabelSubDescriptor>…</AudioChannelLabelSubDescriptor>  <!-- no <SoundfieldGroupLinkID> child -->"),
    editions: &[
        checked(E::V2013, "5.3.6.5", Shall),
        checked(E::V2016, "5.3.6.5", Shall),
        checked(E::V2020, "5.3.6.5", Shall),
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
        checked(E::V2013, "5.3.3", Shall),
        checked(E::V2016, "5.3.3", Shall),
        checked(E::V2020, "5.3.3", Shall),
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
        checked(E::V2013, "5.3.6.5", Shall).severity(Severity::Warning).reason("The item shall be absent when the soundfield group has no primary spoken language, which imferno cannot tell; its absence is reported for review."),
        checked(E::V2016, "5.3.6.5", Shall).severity(Severity::Warning).reason("The item shall be absent when the soundfield group has no primary spoken language, which imferno cannot tell; its absence is reported for review."),
        checked(E::V2020, "5.3.6.5", Shall).severity(Severity::Warning).reason("The item shall be absent when the soundfield group has no primary spoken language, which imferno cannot tell; its absence is reported for review."),
    ],
};

/// ChannelAssignment shall be present on the audio essence descriptor (§5.3.4.2).
pub static ChannelAssignmentMissing: Rule<E> = Rule {
    name: "ChannelAssignmentMissing",
    category: Category::Audio,
    description: "ChannelAssignment shall be present on the audio essence descriptor (§5.3.4.2).",
    example: Some("WAVEPCMDescriptor with no <ChannelAssignment> item"),
    editions: &[
        checked(E::V2013, "5.3.4.2", Shall),
        checked(E::V2016, "5.3.4.2", Shall),
        checked(E::V2020, "5.3.4.2", Shall),
    ],
};

/// ChannelAssignment shall equal the ST 2067-2 Table 5 channel-assignment label UL.
pub static ChannelAssignmentNotMCA: Rule<E> = Rule {
    name: "ChannelAssignmentNotMCA",
    category: Category::Audio,
    description: "ChannelAssignment shall equal the ST 2067-2 Table 5 channel-assignment label UL.",
    example: Some("ChannelAssignment UL with byte 13 = 05h (ADM labeling) instead of the Table 5 value 04h on a plain IMF audio file"),
    editions: &[
        checked(E::V2013, "5.3.4.2", Shall),
        checked(E::V2016, "5.3.4.2", Shall),
        checked(E::V2020, "5.3.4.2", Shall),
    ],
};

/// SoundfieldGroupLabelSubDescriptor shall carry MCATitle (§5.3.6.5 Table 7).
pub static SoundfieldGroupMissingMCATitle: Rule<E> = Rule {
    name: "SoundfieldGroupMissingMCATitle",
    category: Category::Audio,
    description: "SoundfieldGroupLabelSubDescriptor shall carry MCATitle (§5.3.6.5 Table 7).",
    example: Some("SoundfieldGroupLabelSubDescriptor with no <MCATitle> item"),
    editions: &[
        checked(E::V2013, "5.3.6.5", Shall).interpret(
            "Table 7: required on the SoundfieldGroupLabelSubDescriptor.",
            Some("https://github.com/Netflix/photon/issues/138"),
        ),
        checked(E::V2016, "5.3.6.5", Shall).interpret(
            "Table 7: required on the SoundfieldGroupLabelSubDescriptor.",
            Some("https://github.com/Netflix/photon/issues/138"),
        ),
        checked(E::V2020, "5.3.6.5", Shall).interpret(
            "Table 7: required on the SoundfieldGroupLabelSubDescriptor.",
            Some("https://github.com/Netflix/photon/issues/138"),
        ),
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
        checked(E::V2013, "5.3.6.5", Shall).interpret(
            "Table 7: required on the SoundfieldGroupLabelSubDescriptor.",
            Some("https://github.com/Netflix/photon/issues/138"),
        ),
        checked(E::V2016, "5.3.6.5", Shall).interpret(
            "Table 7: required on the SoundfieldGroupLabelSubDescriptor.",
            Some("https://github.com/Netflix/photon/issues/138"),
        ),
        checked(E::V2020, "5.3.6.5", Shall).interpret(
            "Table 7: required on the SoundfieldGroupLabelSubDescriptor.",
            Some("https://github.com/Netflix/photon/issues/138"),
        ),
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
        checked(E::V2013, "5.3.6.5", Shall).interpret(
            "Table 7: required on the SoundfieldGroupLabelSubDescriptor.",
            Some("https://github.com/Netflix/photon/issues/138"),
        ),
        checked(E::V2016, "5.3.6.5", Shall).interpret(
            "Table 7: required on the SoundfieldGroupLabelSubDescriptor.",
            Some("https://github.com/Netflix/photon/issues/138"),
        ),
        checked(E::V2020, "5.3.6.5", Shall).interpret(
            "Table 7: required on the SoundfieldGroupLabelSubDescriptor.",
            Some("https://github.com/Netflix/photon/issues/138"),
        ),
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
        checked(E::V2013, "5.3.6.5", Shall).interpret(
            "Table 7: required on the SoundfieldGroupLabelSubDescriptor.",
            Some("https://github.com/Netflix/photon/issues/138"),
        ),
        checked(E::V2016, "5.3.6.5", Shall).interpret(
            "Table 7: required on the SoundfieldGroupLabelSubDescriptor.",
            Some("https://github.com/Netflix/photon/issues/138"),
        ),
        checked(E::V2020, "5.3.6.5", Shall).interpret(
            "Table 7: required on the SoundfieldGroupLabelSubDescriptor.",
            Some("https://github.com/Netflix/photon/issues/138"),
        ),
    ],
};

/// TimedTextDescriptor UCSEncoding must be UTF-8.
pub static TimedTextUCSEncodingNotUTF8: Rule<E> = Rule {
    name: "TimedTextUCSEncodingNotUTF8",
    category: Category::Subtitle,
    description: "TimedTextDescriptor UCSEncoding must be UTF-8.",
    example: Some("<UCSEncoding>ISO-8859-1</UCSEncoding>"),
    editions: &[
        checked(E::V2013, "5.4.2", Shall),
        checked(E::V2016, "4.3", Shall),
        checked(E::V2020, "4.3", Shall),
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
        absent(E::V2013).verified(),
        checked(E::V2016, "5.4.2", Shall),
        checked(E::V2020, "5.4.2", Shall).interpret("2020 §5.4.2 Note: NamespaceURI may hold designators of superset profiles beyond IMSC 1.1; only an empty value is reported.", None),
    ],
};

/// TimeTextResourceSubDescriptor MIMEType must be image/png, font/otf (2020) or application/x-font-opentype.
pub static TimedTextResourceMIMETypeUnsupported: Rule<E> = Rule {
    name: "TimedTextResourceMIMETypeUnsupported",
    category: Category::Subtitle,
    description: "TimeTextResourceSubDescriptor MIMEType must be image/png, font/otf (2020) or application/x-font-opentype.",
    example: Some("<TimeTextResourceSubDescriptor><MIMEType>application/json</MIMEType></TimeTextResourceSubDescriptor>"),
    editions: &[
        checked(E::V2013, "5.4.5", Shall).interpret("Also enforces the font-resource MIME type of 5.4.6.", None),
        checked(E::V2016, "5.4.5", Shall).interpret("Also enforces the font-resource MIME type of 5.4.6.", None),
        checked(E::V2020, "5.4.5", Shall).interpret("Also enforces the font-resource MIME type of 5.4.6.", None),
    ],
};

/// Timed-text ContainerFormat UL Mapping Kind byte must be 0x13 (IMSC).
pub static TimedTextMappingKindNot0x13: Rule<E> = Rule {
    name: "TimedTextMappingKindNot0x13",
    category: Category::Container,
    description: "Timed-text Essence Container UL Mapping Kind byte (byte 14) must be 13h, Timed Text (ST 429-5 Annex A Table A.1).",
    example: Some(
        "Timed-text ContainerFormat UL byte 14 (Mapping Kind) = 0x12 instead of 0x13 (IMSC)",
    ),
    editions: &[
        checked(E::V2013, "5.4.2", Shall).interpret("5.4.2 requires Data Essence Track Files to conform to SMPTE ST 429-5 (normative reference: ST 429-5:2009), whose normative Annex A Table A.1 sets Mapping Kind 13h (Timed Text) in the essence container label. It applies to the SMPTE-TT track files of 2013 as well.", None),
        checked(E::V2016, "5.4.2", Shall).interpret("5.4.2 requires Data Essence Track Files to conform to SMPTE ST 429-5 (normative reference: ST 429-5:2009), whose normative Annex A Table A.1 sets Mapping Kind 13h (Timed Text) in the essence container label.", None),
        checked(E::V2020, "5.4.2", Shall).interpret("5.4.2 requires Timed Text Track Files to conform to SMPTE ST 429-5 (normative reference: ST 429-5:2017), whose normative Annex A Table A.1 sets Mapping Kind 13h (Timed Text) in the essence container label.", None),
    ],
};

/// The Composition does not have exactly one Main Image Virtual Track.
pub static MainImageVirtualTrackCount: Rule<E> = Rule {
    name: "MainImageVirtualTrackCount",
    category: Category::Structure,
    description: "A Composition shall contain exactly one Main Image Virtual Track.",
    example: Some("<SequenceList/>  <!-- no MainImageSequence in any Segment -->"),
    editions: &[
        checked(E::V2013, "6.3.1", Shall),
        checked(E::V2016, "6.3.1", Shall),
        checked(E::V2020, "6.3.1", Shall),
    ],
};

/// The Composition has no Audio Virtual Track (2013 and 2016 only).
pub static AudioVirtualTrackMissing: Rule<E> = Rule {
    name: "AudioVirtualTrackMissing",
    category: Category::Structure,
    description: "A Composition shall contain one or more Audio Virtual Tracks (ST 2067-2:2013 and 2016; 2020 allows zero).",
    example: Some("A CPL whose Segments contain no MainAudioSequence"),
    editions: &[
        checked(E::V2013, "6.3.2", Shall).interpret("Dropped in 2067-2:2020 (zero or more Audio Virtual Tracks).", Some("https://github.com/Netflix/photon/issues/227")),
        checked(E::V2016, "6.3.2", Shall).interpret("Dropped in 2067-2:2020 (zero or more Audio Virtual Tracks).", Some("https://github.com/Netflix/photon/issues/227")),
        absent(E::V2020).verified(),
    ],
};

/// The CPL is not an IMF Composition Playlist: its namespace is not an
/// ST 2067-3 namespace (a D-Cinema ST 429-7 CPL, or an unknown namespace).
pub static NotImfCompositionPlaylist: Rule<E> = Rule {
    name: "NotImfCompositionPlaylist",
    category: Category::Structure,
    description: "The CPL is not an IMF Composition Playlist: its namespace is not an ST 2067-3 namespace (e.g. a D-Cinema ST 429-7 CPL). No ST 2067-3, Core Constraints or application rules are applied to it.",
    example: Some("<CompositionPlaylist xmlns=\"http://www.smpte-ra.org/schemas/429-7/2006/CPL\">  <!-- a D-Cinema CPL in an IMP -->"),
    editions: &[
        checked(E::V2013, "6", Shall).interpret("2013 has no definition sentence: §6 constrains the Composition Playlist instance, whose schema §4.1 imports from the ST 2067-3:2013 namespace. The text has no shall for this; Error per JP (a CPL that is not an ST 2067-3 CPL is not part of a Composition).", None),
        checked(E::V2016, "6.11", Shall).interpret("§6.11 defines a Composition as one Composition Playlist as specified in ST 2067-3 and the Assets it references. A definition, not a shall sentence; Error per JP.", None),
        checked(E::V2020, "6.11", Shall).interpret("§6.11 defines a Composition as one Composition Playlist as specified in ST 2067-3 and the Assets it references. A definition, not a shall sentence; Error per JP.", None),
    ],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[
    &NotImfCompositionPlaylist,
    &PklMalformedXml,
    &NoCpls,
    &SizeMismatch,
    &FileNotFound,
    &ChecksumMismatch,
    &UnresolvedUuid,
    &TrackFileNotInPackage,
    &DuplicateUuid,
    &PklUnknownNamespace,
    &VirtualTrackEditRate,
    &TimedTextSampleRate,
    &TimedTextEmptyLanguageTag,
    &TimedTextMalformedLanguageTag,
    &AudioSampleRate,
    &ChannelCount,
    &ChannelCountZero,
    &MCASubDescriptors,
    &SoundfieldGroup,
    &MCATagSymbol,
    &SoundfieldChannelCount,
    &DigitalSignature,
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
    &MainImageVirtualTrackCount,
    &AudioVirtualTrackMissing,
];
