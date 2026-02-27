//! Typed validation-code catalogue for SMPTE ST 2067-21 (Application Profile #2E).

use crate::diagnostics::codes::ValidationCode;
use crate::diagnostics::{Category, Severity};

macro_rules! impl_into_string {
    ($t:ty) => {
        impl From<$t> for String {
            fn from(c: $t) -> String {
                <$t as ValidationCode>::code(&c).to_string()
            }
        }
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// ST 2067-21:2020
// ─────────────────────────────────────────────────────────────────────────────

/// Validation codes defined by SMPTE ST 2067-21:2020 (Application Profile #2E).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum St2067_21_2020 {
    /// Application identifier in CPL ExtensionProperties does not match the expected App2E URI.
    AppIdMismatch,
}

impl ValidationCode for St2067_21_2020 {
    fn code(&self) -> &'static str {
        match self {
            Self::AppIdMismatch => "ST2067-21:2020:7.1/AppIdMismatch",
        }
    }
    fn description(&self) -> &'static str {
        match self {
            Self::AppIdMismatch =>
                "Application identifier in CPL ExtensionProperties does not match the expected App2E URI.",
        }
    }
    fn default_severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Metadata
    }
}

impl St2067_21_2020 {
    pub const ALL: &'static [Self] = &[Self::AppIdMismatch];
}

impl_into_string!(St2067_21_2020);

// ─────────────────────────────────────────────────────────────────────────────
// ST 2067-21:2023
// ─────────────────────────────────────────────────────────────────────────────

