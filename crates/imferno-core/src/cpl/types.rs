//! CPL-domain types for SMPTE ST 2067-3.
//!
//! These types were previously defined in `imf-types` and are migrated here as
//! their canonical home, since they are defined by SMPTE ST 2067-3.

use crate::assetmap::{ImfTypeError, SmpteUl};
use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

#[cfg(feature = "wasm")]
use tsify::Tsify;

/// Helper: parse a UL string and return normalized bytes (byte 8 zeroed).
/// Used by `from_ul()` methods on enum types to implement version-agnostic matching.
fn parse_and_normalize_ul(ul: &str) -> Option<[u8; 16]> {
    SmpteUl::parse(ul).ok().map(|u| u.normalized().0)
}

// ─── EditRate ─────────────────────────────────────────────────────────────────

/// A rational frame/sample rate as specified in SMPTE ST 2067-3 §6.
///
/// In XML the value is a space-separated pair: `"60000 1001"`.
/// Common values: `24000/1001` (≈23.976), `24/1`, `25/1`, `30/1`,
/// `30000/1001` (≈29.97), `48000/1`, `96000/1`.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct EditRate {
    pub numerator: u32,
    pub denominator: u32,
}

impl EditRate {
    pub const fn new(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    /// Parse from the XML text form `"<numerator> <denominator>"`.
    pub fn parse(s: &str) -> Result<Self, ImfTypeError> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(ImfTypeError::InvalidEditRate(s.to_string()));
        }
        let numerator = parts[0]
            .parse::<u32>()
            .map_err(|_| ImfTypeError::InvalidEditRate(s.to_string()))?;
        let denominator = parts[1]
            .parse::<u32>()
            .map_err(|_| ImfTypeError::InvalidEditRate(s.to_string()))?;
        if denominator == 0 {
            return Err(ImfTypeError::InvalidEditRate(s.to_string()));
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }

    /// Returns the rate as a floating-point value.
    pub fn as_f64(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    /// Format for display, e.g. `"23.976"` or `"24"`.
    pub fn display(self) -> String {
        let f = self.as_f64();
        if (f.fract()).abs() < 0.001 {
            format!("{}", f as u32)
        } else {
            format!("{:.3}", f)
        }
    }
}

impl std::fmt::Display for EditRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

// ─── LanguageTag ──────────────────────────────────────────────────────────────

/// An RFC 5646 language tag, e.g. `"en"`, `"fr-CA"`, `"zh-Hant"`.
///
/// Validation is minimal (non-empty, trimmed). Full BCP 47 validation is out
/// of scope for the parser layer.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct LanguageTag(pub String);

impl LanguageTag {
    pub fn parse(s: &str) -> Result<Self, ImfTypeError> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ImfTypeError::InvalidLanguageTag(s.to_string()));
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// ─── Resolution ───────────────────────────────────────────────────────────────

/// Pixel dimensions.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

// ─── ContentKind ──────────────────────────────────────────────────────────────

/// IMF composition content kind per SMPTE ST 2067-3 §6.13.
///
/// The `Other` variant preserves unrecognised values so parsers remain
/// forward-compatible with new SMPTE registrations.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub enum ContentKind {
    Feature,
    Trailer,
    Test,
    Promo,
    Teaser,
    RatingBump,
    Advertisement,
    Episode,
    Short,
    Commercial,
    PublicServiceAnnouncement,
    /// Unrecognised value; the original string is preserved.
    Other(String),
}

impl ContentKind {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "feature" | "Feature" => Self::Feature,
            "trailer" | "Trailer" => Self::Trailer,
            "test" | "Test" => Self::Test,
            "promo" | "Promo" => Self::Promo,
            "teaser" | "Teaser" => Self::Teaser,
            "rating-bump" | "RatingBump" | "ratingbump" => Self::RatingBump,
            "advertisement" | "Advertisement" => Self::Advertisement,
            "episode" | "Episode" => Self::Episode,
            "short" | "Short" => Self::Short,
            "commercial" | "Commercial" => Self::Commercial,
            "public-service-announcement" | "PublicServiceAnnouncement" | "psa" | "PSA" => {
                Self::PublicServiceAnnouncement
            }
            other => Self::Other(other.to_string()),
        }
    }
}

impl std::fmt::Display for ContentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Feature => write!(f, "Feature"),
            Self::Trailer => write!(f, "Trailer"),
            Self::Test => write!(f, "Test"),
            Self::Promo => write!(f, "Promo"),
            Self::Teaser => write!(f, "Teaser"),
            Self::RatingBump => write!(f, "RatingBump"),
            Self::Advertisement => write!(f, "Advertisement"),
            Self::Episode => write!(f, "Episode"),
            Self::Short => write!(f, "Short"),
            Self::Commercial => write!(f, "Commercial"),
            Self::PublicServiceAnnouncement => write!(f, "PublicServiceAnnouncement"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

// ─── Compile-time UL helper ───────────────────────────────────────────────────

/// Compile-time helper to create UL byte arrays.
#[allow(clippy::too_many_arguments)]
const fn ul_bytes(
    b0: u8,
    b1: u8,
    b2: u8,
    b3: u8,
    b4: u8,
    b5: u8,
    b6: u8,
    b7: u8,
    b8: u8,
    b9: u8,
    b10: u8,
    b11: u8,
    b12: u8,
    b13: u8,
    b14: u8,
    b15: u8,
) -> [u8; 16] {
    [
        b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15,
    ]
}

// ─── ColorPrimaries ───────────────────────────────────────────────────────────

/// Color primaries SMPTE Universal Label, per ST 2067-21:2023 §7.2.4 and SMPTE Registers.
///
/// Discriminating bytes are in positions 13-16 of the UL (last 4 bytes of the
/// 16-byte label), with the common prefix `060e2b34.0401XXXX.04010101.03......`.
/// Byte 8 (the registry version) is masked per ST 298M (Architecture Decision 1).
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub enum ColorPrimaries {
    /// ITU-R BT.601-7 625-line system — COLOR.1 per ST 2067-21:2023 Table 3
    /// Canonical UL: `060e2b34.04010100.04010101.03020000` (byte 8 masked)
    Bt601_625,
    /// ITU-R BT.601-7 525-line system — COLOR.2 per ST 2067-21:2023 Table 3
    /// Canonical UL: `060e2b34.04010100.04010101.03010000` (byte 8 masked)
    Bt601_525,
    /// ITU-R BT.709 / SMPTE 274M — COLOR.3/COLOR.4 per ST 2067-21:2023 Table 3
    /// Canonical UL: `060e2b34.04010100.04010101.03030000` (byte 8 masked)
    Bt709,
    /// ITU-R BT.2020 / BT.2100 — COLOR.5/COLOR.7/COLOR.8 per ST 2067-21:2023 Table 3
    /// Canonical UL: `060e2b34.04010100.04010101.03040000` (byte 8 masked)
    Bt2020,
    /// DCI P3 (theatre projection, D60 white point)
    /// Canonical UL: `060e2b34.04010100.04010101.03050000` (byte 8 masked)
    DciP3,
    /// P3 D65 (display P3, D65 white point) — COLOR.6 per ST 2067-21:2023 Table 3
    /// Canonical UL: `060e2b34.04010100.04010101.03060000` (byte 8 masked)
    P3D65,
    /// Unrecognised UL; the original string is preserved.
    Unknown(String),
}

/// Canonical UL bytes (byte 8 zeroed) for ColorPrimaries matching.
/// Format: last 4 bytes of normalized 16-byte UL.
const CP_BT601_525: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x03, 0x01, 0x00, 0x00,
);
const CP_BT601_625: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x03, 0x02, 0x00, 0x00,
);
const CP_BT709: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x03, 0x03, 0x00, 0x00,
);
const CP_BT2020: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x03, 0x04, 0x00, 0x00,
);
const CP_DCI_P3: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x03, 0x05, 0x00, 0x00,
);
const CP_P3D65: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x03, 0x06, 0x00, 0x00,
);

