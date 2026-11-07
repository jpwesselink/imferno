//! Application Profile #2E — SMPTE ST 2067-21, one validator per edition.
//!
//! Every edition is validated against its own text only:
//!
//! | Edition | ApplicationIdentification (CPL)            | Validator    |
//! |---------|--------------------------------------------|--------------|
//! | 2014    | `http://www.smpte-ra.org/schemas/2067-21/2014` (§7 Table 5)   | [`App2E2014`] |
//! | 2016    | `http://www.smpte-ra.org/schemas/2067-21/2016` (§7.1 Table 8) | [`App2E2016`] |
//! | 2020 + Am1:2020 | `http://www.smpte-ra.org/ns/2067-21/2020` (§7.1 Table 15) | [`App2E2020`] |
//! | 2023    | `http://www.smpte-ra.org/ns/2067-21/2021` (§7.1 Table 15)     | [`App2E2023`] |
//! | 2026    | `http://www.smpte-ra.org/ns/2067-21/5ED` (§8.1 Figure 2)      | [`App2E2026`] |
//!
//! Findings are reported per [`App2ERule`]; each edition's code enum maps
//! only the rules its text contains. A finding whose rule has no code in the
//! validated edition is dropped, so a validator never emits another
//! edition's code.
//!
//! Table numbers in comments below refer to the 2020/2023 editions (which
//! share their numbering) unless stated otherwise; the 2026 edition numbers
//! the same tables one lower from Table 3 on (e.g. Table 8 → Table 7).

use std::collections::HashSet;

use super::codes::{
    App2ERule, St2067_21_2014, St2067_21_2016, St2067_21_2020, St2067_21_2023, St2067_21_2026,
};
use super::{
    cdci_ref_values, component_ref_values, ColorSystem, ConstraintsValidator, QuantizationSystem,
};
use crate::cpl::CompositionPlaylist;
use crate::cpl::{CodingEquations, ColorPrimaries, EditRate, TransferCharacteristic};
use crate::diagnostics::codes::ValidationCode;
use crate::diagnostics::{Category, Location, Severity, ValidationIssue};

// ═════════════════════════════════════════════════════════════════════════════
// Editions
// ═════════════════════════════════════════════════════════════════════════════

/// A published edition of SMPTE ST 2067-21.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum App2EEdition {
    /// ST 2067-21:2014 (extends ST 2067-20).
    V2014,
    /// ST 2067-21:2016 (extends ST 2067-20).
    V2016,
    /// ST 2067-21:2020 including Amendment 1:2020 (COLOR.8 / HLG).
    V2020,
    /// ST 2067-21:2023.
    V2023,
    /// ST 2067-21:2026.
    V2026,
}

