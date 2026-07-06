//! Typed validation-code catalogue for SMPTE ST 377-1 (MXF) plus the
//! ST 2067-2 / ST 377-4 essence-layer codes emitted by the modules in
//! `mxf::{essence, audio_mca, timed_text, metadata}`, and the
//! `IMFERNO:Mxf/*` engine-internal codes.
//!
//! Every code that ships in a `ValidationIssue` should resolve through
//! one of these enums via `ValidationCode::code()` so renames stay
//! compiler-checked.

use crate::diagnostics::codes::ValidationCode;
use crate::diagnostics::{Category, Severity};

// ─── ST 377-1:2011 — MXF File Format ─────────────────────────────────────────

/// Validation codes defined by SMPTE ST 377-1:2011 (MXF File Format).
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum St377_1_2011 {
    /// File is not a valid MXF container.
    NotMxf,
    /// MXF file could not be parsed; may be truncated or corrupt.
    ParseError,
    /// MXF file contains no essence containers.
    NoEssenceContainers,
    /// Operational pattern is not OP1a as required by IMF.
    Op1a,
    /// First partition in the file is not the Header partition
    /// (ST 377-1 §6.4 requires header-first ordering).
    NonHeaderFirstPartition,
    /// Header partition is Open (in-progress) — finished IMF
    /// deliveries should be ClosedComplete per ST 377-1 §8.3.3.
    HeaderPartitionOpen,
    /// Header partition declares zero header metadata
    /// (HeaderByteCount = 0) — ST 377-1 §8.3.3 requires header
    /// metadata in the header partition.
    MissingHeaderMetadata,
}

impl ValidationCode for St377_1_2011 {
    fn code(&self) -> &'static str {
        match self {
            Self::NotMxf => "ST377-1:2011:5/NotMxf",
            Self::ParseError => "ST377-1:2011:5/ParseError",
            Self::NoEssenceContainers => "ST377-1:2011:11/NoEssenceContainers",
            Self::Op1a => "ST377-1:2011:7/OP1a",
            Self::NonHeaderFirstPartition => "ST377-1:2011:6.4/NonHeaderFirstPartition",
            Self::HeaderPartitionOpen => "ST377-1:2011:8.3.3/HeaderPartitionOpen",
            Self::MissingHeaderMetadata => "ST377-1:2011:8.3.3/MissingHeaderMetadata",
        }
    }
    fn description(&self) -> &'static str {
        match self {
            Self::NotMxf => "File is not a valid MXF container.",
            Self::ParseError => "MXF file could not be parsed; it may be truncated or corrupt.",
            Self::NoEssenceContainers => "MXF file contains no essence containers.",
            Self::Op1a => "MXF operational pattern must be OP1a for IMF packages.",
            Self::NonHeaderFirstPartition => {
                "The first partition in an MXF file must be the Header partition."
            }
            Self::HeaderPartitionOpen => {
                "The header partition status should be ClosedComplete for finished deliveries."
            }
            Self::MissingHeaderMetadata => {
                "The header partition must carry header metadata (HeaderByteCount > 0)."
            }
        }
    }
    fn default_severity(&self) -> Severity {
        match self {
            Self::NotMxf | Self::ParseError | Self::NoEssenceContainers => Severity::Warning,
            Self::HeaderPartitionOpen => Severity::Warning,
            Self::Op1a | Self::NonHeaderFirstPartition | Self::MissingHeaderMetadata => {
                Severity::Error
            }
        }
    }
    fn category(&self) -> Category {
        match self {
            Self::NotMxf | Self::ParseError => Category::Asset,
            Self::NoEssenceContainers | Self::Op1a => Category::Encoding,
            Self::NonHeaderFirstPartition
            | Self::HeaderPartitionOpen
            | Self::MissingHeaderMetadata => Category::Container,
        }
    }
    fn example(&self) -> Option<&'static str> {
        Some(match self {
            Self::NotMxf =>
                "An asset declared in the PKL with MIME `application/mxf` whose header doesn't carry the MXF run-in / partition pack key.",
            Self::ParseError =>
                "An MXF file that was truncated mid-transfer; the footer partition is missing or the RIP is unreadable.",
            Self::NoEssenceContainers =>
                "An MXF file whose header metadata declares no EssenceContainer ULs — the body has no decodable essence.",
            Self::Op1a =>
                "An MXF file whose Preface declares OperationalPattern OP-Atom or OP-3a instead of OP-1a.",
            Self::NonHeaderFirstPartition =>
                "An authoring tool wrote a Body partition before the Header partition, violating ST 377-1 ordering.",
            Self::HeaderPartitionOpen =>
                "A live-streaming export shipped a Header partition with OpenIncomplete status instead of finalising to ClosedComplete.",
            Self::MissingHeaderMetadata =>
                "An MXF file whose Header partition advertises HeaderByteCount = 0 — no metadata sets follow the partition pack.",
        })
    }
}

