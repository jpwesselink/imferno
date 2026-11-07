//! Rule table: SMPTE ST 2067-21 (Application #2E).
//!
//! Generated from the pre-C0 per-edition code enums; edit by hand from now on.

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::{absent, row, AnyRule, Rule};
use crate::diagnostics::{Category, Severity};

pub use crate::validation::App2EEdition;

type E = App2EEdition;

/// Color system designator is not in the permitted set.
pub static ColorSystem: Rule<E> = Rule {
    name: "ColorSystem",
    category: Category::Video,
    description: "Color system designator is not in the permitted set.",
    example: Some("ColorPrimaries + TransferCharacteristic + CodingEquations triple doesn't map to a permitted App2E ColorN system"),
    editions: &[
        row(E::V2014, "6.1", Severity::Error),
        row(E::V2016, "6.1", Severity::Error),
        row(E::V2020, "6.2", Severity::Error),
        row(E::V2023, "6.2", Severity::Error),
        row(E::V2026, "7.2", Severity::Error),
    ],
};

/// BlackRefLevel value is inconsistent with ComponentDepth.
pub static BlackRefLevel: Rule<E> = Rule {
    name: "BlackRefLevel",
    category: Category::Video,
    description: "BlackRefLevel value is inconsistent with ComponentDepth.",
    example: Some("<ComponentDepth>10</ComponentDepth><BlackRefLevel>16</BlackRefLevel>  <!-- expected 64 for 10-bit -->"),
    editions: &[
        row(E::V2014, "6.1.4", Severity::Error),
        row(E::V2016, "6.3", Severity::Error),
        row(E::V2020, "6.4.3", Severity::Error),
        row(E::V2023, "6.4.3", Severity::Error),
        row(E::V2026, "7.4.3", Severity::Error),
    ],
};

/// WhiteRefLevel value is inconsistent with ComponentDepth.
pub static WhiteRefLevel: Rule<E> = Rule {
    name: "WhiteRefLevel",
    category: Category::Video,
    description: "WhiteRefLevel value is inconsistent with ComponentDepth.",
    example: Some("<ComponentDepth>10</ComponentDepth><WhiteRefLevel>235</WhiteRefLevel>  <!-- expected 940 for 10-bit -->"),
    editions: &[
        row(E::V2014, "6.1.4", Severity::Error),
        row(E::V2016, "6.3", Severity::Error),
        row(E::V2020, "6.4.3", Severity::Error),
        row(E::V2023, "6.4.3", Severity::Error),
        row(E::V2026, "7.4.3", Severity::Error),
    ],
};

/// ColorRange value is not in the permitted set.
pub static ColorRange: Rule<E> = Rule {
    name: "ColorRange",
    category: Category::Video,
    description: "ColorRange value is not in the permitted set.",
    example: Some("<ComponentDepth>10</ComponentDepth><ColorRange>1019</ColorRange>  <!-- not in permitted set -->"),
    editions: &[
        row(E::V2014, "6.1.4", Severity::Error),
        row(E::V2016, "6.3", Severity::Error),
        row(E::V2020, "6.4.3", Severity::Error),
        row(E::V2023, "6.4.3", Severity::Error),
        row(E::V2026, "7.4.3", Severity::Error),
    ],
};

/// ApplicationIdentification is required for App2E compositions.
pub static ApplicationIdentification: Rule<E> = Rule {
    name: "ApplicationIdentification",
    category: Category::Metadata,
    description: "ApplicationIdentification is required for App2E compositions.",
    example: Some("<CompositionPlaylist>…</CompositionPlaylist>  <!-- ExtensionProperties has no <ApplicationIdentification> -->"),
    editions: &[
        row(E::V2014, "7", Severity::Error),
        row(E::V2016, "7.1", Severity::Error),
        row(E::V2020, "7.1", Severity::Error),
        row(E::V2023, "7.1", Severity::Error),
        row(E::V2026, "8.1", Severity::Error),
    ],
};

/// ApplicationIdentification in CPL ExtensionProperties does not include the expected App2E URI.
pub static AppIdMismatch: Rule<E> = Rule {
    name: "AppIdMismatch",
    category: Category::Metadata,
    description: "ApplicationIdentification in CPL ExtensionProperties does not include the expected App2E URI.",
    example: Some("<ApplicationIdentification>http://example.org/not-app2e</ApplicationIdentification>"),
    editions: &[
        row(E::V2014, "7", Severity::Warning),
        row(E::V2016, "7.1", Severity::Warning),
        row(E::V2020, "7.1", Severity::Warning),
        row(E::V2023, "7.1", Severity::Warning),
        row(E::V2026, "8.1", Severity::Warning),
    ],
};