impl App2EEdition {
    /// Every edition, oldest first.
    pub const ALL: &'static [Self] = &[
        Self::V2014,
        Self::V2016,
        Self::V2020,
        Self::V2023,
        Self::V2026,
    ];

    /// Human-readable edition name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::V2014 => "ST 2067-21:2014",
            Self::V2016 => "ST 2067-21:2016",
            Self::V2020 => "ST 2067-21:2020 (incl. Am1:2020)",
            Self::V2023 => "ST 2067-21:2023",
            Self::V2026 => "ST 2067-21:2026",
        }
    }

    /// The value the edition requires ApplicationIdentification to include:
    /// 2014 §7 Table 5, 2016 §7.1 Table 8, 2020 and 2023 §7.1 Table 15,
    /// 2026 §8.1 Figure 2. Amendment 1:2020 defines no value of its own.
    pub fn application_identification_uri(&self) -> &'static str {
        match self {
            Self::V2014 => "http://www.smpte-ra.org/schemas/2067-21/2014",
            Self::V2016 => "http://www.smpte-ra.org/schemas/2067-21/2016",
            Self::V2020 => "http://www.smpte-ra.org/ns/2067-21/2020",
            Self::V2023 => "http://www.smpte-ra.org/ns/2067-21/2021",
            Self::V2026 => "http://www.smpte-ra.org/ns/2067-21/5ED",
        }
    }

    /// The edition whose ApplicationIdentification value is `uri`.
    pub fn from_application_identification(uri: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|e| e.application_identification_uri() == uri)
    }

    /// This edition's code for `rule`, or `None` when the edition's text
    /// does not contain the rule.
    pub fn code(&self, rule: App2ERule) -> Option<&'static str> {
        match self {
            Self::V2014 => St2067_21_2014::for_rule(rule).map(|c| c.code()),
            Self::V2016 => St2067_21_2016::for_rule(rule).map(|c| c.code()),
            Self::V2020 => St2067_21_2020::for_rule(rule).map(|c| c.code()),
            Self::V2023 => St2067_21_2023::for_rule(rule).map(|c| c.code()),
            Self::V2026 => St2067_21_2026::for_rule(rule).map(|c| c.code()),
        }
    }

    /// HT-J2K (ISO/IEC 15444-15) is listed in the JPEG 2000 table from the
    /// 2023 edition on (2023 §5.5.2 Table 6, 2026 §6.5.2 Table 5); the 2020
    /// Table 6 has no HT column.
    fn allows_ht(&self) -> bool {
        matches!(self, Self::V2023 | Self::V2026)
    }

    /// The 8K IMF profiles are listed only in the 2026 JPEG 2000 table
    /// (§6.5.2 Table 5, 4097..8192 x 1..6224).
    fn allows_8k(&self) -> bool {
        matches!(self, Self::V2026)
    }

    /// The image-characteristics table each image frame shall fit one column
    /// of ("Each image frame shall conform to one of the permitted
    /// combinations"): 2020 and 2023 §5.2 Table 3, 2026 §6.2.1 Table 2.
    ///
    /// The 2014 (§5.1 Table 2) and 2016 (§5.1 Table 3) tables only state
    /// what implementations "shall support" on top of ST 2067-20, whose own
    /// combinations also remain permitted; they do not close the set, so no
    /// column match is made for those editions.
    fn image_characteristics(&self) -> Option<(&'static str, &'static [ImageColumn])> {
        match self {
            Self::V2014 | Self::V2016 => None,
            Self::V2020 => Some(("§5.2 Table 3", TABLE_3_2020_AM1)),
            Self::V2023 => Some(("§5.2 Table 3", TABLE_3_2023)),
            Self::V2026 => Some(("§6.2.1 Table 2", TABLE_2_2026)),
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Image characteristics tables
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameStructure {
    Progressive,
    Interlaced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sampling {
    S422,
    S444,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorComponents {
    YCbCr,
    Rgb,
}

/// One column of an image-characteristics table: a combination of
/// characteristics an image frame may have. Ranges are "1..max" (the "a..b"
/// notation includes both ends).
struct ImageColumn {
    label: &'static str,
    max_width: u32,
    max_height: u32,
    colorimetry: &'static [ColorSystem],
    bit_depths: &'static [u32],
    structure: FrameStructure,
    frame_rates: &'static [(u32, u32)],
    sampling: Sampling,
    components: &'static [ColorComponents],
    quantization: &'static [QuantizationSystem],
}

const RATES_INTERLACED: &[(u32, u32)] = &[(25, 1), (30, 1), (30000, 1001)];
const RATES_PROGRESSIVE: &[(u32, u32)] = &[
    (24, 1),
    (24000, 1001),
    (25, 1),
    (30, 1),
    (30000, 1001),
    (50, 1),
    (60, 1),
    (60000, 1001),
];
const RATES_PROGRESSIVE_120: &[(u32, u32)] = &[
    (24, 1),
    (24000, 1001),
    (25, 1),
    (30, 1),
    (30000, 1001),
    (50, 1),
    (60, 1),
    (60000, 1001),
    (120, 1),
];

const COLOR_1_2_3: &[ColorSystem] = &[
    ColorSystem::Color1,
    ColorSystem::Color2,
    ColorSystem::Color3,
];
const DEPTH_8_10: &[u32] = &[8, 10];
const DEPTH_10_12: &[u32] = &[10, 12];
const DEPTH_10_12_16: &[u32] = &[10, 12, 16];
const DEPTH_8_TO_16: &[u32] = &[8, 10, 12, 16];
const YCBCR: &[ColorComponents] = &[ColorComponents::YCbCr];
const RGB: &[ColorComponents] = &[ColorComponents::Rgb];
const YCBCR_OR_RGB: &[ColorComponents] = &[ColorComponents::YCbCr, ColorComponents::Rgb];
const QE1: &[QuantizationSystem] = &[QuantizationSystem::Qe1];
const QE1_QE2: &[QuantizationSystem] = &[QuantizationSystem::Qe1, QuantizationSystem::Qe2];

/// Builds the columns shared by the 2020 (with Am1) and 2023 tables.
macro_rules! table_3_columns {
    ($color5:expr) => {
        &[
            ImageColumn {
                label: "1..1920 x 1..1080 interlaced 4:2:2 Y'C'BC'R COLOR.1/2/3",
                max_width: 1920,
                max_height: 1080,
                colorimetry: COLOR_1_2_3,
                bit_depths: DEPTH_8_10,
                structure: FrameStructure::Interlaced,
                frame_rates: RATES_INTERLACED,
                sampling: Sampling::S422,
                components: YCBCR,
                quantization: QE1,
            },
            ImageColumn {
                label: "1..1920 x 1..1080 progressive 4:4:4 COLOR.1/2/3",
                max_width: 1920,
                max_height: 1080,
                colorimetry: COLOR_1_2_3,
                bit_depths: DEPTH_8_10,
                structure: FrameStructure::Progressive,
                frame_rates: RATES_PROGRESSIVE,
                sampling: Sampling::S444,
                components: YCBCR_OR_RGB,
                quantization: QE1,
            },
            ImageColumn {
                label: "1..3840 x 1..2160 progressive 4:2:2 Y'C'BC'R COLOR.4",
                max_width: 3840,
                max_height: 2160,
                colorimetry: &[ColorSystem::Color4],
                bit_depths: DEPTH_8_10,
                structure: FrameStructure::Progressive,
                frame_rates: RATES_PROGRESSIVE,
                sampling: Sampling::S422,
                components: YCBCR,
                quantization: QE1,
            },
            ImageColumn {
                label: "1..3840 x 1..2160 progressive 4:2:2 Y'C'BC'R COLOR.3",
                max_width: 3840,
                max_height: 2160,
                colorimetry: &[ColorSystem::Color3],
                bit_depths: DEPTH_8_TO_16,
                structure: FrameStructure::Progressive,
                frame_rates: RATES_PROGRESSIVE,
                sampling: Sampling::S422,
                components: YCBCR,
                quantization: QE1,
            },
            ImageColumn {
                label: "1..3840 x 1..2160 progressive 4:2:2 Y'C'BC'R COLOR.5/8",
                max_width: 3840,
                max_height: 2160,
                colorimetry: $color5,
                bit_depths: DEPTH_10_12,
                structure: FrameStructure::Progressive,
                frame_rates: RATES_PROGRESSIVE,
                sampling: Sampling::S422,
                components: YCBCR,
                quantization: QE1,
            },
            ImageColumn {
                label: "1..3840 x 1..2160 progressive 4:2:2 Y'C'BC'R COLOR.7",
                max_width: 3840,
                max_height: 2160,
                colorimetry: &[ColorSystem::Color7],
                bit_depths: DEPTH_10_12_16,
                structure: FrameStructure::Progressive,
                frame_rates: RATES_PROGRESSIVE,
                sampling: Sampling::S422,
                components: YCBCR,
                quantization: QE1,
            },
            ImageColumn {
                label: "1..4096 x 1..3112 progressive 4:4:4 R'G'B' COLOR.3",
                max_width: 4096,
                max_height: 3112,
                colorimetry: &[ColorSystem::Color3],
                bit_depths: DEPTH_8_TO_16,
                structure: FrameStructure::Progressive,
                frame_rates: RATES_PROGRESSIVE_120,
                sampling: Sampling::S444,
                components: RGB,
                quantization: QE1_QE2,
            },
            ImageColumn {
                label: "1..4096 x 1..3112 progressive 4:4:4 R'G'B' COLOR.5/8",
                max_width: 4096,
                max_height: 3112,
                colorimetry: $color5,
                bit_depths: DEPTH_10_12,
                structure: FrameStructure::Progressive,
                frame_rates: RATES_PROGRESSIVE_120,
                sampling: Sampling::S444,
                components: RGB,
                quantization: QE1_QE2,
            },
            ImageColumn {
                label: "1..4096 x 1..3112 progressive 4:4:4 R'G'B' COLOR.6/7",
                max_width: 4096,
                max_height: 3112,
                colorimetry: &[ColorSystem::Color6, ColorSystem::Color7],
                bit_depths: DEPTH_10_12_16,
                structure: FrameStructure::Progressive,
                frame_rates: RATES_PROGRESSIVE_120,
                sampling: Sampling::S444,
                components: RGB,
                quantization: QE1_QE2,
            },
        ]
    };
}

/// ST 2067-21:2020 §5.2 Table 3 (p6) as amended by Amendment 1:2020 §3.2
/// (p3–4), which adds COLOR.8 to the 10/12-bit COLOR.5 columns of the
/// 1..3840 and 1..4096 groups. Pixel bit depth of the COLOR.4 column is
/// 8/10: the 2020 table's 8/10 cell spans the 1..1920 columns and the
/// COLOR.4 column, and the Am1 table lists COLOR.4 with 8/10.
const TABLE_3_2020_AM1: &[ImageColumn] =
    table_3_columns!(&[ColorSystem::Color5, ColorSystem::Color8]);

/// ST 2067-21:2023 §5.2 Table 3 (p7): the 2020 table with Am1 consolidated.
const TABLE_3_2023: &[ImageColumn] = table_3_columns!(&[ColorSystem::Color5, ColorSystem::Color8]);

/// ST 2067-21:2026 §6.2.1 Table 2, read from the HTML edition's column
/// spans (10 columns): the 1..1920 group gains a progressive 4:2:2
/// Y'C'BC'R column, COLOR.4 stays at 1..3840 x 1..2160, the other 4:2:2
/// Y'C'BC'R columns go to 1..7680 x 1..4320 and the R'G'B' columns to
/// 1..8192 x 1..6224.
const TABLE_2_2026: &[ImageColumn] = &[
    ImageColumn {
        label: "1..1920 x 1..1080 interlaced 4:2:2 Y'C'BC'R COLOR.1/2/3",
        max_width: 1920,
        max_height: 1080,
        colorimetry: COLOR_1_2_3,
        bit_depths: DEPTH_8_10,
        structure: FrameStructure::Interlaced,
        frame_rates: RATES_INTERLACED,
        sampling: Sampling::S422,
        components: YCBCR,
        quantization: QE1,
    },
    ImageColumn {
        label: "1..1920 x 1..1080 progressive 4:2:2 Y'C'BC'R COLOR.1/2/3",
        max_width: 1920,
        max_height: 1080,
        colorimetry: COLOR_1_2_3,
        bit_depths: DEPTH_8_10,
        structure: FrameStructure::Progressive,
        frame_rates: RATES_PROGRESSIVE,
        sampling: Sampling::S422,
        components: YCBCR,
        quantization: QE1,
    },
    ImageColumn {
        label: "1..1920 x 1..1080 progressive 4:4:4 COLOR.1/2/3",
        max_width: 1920,
        max_height: 1080,
        colorimetry: COLOR_1_2_3,
        bit_depths: DEPTH_8_10,
        structure: FrameStructure::Progressive,
        frame_rates: RATES_PROGRESSIVE,
        sampling: Sampling::S444,
        components: YCBCR_OR_RGB,
        quantization: QE1,
    },
    ImageColumn {
        label: "1..3840 x 1..2160 progressive 4:2:2 Y'C'BC'R COLOR.4",
        max_width: 3840,
        max_height: 2160,
        colorimetry: &[ColorSystem::Color4],
        bit_depths: DEPTH_8_10,
        structure: FrameStructure::Progressive,
        frame_rates: RATES_PROGRESSIVE,
        sampling: Sampling::S422,
        components: YCBCR,
        quantization: QE1,
    },
    ImageColumn {
        label: "1..7680 x 1..4320 progressive 4:2:2 Y'C'BC'R COLOR.3",
        max_width: 7680,
        max_height: 4320,
        colorimetry: &[ColorSystem::Color3],
        bit_depths: DEPTH_8_TO_16,
        structure: FrameStructure::Progressive,
        frame_rates: RATES_PROGRESSIVE,
        sampling: Sampling::S422,
        components: YCBCR,
        quantization: QE1,
    },
    ImageColumn {
        label: "1..7680 x 1..4320 progressive 4:2:2 Y'C'BC'R COLOR.5/8",
        max_width: 7680,
        max_height: 4320,
        colorimetry: &[ColorSystem::Color5, ColorSystem::Color8],
        bit_depths: DEPTH_10_12,
        structure: FrameStructure::Progressive,
        frame_rates: RATES_PROGRESSIVE,
        sampling: Sampling::S422,
        components: YCBCR,
        quantization: QE1,
    },
    ImageColumn {
        label: "1..7680 x 1..4320 progressive 4:2:2 Y'C'BC'R COLOR.7",
        max_width: 7680,
        max_height: 4320,
        colorimetry: &[ColorSystem::Color7],
        bit_depths: DEPTH_10_12_16,
        structure: FrameStructure::Progressive,
        frame_rates: RATES_PROGRESSIVE,
        sampling: Sampling::S422,
        components: YCBCR,
        quantization: QE1,
    },
    ImageColumn {
        label: "1..8192 x 1..6224 progressive 4:4:4 R'G'B' COLOR.3",
        max_width: 8192,
        max_height: 6224,
        colorimetry: &[ColorSystem::Color3],
        bit_depths: DEPTH_8_TO_16,
        structure: FrameStructure::Progressive,
        frame_rates: RATES_PROGRESSIVE_120,
        sampling: Sampling::S444,
        components: RGB,
        quantization: QE1_QE2,
    },
    ImageColumn {
        label: "1..8192 x 1..6224 progressive 4:4:4 R'G'B' COLOR.5/8",
        max_width: 8192,
        max_height: 6224,
        colorimetry: &[ColorSystem::Color5, ColorSystem::Color8],
        bit_depths: DEPTH_10_12,
        structure: FrameStructure::Progressive,
        frame_rates: RATES_PROGRESSIVE_120,
        sampling: Sampling::S444,
        components: RGB,
        quantization: QE1_QE2,
    },
    ImageColumn {
        label: "1..8192 x 1..6224 progressive 4:4:4 R'G'B' COLOR.6/7",
        max_width: 8192,
        max_height: 6224,
        colorimetry: &[ColorSystem::Color6, ColorSystem::Color7],
        bit_depths: DEPTH_10_12_16,
        structure: FrameStructure::Progressive,
        frame_rates: RATES_PROGRESSIVE_120,
        sampling: Sampling::S444,
        components: RGB,
        quantization: QE1_QE2,
    },
];

/// The characteristics of one image descriptor, as far as the descriptor
/// model carries them. `None` means the characteristic cannot be determined
/// (missing or unrecognised item); it then matches any column, and the item
/// itself is reported by the descriptor checks. Stereoscopy is not part of
/// the descriptor model and is not compared.
struct ImageFrame {
    width: Option<u32>,
    height: Option<u32>,
    colorimetry: Option<ColorSystem>,
    bit_depth: Option<u32>,
    structure: Option<FrameStructure>,
    frame_rate: Option<(u32, u32)>,
    sampling: Option<Sampling>,
    components: ColorComponents,
    quantization: Option<QuantizationSystem>,
}

/// A characteristic of an image frame that differs from a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Characteristic {
    Size,
    Colorimetry,
    BitDepth,
    Structure,
    FrameRate,
    Sampling,
    Components,
    Quantization,
}

fn reduce_rate(rate: &EditRate) -> Option<(u32, u32)> {
    fn gcd(a: u32, b: u32) -> u32 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }
    if rate.numerator == 0 || rate.denominator == 0 {
        return None;
    }
    let g = gcd(rate.numerator, rate.denominator);
    Some((rate.numerator / g, rate.denominator / g))
}

/// Pixel bit depth and quantization system of an R'G'B' descriptor, from
/// its Component Max Ref / Min Ref (2020/2023 §6.3.2 Table 11, 2026 §7.3.2
/// Table 10). `None` when the values are not in the table.
fn rgba_depth_and_quantization(
    min_ref: Option<u32>,
    max_ref: Option<u32>,
) -> (Option<u32>, Option<QuantizationSystem>) {
    let (Some(min_ref), Some(max_ref)) = (min_ref, max_ref) else {
        return (None, None);
    };
    for qe in [QuantizationSystem::Qe1, QuantizationSystem::Qe2] {
        for depth in [8, 10, 12, 16] {
            if component_ref_values(qe, depth) == Some((min_ref, max_ref)) {
                return (Some(depth), Some(qe));
            }
        }
    }
    (None, None)
}

/// ST 2067-21 §6.2.1 Table 9 (2026 §7.2.1.2 Table 8): image frame height
/// from StoredHeight. For interlaced (SeparateFields) content StoredHeight
/// is one field, i.e. half the frame height.
fn image_frame_height(stored_height: u32, frame_layout: Option<&str>) -> u32 {
    if frame_layout == Some("SeparateFields") {
        stored_height.saturating_mul(2)
    } else {
        stored_height
    }
}

impl ImageColumn {
    fn mismatches(&self, f: &ImageFrame) -> Vec<Characteristic> {
        let mut out = Vec::new();
        if f.width.is_some_and(|w| !(1..=self.max_width).contains(&w))
            || f.height
                .is_some_and(|h| !(1..=self.max_height).contains(&h))
        {
            out.push(Characteristic::Size);
        }
        if f.colorimetry
            .is_some_and(|c| !self.colorimetry.contains(&c))
        {
            out.push(Characteristic::Colorimetry);
        }
        if f.bit_depth.is_some_and(|d| !self.bit_depths.contains(&d)) {
            out.push(Characteristic::BitDepth);
        }
        if f.structure.is_some_and(|s| s != self.structure) {
            out.push(Characteristic::Structure);
        }
        if f.frame_rate.is_some_and(|r| !self.frame_rates.contains(&r)) {
            out.push(Characteristic::FrameRate);
        }
        if f.sampling.is_some_and(|s| s != self.sampling) {
            out.push(Characteristic::Sampling);
        }
        if !self.components.contains(&f.components) {
            out.push(Characteristic::Components);
        }
        if f.quantization
            .is_some_and(|q| !self.quantization.contains(&q))
        {
            out.push(Characteristic::Quantization);
        }
        out
    }
}

fn names<T: std::fmt::Debug>(items: &[T], f: impl Fn(&T) -> String) -> String {
    items.iter().map(f).collect::<Vec<_>>().join(", ")
}

fn rate_name(&(n, d): &(u32, u32)) -> String {
    if d == 1 {
        n.to_string()
    } else {
        format!("{n}/{d}")
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Validators
// ═════════════════════════════════════════════════════════════════════════════

macro_rules! app2e_validator {
    ($(#[$meta:meta])* $name:ident, $edition:expr, $spec_id:literal) => {
        $(#[$meta])*
        pub struct $name;

        impl ConstraintsValidator for $name {
            fn spec_id(&self) -> &str {
                $spec_id
            }

            fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
                App2E { edition: $edition }.validate(cpl)
            }
        }
    };
}

app2e_validator!(
    /// ST 2067-21:2014 Application #2E validator.
    App2E2014,
    App2EEdition::V2014,
    "ST 2067-21:2014 (App2E)"
);
app2e_validator!(
    /// ST 2067-21:2016 Application #2E validator.
    App2E2016,
    App2EEdition::V2016,
    "ST 2067-21:2016 (App2E)"
);
app2e_validator!(
    /// ST 2067-21:2020 Application #2E validator, including Amendment 1:2020.
    App2E2020,
    App2EEdition::V2020,
    "ST 2067-21:2020 (App2E)"
);
app2e_validator!(
    /// ST 2067-21:2023 Application #2E validator.
    App2E2023,
    App2EEdition::V2023,
    "ST 2067-21:2023 (App2E)"
);
app2e_validator!(
    /// ST 2067-21:2026 Application #2E validator.
    App2E2026,
    App2EEdition::V2026,
    "ST 2067-21:2026 (App2E)"
);

/// ST 2067-2 §6.1 Table 10: ApplicationIdentification is an `xs:list` of
/// URIs (whitespace-separated). ST 2067-21 requires it to include the
/// edition's value, so test membership rather than equality.
fn application_identification_includes(app_id: &str, uri: &str) -> bool {
    app_id.split_whitespace().any(|item| item == uri)
}

/// The App #2E checks of one edition.
pub struct App2E {
    edition: App2EEdition,
}

impl App2E {
    /// A validator for `edition`.
    pub fn new(edition: App2EEdition) -> Self {
        Self { edition }
    }

    /// The edition this validator checks against.
    pub fn edition(&self) -> App2EEdition {
        self.edition
    }

    /// This edition's code for `rule`; empty when the edition does not
    /// contain the rule (such findings are dropped by [`App2E::validate`]).
    fn code(&self, rule: App2ERule) -> &'static str {
        self.edition.code(rule).unwrap_or("")
    }

    /// Run every check of the edition.
    pub fn validate(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        self.validate_application_identification(cpl, &mut issues);
        match self.edition {
            App2EEdition::V2014 | App2EEdition::V2016 => {
                self.validate_colorimetry_items_2014_2016(cpl, &mut issues);
                if self.edition == App2EEdition::V2016 {
                    // 2016 §7.2: MaxCLL / MaxFALL for COLOR.6 / COLOR.7.
                    self.check_hdr_metadata_all(cpl, &mut issues);
                }
            }
            App2EEdition::V2020 | App2EEdition::V2023 | App2EEdition::V2026 => {
                self.validate_image_descriptors(cpl, &mut issues);
                self.validate_image_characteristics(cpl, &mut issues);
                self.validate_homogeneous_image_essence(cpl, &mut issues);
                self.validate_segment_duration(cpl, &mut issues);
                self.validate_descriptor_completeness(cpl, &mut issues);
            }
        }
        // A finding whose rule is not part of this edition has no code.
        issues.retain(|i| !i.code.is_empty());
        issues
    }

    /// ApplicationIdentification shall include the edition's value (2014 §7,
    /// 2016/2020/2023 §7.1, 2026 §8.1). A CPL without the element (or
    /// without ExtensionProperties) does not include it.
    fn validate_application_identification(
        &self,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let uri = self.edition.application_identification_uri();
        let app_id = cpl
            .extension_properties
            .as_ref()
            .and_then(|ext| ext.application_identification.as_deref());
        match app_id {
            None => issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Metadata,
                    self.code(App2ERule::ApplicationIdentification),
                    format!(
                        "ApplicationIdentification is absent; {} requires it to include '{}'",
                        self.edition.name(),
                        uri
                    ),
                )
                .with_location(Location::new().with_cpl(cpl.id)),
            ),
            Some(app_id) if !application_identification_includes(app_id, uri) => issues.push(
                ValidationIssue::new(
                    Severity::Warning,
                    Category::Metadata,
                    self.code(App2ERule::AppIdMismatch),
                    format!(
                        "ApplicationIdentification '{}' does not include the {} value '{}'",
                        app_id,
                        self.edition.name(),
                        uri
                    ),
                )
                .with_location(Location::new().with_cpl(cpl.id)),
            ),
            Some(_) => {}
        }
    }

    /// MaxCLL / MaxFALL note for every image descriptor.
    fn check_hdr_metadata_all(&self, cpl: &CompositionPlaylist, issues: &mut Vec<ValidationIssue>) {
        let Some(edl) = &cpl.essence_descriptor_list else {
            return;
        };
        for ed in &edl.essence_descriptors {
            let (kind, tc) = if let Some(ref rgba) = ed.rgba_descriptor {
                ("RGBADescriptor", rgba.transfer_characteristic.as_ref())
            } else if let Some(ref cdci) = ed.cdci_descriptor {
                ("CDCIDescriptor", cdci.transfer_characteristic.as_ref())
            } else {
                continue;
            };
            let loc = Location::new()
                .with_cpl(cpl.id)
                .with_path(format!("EssenceDescriptor[{}]/{}", ed.id, kind));
            self.check_hdr_metadata(tc, cpl, &loc, issues);
        }
    }

    /// Validate a J2K PictureEssenceCoding UL against the edition's JPEG 2000
    /// profile table (2020 §5.5.2 Table 6, 2023 §5.5.2 Table 6, 2026 §6.5.2
    /// Table 5).
    ///
    /// Rules (§6.2.5):
    /// - `Jpeg2000Ht`: listed from the 2023 edition on; not in the 2020 table.
    /// - `Jpeg2000Imf4k`: stored width must be 2049–4096, stored height ≤ 3112.
    /// - `Jpeg2000Imf2k`: stored width must be 1–2048, stored height ≤ 1556.
    /// - `Jpeg2000Imf8k`: 2026 only; stored width 4097–8192, height ≤ 6224.
    /// - `Jpeg2000Broadcast`: stored width must be 1–3840, stored height ≤ 2160.
    /// - `Jpeg2000` (generic node): no resolution bounds enforced.
    /// - All other J2K variants (should not occur after CODEC_TABLE is correct): error.
    // `clippy::collapsible_match` would suggest moving each `if !cond` into a guard, but
    // the wildcard arm has different semantics (specific to non-J2K codecs), so collapsing
    // would route the "passes-validation" case through it incorrectly.
    #[allow(clippy::collapsible_match)]
    fn validate_j2k_profile(
        &self,
        codec: &crate::cpl::VideoCodec,
        stored_width: u32,
        stored_height: u32,
        loc: &Location,
        issues: &mut Vec<ValidationIssue>,
    ) {
        use crate::cpl::VideoCodec;
        match codec {
            VideoCodec::Jpeg2000Ht => {
                if !self.edition.allows_ht() {
                    // ST 2067-21:2020 Table 6 lists no HT-J2K profile;
                    // HT-J2K is added by the 2023 edition.
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Encoding,
                            self.code(App2ERule::J2KHtNotAllowed),
                            "JPEG 2000 HT (ISO 15444-15) is not a JPEG 2000 profile of \
                             ST 2067-21:2020 (Table 6)",
                        )
                        .with_location(loc.clone()),
                    );
                }
                // When allowed, no resolution bounds check for HT — HT
                // conformance lives in the codestream parameters, not the
                // resolution envelope.
            }
            VideoCodec::Jpeg2000Imf4k => {
                // 4K IMF profile: width in (2048, 4096], height in (0, 3112]
                if !(stored_width > 2048
                    && stored_width <= 4096
                    && stored_height > 0
                    && stored_height <= 3112)
                {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Encoding,
                            self.code(App2ERule::J2K4KResolution),
                            format!(
                                "JPEG 2000 IMF 4K Profile does not support image resolution \
                                 ({}/{}); width must be 2049–4096, height 1–3112",
                                stored_width, stored_height
                            ),
                        )
                        .with_location(loc.clone()),
                    );
                }
            }
            VideoCodec::Jpeg2000Imf2k => {
                // 2K IMF profile: width in (0, 2048], height in (0, 1556]
                if !(stored_width > 0
                    && stored_width <= 2048
                    && stored_height > 0
                    && stored_height <= 1556)
                {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Encoding,
                            self.code(App2ERule::J2K2KResolution),
                            format!(
                                "JPEG 2000 IMF 2K Profile does not support image resolution \
                                 ({}/{}); width must be 1–2048, height 1–1556",
                                stored_width, stored_height
                            ),
                        )
                        .with_location(loc.clone()),
                    );
                }
            }
            VideoCodec::Jpeg2000Imf8k => {
                if !self.edition.allows_8k() {
                    // 2020 and 2023 Table 6 have no 8K column.
                    issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Encoding,
                        self.code(App2ERule::J2KProfileNotAllowed),
                        format!(
                            "JPEG 2000 IMF 8K profiles are not in the JPEG 2000 profile table of {}",
                            self.edition.name()
                        ),
                    )
                    .with_location(loc.clone()),
                );
                } else if !(stored_width > 4096
                    && stored_width <= 8192
                    && stored_height > 0
                    && stored_height <= 6224)
                {
                    // 2026 §6.5.2 Table 5: 8K column is 4097..8192 x 1..6224.
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Encoding,
                            self.code(App2ERule::J2K8KResolution),
                            format!(
                                "JPEG 2000 IMF 8K Profile does not support image resolution \
                             ({}/{}); width must be 4097–8192, height 1–6224",
                                stored_width, stored_height
                            ),
                        )
                        .with_location(loc.clone()),
                    );
                }
            }
            VideoCodec::Jpeg2000Broadcast => {
                // Broadcast Contribution profile: width in (0, 3840], height in (0, 2160]
                if !(stored_width > 0
                    && stored_width <= 3840
                    && stored_height > 0
                    && stored_height <= 2160)
                {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Encoding,
                            self.code(App2ERule::J2KBcpResolution),
                            format!(
                                "JPEG 2000 Broadcast Contribution Profile does not support image \
                                 resolution ({}/{}); width must be 1–3840, height 1–2160",
                                stored_width, stored_height
                            ),
                        )
                        .with_location(loc.clone()),
                    );
                }
            }
            VideoCodec::Jpeg2000 => {
                // Generic J2K node UL (03010100) — accepted without resolution bounds check.
                // In well-formed App2E content this should always be a specific sub-profile,
                // but generic ULs appear in some older packages and are not rejected by the
                // profile check (they fall outside the profile/resolution table).
            }
            _ => {
                // Any non-J2K codec: already caught by is_jpeg2000_family() check upstream.
            }
        }
    }

    /// Validate all image essence descriptors in the CPL.
    fn validate_image_descriptors(
        &self,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let edl = match &cpl.essence_descriptor_list {
            Some(edl) => edl,
            None => return, // Core constraints will catch missing EDL
        };

        for ed in &edl.essence_descriptors {
            if let Some(ref rgba) = ed.rgba_descriptor {
                self.validate_rgba_descriptor(&ed.id.to_string(), rgba, cpl, issues);
            }
            if let Some(ref cdci) = ed.cdci_descriptor {
                self.validate_cdci_descriptor(&ed.id.to_string(), cdci, cpl, issues);
            }
        }
    }

    /// Validate an RGBA (RGB) picture descriptor.
    ///
    /// Checks Table 8 (Generic Picture Descriptor), Table 10 (RGBA Descriptor),
    /// Table 11 (Component Max/Min Ref), and Table 3 (colorimetry systems).
    fn validate_rgba_descriptor(
        &self,
        ed_id: &str,
        rgba: &crate::cpl::RGBADescriptor,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let loc = Location::new()
            .with_cpl(cpl.id)
            .with_path(format!("EssenceDescriptor[{}]/RGBADescriptor", ed_id));

        // ── Table 8: Generic Picture Essence Descriptor ──────────────────────

        // §6.2.5: PictureCompression shall be JPEG 2000 (ISO 15444-1 or 15444-15)
        if let Some(ref codec) = rgba.picture_compression {
            if !codec.is_jpeg2000_family() {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Encoding,
                        self.code(App2ERule::J2KRequired).to_string(),
                        format!(
                            "PictureCompression shall be JPEG 2000 for App2E, found: {}",
                            codec
                        ),
                    )
                    .with_location(loc.clone()),
                );
            } else {
                // §6.2.5: J2K profile must be a recognized App2E profile with valid resolution.
                let w = rgba.stored_width.unwrap_or(0);
                let h = rgba.stored_height.unwrap_or(0);
                self.validate_j2k_profile(codec, w, h, &loc, issues);
            }
        }

        // Table 8: FrameLayout — §6.2.1.3
        if let Some(ref fl) = rgba.frame_layout {
            if fl != "FullFrame" && fl != "SeparateFields" {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::FrameLayout).to_string(),
                        format!(
                            "FrameLayout shall be FullFrame (00h) or SeparateFields (01h), found: {}",
                            fl
                        ),
                    )
                    .with_location(loc.clone()),
                );
            }
        }

        // Table 8: StoredF2Offset — shall not be present
        if rgba.stored_f2_offset.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::StoredF2Offset).to_string(),
                    "StoredF2Offset shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 8: SampledWidth — shall not be present or shall be equal to StoredWidth
        if let Some(sw) = rgba.sampled_width {
            if let Some(stored_w) = rgba.stored_width {
                if sw != stored_w {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Video,
                            self.code(App2ERule::SampledWidth).to_string(),
                            format!(
                                "SampledWidth ({}) shall not be present or shall be equal to StoredWidth ({})",
                                sw, stored_w
                            ),
                        )
                        .with_location(loc.clone()),
                    );
                }
            }
        }

        // Table 8: SampledHeight — shall not be present or shall be equal to StoredHeight
        if let Some(sh) = rgba.sampled_height {
            if let Some(stored_h) = rgba.stored_height {
                if sh != stored_h {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Video,
                            self.code(App2ERule::SampledHeight).to_string(),
                            format!(
                                "SampledHeight ({}) shall not be present or shall be equal to StoredHeight ({})",
                                sh, stored_h
                            ),
                        )
                        .with_location(loc.clone()),
                    );
                }
            }
        }

        // Table 8: SampledXOffset — shall not be present or shall be 0
        if let Some(sxo) = rgba.sampled_x_offset {
            if sxo != 0 {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::SampledXOffset).to_string(),
                        format!(
                            "SampledXOffset shall not be present or shall be 0, found: {}",
                            sxo
                        ),
                    )
                    .with_location(loc.clone()),
                );
            }
        }

        // Table 8: SampledYOffset — shall not be present or shall be 0
        if let Some(syo) = rgba.sampled_y_offset {
            if syo != 0 {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::SampledYOffset).to_string(),
                        format!(
                            "SampledYOffset shall not be present or shall be 0, found: {}",
                            syo
                        ),
                    )
                    .with_location(loc.clone()),
                );
            }
        }

        // Table 8: AlphaTransparency — shall not be present
        if rgba.alpha_transparency.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::AlphaTransparency).to_string(),
                    "AlphaTransparency shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 8: ImageAlignmentOffset — shall not be present
        if rgba.image_alignment_offset.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::ImageAlignmentOffset).to_string(),
                    "ImageAlignmentOffset shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 8: ImageStartOffset — shall not be present
        if rgba.image_start_offset.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::ImageStartOffset).to_string(),
                    "ImageStartOffset shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 8: ImageEndOffset — shall not be present
        if rgba.image_end_offset.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::ImageEndOffset).to_string(),
                    "ImageEndOffset shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 8: FieldDominance — conditional on FrameLayout
        if let Some(ref fl) = rgba.frame_layout {
            if fl == "FullFrame" && rgba.field_dominance.is_some() {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::FieldDominance).to_string(),
                        "FieldDominance shall not be present for progressive (FullFrame) content",
                    )
                    .with_location(loc.clone()),
                );
            }
            if fl == "SeparateFields" && rgba.field_dominance.is_none() {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::FieldDominance).to_string(),
                        "FieldDominance shall be present for interlaced (SeparateFields) content",
                    )
                    .with_location(loc.clone()),
                );
            }
        }

        // Table 9: StoredWidth/StoredHeight vs FrameLayout
        // For interlaced (SeparateFields), StoredHeight = ImageFrameHeight / 2
        // For progressive (FullFrame), StoredHeight = ImageFrameHeight
        // Note: We validate that StoredWidth/StoredHeight are present since they define
        // the image dimensions required by Section 6.2.1.1

        // §6.2.4: ColorPrimaries shall be present and recognized
        match &rgba.color_primaries {
            Some(cp) if matches!(cp, ColorPrimaries::Unknown(_)) => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::ColorPrimariesUnknown).to_string(),
                        format!("Unrecognized ColorPrimaries UL: {}", cp),
                    )
                    .with_location(loc.clone()),
                );
            }
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::ColorPrimariesMissing).to_string(),
                        "ColorPrimaries shall be present",
                    )
                    .with_location(loc.clone()),
                );
            }
            _ => {}
        }

        // §6.2.2: TransferCharacteristic shall be present and recognized
        match &rgba.transfer_characteristic {
            Some(tc) if matches!(tc, TransferCharacteristic::Unknown(_)) => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::TransferCharacteristicUnknown)
                            .to_string(),
                        format!("Unrecognized TransferCharacteristic UL: {}", tc),
                    )
                    .with_location(loc.clone()),
                );
            }
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::TransferCharacteristicMissing)
                            .to_string(),
                        "TransferCharacteristic shall be present",
                    )
                    .with_location(loc.clone()),
                );
            }
            _ => {}
        }

        // ── Table 10: RGBA Descriptor ────────────────────────────────────────

        // Table 10: ComponentMaxRef shall be present
        if rgba.component_max_ref.is_none() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::ComponentMaxRef).to_string(),
                    "ComponentMaxRef shall be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 10: ComponentMinRef shall be present
        if rgba.component_min_ref.is_none() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::ComponentMinRef).to_string(),
                    "ComponentMinRef shall be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 10: ScanningDirection shall be present and equal to 00h
        match &rgba.scanning_direction {
            Some(sd) if sd != "ScanningDirection_LeftToRightTopToBottom" => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::ScanningDirection).to_string(),
                        format!(
                            "ScanningDirection shall be 00h (LeftToRightTopToBottom), found: {}",
                            sd
                        ),
                    )
                    .with_location(loc.clone()),
                );
            }
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::ScanningDirection).to_string(),
                        "ScanningDirection shall be present",
                    )
                    .with_location(loc.clone()),
                );
            }
            _ => {} // Valid: present and equals 00h
        }

        // Table 10: AlphaMaxRef — shall not be present
        if rgba.alpha_max_ref.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::AlphaMaxRef).to_string(),
                    "AlphaMaxRef shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 10: AlphaMinRef — shall not be present
        if rgba.alpha_min_ref.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::AlphaMinRef).to_string(),
                    "AlphaMinRef shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 10: Palette — shall not be present
        if rgba.palette.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::Palette).to_string(),
                    "Palette shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 10: PaletteLayout — shall not be present
        if rgba.palette_layout.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::PaletteLayout).to_string(),
                    "PaletteLayout shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 11: Validate ComponentMaxRef/MinRef values against quantization system.
        // Determine QE from the component ref values themselves:
        // QE.2 has MinRef=0 at all bit depths; QE.1 has MinRef>0.
        if let (Some(min_ref), Some(max_ref)) = (rgba.component_min_ref, rgba.component_max_ref) {
            let qe = if min_ref == 0 {
                QuantizationSystem::Qe2
            } else {
                QuantizationSystem::Qe1
            };
            // Determine bit depth from max_ref
            let bit_depth = match max_ref {
                235 | 255 => Some(8u32),
                940 | 1023 => Some(10),
                3760 | 4095 => Some(12),
                60160 | 65535 => Some(16),
                _ => None,
            };
            if let Some(bd) = bit_depth {
                if let Some((expected_min, expected_max)) = component_ref_values(qe, bd) {
                    if min_ref != expected_min || max_ref != expected_max {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                Category::Video,
                                self.code(App2ERule::ComponentRefValues).to_string(),
                                format!(
                                    "ComponentMinRef={}, ComponentMaxRef={} do not match \
                                     {:?} at {} bits (expected min={}, max={})",
                                    min_ref, max_ref, qe, bd, expected_min, expected_max
                                ),
                            )
                            .with_location(loc.clone()),
                        );
                    }
                }
            }
        }

        // ── Table 3: Colorimetry system ──────────────────────────────────────

        if let (Some(cp), Some(tc)) = (&rgba.color_primaries, &rgba.transfer_characteristic) {
            let color_sys = ColorSystem::from_components(cp, tc, None);
            if color_sys.is_none()
                && !matches!(cp, ColorPrimaries::Unknown(_))
                && !matches!(tc, TransferCharacteristic::Unknown(_))
            {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::ColorSystem).to_string(),
                        format!(
                            "ColorPrimaries={} + TransferCharacteristic={} does not form a \
                             recognized Color System for RGB",
                            cp, tc
                        ),
                    )
                    .with_location(loc.clone()),
                );
            }
        }

        // MaxCLL/MaxFALL for PQ-based systems (2020/2023 §7.5, 2026 §8.3.3)
        self.check_hdr_metadata(rgba.transfer_characteristic.as_ref(), cpl, &loc, issues);

        // ── Table 14: JPEG 2000 Picture Sub Descriptor ────────────────────────
        self.validate_j2k_sub_descriptor(
            rgba.sub_descriptors.as_ref(),
            rgba.picture_compression.as_ref(),
            &loc,
            issues,
        );
    }

    /// Validate a CDCI (Y'C'BC'R) picture descriptor.
    ///
    /// Checks Table 8 (Generic Picture Descriptor), Table 12 (CDCI Descriptor),
    /// Table 13 (Black/White Ref Level), and Table 3 (colorimetry systems).
    fn validate_cdci_descriptor(
        &self,
        ed_id: &str,
        cdci: &crate::cpl::CDCIDescriptor,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let loc = Location::new()
            .with_cpl(cpl.id)
            .with_path(format!("EssenceDescriptor[{}]/CDCIDescriptor", ed_id));

        // ── Table 8: Generic Picture Essence Descriptor ──────────────────────

        // §6.2.5: PictureCompression shall be JPEG 2000
        if let Some(ref codec) = cdci.picture_compression {
            if !codec.is_jpeg2000_family() {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Encoding,
                        self.code(App2ERule::J2KRequired).to_string(),
                        format!(
                            "PictureCompression shall be JPEG 2000 for App2E, found: {}",
                            codec
                        ),
                    )
                    .with_location(loc.clone()),
                );
            } else {
                // §6.2.5: J2K profile must be a recognized App2E profile with valid resolution.
                let w = cdci.stored_width.unwrap_or(0);
                let h = cdci.stored_height.unwrap_or(0);
                self.validate_j2k_profile(codec, w, h, &loc, issues);
            }
        }

        // Table 8: FrameLayout — §6.2.1.3
        if let Some(ref fl) = cdci.frame_layout {
            if fl != "FullFrame" && fl != "SeparateFields" {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::FrameLayout).to_string(),
                        format!(
                            "FrameLayout shall be FullFrame (00h) or SeparateFields (01h), found: {}",
                            fl
                        ),
                    )
                    .with_location(loc.clone()),
                );
            }
        }

        // Table 8: StoredF2Offset — shall not be present
        if cdci.stored_f2_offset.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::StoredF2Offset).to_string(),
                    "StoredF2Offset shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 8: SampledWidth — shall not be present or shall be equal to StoredWidth
        if let Some(sw) = cdci.sampled_width {
            if let Some(stored_w) = cdci.stored_width {
                if sw != stored_w {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Video,
                            self.code(App2ERule::SampledWidth).to_string(),
                            format!(
                                "SampledWidth ({}) shall not be present or shall be equal to StoredWidth ({})",
                                sw, stored_w
                            ),
                        )
                        .with_location(loc.clone()),
                    );
                }
            }
        }

        // Table 8: SampledHeight — shall not be present or shall be equal to StoredHeight
        if let Some(sh) = cdci.sampled_height {
            if let Some(stored_h) = cdci.stored_height {
                if sh != stored_h {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Video,
                            self.code(App2ERule::SampledHeight).to_string(),
                            format!(
                                "SampledHeight ({}) shall not be present or shall be equal to StoredHeight ({})",
                                sh, stored_h
                            ),
                        )
                        .with_location(loc.clone()),
                    );
                }
            }
        }

        // Table 8: SampledXOffset — shall not be present or shall be 0
        if let Some(sxo) = cdci.sampled_x_offset {
            if sxo != 0 {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::SampledXOffset).to_string(),
                        format!(
                            "SampledXOffset shall not be present or shall be 0, found: {}",
                            sxo
                        ),
                    )
                    .with_location(loc.clone()),
                );
            }
        }

        // Table 8: SampledYOffset — shall not be present or shall be 0
        if let Some(syo) = cdci.sampled_y_offset {
            if syo != 0 {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::SampledYOffset).to_string(),
                        format!(
                            "SampledYOffset shall not be present or shall be 0, found: {}",
                            syo
                        ),
                    )
                    .with_location(loc.clone()),
                );
            }
        }

        // Table 8: AlphaTransparency — shall not be present
        if cdci.alpha_transparency.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::AlphaTransparency).to_string(),
                    "AlphaTransparency shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 8: ImageAlignmentOffset — shall not be present
        if cdci.image_alignment_offset.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::ImageAlignmentOffset).to_string(),
                    "ImageAlignmentOffset shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 8: ImageStartOffset — shall not be present
        if cdci.image_start_offset.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::ImageStartOffset).to_string(),
                    "ImageStartOffset shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 8: ImageEndOffset — shall not be present
        if cdci.image_end_offset.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::ImageEndOffset).to_string(),
                    "ImageEndOffset shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 8: FieldDominance — conditional on FrameLayout
        if let Some(ref fl) = cdci.frame_layout {
            if fl == "FullFrame" && cdci.field_dominance.is_some() {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::FieldDominance).to_string(),
                        "FieldDominance shall not be present for progressive (FullFrame) content",
                    )
                    .with_location(loc.clone()),
                );
            }
            if fl == "SeparateFields" && cdci.field_dominance.is_none() {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::FieldDominance).to_string(),
                        "FieldDominance shall be present for interlaced (SeparateFields) content",
                    )
                    .with_location(loc.clone()),
                );
            }
        }

        // §6.2.4: ColorPrimaries shall be present and recognized
        match &cdci.color_primaries {
            Some(cp) if matches!(cp, ColorPrimaries::Unknown(_)) => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::ColorPrimariesUnknown).to_string(),
                        format!("Unrecognized ColorPrimaries UL: {}", cp),
                    )
                    .with_location(loc.clone()),
                );
            }
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::ColorPrimariesMissing).to_string(),
                        "ColorPrimaries shall be present",
                    )
                    .with_location(loc.clone()),
                );
            }
            _ => {}
        }

        // §6.2.2: TransferCharacteristic shall be present and recognized
        match &cdci.transfer_characteristic {
            Some(tc) if matches!(tc, TransferCharacteristic::Unknown(_)) => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::TransferCharacteristicUnknown)
                            .to_string(),
                        format!("Unrecognized TransferCharacteristic UL: {}", tc),
                    )
                    .with_location(loc.clone()),
                );
            }
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::TransferCharacteristicMissing)
                            .to_string(),
                        "TransferCharacteristic shall be present",
                    )
                    .with_location(loc.clone()),
                );
            }
            _ => {}
        }

        // §6.2.3: CodingEquations shall be present for Y'C'BC'R
        match &cdci.coding_equations {
            Some(ce) if matches!(ce, CodingEquations::Unknown(_)) => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::CodingEquationsUnknown).to_string(),
                        format!("Unrecognized CodingEquations UL: {}", ce),
                    )
                    .with_location(loc.clone()),
                );
            }
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::CodingEquationsMissing).to_string(),
                        "CodingEquations shall be present for Y'C'BC'R",
                    )
                    .with_location(loc.clone()),
                );
            }
            _ => {}
        }

        // ── Table 12: CDCI Descriptor ────────────────────────────────────────

        // Table 12: ComponentDepth shall be present. Its value shall be the
        // pixel bit depth, which Table 3 constrains per column; the value is
        // checked by `validate_image_characteristics`.
        if cdci.component_depth.is_none() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::ComponentDepth).to_string(),
                    "ComponentDepth shall be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 12: HorizontalSubsampling — §6.4.2
        // 1 = 4:4:4, 2 = 4:2:2
        match cdci.horizontal_subsampling {
            Some(hs) if hs != 1 && hs != 2 => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::HorizontalSubsampling).to_string(),
                        format!(
                            "HorizontalSubsampling shall be 1 (4:4:4) or 2 (4:2:2), found: {}",
                            hs
                        ),
                    )
                    .with_location(loc.clone()),
                );
            }
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::HorizontalSubsampling).to_string(),
                        "HorizontalSubsampling shall be present and equal to 1 (4:4:4) or 2 (4:2:2)",
                    )
                    .with_location(loc.clone()),
                );
            }
            _ => {}
        }

        // Table 12: VerticalSubsampling shall be 1
        match cdci.vertical_subsampling {
            Some(vs) if vs != 1 => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::VerticalSubsampling).to_string(),
                        format!("VerticalSubsampling shall be 1, found: {}", vs),
                    )
                    .with_location(loc.clone()),
                );
            }
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::VerticalSubsampling).to_string(),
                        "VerticalSubsampling shall be present and equal to 1",
                    )
                    .with_location(loc.clone()),
                );
            }
            _ => {}
        }

        // Table 12: ColorSiting shall be present and 0
        match cdci.color_siting {
            Some(cs) if cs != 0 => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::ColorSiting).to_string(),
                        format!("ColorSiting shall be 0, found: {}", cs),
                    )
                    .with_location(loc.clone()),
                );
            }
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::ColorSiting).to_string(),
                        "ColorSiting shall be present and equal to 0",
                    )
                    .with_location(loc.clone()),
                );
            }
            _ => {}
        }

        // Table 12: ReversedByteOrder — shall not be present
        if cdci.reversed_byte_order.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::ReversedByteOrder).to_string(),
                    "ReversedByteOrder shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 12: PaddingBits — shall not be present
        if cdci.padding_bits.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::PaddingBits).to_string(),
                    "PaddingBits shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 12: AlphaSampleDepth — shall not be present
        if cdci.alpha_sample_depth.is_some() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Video,
                    self.code(App2ERule::AlphaSampleDepth).to_string(),
                    "AlphaSampleDepth shall not be present",
                )
                .with_location(loc.clone()),
            );
        }

        // ── Table 13: Black/White Ref Level and Color Range ──────────────────

        // Determine the color system to look up correct reference values
        if let (Some(cp), Some(tc), Some(ce)) = (
            &cdci.color_primaries,
            &cdci.transfer_characteristic,
            &cdci.coding_equations,
        ) {
            // Table 3: Validate colorimetry system triplet
            let color_sys = ColorSystem::from_components(cp, tc, Some(ce));
            if let Some(ref cs) = color_sys {
                // Table 13: Validate Black/White Ref Level and Color Range
                if let Some(depth) = cdci.component_depth {
                    if let Some((exp_black, exp_white, exp_range)) = cdci_ref_values(cs, depth) {
                        if let Some(black) = cdci.black_ref_level {
                            if black != exp_black {
                                issues.push(
                                    ValidationIssue::new(
                                        Severity::Error,
                                        Category::Video,
                                        self.code(App2ERule::BlackRefLevel).to_string(),
                                        format!(
                                            "BlackRefLevel={} for {} at {}-bit; expected {}",
                                            black, cs, depth, exp_black
                                        ),
                                    )
                                    .with_location(loc.clone()),
                                );
                            }
                        }
                        if let Some(white) = cdci.white_ref_level {
                            if white != exp_white {
                                issues.push(
                                    ValidationIssue::new(
                                        Severity::Error,
                                        Category::Video,
                                        self.code(App2ERule::WhiteRefLevel).to_string(),
                                        format!(
                                            "WhiteRefLevel={} for {} at {}-bit; expected {}",
                                            white, cs, depth, exp_white
                                        ),
                                    )
                                    .with_location(loc.clone()),
                                );
                            }
                        }
                        if let Some(range) = cdci.color_range {
                            if range != exp_range {
                                issues.push(
                                    ValidationIssue::new(
                                        Severity::Error,
                                        Category::Video,
                                        self.code(App2ERule::ColorRange).to_string(),
                                        format!(
                                            "ColorRange={} for {} at {}-bit; expected {}",
                                            range, cs, depth, exp_range
                                        ),
                                    )
                                    .with_location(loc.clone()),
                                );
                            }
                        }
                    }
                }
            } else if !matches!(cp, ColorPrimaries::Unknown(_))
                && !matches!(tc, TransferCharacteristic::Unknown(_))
                && !matches!(ce, CodingEquations::Unknown(_))
            {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::ColorSystem).to_string(),
                        format!(
                            "ColorPrimaries={} + TransferCharacteristic={} + CodingEquations={} \
                             does not form a recognized Color System",
                            cp, tc, ce
                        ),
                    )
                    .with_location(loc.clone()),
                );
            }
        }

        // MaxCLL/MaxFALL for PQ-based systems (2020/2023 §7.5, 2026 §8.3.3)
        self.check_hdr_metadata(cdci.transfer_characteristic.as_ref(), cpl, &loc, issues);

        // ── Table 14: JPEG 2000 Picture Sub Descriptor ────────────────────────
        self.validate_j2k_sub_descriptor(
            cdci.sub_descriptors.as_ref(),
            cdci.picture_compression.as_ref(),
            &loc,
            issues,
        );
    }

    /// MaxCLL and MaxFALL for PQ (ST 2084) content: 2016 §7.2, 2020/2023
    /// §7.5, 2026 §8.3.3 give them 0..1 cardinality for COLOR.6/COLOR.7
    /// (the PQ-based systems). They are OPTIONAL, not required.
    /// We emit an informational note when they are absent.
    fn check_hdr_metadata(
        &self,
        tc: Option<&TransferCharacteristic>,
        cpl: &CompositionPlaylist,
        loc: &Location,
        issues: &mut Vec<ValidationIssue>,
    ) {
        if let Some(TransferCharacteristic::PqSt2084) = tc {
            let has_hdr_metadata = cpl
                .extension_properties
                .as_ref()
                .map(|ext| ext.max_cll.is_some() && ext.max_fall.is_some())
                .unwrap_or(false);
            if !has_hdr_metadata {
                issues.push(
                    ValidationIssue::new(
                        Severity::Info,
                        Category::Video,
                        self.code(App2ERule::MaxCLLMaxFALL).to_string(),
                        "MaxCLL and MaxFALL are not present for PQ (ST 2084) content; \
                         they are optional (0..1 cardinality)",
                    )
                    .with_location(loc.clone())
                    .with_suggestion("Consider adding MaxCLL and MaxFALL to ExtensionProperties"),
                );
            }
        }
    }

    /// Table 14: JPEG 2000 Picture Sub Descriptor validation.
    ///
    /// Shared between RGBA and CDCI descriptor validation.
    fn validate_j2k_sub_descriptor(
        &self,
        sub_descriptors: Option<&crate::cpl::VideoSubDescriptors>,
        picture_compression: Option<&crate::cpl::VideoCodec>,
        loc: &Location,
        issues: &mut Vec<ValidationIssue>,
    ) {
        // Only validate J2K sub-descriptor if codec is JPEG 2000
        let is_j2k = picture_compression
            .map(|pc| pc.is_jpeg2000_family())
            .unwrap_or(false);
        if !is_j2k {
            return;
        }

        let j2k_sub = sub_descriptors.and_then(|sd| sd.jpeg2000_sub_descriptor.as_ref());

        let j2k_sub = match j2k_sub {
            Some(sub) => sub,
            None => {
                // Table 14 says it shall be present, but it might not be serialized in CPL
                // (it's an MXF concept). Emit warning to make this normative gap visible.
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        Category::Encoding,
                        self.code(App2ERule::Jpeg2000SubDescriptor).to_string(),
                        "JPEG2000SubDescriptor is missing; it is required for JPEG 2000 picture essence",
                    )
                    .with_location(loc.clone())
                    .with_suggestion("Include JPEG2000SubDescriptor metadata in CPL/mapping when available"),
                );
                return;
            }
        };

        // Table 14: CodingStyleDefault (Coding Style) — shall be present
        if j2k_sub.coding_style_default.is_none() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Encoding,
                    self.code(App2ERule::CodingStyle).to_string(),
                    "CodingStyleDefault (Coding Style) shall be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 14: J2CLayout — shall be present (§6.5.2)
        if j2k_sub.j2c_layout.is_none() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Encoding,
                    self.code(App2ERule::J2CLayout).to_string(),
                    "J2CLayout shall be present",
                )
                .with_location(loc.clone()),
            );
        }

        // Table 14: J2KExtendedCapabilities — shall be present if ISO/IEC 15444-15 coding
        // Rsiz bit 14 (0x4000 = 16384) indicates Part 15 (HTJ2K) profile
        if let Some(rsiz) = j2k_sub.rsiz {
            if rsiz & 0x4000 != 0 && j2k_sub.j2k_extended_capabilities.is_none() {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Encoding,
                        self.code(App2ERule::J2KExtendedCapabilities).to_string(),
                        "J2KExtendedCapabilities shall be present when ISO/IEC 15444-15 coding is used",
                    )
                    .with_location(loc.clone()),
                );
            }
        }
    }

    /// Section 7.2: All image descriptors in a composition must use the same color system.
    fn validate_homogeneous_image_essence(
        &self,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let edl = match &cpl.essence_descriptor_list {
            Some(edl) => edl,
            None => return,
        };

        let mut color_systems: Vec<ColorSystem> = Vec::new();

        for ed in &edl.essence_descriptors {
            if let Some(ref cdci) = ed.cdci_descriptor {
                if let (Some(cp), Some(tc), Some(ce)) = (
                    &cdci.color_primaries,
                    &cdci.transfer_characteristic,
                    &cdci.coding_equations,
                ) {
                    if let Some(cs) = ColorSystem::from_components(cp, tc, Some(ce)) {
                        color_systems.push(cs);
                    }
                }
            }
            if let Some(ref rgba) = ed.rgba_descriptor {
                if let (Some(cp), Some(tc)) = (&rgba.color_primaries, &rgba.transfer_characteristic)
                {
                    if let Some(cs) = ColorSystem::from_components(cp, tc, None) {
                        color_systems.push(cs);
                    }
                }
            }
        }

        if !color_systems.is_empty() {
            let first = &color_systems[0];
            if !color_systems.iter().all(|cs| cs == first) {
                let unique: HashSet<_> = color_systems.iter().collect();
                let mut systems: Vec<_> = unique.iter().map(|cs| cs.to_string()).collect();
                systems.sort();
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::HomogeneousImageEssence),
                        format!(
                            "Heterogeneous image essence: found {} different color systems ({}); \
                             all image essence in a composition shall use the same color system",
                            unique.len(),
                            systems.join(", ")
                        ),
                    )
                    .with_location(Location::new().with_cpl(cpl.id)),
                );
            }
        }
    }

    /// Segment Duration (2020/2023 §7.4, 2026 §8.3.2): If the average number of audio samples per Composition
    /// Edit Unit is not an integer, the duration of each Segment shall be an integer
    /// multiple of 5/Composition Edit Rate (i.e., segment duration in edit units must
    /// be divisible by 5).
    fn validate_segment_duration(
        &self,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        // Need Composition Edit Rate to compute samples-per-edit-unit.
        let edit_rate = match &cpl.edit_rate {
            Some(er) if er.numerator > 0 && er.denominator > 0 => er,
            _ => return,
        };

        // Collect audio sample rates from WAVEPCMDescriptors.
        let edl = match &cpl.essence_descriptor_list {
            Some(edl) => edl,
            None => return,
        };

        let mut non_integer_audio = false;
        for ed in &edl.essence_descriptors {
            if let Some(ref wave) = ed.wave_pcm_descriptor {
                if let Some(ref asr) = wave.audio_sample_rate {
                    // samples_per_edit_unit = (asr_num / asr_den) / (er_num / er_den)
                    //                       = (asr_num * er_den) / (asr_den * er_num)
                    // Non-integer if (asr_num * er_den) % (asr_den * er_num) != 0
                    let numerator = asr.numerator as u64 * edit_rate.denominator as u64;
                    let denominator = asr.denominator as u64 * edit_rate.numerator as u64;
                    if denominator > 0 && !numerator.is_multiple_of(denominator) {
                        non_integer_audio = true;
                        break;
                    }
                }
            }
        }

        if !non_integer_audio {
            return;
        }

        // Constraint applies: each segment's duration (in edit units) must be divisible by 5.
        // Segment duration = sum of effective durations of resources in the main image track.
        for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
            let mut segment_duration: u64 = 0;
            for seq in &segment.sequence_list.main_image_sequences {
                for resource in &seq.resource_list.resources {
                    let effective = resource
                        .source_duration
                        .unwrap_or(resource.intrinsic_duration);
                    segment_duration += effective;
                }
            }

            if segment_duration > 0 && !segment_duration.is_multiple_of(5) {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Timing,
                        self.code(App2ERule::SegmentDurationMultiple).to_string(),
                        format!(
                            "Segment {} duration ({} edit units) is not a multiple of 5; \
                             segment duration is required to be an integer multiple of \
                             5/Composition Edit Rate when audio samples per edit unit is non-integer",
                            seg_idx + 1,
                            segment_duration,
                        ),
                    )
                    .with_location(
                        Location::new()
                            .with_cpl(cpl.id)
                            .with_segment(seg_idx),
                    ),
                );
            }
        }
    }

    /// ST 2067-21 §6.2: Validate that required fields are present on essence descriptors.
    ///
    /// Image descriptors (RGBA/CDCI) must have: StoredWidth, StoredHeight, SampleRate,
    /// FrameLayout, ColorPrimaries, TransferCharacteristic, PictureCompression.
    fn validate_descriptor_completeness(
        &self,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let edl = match &cpl.essence_descriptor_list {
            Some(edl) => edl,
            None => return,
        };

        for ed in &edl.essence_descriptors {
            let ed_loc = Location::new()
                .with_cpl(cpl.id)
                .with_path(format!("EssenceDescriptor/{}", ed.id));

            // Image descriptor required fields
            if let Some(ref rgba) = ed.rgba_descriptor {
                let mut push = |code: &'static str, field: &'static str| {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Encoding,
                            code.to_string(),
                            format!(
                                "RGBADescriptor {}: required field {} is missing",
                                ed.id, field
                            ),
                        )
                        .with_location(ed_loc.clone()),
                    );
                };
                if rgba.stored_width.is_none() {
                    push(self.code(App2ERule::RequiredStoredWidth), "StoredWidth");
                }
                if rgba.stored_height.is_none() {
                    push(self.code(App2ERule::RequiredStoredHeight), "StoredHeight");
                }
                if rgba.sample_rate.is_none() {
                    push(self.code(App2ERule::RequiredSampleRate), "SampleRate");
                }
                if rgba.frame_layout.is_none() {
                    push(self.code(App2ERule::RequiredFrameLayout), "FrameLayout");
                }
                if rgba.color_primaries.is_none() {
                    push(
                        self.code(App2ERule::RequiredColorPrimaries),
                        "ColorPrimaries",
                    );
                }
                if rgba.transfer_characteristic.is_none() {
                    push(
                        self.code(App2ERule::RequiredTransferCharacteristic),
                        "TransferCharacteristic",
                    );
                }
                if rgba.picture_compression.is_none() {
                    push(
                        self.code(App2ERule::RequiredPictureCompression),
                        "PictureCompression",
                    );
                }
            }

            if let Some(ref cdci) = ed.cdci_descriptor {
                let mut push = |code: &'static str, field: &'static str| {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Encoding,
                            code.to_string(),
                            format!(
                                "CDCIDescriptor {}: required field {} is missing",
                                ed.id, field
                            ),
                        )
                        .with_location(ed_loc.clone()),
                    );
                };
                if cdci.stored_width.is_none() {
                    push(self.code(App2ERule::RequiredStoredWidth), "StoredWidth");
                }
                if cdci.stored_height.is_none() {
                    push(self.code(App2ERule::RequiredStoredHeight), "StoredHeight");
                }
                if cdci.sample_rate.is_none() {
                    push(self.code(App2ERule::RequiredSampleRate), "SampleRate");
                }
                if cdci.frame_layout.is_none() {
                    push(self.code(App2ERule::RequiredFrameLayout), "FrameLayout");
                }
                if cdci.color_primaries.is_none() {
                    push(
                        self.code(App2ERule::RequiredColorPrimaries),
                        "ColorPrimaries",
                    );
                }
                if cdci.transfer_characteristic.is_none() {
                    push(
                        self.code(App2ERule::RequiredTransferCharacteristic),
                        "TransferCharacteristic",
                    );
                }
                if cdci.picture_compression.is_none() {
                    push(
                        self.code(App2ERule::RequiredPictureCompression),
                        "PictureCompression",
                    );
                }
                if cdci.component_depth.is_none() {
                    push(
                        self.code(App2ERule::RequiredComponentDepth),
                        "ComponentDepth",
                    );
                }
            }
        }
    }

    /// Each image frame shall conform to one column of the edition's
    /// image-characteristics table (2020/2023 §5.2 Table 3, 2026 §6.2.1
    /// Table 2): all of its characteristics have to fit the same column.
    ///
    /// When no column fits, the closest column is the one with the fewest
    /// differing characteristics; ties go to a column with the descriptor's
    /// frame structure, then its colour components, then its colorimetry,
    /// then table order. Each
    /// differing characteristic is reported under the closest existing code:
    ///
    /// | Characteristic            | Code                    |
    /// |---------------------------|-------------------------|
    /// | image frame size          | `Resolution`            |
    /// | frame rate                | `FrameRate`             |
    /// | colorimetry               | `ColorSystem`           |
    /// | Y'C'BC'R vs R'G'B'        | `ColorSystem`           |
    /// | pixel bit depth (CDCI)    | `ComponentDepth`        |
    /// | pixel bit depth (RGBA)    | `ComponentRefValues`    |
    /// | quantization (RGBA)       | `ComponentRefValues`    |
    /// | progressive / interlaced  | `FrameLayoutInterlaced` |
    /// | 4:2:2 vs 4:4:4            | `HorizontalSubsampling` |
    fn validate_image_characteristics(
        &self,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let Some((table, columns)) = self.edition.image_characteristics() else {
            return;
        };
        let Some(edl) = &cpl.essence_descriptor_list else {
            return;
        };

        for ed in &edl.essence_descriptors {
            let (frame, kind) = if let Some(ref rgba) = ed.rgba_descriptor {
                let (bit_depth, quantization) =
                    rgba_depth_and_quantization(rgba.component_min_ref, rgba.component_max_ref);
                let colorimetry = match (&rgba.color_primaries, &rgba.transfer_characteristic) {
                    (Some(cp), Some(tc)) => ColorSystem::from_components(cp, tc, None),
                    _ => None,
                };
                (
                    ImageFrame {
                        width: rgba.stored_width,
                        height: rgba
                            .stored_height
                            .map(|h| image_frame_height(h, rgba.frame_layout.as_deref())),
                        colorimetry,
                        bit_depth,
                        structure: frame_structure(rgba.frame_layout.as_deref()),
                        frame_rate: rgba.sample_rate.as_ref().and_then(reduce_rate),
                        sampling: Some(Sampling::S444),
                        components: ColorComponents::Rgb,
                        quantization,
                    },
                    "RGBA",
                )
            } else if let Some(ref cdci) = ed.cdci_descriptor {
                let colorimetry = match (
                    &cdci.color_primaries,
                    &cdci.transfer_characteristic,
                    &cdci.coding_equations,
                ) {
                    (Some(cp), Some(tc), Some(ce)) => {
                        ColorSystem::from_components(cp, tc, Some(ce))
                    }
                    _ => None,
                };
                // 2020/2023 §6.4.2 (2026 §7.4.2): Horizontal Subsampling 01h
                // is 4:4:4 and 02h is 4:2:2; Vertical Subsampling is 01h.
                let sampling = match (cdci.horizontal_subsampling, cdci.vertical_subsampling) {
                    (Some(1), Some(1) | None) => Some(Sampling::S444),
                    (Some(2), Some(1) | None) => Some(Sampling::S422),
                    _ => None,
                };
                (
                    ImageFrame {
                        width: cdci.stored_width,
                        height: cdci
                            .stored_height
                            .map(|h| image_frame_height(h, cdci.frame_layout.as_deref())),
                        colorimetry,
                        bit_depth: cdci.component_depth,
                        structure: frame_structure(cdci.frame_layout.as_deref()),
                        frame_rate: cdci.sample_rate.as_ref().and_then(reduce_rate),
                        sampling,
                        components: ColorComponents::YCbCr,
                        // Table 3 lists only QE.1 for Y'C'BC'R; the CDCI
                        // descriptor does not carry the quantization
                        // system (its ref levels are checked separately).
                        quantization: None,
                    },
                    "CDCI",
                )
            } else {
                continue;
            };

            let candidates: Vec<(usize, Vec<Characteristic>)> = columns
                .iter()
                .enumerate()
                .map(|(i, c)| (i, c.mismatches(&frame)))
                .collect();
            if candidates.iter().any(|(_, m)| m.is_empty()) {
                continue;
            }
            let Some((closest, differences)) = candidates.into_iter().min_by_key(|(i, m)| {
                (
                    m.len(),
                    m.contains(&Characteristic::Structure),
                    m.contains(&Characteristic::Components),
                    m.contains(&Characteristic::Colorimetry),
                    *i,
                )
            }) else {
                continue;
            };
            let column = &columns[closest];
            let loc = Location::new()
                .with_cpl(cpl.id)
                .with_path(format!("EssenceDescriptor/{}", ed.id));

            for difference in differences {
                let (rule, detail) = match difference {
                    Characteristic::Size => (
                        App2ERule::Resolution,
                        format!(
                            "image frame {}x{} is outside 1..{} x 1..{}",
                            frame.width.unwrap_or(0),
                            frame.height.unwrap_or(0),
                            column.max_width,
                            column.max_height
                        ),
                    ),
                    Characteristic::Colorimetry => (
                        App2ERule::ColorSystem,
                        format!(
                            "colorimetry {} is not {}",
                            frame.colorimetry.map(|c| c.designator()).unwrap_or("?"),
                            names(column.colorimetry, |c| c.designator().to_string())
                        ),
                    ),
                    Characteristic::BitDepth => (
                        if kind == "CDCI" {
                            App2ERule::ComponentDepth
                        } else {
                            App2ERule::ComponentRefValues
                        },
                        format!(
                            "pixel bit depth {} is not {}",
                            frame.bit_depth.unwrap_or(0),
                            names(column.bit_depths, |d| d.to_string())
                        ),
                    ),
                    Characteristic::Structure => (
                        App2ERule::FrameLayoutInterlaced,
                        format!(
                            "frame structure is {}, the column is {}",
                            match frame.structure {
                                Some(FrameStructure::Interlaced) => "interlaced",
                                _ => "progressive",
                            },
                            match column.structure {
                                FrameStructure::Interlaced => "interlaced",
                                FrameStructure::Progressive => "progressive",
                            }
                        ),
                    ),
                    Characteristic::FrameRate => (
                        App2ERule::FrameRate,
                        format!(
                            "frame rate {} is not {}",
                            frame.frame_rate.as_ref().map(rate_name).unwrap_or_default(),
                            names(column.frame_rates, rate_name)
                        ),
                    ),
                    Characteristic::Sampling => (
                        App2ERule::HorizontalSubsampling,
                        format!(
                            "sampling is {}, the column is {}",
                            match frame.sampling {
                                Some(Sampling::S422) => "4:2:2",
                                _ => "4:4:4",
                            },
                            match column.sampling {
                                Sampling::S422 => "4:2:2",
                                Sampling::S444 => "4:4:4",
                            }
                        ),
                    ),
                    Characteristic::Components => (
                        App2ERule::ColorSystem,
                        format!(
                            "color components are {}, the column allows {}",
                            match frame.components {
                                ColorComponents::YCbCr => "Y'C'BC'R",
                                ColorComponents::Rgb => "R'G'B'",
                            },
                            names(column.components, |c| match c {
                                ColorComponents::YCbCr => "Y'C'BC'R".to_string(),
                                ColorComponents::Rgb => "R'G'B'".to_string(),
                            })
                        ),
                    ),
                    Characteristic::Quantization => (
                        App2ERule::ComponentRefValues,
                        format!(
                            "quantization {:?} is not {}",
                            frame.quantization.unwrap_or(QuantizationSystem::Qe1),
                            names(column.quantization, |q| format!("{q:?}"))
                        ),
                    ),
                };
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(rule),
                        format!(
                            "{kind} descriptor {}: image characteristics fit no column of {} {}; \
                             closest column \"{}\": {}",
                            ed.id,
                            self.edition.name(),
                            table,
                            column.label,
                            detail
                        ),
                    )
                    .with_location(loc.clone()),
                );
            }
        }
    }

    /// 2014 §6.1.1–6.1.4 and 2016 §6.1.1–6.1.3, §6.2, §6.3: the items an
    /// edition fixes for the colorimetry systems it defines.
    ///
    /// These editions extend ST 2067-20, which defines COLOR.1–COLOR.3; they
    /// only constrain descriptors using a system they define (2014:
    /// COLOR.4/5; 2016: COLOR.4–7). A system counts as used when the
    /// descriptor carries a UL that only that edition's systems use; all of
    /// the system's items then have to carry the edition's values, and the
    /// edition's reference-level tables apply.
    fn validate_colorimetry_items_2014_2016(
        &self,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let systems: &[EditionColorSystem] = match self.edition {
            App2EEdition::V2014 => SYSTEMS_2014,
            App2EEdition::V2016 => SYSTEMS_2016,
            _ => return,
        };
        let Some(edl) = &cpl.essence_descriptor_list else {
            return;
        };

        for ed in &edl.essence_descriptors {
            let (kind, cp, tc, ce) = if let Some(ref rgba) = ed.rgba_descriptor {
                // Coding Equations do not apply to R'G'B' components.
                (
                    "RGBADescriptor",
                    rgba.color_primaries.as_ref(),
                    rgba.transfer_characteristic.as_ref(),
                    None,
                )
            } else if let Some(ref cdci) = ed.cdci_descriptor {
                (
                    "CDCIDescriptor",
                    cdci.color_primaries.as_ref(),
                    cdci.transfer_characteristic.as_ref(),
                    cdci.coding_equations.as_ref(),
                )
            } else {
                continue;
            };
            let loc = Location::new()
                .with_cpl(cpl.id)
                .with_path(format!("EssenceDescriptor[{}]/{}", ed.id, kind));

            let used: Vec<&EditionColorSystem> = systems
                .iter()
                .filter(|s| {
                    tc.is_some_and(|tc| s.indicates_tc && *tc == s.transfer)
                        || cp.is_some_and(|cp| s.indicates_cp && *cp == s.primaries)
                        || ce.is_some_and(|ce| {
                            s.coding.as_ref().is_some_and(|c| s.indicates_ce && ce == c)
                        })
                })
                .collect();
            if used.is_empty() {
                continue;
            }
            let matched = used.iter().find(|s| {
                tc.is_some_and(|tc| *tc == s.transfer)
                    && cp.is_some_and(|cp| *cp == s.primaries)
                    && match (ce, &s.coding) {
                        (Some(ce), Some(required)) => ce == required,
                        _ => true,
                    }
            });
            let Some(system) = matched else {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        self.code(App2ERule::ColorSystem),
                        format!(
                            "{kind} {}: ColorPrimaries={}, TransferCharacteristic={}{} do not \
                             carry the values {} requires for {}",
                            ed.id,
                            cp.map(|v| v.to_string()).unwrap_or_else(|| "absent".into()),
                            tc.map(|v| v.to_string()).unwrap_or_else(|| "absent".into()),
                            ce.map(|v| format!(", CodingEquations={v}"))
                                .unwrap_or_default(),
                            self.edition.name(),
                            used.iter().map(|s| s.name).collect::<Vec<_>>().join(" or ")
                        ),
                    )
                    .with_location(loc),
                );
                continue;
            };

            if let Some(ref cdci) = ed.cdci_descriptor {
                let Some(depth) = cdci.component_depth else {
                    continue;
                };
                let Some(&(_, black, white, range)) =
                    system.ref_levels.iter().find(|(d, ..)| *d == depth)
                else {
                    // The edition gives no values for this bit depth.
                    continue;
                };
                for (rule, item, value, expected) in [
                    (
                        App2ERule::BlackRefLevel,
                        "BlackRefLevel",
                        cdci.black_ref_level,
                        black,
                    ),
                    (
                        App2ERule::WhiteRefLevel,
                        "WhiteRefLevel",
                        cdci.white_ref_level,
                        white,
                    ),
                    (App2ERule::ColorRange, "ColorRange", cdci.color_range, range),
                ] {
                    if let Some(value) = value {
                        if value != expected {
                            issues.push(
                                ValidationIssue::new(
                                    Severity::Error,
                                    Category::Video,
                                    self.code(rule),
                                    format!(
                                        "{item}={value} for {} at {depth}-bit; {} requires {expected}",
                                        system.name,
                                        self.edition.name()
                                    ),
                                )
                                .with_location(loc.clone()),
                            );
                        }
                    }
                }
            } else if let Some(ref rgba) = ed.rgba_descriptor {
                // 2016 §6.2 Table 6 gives Component Min/Max Ref for 12 and
                // 16 bit only (8/10 bit are in ST 2067-20 Table 10).
                if self.edition != App2EEdition::V2016 {
                    continue;
                }
                if let (Some(min_ref), Some(max_ref)) =
                    (rgba.component_min_ref, rgba.component_max_ref)
                {
                    let depth = match max_ref {
                        3760 | 4095 => 12,
                        60160 | 65535 => 16,
                        _ => continue,
                    };
                    let qe = if min_ref == 0 {
                        QuantizationSystem::Qe2
                    } else {
                        QuantizationSystem::Qe1
                    };
                    let expected = match (qe, depth) {
                        (QuantizationSystem::Qe1, 12) => (256, 3760),
                        (QuantizationSystem::Qe1, _) => (4096, 60160),
                        (QuantizationSystem::Qe2, 12) => (0, 4095),
                        (QuantizationSystem::Qe2, _) => (0, 65535),
                    };
                    if (min_ref, max_ref) != expected {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                Category::Video,
                                self.code(App2ERule::ComponentRefValues),
                                format!(
                                    "ComponentMinRef={min_ref}, ComponentMaxRef={max_ref} do not \
                                     match {qe:?} at {depth} bits (expected min={}, max={})",
                                    expected.0, expected.1
                                ),
                            )
                            .with_location(loc),
                        );
                    }
                }
            }
        }
    }
}