impl St377_1_2011 {
    pub const ALL: &'static [Self] = &[
        Self::NotMxf,
        Self::ParseError,
        Self::NoEssenceContainers,
        Self::Op1a,
        Self::NonHeaderFirstPartition,
        Self::HeaderPartitionOpen,
        Self::MissingHeaderMetadata,
    ];
}

impl From<St377_1_2011> for String {
    fn from(c: St377_1_2011) -> String {
        c.code().to_string()
    }
}

// ─── ST 2067-2:2016 — Essence-layer (§5.2 / §5.3 / §5.4) ─────────────────────

/// SMPTE ST 2067-2:2016 essence-layer codes (§5.2 OP1a, §5.3 audio
/// MCA, §5.4 timed text). Cited from the 2016 edition because that's
/// where the MCA work was stabilised; later editions inherit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum St2067_2_2016 {
    /// §5.2 — operational pattern UL is not OP1a (ST 378:2004).
    OperationalPatternNotOP1A,
    /// §5.3.4.1 — audio essence descriptor is not WAVEPCMDescriptor.
    SoundDescriptorNotWAVEPCM,
    /// §5.3.2.2 — AudioSampleRate must be 48 000 or 96 000 Hz.
    AudioSampleRateUnsupported,
    /// §5.3.2.3 — QuantizationBits must be 24.
    QuantizationBitsNot24,
    /// §5.3.6.2 — number of AudioChannelLabelSubDescriptors must
    /// equal ChannelCount.
    ChannelLabelCountMismatch,
    /// §5.3.6.5 Table 7 — MCAChannelID shall be present on each
    /// AudioChannelLabelSubDescriptor, except that channel 1 may omit
    /// it.
    MCAChannelIDMissing,
    /// §5.3.6.3 — exactly one SoundfieldGroupLabelSubDescriptor per
    /// WAVEPCMDescriptor.
    SoundFieldGroupLabelCount,
    /// §5.3.6.5 Table 7 — SoundfieldGroupLinkID shall be present on
    /// every AudioChannelLabelSubDescriptor.
    SoundfieldGroupLinkIDMissing,
    /// §5.3.3 / ST 382 §10 — audio essence must be Wave Clip-Wrapped.
    AudioNotClipWrapped,
    /// §5.3 — RFC-5646 spoken language tag recommended (Warning).
    RFC5646SpokenLanguageMissing,
    /// §5.3.4.2 — ChannelAssignment shall be present.
    ChannelAssignmentMissing,
    /// §5.3.4.2 — ChannelAssignment shall equal the Table 5 label UL
    /// (IMF MCA channel assignment, byte 13 = 04h).
    ChannelAssignmentNotMCA,
    /// §5.3.6.5 Table 7 — SoundfieldGroup MCATitle shall be present.
    SoundfieldGroupMissingMCATitle,
    /// §5.3.6.5 Table 7 — SoundfieldGroup MCATitleVersion shall be present.
    SoundfieldGroupMissingMCATitleVersion,
    /// §5.3.6.5 Table 7 — SoundfieldGroup MCAAudioContentKind shall be present.
    SoundfieldGroupMissingMCAAudioContentKind,
    /// §5.3.6.5 Table 7 — SoundfieldGroup MCAAudioElementKind shall be present.
    SoundfieldGroupMissingMCAAudioElementKind,
    /// §5.4 — timed text UCSEncoding must be UTF-8.
    TimedTextUCSEncodingNotUTF8,
    /// §5.4 — timed text NamespaceURI must be an IMSC1 profile.
    TimedTextNamespaceNotIMSC,
    /// §5.4.5/6 — TimeTextResourceSubDescriptor.MIMEType must be
    /// image/png, font/otf (2020 §5.4.6) or application/x-font-opentype.
    TimedTextResourceMIMETypeUnsupported,
    /// §5.4 / ST 429-5 §7 — timed-text ContainerFormat UL byte 14
    /// (Mapping Kind) must be 0x13 for IMSC.
    TimedTextMappingKindNot0x13,
}

