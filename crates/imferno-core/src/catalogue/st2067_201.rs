//! Rule table: SMPTE ST 2067-201 (IAB Level 0 Plug-in).
//!
//! Generated from the pre-C0 per-edition code enums; edit by hand from now on.

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::{absent, row, AnyRule, Rule, Spec};
use crate::diagnostics::{Category, Severity};

crate::diagnostics::rule::editions! {
    /// Editions of SMPTE ST 2067-201 (IAB Level 0 Plug-in) imferno knows.
    pub enum St2067_201Edition for Spec::St2067_201 {
        V2019 => "2019",
        V2021 => "2021",
        V2026 => "2026",
    }
}

type E = St2067_201Edition;

/// IABEssenceDescriptor: Codec item shall not be present (§5.9).
pub static CodecForbidden: Rule<E> = Rule {
    name: "CodecForbidden",
    category: Category::Audio,
    description: "IABEssenceDescriptor: Codec item shall not be present (§5.9).",
    example: Some(
        "IABEssenceDescriptor with a non-empty Codec UL — ST 2067-201 forbids the item entirely.",
    ),
    editions: &[
        row(E::V2019, "5.9", Severity::Error),
        row(E::V2021, "5.9", Severity::Error),
        row(E::V2026, "5.9", Severity::Error),
    ],
};

/// IABEssenceDescriptor: ElectrospatialFormulation, if present, shall be 15 — multi-channel mode default (§5.9).
pub static ElectrospatialFormulationInvalid: Rule<E> = Rule {
    name: "ElectrospatialFormulationInvalid",
    category: Category::Audio,
    description: "IABEssenceDescriptor: ElectrospatialFormulation, if present, shall be 15 — multi-channel mode default (§5.9).",
    example: Some("IABEssenceDescriptor with ElectrospatialFormulation = 1 instead of 15 (multi-channel mode default)."),
    editions: &[
        row(E::V2019, "5.9", Severity::Error),
        row(E::V2021, "5.9", Severity::Error),
        row(E::V2026, "5.9", Severity::Error),
    ],
};

/// IABEssenceDescriptor: QuantizationBits is missing; shall be 24.
pub static QuantizationBitsMissing: Rule<E> = Rule {
    name: "QuantizationBitsMissing",
    category: Category::Audio,
    description: "IABEssenceDescriptor: QuantizationBits is missing; shall be 24.",
    example: Some("IABEssenceDescriptor with no QuantizationBits item."),
    editions: &[
        row(E::V2019, "5.9", Severity::Warning),
        row(E::V2021, "5.9", Severity::Warning),
        row(E::V2026, "5.9", Severity::Warning),
    ],
};

/// IABEssenceDescriptor: QuantizationBits shall be 24.
pub static QuantizationBitsInvalid: Rule<E> = Rule {
    name: "QuantizationBitsInvalid",
    category: Category::Audio,
    description: "IABEssenceDescriptor: QuantizationBits shall be 24.",
    example: Some("IABEssenceDescriptor with QuantizationBits = 16 instead of 24."),
    editions: &[
        row(E::V2019, "5.9", Severity::Error),
        row(E::V2021, "5.9", Severity::Error),
        row(E::V2026, "5.9", Severity::Error),
    ],
};

/// IABEssenceDescriptor: ContainerFormat is missing.
pub static ContainerFormatMissing: Rule<E> = Rule {
    name: "ContainerFormatMissing",
    category: Category::Audio,
    description: "IABEssenceDescriptor: ContainerFormat is missing.",
    example: Some("IABEssenceDescriptor with no ContainerFormat (EssenceContainer) UL."),
    editions: &[
        row(E::V2019, "5.9", Severity::Warning),
        row(E::V2021, "5.9", Severity::Warning),
        row(E::V2026, "5.9", Severity::Warning),
    ],
};

/// IABEssenceDescriptor: ContainerFormat is not the required IAB container UL.
pub static EssenceContainerInvalid: Rule<E> = Rule {
    name: "EssenceContainerInvalid",
    category: Category::Audio,
    description: "IABEssenceDescriptor: ContainerFormat is not the required IAB container UL.",
    example: Some("IABEssenceDescriptor with ContainerFormat = `0d010301.020c0900` (WAV) instead of the IAB frame-wrapped UL."),
    editions: &[
        row(E::V2019, "5.9", Severity::Error),
        row(E::V2021, "5.9", Severity::Error),
        row(E::V2026, "5.9", Severity::Error),
    ],
};