impl ColorPrimaries {
    /// Map a SMPTE Color Primaries UL to a `ColorPrimaries` variant.
    ///
    /// Per ST 298M and Architecture Decision 1, byte 8 (registry version) is
    /// masked before matching. This makes the comparison version-agnostic.
    pub fn from_ul(ul: &str) -> Self {
        match parse_and_normalize_ul(ul) {
            Some(norm) => match norm {
                b if b == CP_BT601_525 => Self::Bt601_525,
                b if b == CP_BT601_625 => Self::Bt601_625,
                b if b == CP_BT709 => Self::Bt709,
                b if b == CP_BT2020 => Self::Bt2020,
                b if b == CP_DCI_P3 => Self::DciP3,
                b if b == CP_P3D65 => Self::P3D65,
                _ => Self::Unknown(ul.to_string()),
            },
            None => Self::Unknown(ul.to_string()),
        }
    }

    pub fn is_wide_gamut(&self) -> bool {
        matches!(self, Self::Bt2020 | Self::DciP3 | Self::P3D65)
    }
}

impl std::fmt::Display for ColorPrimaries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bt601_625 => write!(f, "BT.601-625"),
            Self::Bt601_525 => write!(f, "BT.601-525"),
            Self::Bt709 => write!(f, "BT.709"),
            Self::Bt2020 => write!(f, "BT.2020"),
            Self::DciP3 => write!(f, "DCI P3"),
            Self::P3D65 => write!(f, "P3 D65"),
            Self::Unknown(s) => write!(f, "Unknown({})", s),
        }
    }
}

// ─── TransferCharacteristic ───────────────────────────────────────────────────

/// Opto-electronic transfer function SMPTE Universal Label, per ST 2067-21:2023 §7.2.2.
///
/// Used in `<TransferCharacteristic>` elements in CPL EssenceDescriptors.
/// Byte 8 (registry version) is masked per ST 298M (Architecture Decision 1).
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub enum TransferCharacteristic {
    /// Linear (scene-referred, e.g. ACES)
    /// Canonical UL: `060e2b34.04010100.04010101.01010000`
    Linear,
    /// ITU-R BT.709 — COLOR.1/2/3 per ST 2067-21:2023 Table 3
    /// Canonical UL: `060e2b34.04010100.04010101.01020000`
    Bt709,
    /// SMPTE 240M (legacy 1080i)
    /// Canonical UL: `060e2b34.04010100.04010101.01030000`
    Smpte240M,
    /// xvYCC 709 — COLOR.4 per ST 2067-21:2023 Table 3
    /// Canonical UL: `060e2b34.04010100.04010101.01080000`
    XvYcc709,
    /// ITU-R BT.2020 (Annex A) — COLOR.5 per ST 2067-21:2023 Table 3
    /// Canonical UL: `060e2b34.04010100.04010101.01090000`
    Bt2020,
    /// SMPTE ST 2084 (PQ / Perceptual Quantiser) — COLOR.6/7 per ST 2067-21:2023 Table 3
    /// Canonical UL: `060e2b34.04010100.04010101.010a0000`
    PqSt2084,
    /// ITU-R BT.2100 Hybrid Log-Gamma — COLOR.8 per ST 2067-21:2023 Table 3
    /// Canonical UL: `060e2b34.04010100.04010101.010b0000`
    Hlg,
    /// Unrecognised UL; the original string is preserved.
    Unknown(String),
}

/// Canonical UL bytes (byte 8 zeroed) for TransferCharacteristic matching.
const TC_LINEAR: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00,
);
const TC_BT709: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x01, 0x02, 0x00, 0x00,
);
const TC_240M: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x01, 0x03, 0x00, 0x00,
);
const TC_XVYCC709: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x01, 0x08, 0x00, 0x00,
);
const TC_BT2020: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x01, 0x09, 0x00, 0x00,
);
const TC_PQ: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x01, 0x0a, 0x00, 0x00,
);
const TC_HLG: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x01, 0x0b, 0x00, 0x00,
);

impl TransferCharacteristic {
    /// Map a SMPTE Transfer Characteristic UL to a variant.
    ///
    /// Per ST 298M and Architecture Decision 1, byte 8 is masked before matching.
    pub fn from_ul(ul: &str) -> Self {
        match parse_and_normalize_ul(ul) {
            Some(norm) => match norm {
                b if b == TC_LINEAR => Self::Linear,
                b if b == TC_BT709 => Self::Bt709,
                b if b == TC_240M => Self::Smpte240M,
                b if b == TC_XVYCC709 => Self::XvYcc709,
                b if b == TC_BT2020 => Self::Bt2020,
                b if b == TC_PQ => Self::PqSt2084,
                b if b == TC_HLG => Self::Hlg,
                _ => Self::Unknown(ul.to_string()),
            },
            None => Self::Unknown(ul.to_string()),
        }
    }

    /// Returns `true` for HDR transfer functions (PQ and HLG).
    pub fn is_hdr(&self) -> bool {
        matches!(self, Self::PqSt2084 | Self::Hlg)
    }
}