/// Validation codes defined by SMPTE ST 2067-21:2023 (Application Profile #2E, UHD/HDR).
///
/// Variant naming notes:
/// - Variants ending in `Unknown` check for unrecognized UL values (wrong value).
/// - Variants starting with `Required` check for missing mandatory fields.
/// - Plain variants (e.g. `ColorPrimaries`) check field presence at §6.2.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum St2067_21_2023 {
    // ── §5.2 Frame rates and resolutions ─────────────────────────────────────
    /// Frame rate is not in the permitted set for App2E.
    FrameRate,
    /// Image resolution is not in the permitted set for App2E.
    Resolution,

    // ── §5.3 Language tags ────────────────────────────────────────────────────
    /// Locale language tag is empty.
    EmptyLanguageTag,
    /// Locale language tag is not a valid BCP-47 subtag.
    MalformedLanguageTag,
    /// Region subtag in a language tag is not valid.
    RegionCode,

    // ── §6.2 Color system ────────────────────────────────────────────────────
    /// Color system designator is not in the permitted set.
    ColorSystem,

    // ── §6.2 Required picture-descriptor fields ───────────────────────────────
    /// RGBA/CDCI descriptor is missing the required `StoredWidth` field.
    RequiredStoredWidth,
    /// RGBA/CDCI descriptor is missing the required `StoredHeight` field.
    RequiredStoredHeight,
    /// RGBA/CDCI descriptor is missing the required `SampleRate` field.
    RequiredSampleRate,
    /// RGBA/CDCI descriptor is missing the required `FrameLayout` field.
    RequiredFrameLayout,
    /// RGBA/CDCI descriptor is missing the required `ColorPrimaries` field.
    RequiredColorPrimaries,
    /// RGBA/CDCI descriptor is missing the required `TransferCharacteristic` field.
    RequiredTransferCharacteristic,
    /// RGBA/CDCI descriptor is missing the required `PictureCompression` field.
    RequiredPictureCompression,
    /// CDCI descriptor is missing the required `ComponentDepth` field.
    RequiredComponentDepth,

    // ── §6.5 Required audio-descriptor fields ─────────────────────────────────
    /// WavePCM descriptor is missing the required `ChannelCount` field.
    RequiredChannelCount,
    /// WavePCM descriptor is missing the required `QuantizationBits` field.
    RequiredQuantizationBits,

    // ── §6.2.1 Picture descriptor constraints ────────────────────────────────
    /// Alpha transparency mode is not permitted in App2E.
    AlphaTransparency,
    /// `CodingEquations` field is absent from the picture descriptor (§6.2.1 Table 8).
    CodingEquations,
    /// `ColorPrimaries` field is absent from the picture descriptor (§6.2.1 Table 8).
    ColorPrimaries,
    /// `FieldDominance` value is not permitted for the declared `FrameLayout`.
    FieldDominance,
    /// `FrameLayout` value is not in the permitted set for App2E.
    FrameLayout,
    /// `FrameLayout` declares interlaced content, which is not permitted in App2E.
    FrameLayoutInterlaced,
    /// `ImageAlignmentOffset` value is not zero as required.
    ImageAlignmentOffset,
    /// `ImageEndOffset` value is not zero as required.
    ImageEndOffset,
    /// `ImageStartOffset` value is not zero as required.
    ImageStartOffset,
    /// `SampledHeight` does not equal `StoredHeight` as required.
    SampledHeight,
    /// `SampledWidth` does not equal `StoredWidth` as required.
    SampledWidth,
    /// `SampledXOffset` is not zero as required.
    SampledXOffset,
    /// `SampledYOffset` is not zero as required.
    SampledYOffset,
    /// `StoredF2Offset` is not zero as required for interlaced content.
    StoredF2Offset,
    /// `TransferCharacteristic` field is absent from the picture descriptor (§6.2.1 Table 8).
    TransferCharacteristic,

    // ── §6.2.2 Transfer characteristic value ────────────────────────────────
    /// `TransferCharacteristic` UL is present but not a recognized value (§6.2.2).
    TransferCharacteristicUnknown,

    // ── §6.2.3 Coding equations value ────────────────────────────────────────
    /// `CodingEquations` UL is present but not a recognized value (§6.2.3).
    CodingEquationsUnknown,

    // ── §6.2.4 Color primaries value ─────────────────────────────────────────
    /// `ColorPrimaries` UL is present but not a recognized value (§6.2.4).
    ColorPrimariesUnknown,

    // ── §6.2.5 JPEG 2000 requirement ─────────────────────────────────────────
    /// Video essence is not JPEG 2000 encoded as required by App2E.
    J2KRequired,

    // ── §6.3 RGBA descriptor ─────────────────────────────────────────────────
    /// `AlphaMaxRef` value is not permitted.
    AlphaMaxRef,
    /// `AlphaMinRef` value is not permitted.
    AlphaMinRef,
    /// `ComponentMaxRef` value is not in the permitted range.
    ComponentMaxRef,
    /// `ComponentMinRef` value is not in the permitted range.
    ComponentMinRef,
    /// `Palette` is present; palette images are not permitted in App2E.
    Palette,
    /// `PaletteLayout` is present; palette layout is not permitted in App2E.
    PaletteLayout,
    /// `ScanningDirection` value is not in the permitted set.
    ScanningDirection,

    // ── §6.3.2 Component reference values ────────────────────────────────────
    /// Component max/min reference values are inconsistent with bit depth.
    ComponentRefValues,

    // ── §6.4 Bit depth and chroma ────────────────────────────────────────────
    /// `AlphaSampleDepth` value is not permitted.
    AlphaSampleDepth,
    /// `ColorSiting` value is not in the permitted set.
    ColorSiting,
    /// `ComponentDepth` value is not in the permitted set (8/10/12/16).
    ComponentDepth,
    /// `HorizontalSubsampling` value is not in the permitted set.
    HorizontalSubsampling,
    /// `PaddingBits` value is not zero as required.
    PaddingBits,
    /// `ReversedByteOrder` flag is set; byte reversal is not permitted.
    ReversedByteOrder,
    /// `VerticalSubsampling` value is not in the permitted set.
    VerticalSubsampling,

    // ── §6.4.3 Luma range ────────────────────────────────────────────────────
    /// `BlackRefLevel` value is inconsistent with bit depth.
    BlackRefLevel,
    /// `ColorRange` value is not in the permitted set.
    ColorRange,
    /// `WhiteRefLevel` value is inconsistent with bit depth.
    WhiteRefLevel,

    // ── §6.5 Audio ───────────────────────────────────────────────────────────
    /// Audio sample rate is not 48 kHz as required.
    AudioSampleRate,
    /// `QuantizationBits` value is not in the permitted set (16/24).
    QuantizationBits,

    // ── §6.5.2 JPEG 2000 sub-descriptor ─────────────────────────────────────
    /// J2K codestream coding style is not compliant.
    CodingStyle,
    /// JPEG 2000 codestream layout (J2C) does not meet App2E requirements.
    J2CLayout,
    /// JPEG 2000 extended capabilities are declared but not permitted.
    J2KExtendedCapabilities,
    /// JPEG2000SubDescriptor is absent or incomplete.
    Jpeg2000SubDescriptor,

    // ── §6.2.5 JPEG 2000 resolution profiles ────────────────────────────────
    /// JPEG 2000 HT (ISO 15444-15) is not permitted by the App2E 2020 edition.
    J2KHtNotAllowed,
    /// JPEG 2000 IMF 4K Profile: stored resolution is outside the permitted range.
    J2K4KResolution,
    /// JPEG 2000 IMF 2K Profile: stored resolution is outside the permitted range.
    J2K2KResolution,
    /// JPEG 2000 Broadcast Contribution Profile: stored resolution is outside the permitted range.
    J2KBcpResolution,

    // ── §7.1 Application identification ──────────────────────────────────────
    /// ApplicationIdentification is required for App2E compositions.
    ApplicationIdentification,
    /// ContentMaturityRating agency is empty.
    ContentMaturityRatingAgency,
    /// ContentMaturityRating agency is not a valid xs:anyURI.
    ContentMaturityRatingAgencyUri,

    // ── §7.2 Homogeneous image essence ───────────────────────────────────────
    /// All image essence in a composition shall use the same color system.
    HomogeneousImageEssence,

    // ── §7.1 Application identification ──────────────────────────────────────
    /// Application identifier in CPL ExtensionProperties does not match the App2E URI.
    AppIdMismatch,

    // ── §7.4 Segment duration ─────────────────────────────────────────────────
    /// Segment duration is not an integer multiple of 5 edit units as required by App2E.
    SegmentDurationMultiple,

    // ── §7.5 HDR metadata ────────────────────────────────────────────────────
    /// `MaxCLL` / `MaxFALL` HDR metadata is absent; recommended for HDR content.
    MaxCLLMaxFALL,
}