fn frame_structure(frame_layout: Option<&str>) -> Option<FrameStructure> {
    match frame_layout {
        Some("FullFrame") => Some(FrameStructure::Progressive),
        Some("SeparateFields") => Some(FrameStructure::Interlaced),
        _ => None,
    }
}

/// A colorimetry system as defined by the 2014 or 2016 edition: the UL
/// values of §6.1.1–6.1.3 and the CDCI reference levels of 2014 §6.1.4
/// Table 4 / 2016 §6.3 Table 7 as `(bit depth, black, white, color range)`.
struct EditionColorSystem {
    name: &'static str,
    transfer: TransferCharacteristic,
    primaries: ColorPrimaries,
    coding: Option<CodingEquations>,
    /// Whether the item's value is used only by systems this edition
    /// defines (and so signals that one of them is used).
    indicates_tc: bool,
    indicates_cp: bool,
    indicates_ce: bool,
    ref_levels: &'static [(u32, u32, u32, u32)],
}

const REF_COLOR4: &[(u32, u32, u32, u32)] = &[(8, 16, 235, 254), (10, 64, 940, 1013)];

/// ST 2067-21:2014 §6.1.1–6.1.4 (p5–6). COLOR.5 uses the BT.709 Coding
/// Equations UL in this edition (§6.1.2), and Table 4 gives COLOR.5 values
/// for 10 bit only.
const SYSTEMS_2014: &[EditionColorSystem] = &[
    EditionColorSystem {
        name: "COLOR.4",
        transfer: TransferCharacteristic::XvYcc709,
        primaries: ColorPrimaries::Bt709,
        coding: Some(CodingEquations::Bt709),
        indicates_tc: true,
        indicates_cp: false,
        indicates_ce: false,
        ref_levels: REF_COLOR4,
    },
    EditionColorSystem {
        name: "COLOR.5",
        transfer: TransferCharacteristic::Bt2020,
        primaries: ColorPrimaries::Bt2020,
        coding: Some(CodingEquations::Bt709),
        indicates_tc: true,
        indicates_cp: true,
        indicates_ce: false,
        ref_levels: &[(10, 64, 940, 897)],
    },
];