impl std::fmt::Display for TransferCharacteristic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Linear => write!(f, "Linear"),
            Self::Bt709 => write!(f, "BT.709"),
            Self::Smpte240M => write!(f, "SMPTE 240M"),
            Self::XvYcc709 => write!(f, "xvYCC 709"),
            Self::Bt2020 => write!(f, "BT.2020"),
            Self::PqSt2084 => write!(f, "SMPTE ST 2084 (PQ)"),
            Self::Hlg => write!(f, "HLG"),
            Self::Unknown(s) => write!(f, "Unknown({})", s),
        }
    }
}

// ─── VideoCodec ───────────────────────────────────────────────────────────────

/// Video codec identified by SMPTE PictureEssenceCoding Universal Label.
///
/// Per ST 2067-21:2023 §7.2.5 and Annex F. The full UL-to-codec mapping lives here.
/// Called once at parse time; no runtime string decoding in downstream crates.
///
/// Note: VideoCodec ULs have varying item-specific bytes, so we match on byte
/// patterns rather than exact 16-byte comparisons. Byte 8 is still masked.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub enum VideoCodec {
    /// JPEG 2000 — generic/unspecified profile (ST 422 §A, byte [14]=0x01 without
    /// a recognized broadcast sub-level, or other unspecified node UL).
    /// Not a recognized IMF Application Profile #2E profile.
    Jpeg2000,
    /// JPEG 2000 — IMF 2K profile (ST 422 §A.2, byte [14]=0x02).
    /// Supports stored widths ≤ 2048 per ST 2067-21.
    Jpeg2000Imf2k,
    /// JPEG 2000 — IMF 4K profile (ST 422 §A.3, byte [14]=0x03).
    /// Supports stored widths 2049–4096 per ST 2067-21.
    Jpeg2000Imf4k,
    /// JPEG 2000 — Broadcast Contribution profile (ST 422 §A.1, byte [14]=0x01,
    /// sub-levels 0x11–0x17). Supports stored widths ≤ 3840 per ST 2067-21.
    Jpeg2000Broadcast,
    /// JPEG 2000 Part 15 — High Throughput (ISO 15444-15, ST 2067-21 §6.2.5).
    /// Only allowed in App #2E from spec year 2021 onward.
    Jpeg2000Ht,
    /// VC-5 (SMPTE ST 2117)
    Vc5,
    /// MPEG-2 (ISO 13818-2)
    Mpeg2,
    /// H.264 / AVC (ISO 14496-10)
    H264,
    /// H.265 / HEVC (ISO 23008-2)
    H265,
    /// Apple ProRes (used in some IMF Application profiles)
    ProRes,
    /// AV1 (AOMedia Video 1)
    Av1,
    /// Unrecognised UL; the original string is preserved.
    Unknown(String),
}

