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
// Edition-agnostic App #2E rules
// ─────────────────────────────────────────────────────────────────────────────

/// An App #2E check, independent of the ST 2067-21 edition.
///
/// The validator reports findings by rule; each edition's code enum
/// ([`St2067_21_2014`], [`St2067_21_2016`], [`St2067_21_2020`],
/// [`St2067_21_2023`], [`St2067_21_2026`]) maps the rules that edition's
/// text actually contains to a code carrying that edition's label and
/// clause. A rule with no code in an edition is not part of that edition
/// and is never reported under it.
///
/// Variant naming notes:
/// - Variants ending in `Unknown` check for unrecognized UL values (wrong value).
/// - Variants starting with `Required` check for missing mandatory fields.
/// - `*Missing` variants (e.g. `ColorPrimariesMissing`) check field presence.
/// - `*Unknown` variants (e.g. `ColorPrimariesUnknown`) check value validity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum App2ERule {
    // ── Frame rates and resolutions ─────────────────────────────────────
    /// Frame rate is not in the permitted set for App2E.
    FrameRate,
    /// Image resolution is not in the permitted set for App2E.
    Resolution,

    // ── Color system ────────────────────────────────────────────────────
    /// Color system designator is not in the permitted set.
    ColorSystem,

    // ── Required picture-descriptor fields ───────────────────────────────
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

    // ── Picture descriptor constraints ────────────────────────────────
    /// Alpha transparency mode is not permitted in App2E.
    AlphaTransparency,
    /// `CodingEquations` field is absent from the picture descriptor.
    CodingEquationsMissing,
    /// `ColorPrimaries` field is absent from the picture descriptor.
    ColorPrimariesMissing,
    /// `FieldDominance` value is not permitted for the declared `FrameLayout`.
    FieldDominance,
    /// `FrameLayout` value is not in the permitted set for App2E.
    FrameLayout,
    /// Frame structure (progressive/interlaced) outside the closest image-characteristics column.
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
    /// `TransferCharacteristic` field is absent from the picture descriptor.
    TransferCharacteristicMissing,

    // ── Transfer characteristic value ────────────────────────────────
    /// `TransferCharacteristic` UL is present but not a recognized value.
    TransferCharacteristicUnknown,

    // ── Coding equations value ────────────────────────────────────────
    /// `CodingEquations` UL is present but not a recognized value.
    CodingEquationsUnknown,

    // ── Color primaries value ─────────────────────────────────────────
    /// `ColorPrimaries` UL is present but not a recognized value.
    ColorPrimariesUnknown,

    // ── JPEG 2000 requirement ─────────────────────────────────────────
    /// Video essence is not JPEG 2000 encoded as required by App2E.
    J2KRequired,

    // ── RGBA descriptor ─────────────────────────────────────────────────
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

    // ── Component reference values ────────────────────────────────────
    /// Component max/min reference values are inconsistent with bit depth.
    ComponentRefValues,

    // ── Bit depth and chroma ────────────────────────────────────────────
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

    // ── Luma range ────────────────────────────────────────────────────
    /// `BlackRefLevel` value is inconsistent with bit depth.
    BlackRefLevel,
    /// `ColorRange` value is not in the permitted set.
    ColorRange,
    /// `WhiteRefLevel` value is inconsistent with bit depth.
    WhiteRefLevel,

    // ── JPEG 2000 sub-descriptor ─────────────────────────────────────
    /// J2K codestream coding style is not compliant.
    CodingStyle,
    /// JPEG 2000 codestream layout (J2C) does not meet App2E requirements.
    J2CLayout,
    /// `J2KExtendedCapabilities` is missing although ISO/IEC 15444-15
    /// (HTJ2K) coding is used.
    J2KExtendedCapabilities,
    /// JPEG2000SubDescriptor is absent or incomplete.
    Jpeg2000SubDescriptor,

    // ── JPEG 2000 resolution profiles ────────────────────────────────
    /// JPEG 2000 HT (ISO 15444-15) is not permitted by the App2E 2020 edition.
    J2KHtNotAllowed,
    /// JPEG 2000 IMF 4K Profile: stored resolution is outside the permitted range.
    J2K4KResolution,
    /// JPEG 2000 IMF 2K Profile: stored resolution is outside the permitted range.
    J2K2KResolution,
    /// JPEG 2000 Broadcast Contribution Profile: stored resolution is outside the permitted range.
    J2KBcpResolution,
    /// JPEG 2000 IMF 8K Profile: stored resolution is outside the permitted range.
    J2K8KResolution,
    /// The JPEG 2000 profile is not one the edition's JPEG 2000 table lists.
    J2KProfileNotAllowed,

    // ── Application identification ──────────────────────────────────────
    /// ApplicationIdentification is required for App2E compositions.
    ApplicationIdentification,

    // ── Homogeneous image essence ───────────────────────────────────────
    /// All image essence in a composition shall use the same color system.
    HomogeneousImageEssence,

    // ── Application identification ──────────────────────────────────────
    /// Application identifier in CPL ExtensionProperties does not match the App2E URI.
    AppIdMismatch,

    // ── Segment duration ─────────────────────────────────────────────────
    /// Segment duration is not an integer multiple of 5 edit units as required by App2E.
    SegmentDurationMultiple,

    // ── HDR metadata ────────────────────────────────────────────────────
    /// `MaxCLL` / `MaxFALL` HDR metadata is absent; recommended for HDR content.
    MaxCLLMaxFALL,
}