/// IABEssenceDescriptor: AudioSampleRate shall be present (§5.9).
pub static AudioSamplingRateMissing: Rule<E> = Rule {
    name: "AudioSamplingRateMissing",
    category: Category::Audio,
    description: "IABEssenceDescriptor: AudioSampleRate shall be present (§5.9).",
    example: Some("IABEssenceDescriptor with no AudioSampleRate."),
    editions: &[
        row(E::V2019, "5.9", Severity::Error),
        row(E::V2021, "5.9", Severity::Error),
        row(E::V2026, "5.9", Severity::Error),
    ],
};

/// IABEssenceDescriptor: AudioSampleRate shall match the bitstream SampleRate — 48 kHz or 96 kHz per ST 2098-2.
pub static AudioSamplingRateInvalid: Rule<E> = Rule {
    name: "AudioSamplingRateInvalid",
    category: Category::Audio,
    description: "IABEssenceDescriptor: AudioSampleRate shall match the bitstream SampleRate — 48 kHz or 96 kHz per ST 2098-2.",
    example: Some("IABEssenceDescriptor with AudioSampleRate = 44100/1 — not an ST 2098-2 sampling rate (48000/1 or 96000/1)."),
    editions: &[
        row(E::V2019, "5.9", Severity::Error),
        row(E::V2021, "5.9", Severity::Error),
        row(E::V2026, "5.9", Severity::Error),
    ],
};

/// IABEssenceDescriptor: SoundCompression is missing.
pub static SoundCompressionMissing: Rule<E> = Rule {
    name: "SoundCompressionMissing",
    category: Category::Audio,
    description: "IABEssenceDescriptor: SoundCompression is missing.",
    example: Some("IABEssenceDescriptor with no SoundCompression UL."),
    editions: &[
        row(E::V2019, "5.9", Severity::Warning),
        row(E::V2021, "5.9", Severity::Warning),
        row(E::V2026, "5.9", Severity::Warning),
    ],
};

/// IABEssenceDescriptor: SoundCompression is not the required IAB compression UL.
pub static SoundCompressionInvalid: Rule<E> = Rule {
    name: "SoundCompressionInvalid",
    category: Category::Audio,
    description: "IABEssenceDescriptor: SoundCompression is not the required IAB compression UL.",
    example: Some("IABEssenceDescriptor with SoundCompression = uncompressed PCM UL instead of the IAB compression UL."),
    editions: &[
        row(E::V2019, "5.9", Severity::Error),
        row(E::V2021, "5.9", Severity::Error),
        row(E::V2026, "5.9", Severity::Error),
    ],
};

/// IABEssenceDescriptor: ChannelCount shall be the distinguished value 0 (2019 edition).
pub static ChannelCountNotZero: Rule<E> = Rule {
    name: "ChannelCountNotZero",
    category: Category::Audio,
    description: "IABEssenceDescriptor: ChannelCount shall be the distinguished value 0 (2019 edition).",
    example: Some("IABEssenceDescriptor with ChannelCount = 2 — the 2019 edition mandates the distinguished value 0."),
    editions: &[
        row(E::V2019, "5.9", Severity::Error),
        absent(E::V2021),
        absent(E::V2026),
    ],
};

/// IABEssenceDescriptor: IABSoundfieldLabelSubDescriptor shall be present.
pub static SubDescriptorMissing: Rule<E> = Rule {
    name: "SubDescriptorMissing",
    category: Category::Audio,
    description: "IABEssenceDescriptor: IABSoundfieldLabelSubDescriptor shall be present.",
    example: Some("IABEssenceDescriptor with no IABSoundfieldLabelSubDescriptor in its SubDescriptors strong-ref list."),
    editions: &[
        row(E::V2019, "5.10.2", Severity::Error),
        row(E::V2021, "5.10.2", Severity::Error),
        row(E::V2026, "5.10.2", Severity::Error),
    ],
};

/// IABSoundfieldLabelSubDescriptor: MCATagSymbol is missing; shall be "IAB".
pub static MCATagSymbolMissing: Rule<E> = Rule {
    name: "MCATagSymbolMissing",
    category: Category::Audio,
    description: "IABSoundfieldLabelSubDescriptor: MCATagSymbol is missing; shall be \"IAB\".",
    example: Some("IABSoundfieldLabelSubDescriptor with no MCATagSymbol item."),
    editions: &[
        row(E::V2019, "5.10.4", Severity::Error),
        row(E::V2021, "5.10.4", Severity::Error),
        row(E::V2026, "5.10.4", Severity::Error),
    ],
};