/// Known PictureEssenceCoding ULs (byte 8 zeroed, last 4 bytes discriminating).
/// Bytes 13-16 of normalized UL (after common prefix 060e2b34.040101XX.04010202.)
///
/// Sources: SMPTE ST 422, ST 2067-21:2023 §6.2.5 + Annex G, test corpus verification.
/// Profile classification mirrors Photon JPEG2000.java (isIMF4KProfile / isIMF2KProfile /
/// isBroadcastProfile / isAPP2HT). ULs not matching a recognized App2E profile stay
/// as Jpeg2000 (generic), which the App2E validator then rejects.
const CODEC_TABLE: &[([u8; 4], VideoCodec)] = &[
    // ── JPEG 2000 generic / unspecified profile ───────────────────────────────
    // byte [14]=0x01, byte [15]=0x00 — J2K node (no specific profile sub-level)
    ([0x03, 0x01, 0x01, 0x00], VideoCodec::Jpeg2000),
    // Note: byte [14]=0x01, byte [15]=0x01 ("Profile 0/1") is NOT a recognized BCP
    // sub-level (valid levels are 0x11–0x17). Photon rejects it as "Invalid JPEG 2000
    // Profile". It is intentionally absent from this table → maps to Unknown.
    //
    // ── JPEG 2000 Broadcast Contribution profile (ST 422 §A.1) ───────────────
    // byte [14]=0x01, byte [15]=0x11–0x17 (single-tile and multi-tile reversible)
    ([0x03, 0x01, 0x01, 0x11], VideoCodec::Jpeg2000Broadcast), // BCP Single Tile Level 1
    ([0x03, 0x01, 0x01, 0x12], VideoCodec::Jpeg2000Broadcast), // BCP Single Tile Level 2
    ([0x03, 0x01, 0x01, 0x13], VideoCodec::Jpeg2000Broadcast), // BCP Single Tile Level 3
    ([0x03, 0x01, 0x01, 0x14], VideoCodec::Jpeg2000Broadcast), // BCP Single Tile Level 4
    ([0x03, 0x01, 0x01, 0x15], VideoCodec::Jpeg2000Broadcast), // BCP Single Tile Level 5
    ([0x03, 0x01, 0x01, 0x16], VideoCodec::Jpeg2000Broadcast), // BCP Multi-tile Reversible Level 6
    ([0x03, 0x01, 0x01, 0x17], VideoCodec::Jpeg2000Broadcast), // BCP Multi-tile Reversible Level 7
    //
    // ── JPEG 2000 IMF 2K profile (ST 422 §A.2) ───────────────────────────────
    // byte [14]=0x02 — 2K lossy and reversible sub-levels
    ([0x03, 0x01, 0x02, 0x03], VideoCodec::Jpeg2000Imf2k), // M1S1
    ([0x03, 0x01, 0x02, 0x05], VideoCodec::Jpeg2000Imf2k), // M2S1
    ([0x03, 0x01, 0x02, 0x07], VideoCodec::Jpeg2000Imf2k), // M3S1
    ([0x03, 0x01, 0x02, 0x09], VideoCodec::Jpeg2000Imf2k), // M4S1
    ([0x03, 0x01, 0x02, 0x0a], VideoCodec::Jpeg2000Imf2k), // M4S2
    ([0x03, 0x01, 0x02, 0x0c], VideoCodec::Jpeg2000Imf2k), // M5S1
    ([0x03, 0x01, 0x02, 0x0d], VideoCodec::Jpeg2000Imf2k), // M5S2
    ([0x03, 0x01, 0x02, 0x0e], VideoCodec::Jpeg2000Imf2k), // M5S3
    ([0x03, 0x01, 0x02, 0x10], VideoCodec::Jpeg2000Imf2k), // M6S1
    ([0x03, 0x01, 0x02, 0x11], VideoCodec::Jpeg2000Imf2k), // M6S2
    ([0x03, 0x01, 0x02, 0x12], VideoCodec::Jpeg2000Imf2k), // M6S3 (observed in test corpus)
    ([0x03, 0x01, 0x02, 0x13], VideoCodec::Jpeg2000Imf2k), // M6S4
    ([0x03, 0x01, 0x05, 0x02], VideoCodec::Jpeg2000Imf2k), // M1S0 reversible
    ([0x03, 0x01, 0x05, 0x04], VideoCodec::Jpeg2000Imf2k), // M2S0 reversible
    ([0x03, 0x01, 0x05, 0x06], VideoCodec::Jpeg2000Imf2k), // M3S0 reversible
    ([0x03, 0x01, 0x05, 0x08], VideoCodec::Jpeg2000Imf2k), // M4S0 reversible
    ([0x03, 0x01, 0x05, 0x0b], VideoCodec::Jpeg2000Imf2k), // M5S0 reversible
    ([0x03, 0x01, 0x05, 0x0f], VideoCodec::Jpeg2000Imf2k), // M6S0 reversible
    //
    // ── JPEG 2000 IMF 4K profile (ST 422 §A.3) ───────────────────────────────
    // byte [14]=0x03 — 4K lossy sub-levels
    ([0x03, 0x01, 0x03, 0x03], VideoCodec::Jpeg2000Imf4k), // M1S1
    ([0x03, 0x01, 0x03, 0x05], VideoCodec::Jpeg2000Imf4k), // M2S1
    ([0x03, 0x01, 0x03, 0x07], VideoCodec::Jpeg2000Imf4k), // M3S1
    ([0x03, 0x01, 0x03, 0x09], VideoCodec::Jpeg2000Imf4k), // M4S1
    ([0x03, 0x01, 0x03, 0x0a], VideoCodec::Jpeg2000Imf4k), // M4S2
    ([0x03, 0x01, 0x03, 0x0c], VideoCodec::Jpeg2000Imf4k), // M5S1
    ([0x03, 0x01, 0x03, 0x0d], VideoCodec::Jpeg2000Imf4k), // M5S2
    ([0x03, 0x01, 0x03, 0x0e], VideoCodec::Jpeg2000Imf4k), // M5S3
    ([0x03, 0x01, 0x03, 0x10], VideoCodec::Jpeg2000Imf4k), // M6S1 (PHDR variant in corpus)
    ([0x03, 0x01, 0x03, 0x11], VideoCodec::Jpeg2000Imf4k), // M6S2
    ([0x03, 0x01, 0x03, 0x12], VideoCodec::Jpeg2000Imf4k), // M6S3 (observed in test corpus)
    ([0x03, 0x01, 0x03, 0x13], VideoCodec::Jpeg2000Imf4k), // M6S4
    ([0x03, 0x01, 0x03, 0x15], VideoCodec::Jpeg2000Imf4k), // M7S1
    ([0x03, 0x01, 0x03, 0x16], VideoCodec::Jpeg2000Imf4k), // M7S2
    ([0x03, 0x01, 0x03, 0x17], VideoCodec::Jpeg2000Imf4k), // M7S3
    ([0x03, 0x01, 0x03, 0x18], VideoCodec::Jpeg2000Imf4k), // M7S4
    ([0x03, 0x01, 0x03, 0x19], VideoCodec::Jpeg2000Imf4k), // M7S5 (observed in test corpus)
    ([0x03, 0x01, 0x03, 0x1b], VideoCodec::Jpeg2000Imf4k), // M8S1
    ([0x03, 0x01, 0x03, 0x1c], VideoCodec::Jpeg2000Imf4k), // M8S2
    ([0x03, 0x01, 0x03, 0x1d], VideoCodec::Jpeg2000Imf4k), // M8S3
    ([0x03, 0x01, 0x03, 0x1e], VideoCodec::Jpeg2000Imf4k), // M8S4
    ([0x03, 0x01, 0x03, 0x1f], VideoCodec::Jpeg2000Imf4k), // M8S5
    ([0x03, 0x01, 0x03, 0x20], VideoCodec::Jpeg2000Imf4k), // M8S6
    ([0x03, 0x01, 0x06, 0x02], VideoCodec::Jpeg2000Imf4k), // 4K M1S0 reversible
    ([0x03, 0x01, 0x06, 0x04], VideoCodec::Jpeg2000Imf4k), // 4K M2S0 reversible
    ([0x03, 0x01, 0x06, 0x06], VideoCodec::Jpeg2000Imf4k), // 4K M3S0 reversible
    ([0x03, 0x01, 0x06, 0x08], VideoCodec::Jpeg2000Imf4k), // 4K M4S0 reversible
    ([0x03, 0x01, 0x06, 0x0b], VideoCodec::Jpeg2000Imf4k), // 4K M5S0 reversible
    ([0x03, 0x01, 0x06, 0x0f], VideoCodec::Jpeg2000Imf4k), // 4K M6S0 reversible
    ([0x03, 0x01, 0x06, 0x14], VideoCodec::Jpeg2000Imf4k), // 4K M7S0 reversible
    ([0x03, 0x01, 0x06, 0x1a], VideoCodec::Jpeg2000Imf4k), // 4K M8S0 reversible
    //
    // Note: byte [14]=0x07 ("8K") is not a recognized App2E profile in Photon.
    // It is intentionally absent → maps to Unknown → fails is_jpeg2000_family().
    //
    // ── JPEG 2000 HT (ISO 15444-15, ST 2067-21 §6.2.5 + Annex I) ──────────
    ([0x03, 0x01, 0x08, 0x00], VideoCodec::Jpeg2000Ht), // HT-J2K generic
    ([0x03, 0x01, 0x08, 0x01], VideoCodec::Jpeg2000Ht), // HT-J2K (ST 2067-21 §6.2.5)
    // ── VC-5 (ST 2117) ──────────────────────────────────────────────────────
    ([0x03, 0x05, 0x00, 0x00], VideoCodec::Vc5), // VC-5 (observed in test corpus)
    // ── MPEG-2 (ISO 13818-2) ────────────────────────────────────────────────
    ([0x01, 0x02, 0x01, 0x01], VideoCodec::Mpeg2),
    ([0x01, 0x02, 0x01, 0x02], VideoCodec::Mpeg2),
    ([0x03, 0x02, 0x00, 0x00], VideoCodec::Mpeg2),
    // ── H.264 / AVC (ISO 14496-10) ──────────────────────────────────────────
    ([0x01, 0x02, 0x01, 0x0a], VideoCodec::H264),
    ([0x01, 0x02, 0x01, 0x0b], VideoCodec::H264),
    // ── H.265 / HEVC (ISO 23008-2) ──────────────────────────────────────────
    ([0x01, 0x02, 0x01, 0x10], VideoCodec::H265),
    // ── Apple ProRes (SMPTE RDD 44) ──────────────────────────────────────────
    // Bytes 13-16 of PictureEssenceCoding UL: 060e2b34.0401010d.04010202.0306XXYY
    ([0x03, 0x06, 0x03, 0x00], VideoCodec::ProRes), // Generic
    ([0x03, 0x06, 0x03, 0x01], VideoCodec::ProRes), // ProRes 422 Proxy
    ([0x03, 0x06, 0x03, 0x02], VideoCodec::ProRes), // ProRes 422 LT
    ([0x03, 0x06, 0x03, 0x03], VideoCodec::ProRes), // ProRes 422
    ([0x03, 0x06, 0x03, 0x04], VideoCodec::ProRes), // ProRes 422 HQ
    ([0x03, 0x06, 0x03, 0x05], VideoCodec::ProRes), // ProRes 4444
    ([0x03, 0x06, 0x03, 0x06], VideoCodec::ProRes), // ProRes 4444 XQ
];