/// Component max/min reference values are inconsistent with bit depth.
pub static ComponentRefValues: Rule<E> = Rule {
    name: "ComponentRefValues",
    category: Category::Video,
    description: "Component max/min reference values are inconsistent with bit depth.",
    example: Some("<ComponentDepth>10</ComponentDepth><ComponentMaxRef>4096</ComponentMaxRef>  <!-- 4096 > (2^10)-1 -->"),
    editions: &[
        absent(E::V2014),
        row(E::V2016, "6.2", Severity::Error),
        row(E::V2020, "6.3.2", Severity::Error),
        row(E::V2023, "6.3.2", Severity::Error),
        row(E::V2026, "7.3.2", Severity::Error),
    ],
};

/// MaxCLL / MaxFALL HDR metadata is absent; recommended for HDR content.
pub static MaxCLLMaxFALL: Rule<E> = Rule {
    name: "MaxCLLMaxFALL",
    category: Category::Video,
    description: "MaxCLL / MaxFALL HDR metadata is absent; recommended for HDR content.",
    example: Some("HDR content (PQ TransferCharacteristic) with no <MaxCLL> / <MaxFALL> in ExtensionProperties"),
    editions: &[
        absent(E::V2014),
        row(E::V2016, "7.2", Severity::Info),
        row(E::V2020, "7.5", Severity::Info),
        row(E::V2023, "7.5", Severity::Info),
        row(E::V2026, "8.3.3", Severity::Info),
    ],
};

/// Frame rate is not in the permitted set for App2E.
pub static FrameRate: Rule<E> = Rule {
    name: "FrameRate",
    category: Category::Video,
    description: "Frame rate is not in the permitted set for App2E.",
    example: Some("<SampleRate>48/1</SampleRate>  <!-- 48 fps is not in Table 3 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "5.2", Severity::Error),
        row(E::V2023, "5.2", Severity::Error),
        row(E::V2026, "6.2.1", Severity::Error),
    ],
};

/// Image resolution is not in the permitted set for App2E.
pub static Resolution: Rule<E> = Rule {
    name: "Resolution",
    category: Category::Video,
    description: "Image resolution is not in the permitted set for App2E.",
    example: Some("<StoredWidth>9000</StoredWidth><StoredHeight>7000</StoredHeight>  <!-- larger than every image-characteristics column -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "5.2", Severity::Error),
        row(E::V2023, "5.2", Severity::Error),
        row(E::V2026, "6.2.1", Severity::Error),
    ],
};