/// IABSoundfieldLabelSubDescriptor: MCATagSymbol shall be "IAB".
pub static MCATagSymbolInvalid: Rule<E> = Rule {
    name: "MCATagSymbolInvalid",
    category: Category::Audio,
    description: "IABSoundfieldLabelSubDescriptor: MCATagSymbol shall be \"IAB\".",
    example: Some("IABSoundfieldLabelSubDescriptor with MCATagSymbol = \"71\" instead of \"IAB\"."),
    editions: &[
        row(E::V2019, "5.10.4", Severity::Error),
        row(E::V2021, "5.10.4", Severity::Error),
        row(E::V2026, "5.10.4", Severity::Error),
    ],
};

/// IABSoundfieldLabelSubDescriptor: MCATagName is missing; shall be "IAB".
pub static MCATagNameMissing: Rule<E> = Rule {
    name: "MCATagNameMissing",
    category: Category::Audio,
    description: "IABSoundfieldLabelSubDescriptor: MCATagName is missing; shall be \"IAB\".",
    example: Some("IABSoundfieldLabelSubDescriptor with no MCATagName item."),
    editions: &[
        row(E::V2019, "5.10.3", Severity::Error),
        row(E::V2021, "5.10.3", Severity::Error),
        row(E::V2026, "5.10.3", Severity::Error),
    ],
};

/// IABSoundfieldLabelSubDescriptor: MCATagName shall be "IAB".
pub static MCATagNameInvalid: Rule<E> = Rule {
    name: "MCATagNameInvalid",
    category: Category::Audio,
    description: "IABSoundfieldLabelSubDescriptor: MCATagName shall be \"IAB\".",
    example: Some("IABSoundfieldLabelSubDescriptor with MCATagName = \"Immersive Audio Bitstream\" instead of \"IAB\"."),
    editions: &[
        row(E::V2019, "5.10.4", Severity::Error),
        row(E::V2021, "5.10.4", Severity::Error),
        row(E::V2026, "5.10.4", Severity::Error),
    ],
};

/// IABSoundfieldLabelSubDescriptor: MCALabelDictionaryID is missing.
pub static MCALabelDictionaryIDMissing: Rule<E> = Rule {
    name: "MCALabelDictionaryIDMissing",
    category: Category::Audio,
    description: "IABSoundfieldLabelSubDescriptor: MCALabelDictionaryID is missing.",
    example: Some("IABSoundfieldLabelSubDescriptor with no MCALabelDictionaryID UL."),
    editions: &[
        row(E::V2019, "5.10.4", Severity::Error),
        row(E::V2021, "5.10.4", Severity::Error),
        row(E::V2026, "5.10.4", Severity::Error),
    ],
};

/// IABSoundfieldLabelSubDescriptor: MCALabelDictionaryID is not the required IAB label UL.
pub static MCALabelDictionaryIDInvalid: Rule<E> = Rule {
    name: "MCALabelDictionaryIDInvalid",
    category: Category::Audio,
    description: "IABSoundfieldLabelSubDescriptor: MCALabelDictionaryID is not the required IAB label UL.",
    example: Some("IABSoundfieldLabelSubDescriptor with MCALabelDictionaryID set to a 5.1 surround UL instead of the IAB label UL."),
    editions: &[
        row(E::V2019, "5.10.4", Severity::Error),
        row(E::V2021, "5.10.4", Severity::Error),
        row(E::V2026, "5.10.4", Severity::Error),
    ],
};

/// IABSequence shall contain at least one Resource (§6.2).
pub static IABSequenceNoResources: Rule<E> = Rule {
    name: "IABSequenceNoResources",
    category: Category::Audio,
    description: "IABSequence shall contain at least one Resource (§6.2).",
    example: Some("An IABSequence with an empty `<ResourceList>`."),
    editions: &[
        row(E::V2019, "6.2", Severity::Error),
        row(E::V2021, "6.2", Severity::Error),
        row(E::V2026, "6.2.1", Severity::Error),
    ],
};

/// IABSequence Resource.SourceEncoding does not reference an IABEssenceDescriptor (§6.2).
pub static IABSequenceSourceEncodingInvalid: Rule<E> = Rule {
    name: "IABSequenceSourceEncodingInvalid",
    category: Category::Audio,
    description: "IABSequence Resource.SourceEncoding does not reference an IABEssenceDescriptor (§6.2).",
    example: Some("An IABSequence Resource whose SourceEncoding references a WAVEPCMDescriptor instead of an IABEssenceDescriptor."),
    editions: &[
        row(E::V2019, "6.2", Severity::Error),
        row(E::V2021, "6.2", Severity::Error),
        row(E::V2026, "6.2.1", Severity::Error),
    ],
};