impl App2ERule {
    pub fn description(&self) -> &'static str {
        match self {
            Self::FrameRate => "Frame rate is not in the permitted set for App2E.",
            Self::Resolution => "Image resolution is not in the permitted set for App2E.",
            Self::ColorSystem => "Color system designator is not in the permitted set.",
            Self::RequiredStoredWidth => "RGBA/CDCI descriptor is missing the required StoredWidth field.",
            Self::RequiredStoredHeight => "RGBA/CDCI descriptor is missing the required StoredHeight field.",
            Self::RequiredSampleRate => "RGBA/CDCI descriptor is missing the required SampleRate field.",
            Self::RequiredFrameLayout => "RGBA/CDCI descriptor is missing the required FrameLayout field.",
            Self::RequiredColorPrimaries => "RGBA/CDCI descriptor is missing the required ColorPrimaries field.",
            Self::RequiredTransferCharacteristic => "RGBA/CDCI descriptor is missing the required TransferCharacteristic field.",
            Self::RequiredPictureCompression => "RGBA/CDCI descriptor is missing the required PictureCompression field.",
            Self::RequiredComponentDepth => "CDCI descriptor is missing the required ComponentDepth field.",
            Self::AlphaTransparency => "Alpha transparency mode is not permitted in App2E.",
            Self::CodingEquationsMissing => "CodingEquations field is absent from the picture descriptor.",
            Self::ColorPrimariesMissing => "ColorPrimaries field is absent from the picture descriptor.",
            Self::FieldDominance => "FieldDominance value is not permitted for the declared FrameLayout.",
            Self::FrameLayout => "FrameLayout value is not in the permitted set for App2E.",
            Self::FrameLayoutInterlaced => "Image frame structure (progressive or interlaced) does not fit the closest permitted column of the edition's image-characteristics table.",
            Self::ImageAlignmentOffset => "ImageAlignmentOffset must be zero.",
            Self::ImageEndOffset => "ImageEndOffset must be zero.",
            Self::ImageStartOffset => "ImageStartOffset must be zero.",
            Self::SampledHeight => "SampledHeight must equal StoredHeight.",
            Self::SampledWidth => "SampledWidth must equal StoredWidth.",
            Self::SampledXOffset => "SampledXOffset must be zero.",
            Self::SampledYOffset => "SampledYOffset must be zero.",
            Self::StoredF2Offset => "StoredF2Offset must be zero.",
            Self::TransferCharacteristicMissing => "TransferCharacteristic field is absent from the picture descriptor.",
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
            Self::ComponentDepth => "ComponentDepth is missing, or the pixel bit depth is not permitted for the image characteristics.",
            Self::HorizontalSubsampling => "HorizontalSubsampling value is not in the permitted set.",
            Self::PaddingBits => "PaddingBits must be zero.",
            Self::ReversedByteOrder => "ReversedByteOrder flag is set; byte reversal is not permitted.",
            Self::VerticalSubsampling => "VerticalSubsampling value is not in the permitted set.",
            Self::BlackRefLevel => "BlackRefLevel value is inconsistent with ComponentDepth.",
            Self::ColorRange => "ColorRange value is not in the permitted set.",
            Self::WhiteRefLevel => "WhiteRefLevel value is inconsistent with ComponentDepth.",
            Self::CodingStyle => "JPEG 2000 codestream coding style is not compliant.",
            Self::J2CLayout => "JPEG 2000 codestream layout does not meet App2E requirements.",
            // Table 14 requires the item when ISO/IEC 15444-15 coding is used:
            // https://github.com/SMPTE/st2067-21/issues/2
            Self::J2KExtendedCapabilities => "J2KExtendedCapabilities shall be present when ISO/IEC 15444-15 (HTJ2K) coding is used, but it is missing.",
            Self::Jpeg2000SubDescriptor => "JPEG2000SubDescriptor is absent or incomplete.",
            Self::J2KHtNotAllowed                => "JPEG 2000 HT (ISO 15444-15) is not permitted by App2E 2020.",
            Self::J2K4KResolution                => "JPEG 2000 IMF 4K Profile: stored resolution is outside the permitted range.",
            Self::J2K2KResolution                => "JPEG 2000 IMF 2K Profile: stored resolution is outside the permitted range.",
            Self::J2KBcpResolution               => "JPEG 2000 Broadcast Contribution Profile: stored resolution is outside the permitted range.",
            Self::J2K8KResolution                => "JPEG 2000 IMF 8K Profile: stored resolution is outside the permitted range.",
            Self::J2KProfileNotAllowed           => "The JPEG 2000 profile is not listed in the edition's JPEG 2000 profile table.",
            Self::ApplicationIdentification      => "ApplicationIdentification is required for App2E compositions.",
            Self::HomogeneousImageEssence        => "All image essence in a composition shall use the same color system.",
            Self::AppIdMismatch => "ApplicationIdentification in CPL ExtensionProperties does not include the expected App2E URI.",
            Self::SegmentDurationMultiple => "Segment duration must be an integer multiple of 5 edit units.",
            Self::MaxCLLMaxFALL => "MaxCLL / MaxFALL HDR metadata is absent; recommended for HDR content.",
        }
    }

    pub fn default_severity(&self) -> Severity {
        match self {
            Self::MaxCLLMaxFALL => Severity::Info,
            Self::AppIdMismatch | Self::Jpeg2000SubDescriptor => Severity::Warning,
            _ => Severity::Error,
        }
    }

    pub fn category(&self) -> Category {
        match self {
            Self::AppIdMismatch | Self::ApplicationIdentification => Category::Metadata,

            Self::J2KRequired
            | Self::CodingStyle
            | Self::J2CLayout
            | Self::J2KExtendedCapabilities
            | Self::Jpeg2000SubDescriptor
            | Self::J2KHtNotAllowed
            | Self::J2K4KResolution
            | Self::J2K2KResolution
            | Self::J2KBcpResolution
            | Self::J2K8KResolution
            | Self::J2KProfileNotAllowed
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
    pub fn example(&self) -> Option<&'static str> {
        Some(match self {
            Self::FrameRate =>
                "<SampleRate>48/1</SampleRate>  <!-- 48 fps is not in Table 3 -->",
            Self::Resolution =>
                "<StoredWidth>9000</StoredWidth><StoredHeight>7000</StoredHeight>  <!-- larger than every image-characteristics column -->",
            Self::ColorSystem =>
                "ColorPrimaries + TransferCharacteristic + CodingEquations triple doesn't map to a permitted App2E ColorN system",
            Self::RequiredStoredWidth =>
                "<RGBADescriptor>…</RGBADescriptor>  <!-- no <StoredWidth> child -->",
            Self::RequiredStoredHeight =>
                "<RGBADescriptor>…</RGBADescriptor>  <!-- no <StoredHeight> child -->",
            Self::RequiredSampleRate =>
                "<RGBADescriptor>…</RGBADescriptor>  <!-- no <SampleRate> child -->",
            Self::RequiredFrameLayout =>
                "<RGBADescriptor>…</RGBADescriptor>  <!-- no <FrameLayout> child -->",
            Self::RequiredColorPrimaries =>
                "<RGBADescriptor>…</RGBADescriptor>  <!-- no <ColorPrimaries> UL child -->",
            Self::RequiredTransferCharacteristic =>
                "<RGBADescriptor>…</RGBADescriptor>  <!-- no <TransferCharacteristic> UL child -->",
            Self::RequiredPictureCompression =>
                "<RGBADescriptor>…</RGBADescriptor>  <!-- no <PictureCompression> UL child -->",
            Self::RequiredComponentDepth =>
                "<CDCIDescriptor>…</CDCIDescriptor>  <!-- no <ComponentDepth> child -->",
            Self::AlphaTransparency =>
                "<AlphaTransparency>true</AlphaTransparency>  <!-- App2E disallows alpha -->",
            Self::CodingEquationsMissing =>
                "<CDCIDescriptor>…</CDCIDescriptor>  <!-- no <CodingEquations> UL child -->",
            Self::ColorPrimariesMissing =>
                "<CDCIDescriptor>…</CDCIDescriptor>  <!-- no <ColorPrimaries> UL child -->",
            Self::FieldDominance =>
                "<FieldDominance>2</FieldDominance> on <FrameLayout>FullFrame</FrameLayout>",
            Self::FrameLayout =>
                "<FrameLayout>MixedFields</FrameLayout>  <!-- not in {FullFrame,SeparateFields,SingleField} -->",
            Self::FrameLayoutInterlaced =>
                "<FrameLayout>SeparateFields</FrameLayout> with <SampleRate>24/1</SampleRate>  <!-- interlaced only at 25, 30 or 30000/1001 -->",
            Self::ImageAlignmentOffset =>
                "<ImageAlignmentOffset>16</ImageAlignmentOffset>  <!-- must be 0 -->",
            Self::ImageEndOffset =>
                "<ImageEndOffset>32</ImageEndOffset>  <!-- must be 0 -->",
            Self::ImageStartOffset =>
                "<ImageStartOffset>16</ImageStartOffset>  <!-- must be 0 -->",
            Self::SampledHeight =>
                "<StoredHeight>1080</StoredHeight><SampledHeight>1056</SampledHeight>",
            Self::SampledWidth =>
                "<StoredWidth>1920</StoredWidth><SampledWidth>1872</SampledWidth>",
            Self::SampledXOffset =>
                "<SampledXOffset>4</SampledXOffset>  <!-- must be 0 -->",
            Self::SampledYOffset =>
                "<SampledYOffset>4</SampledYOffset>  <!-- must be 0 -->",
            Self::StoredF2Offset =>
                "<StoredF2Offset>1</StoredF2Offset>  <!-- must be 0 -->",
            Self::TransferCharacteristicMissing =>
                "<CDCIDescriptor>…</CDCIDescriptor>  <!-- no <TransferCharacteristic> UL child -->",
            Self::TransferCharacteristicUnknown =>
                "<TransferCharacteristic>urn:smpte:ul:060e2b34.04010101.04010101.01010500</TransferCharacteristic>  <!-- UL not in §6.2.2 table -->",
            Self::CodingEquationsUnknown =>
                "<CodingEquations>urn:smpte:ul:060e2b34.04010101.04010101.02020500</CodingEquations>  <!-- UL not in §6.2.3 table -->",
            Self::ColorPrimariesUnknown =>
                "<ColorPrimaries>urn:smpte:ul:060e2b34.04010101.04010101.03030500</ColorPrimaries>  <!-- UL not in §6.2.4 table -->",
            Self::J2KRequired =>
                "<PictureCompression>urn:smpte:ul:060e2b34.04010101.04010202.01020500</PictureCompression>  <!-- not a JPEG-2000 UL -->",
            Self::AlphaMaxRef =>
                "<AlphaMaxRef>1024</AlphaMaxRef>  <!-- outside permitted depth-derived range -->",
            Self::AlphaMinRef =>
                "<AlphaMinRef>-1</AlphaMinRef>  <!-- outside permitted depth-derived range -->",
            Self::ComponentMaxRef =>
                "<ComponentMaxRef>9999</ComponentMaxRef>  <!-- beyond (2^ComponentDepth)-1 -->",
            Self::ComponentMinRef =>
                "<ComponentMinRef>-5</ComponentMinRef>  <!-- must be ≥ 0 -->",
            Self::Palette =>
                "<Palette>…</Palette>  <!-- palette images forbidden in App2E -->",
            Self::PaletteLayout =>
                "<PaletteLayout>…</PaletteLayout>  <!-- forbidden in App2E -->",
            Self::ScanningDirection =>
                "<ScanningDirection>5</ScanningDirection>  <!-- not in permitted set -->",
            Self::ComponentRefValues =>
                "<ComponentDepth>10</ComponentDepth><ComponentMaxRef>4096</ComponentMaxRef>  <!-- 4096 > (2^10)-1 -->",
            Self::AlphaSampleDepth =>
                "<AlphaSampleDepth>7</AlphaSampleDepth>  <!-- not in {8,10,12,16} -->",
            Self::ColorSiting =>
                "<ColorSiting>9</ColorSiting>  <!-- not in permitted set -->",
            Self::ComponentDepth =>
                "<ComponentDepth>9</ComponentDepth>  <!-- not in {8,10,12,16} -->",
            Self::HorizontalSubsampling =>
                "<HorizontalSubsampling>3</HorizontalSubsampling>  <!-- not in {1,2,4} -->",
            Self::PaddingBits =>
                "<PaddingBits>4</PaddingBits>  <!-- must be 0 -->",
            Self::ReversedByteOrder =>
                "<ReversedByteOrder>true</ReversedByteOrder>  <!-- byte reversal forbidden -->",
            Self::VerticalSubsampling =>
                "<VerticalSubsampling>3</VerticalSubsampling>  <!-- not in {1,2} -->",
            Self::BlackRefLevel =>
                "<ComponentDepth>10</ComponentDepth><BlackRefLevel>16</BlackRefLevel>  <!-- expected 64 for 10-bit -->",
            Self::ColorRange =>
                "<ComponentDepth>10</ComponentDepth><ColorRange>1019</ColorRange>  <!-- not in permitted set -->",
            Self::WhiteRefLevel =>
                "<ComponentDepth>10</ComponentDepth><WhiteRefLevel>235</WhiteRefLevel>  <!-- expected 940 for 10-bit -->",
            Self::CodingStyle =>
                "JPEG2000SubDescriptor CodingStyleDefault Scod byte declares non-compliant entropy coding",
            Self::J2CLayout =>
                "JPEG2000SubDescriptor declares >1 layer or unsupported precinct sizes",
            Self::J2KExtendedCapabilities =>
                "<JPEG2000SubDescriptor><Rsiz>16384</Rsiz>…</JPEG2000SubDescriptor>  <!-- HTJ2K (Rsiz bit 14) but no <J2KExtendedCapabilities> -->",
            Self::Jpeg2000SubDescriptor =>
                "<PictureDescriptor>…</PictureDescriptor>  <!-- no JPEG2000SubDescriptor child -->",
            Self::J2KHtNotAllowed =>
                "JPEG2000SubDescriptor Profile field declares HTJ2K (ISO 15444-15)",
            Self::J2K4KResolution =>
                "<StoredWidth>5000</StoredWidth>  <!-- IMF 4K profile caps width at 4096 -->",
            Self::J2K2KResolution =>
                "<StoredWidth>2400</StoredWidth>  <!-- IMF 2K profile caps width at 2048 -->",
            Self::J2KBcpResolution =>
                "<StoredWidth>4096</StoredWidth> with a Broadcast Contribution profile  <!-- profile column is 1..3840 x 1..2160 -->",
            Self::J2K8KResolution =>
                "<StoredWidth>3840</StoredWidth> with an 8K IMF profile  <!-- profile column is 4097..8192 x 1..6224 -->",
            Self::J2KProfileNotAllowed =>
                "<PictureCompression>urn:smpte:ul:060e2b34.0401010d.04010202.03010403</PictureCompression>  <!-- 8K IMF profile, not in the 2023 table -->",
            Self::ApplicationIdentification =>
                "<CompositionPlaylist>…</CompositionPlaylist>  <!-- ExtensionProperties has no <ApplicationIdentification> -->",
            Self::HomogeneousImageEssence =>
                "Composition mixes Color1 (BT.709) and Color3 (BT.2020) resources across segments",
            Self::AppIdMismatch =>
                "<ApplicationIdentification>http://example.org/not-app2e</ApplicationIdentification>",
            Self::SegmentDurationMultiple =>
                "<Resource><SourceDuration>7</SourceDuration></Resource>  <!-- not a multiple of 5 -->",
            Self::MaxCLLMaxFALL =>
                "HDR content (PQ TransferCharacteristic) with no <MaxCLL> / <MaxFALL> in ExtensionProperties",
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-edition code catalogues
// ─────────────────────────────────────────────────────────────────────────────

/// Defines one edition's code enum: the rules that exist in that edition's
/// text, each with that edition's label and clause.
macro_rules! define_app2e_edition {
    (
        $(#[$meta:meta])*
        $name:ident { $($variant:ident => $code:literal,)* }
    ) => {
        $(#[$meta])*
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
        pub enum $name {
            $(
                #[doc = $code]
                $variant,
            )*
        }

        impl $name {
            /// Every code of this edition.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)*];

            /// The edition-agnostic rule this code reports.
            pub fn rule(&self) -> App2ERule {
                match self {
                    $(Self::$variant => App2ERule::$variant,)*
                }
            }

            /// The code for `rule` in this edition, or `None` when the
            /// edition's text does not contain the rule.
            #[allow(unreachable_patterns)]
            pub fn for_rule(rule: App2ERule) -> Option<Self> {
                match rule {
                    $(App2ERule::$variant => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }

        impl ValidationCode for $name {
            fn code(&self) -> &'static str {
                match self {
                    $(Self::$variant => $code,)*
                }
            }
            fn description(&self) -> &'static str {
                self.rule().description()
            }
            fn default_severity(&self) -> Severity {
                self.rule().default_severity()
            }
            fn category(&self) -> Category {
                self.rule().category()
            }
            fn example(&self) -> Option<&'static str> {
                self.rule().example()
            }
        }

        impl_into_string!($name);
    };
}

define_app2e_edition! {
    /// Validation codes for SMPTE ST 2067-21:2014 (App #2E, first edition).
    ///
    /// The 2014 edition extends ST 2067-20 (App #2) and only adds
    /// constraints for COLOR.4/COLOR.5: the UL values of §6.1.1–6.1.3, the
    /// Black/White Ref Level and Color Range values of §6.1.4 Table 4, and
    /// the ApplicationIdentification value of §7 Table 5.
    St2067_21_2014 {
        ColorSystem => "ST2067-21:2014:6.1/ColorSystem",
        BlackRefLevel => "ST2067-21:2014:6.1.4/BlackRefLevel",
        WhiteRefLevel => "ST2067-21:2014:6.1.4/WhiteRefLevel",
        ColorRange => "ST2067-21:2014:6.1.4/ColorRange",
        ApplicationIdentification => "ST2067-21:2014:7/ApplicationIdentification",
        AppIdMismatch => "ST2067-21:2014:7/AppIdMismatch",
    }
}

define_app2e_edition! {
    /// Validation codes for SMPTE ST 2067-21:2016 (App #2E).
    ///
    /// The 2016 edition extends ST 2067-20 (App #2) and adds constraints for
    /// COLOR.4–COLOR.7: the UL values of §6.1.1–6.1.3, Component Max/Min Ref
    /// for 12/16-bit R'G'B' (§6.2 Table 6), Black/White Ref Level and Color
    /// Range (§6.3 Table 7), ApplicationIdentification (§7.1 Table 8) and
    /// MaxCLL/MaxFALL (§7.2).
    St2067_21_2016 {
        ColorSystem => "ST2067-21:2016:6.1/ColorSystem",
        ComponentRefValues => "ST2067-21:2016:6.2/ComponentRefValues",
        BlackRefLevel => "ST2067-21:2016:6.3/BlackRefLevel",
        WhiteRefLevel => "ST2067-21:2016:6.3/WhiteRefLevel",
        ColorRange => "ST2067-21:2016:6.3/ColorRange",
        ApplicationIdentification => "ST2067-21:2016:7.1/ApplicationIdentification",
        AppIdMismatch => "ST2067-21:2016:7.1/AppIdMismatch",
        MaxCLLMaxFALL => "ST2067-21:2016:7.2/MaxCLLMaxFALL",
    }
}

define_app2e_edition! {
    /// Validation codes for SMPTE ST 2067-21:2020 (App #2E), including
    /// Amendment 1:2020 (COLOR.8 / HLG).
    ///
    /// Clause numbering of §5–§7 is identical to the 2023 edition. Not in
    /// this edition: HT-J2K (and with it J2KExtendedCapabilities, Table 14
    /// of 2023), which is reported as `J2KHtNotAllowed` instead.
    St2067_21_2020 {
        FrameRate => "ST2067-21:2020:5.2/FrameRate",
        Resolution => "ST2067-21:2020:5.2/Resolution",
        ColorSystem => "ST2067-21:2020:6.2/ColorSystem",
        RequiredStoredWidth => "ST2067-21:2020:6.2/Required-StoredWidth",
        RequiredStoredHeight => "ST2067-21:2020:6.2/Required-StoredHeight",
        RequiredSampleRate => "ST2067-21:2020:6.2/Required-SampleRate",
        RequiredFrameLayout => "ST2067-21:2020:6.2/Required-FrameLayout",
        RequiredColorPrimaries => "ST2067-21:2020:6.2/Required-ColorPrimaries",
        RequiredTransferCharacteristic => "ST2067-21:2020:6.2/Required-TransferCharacteristic",
        RequiredPictureCompression => "ST2067-21:2020:6.2/Required-PictureCompression",
        RequiredComponentDepth => "ST2067-21:2020:6.2/Required-ComponentDepth",
        AlphaTransparency => "ST2067-21:2020:6.2.1/AlphaTransparency",
        CodingEquationsMissing => "ST2067-21:2020:6.2.1/CodingEquationsMissing",
        ColorPrimariesMissing => "ST2067-21:2020:6.2.1/ColorPrimariesMissing",
        FieldDominance => "ST2067-21:2020:6.2.1/FieldDominance",
        FrameLayout => "ST2067-21:2020:6.2.1/FrameLayout",
        FrameLayoutInterlaced => "ST2067-21:2020:6.2.1/FrameLayoutInterlaced",
        ImageAlignmentOffset => "ST2067-21:2020:6.2.1/ImageAlignmentOffset",
        ImageEndOffset => "ST2067-21:2020:6.2.1/ImageEndOffset",
        ImageStartOffset => "ST2067-21:2020:6.2.1/ImageStartOffset",
        SampledHeight => "ST2067-21:2020:6.2.1/SampledHeight",
        SampledWidth => "ST2067-21:2020:6.2.1/SampledWidth",
        SampledXOffset => "ST2067-21:2020:6.2.1/SampledXOffset",
        SampledYOffset => "ST2067-21:2020:6.2.1/SampledYOffset",
        StoredF2Offset => "ST2067-21:2020:6.2.1/StoredF2Offset",
        TransferCharacteristicMissing => "ST2067-21:2020:6.2.1/TransferCharacteristicMissing",
        TransferCharacteristicUnknown => "ST2067-21:2020:6.2.2/TransferCharacteristicUnknown",
        CodingEquationsUnknown => "ST2067-21:2020:6.2.3/CodingEquationsUnknown",
        ColorPrimariesUnknown => "ST2067-21:2020:6.2.4/ColorPrimariesUnknown",
        J2KRequired => "ST2067-21:2020:6.2.5/J2KRequired",
        AlphaMaxRef => "ST2067-21:2020:6.3/AlphaMaxRef",
        AlphaMinRef => "ST2067-21:2020:6.3/AlphaMinRef",
        ComponentMaxRef => "ST2067-21:2020:6.3/ComponentMaxRef",
        ComponentMinRef => "ST2067-21:2020:6.3/ComponentMinRef",
        Palette => "ST2067-21:2020:6.3/Palette",
        PaletteLayout => "ST2067-21:2020:6.3/PaletteLayout",
        ScanningDirection => "ST2067-21:2020:6.3/ScanningDirection",
        ComponentRefValues => "ST2067-21:2020:6.3.2/ComponentRefValues",
        AlphaSampleDepth => "ST2067-21:2020:6.4/AlphaSampleDepth",
        ColorSiting => "ST2067-21:2020:6.4/ColorSiting",
        ComponentDepth => "ST2067-21:2020:6.4/ComponentDepth",
        HorizontalSubsampling => "ST2067-21:2020:6.4/HorizontalSubsampling",
        PaddingBits => "ST2067-21:2020:6.4/PaddingBits",
        ReversedByteOrder => "ST2067-21:2020:6.4/ReversedByteOrder",
        VerticalSubsampling => "ST2067-21:2020:6.4/VerticalSubsampling",
        BlackRefLevel => "ST2067-21:2020:6.4.3/BlackRefLevel",
        ColorRange => "ST2067-21:2020:6.4.3/ColorRange",
        WhiteRefLevel => "ST2067-21:2020:6.4.3/WhiteRefLevel",
        CodingStyle => "ST2067-21:2020:6.5.2/CodingStyle",
        J2CLayout => "ST2067-21:2020:6.5.2/J2CLayout",
        Jpeg2000SubDescriptor => "ST2067-21:2020:6.5.2/JPEG2000SubDescriptor",
        J2KHtNotAllowed => "ST2067-21:2020:6.2.5/J2K-HT-Not-Allowed",
        J2K4KResolution => "ST2067-21:2020:6.2.5/J2K-4K-Resolution",
        J2K2KResolution => "ST2067-21:2020:6.2.5/J2K-2K-Resolution",
        J2KBcpResolution => "ST2067-21:2020:6.2.5/J2K-BCP-Resolution",
        J2KProfileNotAllowed => "ST2067-21:2020:6.2.5/J2K-Profile-Not-Allowed",
        ApplicationIdentification => "ST2067-21:2020:7.1/ApplicationIdentification",
        HomogeneousImageEssence => "ST2067-21:2020:7.2/HomogeneousImageEssence",
        AppIdMismatch => "ST2067-21:2020:7.1/AppIdMismatch",
        SegmentDurationMultiple => "ST2067-21:2020:7.4/SegmentDurationMultiple",
        MaxCLLMaxFALL => "ST2067-21:2020:7.5/MaxCLLMaxFALL",
    }
}

define_app2e_edition! {
    /// Validation codes for SMPTE ST 2067-21:2023 (App #2E).
    ///
    /// Code strings are kept as published by earlier releases.
    St2067_21_2023 {
        FrameRate => "ST2067-21:2023:5.2/FrameRate",
        Resolution => "ST2067-21:2023:5.2/Resolution",
        ColorSystem => "ST2067-21:2023:6.2/ColorSystem",
        RequiredStoredWidth => "ST2067-21:2023:6.2/Required-StoredWidth",
        RequiredStoredHeight => "ST2067-21:2023:6.2/Required-StoredHeight",
        RequiredSampleRate => "ST2067-21:2023:6.2/Required-SampleRate",
        RequiredFrameLayout => "ST2067-21:2023:6.2/Required-FrameLayout",
        RequiredColorPrimaries => "ST2067-21:2023:6.2/Required-ColorPrimaries",
        RequiredTransferCharacteristic => "ST2067-21:2023:6.2/Required-TransferCharacteristic",
        RequiredPictureCompression => "ST2067-21:2023:6.2/Required-PictureCompression",
        RequiredComponentDepth => "ST2067-21:2023:6.2/Required-ComponentDepth",
        AlphaTransparency => "ST2067-21:2023:6.2.1/AlphaTransparency",
        CodingEquationsMissing => "ST2067-21:2023:6.2.1/CodingEquationsMissing",
        ColorPrimariesMissing => "ST2067-21:2023:6.2.1/ColorPrimariesMissing",
        FieldDominance => "ST2067-21:2023:6.2.1/FieldDominance",
        FrameLayout => "ST2067-21:2023:6.2.1/FrameLayout",
        FrameLayoutInterlaced => "ST2067-21:2023:6.2.1/FrameLayoutInterlaced",
        ImageAlignmentOffset => "ST2067-21:2023:6.2.1/ImageAlignmentOffset",
        ImageEndOffset => "ST2067-21:2023:6.2.1/ImageEndOffset",
        ImageStartOffset => "ST2067-21:2023:6.2.1/ImageStartOffset",
        SampledHeight => "ST2067-21:2023:6.2.1/SampledHeight",
        SampledWidth => "ST2067-21:2023:6.2.1/SampledWidth",
        SampledXOffset => "ST2067-21:2023:6.2.1/SampledXOffset",
        SampledYOffset => "ST2067-21:2023:6.2.1/SampledYOffset",
        StoredF2Offset => "ST2067-21:2023:6.2.1/StoredF2Offset",
        TransferCharacteristicMissing => "ST2067-21:2023:6.2.1/TransferCharacteristicMissing",
        TransferCharacteristicUnknown => "ST2067-21:2023:6.2.2/TransferCharacteristicUnknown",
        CodingEquationsUnknown => "ST2067-21:2023:6.2.3/CodingEquationsUnknown",
        ColorPrimariesUnknown => "ST2067-21:2023:6.2.4/ColorPrimariesUnknown",
        J2KRequired => "ST2067-21:2023:6.2.5/J2KRequired",
        AlphaMaxRef => "ST2067-21:2023:6.3/AlphaMaxRef",
        AlphaMinRef => "ST2067-21:2023:6.3/AlphaMinRef",
        ComponentMaxRef => "ST2067-21:2023:6.3/ComponentMaxRef",
        ComponentMinRef => "ST2067-21:2023:6.3/ComponentMinRef",
        Palette => "ST2067-21:2023:6.3/Palette",
        PaletteLayout => "ST2067-21:2023:6.3/PaletteLayout",
        ScanningDirection => "ST2067-21:2023:6.3/ScanningDirection",
        ComponentRefValues => "ST2067-21:2023:6.3.2/ComponentRefValues",
        AlphaSampleDepth => "ST2067-21:2023:6.4/AlphaSampleDepth",
        ColorSiting => "ST2067-21:2023:6.4/ColorSiting",
        ComponentDepth => "ST2067-21:2023:6.4/ComponentDepth",
        HorizontalSubsampling => "ST2067-21:2023:6.4/HorizontalSubsampling",
        PaddingBits => "ST2067-21:2023:6.4/PaddingBits",
        ReversedByteOrder => "ST2067-21:2023:6.4/ReversedByteOrder",
        VerticalSubsampling => "ST2067-21:2023:6.4/VerticalSubsampling",
        BlackRefLevel => "ST2067-21:2023:6.4.3/BlackRefLevel",
        ColorRange => "ST2067-21:2023:6.4.3/ColorRange",
        WhiteRefLevel => "ST2067-21:2023:6.4.3/WhiteRefLevel",
        CodingStyle => "ST2067-21:2023:6.5.2/CodingStyle",
        J2CLayout => "ST2067-21:2023:6.5.2/J2CLayout",
        J2KExtendedCapabilities => "ST2067-21:2023:6.5.2/J2KExtendedCapabilities",
        Jpeg2000SubDescriptor => "ST2067-21:2023:6.5.2/JPEG2000SubDescriptor",
        J2K4KResolution => "ST2067-21:2023:6.2.5/J2K-4K-Resolution",
        J2K2KResolution => "ST2067-21:2023:6.2.5/J2K-2K-Resolution",
        J2KBcpResolution => "ST2067-21:2023:6.2.5/J2K-BCP-Resolution",
        J2KProfileNotAllowed => "ST2067-21:2023:6.2.5/J2K-Profile-Not-Allowed",
        ApplicationIdentification => "ST2067-21:2023:7.1/ApplicationIdentification",
        HomogeneousImageEssence => "ST2067-21:2023:7.2/HomogeneousImageEssence",
        AppIdMismatch => "ST2067-21:2023:7.1/AppIdMismatch",
        SegmentDurationMultiple => "ST2067-21:2023:7.4/SegmentDurationMultiple",
        MaxCLLMaxFALL => "ST2067-21:2023:7.5/MaxCLLMaxFALL",
    }
}

define_app2e_edition! {
    /// Validation codes for SMPTE ST 2067-21:2026 (App #2E, HTML edition).
    ///
    /// Clauses shift by one against 2023 (Image Essence is §6, Image Track
    /// Files §7, Composition §8). Not in this edition: AlphaTransparency
    /// (Table 7 says "Shall be ignored") and HT-J2K exclusion.
    St2067_21_2026 {
        FrameRate => "ST2067-21:2026:6.2.1/FrameRate",
        Resolution => "ST2067-21:2026:6.2.1/Resolution",
        ColorSystem => "ST2067-21:2026:7.2/ColorSystem",
        RequiredStoredWidth => "ST2067-21:2026:7.2.1.1/Required-StoredWidth",
        RequiredStoredHeight => "ST2067-21:2026:7.2.1.1/Required-StoredHeight",
        RequiredSampleRate => "ST2067-21:2026:7.2.1.1/Required-SampleRate",
        RequiredFrameLayout => "ST2067-21:2026:7.2.1.1/Required-FrameLayout",
        RequiredColorPrimaries => "ST2067-21:2026:7.2.1.1/Required-ColorPrimaries",
        RequiredTransferCharacteristic => "ST2067-21:2026:7.2.1.1/Required-TransferCharacteristic",
        RequiredPictureCompression => "ST2067-21:2026:7.2.1.1/Required-PictureCompression",
        RequiredComponentDepth => "ST2067-21:2026:7.4.1/Required-ComponentDepth",
        CodingEquationsMissing => "ST2067-21:2026:7.2.1.1/CodingEquationsMissing",
        ColorPrimariesMissing => "ST2067-21:2026:7.2.1.1/ColorPrimariesMissing",
        FieldDominance => "ST2067-21:2026:7.2.1.1/FieldDominance",
        FrameLayout => "ST2067-21:2026:7.2.1.4/FrameLayout",
        FrameLayoutInterlaced => "ST2067-21:2026:6.2.1/FrameLayoutInterlaced",
        ImageAlignmentOffset => "ST2067-21:2026:7.2.1.1/ImageAlignmentOffset",
        ImageEndOffset => "ST2067-21:2026:7.2.1.1/ImageEndOffset",
        ImageStartOffset => "ST2067-21:2026:7.2.1.1/ImageStartOffset",
        SampledHeight => "ST2067-21:2026:7.2.1.1/SampledHeight",
        SampledWidth => "ST2067-21:2026:7.2.1.1/SampledWidth",
        SampledXOffset => "ST2067-21:2026:7.2.1.1/SampledXOffset",
        SampledYOffset => "ST2067-21:2026:7.2.1.1/SampledYOffset",
        StoredF2Offset => "ST2067-21:2026:7.2.1.1/StoredF2Offset",
        TransferCharacteristicMissing => "ST2067-21:2026:7.2.1.1/TransferCharacteristicMissing",
        TransferCharacteristicUnknown => "ST2067-21:2026:7.2.2/TransferCharacteristicUnknown",
        CodingEquationsUnknown => "ST2067-21:2026:7.2.3/CodingEquationsUnknown",
        ColorPrimariesUnknown => "ST2067-21:2026:7.2.4/ColorPrimariesUnknown",
        J2KRequired => "ST2067-21:2026:7.2.5/J2KRequired",
        AlphaMaxRef => "ST2067-21:2026:7.3.1/AlphaMaxRef",
        AlphaMinRef => "ST2067-21:2026:7.3.1/AlphaMinRef",
        ComponentMaxRef => "ST2067-21:2026:7.3.1/ComponentMaxRef",
        ComponentMinRef => "ST2067-21:2026:7.3.1/ComponentMinRef",
        Palette => "ST2067-21:2026:7.3.1/Palette",
        PaletteLayout => "ST2067-21:2026:7.3.1/PaletteLayout",
        ScanningDirection => "ST2067-21:2026:7.3.1/ScanningDirection",
        ComponentRefValues => "ST2067-21:2026:7.3.2/ComponentRefValues",
        AlphaSampleDepth => "ST2067-21:2026:7.4.1/AlphaSampleDepth",
        ColorSiting => "ST2067-21:2026:7.4.1/ColorSiting",
        ComponentDepth => "ST2067-21:2026:7.4.1/ComponentDepth",
        HorizontalSubsampling => "ST2067-21:2026:7.4.2/HorizontalSubsampling",
        PaddingBits => "ST2067-21:2026:7.4.1/PaddingBits",
        ReversedByteOrder => "ST2067-21:2026:7.4.1/ReversedByteOrder",
        VerticalSubsampling => "ST2067-21:2026:7.4.1/VerticalSubsampling",
        BlackRefLevel => "ST2067-21:2026:7.4.3/BlackRefLevel",
        ColorRange => "ST2067-21:2026:7.4.3/ColorRange",
        WhiteRefLevel => "ST2067-21:2026:7.4.3/WhiteRefLevel",
        CodingStyle => "ST2067-21:2026:7.5.1/CodingStyle",
        J2CLayout => "ST2067-21:2026:7.5.2/J2CLayout",
        J2KExtendedCapabilities => "ST2067-21:2026:7.5.1/J2KExtendedCapabilities",
        Jpeg2000SubDescriptor => "ST2067-21:2026:7.5.1/JPEG2000SubDescriptor",
        J2K4KResolution => "ST2067-21:2026:7.2.5/J2K-4K-Resolution",
        J2K2KResolution => "ST2067-21:2026:7.2.5/J2K-2K-Resolution",
        J2KBcpResolution => "ST2067-21:2026:7.2.5/J2K-BCP-Resolution",
        J2K8KResolution => "ST2067-21:2026:7.2.5/J2K-8K-Resolution",
        ApplicationIdentification => "ST2067-21:2026:8.1/ApplicationIdentification",
        AppIdMismatch => "ST2067-21:2026:8.1/AppIdMismatch",
        HomogeneousImageEssence => "ST2067-21:2026:8.2/HomogeneousImageEssence",
        SegmentDurationMultiple => "ST2067-21:2026:8.3.2/SegmentDurationMultiple",
        MaxCLLMaxFALL => "ST2067-21:2026:8.3.3/MaxCLLMaxFALL",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// https://github.com/SMPTE/st2067-21/issues/2: the item is required
    /// when HTJ2K is used, so the catalogue must not describe it as
    /// "not permitted".
    #[test]
    fn j2k_extended_capabilities_catalogue_text_matches_rule() {
        for desc in [
            St2067_21_2023::J2KExtendedCapabilities.description(),
            St2067_21_2026::J2KExtendedCapabilities.description(),
        ] {
            assert!(desc.contains("shall be present"), "{desc}");
            assert!(!desc.contains("not permitted"), "{desc}");
        }
        let example = App2ERule::J2KExtendedCapabilities.example().unwrap();
        assert!(!example.contains("not permitted"), "{example}");
    }
}