impl ValidationCode for St2067_21_2023 {
    fn code(&self) -> &'static str {
        match self {
            Self::FrameRate => "ST2067-21:2023:5.2/FrameRate",
            Self::Resolution => "ST2067-21:2023:5.2/Resolution",
            Self::EmptyLanguageTag => "ST2067-21:2023:5.3/EmptyLanguageTag",
            Self::MalformedLanguageTag => "ST2067-21:2023:5.3/MalformedLanguageTag",
            Self::RegionCode => "ST2067-21:2023:5.3/RegionCode",
            Self::ColorSystem => "ST2067-21:2023:6.2/ColorSystem",
            Self::RequiredStoredWidth => "ST2067-21:2023:6.2/Required-StoredWidth",
            Self::RequiredStoredHeight => "ST2067-21:2023:6.2/Required-StoredHeight",
            Self::RequiredSampleRate => "ST2067-21:2023:6.2/Required-SampleRate",
            Self::RequiredFrameLayout => "ST2067-21:2023:6.2/Required-FrameLayout",
            Self::RequiredColorPrimaries => "ST2067-21:2023:6.2/Required-ColorPrimaries",
            Self::RequiredTransferCharacteristic => {
                "ST2067-21:2023:6.2/Required-TransferCharacteristic"
            }
            Self::RequiredPictureCompression => "ST2067-21:2023:6.2/Required-PictureCompression",
            Self::RequiredComponentDepth => "ST2067-21:2023:6.2/Required-ComponentDepth",
            Self::RequiredChannelCount => "ST2067-21:2023:6.5/Required-ChannelCount",
            Self::RequiredQuantizationBits => "ST2067-21:2023:6.5/Required-QuantizationBits",
            Self::AlphaTransparency => "ST2067-21:2023:6.2.1/AlphaTransparency",
            Self::CodingEquations => "ST2067-21:2023:6.2.1/CodingEquations",
            Self::ColorPrimaries => "ST2067-21:2023:6.2.1/ColorPrimaries",
            Self::FieldDominance => "ST2067-21:2023:6.2.1/FieldDominance",
            Self::FrameLayout => "ST2067-21:2023:6.2.1/FrameLayout",
            Self::FrameLayoutInterlaced => "ST2067-21:2023:6.2.1/FrameLayoutInterlaced",
            Self::ImageAlignmentOffset => "ST2067-21:2023:6.2.1/ImageAlignmentOffset",
            Self::ImageEndOffset => "ST2067-21:2023:6.2.1/ImageEndOffset",
            Self::ImageStartOffset => "ST2067-21:2023:6.2.1/ImageStartOffset",
            Self::SampledHeight => "ST2067-21:2023:6.2.1/SampledHeight",
            Self::SampledWidth => "ST2067-21:2023:6.2.1/SampledWidth",
            Self::SampledXOffset => "ST2067-21:2023:6.2.1/SampledXOffset",
            Self::SampledYOffset => "ST2067-21:2023:6.2.1/SampledYOffset",
            Self::StoredF2Offset => "ST2067-21:2023:6.2.1/StoredF2Offset",
            Self::TransferCharacteristic => "ST2067-21:2023:6.2.1/TransferCharacteristic",
            Self::TransferCharacteristicUnknown => "ST2067-21:2023:6.2.2/TransferCharacteristic",
            Self::CodingEquationsUnknown => "ST2067-21:2023:6.2.3/CodingEquations",
            Self::ColorPrimariesUnknown => "ST2067-21:2023:6.2.4/ColorPrimaries",
            Self::J2KRequired => "ST2067-21:2023:6.2.5/J2KRequired",
            Self::AlphaMaxRef => "ST2067-21:2023:6.3/AlphaMaxRef",
            Self::AlphaMinRef => "ST2067-21:2023:6.3/AlphaMinRef",
            Self::ComponentMaxRef => "ST2067-21:2023:6.3/ComponentMaxRef",
            Self::ComponentMinRef => "ST2067-21:2023:6.3/ComponentMinRef",
            Self::Palette => "ST2067-21:2023:6.3/Palette",
            Self::PaletteLayout => "ST2067-21:2023:6.3/PaletteLayout",
            Self::ScanningDirection => "ST2067-21:2023:6.3/ScanningDirection",
            Self::ComponentRefValues => "ST2067-21:2023:6.3.2/ComponentRefValues",
            Self::AlphaSampleDepth => "ST2067-21:2023:6.4/AlphaSampleDepth",
            Self::ColorSiting => "ST2067-21:2023:6.4/ColorSiting",
            Self::ComponentDepth => "ST2067-21:2023:6.4/ComponentDepth",
            Self::HorizontalSubsampling => "ST2067-21:2023:6.4/HorizontalSubsampling",
            Self::PaddingBits => "ST2067-21:2023:6.4/PaddingBits",
            Self::ReversedByteOrder => "ST2067-21:2023:6.4/ReversedByteOrder",
            Self::VerticalSubsampling => "ST2067-21:2023:6.4/VerticalSubsampling",
            Self::BlackRefLevel => "ST2067-21:2023:6.4.3/BlackRefLevel",
            Self::ColorRange => "ST2067-21:2023:6.4.3/ColorRange",
            Self::WhiteRefLevel => "ST2067-21:2023:6.4.3/WhiteRefLevel",
            Self::AudioSampleRate => "ST2067-21:2023:6.5/AudioSampleRate",
            Self::QuantizationBits => "ST2067-21:2023:6.5/QuantizationBits",
            Self::CodingStyle => "ST2067-21:2023:6.5.2/CodingStyle",
            Self::J2CLayout => "ST2067-21:2023:6.5.2/J2CLayout",
            Self::J2KExtendedCapabilities => "ST2067-21:2023:6.5.2/J2KExtendedCapabilities",
            Self::Jpeg2000SubDescriptor => "ST2067-21:2023:6.5.2/JPEG2000SubDescriptor",
            Self::J2KHtNotAllowed => "ST2067-21:2023:6.2.5/J2K-HT-Not-Allowed",
            Self::J2K4KResolution => "ST2067-21:2023:6.2.5/J2K-4K-Resolution",
            Self::J2K2KResolution => "ST2067-21:2023:6.2.5/J2K-2K-Resolution",
            Self::J2KBcpResolution => "ST2067-21:2023:6.2.5/J2K-BCP-Resolution",
            Self::ApplicationIdentification => "ST2067-21:2023:7.1/ApplicationIdentification",
            Self::ContentMaturityRatingAgency => "ST2067-21:2023:7.1/ContentMaturityRating-Agency",
            Self::ContentMaturityRatingAgencyUri => {
                "ST2067-21:2023:7.1/ContentMaturityRating-Agency-URI"
            }
            Self::HomogeneousImageEssence => "ST2067-21:2023:7.2/HomogeneousImageEssence",
            Self::AppIdMismatch => "ST2067-21:2023:7.1/AppIdMismatch",
            Self::SegmentDurationMultiple => "ST2067-21:2023:7.4/SegmentDurationMultiple",
            Self::MaxCLLMaxFALL => "ST2067-21:2023:7.5/MaxCLLMaxFALL",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::FrameRate => "Frame rate is not in the permitted set for App2E.",
            Self::Resolution => "Image resolution is not in the permitted set for App2E.",
            Self::EmptyLanguageTag => "Locale language tag is empty.",
            Self::MalformedLanguageTag => "Locale language tag is not a valid BCP-47 subtag.",
            Self::RegionCode => "Region subtag in a language tag is not valid.",
            Self::ColorSystem => "Color system designator is not in the permitted set.",
            Self::RequiredStoredWidth => "RGBA/CDCI descriptor is missing the required StoredWidth field.",
            Self::RequiredStoredHeight => "RGBA/CDCI descriptor is missing the required StoredHeight field.",
            Self::RequiredSampleRate => "RGBA/CDCI descriptor is missing the required SampleRate field.",
            Self::RequiredFrameLayout => "RGBA/CDCI descriptor is missing the required FrameLayout field.",
            Self::RequiredColorPrimaries => "RGBA/CDCI descriptor is missing the required ColorPrimaries field.",
            Self::RequiredTransferCharacteristic => "RGBA/CDCI descriptor is missing the required TransferCharacteristic field.",
            Self::RequiredPictureCompression => "RGBA/CDCI descriptor is missing the required PictureCompression field.",
            Self::RequiredComponentDepth => "CDCI descriptor is missing the required ComponentDepth field.",
            Self::RequiredChannelCount => "WavePCM descriptor is missing the required ChannelCount field.",
            Self::RequiredQuantizationBits => "WavePCM descriptor is missing the required QuantizationBits field.",
            Self::AlphaTransparency => "Alpha transparency mode is not permitted in App2E.",
            Self::CodingEquations => "CodingEquations field is absent from the picture descriptor (Table 8).",
            Self::ColorPrimaries => "ColorPrimaries field is absent from the picture descriptor (Table 8).",
            Self::FieldDominance => "FieldDominance value is not permitted for the declared FrameLayout.",
            Self::FrameLayout => "FrameLayout value is not in the permitted set for App2E.",
            Self::FrameLayoutInterlaced => "FrameLayout declares interlaced content, which is not permitted in App2E.",
            Self::ImageAlignmentOffset => "ImageAlignmentOffset must be zero.",
            Self::ImageEndOffset => "ImageEndOffset must be zero.",
            Self::ImageStartOffset => "ImageStartOffset must be zero.",
            Self::SampledHeight => "SampledHeight must equal StoredHeight.",
            Self::SampledWidth => "SampledWidth must equal StoredWidth.",
            Self::SampledXOffset => "SampledXOffset must be zero.",
            Self::SampledYOffset => "SampledYOffset must be zero.",
            Self::StoredF2Offset => "StoredF2Offset must be zero.",
            Self::TransferCharacteristic => "TransferCharacteristic field is absent from the picture descriptor (Table 8).",
            Self::TransferCharacteristicUnknown => "TransferCharacteristic UL is present but not a recognized value.",
            Self::CodingEquationsUnknown => "CodingEquations UL is present but not a recognized value.",
            Self::ColorPrimariesUnknown => "ColorPrimaries UL is present but not a recognized value.",
            Self::J2KRequired => "Video essence is not JPEG 2000 encoded as required by App2E.",
            Self::AlphaMaxRef => "AlphaMaxRef value is not permitted.",
            Self::AlphaMinRef => "AlphaMinRef value is not permitted.",
            Self::ComponentMaxRef => "ComponentMaxRef value is not in the permitted range.",
            Self::ComponentMinRef => "ComponentMinRef value is not in the permitted range.",
            Self::Palette => "Palette is present; palette images are not permitted in App2E.",
            Self::PaletteLayout => "PaletteLayout is present; palette layout is not permitted in App2E.",
            Self::ScanningDirection => "ScanningDirection value is not in the permitted set.",
            Self::ComponentRefValues => "Component max/min reference values are inconsistent with bit depth.",
            Self::AlphaSampleDepth => "AlphaSampleDepth value is not permitted.",
            Self::ColorSiting => "ColorSiting value is not in the permitted set.",
            Self::ComponentDepth => "ComponentDepth value is not in the permitted set (8 / 10 / 12 / 16).",
            Self::HorizontalSubsampling => "HorizontalSubsampling value is not in the permitted set.",
            Self::PaddingBits => "PaddingBits must be zero.",
            Self::ReversedByteOrder => "ReversedByteOrder flag is set; byte reversal is not permitted.",
            Self::VerticalSubsampling => "VerticalSubsampling value is not in the permitted set.",
            Self::BlackRefLevel => "BlackRefLevel value is inconsistent with ComponentDepth.",
            Self::ColorRange => "ColorRange value is not in the permitted set.",
            Self::WhiteRefLevel => "WhiteRefLevel value is inconsistent with ComponentDepth.",
            Self::AudioSampleRate => "Audio sample rate must be 48 000 Hz.",
            Self::QuantizationBits => "QuantizationBits must be 16 or 24.",
            Self::CodingStyle => "JPEG 2000 codestream coding style is not compliant.",
            Self::J2CLayout => "JPEG 2000 codestream layout does not meet App2E requirements.",
            Self::J2KExtendedCapabilities => "JPEG 2000 extended capabilities are declared but not permitted.",
            Self::Jpeg2000SubDescriptor => "JPEG2000SubDescriptor is absent or incomplete.",
            Self::J2KHtNotAllowed                => "JPEG 2000 HT (ISO 15444-15) is not permitted by App2E 2020.",
            Self::J2K4KResolution                => "JPEG 2000 IMF 4K Profile: stored resolution is outside the permitted range.",
            Self::J2K2KResolution                => "JPEG 2000 IMF 2K Profile: stored resolution is outside the permitted range.",
            Self::J2KBcpResolution               => "JPEG 2000 Broadcast Contribution Profile: stored resolution is outside the permitted range.",
            Self::ApplicationIdentification      => "ApplicationIdentification is required for App2E compositions.",
            Self::ContentMaturityRatingAgency    => "ContentMaturityRating Agency is empty.",
            Self::ContentMaturityRatingAgencyUri => "ContentMaturityRating Agency is not a valid xs:anyURI.",
            Self::HomogeneousImageEssence        => "All image essence in a composition shall use the same color system.",
            Self::AppIdMismatch => "Application identifier in CPL ExtensionProperties does not match the expected App2E URI.",
            Self::SegmentDurationMultiple => "Segment duration must be an integer multiple of 5 edit units.",
            Self::MaxCLLMaxFALL => "MaxCLL / MaxFALL HDR metadata is absent; recommended for HDR content.",
        }
    }

    fn default_severity(&self) -> Severity {
        match self {
            Self::MaxCLLMaxFALL => Severity::Info,
            Self::AppIdMismatch | Self::Jpeg2000SubDescriptor => Severity::Warning,
            Self::ContentMaturityRatingAgency | Self::ContentMaturityRatingAgencyUri => {
                Severity::Error
            }
            _ => Severity::Error,
        }
    }

    fn category(&self) -> Category {
        match self {
            Self::EmptyLanguageTag
            | Self::MalformedLanguageTag
            | Self::RegionCode
            | Self::AppIdMismatch
            | Self::ApplicationIdentification
            | Self::ContentMaturityRatingAgency
            | Self::ContentMaturityRatingAgencyUri => Category::Metadata,

            Self::RequiredChannelCount
            | Self::RequiredQuantizationBits
            | Self::AudioSampleRate
            | Self::QuantizationBits => Category::Audio,

            Self::J2KRequired
            | Self::CodingStyle
            | Self::J2CLayout
            | Self::J2KExtendedCapabilities
            | Self::Jpeg2000SubDescriptor
            | Self::J2KHtNotAllowed
            | Self::J2K4KResolution
            | Self::J2K2KResolution
            | Self::J2KBcpResolution
            | Self::RequiredStoredWidth
            | Self::RequiredStoredHeight
            | Self::RequiredSampleRate
            | Self::RequiredFrameLayout
            | Self::RequiredColorPrimaries
            | Self::RequiredTransferCharacteristic
            | Self::RequiredPictureCompression
            | Self::RequiredComponentDepth => Category::Encoding,

            Self::SegmentDurationMultiple => Category::Timing,

            _ => Category::Video,
        }
    }
}