/// IAB Track File shall not contain AudioChannelLabel, SoundfieldGroupLabel or GroupOfSoundfieldGroupsLabel SubDescriptors (§5.10.2).
pub static ForbiddenMCASubDescriptor: Rule<E> = Rule {
    name: "ForbiddenMCASubDescriptor",
    category: Category::Audio,
    description: "IAB Track File shall not contain AudioChannelLabel, SoundfieldGroupLabel or GroupOfSoundfieldGroupsLabel SubDescriptors (§5.10.2).",
    example: Some("IAB Track File header metadata carrying an AudioChannelLabelSubDescriptor alongside the IABSoundfieldLabelSubDescriptor."),
    editions: &[
        row(E::V2019, "5.10.2", Severity::Error),
        row(E::V2021, "5.10.2", Severity::Error),
        row(E::V2026, "5.10.2", Severity::Error),
    ],
};

/// IAB Track File shall contain exactly one IABSoundfieldLabelSubDescriptor (§5.10.2).
pub static SubDescriptorDuplicate: Rule<E> = Rule {
    name: "SubDescriptorDuplicate",
    category: Category::Audio,
    description:
        "IAB Track File shall contain exactly one IABSoundfieldLabelSubDescriptor (§5.10.2).",
    example: Some(
        "IAB Track File header metadata with two IABSoundfieldLabelSubDescriptor instances.",
    ),
    editions: &[
        row(E::V2019, "5.10.2", Severity::Error),
        row(E::V2021, "5.10.2", Severity::Error),
        row(E::V2026, "5.10.2", Severity::Error),
    ],
};

/// MCAChannelID shall not be present in the IAB Soundfield Label SubDescriptor (Annex C.2).
pub static MCAChannelIDForbidden: Rule<E> = Rule {
    name: "MCAChannelIDForbidden",
    category: Category::Audio,
    description: "MCAChannelID shall not be present in the IAB Soundfield Label SubDescriptor (Annex C.2).",
    example: Some("IABSoundfieldLabelSubDescriptor carrying <MCAChannelID>1</MCAChannelID> — the item is excluded by Annex C Table C.1."),
    editions: &[
        row(E::V2019, "C.2", Severity::Error),
        row(E::V2021, "C.2", Severity::Error),
        row(E::V2026, "C.2", Severity::Error),
    ],
};

/// IAB Track File Edit Rate shall be an integer multiple of the Main Image Virtual Track Edit Rate (§6.2).
pub static EditRateNotIntegerMultiple: Rule<E> = Rule {
    name: "EditRateNotIntegerMultiple",
    category: Category::Audio,
    description: "IAB Track File Edit Rate shall be an integer multiple of the Main Image Virtual Track Edit Rate (§6.2).",
    example: Some("IABSequence Resource with EditRate 25/1 against a 24/1 Main Image Virtual Track (25 is not an integer multiple of 24)."),
    editions: &[
        row(E::V2019, "6.2", Severity::Error),
        row(E::V2021, "6.2", Severity::Error),
        row(E::V2026, "6.2.1", Severity::Error),
    ],
};

/// Track File should carry an IABChannelSubDescriptor for each channel of each BedDefinition (§5.10.2; set defined in Annex E).
pub static IabChannelSubDescriptorRecommended: Rule<E> = Rule {
    name: "IabChannelSubDescriptorRecommended",
    category: Category::Audio,
    description: "Track File should carry an IABChannelSubDescriptor for each channel of each BedDefinition (§5.10.2; set defined in Annex E).",
    example: Some("<IABEssenceDescriptor><SubDescriptors><IABSoundfieldLabelSubDescriptor/></SubDescriptors></IABEssenceDescriptor>  <!-- no <IABChannelSubDescriptor> entries -->"),
    editions: &[
        absent(E::V2019),
        absent(E::V2021),
        row(E::V2026, "5.10.2", Severity::Warning),
    ],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[
    &CodecForbidden,
    &ElectrospatialFormulationInvalid,
    &QuantizationBitsMissing,
    &QuantizationBitsInvalid,
    &ContainerFormatMissing,
    &EssenceContainerInvalid,
    &AudioSamplingRateMissing,
    &AudioSamplingRateInvalid,
    &SoundCompressionMissing,
    &SoundCompressionInvalid,
    &ChannelCountNotZero,
    &SubDescriptorMissing,
    &MCATagSymbolMissing,
    &MCATagSymbolInvalid,
    &MCATagNameMissing,
    &MCATagNameInvalid,
    &MCALabelDictionaryIDMissing,
    &MCALabelDictionaryIDInvalid,
    &IABSequenceNoResources,
    &IABSequenceSourceEncodingInvalid,
    &ForbiddenMCASubDescriptor,
    &SubDescriptorDuplicate,
    &MCAChannelIDForbidden,
    &EditRateNotIntegerMultiple,
    &IabChannelSubDescriptorRecommended,
];