impl VideoCodec {
    /// Returns `true` if this codec is in the JPEG 2000 family (standard or HT).
    ///
    /// Includes all recognized J2K profile variants (Generic, IMF 2K, IMF 4K, Broadcast, HT).
    /// ST 2067-21 §6.2.5 requires JPEG 2000; the App2E validator further restricts which
    /// profile sub-variants are allowed.
    pub fn is_jpeg2000_family(&self) -> bool {
        matches!(
            self,
            Self::Jpeg2000
                | Self::Jpeg2000Imf2k
                | Self::Jpeg2000Imf4k
                | Self::Jpeg2000Broadcast
                | Self::Jpeg2000Ht
        )
    }

    /// Map a SMPTE PictureEssenceCoding UL to a `VideoCodec` variant.
    ///
    /// Byte 8 (registry version) is masked per ST 298M.
    /// Matching is table-based against normalized last-4-byte patterns.
    pub fn from_ul(ul: &str) -> Self {
        let norm = match parse_and_normalize_ul(ul) {
            Some(n) => n,
            None => return Self::Unknown(ul.to_string()),
        };

        // Verify the UL is in the Picture Coding parameter space
        // Bytes 9-12 should be [04, 01, 02, 02]
        if norm[8..12] != [0x04, 0x01, 0x02, 0x02] {
            return Self::Unknown(ul.to_string());
        }

        // Match last 4 bytes against the codec table
        let tail = [norm[12], norm[13], norm[14], norm[15]];
        for (pattern, codec) in CODEC_TABLE {
            if tail == *pattern {
                return codec.clone();
            }
        }

        Self::Unknown(ul.to_string())
    }
}

impl std::fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Jpeg2000 => write!(f, "JPEG 2000"),
            Self::Jpeg2000Imf2k => write!(f, "JPEG 2000 IMF 2K"),
            Self::Jpeg2000Imf4k => write!(f, "JPEG 2000 IMF 4K"),
            Self::Jpeg2000Broadcast => write!(f, "JPEG 2000 Broadcast"),
            Self::Jpeg2000Ht => write!(f, "JPEG 2000 HT"),
            Self::Vc5 => write!(f, "VC-5"),
            Self::Mpeg2 => write!(f, "MPEG-2"),
            Self::H264 => write!(f, "H.264/AVC"),
            Self::H265 => write!(f, "H.265/HEVC"),
            Self::ProRes => write!(f, "ProRes"),
            Self::Av1 => write!(f, "AV1"),
            Self::Unknown(s) => write!(f, "Unknown({})", s),
        }
    }
}

// ─── CodingEquations ─────────────────────────────────────────────────────────

/// Coding equations SMPTE Universal Label, per ST 2067-21:2023 §7.2.3.
///
/// Defines the Y'C'BC'R matrix coefficients. Required for CDCI descriptors;
/// R'G'B' descriptors (COLOR.6) do not use CodingEquations.
/// Byte 8 (registry version) is masked per ST 298M (Architecture Decision 1).
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub enum CodingEquations {
    /// ITU-R BT.601 — COLOR.1/COLOR.2 per ST 2067-21:2023 Table 3
    /// Canonical UL: `060e2b34.04010100.04010101.02010000`
    Bt601,
    /// ITU-R BT.709 — COLOR.3/COLOR.4 per ST 2067-21:2023 Table 3
    /// Canonical UL: `060e2b34.04010100.04010101.02020000`
    Bt709,
    /// ITU-R BT.2020 Non-Constant Luminance (Annex D) — COLOR.5/7/8 per ST 2067-21:2023 Table 3
    /// Canonical UL: `060e2b34.04010100.04010101.02060000`
    Bt2020Ncl,
    /// Unrecognised UL; the original string is preserved.
    Unknown(String),
}

/// Canonical UL bytes (byte 8 zeroed) for CodingEquations matching.
const CE_BT601: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x02, 0x01, 0x00, 0x00,
);
const CE_BT709: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x02, 0x02, 0x00, 0x00,
);
const CE_BT2020NCL: [u8; 16] = ul_bytes(
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x00, 0x04, 0x01, 0x01, 0x01, 0x02, 0x06, 0x00, 0x00,
);

impl CodingEquations {
    /// Map a SMPTE Coding Equations UL to a variant.
    ///
    /// Per ST 298M and Architecture Decision 1, byte 8 is masked before matching.
    pub fn from_ul(ul: &str) -> Self {
        match parse_and_normalize_ul(ul) {
            Some(norm) => match norm {
                b if b == CE_BT601 => Self::Bt601,
                b if b == CE_BT709 => Self::Bt709,
                b if b == CE_BT2020NCL => Self::Bt2020Ncl,
                _ => Self::Unknown(ul.to_string()),
            },
            None => Self::Unknown(ul.to_string()),
        }
    }
}

impl std::fmt::Display for CodingEquations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bt601 => write!(f, "BT.601"),
            Self::Bt709 => write!(f, "BT.709"),
            Self::Bt2020Ncl => write!(f, "BT.2020 NCL"),
            Self::Unknown(s) => write!(f, "Unknown({})", s),
        }
    }
}