impl St2067_21_2023 {
    pub const ALL: &'static [Self] = &[
        Self::FrameRate,
        Self::Resolution,
        Self::EmptyLanguageTag,
        Self::MalformedLanguageTag,
        Self::RegionCode,
        Self::ColorSystem,
        Self::RequiredStoredWidth,
        Self::RequiredStoredHeight,
        Self::RequiredSampleRate,
        Self::RequiredFrameLayout,
        Self::RequiredColorPrimaries,
        Self::RequiredTransferCharacteristic,
        Self::RequiredPictureCompression,
        Self::RequiredComponentDepth,
        Self::RequiredChannelCount,
        Self::RequiredQuantizationBits,
        Self::AlphaTransparency,
        Self::CodingEquations,
        Self::ColorPrimaries,
        Self::FieldDominance,
        Self::FrameLayout,
        Self::FrameLayoutInterlaced,
        Self::ImageAlignmentOffset,
        Self::ImageEndOffset,
        Self::ImageStartOffset,
        Self::SampledHeight,
        Self::SampledWidth,
        Self::SampledXOffset,
        Self::SampledYOffset,
        Self::StoredF2Offset,
        Self::TransferCharacteristic,
        Self::TransferCharacteristicUnknown,
        Self::CodingEquationsUnknown,
        Self::ColorPrimariesUnknown,
        Self::J2KRequired,
        Self::AlphaMaxRef,
        Self::AlphaMinRef,
        Self::ComponentMaxRef,
        Self::ComponentMinRef,
        Self::Palette,
        Self::PaletteLayout,
        Self::ScanningDirection,
        Self::ComponentRefValues,
        Self::AlphaSampleDepth,
        Self::ColorSiting,
        Self::ComponentDepth,
        Self::HorizontalSubsampling,
        Self::PaddingBits,
        Self::ReversedByteOrder,
        Self::VerticalSubsampling,
        Self::BlackRefLevel,
        Self::ColorRange,
        Self::WhiteRefLevel,
        Self::AudioSampleRate,
        Self::QuantizationBits,
        Self::CodingStyle,
        Self::J2CLayout,
        Self::J2KExtendedCapabilities,
        Self::Jpeg2000SubDescriptor,
        Self::J2KHtNotAllowed,
        Self::J2K4KResolution,
        Self::J2K2KResolution,
        Self::J2KBcpResolution,
        Self::ApplicationIdentification,
        Self::ContentMaturityRatingAgency,
        Self::ContentMaturityRatingAgencyUri,
        Self::HomogeneousImageEssence,
        Self::AppIdMismatch,
        Self::SegmentDurationMultiple,
        Self::MaxCLLMaxFALL,
    ];
}