impl ValidationCode for St2067_2_2016 {
    fn code(&self) -> &'static str {
        match self {
            Self::OperationalPatternNotOP1A => "ST2067-2:2016:5.2/OperationalPatternNotOP1A",
            Self::SoundDescriptorNotWAVEPCM => "ST2067-2:2016:5.3.4.1/SoundDescriptorNotWAVEPCM",
            Self::AudioSampleRateUnsupported => "ST2067-2:2016:5.3.2.2/AudioSampleRateUnsupported",
            Self::QuantizationBitsNot24 => "ST2067-2:2016:5.3.2.3/QuantizationBitsNot24",
            Self::ChannelLabelCountMismatch => "ST2067-2:2016:5.3.6.2/ChannelLabelCountMismatch",
            Self::MCAChannelIDMissing => "ST2067-2:2016:5.3.6.5/MCAChannelIDMissing",
            Self::SoundFieldGroupLabelCount => "ST2067-2:2016:5.3.6.3/SoundFieldGroupLabelCount",
            Self::SoundfieldGroupLinkIDMissing => {
                "ST2067-2:2016:5.3.6.5/SoundfieldGroupLinkIDMissing"
            }
            Self::AudioNotClipWrapped => "ST2067-2:2016:5.3.3/AudioNotClipWrapped",
            Self::RFC5646SpokenLanguageMissing => "ST2067-2:2016:5.3/RFC5646SpokenLanguageMissing",
            Self::ChannelAssignmentMissing => "ST2067-2:2016:5.3.4.2/ChannelAssignmentMissing",
            Self::ChannelAssignmentNotMCA => "ST2067-2:2016:5.3.4.2/ChannelAssignmentNotMCA",
            Self::SoundfieldGroupMissingMCATitle => {
                "ST2067-2:2016:5.3.6.5/SoundfieldGroupMissing/MCATitle"
            }
            Self::SoundfieldGroupMissingMCATitleVersion => {
                "ST2067-2:2016:5.3.6.5/SoundfieldGroupMissing/MCATitleVersion"
            }
            Self::SoundfieldGroupMissingMCAAudioContentKind => {
                "ST2067-2:2016:5.3.6.5/SoundfieldGroupMissing/MCAAudioContentKind"
            }
            Self::SoundfieldGroupMissingMCAAudioElementKind => {
                "ST2067-2:2016:5.3.6.5/SoundfieldGroupMissing/MCAAudioElementKind"
            }
            Self::TimedTextUCSEncodingNotUTF8 => "ST2067-2:2016:5.4/TimedTextUCSEncodingNotUTF8",
            Self::TimedTextNamespaceNotIMSC => "ST2067-2:2016:5.4/TimedTextNamespaceNotIMSC",
            Self::TimedTextResourceMIMETypeUnsupported => {
                "ST2067-2:2016:5.4.5/TimedTextResourceMIMETypeUnsupported"
            }
            Self::TimedTextMappingKindNot0x13 => "ST2067-2:2016:5.4/TimedTextMappingKindNot0x13",
        }
    }
    fn description(&self) -> &'static str {
        match self {
            Self::OperationalPatternNotOP1A =>
                "MXF essence Operational Pattern UL is not OP1a (ST 378:2004).",
            Self::SoundDescriptorNotWAVEPCM =>
                "Audio essence descriptor must be WAVEPCMDescriptor.",
            Self::AudioSampleRateUnsupported =>
                "AudioSampleRate must be 48000 Hz or 96000 Hz.",
            Self::QuantizationBitsNot24 => "QuantizationBits must be 24.",
            Self::ChannelLabelCountMismatch =>
                "Number of AudioChannelLabelSubDescriptors must equal ChannelCount.",
            Self::MCAChannelIDMissing =>
                "MCAChannelID shall be present per channel (channel 1 may omit it) — §5.3.6.5 Table 7.",
            Self::SoundFieldGroupLabelCount =>
                "Exactly one SoundfieldGroupLabelSubDescriptor is required per WAVEPCMDescriptor.",
            Self::SoundfieldGroupLinkIDMissing =>
                "Every AudioChannelLabelSubDescriptor shall carry a SoundfieldGroupLinkID (§5.3.6.5 Table 7).",
            Self::AudioNotClipWrapped =>
                "Audio essence must be Wave Clip-Wrapped (ST 382 §10).",
            Self::RFC5646SpokenLanguageMissing =>
                "Audio descriptor should carry an RFC-5646 spoken language tag.",
            Self::ChannelAssignmentMissing =>
                "ChannelAssignment shall be present on the audio essence descriptor (§5.3.4.2).",
            Self::ChannelAssignmentNotMCA =>
                "ChannelAssignment shall equal the ST 2067-2 Table 5 channel-assignment label UL.",
            Self::SoundfieldGroupMissingMCATitle =>
                "SoundfieldGroupLabelSubDescriptor shall carry MCATitle (§5.3.6.5 Table 7).",
            Self::SoundfieldGroupMissingMCATitleVersion =>
                "SoundfieldGroupLabelSubDescriptor shall carry MCATitleVersion (§5.3.6.5 Table 7).",
            Self::SoundfieldGroupMissingMCAAudioContentKind =>
                "SoundfieldGroupLabelSubDescriptor shall carry MCAAudioContentKind (§5.3.6.5 Table 7).",
            Self::SoundfieldGroupMissingMCAAudioElementKind =>
                "SoundfieldGroupLabelSubDescriptor shall carry MCAAudioElementKind (§5.3.6.5 Table 7).",
            Self::TimedTextUCSEncodingNotUTF8 =>
                "TimedTextDescriptor UCSEncoding must be UTF-8.",
            Self::TimedTextNamespaceNotIMSC =>
                "TimedTextDescriptor NamespaceURI must be one of the IMSC1 profile namespaces.",
            Self::TimedTextResourceMIMETypeUnsupported =>
                "TimeTextResourceSubDescriptor MIMEType must be image/png, font/otf (2020) or application/x-font-opentype.",
            Self::TimedTextMappingKindNot0x13 =>
                "Timed-text ContainerFormat UL Mapping Kind byte must be 0x13 (IMSC).",
        }
    }
    fn default_severity(&self) -> Severity {
        match self {
            // Recommendation-level rule emitted as Warning so operators
            // can `Off` it when their pipeline doesn't require the field.
            // (The SoundfieldGroupMissingMCA* rules were moved to Error:
            // Table 7 marks them SHALL for the SoundfieldGroup column —
            // AUDIT-11.)
            Self::RFC5646SpokenLanguageMissing => Severity::Warning,
            // Everything else is a "SHALL" rule — Error severity.
            _ => Severity::Error,
        }
    }
    fn category(&self) -> Category {
        match self {
            // Audio essence layer.
            Self::SoundDescriptorNotWAVEPCM
            | Self::AudioSampleRateUnsupported
            | Self::QuantizationBitsNot24
            | Self::ChannelLabelCountMismatch
            | Self::MCAChannelIDMissing
            | Self::SoundFieldGroupLabelCount
            | Self::SoundfieldGroupLinkIDMissing
            | Self::RFC5646SpokenLanguageMissing
            | Self::ChannelAssignmentMissing
            | Self::ChannelAssignmentNotMCA
            | Self::SoundfieldGroupMissingMCATitle
            | Self::SoundfieldGroupMissingMCATitleVersion
            | Self::SoundfieldGroupMissingMCAAudioContentKind
            | Self::SoundfieldGroupMissingMCAAudioElementKind => Category::Audio,
            // Timed-text essence layer.
            Self::TimedTextUCSEncodingNotUTF8
            | Self::TimedTextNamespaceNotIMSC
            | Self::TimedTextResourceMIMETypeUnsupported => Category::Subtitle,
            // Container-layer rules (operational pattern, wrapping,
            // mapping kind — properties of the MXF wrapper, not the
            // essence inside).
            Self::OperationalPatternNotOP1A
            | Self::AudioNotClipWrapped
            | Self::TimedTextMappingKindNot0x13 => Category::Container,
        }
    }
    fn example(&self) -> Option<&'static str> {
        Some(match self {
            Self::OperationalPatternNotOP1A =>
                "Header partition pack OP UL bytes 13..14 = 02 03 (OP2a) instead of OP1a (01 01)",
            Self::SoundDescriptorNotWAVEPCM =>
                "Audio essence descriptor is AES3PCMDescriptor instead of WAVEPCMDescriptor",
            Self::AudioSampleRateUnsupported =>
                "<AudioSampleRate>44100/1</AudioSampleRate>  <!-- only 48000 or 96000 allowed -->",
            Self::QuantizationBitsNot24 =>
                "<QuantizationBits>16</QuantizationBits>",
            Self::ChannelLabelCountMismatch =>
                "<ChannelCount>6</ChannelCount> but only 2 AudioChannelLabelSubDescriptor entries present",
            Self::MCAChannelIDMissing =>
                "ChannelCount=6 but no AudioChannelLabelSubDescriptor carries MCAChannelID=4 (channels 2..N must be identified)",
            Self::SoundFieldGroupLabelCount =>
                "WAVEPCMDescriptor with 2 SoundfieldGroupLabelSubDescriptor children (must be exactly 1)",
            Self::SoundfieldGroupLinkIDMissing =>
                "<AudioChannelLabelSubDescriptor>…</AudioChannelLabelSubDescriptor>  <!-- no <SoundfieldGroupLinkID> child -->",
            Self::AudioNotClipWrapped =>
                "ContainerFormat UL byte 14 = 0x02 (frame-wrapped) instead of 0x06 (clip-wrapped)",
            Self::RFC5646SpokenLanguageMissing =>
                "No RFC5646SpokenLanguage tag on any AudioChannelLabel/SoundfieldGroupLabel sub-descriptor",
            Self::ChannelAssignmentMissing =>
                "WAVEPCMDescriptor with no <ChannelAssignment> item",
            Self::ChannelAssignmentNotMCA =>
                "ChannelAssignment UL with byte 13 = 05h (ADM labeling) instead of the Table 5 value 04h on a plain IMF audio file",
            Self::SoundfieldGroupMissingMCATitle =>
                "SoundfieldGroupLabelSubDescriptor with no <MCATitle> item",
            Self::SoundfieldGroupMissingMCATitleVersion =>
                "SoundfieldGroupLabelSubDescriptor with no <MCATitleVersion> item",
            Self::SoundfieldGroupMissingMCAAudioContentKind =>
                "SoundfieldGroupLabelSubDescriptor with no <MCAAudioContentKind> item",
            Self::SoundfieldGroupMissingMCAAudioElementKind =>
                "SoundfieldGroupLabelSubDescriptor with no <MCAAudioElementKind> item",
            Self::TimedTextUCSEncodingNotUTF8 =>
                "<UCSEncoding>ISO-8859-1</UCSEncoding>",
            Self::TimedTextNamespaceNotIMSC =>
                "<NamespaceURI>http://www.w3.org/ns/ttml</NamespaceURI>  <!-- not an IMSC1 profile URI -->",
            Self::TimedTextResourceMIMETypeUnsupported =>
                "<TimeTextResourceSubDescriptor><MIMEType>application/json</MIMEType></TimeTextResourceSubDescriptor>",
            Self::TimedTextMappingKindNot0x13 =>
                "Timed-text ContainerFormat UL byte 14 (Mapping Kind) = 0x12 instead of 0x13 (IMSC)",
        })
    }
}