const REF_COLOR5_7_2016: &[(u32, u32, u32, u32)] = &[
    (10, 64, 940, 897),
    (12, 256, 3760, 3585),
    (16, 4096, 60160, 57345),
];

/// ST 2067-21:2016 §6.1.1–6.1.3 and §6.3 Table 7 (p8–10). §6.1.2 gives no
/// Coding Equations value for COLOR.6 (R'G'B' only), and Table 7 has no
/// COLOR.6 entry.
const SYSTEMS_2016: &[EditionColorSystem] = &[
    EditionColorSystem {
        name: "COLOR.4",
        transfer: TransferCharacteristic::XvYcc709,
        primaries: ColorPrimaries::Bt709,
        coding: Some(CodingEquations::Bt709),
        indicates_tc: true,
        indicates_cp: false,
        indicates_ce: false,
        ref_levels: REF_COLOR4,
    },
    EditionColorSystem {
        name: "COLOR.5",
        transfer: TransferCharacteristic::Bt2020,
        primaries: ColorPrimaries::Bt2020,
        coding: Some(CodingEquations::Bt2020Ncl),
        indicates_tc: true,
        indicates_cp: true,
        indicates_ce: true,
        ref_levels: REF_COLOR5_7_2016,
    },
    EditionColorSystem {
        name: "COLOR.6",
        transfer: TransferCharacteristic::PqSt2084,
        primaries: ColorPrimaries::P3D65,
        coding: None,
        indicates_tc: true,
        indicates_cp: true,
        indicates_ce: false,
        ref_levels: &[],
    },
    EditionColorSystem {
        name: "COLOR.7",
        transfer: TransferCharacteristic::PqSt2084,
        primaries: ColorPrimaries::Bt2020,
        coding: Some(CodingEquations::Bt2020Ncl),
        indicates_tc: true,
        indicates_cp: true,
        indicates_ce: true,
        ref_levels: REF_COLOR5_7_2016,
    },
];