// ─── McaTagSymbol ─────────────────────────────────────────────────────────────

/// MCA (Multi-Channel Audio) tag symbol per SMPTE ST 377-4.
///
/// Used in `<MCATagSymbol>` in `SoundfieldGroupLabelSubDescriptor` and
/// `AudioChannelLabelSubDescriptor`.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub enum McaTagSymbol {
    // ── Soundfield group labels ──────────────────────────────────────────────
    /// 5.1 Surround
    Sg51,
    /// 7.1 Surround
    Sg71,
    /// 7.1 DS (Dolby Surround)
    Sg71Ds,
    /// Stereo / Lt-Rt matrixed
    SgSt,
    /// Mono
    SgMono,
    /// IAB (Immersive Audio Bitstream / Dolby Atmos)
    Iab,
    // ── Audio channel labels ─────────────────────────────────────────────────
    Left,
    Right,
    Center,
    Lfe,
    LeftSurround,
    RightSurround,
    LeftSideSurround,
    RightSideSurround,
    LeftRearSurround,
    RightRearSurround,
    /// Unrecognised; the original string is preserved.
    Other(String),
}

impl McaTagSymbol {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "sg51" | "Sg51" | "51" => Self::Sg51,
            "sg71" | "Sg71" | "71" => Self::Sg71,
            "sg71DS" | "Sg71Ds" => Self::Sg71Ds,
            "sgST" | "sgSt" | "SgSt" => Self::SgSt,
            "sgMono" | "SgMono" => Self::SgMono,
            "IAB" | "iab" => Self::Iab,
            "L" => Self::Left,
            "R" => Self::Right,
            "C" => Self::Center,
            "LFE" | "LFE1" => Self::Lfe,
            "Ls" | "LS" => Self::LeftSurround,
            "Rs" | "RS" => Self::RightSurround,
            "Lss" | "LSS" => Self::LeftSideSurround,
            "Rss" | "RSS" => Self::RightSideSurround,
            "Lrs" | "LRS" => Self::LeftRearSurround,
            "Rrs" | "RRS" => Self::RightRearSurround,
            other => Self::Other(other.to_string()),
        }
    }
}

impl std::fmt::Display for McaTagSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sg51 => write!(f, "5.1"),
            Self::Sg71 => write!(f, "7.1"),
            Self::Sg71Ds => write!(f, "7.1 DS"),
            Self::SgSt => write!(f, "Stereo"),
            Self::SgMono => write!(f, "Mono"),
            Self::Iab => write!(f, "IAB"),
            Self::Left => write!(f, "L"),
            Self::Right => write!(f, "R"),
            Self::Center => write!(f, "C"),
            Self::Lfe => write!(f, "LFE"),
            Self::LeftSurround => write!(f, "Ls"),
            Self::RightSurround => write!(f, "Rs"),
            Self::LeftSideSurround => write!(f, "Lss"),
            Self::RightSideSurround => write!(f, "Rss"),
            Self::LeftRearSurround => write!(f, "Lrs"),
            Self::RightRearSurround => write!(f, "Rrs"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

// ─── MarkerLabel ──────────────────────────────────────────────────────────────

/// CPL composition marker labels per SMPTE ST 2067-3 §7.4.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub enum MarkerLabel {
    /// First Frame Of Content
    Ffoc,
    /// Last Frame Of Content
    Lfoc,
    /// First Frame After Credits
    Ffac,
    /// Last Frame After Credits
    Lfac,
    /// First Frame of Moving Content (title card start)
    Ffmc,
    /// Last Frame of Moving Content (title card end)
    Lfmc,
    /// First Frame of Title
    Ffhc,
    /// Last Frame of Title
    Lfhc,
    /// Unrecognised; the original string is preserved.
    Other(String),
}

impl MarkerLabel {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "FFOC" => Self::Ffoc,
            "LFOC" => Self::Lfoc,
            "FFAC" => Self::Ffac,
            "LFAC" => Self::Lfac,
            "FFMC" => Self::Ffmc,
            "LFMC" => Self::Lfmc,
            "FFHC" => Self::Ffhc,
            "LFHC" => Self::Lfhc,
            other => Self::Other(other.to_string()),
        }
    }
}

impl std::fmt::Display for MarkerLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ffoc => write!(f, "FFOC"),
            Self::Lfoc => write!(f, "LFOC"),
            Self::Ffac => write!(f, "FFAC"),
            Self::Lfac => write!(f, "LFAC"),
            Self::Ffmc => write!(f, "FFMC"),
            Self::Lfmc => write!(f, "LFMC"),
            Self::Ffhc => write!(f, "FFHC"),
            Self::Lfhc => write!(f, "LFHC"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

// ─── Namespace / Spec Version ─────────────────────────────────────────────────

/// The detected SMPTE spec version of a CPL document, derived from its root xmlns.
///
/// Different spec versions have different schema constraints. The namespace URI
/// in the root element is the authoritative signal for which spec edition applies.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub enum CplNamespace {
    /// SMPTE ST 2067-3:2013 — `http://www.smpte-ra.org/schemas/2067-3/2013`
    Smpte2067_3_2013,
    /// SMPTE ST 2067-3:2016 — `http://www.smpte-ra.org/schemas/2067-3/2016`
    Smpte2067_3_2016,
    /// SMPTE ST 2067-3:2020 — `http://www.smpte-ra.org/ns/2067-3/2020`
    /// Note: `schemas` → `ns` path change in 2020 editions.
    Smpte2067_3_2020,
    /// DCI era — `http://www.smpte-ra.org/schemas/429-7/2006/CPL`
    Dci429_7,
    /// Unrecognised namespace; the original URI is preserved.
    Unknown(String),
}