impl St2067_2_2016 {
    pub const ALL: &'static [Self] = &[
        Self::OperationalPatternNotOP1A,
        Self::SoundDescriptorNotWAVEPCM,
        Self::AudioSampleRateUnsupported,
        Self::QuantizationBitsNot24,
        Self::ChannelLabelCountMismatch,
        Self::MCAChannelIDMissing,
        Self::SoundFieldGroupLabelCount,
        Self::SoundfieldGroupLinkIDMissing,
        Self::AudioNotClipWrapped,
        Self::RFC5646SpokenLanguageMissing,
        Self::ChannelAssignmentMissing,
        Self::ChannelAssignmentNotMCA,
        Self::SoundfieldGroupMissingMCATitle,
        Self::SoundfieldGroupMissingMCATitleVersion,
        Self::SoundfieldGroupMissingMCAAudioContentKind,
        Self::SoundfieldGroupMissingMCAAudioElementKind,
        Self::TimedTextUCSEncodingNotUTF8,
        Self::TimedTextNamespaceNotIMSC,
        Self::TimedTextResourceMIMETypeUnsupported,
        Self::TimedTextMappingKindNot0x13,
    ];
}

impl From<St2067_2_2016> for String {
    fn from(c: St2067_2_2016) -> String {
        c.code().to_string()
    }
}