/// RGBA/CDCI descriptor is missing the required StoredWidth field.
pub static RequiredStoredWidth: Rule<E> = Rule {
    name: "RequiredStoredWidth",
    category: Category::Encoding,
    description: "RGBA/CDCI descriptor is missing the required StoredWidth field.",
    example: Some("<RGBADescriptor>…</RGBADescriptor>  <!-- no <StoredWidth> child -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2", Severity::Error),
        row(E::V2023, "6.2", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// RGBA/CDCI descriptor is missing the required StoredHeight field.
pub static RequiredStoredHeight: Rule<E> = Rule {
    name: "RequiredStoredHeight",
    category: Category::Encoding,
    description: "RGBA/CDCI descriptor is missing the required StoredHeight field.",
    example: Some("<RGBADescriptor>…</RGBADescriptor>  <!-- no <StoredHeight> child -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2", Severity::Error),
        row(E::V2023, "6.2", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// RGBA/CDCI descriptor is missing the required SampleRate field.
pub static RequiredSampleRate: Rule<E> = Rule {
    name: "RequiredSampleRate",
    category: Category::Encoding,
    description: "RGBA/CDCI descriptor is missing the required SampleRate field.",
    example: Some("<RGBADescriptor>…</RGBADescriptor>  <!-- no <SampleRate> child -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2", Severity::Error),
        row(E::V2023, "6.2", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// RGBA/CDCI descriptor is missing the required FrameLayout field.
pub static RequiredFrameLayout: Rule<E> = Rule {
    name: "RequiredFrameLayout",
    category: Category::Encoding,
    description: "RGBA/CDCI descriptor is missing the required FrameLayout field.",
    example: Some("<RGBADescriptor>…</RGBADescriptor>  <!-- no <FrameLayout> child -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2", Severity::Error),
        row(E::V2023, "6.2", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// RGBA/CDCI descriptor is missing the required ColorPrimaries field.
pub static RequiredColorPrimaries: Rule<E> = Rule {
    name: "RequiredColorPrimaries",
    category: Category::Encoding,
    description: "RGBA/CDCI descriptor is missing the required ColorPrimaries field.",
    example: Some("<RGBADescriptor>…</RGBADescriptor>  <!-- no <ColorPrimaries> UL child -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2", Severity::Error),
        row(E::V2023, "6.2", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// RGBA/CDCI descriptor is missing the required TransferCharacteristic field.
pub static RequiredTransferCharacteristic: Rule<E> = Rule {
    name: "RequiredTransferCharacteristic",
    category: Category::Encoding,
    description: "RGBA/CDCI descriptor is missing the required TransferCharacteristic field.",
    example: Some(
        "<RGBADescriptor>…</RGBADescriptor>  <!-- no <TransferCharacteristic> UL child -->",
    ),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2", Severity::Error),
        row(E::V2023, "6.2", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// RGBA/CDCI descriptor is missing the required PictureCompression field.
pub static RequiredPictureCompression: Rule<E> = Rule {
    name: "RequiredPictureCompression",
    category: Category::Encoding,
    description: "RGBA/CDCI descriptor is missing the required PictureCompression field.",
    example: Some("<RGBADescriptor>…</RGBADescriptor>  <!-- no <PictureCompression> UL child -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2", Severity::Error),
        row(E::V2023, "6.2", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// CDCI descriptor is missing the required ComponentDepth field.
pub static RequiredComponentDepth: Rule<E> = Rule {
    name: "RequiredComponentDepth",
    category: Category::Encoding,
    description: "CDCI descriptor is missing the required ComponentDepth field.",
    example: Some("<CDCIDescriptor>…</CDCIDescriptor>  <!-- no <ComponentDepth> child -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2", Severity::Error).unverified(),
        row(E::V2023, "6.2", Severity::Error).unverified(),
        row(E::V2026, "7.4.1", Severity::Error),
    ],
};

/// Alpha transparency mode is not permitted in App2E.
pub static AlphaTransparency: Rule<E> = Rule {
    name: "AlphaTransparency",
    category: Category::Video,
    description: "Alpha transparency mode is not permitted in App2E.",
    example: Some("<AlphaTransparency>true</AlphaTransparency>  <!-- App2E disallows alpha -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error),
        row(E::V2023, "6.2.1", Severity::Error),
        absent(E::V2026),
    ],
};

/// CodingEquations field is absent from the picture descriptor.
pub static CodingEquationsMissing: Rule<E> = Rule {
    name: "CodingEquationsMissing",
    category: Category::Video,
    description: "CodingEquations field is absent from the picture descriptor.",
    example: Some("<CDCIDescriptor>…</CDCIDescriptor>  <!-- no <CodingEquations> UL child -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error),
        row(E::V2023, "6.2.1", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// ColorPrimaries field is absent from the picture descriptor.
pub static ColorPrimariesMissing: Rule<E> = Rule {
    name: "ColorPrimariesMissing",
    category: Category::Video,
    description: "ColorPrimaries field is absent from the picture descriptor.",
    example: Some("<CDCIDescriptor>…</CDCIDescriptor>  <!-- no <ColorPrimaries> UL child -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error),
        row(E::V2023, "6.2.1", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// FieldDominance value is not permitted for the declared FrameLayout.
pub static FieldDominance: Rule<E> = Rule {
    name: "FieldDominance",
    category: Category::Video,
    description: "FieldDominance value is not permitted for the declared FrameLayout.",
    example: Some("<FieldDominance>2</FieldDominance> on <FrameLayout>FullFrame</FrameLayout>"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error),
        row(E::V2023, "6.2.1", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// FrameLayout value is not in the permitted set for App2E.
pub static FrameLayout: Rule<E> = Rule {
    name: "FrameLayout",
    category: Category::Video,
    description: "FrameLayout value is not in the permitted set for App2E.",
    example: Some("<FrameLayout>MixedFields</FrameLayout>  <!-- not in {FullFrame,SeparateFields,SingleField} -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error),
        row(E::V2023, "6.2.1", Severity::Error),
        row(E::V2026, "7.2.1.4", Severity::Error),
    ],
};

/// Image frame structure (progressive or interlaced) does not fit the closest permitted column of the edition's image-characteristics table.
pub static FrameLayoutInterlaced: Rule<E> = Rule {
    name: "FrameLayoutInterlaced",
    category: Category::Video,
    description: "Image frame structure (progressive or interlaced) does not fit the closest permitted column of the edition's image-characteristics table.",
    example: Some("<FrameLayout>SeparateFields</FrameLayout> with <SampleRate>24/1</SampleRate>  <!-- interlaced only at 25, 30 or 30000/1001 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error).unverified(),
        row(E::V2023, "6.2.1", Severity::Error).unverified(),
        row(E::V2026, "6.2.1", Severity::Error),
    ],
};

/// ImageAlignmentOffset must be zero.
pub static ImageAlignmentOffset: Rule<E> = Rule {
    name: "ImageAlignmentOffset",
    category: Category::Video,
    description: "ImageAlignmentOffset must be zero.",
    example: Some("<ImageAlignmentOffset>16</ImageAlignmentOffset>  <!-- must be 0 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error),
        row(E::V2023, "6.2.1", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// ImageEndOffset must be zero.
pub static ImageEndOffset: Rule<E> = Rule {
    name: "ImageEndOffset",
    category: Category::Video,
    description: "ImageEndOffset must be zero.",
    example: Some("<ImageEndOffset>32</ImageEndOffset>  <!-- must be 0 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error),
        row(E::V2023, "6.2.1", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// ImageStartOffset must be zero.
pub static ImageStartOffset: Rule<E> = Rule {
    name: "ImageStartOffset",
    category: Category::Video,
    description: "ImageStartOffset must be zero.",
    example: Some("<ImageStartOffset>16</ImageStartOffset>  <!-- must be 0 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error),
        row(E::V2023, "6.2.1", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// SampledHeight must equal StoredHeight.
pub static SampledHeight: Rule<E> = Rule {
    name: "SampledHeight",
    category: Category::Video,
    description: "SampledHeight must equal StoredHeight.",
    example: Some("<StoredHeight>1080</StoredHeight><SampledHeight>1056</SampledHeight>"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error),
        row(E::V2023, "6.2.1", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// SampledWidth must equal StoredWidth.
pub static SampledWidth: Rule<E> = Rule {
    name: "SampledWidth",
    category: Category::Video,
    description: "SampledWidth must equal StoredWidth.",
    example: Some("<StoredWidth>1920</StoredWidth><SampledWidth>1872</SampledWidth>"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error),
        row(E::V2023, "6.2.1", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// SampledXOffset must be zero.
pub static SampledXOffset: Rule<E> = Rule {
    name: "SampledXOffset",
    category: Category::Video,
    description: "SampledXOffset must be zero.",
    example: Some("<SampledXOffset>4</SampledXOffset>  <!-- must be 0 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error),
        row(E::V2023, "6.2.1", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// SampledYOffset must be zero.
pub static SampledYOffset: Rule<E> = Rule {
    name: "SampledYOffset",
    category: Category::Video,
    description: "SampledYOffset must be zero.",
    example: Some("<SampledYOffset>4</SampledYOffset>  <!-- must be 0 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error),
        row(E::V2023, "6.2.1", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// StoredF2Offset must be zero.
pub static StoredF2Offset: Rule<E> = Rule {
    name: "StoredF2Offset",
    category: Category::Video,
    description: "StoredF2Offset must be zero.",
    example: Some("<StoredF2Offset>1</StoredF2Offset>  <!-- must be 0 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error),
        row(E::V2023, "6.2.1", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// TransferCharacteristic field is absent from the picture descriptor.
pub static TransferCharacteristicMissing: Rule<E> = Rule {
    name: "TransferCharacteristicMissing",
    category: Category::Video,
    description: "TransferCharacteristic field is absent from the picture descriptor.",
    example: Some(
        "<CDCIDescriptor>…</CDCIDescriptor>  <!-- no <TransferCharacteristic> UL child -->",
    ),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.1", Severity::Error),
        row(E::V2023, "6.2.1", Severity::Error),
        row(E::V2026, "7.2.1.1", Severity::Error),
    ],
};

/// TransferCharacteristic UL is present but not a recognized value.
pub static TransferCharacteristicUnknown: Rule<E> = Rule {
    name: "TransferCharacteristicUnknown",
    category: Category::Video,
    description: "TransferCharacteristic UL is present but not a recognized value.",
    example: Some("<TransferCharacteristic>urn:smpte:ul:060e2b34.04010101.04010101.01010500</TransferCharacteristic>  <!-- UL not in §6.2.2 table -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.2", Severity::Error),
        row(E::V2023, "6.2.2", Severity::Error),
        row(E::V2026, "7.2.2", Severity::Error),
    ],
};

/// CodingEquations UL is present but not a recognized value.
pub static CodingEquationsUnknown: Rule<E> = Rule {
    name: "CodingEquationsUnknown",
    category: Category::Video,
    description: "CodingEquations UL is present but not a recognized value.",
    example: Some("<CodingEquations>urn:smpte:ul:060e2b34.04010101.04010101.02020500</CodingEquations>  <!-- UL not in §6.2.3 table -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.3", Severity::Error),
        row(E::V2023, "6.2.3", Severity::Error),
        row(E::V2026, "7.2.3", Severity::Error),
    ],
};

/// ColorPrimaries UL is present but not a recognized value.
pub static ColorPrimariesUnknown: Rule<E> = Rule {
    name: "ColorPrimariesUnknown",
    category: Category::Video,
    description: "ColorPrimaries UL is present but not a recognized value.",
    example: Some("<ColorPrimaries>urn:smpte:ul:060e2b34.04010101.04010101.03030500</ColorPrimaries>  <!-- UL not in §6.2.4 table -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.4", Severity::Error),
        row(E::V2023, "6.2.4", Severity::Error),
        row(E::V2026, "7.2.4", Severity::Error),
    ],
};

/// Video essence is not JPEG 2000 encoded as required by App2E.
pub static J2KRequired: Rule<E> = Rule {
    name: "J2KRequired",
    category: Category::Encoding,
    description: "Video essence is not JPEG 2000 encoded as required by App2E.",
    example: Some("<PictureCompression>urn:smpte:ul:060e2b34.04010101.04010202.01020500</PictureCompression>  <!-- not a JPEG-2000 UL -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.5", Severity::Error),
        row(E::V2023, "6.2.5", Severity::Error),
        row(E::V2026, "7.2.5", Severity::Error),
    ],
};

/// AlphaMaxRef value is not permitted.
pub static AlphaMaxRef: Rule<E> = Rule {
    name: "AlphaMaxRef",
    category: Category::Video,
    description: "AlphaMaxRef value is not permitted.",
    example: Some(
        "<AlphaMaxRef>1024</AlphaMaxRef>  <!-- outside permitted depth-derived range -->",
    ),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.3", Severity::Error),
        row(E::V2023, "6.3", Severity::Error),
        row(E::V2026, "7.3.1", Severity::Error),
    ],
};

/// AlphaMinRef value is not permitted.
pub static AlphaMinRef: Rule<E> = Rule {
    name: "AlphaMinRef",
    category: Category::Video,
    description: "AlphaMinRef value is not permitted.",
    example: Some("<AlphaMinRef>-1</AlphaMinRef>  <!-- outside permitted depth-derived range -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.3", Severity::Error),
        row(E::V2023, "6.3", Severity::Error),
        row(E::V2026, "7.3.1", Severity::Error),
    ],
};

/// ComponentMaxRef value is not in the permitted range.
pub static ComponentMaxRef: Rule<E> = Rule {
    name: "ComponentMaxRef",
    category: Category::Video,
    description: "ComponentMaxRef value is not in the permitted range.",
    example: Some("<ComponentMaxRef>9999</ComponentMaxRef>  <!-- beyond (2^ComponentDepth)-1 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.3", Severity::Error),
        row(E::V2023, "6.3", Severity::Error),
        row(E::V2026, "7.3.1", Severity::Error),
    ],
};

/// ComponentMinRef value is not in the permitted range.
pub static ComponentMinRef: Rule<E> = Rule {
    name: "ComponentMinRef",
    category: Category::Video,
    description: "ComponentMinRef value is not in the permitted range.",
    example: Some("<ComponentMinRef>-5</ComponentMinRef>  <!-- must be ≥ 0 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.3", Severity::Error),
        row(E::V2023, "6.3", Severity::Error),
        row(E::V2026, "7.3.1", Severity::Error),
    ],
};

/// Palette is present; palette images are not permitted in App2E.
pub static Palette: Rule<E> = Rule {
    name: "Palette",
    category: Category::Video,
    description: "Palette is present; palette images are not permitted in App2E.",
    example: Some("<Palette>…</Palette>  <!-- palette images forbidden in App2E -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.3", Severity::Error),
        row(E::V2023, "6.3", Severity::Error),
        row(E::V2026, "7.3.1", Severity::Error),
    ],
};

/// PaletteLayout is present; palette layout is not permitted in App2E.
pub static PaletteLayout: Rule<E> = Rule {
    name: "PaletteLayout",
    category: Category::Video,
    description: "PaletteLayout is present; palette layout is not permitted in App2E.",
    example: Some("<PaletteLayout>…</PaletteLayout>  <!-- forbidden in App2E -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.3", Severity::Error),
        row(E::V2023, "6.3", Severity::Error),
        row(E::V2026, "7.3.1", Severity::Error),
    ],
};

/// ScanningDirection value is not in the permitted set.
pub static ScanningDirection: Rule<E> = Rule {
    name: "ScanningDirection",
    category: Category::Video,
    description: "ScanningDirection value is not in the permitted set.",
    example: Some("<ScanningDirection>5</ScanningDirection>  <!-- not in permitted set -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.3", Severity::Error),
        row(E::V2023, "6.3", Severity::Error),
        row(E::V2026, "7.3.1", Severity::Error),
    ],
};

/// AlphaSampleDepth value is not permitted.
pub static AlphaSampleDepth: Rule<E> = Rule {
    name: "AlphaSampleDepth",
    category: Category::Video,
    description: "AlphaSampleDepth value is not permitted.",
    example: Some("<AlphaSampleDepth>7</AlphaSampleDepth>  <!-- not in {8,10,12,16} -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.4", Severity::Error),
        row(E::V2023, "6.4", Severity::Error),
        row(E::V2026, "7.4.1", Severity::Error),
    ],
};

/// ColorSiting value is not in the permitted set.
pub static ColorSiting: Rule<E> = Rule {
    name: "ColorSiting",
    category: Category::Video,
    description: "ColorSiting value is not in the permitted set.",
    example: Some("<ColorSiting>9</ColorSiting>  <!-- not in permitted set -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.4", Severity::Error),
        row(E::V2023, "6.4", Severity::Error),
        row(E::V2026, "7.4.1", Severity::Error),
    ],
};

/// ComponentDepth is missing, or the pixel bit depth is not permitted for the image characteristics.
pub static ComponentDepth: Rule<E> = Rule {
    name: "ComponentDepth",
    category: Category::Video,
    description: "ComponentDepth is missing, or the pixel bit depth is not permitted for the image characteristics.",
    example: Some("<ComponentDepth>9</ComponentDepth>  <!-- not in {8,10,12,16} -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.4", Severity::Error),
        row(E::V2023, "6.4", Severity::Error),
        row(E::V2026, "7.4.1", Severity::Error),
    ],
};

/// HorizontalSubsampling value is not in the permitted set.
pub static HorizontalSubsampling: Rule<E> = Rule {
    name: "HorizontalSubsampling",
    category: Category::Video,
    description: "HorizontalSubsampling value is not in the permitted set.",
    example: Some("<HorizontalSubsampling>3</HorizontalSubsampling>  <!-- not in {1,2,4} -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.4", Severity::Error),
        row(E::V2023, "6.4", Severity::Error),
        row(E::V2026, "7.4.2", Severity::Error),
    ],
};

/// PaddingBits must be zero.
pub static PaddingBits: Rule<E> = Rule {
    name: "PaddingBits",
    category: Category::Video,
    description: "PaddingBits must be zero.",
    example: Some("<PaddingBits>4</PaddingBits>  <!-- must be 0 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.4", Severity::Error),
        row(E::V2023, "6.4", Severity::Error),
        row(E::V2026, "7.4.1", Severity::Error),
    ],
};

/// ReversedByteOrder flag is set; byte reversal is not permitted.
pub static ReversedByteOrder: Rule<E> = Rule {
    name: "ReversedByteOrder",
    category: Category::Video,
    description: "ReversedByteOrder flag is set; byte reversal is not permitted.",
    example: Some("<ReversedByteOrder>true</ReversedByteOrder>  <!-- byte reversal forbidden -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.4", Severity::Error),
        row(E::V2023, "6.4", Severity::Error),
        row(E::V2026, "7.4.1", Severity::Error),
    ],
};

/// VerticalSubsampling value is not in the permitted set.
pub static VerticalSubsampling: Rule<E> = Rule {
    name: "VerticalSubsampling",
    category: Category::Video,
    description: "VerticalSubsampling value is not in the permitted set.",
    example: Some("<VerticalSubsampling>3</VerticalSubsampling>  <!-- not in {1,2} -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.4", Severity::Error),
        row(E::V2023, "6.4", Severity::Error),
        row(E::V2026, "7.4.1", Severity::Error),
    ],
};

/// JPEG 2000 codestream coding style is not compliant.
pub static CodingStyle: Rule<E> = Rule {
    name: "CodingStyle",
    category: Category::Encoding,
    description: "JPEG 2000 codestream coding style is not compliant.",
    example: Some(
        "JPEG2000SubDescriptor CodingStyleDefault Scod byte declares non-compliant entropy coding",
    ),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.5.2", Severity::Error).unverified(),
        row(E::V2023, "6.5.2", Severity::Error).unverified(),
        row(E::V2026, "7.5.1", Severity::Error),
    ],
};

/// JPEG 2000 codestream layout does not meet App2E requirements.
pub static J2CLayout: Rule<E> = Rule {
    name: "J2CLayout",
    category: Category::Encoding,
    description: "JPEG 2000 codestream layout does not meet App2E requirements.",
    example: Some("JPEG2000SubDescriptor declares >1 layer or unsupported precinct sizes"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.5.2", Severity::Error).unverified(),
        row(E::V2023, "6.5.2", Severity::Error).unverified(),
        row(E::V2026, "7.5.2", Severity::Error),
    ],
};

/// JPEG2000SubDescriptor is absent or incomplete.
pub static Jpeg2000SubDescriptor: Rule<E> = Rule {
    name: "Jpeg2000SubDescriptor",
    category: Category::Encoding,
    description: "JPEG2000SubDescriptor is absent or incomplete.",
    example: Some(
        "<PictureDescriptor>…</PictureDescriptor>  <!-- no JPEG2000SubDescriptor child -->",
    ),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.5.2", Severity::Warning).unverified(),
        row(E::V2023, "6.5.2", Severity::Warning).unverified(),
        row(E::V2026, "7.5.1", Severity::Warning),
    ],
};

/// JPEG 2000 HT (ISO 15444-15) is not permitted by App2E 2020.
pub static J2KHtNotAllowed: Rule<E> = Rule {
    name: "J2KHtNotAllowed",
    category: Category::Encoding,
    description: "JPEG 2000 HT (ISO 15444-15) is not permitted by App2E 2020.",
    example: Some("JPEG2000SubDescriptor Profile field declares HTJ2K (ISO 15444-15)"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.5", Severity::Error),
        absent(E::V2023),
        absent(E::V2026),
    ],
};

/// JPEG 2000 IMF 4K Profile: stored resolution is outside the permitted range.
pub static J2K4KResolution: Rule<E> = Rule {
    name: "J2K4KResolution",
    category: Category::Encoding,
    description: "JPEG 2000 IMF 4K Profile: stored resolution is outside the permitted range.",
    example: Some("<StoredWidth>5000</StoredWidth>  <!-- IMF 4K profile caps width at 4096 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.5", Severity::Error),
        row(E::V2023, "6.2.5", Severity::Error),
        row(E::V2026, "7.2.5", Severity::Error),
    ],
};

/// JPEG 2000 IMF 2K Profile: stored resolution is outside the permitted range.
pub static J2K2KResolution: Rule<E> = Rule {
    name: "J2K2KResolution",
    category: Category::Encoding,
    description: "JPEG 2000 IMF 2K Profile: stored resolution is outside the permitted range.",
    example: Some("<StoredWidth>2400</StoredWidth>  <!-- IMF 2K profile caps width at 2048 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.5", Severity::Error),
        row(E::V2023, "6.2.5", Severity::Error),
        row(E::V2026, "7.2.5", Severity::Error),
    ],
};

/// JPEG 2000 Broadcast Contribution Profile: stored resolution is outside the permitted range.
pub static J2KBcpResolution: Rule<E> = Rule {
    name: "J2KBcpResolution",
    category: Category::Encoding,
    description: "JPEG 2000 Broadcast Contribution Profile: stored resolution is outside the permitted range.",
    example: Some("<StoredWidth>4096</StoredWidth> with a Broadcast Contribution profile  <!-- profile column is 1..3840 x 1..2160 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.5", Severity::Error),
        row(E::V2023, "6.2.5", Severity::Error),
        row(E::V2026, "7.2.5", Severity::Error),
    ],
};

/// The JPEG 2000 profile is not listed in the edition's JPEG 2000 profile table.
pub static J2KProfileNotAllowed: Rule<E> = Rule {
    name: "J2KProfileNotAllowed",
    category: Category::Encoding,
    description: "The JPEG 2000 profile is not listed in the edition's JPEG 2000 profile table.",
    example: Some("<PictureCompression>urn:smpte:ul:060e2b34.0401010d.04010202.03010403</PictureCompression>  <!-- 8K IMF profile, not in the 2023 table -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "6.2.5", Severity::Error),
        row(E::V2023, "6.2.5", Severity::Error),
        absent(E::V2026),
    ],
};

/// All image essence in a composition shall use the same color system.
pub static HomogeneousImageEssence: Rule<E> = Rule {
    name: "HomogeneousImageEssence",
    category: Category::Video,
    description: "All image essence in a composition shall use the same color system.",
    example: Some(
        "Composition mixes Color1 (BT.709) and Color3 (BT.2020) resources across segments",
    ),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "7.2", Severity::Error),
        row(E::V2023, "7.2", Severity::Error),
        row(E::V2026, "8.2", Severity::Error),
    ],
};

/// Segment duration must be an integer multiple of 5 edit units.
pub static SegmentDurationMultiple: Rule<E> = Rule {
    name: "SegmentDurationMultiple",
    category: Category::Timing,
    description: "Segment duration must be an integer multiple of 5 edit units.",
    example: Some(
        "<Resource><SourceDuration>7</SourceDuration></Resource>  <!-- not a multiple of 5 -->",
    ),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        row(E::V2020, "7.4", Severity::Error),
        row(E::V2023, "7.4", Severity::Error),
        row(E::V2026, "8.3.2", Severity::Error),
    ],
};

/// J2KExtendedCapabilities shall be present when ISO/IEC 15444-15 (HTJ2K) coding is used, but it is missing.
pub static J2KExtendedCapabilities: Rule<E> = Rule {
    name: "J2KExtendedCapabilities",
    category: Category::Encoding,
    description: "J2KExtendedCapabilities shall be present when ISO/IEC 15444-15 (HTJ2K) coding is used, but it is missing.",
    example: Some("<JPEG2000SubDescriptor><Rsiz>16384</Rsiz>…</JPEG2000SubDescriptor>  <!-- HTJ2K (Rsiz bit 14) but no <J2KExtendedCapabilities> -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        absent(E::V2020),
        row(E::V2023, "6.5.2", Severity::Error).unverified(),
        row(E::V2026, "7.5.1", Severity::Error),
    ],
};

/// HTJ2K irreversible (APP2.HT.IRV) codestream: the MAGB_P set signalled in Ccap15 allows parameter B above 31. APP2.HT.REV is not checked: Table H.1 gives its B as a value calculated per Table H.2, not a limit (SMPTE st2067-21 issue 7, open for REV).
pub static HtIrvParameterBMax: Rule<E> = Rule {
    name: "HtIrvParameterBMax",
    category: Category::Encoding,
    description: "HTJ2K irreversible (APP2.HT.IRV) codestream: the MAGB_P set signalled in Ccap15 allows parameter B above 31. APP2.HT.REV is not checked: Table H.1 gives its B as a value calculated per Table H.2, not a limit (SMPTE st2067-21 issue 7, open for REV).",
    example: Some("<J2KExtendedCapabilities><Pcap>131072</Pcap><Ccapi><UInt16>53</UInt16></Ccapi></J2KExtendedCapabilities>  <!-- HTIRV, MAGB_P 21: B = 35 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        absent(E::V2020),
        absent(E::V2023),
        row(E::V2026, "H", Severity::Error),
    ],
};

/// HTJ2K irreversible (APP2.HT.IRV) codestream: the MAGB_P set signalled in Ccap15 allows parameter B above the recommended value for its bit depth and multiple component transform.
pub static HtIrvParameterBRecommended: Rule<E> = Rule {
    name: "HtIrvParameterBRecommended",
    category: Category::Encoding,
    description: "HTJ2K irreversible (APP2.HT.IRV) codestream: the MAGB_P set signalled in Ccap15 allows parameter B above the recommended value for its bit depth and multiple component transform.",
    example: Some("<Ssiz>9</Ssiz> … <Ccapi><UInt16>36</UInt16></Ccapi>  <!-- 10-bit HTIRV, MAGB_P 4: B = 12, table value 11 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        absent(E::V2020),
        absent(E::V2023),
        row(E::V2026, "H", Severity::Warning),
    ],
};

/// JPEG 2000 IMF 8K Profile: stored resolution is outside the permitted range.
pub static J2K8KResolution: Rule<E> = Rule {
    name: "J2K8KResolution",
    category: Category::Encoding,
    description: "JPEG 2000 IMF 8K Profile: stored resolution is outside the permitted range.",
    example: Some("<StoredWidth>3840</StoredWidth> with an 8K IMF profile  <!-- profile column is 4097..8192 x 1..6224 -->"),
    editions: &[
        absent(E::V2014),
        absent(E::V2016),
        absent(E::V2020),
        absent(E::V2023),
        row(E::V2026, "7.2.5", Severity::Error),
    ],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[
    &ColorSystem,
    &BlackRefLevel,
    &WhiteRefLevel,
    &ColorRange,
    &ApplicationIdentification,
    &AppIdMismatch,
    &ComponentRefValues,
    &MaxCLLMaxFALL,
    &FrameRate,
    &Resolution,
    &RequiredStoredWidth,
    &RequiredStoredHeight,
    &RequiredSampleRate,
    &RequiredFrameLayout,
    &RequiredColorPrimaries,
    &RequiredTransferCharacteristic,
    &RequiredPictureCompression,
    &RequiredComponentDepth,
    &AlphaTransparency,
    &CodingEquationsMissing,
    &ColorPrimariesMissing,
    &FieldDominance,
    &FrameLayout,
    &FrameLayoutInterlaced,
    &ImageAlignmentOffset,
    &ImageEndOffset,
    &ImageStartOffset,
    &SampledHeight,
    &SampledWidth,
    &SampledXOffset,
    &SampledYOffset,
    &StoredF2Offset,
    &TransferCharacteristicMissing,
    &TransferCharacteristicUnknown,
    &CodingEquationsUnknown,
    &ColorPrimariesUnknown,
    &J2KRequired,
    &AlphaMaxRef,
    &AlphaMinRef,
    &ComponentMaxRef,
    &ComponentMinRef,
    &Palette,
    &PaletteLayout,
    &ScanningDirection,
    &AlphaSampleDepth,
    &ColorSiting,
    &ComponentDepth,
    &HorizontalSubsampling,
    &PaddingBits,
    &ReversedByteOrder,
    &VerticalSubsampling,
    &CodingStyle,
    &J2CLayout,
    &Jpeg2000SubDescriptor,
    &J2KHtNotAllowed,
    &J2K4KResolution,
    &J2K2KResolution,
    &J2KBcpResolution,
    &J2KProfileNotAllowed,
    &HomogeneousImageEssence,
    &SegmentDurationMultiple,
    &J2KExtendedCapabilities,
    &HtIrvParameterBMax,
    &HtIrvParameterBRecommended,
    &J2K8KResolution,
];