impl CplNamespace {
    /// Detect CPL spec version from a namespace URI.
    pub fn from_uri(uri: &str) -> Self {
        match uri.trim() {
            "http://www.smpte-ra.org/schemas/2067-3/2013" => Self::Smpte2067_3_2013,
            "http://www.smpte-ra.org/schemas/2067-3/2016" => Self::Smpte2067_3_2016,
            "http://www.smpte-ra.org/ns/2067-3/2020" => Self::Smpte2067_3_2020,
            "http://www.smpte-ra.org/schemas/429-7/2006/CPL" => Self::Dci429_7,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Returns the normative spec document identifier (e.g., "ST 2067-3:2020").
    pub fn spec_id(&self) -> &str {
        match self {
            Self::Smpte2067_3_2013 => "ST 2067-3:2013",
            Self::Smpte2067_3_2016 => "ST 2067-3:2016",
            Self::Smpte2067_3_2020 => "ST 2067-3:2020",
            Self::Dci429_7 => "ST 429-7:2006",
            Self::Unknown(_) => "Unknown",
        }
    }

    /// Returns the spec edition year for known namespaces.
    pub fn year(&self) -> Option<u16> {
        match self {
            Self::Smpte2067_3_2013 => Some(2013),
            Self::Smpte2067_3_2016 => Some(2016),
            Self::Smpte2067_3_2020 => Some(2020),
            Self::Dci429_7 => Some(2006),
            Self::Unknown(_) => None,
        }
    }
}

impl Default for CplNamespace {
    fn default() -> Self {
        Self::Smpte2067_3_2013
    }
}

impl std::fmt::Display for CplNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Smpte2067_3_2013 => write!(f, "http://www.smpte-ra.org/schemas/2067-3/2013"),
            Self::Smpte2067_3_2016 => write!(f, "http://www.smpte-ra.org/schemas/2067-3/2016"),
            Self::Smpte2067_3_2020 => write!(f, "http://www.smpte-ra.org/ns/2067-3/2020"),
            Self::Dci429_7 => write!(f, "http://www.smpte-ra.org/schemas/429-7/2006/CPL"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // ── EditRate ──────────────────────────────────────────────────────────────

    /// SMPTE ST 2067-3 §6: EditRate is a rational number expressed as "num den".
    #[test]
    fn edit_rate_parse_valid() {
        let r = EditRate::parse("24000 1001").unwrap();
        assert_eq!(r.numerator, 24000);
        assert_eq!(r.denominator, 1001);
    }

    #[test]
    fn edit_rate_parse_integer() {
        let r = EditRate::parse("25 1").unwrap();
        assert_eq!(r.numerator, 25);
        assert_eq!(r.denominator, 1);
        assert_eq!(r.display(), "25");
    }

    #[test]
    fn edit_rate_parse_invalid() {
        assert!(EditRate::parse("24").is_err());
        assert!(EditRate::parse("24 0").is_err()); // zero denominator
        assert!(EditRate::parse("").is_err());
    }

    #[test]
    fn edit_rate_display() {
        let r = EditRate::new(24000, 1001);
        assert_eq!(r.to_string(), "24000/1001");
        assert_eq!(r.display(), "23.976");
    }

    // ── LanguageTag ───────────────────────────────────────────────────────────

    /// RFC 5646: language tags must be non-empty.
    #[test]
    fn language_tag_parse_valid() {
        let t = LanguageTag::parse("en").unwrap();
        assert_eq!(t.as_str(), "en");
    }

    #[test]
    fn language_tag_parse_trims_whitespace() {
        let t = LanguageTag::parse("  fr-CA  ").unwrap();
        assert_eq!(t.as_str(), "fr-CA");
    }

    #[test]
    fn language_tag_parse_empty_fails() {
        assert!(LanguageTag::parse("").is_err());
        assert!(LanguageTag::parse("   ").is_err());
    }

    // ── ContentKind ───────────────────────────────────────────────────────────

    #[test]
    fn content_kind_parse_known() {
        assert_eq!(ContentKind::parse("feature"), ContentKind::Feature);
        assert_eq!(ContentKind::parse("Feature"), ContentKind::Feature);
        assert_eq!(ContentKind::parse("trailer"), ContentKind::Trailer);
        assert_eq!(ContentKind::parse("episode"), ContentKind::Episode);
    }

    #[test]
    fn content_kind_parse_unknown() {
        assert_eq!(
            ContentKind::parse("custom-kind"),
            ContentKind::Other("custom-kind".to_string())
        );
    }

    // ── ColorPrimaries ────────────────────────────────────────────────────────

    /// SMPTE ST 2067-21:2023 §7.2.4: ColorPrimaries ULs map to named primaries.
    #[test]
    fn color_primaries_bt709() {
        let cp = ColorPrimaries::from_ul("urn:smpte:ul:060e2b34.04010101.04010101.03030000");
        assert_eq!(cp, ColorPrimaries::Bt709);
    }

    #[test]
    fn color_primaries_bt2020() {
        let cp = ColorPrimaries::from_ul("urn:smpte:ul:060e2b34.04010101.04010101.03040000");
        assert_eq!(cp, ColorPrimaries::Bt2020);
    }

    #[test]
    fn color_primaries_bt601_625() {
        let cp = ColorPrimaries::from_ul("urn:smpte:ul:060e2b34.04010101.04010101.03020000");
        assert_eq!(cp, ColorPrimaries::Bt601_625);
    }

    #[test]
    fn color_primaries_bt601_525() {
        let cp = ColorPrimaries::from_ul("urn:smpte:ul:060e2b34.04010101.04010101.03010000");
        assert_eq!(cp, ColorPrimaries::Bt601_525);
    }

    #[test]
    fn color_primaries_p3d65() {
        let cp = ColorPrimaries::from_ul("urn:smpte:ul:060e2b34.04010101.04010101.03060000");
        assert_eq!(cp, ColorPrimaries::P3D65);
    }

    #[test]
    fn color_primaries_unknown_ul() {
        let cp = ColorPrimaries::from_ul("urn:smpte:ul:060e2b34.04010101.04010101.03ff0000");
        assert!(matches!(cp, ColorPrimaries::Unknown(_)));
    }

    /// Architecture Decision 1: byte 8 (registry version) is masked — version 01 == version 04.
    #[test]
    fn color_primaries_byte8_masked() {
        let v01 = ColorPrimaries::from_ul("urn:smpte:ul:060e2b34.04010101.04010101.03030000");
        let v04 = ColorPrimaries::from_ul("urn:smpte:ul:060e2b34.04010104.04010101.03030000");
        assert_eq!(v01, v04);
        assert_eq!(v01, ColorPrimaries::Bt709);
    }

    // ── TransferCharacteristic ────────────────────────────────────────────────

    /// SMPTE ST 2067-21:2023 §7.2.2: TransferCharacteristic ULs.
    #[test]
    fn transfer_characteristic_pq() {
        let tc =
            TransferCharacteristic::from_ul("urn:smpte:ul:060e2b34.04010101.04010101.010a0000");
        assert_eq!(tc, TransferCharacteristic::PqSt2084);
        assert!(tc.is_hdr());
    }

    #[test]
    fn transfer_characteristic_hlg() {
        let tc =
            TransferCharacteristic::from_ul("urn:smpte:ul:060e2b34.04010101.04010101.010b0000");
        assert_eq!(tc, TransferCharacteristic::Hlg);
        assert!(tc.is_hdr());
    }

    #[test]
    fn transfer_characteristic_bt709() {
        let tc =
            TransferCharacteristic::from_ul("urn:smpte:ul:060e2b34.04010101.04010101.01020000");
        assert_eq!(tc, TransferCharacteristic::Bt709);
        assert!(!tc.is_hdr());
    }

    // ── CodingEquations ───────────────────────────────────────────────────────

    /// SMPTE ST 2067-21:2023 §7.2.3: CodingEquations ULs.
    #[test]
    fn coding_equations_bt601() {
        let ce = CodingEquations::from_ul("urn:smpte:ul:060e2b34.04010101.04010101.02010000");
        assert_eq!(ce, CodingEquations::Bt601);
    }

    #[test]
    fn coding_equations_bt709() {
        let ce = CodingEquations::from_ul("urn:smpte:ul:060e2b34.04010101.04010101.02020000");
        assert_eq!(ce, CodingEquations::Bt709);
    }

    #[test]
    fn coding_equations_bt2020ncl() {
        let ce = CodingEquations::from_ul("urn:smpte:ul:060e2b34.04010101.04010101.02060000");
        assert_eq!(ce, CodingEquations::Bt2020Ncl);
    }

    // ── VideoCodec ────────────────────────────────────────────────────────────

    /// SMPTE ST 422 §A.3: 4K IMF profile UL maps to Jpeg2000Imf4k.
    #[test]
    fn video_codec_jpeg2000_imf4k() {
        let vc = VideoCodec::from_ul("urn:smpte:ul:060e2b34.0401010d.04010202.03010312");
        assert_eq!(vc, VideoCodec::Jpeg2000Imf4k);
        assert!(vc.is_jpeg2000_family());
    }

    /// SMPTE ST 422 §A.2: 2K IMF profile UL maps to Jpeg2000Imf2k.
    #[test]
    fn video_codec_jpeg2000_imf2k() {
        let vc = VideoCodec::from_ul("urn:smpte:ul:060e2b34.0401010d.04010202.03010212");
        assert_eq!(vc, VideoCodec::Jpeg2000Imf2k);
        assert!(vc.is_jpeg2000_family());
    }

    /// SMPTE ST 422 §A.1: BCP Level 5 UL maps to Jpeg2000Broadcast.
    #[test]
    fn video_codec_jpeg2000_broadcast() {
        let vc = VideoCodec::from_ul("urn:smpte:ul:060e2b34.04010107.04010202.03010115");
        assert_eq!(vc, VideoCodec::Jpeg2000Broadcast);
        assert!(vc.is_jpeg2000_family());
    }

    /// Generic J2K node UL (byte [15]=0x00) maps to Jpeg2000 (generic).
    #[test]
    fn video_codec_jpeg2000_generic() {
        let vc = VideoCodec::from_ul("urn:smpte:ul:060e2b34.04010107.04010202.03010100");
        assert_eq!(vc, VideoCodec::Jpeg2000);
        assert!(vc.is_jpeg2000_family());
    }

    /// UL 03010101 (Profile 0/1) is not a recognized App2E profile — maps to Unknown.
    /// Photon rejects it as "Invalid JPEG 2000 Profile" (not a BCP sub-level 0x11-0x17).
    #[test]
    fn video_codec_jpeg2000_unrecognized_maps_to_unknown() {
        let vc = VideoCodec::from_ul("urn:smpte:ul:060e2b34.04010107.04010202.03010101");
        assert!(
            matches!(vc, VideoCodec::Unknown(_)),
            "expected Unknown, got {:?}",
            vc
        );
        assert!(!vc.is_jpeg2000_family());
    }

    #[test]
    fn video_codec_jpeg2000_ht() {
        let vc = VideoCodec::from_ul("urn:smpte:ul:060e2b34.04010101.04010202.03010801");
        assert_eq!(vc, VideoCodec::Jpeg2000Ht);
        assert!(vc.is_jpeg2000_family());
    }

    #[test]
    fn video_codec_unknown_non_picture_ul() {
        // UL with wrong bytes 9-12 (not picture coding)
        let vc = VideoCodec::from_ul("urn:smpte:ul:060e2b34.04010101.04010101.03030000");
        assert!(matches!(vc, VideoCodec::Unknown(_)));
    }

    // ── CplNamespace ──────────────────────────────────────────────────────────

    /// SMPTE ST 2067-3: CPL namespace URIs identify the spec version.
    #[test]
    fn cpl_namespace_from_uri_2020() {
        let ns = CplNamespace::from_uri("http://www.smpte-ra.org/ns/2067-3/2020");
        assert_eq!(ns, CplNamespace::Smpte2067_3_2020);
        assert_eq!(ns.year(), Some(2020));
        assert_eq!(ns.spec_id(), "ST 2067-3:2020");
    }

    #[test]
    fn cpl_namespace_from_uri_2016() {
        let ns = CplNamespace::from_uri("http://www.smpte-ra.org/schemas/2067-3/2016");
        assert_eq!(ns, CplNamespace::Smpte2067_3_2016);
    }

    #[test]
    fn cpl_namespace_unknown() {
        let ns = CplNamespace::from_uri("http://example.com/custom");
        assert!(matches!(ns, CplNamespace::Unknown(_)));
        assert_eq!(ns.year(), None);
    }

    // ── McaTagSymbol ─────────────────────────────────────────────────────────

    #[test]
    fn mca_tag_symbol_parse() {
        assert_eq!(McaTagSymbol::parse("L"), McaTagSymbol::Left);
        assert_eq!(McaTagSymbol::parse("R"), McaTagSymbol::Right);
        assert_eq!(McaTagSymbol::parse("IAB"), McaTagSymbol::Iab);
        assert_eq!(McaTagSymbol::parse("sg51"), McaTagSymbol::Sg51);
        assert_eq!(
            McaTagSymbol::parse("custom"),
            McaTagSymbol::Other("custom".to_string())
        );
    }

    // ── MarkerLabel ───────────────────────────────────────────────────────────

    #[test]
    fn marker_label_parse() {
        assert_eq!(MarkerLabel::parse("FFOC"), MarkerLabel::Ffoc);
        assert_eq!(MarkerLabel::parse("LFOC"), MarkerLabel::Lfoc);
        assert_eq!(
            MarkerLabel::parse("CUSTOM"),
            MarkerLabel::Other("CUSTOM".to_string())
        );
    }
}