// ─── ST 377-4:2012 — MCA sub-descriptor linkage ──────────────────────────────

/// Validation codes for SMPTE ST 377-4:2012 MCA sub-descriptor linkage
/// rules — the MCALinkID (§6.3 Table 3, Req) / SoundfieldGroupLinkID
/// (§6.4.1) correlation that ties channel labels to their soundfield
/// group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum St377_4_2012 {
    /// §6.3 Table 3 — MCALinkID is a required item on every MCA
    /// sub-descriptor.
    MCALinkIDMissing,
    /// §6.4.1 — AudioChannelLabelSubDescriptor's SoundfieldGroupLinkID
    /// shall be the MCA Link ID of the SoundfieldGroupLabelSubDescriptor
    /// to which the channel belongs.
    SoundfieldGroupLinkIDMismatch,
}

impl ValidationCode for St377_4_2012 {
    fn code(&self) -> &'static str {
        match self {
            Self::MCALinkIDMissing => "ST377-4:2012:6.3/MCALinkIDMissing",
            Self::SoundfieldGroupLinkIDMismatch => {
                "ST377-4:2012:6.4.1/SoundfieldGroupLinkIDMismatch"
            }
        }
    }
    fn description(&self) -> &'static str {
        match self {
            Self::MCALinkIDMissing => "Every MCA sub-descriptor shall carry an MCALinkID (§6.3 Table 3, Req).",
            Self::SoundfieldGroupLinkIDMismatch =>
                "AudioChannelLabelSubDescriptor SoundfieldGroupLinkID shall equal the SoundfieldGroup's MCALinkID (§6.4.1).",
        }
    }
    fn default_severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> Category {
        Category::Audio
    }
    fn example(&self) -> Option<&'static str> {
        Some(match self {
            Self::MCALinkIDMissing =>
                "<AudioChannelLabelSubDescriptor>…</AudioChannelLabelSubDescriptor>  <!-- no <MCALinkID> child -->",
            Self::SoundfieldGroupLinkIDMismatch =>
                "<AudioChannelLabelSubDescriptor><SoundfieldGroupLinkID>urn:uuid:abc…</SoundfieldGroupLinkID></AudioChannelLabelSubDescriptor>  <!-- but the parent SoundfieldGroupLabelSubDescriptor MCALinkID = urn:uuid:def… -->",
        })
    }
}