impl_into_string!(St2067_21_2023);

// ─────────────────────────────────────────────────────────────────────────────
// ST 2067-21:2025
// ─────────────────────────────────────────────────────────────────────────────

/// Validation codes defined by SMPTE ST 2067-21:2025 (App2E, timed-text additions).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum St2067_21_2025 {
    /// Timed text track designated as Forced Narrative (FN) does not comply with §5.6.
    FNTimedText,
    /// Timed text track designated as HI-Caption (HIC) does not comply with §5.6.
    HICTimedText,
}

impl ValidationCode for St2067_21_2025 {
    fn code(&self) -> &'static str {
        match self {
            Self::FNTimedText => "ST2067-21:2025:5.6/FNTimedText",
            Self::HICTimedText => "ST2067-21:2025:5.6/HICTimedText",
        }
    }
    fn description(&self) -> &'static str {
        match self {
            Self::FNTimedText => {
                "Timed text track designated as Forced Narrative (FN) does not comply with §5.6."
            }
            Self::HICTimedText => {
                "Timed text track designated as HI-Caption (HIC) does not comply with §5.6."
            }
        }
    }
    fn default_severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> Category {
        Category::Subtitle
    }
}

impl St2067_21_2025 {
    pub const ALL: &'static [Self] = &[Self::FNTimedText, Self::HICTimedText];
}

impl_into_string!(St2067_21_2025);