impl St377_4_2012 {
    pub const ALL: &'static [Self] = &[Self::MCALinkIDMissing, Self::SoundfieldGroupLinkIDMismatch];
}

impl From<St377_4_2012> for String {
    fn from(c: St377_4_2012) -> String {
        c.code().to_string()
    }
}

// ─── IMFERNO engine-internal MXF codes ───────────────────────────────────────

/// Engine-internal codes the MXF pipeline emits when it can't reach
/// the spec layer — open failures, parse failures, conversion errors,
/// and the informational "essence containers detected" trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum ImfernoMxf {
    /// Could not open the MXF file for reading.
    OpenFailed,
    /// Failed to parse the MXF header partition pack.
    PartitionPackParseFailed,
    /// Failed to convert the MXF header metadata to RegXML for
    /// downstream essence-rule application.
    RegXmlConversionFailed,
    /// Informational: the MXF declares N essence containers in its
    /// header partition. Emitted on every clean MXF for report context.
    EssenceContainersDetected,
}

impl ValidationCode for ImfernoMxf {
    fn code(&self) -> &'static str {
        match self {
            Self::OpenFailed => "IMFERNO:Mxf/OpenFailed",
            Self::PartitionPackParseFailed => "IMFERNO:Mxf/PartitionPackParseFailed",
            Self::RegXmlConversionFailed => "IMFERNO:Mxf/RegXmlConversionFailed",
            Self::EssenceContainersDetected => "IMFERNO:Mxf/EssenceContainersDetected",
        }
    }
    fn description(&self) -> &'static str {
        match self {
            Self::OpenFailed => "MXF file could not be opened for reading.",
            Self::PartitionPackParseFailed => "MXF header partition pack failed to parse.",
            Self::RegXmlConversionFailed => "MXF header metadata could not be converted to RegXML.",
            Self::EssenceContainersDetected => {
                "Informational trace of how many essence containers the MXF declares."
            }
        }
    }
    fn default_severity(&self) -> Severity {
        match self {
            Self::OpenFailed | Self::PartitionPackParseFailed => Severity::Critical,
            Self::RegXmlConversionFailed => Severity::Warning,
            Self::EssenceContainersDetected => Severity::Info,
        }
    }
    fn category(&self) -> Category {
        Category::Container
    }
    fn example(&self) -> Option<&'static str> {
        Some(match self {
            Self::OpenFailed =>
                "fs::File::open(\"audio1.mxf\") returned Err (file missing or unreadable)",
            Self::PartitionPackParseFailed =>
                "Header partition pack KLV is malformed (e.g. BER length undershoots the minimum 88-byte body)",
            Self::RegXmlConversionFailed =>
                "smpte-mxf could not convert header metadata to RegXML (e.g. unknown KLV set in header)",
            Self::EssenceContainersDetected =>
                "Header partition pack declares N essence container ULs — info notice surfaces the count for traceability",
        })
    }
}

impl ImfernoMxf {
    pub const ALL: &'static [Self] = &[
        Self::OpenFailed,
        Self::PartitionPackParseFailed,
        Self::RegXmlConversionFailed,
        Self::EssenceContainersDetected,
    ];
}

impl From<ImfernoMxf> for String {
    fn from(c: ImfernoMxf) -> String {
        c.code().to_string()
    }
}
