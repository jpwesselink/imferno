//! SMPTE ST 2067 Constraints Validation Framework
//!
//! Implements the factory + trait pattern for normative IMF constraints validation,
//! modeled after Netflix Photon's `ConstraintsValidator` architecture.
//!
//! Each SMPTE spec version gets its own validator struct. The factory dispatches
//! by namespace URI. Multiple validators run per CPL — typically one for core
//! constraints and one (or more) for application profiles.
//!
//! ## Validation Architecture
//!
//! ```text
//! CPL namespaces ──→ Factory ──→ [CoreConstraints2020, App2E2021, ...]
//!                                  │                    │
//!                             validate_cpl()       validate_cpl()
//!                                  │                    │
//!                          Vec<ValidationIssue>  Vec<ValidationIssue>
//!                                  └────── merge ───────┘
//! ```
//!
//! ## Implemented Specs
//!
//! - **ST 2067-2:2020** Core Constraints (`CoreConstraints2020`)
//! - **ST 2067-2:2016** Core Constraints (`CoreConstraints2016`)
//! - **ST 2067-2:2013** Core Constraints (`CoreConstraints2013`)
//! - **ST 2067-21:2023** Application Profile #2E (`App2E2021`)
//! - **ST 2067-201:2019/2021** IAB Level 0 Plug-in (`App2E2021` ruleset mapping)

pub mod codes;
pub mod iab;
pub mod iab_codes;
pub mod isxd;
pub mod isxd_codes;

pub use iab::{
    AppIabPlugin2019, AppIabPlugin2021, URI_2019, URI_2019_SCHEMAS, URI_2021, URI_2021_SCHEMAS,
};
pub use isxd::{AppIsxdPlugin2022, URI_2022};

use std::collections::{HashMap, HashSet};

use self::codes::{St2067_21_2020, St2067_21_2023, St2067_21_2025};
use crate::assetmap::codes::CoreConstraintsCode;
use crate::cpl::codes::{St2067_3Code, St2067_3_2013, St2067_3_2016, St2067_3_2020};
use crate::cpl::CompositionPlaylist;
use crate::cpl::{CodingEquations, ColorPrimaries, CplNamespace, EditRate, TransferCharacteristic};
use crate::diagnostics::codes::ValidationCode;
use crate::diagnostics::{Category, Location, Severity, ValidationIssue};

// ═════════════════════════════════════════════════════════════════════════════
// Trait
// ═════════════════════════════════════════════════════════════════════════════

/// A normative constraints validator, dispatched by namespace URI.
///
/// Each SMPTE spec version implements this trait. Multiple validators
/// run per CPL — one for core constraints, one for CPL schema, and one
/// (or more) for application profiles.
///
/// Modeled after Photon's `ConstraintsValidator` interface.
pub trait ConstraintsValidator {
    /// Human-readable specification identifier, e.g. "ST 2067-2:2020 Core Constraints".
    fn spec_id(&self) -> &str;

    /// Validate a CPL against this constraint set.
    /// Returns a list of validation issues (may be empty for compliant content).
    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue>;
}

// ═════════════════════════════════════════════════════════════════════════════
// Factory
// ═════════════════════════════════════════════════════════════════════════════

/// Registry abstraction for resolving validators by namespace/profile URIs.
pub trait ValidatorRegistry {
    /// Resolve a single validator by namespace URI.
    fn resolve_namespace(&self, namespace_uri: &str) -> Option<Box<dyn ConstraintsValidator>>;

    /// Resolve all applicable validators for a CPL.
    fn resolve_for_cpl(&self, cpl: &CompositionPlaylist) -> Vec<Box<dyn ConstraintsValidator>> {
        let mut validators: Vec<Box<dyn ConstraintsValidator>> = Vec::new();

        let core_ns = match &cpl.namespace {
            CplNamespace::Smpte2067_3_2013 => Some("http://www.smpte-ra.org/schemas/2067-2/2013"),
            CplNamespace::Smpte2067_3_2016 => Some("http://www.smpte-ra.org/schemas/2067-2/2016"),
            CplNamespace::Smpte2067_3_2020 => Some("http://www.smpte-ra.org/ns/2067-2/2020"),
            _ => None,
        };
        if let Some(ns) = core_ns {
            if let Some(v) = self.resolve_namespace(ns) {
                validators.push(v);
            }
        }

        if let Some(ref ext) = cpl.extension_properties {
            if let Some(ref app_id) = ext.application_identification {
                for uri in app_id.split_whitespace() {
                    if let Some(v) = self.resolve_namespace(uri) {
                        validators.push(v);
                    }
                }
            }
        }

        validators
    }
}

/// Built-in namespace/profile resolver for supported SMPTE validators.
pub struct BuiltinValidatorRegistry;

impl ValidatorRegistry for BuiltinValidatorRegistry {
    fn resolve_namespace(&self, namespace_uri: &str) -> Option<Box<dyn ConstraintsValidator>> {
        match namespace_uri {
            "http://www.smpte-ra.org/schemas/2067-2/2013" => Some(Box::new(CoreConstraints2013)),
            "http://www.smpte-ra.org/schemas/2067-2/2016" => Some(Box::new(CoreConstraints2016)),
            "http://www.smpte-ra.org/ns/2067-2/2020" => Some(Box::new(CoreConstraints2020)),
            "http://www.smpte-ra.org/ns/2067-21/2020" => Some(Box::new(App2E2020)),
            "http://www.smpte-ra.org/schemas/2067-21/2014"
            | "http://www.smpte-ra.org/schemas/2067-21/2015"
            | "http://www.smpte-ra.org/schemas/2067-21/2016"
            | "http://www.smpte-ra.org/ns/2067-21/2021"
            | "http://www.smpte-ra.org/ns/2067-21/2023" => Some(Box::new(App2E2021)),
            _ => None,
        }
    }
}

/// Optional validator selection overrides for registry-driven dispatch.
///
/// This allows callers (e.g. CLI) to pin the validator namespace(s) used during
/// dispatch instead of relying solely on CPL-declared namespaces/app IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreSpecTarget {
    St2067_2_2013,
    St2067_2_2016,
    St2067_2_2020,
}

impl CoreSpecTarget {
    pub fn namespace_uri(&self) -> &'static str {
        match self {
            Self::St2067_2_2013 => "http://www.smpte-ra.org/schemas/2067-2/2013",
            Self::St2067_2_2016 => "http://www.smpte-ra.org/schemas/2067-2/2016",
            Self::St2067_2_2020 => "http://www.smpte-ra.org/ns/2067-2/2020",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppSpecTarget {
    St2067_21_2020,
    St2067_21_2021,
    St2067_21_2023,
}

impl AppSpecTarget {
    pub fn application_identification_uri(&self) -> &'static str {
        match self {
            Self::St2067_21_2020 => "http://www.smpte-ra.org/ns/2067-21/2020",
            Self::St2067_21_2021 => "http://www.smpte-ra.org/ns/2067-21/2021",
            Self::St2067_21_2023 => "http://www.smpte-ra.org/ns/2067-21/2023",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ValidatorSelection {
    /// Preferred typed core constraints selection.
    pub core_spec: Option<CoreSpecTarget>,

    /// Preferred typed app profile selection.
    ///
    /// When set, these app specs are used instead of CPL `ApplicationIdentification`.
    pub app_specs: Option<Vec<AppSpecTarget>>,

    /// Override for the core constraints namespace URI.
    ///
    /// Example: `http://www.smpte-ra.org/schemas/2067-2/2016`
    ///
    /// Prefer `core_spec` for strongly typed selection.
    pub core_namespace_uri: Option<String>,

    /// Override for ApplicationIdentification namespace URIs.
    ///
    /// When set, these URIs are used instead of CPL `ApplicationIdentification`.
    /// Prefer `app_specs` for strongly typed selection.
    pub application_identification_uris: Option<Vec<String>>,
}

/// Registry that applies caller-provided selection overrides, then resolves
/// resulting URIs via built-in validator mappings.
pub struct ConfigurableValidatorRegistry {
    selection: ValidatorSelection,
}

impl ConfigurableValidatorRegistry {
    pub fn new(selection: ValidatorSelection) -> Self {
        Self { selection }
    }
}

impl ValidatorRegistry for ConfigurableValidatorRegistry {
    fn resolve_namespace(&self, namespace_uri: &str) -> Option<Box<dyn ConstraintsValidator>> {
        BuiltinValidatorRegistry.resolve_namespace(namespace_uri)
    }

    fn resolve_for_cpl(&self, cpl: &CompositionPlaylist) -> Vec<Box<dyn ConstraintsValidator>> {
        let mut validators: Vec<Box<dyn ConstraintsValidator>> = Vec::new();

        let core_ns = if let Some(core_spec) = self.selection.core_spec {
            Some(core_spec.namespace_uri())
        } else if let Some(uri) = self.selection.core_namespace_uri.as_deref() {
            Some(uri)
        } else {
            match &cpl.namespace {
                CplNamespace::Smpte2067_3_2013 => {
                    Some("http://www.smpte-ra.org/schemas/2067-2/2013")
                }
                CplNamespace::Smpte2067_3_2016 => {
                    Some("http://www.smpte-ra.org/schemas/2067-2/2016")
                }
                CplNamespace::Smpte2067_3_2020 => Some("http://www.smpte-ra.org/ns/2067-2/2020"),
                _ => None,
            }
        };

        if let Some(ns) = core_ns {
            if let Some(v) = self.resolve_namespace(ns) {
                validators.push(v);
            }
        }

        if let Some(ref app_specs) = self.selection.app_specs {
            for spec in app_specs {
                if let Some(v) = self.resolve_namespace(spec.application_identification_uri()) {
                    validators.push(v);
                }
            }
        } else if let Some(ref app_uris) = self.selection.application_identification_uris {
            for uri in app_uris {
                if let Some(v) = self.resolve_namespace(uri) {
                    validators.push(v);
                }
            }
        } else if let Some(ref ext) = cpl.extension_properties {
            if let Some(ref app_id) = ext.application_identification {
                for uri in app_id.split_whitespace() {
                    if let Some(v) = self.resolve_namespace(uri) {
                        validators.push(v);
                    }
                }
            }
        }

        validators
    }
}

/// Look up a single validator by namespace URI.
///
/// Returns `None` for unrecognized namespaces.
pub fn get_validator(namespace_uri: &str) -> Option<Box<dyn ConstraintsValidator>> {
    BuiltinValidatorRegistry.resolve_namespace(namespace_uri)
}

/// Collect all applicable validators for a CPL.
///
/// Collects namespace URIs from:
/// 1. CPL root namespace → maps to core constraints version
/// 2. ApplicationIdentification → maps to application profile
///
/// Modeled after Photon's `IMPValidator.validateComposition()` namespace collection.
pub fn get_validators_for_cpl(cpl: &CompositionPlaylist) -> Vec<Box<dyn ConstraintsValidator>> {
    BuiltinValidatorRegistry.resolve_for_cpl(cpl)
}

/// Run all applicable validators on a CPL and collect issues.
///
/// This is the main entry point for validation.
pub fn validate_cpl(cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
    validate_cpl_with_builtin_registry(cpl)
}

/// Run all applicable built-in validators on a CPL and collect issues.
pub fn validate_cpl_with_builtin_registry(cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
    let validators = get_validators_for_cpl(cpl);
    let mut all_issues = Vec::new();
    for v in &validators {
        all_issues.extend(v.validate_cpl(cpl));
    }
    all_issues
}

/// Run all applicable validators from a provided registry on a CPL.
///
/// This is designed to plug directly into `imf-parser`'s
/// `validate_package_structure_with_cpl_validator` seam.
pub fn validate_cpl_with_registry(
    cpl: &CompositionPlaylist,
    registry: &dyn ValidatorRegistry,
) -> Vec<ValidationIssue> {
    let validators = registry.resolve_for_cpl(cpl);
    let mut all_issues = Vec::new();
    for v in &validators {
        all_issues.extend(v.validate_cpl(cpl));
    }
    all_issues
}

// ═════════════════════════════════════════════════════════════════════════════
// Colorimetry — ST 2067-21:2023 Table 2 / Table 3
// ═════════════════════════════════════════════════════════════════════════════

/// Named colorimetry systems per ST 2067-21:2023 Table 3.
///
/// Each system is a normative combination of ColorPrimaries, TransferCharacteristic,
/// and CodingEquations. The validator identifies a color system from the descriptor's
/// UL values, then checks it against the allowed set for the image characteristic row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSystem {
    /// COLOR.1: BT.601-625 primaries, BT.709 TC, BT.601 CE (SD PAL)
    Color1,
    /// COLOR.2: BT.601-525 primaries, BT.709 TC, BT.601 CE (SD NTSC)
    Color2,
    /// COLOR.3: BT.709 primaries, BT.709 TC, BT.709 CE (HD SDR)
    Color3,
    /// COLOR.4: BT.709 primaries, xvYCC 709 TC, BT.709 CE (extended gamut)
    Color4,
    /// COLOR.5: BT.2020 primaries, BT.2020 TC, BT.2020 NCL CE (UHD SDR)
    Color5,
    /// COLOR.6: P3 D65 primaries, PQ TC, None/RGB (HDR RGB)
    Color6,
    /// COLOR.7: BT.2020 primaries, PQ TC, BT.2020 NCL CE (HDR10)
    Color7,
    /// COLOR.8: BT.2020 primaries, HLG TC, BT.2020 NCL CE (HLG)
    Color8,
}

impl ColorSystem {
    /// Resolve a colorimetry system from its component ULs.
    ///
    /// Returns `None` if the combination does not match any defined Color System.
    /// For COLOR.6 (RGB), `coding_eq` should be `None`.
    pub fn from_components(
        primaries: &ColorPrimaries,
        transfer: &TransferCharacteristic,
        coding_eq: Option<&CodingEquations>,
    ) -> Option<Self> {
        // Match CDCI (Y'C'BC'R) descriptors — CodingEquations is present.
        // Match RGBA (R'G'B') descriptors — CodingEquations is None.
        // For RGB content, the color system is determined by ColorPrimaries + TransferCharacteristic only.
        match (primaries, transfer, coding_eq) {
            // CDCI matches (explicit CodingEquations)
            (
                ColorPrimaries::Bt601_625,
                TransferCharacteristic::Bt709,
                Some(CodingEquations::Bt601),
            ) => Some(Self::Color1),
            (
                ColorPrimaries::Bt601_525,
                TransferCharacteristic::Bt709,
                Some(CodingEquations::Bt601),
            ) => Some(Self::Color2),
            (
                ColorPrimaries::Bt709,
                TransferCharacteristic::Bt709,
                Some(CodingEquations::Bt709),
            ) => Some(Self::Color3),
            (
                ColorPrimaries::Bt709,
                TransferCharacteristic::XvYcc709,
                Some(CodingEquations::Bt709),
            ) => Some(Self::Color4),
            (
                ColorPrimaries::Bt2020,
                TransferCharacteristic::Bt2020,
                Some(CodingEquations::Bt2020Ncl),
            ) => Some(Self::Color5),
            (
                ColorPrimaries::Bt2020,
                TransferCharacteristic::PqSt2084,
                Some(CodingEquations::Bt2020Ncl),
            ) => Some(Self::Color7),
            (
                ColorPrimaries::Bt2020,
                TransferCharacteristic::Hlg,
                Some(CodingEquations::Bt2020Ncl),
            ) => Some(Self::Color8),
            // RGB matches (CodingEquations not applicable for R'G'B' content)
            (ColorPrimaries::Bt601_625, TransferCharacteristic::Bt709, None) => Some(Self::Color1),
            (ColorPrimaries::Bt601_525, TransferCharacteristic::Bt709, None) => Some(Self::Color2),
            (ColorPrimaries::Bt709, TransferCharacteristic::Bt709, None) => Some(Self::Color3),
            (ColorPrimaries::Bt709, TransferCharacteristic::XvYcc709, None) => Some(Self::Color4),
            (ColorPrimaries::Bt2020, TransferCharacteristic::Bt2020, None) => Some(Self::Color5),
            (ColorPrimaries::P3D65, TransferCharacteristic::PqSt2084, None) => Some(Self::Color6),
            (ColorPrimaries::Bt2020, TransferCharacteristic::PqSt2084, None) => Some(Self::Color7),
            (ColorPrimaries::Bt2020, TransferCharacteristic::Hlg, None) => Some(Self::Color8),
            _ => None,
        }
    }

    /// Returns `true` for HDR color systems (PQ or HLG based).
    pub fn is_hdr(&self) -> bool {
        matches!(self, Self::Color6 | Self::Color7 | Self::Color8)
    }

    /// Returns `true` for PQ-based systems that require MaxCLL/MaxFALL.
    /// Per ST 2067-21:2023 section 8.3.3.
    pub fn requires_hdr_metadata(&self) -> bool {
        matches!(self, Self::Color6 | Self::Color7)
    }
}

impl std::fmt::Display for ColorSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Color1 => write!(f, "COLOR.1 (BT.601-625 / BT.709 / BT.601)"),
            Self::Color2 => write!(f, "COLOR.2 (BT.601-525 / BT.709 / BT.601)"),
            Self::Color3 => write!(f, "COLOR.3 (BT.709 / BT.709 / BT.709)"),
            Self::Color4 => write!(f, "COLOR.4 (BT.709 / xvYCC 709 / BT.709)"),
            Self::Color5 => write!(f, "COLOR.5 (BT.2020 / BT.2020 / BT.2020 NCL)"),
            Self::Color6 => write!(f, "COLOR.6 (P3 D65 / PQ / RGB)"),
            Self::Color7 => write!(f, "COLOR.7 (BT.2020 / PQ / BT.2020 NCL)"),
            Self::Color8 => write!(f, "COLOR.8 (BT.2020 / HLG / BT.2020 NCL)"),
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Core Constraints — ST 2067-2
//
// Shared validation logic used by all core constraints versions.
// Mirrors Photon's `IMFCoreConstraintsValidator` abstract base class.
// ═════════════════════════════════════════════════════════════════════════════

/// Validate xs:dateTime format (simplified check without chrono dependency).
/// Accepts: YYYY-MM-DDThh:mm:ss, YYYY-MM-DDThh:mm:ss.f, with optional Z or +/-hh:mm timezone.
fn is_valid_xs_datetime(s: &str) -> bool {
    // Minimum valid: 2024-01-01T00:00:00 (19 chars)
    if s.len() < 19 {
        return false;
    }
    // Check basic structure: digits, dashes, T, colons
    let bytes = s.as_bytes();
    bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[0..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[8..10].iter().all(|b| b.is_ascii_digit())
        && bytes[11..13].iter().all(|b| b.is_ascii_digit())
        && bytes[14..16].iter().all(|b| b.is_ascii_digit())
        && bytes[17..19].iter().all(|b| b.is_ascii_digit())
}

/// Validates SMPTE timecode address format per st2067-3a-2016.xsd `TimecodeType`.
///
/// Pattern: `[0-2][0-9](sep)[0-5][0-9](sep)[0-5][0-9](sep)[0-5][0-9]`
/// where sep ∈ { : / ; , . + - }
fn is_valid_timecode_address(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 11 {
        return false;
    }
    let sep = |c: u8| matches!(c, b':' | b'/' | b';' | b',' | b'.' | b'+' | b'-');
    b[0].is_ascii_digit()
        && b[0] <= b'2'
        && b[1].is_ascii_digit()
        && sep(b[2])
        && b[3].is_ascii_digit()
        && b[3] <= b'5'
        && b[4].is_ascii_digit()
        && sep(b[5])
        && b[6].is_ascii_digit()
        && b[6] <= b'5'
        && b[7].is_ascii_digit()
        && sep(b[8])
        && b[9].is_ascii_digit()
        && b[10].is_ascii_digit()
}

/// Validates `TotalRunningTime` format per st2067-3a-2016.xsd.
///
/// Pattern: `[0-9][0-9]:[0-5][0-9]:[0-5][0-9]`  (HH:MM:SS, exactly 8 chars)
fn is_valid_total_running_time(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 8
        && b[0].is_ascii_digit()
        && b[1].is_ascii_digit()
        && b[2] == b':'
        && b[3].is_ascii_digit()
        && b[3] <= b'5'
        && b[4].is_ascii_digit()
        && b[5] == b':'
        && b[6].is_ascii_digit()
        && b[6] <= b'5'
        && b[7].is_ascii_digit()
}

/// Validates xs:anyURI: must not contain ASCII whitespace.
///
/// xs:anyURI allows empty strings per XSD spec, but IMF requires non-empty Agency values
/// (checked separately). This validates the character-level constraint.
fn is_valid_any_uri(s: &str) -> bool {
    !s.chars().any(|c| c.is_ascii_whitespace())
}

/// XSD SequenceType: ResourceList must have at least one Resource.
fn validate_resource_list_non_empty(
    cpl: &CompositionPlaylist,
    code: fn(CoreConstraintsCode) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    use crate::cpl::SequenceAccess;

    for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
        let sl = &segment.sequence_list;

        fn check_seq<S: SequenceAccess>(
            seqs: &[S],
            track_type: &str,
            cpl_id: &str,
            seg_idx: usize,
            code: fn(CoreConstraintsCode) -> &'static str,
            issues: &mut Vec<ValidationIssue>,
        ) {
            for seq in seqs {
                if seq.resource_list().resources.is_empty() {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Structure,
                            code(CoreConstraintsCode::ResourceListEmpty),
                            format!(
                                "{} {} in Segment {} has an empty ResourceList",
                                track_type,
                                seq.id(),
                                seg_idx + 1,
                            ),
                        )
                        .with_location(
                            Location::new()
                                .with_cpl(cpl_id.to_string())
                                .with_segment(seg_idx),
                        ),
                    );
                }
            }
        }

        let cpl_id = cpl.id.to_string();
        check_seq(
            &sl.main_image_sequences,
            "MainImageSequence",
            &cpl_id,
            seg_idx,
            code,
            issues,
        );
        check_seq(
            &sl.main_audio_sequences,
            "MainAudioSequence",
            &cpl_id,
            seg_idx,
            code,
            issues,
        );
        check_seq(
            &sl.subtitles_sequences,
            "SubtitlesSequence",
            &cpl_id,
            seg_idx,
            code,
            issues,
        );
        check_seq(
            &sl.marker_sequences,
            "MarkerSequence",
            &cpl_id,
            seg_idx,
            code,
            issues,
        );
        check_seq(
            &sl.hearing_impaired_captions_sequences,
            "HearingImpairedCaptionsSequence",
            &cpl_id,
            seg_idx,
            code,
            issues,
        );
        check_seq(
            &sl.forced_narrative_sequences,
            "ForcedNarrativeSequence",
            &cpl_id,
            seg_idx,
            code,
            issues,
        );
        check_seq(
            &sl.iab_sequences,
            "IABSequence",
            &cpl_id,
            seg_idx,
            code,
            issues,
        );
        check_seq(
            &sl.isxd_sequences,
            "ISXDSequence",
            &cpl_id,
            seg_idx,
            code,
            issues,
        );
    }
}

/// Shared core structure checks applied by all spec versions.
fn validate_core_structure(
    cpl: &CompositionPlaylist,
    code: fn(CoreConstraintsCode) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let loc = Location::new().with_cpl(cpl.id.to_string());

    // ContentTitle must be non-empty
    if cpl.content_title.text.trim().is_empty() {
        issues.push(
            ValidationIssue::new(
                Severity::Error,
                Category::Metadata,
                code(CoreConstraintsCode::ContentTitle),
                "ContentTitle shall not be empty",
            )
            .with_location(loc.clone()),
        );
    }

    // XSD: TotalRunningTime pattern [0-9][0-9]:[0-5][0-9]:[0-5][0-9]
    if let Some(ref trt) = cpl.total_running_time {
        if !is_valid_total_running_time(trt) {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Metadata,
                    code(CoreConstraintsCode::TotalRunningTimeFormat),
                    format!(
                        "TotalRunningTime '{}' does not match required format HH:MM:SS \
                         (pattern [0-9][0-9]:[0-5][0-9]:[0-5][0-9])",
                        trt,
                    ),
                )
                .with_location(loc.clone()),
            );
        }
    }

    // SegmentList must have at least one Segment
    if cpl.segment_list.segments.is_empty() {
        issues.push(
            ValidationIssue::new(
                Severity::Critical,
                Category::Structure,
                code(CoreConstraintsCode::SegmentList),
                "SegmentList shall contain at least one Segment",
            )
            .with_location(loc.clone()),
        );
    }

    // Each Segment must have at least one sequence
    for (i, segment) in cpl.segment_list.segments.iter().enumerate() {
        let seg_loc = Location::new().with_cpl(cpl.id.to_string()).with_segment(i);

        let sl = &segment.sequence_list;
        let has_sequences = !sl.main_image_sequences.is_empty()
            || !sl.main_audio_sequences.is_empty()
            || !sl.subtitles_sequences.is_empty()
            || !sl.marker_sequences.is_empty()
            || !sl.hearing_impaired_captions_sequences.is_empty()
            || !sl.forced_narrative_sequences.is_empty()
            || !sl.iab_sequences.is_empty()
            || !sl.isxd_sequences.is_empty();

        if !has_sequences {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Structure,
                    code(CoreConstraintsCode::Segment),
                    format!("Segment {} contains no sequences", i + 1),
                )
                .with_location(seg_loc),
            );
        }
    }

    // XSD §88: EditRate is required on the composition
    if cpl.edit_rate.is_none() {
        issues.push(
            ValidationIssue::new(
                Severity::Error,
                Category::Structure,
                code(CoreConstraintsCode::EditRate),
                "CPL EditRate is required per XSD schema (st2067-3a §88)",
            )
            .with_location(loc.clone()),
        );
    }

    // XSD §66: IssueDate must be a valid xs:dateTime
    if cpl.issue_date.is_empty() {
        issues.push(
            ValidationIssue::new(
                Severity::Error,
                Category::Metadata,
                code(CoreConstraintsCode::IssueDate),
                "CPL IssueDate shall not be empty",
            )
            .with_location(loc.clone()),
        );
    } else if !is_valid_xs_datetime(&cpl.issue_date) {
        issues.push(
            ValidationIssue::new(
                Severity::Warning,
                Category::Metadata,
                code(CoreConstraintsCode::IssueDateFormat),
                format!(
                    "IssueDate '{}' is not a valid xs:dateTime format (expected YYYY-MM-DDThh:mm:ss[.f][Z|+hh:mm])",
                    cpl.issue_date,
                ),
            )
            .with_location(loc.clone()),
        );
    }

    // XSD §121-127: CompositionTimecode completeness (if present, all sub-fields required)
    if let Some(ref tc) = cpl.composition_timecode {
        let tc_loc = loc.clone();
        if tc.timecode_drop_frame.is_none() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Structure,
                    code(CoreConstraintsCode::CompositionTimecodeDropFrame),
                    "CompositionTimecode.TimecodeDropFrame is required when CompositionTimecode is present",
                )
                .with_location(tc_loc.clone()),
            );
        }
        if tc.timecode_rate.is_none() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Structure,
                    code(CoreConstraintsCode::CompositionTimecodeRate),
                    "CompositionTimecode.TimecodeRate is required when CompositionTimecode is present",
                )
                .with_location(tc_loc.clone()),
            );
        }
        if tc.timecode_start_address.is_none() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Structure,
                    code(CoreConstraintsCode::CompositionTimecodeStartAddress),
                    "CompositionTimecode.TimecodeStartAddress is required when CompositionTimecode is present",
                )
                .with_location(tc_loc.clone()),
            );
        }
        // XSD xs:positiveInteger: TimecodeRate must be > 0
        if let Some(rate) = tc.timecode_rate {
            if rate == 0 {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Timing,
                        code(CoreConstraintsCode::CompositionTimecodeRateZero),
                        "CompositionTimecode.TimecodeRate shall be a positive integer (xs:positiveInteger); 0 is not valid",
                    )
                    .with_location(tc_loc.clone()),
                );
            }
        }
        // XSD TimecodeType pattern: HH:MM:SS:FF with separator in { : / ; , . + - }
        if let Some(ref addr) = tc.timecode_start_address {
            if !is_valid_timecode_address(addr) {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Timing,
                        code(CoreConstraintsCode::CompositionTimecodeStartAddressFormat),
                        format!(
                            "TimecodeStartAddress '{}' does not match SMPTE timecode format \
                             HH:MM:SS:FF (separators: : / ; , . + -)",
                            addr,
                        ),
                    )
                    .with_location(tc_loc),
                );
            }
        }
    }

    // XSD §121-127: CompositionTimecode.TimecodeRate should match CPL EditRate
    if let (Some(ref tc), Some(ref er)) = (&cpl.composition_timecode, &cpl.edit_rate) {
        if let Some(tc_rate) = tc.timecode_rate {
            // EditRate as integer fps (for non-drop-frame comparison)
            let edit_fps = if er.denominator > 0 {
                (er.numerator as f64 / er.denominator as f64).round() as u32
            } else {
                0
            };
            if tc_rate != edit_fps {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        Category::Timing,
                        code(CoreConstraintsCode::CompositionTimecodeRateMismatch),
                        format!(
                            "CompositionTimecode.TimecodeRate {} does not match CPL EditRate {}/{}  (≈{} fps)",
                            tc_rate, er.numerator, er.denominator, edit_fps,
                        ),
                    )
                    .with_location(loc.clone()),
                );
            }
        }
    }

    // XSD: LocaleList must have at least one Locale (minOccurs=1)
    if let Some(ref ll) = cpl.locale_list {
        if ll.locales.is_empty() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Structure,
                    code(CoreConstraintsCode::LocaleListNonEmpty),
                    "LocaleList shall contain at least one Locale (XSD minOccurs=1)",
                )
                .with_location(loc.clone()),
            );
        }
    }

    // ST 2067-3 constraints delegated to st2067-3 crate (§5.5.1.2, §6.4.2, §6.11, §6.12, §7.3, §7.4)
    issues.extend(crate::cpl::validate_cpl_constraints(cpl));

    // XSD SequenceType: ResourceList must have at least one Resource
    validate_resource_list_non_empty(cpl, code, issues);

    // UUID uniqueness
    validate_uuid_uniqueness(cpl, code, issues);

    // Resource constraints (duration, timing)
    validate_resource_constraints(cpl, code, issues);

    // Virtual track continuity across segments
    validate_virtual_track_continuity(cpl, code, issues);

    // Virtual track edit rate consistency
    validate_virtual_track_edit_rates(cpl, code, issues);

    // Audio MCA label validation
    validate_audio_mca_labels(cpl, code, issues);

    // Timed text descriptor constraints (ST 2067-2 §10)
    validate_timed_text_extended(cpl, code, issues);

    // Segment track duration consistency
    validate_segment_track_durations(cpl, issues);

    // Digital Signatures (ST 2067-2 §8)
    validate_digital_signature_notice(cpl, code, issues);

    // §6.4.2: Every EssenceDescriptor must be referenced by at least one Resource.
    // Applies to all spec versions (no-op when EDL is absent).
    validate_dangling_essence_descriptors(cpl, code, issues);
}

// ─────────────────────────────────────────────────────────────────────────────
// UUID uniqueness validation
// ─────────────────────────────────────────────────────────────────────────────

/// All Segment Ids, EssenceDescriptor Ids, and Resource Ids shall be unique within a CPL.
///
/// ST 2067-2 §6.1: "Each Id element shall be unique within the scope of the Composition."
fn validate_uuid_uniqueness(
    cpl: &CompositionPlaylist,
    code: fn(CoreConstraintsCode) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let cpl_loc = Location::new().with_cpl(cpl.id.to_string());

    // Segment ID uniqueness
    let mut segment_ids = HashSet::new();
    for (i, segment) in cpl.segment_list.segments.iter().enumerate() {
        let id_str = segment.id.to_string();
        if !segment_ids.insert(id_str.clone()) {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Structure,
                    code(CoreConstraintsCode::UniqueSegmentId),
                    format!("Duplicate Segment Id '{}' at index {}", id_str, i),
                )
                .with_location(cpl_loc.clone().with_segment(i)),
            );
        }
    }

    // EssenceDescriptor ID uniqueness
    // (empty EDL is reported by st2067-3::validate_cpl_constraints as 6.4.2/EssenceDescriptorListEmpty)
    if let Some(ref edl) = cpl.essence_descriptor_list {
        let mut ed_ids = HashSet::new();
        for ed in &edl.essence_descriptors {
            let id_str = ed.id.to_string();
            if !ed_ids.insert(id_str.clone()) {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Structure,
                        code(CoreConstraintsCode::UniqueEssenceDescriptorId),
                        format!("Duplicate EssenceDescriptor Id '{}'", id_str),
                    )
                    .with_location(cpl_loc.clone()),
                );
            }
        }
    }

    // Resource ID uniqueness (across all segments and sequences)
    let mut resource_ids = HashSet::new();
    for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
        let mut check_resources = |resources: &[crate::cpl::Resource], track_type: &str| {
            for resource in resources {
                let id_str = resource.id.to_string();
                if !resource_ids.insert(id_str.clone()) {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Structure,
                            code(CoreConstraintsCode::UniqueResourceId),
                            format!(
                                "Duplicate Resource Id '{}' in {} segment {}",
                                id_str,
                                track_type,
                                seg_idx + 1,
                            ),
                        )
                        .with_location(cpl_loc.clone().with_segment(seg_idx)),
                    );
                }
            }
        };

        let sl = &segment.sequence_list;
        for seq in &sl.main_image_sequences {
            check_resources(&seq.resource_list.resources, "MainImageSequence");
        }
        for seq in &sl.main_audio_sequences {
            check_resources(&seq.resource_list.resources, "MainAudioSequence");
        }
        for seq in &sl.subtitles_sequences {
            check_resources(&seq.resource_list.resources, "SubtitlesSequence");
        }
        for seq in &sl.hearing_impaired_captions_sequences {
            check_resources(
                &seq.resource_list.resources,
                "HearingImpairedCaptionsSequence",
            );
        }
        for seq in &sl.forced_narrative_sequences {
            check_resources(&seq.resource_list.resources, "ForcedNarrativeSequence");
        }
        for seq in &sl.iab_sequences {
            check_resources(&seq.resource_list.resources, "IABSequence");
        }
        for seq in &sl.marker_sequences {
            check_resources(&seq.resource_list.resources, "MarkerSequence");
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Resource constraints validation
// ─────────────────────────────────────────────────────────────────────────────

/// Validate individual resource constraints.
///
/// ST 2067-2 §6.10:
/// - IntrinsicDuration shall be greater than 0.
/// - If EntryPoint is present, EntryPoint + SourceDuration <= IntrinsicDuration.
/// - If RepeatCount is present, it shall be greater than 0.
/// - Non-marker resources shall have TrackFileId and SourceEncoding.
fn validate_resource_constraints(
    cpl: &CompositionPlaylist,
    code: fn(CoreConstraintsCode) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    use crate::cpl::SequenceAccess;

    for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
        // Helper: validate resources within a sequence
        let validate_resources =
            |seq: &dyn SequenceAccess,
             track_type: &str,
             is_marker: bool,
             issues: &mut Vec<ValidationIssue>| {
                for (res_idx, resource) in seq.resource_list().resources.iter().enumerate() {
                    let res_loc = Location::new()
                        .with_cpl(cpl.id.to_string())
                        .with_segment(seg_idx)
                        .with_resource(res_idx);

                    // IntrinsicDuration > 0
                    if resource.intrinsic_duration == 0 {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                Category::Timing,
                                code(CoreConstraintsCode::IntrinsicDuration),
                                format!(
                                    "{} resource {} IntrinsicDuration shall be greater than 0",
                                    track_type, resource.id,
                                ),
                            )
                            .with_location(res_loc.clone()),
                        );
                    }

                    // B: EntryPoint < IntrinsicDuration (ST 2067-2 §6.10)
                    let entry_point = resource.entry_point.unwrap_or(0);
                    if resource.entry_point.is_some()
                        && resource.intrinsic_duration > 0
                        && entry_point >= resource.intrinsic_duration
                    {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                Category::Timing,
                                code(CoreConstraintsCode::EntryPoint),
                                format!(
                                    "{} resource {}: EntryPoint ({}) shall be less than \
                                 IntrinsicDuration ({})",
                                    track_type,
                                    resource.id,
                                    entry_point,
                                    resource.intrinsic_duration,
                                ),
                            )
                            .with_location(res_loc.clone()),
                        );
                    }

                    // A: SourceDuration > 0 (ST 2067-2 §6.10)
                    // EntryPoint + SourceDuration <= IntrinsicDuration
                    if let Some(source_duration) = resource.source_duration {
                        if source_duration == 0 {
                            issues.push(
                                ValidationIssue::new(
                                    Severity::Error,
                                    Category::Timing,
                                    code(CoreConstraintsCode::SourceDuration),
                                    format!(
                                        "{} resource {}: SourceDuration shall be greater than 0",
                                        track_type, resource.id,
                                    ),
                                )
                                .with_location(res_loc.clone()),
                            );
                        }
                        if entry_point + source_duration > resource.intrinsic_duration {
                            issues.push(
                                ValidationIssue::new(
                                    Severity::Error,
                                    Category::Timing,
                                    code(CoreConstraintsCode::ResourceDuration),
                                    format!(
                                    "{} resource {}: EntryPoint ({}) + SourceDuration ({}) = {} \
                                     exceeds IntrinsicDuration ({})",
                                    track_type, resource.id,
                                    entry_point, source_duration,
                                    entry_point + source_duration,
                                    resource.intrinsic_duration,
                                ),
                                )
                                .with_location(res_loc.clone()),
                            );
                        }
                    }

                    // RepeatCount > 0 (if present)
                    if let Some(repeat_count) = resource.repeat_count {
                        if repeat_count == 0 {
                            issues.push(
                                ValidationIssue::new(
                                    Severity::Error,
                                    Category::Timing,
                                    code(CoreConstraintsCode::RepeatCount),
                                    format!(
                                        "{} resource {} RepeatCount shall be greater than 0",
                                        track_type, resource.id,
                                    ),
                                )
                                .with_location(res_loc.clone()),
                            );
                        }
                    }

                    // Non-marker resources shall have TrackFileId
                    if !is_marker && resource.track_file_id.is_none() {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                Category::Reference,
                                code(CoreConstraintsCode::TrackFileId),
                                format!(
                                    "{} resource {} is missing TrackFileId",
                                    track_type, resource.id,
                                ),
                            )
                            .with_location(res_loc.clone()),
                        );
                    }
                }
            };

        let sl = &segment.sequence_list;
        for seq in &sl.main_image_sequences {
            validate_resources(seq, "MainImageSequence", false, issues);
        }
        for seq in &sl.main_audio_sequences {
            validate_resources(seq, "MainAudioSequence", false, issues);
        }
        for seq in &sl.subtitles_sequences {
            validate_resources(seq, "SubtitlesSequence", false, issues);
        }
        for seq in &sl.hearing_impaired_captions_sequences {
            validate_resources(seq, "HearingImpairedCaptionsSequence", false, issues);
        }
        for seq in &sl.forced_narrative_sequences {
            validate_resources(seq, "ForcedNarrativeSequence", false, issues);
        }
        for seq in &sl.iab_sequences {
            validate_resources(seq, "IABSequence", false, issues);
        }
        for seq in &sl.marker_sequences {
            validate_resources(seq, "MarkerSequence", true, issues);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Virtual track continuity
// ─────────────────────────────────────────────────────────────────────────────

/// Helper: collect all (TrackId, track_type) pairs from a segment's sequences.
fn collect_track_ids(segment: &crate::cpl::Segment) -> HashMap<String, &'static str> {
    use crate::cpl::SequenceAccess;

    let mut track_ids = HashMap::new();
    let sl = &segment.sequence_list;

    for seq in &sl.main_image_sequences {
        track_ids.insert(seq.track_id().to_string(), "MainImageSequence");
    }
    for seq in &sl.main_audio_sequences {
        track_ids.insert(seq.track_id().to_string(), "MainAudioSequence");
    }
    for seq in &sl.subtitles_sequences {
        track_ids.insert(seq.track_id().to_string(), "SubtitlesSequence");
    }
    for seq in &sl.hearing_impaired_captions_sequences {
        track_ids.insert(
            seq.track_id().to_string(),
            "HearingImpairedCaptionsSequence",
        );
    }
    for seq in &sl.forced_narrative_sequences {
        track_ids.insert(seq.track_id().to_string(), "ForcedNarrativeSequence");
    }
    for seq in &sl.iab_sequences {
        track_ids.insert(seq.track_id().to_string(), "IABSequence");
    }
    // Marker sequences participate in virtual tracks but are optional per segment

    track_ids
}

/// Virtual track continuity: essence-bearing tracks that appear in one segment
/// must appear in all segments.
///
/// ST 2067-2 §6.9: "A Virtual Track shall be present in every Segment of the
/// Composition Playlist."
fn validate_virtual_track_continuity(
    cpl: &CompositionPlaylist,
    code: fn(CoreConstraintsCode) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let segments = &cpl.segment_list.segments;
    if segments.len() < 2 {
        return; // Nothing to check with 0 or 1 segments
    }

    // Collect the union of all essence-bearing TrackIds across all segments
    let mut all_track_ids: HashMap<String, &'static str> = HashMap::new();
    let mut per_segment_tracks: Vec<HashSet<String>> = Vec::new();

    for segment in segments {
        let tracks = collect_track_ids(segment);
        let track_set: HashSet<String> = tracks.keys().cloned().collect();
        for (id, tt) in &tracks {
            all_track_ids.entry(id.clone()).or_insert(tt);
        }
        per_segment_tracks.push(track_set);
    }

    // Check that every track ID appears in every segment
    for (track_id, track_type) in &all_track_ids {
        for (seg_idx, seg_tracks) in per_segment_tracks.iter().enumerate() {
            if !seg_tracks.contains(track_id) {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Structure,
                        code(CoreConstraintsCode::VirtualTrackContinuity),
                        format!(
                            "{} virtual track '{}' is missing from segment {} \
                             but is present in other segments; \
                             a virtual track shall be present in every segment",
                            track_type,
                            track_id,
                            seg_idx + 1,
                        ),
                    )
                    .with_location(
                        Location::new()
                            .with_cpl(cpl.id.to_string())
                            .with_segment(seg_idx),
                    ),
                );
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Virtual track edit rate consistency
// ─────────────────────────────────────────────────────────────────────────────

/// All resources in a virtual track shall have the same edit rate.
///
/// ST 2067-2 §6.9.3: "The value of the EditRate element within each Resource
/// of a Virtual Track shall be identical."
fn validate_virtual_track_edit_rates(
    cpl: &CompositionPlaylist,
    code: fn(CoreConstraintsCode) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    use crate::cpl::SequenceAccess;

    // Map: TrackId → first-seen resolved EditRate.
    // None on a resource means "inherit the CPL's edit rate" per ST 2067-3 §6.9.3,
    // so we resolve it to the CPL value before recording or comparing.
    let mut track_edit_rates: HashMap<String, EditRate> = HashMap::new();

    for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
        let check_sequence =
            |seq: &dyn SequenceAccess,
             track_type: &str,
             issues: &mut Vec<ValidationIssue>,
             track_edit_rates: &mut HashMap<String, EditRate>| {
                let track_id = seq.track_id().to_string();
                for resource in &seq.resource_list().resources {
                    // Resolve: absent EditRate inherits the CPL's edit rate.
                    // If neither resource nor CPL has an edit rate, skip this resource.
                    let resolved_er = match resource.edit_rate.or(cpl.edit_rate) {
                        Some(er) => er,
                        None => continue,
                    };
                    match track_edit_rates.get(&track_id) {
                        None => {
                            // First resource for this track — record its resolved edit rate
                            track_edit_rates.insert(track_id.clone(), resolved_er);
                        }
                        Some(&first_er) => {
                            // Subsequent resource — compare resolved values
                            if resolved_er != first_er {
                                issues.push(
                                ValidationIssue::new(
                                    Severity::Error,
                                    Category::Timing,
                                    code(CoreConstraintsCode::VirtualTrackEditRate),
                                    format!(
                                        "{} virtual track '{}': resource {} has EditRate {}/{} \
                                         but earlier resources have {}/{}; \
                                         all resources in a virtual track shall have the same edit rate",
                                        track_type, track_id, resource.id,
                                        resolved_er.numerator, resolved_er.denominator,
                                        first_er.numerator, first_er.denominator,
                                    ),
                                )
                                .with_location(
                                    Location::new()
                                        .with_cpl(cpl.id.to_string())
                                        .with_segment(seg_idx),
                                ),
                            );
                            }
                        }
                    }
                }
            };

        let sl = &segment.sequence_list;
        for seq in &sl.main_image_sequences {
            check_sequence(seq, "MainImageSequence", issues, &mut track_edit_rates);
        }
        for seq in &sl.main_audio_sequences {
            check_sequence(seq, "MainAudioSequence", issues, &mut track_edit_rates);
        }
        for seq in &sl.subtitles_sequences {
            check_sequence(seq, "SubtitlesSequence", issues, &mut track_edit_rates);
        }
        for seq in &sl.hearing_impaired_captions_sequences {
            check_sequence(
                seq,
                "HearingImpairedCaptionsSequence",
                issues,
                &mut track_edit_rates,
            );
        }
        for seq in &sl.forced_narrative_sequences {
            check_sequence(
                seq,
                "ForcedNarrativeSequence",
                issues,
                &mut track_edit_rates,
            );
        }
        for seq in &sl.iab_sequences {
            check_sequence(seq, "IABSequence", issues, &mut track_edit_rates);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Timed text extended validation (ST 2067-2 §10)
// ─────────────────────────────────────────────────────────────────────────────

/// Extended validation for DCTimedTextDescriptor beyond NamespaceURI.
///
/// ST 2067-2 §10: SampleRate should be present and consistent with the CPL EditRate.
/// Language tags in RFC5646LanguageTagList should be well-formed.
fn validate_timed_text_extended(
    cpl: &CompositionPlaylist,
    code: fn(CoreConstraintsCode) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let edl = match &cpl.essence_descriptor_list {
        Some(edl) => edl,
        None => return,
    };

    for ed in &edl.essence_descriptors {
        let tt = match &ed.dc_timed_text_descriptor {
            Some(tt) => tt,
            None => continue,
        };

        let ed_loc = Location::new()
            .with_cpl(cpl.id.to_string())
            .with_path(format!("EssenceDescriptor/{}", ed.id));

        // SampleRate should be present for timed text
        if tt.sample_rate.is_none() {
            issues.push(
                ValidationIssue::new(
                    Severity::Warning,
                    Category::Subtitle,
                    code(CoreConstraintsCode::TimedTextSampleRate),
                    format!(
                        "DCTimedTextDescriptor {}: SampleRate is missing; \
                         should be present for frame-accurate subtitle timing",
                        ed.id,
                    ),
                )
                .with_location(ed_loc.clone()),
            );
        }

        // Validate language tags look well-formed (basic structural check)
        for tag in &tt.rfc5646_language_tag_list {
            let s = &tag.0;
            if s.is_empty() {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        Category::Subtitle,
                        code(CoreConstraintsCode::TimedTextEmptyLanguageTag),
                        format!(
                            "DCTimedTextDescriptor {}: empty language tag in RFC5646LanguageTagList",
                            ed.id,
                        ),
                    )
                    .with_location(ed_loc.clone()),
                );
            } else if !s.chars().next().unwrap_or(' ').is_ascii_alphabetic() {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        Category::Subtitle,
                        code(CoreConstraintsCode::TimedTextMalformedLanguageTag),
                        format!(
                            "DCTimedTextDescriptor {}: language tag '{}' does not start with an ASCII letter (RFC 5646 primary subtag)",
                            ed.id, s,
                        ),
                    )
                    .with_location(ed_loc.clone()),
                );
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Audio MCA label validation
// ─────────────────────────────────────────────────────────────────────────────

/// Expected channel count for known soundfield groups.
fn expected_channel_count(tag: &crate::cpl::McaTagSymbol) -> Option<u32> {
    use crate::cpl::McaTagSymbol;
    match tag {
        McaTagSymbol::SgMono => Some(1),
        McaTagSymbol::SgSt => Some(2),
        McaTagSymbol::Sg51 => Some(6),
        McaTagSymbol::Sg71 | McaTagSymbol::Sg71Ds => Some(8),
        _ => None, // IAB, Other, channel labels — no fixed count
    }
}

/// Validate audio MCA (Multi-Channel Audio) labels in essence descriptors.
///
/// Checks per ST 377-4 / ST 2067-2:
/// - WAVEPCMDescriptor with ChannelCount > 0 should have MCA sub-descriptors
/// - SoundfieldGroupLabelSubDescriptor should have MCATagSymbol
/// - Soundfield channel count should be consistent with WAVEPCMDescriptor.ChannelCount
/// - Audio sample rate should be present
fn validate_audio_mca_labels(
    cpl: &CompositionPlaylist,
    code: fn(CoreConstraintsCode) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let edl = match &cpl.essence_descriptor_list {
        Some(edl) => edl,
        None => return,
    };

    for ed in &edl.essence_descriptors {
        let Some(ref wave) = ed.wave_pcm_descriptor else {
            continue;
        };

        let ed_loc = Location::new()
            .with_cpl(cpl.id.to_string())
            .with_path(format!("EssenceDescriptor/{}", ed.id));

        // Audio sample rate should be present
        if wave.audio_sample_rate.is_none() && wave.sample_rate.is_none() {
            issues.push(
                ValidationIssue::new(
                    Severity::Warning,
                    Category::Audio,
                    code(CoreConstraintsCode::AudioSampleRate),
                    format!(
                        "WAVEPCMDescriptor {} has no AudioSampleRate or SampleRate",
                        ed.id,
                    ),
                )
                .with_location(ed_loc.clone()),
            );
        }

        // ChannelCount should be present and > 0
        let channel_count = match wave.channel_count {
            Some(0) => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Audio,
                        code(CoreConstraintsCode::ChannelCount),
                        format!("WAVEPCMDescriptor {} has ChannelCount of 0", ed.id,),
                    )
                    .with_location(ed_loc.clone()),
                );
                continue;
            }
            Some(n) => n,
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        Category::Audio,
                        code(CoreConstraintsCode::ChannelCount),
                        format!("WAVEPCMDescriptor {} has no ChannelCount", ed.id,),
                    )
                    .with_location(ed_loc.clone()),
                );
                continue;
            }
        };

        // Check for MCA sub-descriptors
        let sub = match &wave.sub_descriptors {
            Some(sub) => sub,
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        Category::Audio,
                        code(CoreConstraintsCode::MCASubDescriptors),
                        format!(
                            "WAVEPCMDescriptor {} ({} channels) has no SubDescriptors; \
                             MCA labels are recommended for audio channel identification",
                            ed.id, channel_count,
                        ),
                    )
                    .with_location(ed_loc.clone()),
                );
                continue;
            }
        };

        let sf = match &sub.soundfield_group_label_sub_descriptor {
            Some(sf) => sf,
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        Category::Audio,
                        code(CoreConstraintsCode::SoundfieldGroup),
                        format!(
                            "WAVEPCMDescriptor {} ({} channels) has SubDescriptors \
                             but no SoundfieldGroupLabelSubDescriptor",
                            ed.id, channel_count,
                        ),
                    )
                    .with_location(ed_loc.clone()),
                );
                continue;
            }
        };

        // MCATagSymbol should be present
        let tag = match &sf.mca_tag_symbol {
            Some(tag) => tag,
            None => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        Category::Audio,
                        code(CoreConstraintsCode::MCATagSymbol),
                        format!(
                            "SoundfieldGroupLabelSubDescriptor for {} is missing MCATagSymbol",
                            ed.id,
                        ),
                    )
                    .with_location(ed_loc.clone()),
                );
                continue;
            }
        };

        // Channel count consistency with soundfield group
        if let Some(expected) = expected_channel_count(tag) {
            if channel_count != expected {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Audio,
                        code(CoreConstraintsCode::SoundfieldChannelCount),
                        format!(
                            "WAVEPCMDescriptor {} has ChannelCount {} but MCATagSymbol '{}' \
                             expects {} channels",
                            ed.id, channel_count, tag, expected,
                        ),
                    )
                    .with_location(ed_loc),
                );
            }
        }
    }
}

/// Validate timeline duration consistency within each segment.
///
/// Per ST 2067-2, within a single segment all virtual tracks (essence sequences)
/// must span the same total duration. The effective duration of a resource is:
///   effective = (SourceDuration or (IntrinsicDuration - EntryPoint)) × RepeatCount
fn validate_segment_track_durations(cpl: &CompositionPlaylist, issues: &mut Vec<ValidationIssue>) {
    use crate::cpl::SequenceAccess;

    /// Compute the total effective duration of a sequence in real seconds (f64).
    ///
    /// Each resource's frame count is divided by its edit rate (falling back to
    /// the CPL's edit rate when absent). This normalises across track types that
    /// use different edit rates (e.g. video at 24/1 and audio at 48000/1).
    /// Returns None if no edit rate is available to normalise with.
    fn sequence_duration_seconds(
        seq: &dyn SequenceAccess,
        cpl_edit_rate: Option<&EditRate>,
    ) -> Option<f64> {
        let total: f64 = seq
            .resource_list()
            .resources
            .iter()
            .map(|r| {
                let er = r.edit_rate.as_ref().or(cpl_edit_rate)?;
                let numer = er.numerator as f64;
                let denom = er.denominator as f64;
                if numer == 0.0 || denom == 0.0 {
                    return Some(0.0);
                }
                let fps = numer / denom;
                let entry = r.entry_point.unwrap_or(0) as f64;
                let dur = r
                    .source_duration
                    .map(|d| d as f64)
                    .unwrap_or_else(|| (r.intrinsic_duration as f64) - entry);
                let repeat = r.repeat_count.unwrap_or(1).max(1) as f64;
                Some((dur / fps) * repeat)
            })
            .try_fold(0.0_f64, |acc, v| v.map(|x| acc + x))?;
        Some(total)
    }

    // Two durations are considered equal if they differ by less than 1 ms.
    // This tolerates minor rounding in edit-rate arithmetic while still catching
    // genuine mismatches (which are always off by at least one full frame).
    const TOLERANCE_SECONDS: f64 = 0.001;

    let seg_dur_code: &'static str = match &cpl.namespace {
        CplNamespace::Dci429_7 | CplNamespace::Smpte2067_3_2013 => {
            St2067_3_2013::for_code(St2067_3Code::SegmentDuration)
        }
        CplNamespace::Smpte2067_3_2016 => St2067_3_2016::for_code(St2067_3Code::SegmentDuration),
        CplNamespace::Smpte2067_3_2020 | CplNamespace::Unknown(_) => {
            St2067_3_2020::for_code(St2067_3Code::SegmentDuration)
        }
    };

    for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
        let sl = &segment.sequence_list;
        let seg_loc = Location::new()
            .with_cpl(cpl.id.to_string())
            .with_segment(seg_idx);

        let er = cpl.edit_rate.as_ref();

        // Collect (track_type, track_id, duration_seconds) for all non-marker sequences.
        // Skip sequences where no edit rate is available to normalise with.
        let mut track_durations: Vec<(&str, String, f64)> = Vec::new();

        let push =
            |v: &mut Vec<(&str, String, f64)>, label: &'static str, seq: &dyn SequenceAccess| {
                if let Some(secs) = sequence_duration_seconds(seq, er) {
                    v.push((label, seq.track_id().to_string(), secs));
                }
            };

        for seq in &sl.main_image_sequences {
            push(&mut track_durations, "MainImage", seq);
        }
        for seq in &sl.main_audio_sequences {
            push(&mut track_durations, "MainAudio", seq);
        }
        for seq in &sl.subtitles_sequences {
            push(&mut track_durations, "Subtitles", seq);
        }
        for seq in &sl.hearing_impaired_captions_sequences {
            push(&mut track_durations, "HICaptions", seq);
        }
        for seq in &sl.forced_narrative_sequences {
            push(&mut track_durations, "ForcedNarrative", seq);
        }
        for seq in &sl.iab_sequences {
            push(&mut track_durations, "IAB", seq);
        }
        for seq in &sl.isxd_sequences {
            push(&mut track_durations, "ISXD", seq);
        }

        if track_durations.len() < 2 {
            continue; // Nothing to compare
        }

        // All tracks in a segment should have the same real-time duration
        let first_secs = track_durations[0].2;
        for (track_type, track_id, duration_secs) in &track_durations[1..] {
            if (duration_secs - first_secs).abs() > TOLERANCE_SECONDS {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Timing,
                        seg_dur_code,
                        format!(
                            "Segment {} {} track {}: duration {:.3}s differs from {} track {}: duration {:.3}s — \
                             all virtual tracks in a segment shall have equal duration",
                            seg_idx + 1,
                            track_type,
                            &track_id[..8.min(track_id.len())],
                            duration_secs,
                            track_durations[0].0,
                            &track_durations[0].1[..8.min(track_durations[0].1.len())],
                            first_secs,
                        ),
                    )
                    .with_location(seg_loc.clone()),
                );
            }
        }
    }
}

/// ST 2067-2 §8: Digital signature notice.
///
/// Digital signatures (XML-DSIG Signer/Signature elements) are not currently
/// parsed or validated. This outputs an info-level notice for CPLs using spec
/// versions that support digital signatures.
fn validate_digital_signature_notice(
    cpl: &CompositionPlaylist,
    code: fn(CoreConstraintsCode) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    // Digital signatures are supported from ST 2067-2:2016 onwards.
    // Since we don't parse Signer/Signature XML elements, we cannot validate them.
    // Output an info notice for awareness.
    let supports_signatures = matches!(
        cpl.namespace,
        CplNamespace::Smpte2067_3_2016 | CplNamespace::Smpte2067_3_2020
    );
    if supports_signatures {
        issues.push(
            ValidationIssue::new(
                Severity::Info,
                Category::Security,
                code(CoreConstraintsCode::DigitalSignature),
                "Digital signature validation (ST 2067-2 §8) is not currently performed; \
                 Signer/Signature XML elements are not parsed",
            )
            .with_location(Location::new().with_cpl(cpl.id.to_string())),
        );
    }
}

/// ST 2067-2 §6.4.2: Every EssenceDescriptor in the EssenceDescriptorList shall be
/// referenced by at least one Resource's SourceEncoding element.
///
/// An ED that exists in the EDL but is not referenced by any Resource is a
/// "dangling" descriptor — it occupies space and implies metadata but has no
/// corresponding essence in the composition. This is rejected by ST 2067-2.
///
/// Ported from: Photon `IMFCPLValidatorTest.invalidCPLdanglingED`.
fn validate_dangling_essence_descriptors(
    cpl: &CompositionPlaylist,
    code: fn(CoreConstraintsCode) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    let edl = match &cpl.essence_descriptor_list {
        Some(edl) => edl,
        None => return,
    };

    // Collect the set of all SourceEncoding UUIDs referenced by any Resource.
    let mut referenced: HashSet<String> = HashSet::new();

    for segment in &cpl.segment_list.segments {
        let sl = &segment.sequence_list;
        for seq in &sl.main_image_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref se) = r.source_encoding {
                    referenced.insert(se.to_string());
                }
            }
        }
        for seq in &sl.main_audio_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref se) = r.source_encoding {
                    referenced.insert(se.to_string());
                }
            }
        }
        for seq in &sl.subtitles_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref se) = r.source_encoding {
                    referenced.insert(se.to_string());
                }
            }
        }
        for seq in &sl.iab_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref se) = r.source_encoding {
                    referenced.insert(se.to_string());
                }
            }
        }
        for seq in &sl.hearing_impaired_captions_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref se) = r.source_encoding {
                    referenced.insert(se.to_string());
                }
            }
        }
        for seq in &sl.forced_narrative_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref se) = r.source_encoding {
                    referenced.insert(se.to_string());
                }
            }
        }
        for seq in &sl.isxd_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref se) = r.source_encoding {
                    referenced.insert(se.to_string());
                }
            }
        }
    }

    // Any ED whose ID is not in the referenced set is dangling.
    for ed in &edl.essence_descriptors {
        let id = ed.id.to_string();
        if !referenced.contains(&id) {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Reference,
                    code(CoreConstraintsCode::DanglingEssenceDescriptor),
                    format!(
                        "EssenceDescriptor {} is present in EssenceDescriptorList but not \
                         referenced by any Resource's SourceEncoding (ST 2067-2 §6.4.2)",
                        id
                    ),
                )
                .with_location(Location::new().with_cpl(cpl.id.to_string())),
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ST 2067-2:2020 Core Constraints
// ─────────────────────────────────────────────────────────────────────────────

/// ST 2067-2:2020 Core Constraints validator.
///
/// Strictest version: EssenceDescriptorList required, SourceEncoding references
/// must resolve, ApplicationIdentification required for application profiles.
pub struct CoreConstraints2020;

impl ConstraintsValidator for CoreConstraints2020 {
    fn spec_id(&self) -> &str {
        "ST 2067-2:2020"
    }

    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let loc = Location::new().with_cpl(cpl.id.to_string());

        validate_core_structure(
            cpl,
            crate::assetmap::codes::St2067_2_2020_Core::for_code,
            &mut issues,
        );

        // 2020-specific: EssenceDescriptorList is required
        if cpl.essence_descriptor_list.is_none() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Structure,
                    crate::assetmap::codes::St2067_2_2020_Core::for_code(
                        CoreConstraintsCode::EssenceDescriptorList,
                    ),
                    "EssenceDescriptorList is required per ST 2067-2:2020",
                )
                .with_location(loc),
            );
        }

        // SourceEncoding references are validated by st2067-3::validate_cpl_constraints
        // (called from validate_core_structure) for all sequence types.

        issues
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ST 2067-2:2016 Core Constraints
// ─────────────────────────────────────────────────────────────────────────────

/// ST 2067-2:2016 Core Constraints validator.
///
/// EssenceDescriptorList is required.
pub struct CoreConstraints2016;

impl ConstraintsValidator for CoreConstraints2016 {
    fn spec_id(&self) -> &str {
        "ST 2067-2:2016"
    }

    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        validate_core_structure(
            cpl,
            crate::assetmap::codes::St2067_2_2016_Core::for_code,
            &mut issues,
        );

        // 2016: EssenceDescriptorList is required (same as 2020)
        if cpl.essence_descriptor_list.is_none() {
            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    Category::Structure,
                    crate::assetmap::codes::St2067_2_2016_Core::for_code(
                        CoreConstraintsCode::EssenceDescriptorList,
                    ),
                    "EssenceDescriptorList is required per ST 2067-2:2016",
                )
                .with_location(Location::new().with_cpl(cpl.id.to_string())),
            );
        }

        // SourceEncoding references validated by st2067-3::validate_cpl_constraints.

        issues
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ST 2067-2:2013 Core Constraints
// ─────────────────────────────────────────────────────────────────────────────

/// ST 2067-2:2013 Core Constraints validator.
///
/// Most permissive version: EssenceDescriptorList is optional,
/// ApplicationIdentification does not exist.
pub struct CoreConstraints2013;

impl ConstraintsValidator for CoreConstraints2013 {
    fn spec_id(&self) -> &str {
        "ST 2067-2:2013"
    }

    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        validate_core_structure(
            cpl,
            crate::assetmap::codes::St2067_2_2013_Core::for_code,
            &mut issues,
        );

        // 2013: EssenceDescriptorList is OPTIONAL — no check
        // 2013: ApplicationIdentification does not exist — no check
        // SourceEncoding references validated by st2067-3::validate_cpl_constraints.

        issues
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Quantization — ST 2067-21:2023 Table 5 / Table 11 / Table 13
// ═════════════════════════════════════════════════════════════════════════════

/// Quantization system per ST 2067-21:2023 Table 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizationSystem {
    /// QE.1: Narrow range (BT.709/BT.601 quantization)
    /// R'G'B': D = INT((219 * X + 16) * 2^(n-8))
    Qe1,
    /// QE.2: Full range
    /// R'G'B': D = INT(X * (2^n - 1))
    Qe2,
}

/// Table 11: Component Max Ref and Component Min Ref values (RGBA).
///
/// Returns `(min_ref, max_ref)` for the given quantization system and bit depth.
pub fn component_ref_values(qe: QuantizationSystem, bit_depth: u32) -> Option<(u32, u32)> {
    match (qe, bit_depth) {
        // QE.1: narrow range
        (QuantizationSystem::Qe1, 8) => Some((16, 235)),
        (QuantizationSystem::Qe1, 10) => Some((64, 940)),
        (QuantizationSystem::Qe1, 12) => Some((256, 3760)),
        (QuantizationSystem::Qe1, 16) => Some((4096, 60160)),
        // QE.2: full range
        (QuantizationSystem::Qe2, 8) => Some((0, 255)),
        (QuantizationSystem::Qe2, 10) => Some((0, 1023)),
        (QuantizationSystem::Qe2, 12) => Some((0, 4095)),
        (QuantizationSystem::Qe2, 16) => Some((0, 65535)),
        _ => None,
    }
}

/// Table 13: Black Ref Level, White Ref Level, and Color Range (CDCI).
///
/// Returns `(black_ref, white_ref, color_range)` for the given color system and bit depth.
pub fn cdci_ref_values(color_sys: &ColorSystem, bit_depth: u32) -> Option<(u32, u32, u32)> {
    match (color_sys, bit_depth) {
        // COLOR.1, COLOR.2, COLOR.3
        (ColorSystem::Color1 | ColorSystem::Color2 | ColorSystem::Color3, 8) => {
            Some((16, 235, 225))
        }
        (ColorSystem::Color1 | ColorSystem::Color2 | ColorSystem::Color3, 10) => {
            Some((64, 940, 897))
        }
        (ColorSystem::Color1 | ColorSystem::Color2 | ColorSystem::Color3, 12) => {
            Some((256, 3760, 3585))
        }
        (ColorSystem::Color1 | ColorSystem::Color2 | ColorSystem::Color3, 16) => {
            Some((4096, 60160, 57345))
        }
        // COLOR.4
        (ColorSystem::Color4, 8) => Some((16, 235, 254)),
        (ColorSystem::Color4, 10) => Some((64, 940, 1013)),
        // COLOR.5, COLOR.7, COLOR.8
        (ColorSystem::Color5 | ColorSystem::Color7 | ColorSystem::Color8, 10) => {
            Some((64, 940, 897))
        }
        (ColorSystem::Color5 | ColorSystem::Color7 | ColorSystem::Color8, 12) => {
            Some((256, 3760, 3585))
        }
        (ColorSystem::Color5 | ColorSystem::Color7 | ColorSystem::Color8, 16) => {
            Some((4096, 60160, 57345))
        }
        _ => None,
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Application Profile #2E — ST 2067-21
//
// Validates image essence against Table 2 (allowed parameters) and
// Table 3 (named colorimetry systems).
//
// Mirrors Photon's `IMFApp2EConstraintsValidator` abstract base class.
// ═════════════════════════════════════════════════════════════════════════════

/// The exact ApplicationIdentification URI per ST 2067-21:2023 Table 15.
const APP2E_APPLICATION_IDENTIFICATION: &str = "http://www.smpte-ra.org/ns/2067-21/2021";

/// ST 2067-21:2023 Application Profile #2E Validator.
///
/// Validates:
/// - Table 8: Generic Picture Essence Descriptor constraints (FrameLayout, etc.)
///
/// Validate a J2K PictureEssenceCoding UL against the App2E profile constraints.
///
/// Ported from Photon `JPEG2000.java` + `IMFApp2E2021ConstraintsValidator.validateJ2KProfile()`.
///
/// Rules (§6.2.5):
/// - `Jpeg2000Ht`: allowed only when `allow_ht = true` (App2E from 2021 onward).
/// - `Jpeg2000Imf4k`: stored width must be 2049–4096, stored height ≤ 3112.
/// - `Jpeg2000Imf2k`: stored width must be 1–2048, stored height ≤ 1556.
/// - `Jpeg2000Broadcast`: stored width must be 1–3840, stored height ≤ 2160.
/// - `Jpeg2000` (generic node): no resolution bounds enforced.
/// - All other J2K variants (should not occur after CODEC_TABLE is correct): error.
fn validate_j2k_profile(
    codec: &crate::cpl::VideoCodec,
    stored_width: u32,
    stored_height: u32,
    allow_ht: bool,
    loc: &Location,
    issues: &mut Vec<ValidationIssue>,
) {
    use crate::cpl::VideoCodec;
    match codec {
        VideoCodec::Jpeg2000Ht => {
            if !allow_ht {
                // App2E 2020 does not include HT-J2K. Added in 2021.
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Encoding,
                        St2067_21_2023::J2KHtNotAllowed.code(),
                        "JPEG 2000 HT (ISO 15444-15) is not permitted by App2E 2020. \
                         HT-J2K was introduced in ST 2067-21:2021.",
                    )
                    .with_location(loc.clone()),
                );
            }
            // When allowed, no resolution bounds check for HT (Photon delegates to
            // validateHTConstraints which checks codestream parameters, not resolution).
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
                        St2067_21_2023::J2K4KResolution.code(),
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
                        St2067_21_2023::J2K2KResolution.code(),
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
                        St2067_21_2023::J2KBcpResolution.code(),
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

/// - Table 10: RGBA Descriptor constraints (ComponentMaxRef/MinRef, ScanningDirection)
/// - Table 11: Component Max/Min Ref values by quantization system and bit depth
/// - Table 12: CDCI Descriptor constraints (subsampling, siting, ref levels)
/// - Table 13: Black/White Ref Level and Color Range by colorimetry and bit depth
/// - Table 3: Colorimetry system cross-validation (COLOR.1 through COLOR.8)
/// - §6.2.5: JPEG 2000 codec requirement + J2K profile/resolution validation
/// - §7.1 / Table 15: ApplicationIdentification exact URI
/// - §7.2: Homogeneous image essence within a composition
/// - §7.5: MaxCLL/MaxFALL presence for PQ-based HDR (COLOR.6/COLOR.7)
pub struct App2E2021;

impl App2E2021 {
    /// Validate all image essence descriptors in the CPL.
    /// `allow_ht`: when false, HT-J2K is rejected (App2E 2020 mode).
    fn validate_image_descriptors(
        &self,
        cpl: &CompositionPlaylist,
        allow_ht: bool,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let edl = match &cpl.essence_descriptor_list {
            Some(edl) => edl,
            None => return, // Core constraints will catch missing EDL
        };

        for ed in &edl.essence_descriptors {
            if let Some(ref rgba) = ed.rgba_descriptor {
                self.validate_rgba_descriptor(&ed.id.to_string(), rgba, cpl, allow_ht, issues);
            }
            if let Some(ref cdci) = ed.cdci_descriptor {
                self.validate_cdci_descriptor(&ed.id.to_string(), cdci, cpl, allow_ht, issues);
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
        allow_ht: bool,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let loc = Location::new()
            .with_cpl(cpl.id.to_string())
            .with_path(format!("EssenceDescriptor[{}]/RGBADescriptor", ed_id));

        // ── Table 8: Generic Picture Essence Descriptor ──────────────────────

        // §6.2.5: PictureCompression shall be JPEG 2000 (ISO 15444-1 or 15444-15)
        if let Some(ref codec) = rgba.picture_compression {
            if !codec.is_jpeg2000_family() {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Encoding,
                        St2067_21_2023::J2KRequired.code().to_string(),
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
                validate_j2k_profile(codec, w, h, allow_ht, &loc, issues);
            }
        }

        // Table 8: FrameLayout — §6.2.1.3
        if let Some(ref fl) = rgba.frame_layout {
            if fl != "FullFrame" && fl != "SeparateFields" {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        St2067_21_2023::FrameLayout.code().to_string(),
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
                    St2067_21_2023::StoredF2Offset.code().to_string(),
                    "StoredF2Offset shall not be present (Table 8)",
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
                            St2067_21_2023::SampledWidth.code().to_string(),
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
                            St2067_21_2023::SampledHeight.code().to_string(),
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
                        St2067_21_2023::SampledXOffset.code().to_string(),
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
                        St2067_21_2023::SampledYOffset.code().to_string(),
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
                    St2067_21_2023::AlphaTransparency.code().to_string(),
                    "AlphaTransparency shall not be present (Table 8)",
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
                    St2067_21_2023::ImageAlignmentOffset.code().to_string(),
                    "ImageAlignmentOffset shall not be present (Table 8)",
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
                    St2067_21_2023::ImageStartOffset.code().to_string(),
                    "ImageStartOffset shall not be present (Table 8)",
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
                    St2067_21_2023::ImageEndOffset.code().to_string(),
                    "ImageEndOffset shall not be present (Table 8)",
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
                        St2067_21_2023::FieldDominance.code().to_string(),
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
                        St2067_21_2023::FieldDominance.code().to_string(),
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
                        St2067_21_2023::ColorPrimariesUnknown.code().to_string(),
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
                        St2067_21_2023::ColorPrimaries.code().to_string(),
                        "ColorPrimaries shall be present (Table 8)",
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
                        St2067_21_2023::TransferCharacteristicUnknown
                            .code()
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
                        St2067_21_2023::TransferCharacteristic.code().to_string(),
                        "TransferCharacteristic shall be present (Table 8)",
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
                    St2067_21_2023::ComponentMaxRef.code().to_string(),
                    "ComponentMaxRef shall be present (Table 10)",
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
                    St2067_21_2023::ComponentMinRef.code().to_string(),
                    "ComponentMinRef shall be present (Table 10)",
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
                        St2067_21_2023::ScanningDirection.code().to_string(),
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
                        St2067_21_2023::ScanningDirection.code().to_string(),
                        "ScanningDirection shall be present (Table 10)",
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
                    St2067_21_2023::AlphaMaxRef.code().to_string(),
                    "AlphaMaxRef shall not be present (Table 10)",
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
                    St2067_21_2023::AlphaMinRef.code().to_string(),
                    "AlphaMinRef shall not be present (Table 10)",
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
                    St2067_21_2023::Palette.code().to_string(),
                    "Palette shall not be present (Table 10)",
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
                    St2067_21_2023::PaletteLayout.code().to_string(),
                    "PaletteLayout shall not be present (Table 10)",
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
                                St2067_21_2023::ComponentRefValues.code().to_string(),
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
                        St2067_21_2023::ColorSystem.code().to_string(),
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

        // §7.5: MaxCLL/MaxFALL for PQ-based systems (optional per spec)
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
        allow_ht: bool,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let loc = Location::new()
            .with_cpl(cpl.id.to_string())
            .with_path(format!("EssenceDescriptor[{}]/CDCIDescriptor", ed_id));

        // ── Table 8: Generic Picture Essence Descriptor ──────────────────────

        // §6.2.5: PictureCompression shall be JPEG 2000
        if let Some(ref codec) = cdci.picture_compression {
            if !codec.is_jpeg2000_family() {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Encoding,
                        St2067_21_2023::J2KRequired.code().to_string(),
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
                validate_j2k_profile(codec, w, h, allow_ht, &loc, issues);
            }
        }

        // Table 8: FrameLayout — §6.2.1.3
        if let Some(ref fl) = cdci.frame_layout {
            if fl != "FullFrame" && fl != "SeparateFields" {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        St2067_21_2023::FrameLayout.code().to_string(),
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
                    St2067_21_2023::StoredF2Offset.code().to_string(),
                    "StoredF2Offset shall not be present (Table 8)",
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
                            St2067_21_2023::SampledWidth.code().to_string(),
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
                            St2067_21_2023::SampledHeight.code().to_string(),
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
                        St2067_21_2023::SampledXOffset.code().to_string(),
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
                        St2067_21_2023::SampledYOffset.code().to_string(),
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
                    St2067_21_2023::AlphaTransparency.code().to_string(),
                    "AlphaTransparency shall not be present (Table 8)",
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
                    St2067_21_2023::ImageAlignmentOffset.code().to_string(),
                    "ImageAlignmentOffset shall not be present (Table 8)",
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
                    St2067_21_2023::ImageStartOffset.code().to_string(),
                    "ImageStartOffset shall not be present (Table 8)",
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
                    St2067_21_2023::ImageEndOffset.code().to_string(),
                    "ImageEndOffset shall not be present (Table 8)",
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
                        St2067_21_2023::FieldDominance.code().to_string(),
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
                        St2067_21_2023::FieldDominance.code().to_string(),
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
                        St2067_21_2023::ColorPrimariesUnknown.code().to_string(),
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
                        St2067_21_2023::ColorPrimaries.code().to_string(),
                        "ColorPrimaries shall be present (Table 8)",
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
                        St2067_21_2023::TransferCharacteristicUnknown
                            .code()
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
                        St2067_21_2023::TransferCharacteristic.code().to_string(),
                        "TransferCharacteristic shall be present (Table 8)",
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
                        St2067_21_2023::CodingEquationsUnknown.code().to_string(),
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
                        St2067_21_2023::CodingEquations.code().to_string(),
                        "CodingEquations shall be present for Y'C'BC'R (Table 8)",
                    )
                    .with_location(loc.clone()),
                );
            }
            _ => {}
        }

        // ── Table 12: CDCI Descriptor ────────────────────────────────────────

        // Table 12: ComponentDepth shall be present and equal to pixel bit depth (8/10/12/16)
        match cdci.component_depth {
            Some(depth) if !matches!(depth, 8 | 10 | 12 | 16) => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        St2067_21_2023::ComponentDepth.code().to_string(),
                        format!(
                            "ComponentDepth {} is not allowed; shall be 8, 10, 12, or 16",
                            depth
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
                        St2067_21_2023::ComponentDepth.code().to_string(),
                        "ComponentDepth shall be present (Table 12)",
                    )
                    .with_location(loc.clone()),
                );
            }
            _ => {}
        }

        // Table 12: HorizontalSubsampling — §6.4.2
        // 1 = 4:4:4, 2 = 4:2:2
        match cdci.horizontal_subsampling {
            Some(hs) if hs != 1 && hs != 2 => {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        St2067_21_2023::HorizontalSubsampling.code().to_string(),
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
                        St2067_21_2023::HorizontalSubsampling.code().to_string(),
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
                        St2067_21_2023::VerticalSubsampling.code().to_string(),
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
                        St2067_21_2023::VerticalSubsampling.code().to_string(),
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
                        St2067_21_2023::ColorSiting.code().to_string(),
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
                        St2067_21_2023::ColorSiting.code().to_string(),
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
                    St2067_21_2023::ReversedByteOrder.code().to_string(),
                    "ReversedByteOrder shall not be present (Table 12)",
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
                    St2067_21_2023::PaddingBits.code().to_string(),
                    "PaddingBits shall not be present (Table 12)",
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
                    St2067_21_2023::AlphaSampleDepth.code().to_string(),
                    "AlphaSampleDepth shall not be present (Table 12)",
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
                                        St2067_21_2023::BlackRefLevel.code().to_string(),
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
                                        St2067_21_2023::WhiteRefLevel.code().to_string(),
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
                                        St2067_21_2023::ColorRange.code().to_string(),
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
                        St2067_21_2023::ColorSystem.code().to_string(),
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

        // §7.5: MaxCLL/MaxFALL for PQ-based systems (optional per spec)
        self.check_hdr_metadata(cdci.transfer_characteristic.as_ref(), cpl, &loc, issues);

        // ── Table 14: JPEG 2000 Picture Sub Descriptor ────────────────────────
        self.validate_j2k_sub_descriptor(
            cdci.sub_descriptors.as_ref(),
            cdci.picture_compression.as_ref(),
            &loc,
            issues,
        );
    }

    /// Section 7.5: MaxCLL and MaxFALL for PQ (ST 2084) content.
    ///
    /// Per ST 2067-21:2023 §7.5, MaxCLL and MaxFALL have 0..1 cardinality for
    /// COLOR.6/COLOR.7 (PQ-based systems). They are OPTIONAL, not required.
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
                        St2067_21_2023::MaxCLLMaxFALL.code().to_string(),
                        "MaxCLL and MaxFALL are not present for PQ (ST 2084) content; \
                         per §7.5 they are optional (0..1 cardinality)",
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
                        St2067_21_2023::Jpeg2000SubDescriptor.code().to_string(),
                        "JPEG2000SubDescriptor is missing; Table 14 requires this descriptor for JPEG 2000 picture essence",
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
                    St2067_21_2023::CodingStyle.code().to_string(),
                    "CodingStyleDefault (Coding Style) shall be present (Table 14)",
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
                    St2067_21_2023::J2CLayout.code().to_string(),
                    "J2CLayout shall be present (Table 14, §6.5.2)",
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
                        St2067_21_2023::J2KExtendedCapabilities.code().to_string(),
                        "J2KExtendedCapabilities shall be present when ISO/IEC 15444-15 coding is used (Table 14)",
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
                        St2067_21_2023::HomogeneousImageEssence.code(),
                        format!(
                            "Heterogeneous image essence: found {} different color systems ({}); \
                             all image essence in a composition shall use the same color system",
                            unique.len(),
                            systems.join(", ")
                        ),
                    )
                    .with_location(Location::new().with_cpl(cpl.id.to_string())),
                );
            }
        }
    }

    /// §7.4 Segment Duration: If the average number of audio samples per Composition
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
                        St2067_21_2023::SegmentDurationMultiple.code().to_string(),
                        format!(
                            "Segment {} duration ({} edit units) is not a multiple of 5; \
                             §7.4 requires segment duration to be an integer multiple of \
                             5/Composition Edit Rate when audio samples per edit unit is non-integer",
                            seg_idx + 1,
                            segment_duration,
                        ),
                    )
                    .with_location(
                        Location::new()
                            .with_cpl(cpl.id.to_string())
                            .with_segment(seg_idx),
                    ),
                );
            }
        }
    }

    /// Common App2E validation rules, parameterised by `allow_ht`.
    ///
    /// Called by both `App2E2021::validate_cpl` (allow_ht = true) and
    /// `App2E2020::validate_cpl` (allow_ht = false). This avoids duplicating
    /// the complete rule set for the 2020 vs 2021 HT-J2K difference.
    pub fn validate_all(
        &self,
        cpl: &CompositionPlaylist,
        allow_ht: bool,
        issues: &mut Vec<ValidationIssue>,
    ) {
        // Validate image descriptors (Tables 8/10/11/12/13)
        self.validate_image_descriptors(cpl, allow_ht, issues);
        // §8.2: Homogeneous image essence
        self.validate_homogeneous_image_essence(cpl, issues);
        // §7.4: Segment duration alignment for non-integer audio sample counts
        self.validate_segment_duration(cpl, issues);
        // §6.5: Audio quantization bits
        self.validate_audio_quantization(cpl, issues);
        // Tables 4-6: Image resolution
        self.validate_image_resolution(cpl, issues);
        // Tables 4-6: Image frame rate
        self.validate_image_frame_rate(cpl, issues);
        // §6.5: Audio sample rate (48000 Hz)
        self.validate_audio_sample_rate(cpl, issues);
        // §6.2: Essence descriptor completeness
        self.validate_descriptor_completeness(cpl, issues);
        // §6.2.1: FrameLayout shall be FullFrame (progressive)
        self.validate_frame_layout_progressive(cpl, issues);
        // §7.3: Audio channel layout homogeneity within a composition
        self.validate_audio_channel_homogeneity(cpl, issues);
        // §5.6: HearingImpaired/ForcedNarrative captions shall use timed text
        self.validate_caption_track_constraints(cpl, issues);
        // §5.1.3: ContentMaturityRating shall use recognized agency/rating pairs
        self.validate_content_maturity_rating(cpl, issues);
        // §5.3: LocaleList language/region validation
        self.validate_locale_list(cpl, issues);
    }
}

impl ConstraintsValidator for App2E2021 {
    fn spec_id(&self) -> &str {
        "ST 2067-21:2023 (App2E)"
    }

    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // §7.1 / Table 15: ApplicationIdentification shall be the exact URI
        if let Some(ref ext) = cpl.extension_properties {
            if let Some(ref app_id) = ext.application_identification {
                if app_id != APP2E_APPLICATION_IDENTIFICATION {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Warning,
                            Category::Metadata,
                            St2067_21_2023::AppIdMismatch.code().to_string(),
                            format!(
                                "ApplicationIdentification '{}' does not match Table 15 value '{}'",
                                app_id, APP2E_APPLICATION_IDENTIFICATION
                            ),
                        )
                        .with_location(Location::new().with_cpl(cpl.id.to_string())),
                    );
                }
            }
        }

        self.validate_all(
            cpl,
            true, /* HT-J2K permitted in App2E 2021 */
            &mut issues,
        );

        issues
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// App2E 2020 Validator
// ═════════════════════════════════════════════════════════════════════════════

/// The exact ApplicationIdentification URI per ST 2067-21:2020 Table 15.
const APP2E_2020_APPLICATION_IDENTIFICATION: &str = "http://www.smpte-ra.org/ns/2067-21/2020";

/// ST 2067-21:2020 Application Profile #2E Validator.
///
/// Identical to App2E 2021 except HT-J2K (ISO 15444-15) is **not** permitted.
/// Per Photon `IMFApp2E2020ConstraintsValidator`, HT was only added in 2021.
pub struct App2E2020;

impl ConstraintsValidator for App2E2020 {
    fn spec_id(&self) -> &str {
        "ST 2067-21:2020 (App2E)"
    }

    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // §7.1 / Table 15: ApplicationIdentification shall be the exact URI
        if let Some(ref ext) = cpl.extension_properties {
            if let Some(ref app_id) = ext.application_identification {
                if app_id != APP2E_2020_APPLICATION_IDENTIFICATION {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Warning,
                            Category::Metadata,
                            St2067_21_2020::AppIdMismatch.code().to_string(),
                            format!(
                                "ApplicationIdentification '{}' does not match Table 15 value '{}'",
                                app_id, APP2E_2020_APPLICATION_IDENTIFICATION
                            ),
                        )
                        .with_location(Location::new().with_cpl(cpl.id.to_string())),
                    );
                }
            }
        }

        App2E2021.validate_all(
            cpl,
            false, /* HT-J2K not permitted in App2E 2020 */
            &mut issues,
        );

        issues
    }
}

impl App2E2021 {
    /// ST 2067-21 §6.5: QuantizationBits shall be 16 or 24.
    fn validate_audio_quantization(
        &self,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let edl = match &cpl.essence_descriptor_list {
            Some(edl) => edl,
            None => return,
        };

        for ed in &edl.essence_descriptors {
            if let Some(ref wave) = ed.wave_pcm_descriptor {
                if let Some(qb) = wave.quantization_bits {
                    if qb != 16 && qb != 24 {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                Category::Audio,
                                St2067_21_2023::QuantizationBits.code().to_string(),
                                format!(
                                    "WAVEPCMDescriptor {} has QuantizationBits {} \
                                     but ST 2067-21 §6.5 requires 16 or 24",
                                    ed.id, qb,
                                ),
                            )
                            .with_location(
                                Location::new()
                                    .with_cpl(cpl.id.to_string())
                                    .with_path(format!("EssenceDescriptor/{}", ed.id)),
                            ),
                        );
                    }
                }
            }
        }
    }
}

// ── ST 2067-21 Tables 4-6: Image system constraints ─────────────────────────

/// Allowed image system tiers per ST 2067-21 Tables 4-6.
///
/// Each tier defines valid (StoredWidth, StoredHeight) pairs and the set of
/// allowed frame rates (as EditRate numerator/denominator pairs).
struct ImageSystemTier {
    name: &'static str,
    dimensions: &'static [(u32, u32)],
}

/// ST 2067-21 Tables 4-6: Allowed frame rates for all image system tiers.
///
/// These are common across 2K, 4K, and 8K tiers.
const ALLOWED_FRAME_RATES: &[(u32, u32)] = &[
    (24, 1),       // 24 fps
    (24000, 1001), // 23.976 fps
    (25, 1),       // 25 fps
    (30, 1),       // 30 fps
    (30000, 1001), // 29.97 fps
    (48, 1),       // 48 fps (HFR)
    (48000, 1001), // 47.952 fps (HFR)
    (50, 1),       // 50 fps (HFR)
    (60, 1),       // 60 fps (HFR)
    (60000, 1001), // 59.94 fps (HFR)
];

/// All defined image system tiers.
const IMAGE_SYSTEM_TIERS: &[ImageSystemTier] = &[
    ImageSystemTier {
        name: "2K (Table 4)",
        dimensions: &[(1920, 1080), (2048, 1080)],
    },
    ImageSystemTier {
        name: "4K (Table 5)",
        dimensions: &[(3840, 2160), (4096, 2160)],
    },
    ImageSystemTier {
        name: "8K (Table 6)",
        dimensions: &[(7680, 4320)],
    },
];

impl App2E2021 {
    /// ST 2067-21 §5.2: Validate image resolution (StoredWidth × StoredHeight).
    ///
    /// The stored dimensions must match one of the allowed image system tiers.
    fn validate_image_resolution(
        &self,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let edl = match &cpl.essence_descriptor_list {
            Some(edl) => edl,
            None => return,
        };

        let all_allowed: Vec<(u32, u32)> = IMAGE_SYSTEM_TIERS
            .iter()
            .flat_map(|t| t.dimensions.iter().copied())
            .collect();

        for ed in &edl.essence_descriptors {
            let (width, height, desc_type) = if let Some(ref rgba) = ed.rgba_descriptor {
                (rgba.stored_width, rgba.stored_height, "RGBA")
            } else if let Some(ref cdci) = ed.cdci_descriptor {
                (cdci.stored_width, cdci.stored_height, "CDCI")
            } else {
                continue;
            };

            let (w, h) = match (width, height) {
                (Some(w), Some(h)) => (w, h),
                _ => continue, // Completeness validation will catch missing fields
            };

            if !all_allowed.contains(&(w, h)) {
                let tier_names: Vec<&str> = IMAGE_SYSTEM_TIERS.iter().map(|t| t.name).collect();
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        St2067_21_2023::Resolution.code().to_string(),
                        format!(
                            "{} descriptor {}: StoredWidth×StoredHeight {}×{} is not an allowed App2E image system dimension (allowed tiers: {})",
                            desc_type, ed.id, w, h, tier_names.join(", ")
                        ),
                    )
                    .with_location(
                        Location::new()
                            .with_cpl(cpl.id.to_string())
                            .with_path(format!("EssenceDescriptor/{}", ed.id)),
                    ),
                );
            }
        }
    }

    /// ST 2067-21 §5.2: Validate image frame rate (SampleRate on descriptor).
    ///
    /// The descriptor's SampleRate must be one of the allowed edit rates.
    fn validate_image_frame_rate(
        &self,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let edl = match &cpl.essence_descriptor_list {
            Some(edl) => edl,
            None => return,
        };

        for ed in &edl.essence_descriptors {
            let (sample_rate, desc_type) = if let Some(ref rgba) = ed.rgba_descriptor {
                (rgba.sample_rate.as_ref(), "RGBA")
            } else if let Some(ref cdci) = ed.cdci_descriptor {
                (cdci.sample_rate.as_ref(), "CDCI")
            } else {
                continue;
            };

            let rate = match sample_rate {
                Some(r) => r,
                None => continue, // Completeness validation will catch missing SampleRate
            };

            let is_allowed = ALLOWED_FRAME_RATES
                .iter()
                .any(|&(n, d)| rate.numerator == n && rate.denominator == d);

            if !is_allowed {
                let allowed_str: Vec<String> = ALLOWED_FRAME_RATES
                    .iter()
                    .map(|&(n, d)| {
                        let fps = n as f64 / d as f64;
                        format!("{:.3}", fps)
                    })
                    .collect();
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Video,
                        St2067_21_2023::FrameRate.code().to_string(),
                        format!(
                            "{} descriptor {}: SampleRate {}/{} ({:.3} fps) is not an allowed App2E frame rate (allowed: {} fps)",
                            desc_type, ed.id, rate.numerator, rate.denominator,
                            rate.as_f64(),
                            allowed_str.join(", ")
                        ),
                    )
                    .with_location(
                        Location::new()
                            .with_cpl(cpl.id.to_string())
                            .with_path(format!("EssenceDescriptor/{}", ed.id)),
                    ),
                );
            }
        }
    }

    /// ST 2067-21 §6.5: AudioSampleRate shall be 48000 Hz.
    fn validate_audio_sample_rate(
        &self,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let edl = match &cpl.essence_descriptor_list {
            Some(edl) => edl,
            None => return,
        };

        for ed in &edl.essence_descriptors {
            let wave = match &ed.wave_pcm_descriptor {
                Some(w) => w,
                None => continue,
            };

            if let Some(ref rate) = wave.audio_sample_rate {
                // 48000 Hz = 48000/1
                if rate.numerator != 48000 || rate.denominator != 1 {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Audio,
                            St2067_21_2023::AudioSampleRate.code().to_string(),
                            format!(
                                "WAVEPCMDescriptor {}: AudioSampleRate {}/{} ({} Hz) is not 48000 Hz",
                                ed.id, rate.numerator, rate.denominator,
                                rate.numerator / rate.denominator.max(1)
                            ),
                        )
                        .with_location(
                            Location::new()
                                .with_cpl(cpl.id.to_string())
                                .with_path(format!("EssenceDescriptor/{}", ed.id)),
                        ),
                    );
                }
            }
        }
    }

    /// ST 2067-21 §6.2: Validate that required fields are present on essence descriptors.
    ///
    /// Image descriptors (RGBA/CDCI) must have: StoredWidth, StoredHeight, SampleRate,
    /// FrameLayout, ColorPrimaries, TransferCharacteristic, PictureCompression.
    /// Audio descriptors (WAVE PCM) must have: ChannelCount, QuantizationBits.
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
                .with_cpl(cpl.id.to_string())
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
                    push(St2067_21_2023::RequiredStoredWidth.code(), "StoredWidth");
                }
                if rgba.stored_height.is_none() {
                    push(St2067_21_2023::RequiredStoredHeight.code(), "StoredHeight");
                }
                if rgba.sample_rate.is_none() {
                    push(St2067_21_2023::RequiredSampleRate.code(), "SampleRate");
                }
                if rgba.frame_layout.is_none() {
                    push(St2067_21_2023::RequiredFrameLayout.code(), "FrameLayout");
                }
                if rgba.color_primaries.is_none() {
                    push(
                        St2067_21_2023::RequiredColorPrimaries.code(),
                        "ColorPrimaries",
                    );
                }
                if rgba.transfer_characteristic.is_none() {
                    push(
                        St2067_21_2023::RequiredTransferCharacteristic.code(),
                        "TransferCharacteristic",
                    );
                }
                if rgba.picture_compression.is_none() {
                    push(
                        St2067_21_2023::RequiredPictureCompression.code(),
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
                    push(St2067_21_2023::RequiredStoredWidth.code(), "StoredWidth");
                }
                if cdci.stored_height.is_none() {
                    push(St2067_21_2023::RequiredStoredHeight.code(), "StoredHeight");
                }
                if cdci.sample_rate.is_none() {
                    push(St2067_21_2023::RequiredSampleRate.code(), "SampleRate");
                }
                if cdci.frame_layout.is_none() {
                    push(St2067_21_2023::RequiredFrameLayout.code(), "FrameLayout");
                }
                if cdci.color_primaries.is_none() {
                    push(
                        St2067_21_2023::RequiredColorPrimaries.code(),
                        "ColorPrimaries",
                    );
                }
                if cdci.transfer_characteristic.is_none() {
                    push(
                        St2067_21_2023::RequiredTransferCharacteristic.code(),
                        "TransferCharacteristic",
                    );
                }
                if cdci.picture_compression.is_none() {
                    push(
                        St2067_21_2023::RequiredPictureCompression.code(),
                        "PictureCompression",
                    );
                }
                if cdci.component_depth.is_none() {
                    push(
                        St2067_21_2023::RequiredComponentDepth.code(),
                        "ComponentDepth",
                    );
                }
            }

            if let Some(ref wave) = ed.wave_pcm_descriptor {
                let mut push = |code: &'static str, field: &'static str| {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Audio,
                            code.to_string(),
                            format!(
                                "WAVEPCMDescriptor {}: required field {} is missing",
                                ed.id, field
                            ),
                        )
                        .with_location(ed_loc.clone()),
                    );
                };
                if wave.channel_count.is_none() {
                    push(St2067_21_2023::RequiredChannelCount.code(), "ChannelCount");
                }
                if wave.quantization_bits.is_none() {
                    push(
                        St2067_21_2023::RequiredQuantizationBits.code(),
                        "QuantizationBits",
                    );
                }
            }
        }
    }

    /// ST 2067-21 §6.2.1: FrameLayout shall be FullFrame (progressive) for App2E.
    ///
    /// All image systems defined in Tables 4-6 (2K, 4K, 8K) are progressive-scan only.
    /// SeparateFields (interlaced) is not permitted for App2E content.
    fn validate_frame_layout_progressive(
        &self,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        let edl = match &cpl.essence_descriptor_list {
            Some(edl) => edl,
            None => return,
        };

        for ed in &edl.essence_descriptors {
            let (frame_layout, desc_type) = if let Some(ref rgba) = ed.rgba_descriptor {
                (rgba.frame_layout.as_deref(), "RGBA")
            } else if let Some(ref cdci) = ed.cdci_descriptor {
                (cdci.frame_layout.as_deref(), "CDCI")
            } else {
                continue;
            };

            if let Some(fl) = frame_layout {
                if fl != "FullFrame" {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            Category::Video,
                            St2067_21_2023::FrameLayoutInterlaced.code().to_string(),
                            format!(
                                "{} descriptor {}: FrameLayout '{}' is not permitted for App2E; \
                                 all image systems (Tables 4-6) shall be FullFrame (progressive)",
                                desc_type, ed.id, fl,
                            ),
                        )
                        .with_location(
                            Location::new()
                                .with_cpl(cpl.id.to_string())
                                .with_path(format!("EssenceDescriptor/{}", ed.id)),
                        )
                        .with_suggestion("Set FrameLayout to FullFrame (00h)"),
                    );
                }
            }
        }
    }

    /// ST 2067-21 §7.3 homogeneity applies *within* a virtual track: all resources
    /// in one virtual track must reference equivalent essence descriptors (same
    /// SourceEncoding). It does NOT require all audio virtual tracks in the CPL
    /// to have the same channel count — a CPL with separate 5.1 and stereo tracks
    /// is completely valid and normal.
    ///
    /// The previous implementation compared channel counts across *all*
    /// WAVEPCMDescriptors in the EssenceDescriptorList, which produced false
    /// positives for every multi-track CPL. That check is removed.
    ///
    /// True per-virtual-track descriptor homogeneity is enforced by the
    /// `6.9/Homogeneity` SourceEncoding check in CoreConstraints.
    #[allow(clippy::ptr_arg)]
    fn validate_audio_channel_homogeneity(
        &self,
        _cpl: &CompositionPlaylist,
        _issues: &mut Vec<ValidationIssue>,
    ) {
        // Intentionally empty — see doc comment above.
    }

    /// ST 2067-21 §5.6: HearingImpairedCaptions and ForcedNarrative sequences
    /// shall reference timed text essence descriptors.
    fn validate_caption_track_constraints(
        &self,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        // Build map from EssenceDescriptor ID → has DCTimedTextDescriptor
        let ed_map: HashMap<String, bool> = cpl
            .essence_descriptor_list
            .as_ref()
            .map(|edl| {
                edl.essence_descriptors
                    .iter()
                    .map(|ed| (ed.id.to_string(), ed.dc_timed_text_descriptor.is_some()))
                    .collect()
            })
            .unwrap_or_default();

        // Check HIC sequences
        for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
            for seq in &segment.sequence_list.hearing_impaired_captions_sequences {
                for (res_idx, resource) in seq.resource_list.resources.iter().enumerate() {
                    if let Some(ref se) = resource.source_encoding {
                        let se_str = se.to_string();
                        if let Some(&has_ttml) = ed_map.get(&se_str) {
                            if !has_ttml {
                                issues.push(
                                    ValidationIssue::new(
                                        Severity::Error,
                                        Category::Subtitle,
                                        St2067_21_2025::HICTimedText.code().to_string(),
                                        format!(
                                            "HearingImpairedCaptions resource {} references descriptor '{}' \
                                             which is not a DCTimedTextDescriptor",
                                            res_idx, se_str,
                                        ),
                                    )
                                    .with_location(
                                        Location::new()
                                            .with_cpl(cpl.id.to_string())
                                            .with_segment(seg_idx)
                                            .with_resource(res_idx),
                                    ),
                                );
                            }
                        }
                    }
                }
            }

            // Check ForcedNarrative sequences
            for seq in &segment.sequence_list.forced_narrative_sequences {
                for (res_idx, resource) in seq.resource_list.resources.iter().enumerate() {
                    if let Some(ref se) = resource.source_encoding {
                        let se_str = se.to_string();
                        if let Some(&has_ttml) = ed_map.get(&se_str) {
                            if !has_ttml {
                                issues.push(
                                    ValidationIssue::new(
                                        Severity::Error,
                                        Category::Subtitle,
                                        St2067_21_2025::FNTimedText.code().to_string(),
                                        format!(
                                            "ForcedNarrative resource {} references descriptor '{}' \
                                             which is not a DCTimedTextDescriptor",
                                            res_idx, se_str,
                                        ),
                                    )
                                    .with_location(
                                        Location::new()
                                            .with_cpl(cpl.id.to_string())
                                            .with_segment(seg_idx)
                                            .with_resource(res_idx),
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// ST 2067-21 §5.1.3: ContentMaturityRating — if present, agency must be non-empty.
    /// Also validates ApplicationIdentification is present (required for App2E).
    fn validate_content_maturity_rating(
        &self,
        cpl: &CompositionPlaylist,
        issues: &mut Vec<ValidationIssue>,
    ) {
        // App2E requires ApplicationIdentification when ExtensionProperties is present.
        // If ExtensionProperties is absent, the check is deferred (CPL may lack it
        // because it's being tested for other properties only).
        if let Some(ref ext) = cpl.extension_properties {
            if ext.application_identification.is_none() {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Metadata,
                        St2067_21_2023::ApplicationIdentification.code(),
                        "ApplicationIdentification is required for App2E compositions",
                    )
                    .with_location(Location::new().with_cpl(cpl.id.to_string())),
                );
            }
        }

        // Validate ContentMaturityRating entries in each Locale
        if let Some(ref locale_list) = cpl.locale_list {
            for (locale_idx, locale) in locale_list.locales.iter().enumerate() {
                if let Some(ref cmr_list) = locale.content_maturity_rating_list {
                    for (rating_idx, rating) in cmr_list.ratings.iter().enumerate() {
                        // Agency is required and must be non-empty
                        if rating.agency.trim().is_empty() {
                            issues.push(
                                ValidationIssue::new(
                                    Severity::Error,
                                    Category::Metadata,
                                    St2067_21_2023::ContentMaturityRatingAgency.code(),
                                    format!(
                                        "ContentMaturityRating[{}] in Locale[{}] has empty Agency",
                                        rating_idx, locale_idx
                                    ),
                                )
                                .with_location(Location::new().with_cpl(cpl.id.to_string())),
                            );
                        } else if !is_valid_any_uri(&rating.agency) {
                            // XSD: Agency is xs:anyURI — no ASCII whitespace allowed
                            issues.push(
                                ValidationIssue::new(
                                    Severity::Error,
                                    Category::Metadata,
                                    St2067_21_2023::ContentMaturityRatingAgencyUri.code(),
                                    format!(
                                        "ContentMaturityRating[{}] in Locale[{}] Agency '{}' \
                                         is not a valid xs:anyURI (contains whitespace)",
                                        rating_idx, locale_idx, rating.agency,
                                    ),
                                )
                                .with_location(Location::new().with_cpl(cpl.id.to_string())),
                            );
                        }
                    }
                }
            }
        }
    }

    /// ST 2067-21 §5.3: Validate LocaleList language tags and region codes.
    ///
    /// Language tags should be well-formed RFC 5646 (start with alpha).
    /// Region codes should be well-formed ISO 3166-1 alpha-2 (2 uppercase letters).
    fn validate_locale_list(&self, cpl: &CompositionPlaylist, issues: &mut Vec<ValidationIssue>) {
        let locale_list = match &cpl.locale_list {
            Some(ll) => ll,
            None => return,
        };

        let cpl_loc = Location::new().with_cpl(cpl.id.to_string());

        for (i, locale) in locale_list.locales.iter().enumerate() {
            // Validate language tags
            if let Some(ref ll) = locale.language_list {
                for tag in &ll.languages {
                    let s = &tag.0;
                    if s.is_empty() {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Warning,
                                Category::Metadata,
                                St2067_21_2023::EmptyLanguageTag.code().to_string(),
                                format!("Locale[{}]: empty language tag in LanguageList", i),
                            )
                            .with_location(cpl_loc.clone()),
                        );
                    } else if !s.chars().next().unwrap_or(' ').is_ascii_alphabetic() {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Warning,
                                Category::Metadata,
                                St2067_21_2023::MalformedLanguageTag.code().to_string(),
                                format!(
                                    "Locale[{}]: language tag '{}' does not start with an ASCII letter (RFC 5646)",
                                    i, s,
                                ),
                            )
                            .with_location(cpl_loc.clone()),
                        );
                    }
                }
            }

            // Validate region codes: BCP 47 §2.2.4 — 2-letter ISO 3166-1 or 3-digit UN M.49
            if let Some(ref rl) = locale.region_list {
                for region in &rl.regions {
                    let is_alpha2 =
                        region.len() == 2 && region.chars().all(|c| c.is_ascii_uppercase());
                    let is_un_m49 = region.len() == 3 && region.chars().all(|c| c.is_ascii_digit());
                    if !is_alpha2 && !is_un_m49 {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Warning,
                                Category::Metadata,
                                St2067_21_2023::RegionCode.code().to_string(),
                                format!(
                                    "Locale[{}]: region '{}' is not a valid BCP 47 region subtag (expected 2-letter ISO 3166-1 or 3-digit UN M.49)",
                                    i, region,
                                ),
                            )
                            .with_location(cpl_loc.clone()),
                        );
                    }
                }
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Tests
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assetmap::ImfUuid;
    use crate::cpl::{
        CDCIDescriptor, CompositionTimecode, ContentKindElement, ContentMaturityRating,
        ContentMaturityRatingList, ContentVersion, ContentVersionList, DCTimedTextDescriptor,
        EssenceDescriptor, EssenceDescriptorList, ExtensionProperties, ForcedNarrativeSequence,
        HearingImpairedCaptionsSequence, LanguageList, LanguageString, Locale, LocaleList,
        MainAudioSequence, MainImageSequence, MarkerInfo, MarkerLabelElement, MarkerSequence,
        RGBADescriptor, RegionList, Resource, ResourceList, Segment, SegmentList, SequenceList,
        SubtitlesSequence, WAVEPCMDescriptor,
    };
    use crate::cpl::{ContentKind, EditRate, LanguageTag, MarkerLabel, VideoCodec};
    use crate::diagnostics::codes::ValidationCode;
    use crate::validation::codes::St2067_21_2023;

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn uuid(n: u8) -> ImfUuid {
        ImfUuid::parse(&format!("00000000-0000-0000-0000-{:012}", n)).unwrap()
    }

    fn empty_sequence_list() -> SequenceList {
        SequenceList {
            marker_sequences: vec![],
            main_image_sequences: vec![],
            main_audio_sequences: vec![],
            subtitles_sequences: vec![],
            hearing_impaired_captions_sequences: vec![],
            forced_narrative_sequences: vec![],
            iab_sequences: vec![],
            isxd_sequences: vec![],
        }
    }

    fn make_resource(source_encoding: Option<ImfUuid>) -> Resource {
        Resource {
            id: uuid(99),
            annotation: None,
            edit_rate: None,
            intrinsic_duration: 100,
            entry_point: None,
            source_duration: None,
            source_encoding,
            track_file_id: Some(uuid(50)),
            repeat_count: None,
            key_id: None,
            hash: None,
            markers: vec![],
        }
    }

    fn minimal_cpl() -> CompositionPlaylist {
        CompositionPlaylist {
            namespace: CplNamespace::Smpte2067_3_2020,
            id: uuid(1),
            annotation: None,
            issue_date: "2024-01-01T00:00:00Z".to_string(),
            issuer: None,
            creator: None,
            content_originator: None,
            content_title: LanguageString {
                text: "Test".to_string(),
                language: Some(LanguageTag("en".to_string())),
            },
            content_kind: ContentKindElement::from(ContentKind::Feature),
            content_version_list: None,
            locale_list: None,
            essence_descriptor_list: None,
            edit_rate: None,
            total_running_time: None,
            extension_properties: None,
            composition_timecode: None,
            segment_list: SegmentList {
                segments: vec![Segment {
                    id: uuid(2),
                    sequence_list: empty_sequence_list(),
                }],
            },
            has_signer: false,
            has_signature: false,
        }
    }

    fn cpl_with_cdci_descriptor(
        primaries: ColorPrimaries,
        transfer: TransferCharacteristic,
        coding_eq: CodingEquations,
        depth: u32,
    ) -> CompositionPlaylist {
        let ed_id = uuid(10);
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: ed_id,
                rgba_descriptor: None,
                cdci_descriptor: Some(CDCIDescriptor {
                    instance_id: None,
                    stored_width: Some(1920),
                    stored_height: Some(1080),
                    display_width: Some(1920),
                    display_height: Some(1080),
                    sample_rate: Some(EditRate::new(24, 1)),
                    image_aspect_ratio: None,
                    color_primaries: Some(primaries),
                    transfer_characteristic: Some(transfer),
                    coding_equations: Some(coding_eq),
                    picture_compression: Some(VideoCodec::Jpeg2000),
                    component_depth: Some(depth),
                    frame_layout: Some("FullFrame".to_string()),
                    display_f2_offset: None,
                    horizontal_subsampling: Some(2),
                    vertical_subsampling: Some(1),
                    color_siting: Some(0),
                    black_ref_level: Some(64),
                    white_ref_level: Some(940),
                    color_range: Some(897),
                    stored_f2_offset: None,
                    sampled_width: None,
                    sampled_height: None,
                    sampled_x_offset: None,
                    sampled_y_offset: None,
                    alpha_transparency: None,
                    image_alignment_offset: None,
                    image_start_offset: None,
                    image_end_offset: None,
                    field_dominance: None,
                    reversed_byte_order: None,
                    padding_bits: None,
                    alpha_sample_depth: None,
                    linked_track_id: None,
                    active_width: None,
                    active_height: None,
                    sub_descriptors: None,
                }),
                wave_pcm_descriptor: None,
                dc_timed_text_descriptor: None,
                iab_essence_descriptor: None,
                isxd_data_essence_descriptor: None,
            }],
        });
        let mut sl = empty_sequence_list();
        sl.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: uuid(4),
            resource_list: ResourceList {
                resources: vec![make_resource(Some(ed_id))],
            },
        });
        sl.main_audio_sequences.push(MainAudioSequence {
            id: uuid(5),
            track_id: uuid(6),
            resource_list: ResourceList {
                resources: vec![make_resource(Some(uuid(11)))],
            },
        });
        cpl.segment_list.segments[0].sequence_list = sl;
        cpl.essence_descriptor_list
            .as_mut()
            .unwrap()
            .essence_descriptors
            .push(EssenceDescriptor {
                id: uuid(11),
                rgba_descriptor: None,
                cdci_descriptor: None,
                wave_pcm_descriptor: Some(WAVEPCMDescriptor {
                    instance_id: None,
                    sample_rate: None,
                    audio_sample_rate: None,
                    channel_count: Some(6),
                    quantization_bits: Some(24),
                    linked_track_id: None,
                    sub_descriptors: None,
                }),
                dc_timed_text_descriptor: None,
                iab_essence_descriptor: None,
                isxd_data_essence_descriptor: None,
            });
        cpl
    }

    fn cpl_with_rgba_descriptor(
        primaries: ColorPrimaries,
        transfer: TransferCharacteristic,
    ) -> CompositionPlaylist {
        let ed_id = uuid(10);
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: ed_id,
                rgba_descriptor: Some(RGBADescriptor {
                    instance_id: None,
                    display_width: Some(1920),
                    display_height: Some(1080),
                    stored_width: Some(1920),
                    stored_height: Some(1080),
                    sample_rate: Some(EditRate::new(24, 1)),
                    image_aspect_ratio: None,
                    color_primaries: Some(primaries),
                    transfer_characteristic: Some(transfer),
                    coding_equations: None,
                    picture_compression: Some(VideoCodec::Jpeg2000Ht),
                    frame_layout: Some("FullFrame".to_string()),
                    display_f2_offset: None,
                    component_max_ref: Some(1023),
                    component_min_ref: Some(0),
                    scanning_direction: Some(
                        "ScanningDirection_LeftToRightTopToBottom".to_string(),
                    ),
                    stored_f2_offset: None,
                    sampled_width: None,
                    sampled_height: None,
                    sampled_x_offset: None,
                    sampled_y_offset: None,
                    alpha_transparency: None,
                    image_alignment_offset: None,
                    image_start_offset: None,
                    image_end_offset: None,
                    field_dominance: None,
                    alpha_max_ref: None,
                    alpha_min_ref: None,
                    palette: None,
                    palette_layout: None,
                    linked_track_id: None,
                    sub_descriptors: None,
                }),
                cdci_descriptor: None,
                wave_pcm_descriptor: None,
                dc_timed_text_descriptor: None,
                iab_essence_descriptor: None,
                isxd_data_essence_descriptor: None,
            }],
        });
        let mut sl = empty_sequence_list();
        sl.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: uuid(4),
            resource_list: ResourceList {
                resources: vec![make_resource(Some(ed_id))],
            },
        });
        cpl.segment_list.segments[0].sequence_list = sl;
        cpl
    }

    // ── Trait + Factory Tests ────────────────────────────────────────────────

    #[test]
    fn factory_returns_core_2020_for_namespace() {
        let v = get_validator("http://www.smpte-ra.org/ns/2067-2/2020");
        assert!(v.is_some());
        assert_eq!(v.unwrap().spec_id(), "ST 2067-2:2020");
    }

    #[test]
    fn factory_returns_core_2016_for_namespace() {
        let v = get_validator("http://www.smpte-ra.org/schemas/2067-2/2016");
        assert!(v.is_some());
        assert_eq!(v.unwrap().spec_id(), "ST 2067-2:2016");
    }

    #[test]
    fn factory_returns_core_2013_for_namespace() {
        let v = get_validator("http://www.smpte-ra.org/schemas/2067-2/2013");
        assert!(v.is_some());
        assert_eq!(v.unwrap().spec_id(), "ST 2067-2:2013");
    }

    #[test]
    fn factory_returns_app2e_for_2021_namespace() {
        let v = get_validator("http://www.smpte-ra.org/ns/2067-21/2021");
        assert!(v.is_some());
        assert_eq!(v.unwrap().spec_id(), "ST 2067-21:2023 (App2E)");
    }

    #[test]
    fn factory_returns_app2e_for_2023_namespace() {
        let v = get_validator("http://www.smpte-ra.org/ns/2067-21/2023");
        assert!(v.is_some());
        assert_eq!(v.unwrap().spec_id(), "ST 2067-21:2023 (App2E)");
    }

    #[test]
    fn factory_returns_none_for_unknown() {
        assert!(get_validator("http://example.com/unknown").is_none());
    }

    #[test]
    fn registry_resolves_core_namespace() {
        let registry = BuiltinValidatorRegistry;
        let v = registry.resolve_namespace("http://www.smpte-ra.org/ns/2067-2/2020");
        assert!(v.is_some());
        assert_eq!(v.unwrap().spec_id(), "ST 2067-2:2020");
    }

    #[test]
    fn registry_returns_none_for_unknown_namespace() {
        let registry = BuiltinValidatorRegistry;
        assert!(registry
            .resolve_namespace("http://example.com/unknown")
            .is_none());
    }

    #[test]
    fn get_validators_for_cpl_returns_core_2020() {
        let cpl = minimal_cpl();
        let validators = get_validators_for_cpl(&cpl);
        assert_eq!(validators.len(), 1);
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2020");
    }

    #[test]
    fn get_validators_for_cpl_returns_core_plus_app2e() {
        let mut cpl = minimal_cpl();
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some("http://www.smpte-ra.org/ns/2067-21/2021".to_string()),
            max_cll: None,
            max_fall: None,
        });
        let validators = get_validators_for_cpl(&cpl);
        assert_eq!(validators.len(), 2);
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2020");
        assert_eq!(validators[1].spec_id(), "ST 2067-21:2023 (App2E)");
    }

    #[test]
    fn registry_resolve_for_cpl_handles_multiple_app_uris() {
        let mut cpl = minimal_cpl();
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some(
                "http://www.smpte-ra.org/ns/2067-21/2021 http://example.com/unknown".to_string(),
            ),
            max_cll: None,
            max_fall: None,
        });
        let registry = BuiltinValidatorRegistry;
        let validators = registry.resolve_for_cpl(&cpl);
        assert_eq!(validators.len(), 2);
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2020");
        assert_eq!(validators[1].spec_id(), "ST 2067-21:2023 (App2E)");
    }

    #[test]
    fn registry_and_factory_have_same_resolution() {
        let mut cpl = minimal_cpl();
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some("http://www.smpte-ra.org/ns/2067-21/2021".to_string()),
            max_cll: None,
            max_fall: None,
        });

        let via_factory = get_validators_for_cpl(&cpl);
        let registry = BuiltinValidatorRegistry;
        let via_registry = registry.resolve_for_cpl(&cpl);

        let factory_ids: Vec<_> = via_factory
            .iter()
            .map(|v| v.spec_id().to_string())
            .collect();
        let registry_ids: Vec<_> = via_registry
            .iter()
            .map(|v| v.spec_id().to_string())
            .collect();
        assert_eq!(factory_ids, registry_ids);
    }

    #[test]
    fn configurable_registry_overrides_core_namespace_version() {
        let cpl = minimal_cpl();
        let registry = ConfigurableValidatorRegistry::new(ValidatorSelection {
            core_spec: Some(CoreSpecTarget::St2067_2_2016),
            ..Default::default()
        });

        let validators = registry.resolve_for_cpl(&cpl);
        assert_eq!(validators.len(), 1);
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2016");
    }

    #[test]
    fn configurable_registry_overrides_application_identification_uris() {
        let cpl = minimal_cpl();
        let registry = ConfigurableValidatorRegistry::new(ValidatorSelection {
            app_specs: Some(vec![AppSpecTarget::St2067_21_2021]),
            ..Default::default()
        });

        let validators = registry.resolve_for_cpl(&cpl);
        assert_eq!(validators.len(), 2);
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2020");
        assert_eq!(validators[1].spec_id(), "ST 2067-21:2023 (App2E)");
    }

    #[test]
    fn validate_cpl_with_registry_matches_manual_merge() {
        let mut cpl = minimal_cpl();
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some("http://www.smpte-ra.org/ns/2067-21/2021".to_string()),
            max_cll: None,
            max_fall: None,
        });

        let registry = BuiltinValidatorRegistry;
        let via_helper = validate_cpl_with_registry(&cpl, &registry);

        let validators = registry.resolve_for_cpl(&cpl);
        let mut manual = Vec::new();
        for v in &validators {
            manual.extend(v.validate_cpl(&cpl));
        }

        assert_eq!(via_helper.len(), manual.len());
    }

    // ── Registry auto-detection — older namespaces ───────────────────────────

    #[test]
    fn get_validators_for_cpl_returns_core_2016_for_2016_namespace() {
        let mut cpl = minimal_cpl();
        cpl.namespace = CplNamespace::Smpte2067_3_2016;
        let validators = get_validators_for_cpl(&cpl);
        assert_eq!(validators.len(), 1);
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2016");
    }

    #[test]
    fn get_validators_for_cpl_returns_core_2013_for_2013_namespace() {
        let mut cpl = minimal_cpl();
        cpl.namespace = CplNamespace::Smpte2067_3_2013;
        let validators = get_validators_for_cpl(&cpl);
        assert_eq!(validators.len(), 1);
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2013");
    }

    #[test]
    fn get_validators_for_cpl_returns_empty_for_dci_legacy_namespace() {
        // DCI-era CPLs (ST 429-7:2006) have no matching core validator.
        let mut cpl = minimal_cpl();
        cpl.namespace = CplNamespace::Dci429_7;
        let validators = get_validators_for_cpl(&cpl);
        assert_eq!(
            validators.len(),
            0,
            "DCI namespace should yield no validators"
        );
    }

    #[test]
    fn get_validators_for_cpl_auto_detects_old_style_app2e_2016_namespace() {
        // Old-style App2E URIs using the "schemas" path (pre-2020) should also resolve.
        let mut cpl = minimal_cpl();
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some(
                "http://www.smpte-ra.org/schemas/2067-21/2016".to_string(),
            ),
            max_cll: None,
            max_fall: None,
        });
        let validators = get_validators_for_cpl(&cpl);
        assert_eq!(validators.len(), 2);
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2020");
        assert_eq!(validators[1].spec_id(), "ST 2067-21:2023 (App2E)");
    }

    #[test]
    fn configurable_registry_empty_app_specs_suppresses_app_profile() {
        // --app2e-spec none: empty vec means no app validators even if CPL declares one.
        let mut cpl = minimal_cpl();
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some("http://www.smpte-ra.org/ns/2067-21/2021".to_string()),
            max_cll: None,
            max_fall: None,
        });
        let registry = ConfigurableValidatorRegistry::new(ValidatorSelection {
            app_specs: Some(vec![]),
            ..Default::default()
        });
        let validators = registry.resolve_for_cpl(&cpl);
        assert_eq!(
            validators.len(),
            1,
            "empty app_specs should suppress app profile"
        );
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2020");
    }

    #[test]
    fn configurable_registry_raw_string_core_uri_override() {
        // Raw string URI override for callers that need to target an unlisted namespace.
        let mut cpl = minimal_cpl();
        cpl.namespace = CplNamespace::Smpte2067_3_2020;
        let registry = ConfigurableValidatorRegistry::new(ValidatorSelection {
            core_namespace_uri: Some("http://www.smpte-ra.org/schemas/2067-2/2016".to_string()),
            ..Default::default()
        });
        let validators = registry.resolve_for_cpl(&cpl);
        assert_eq!(validators.len(), 1);
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2016");
    }

    #[test]
    fn configurable_registry_raw_string_app_uri_override() {
        let cpl = minimal_cpl();
        let registry = ConfigurableValidatorRegistry::new(ValidatorSelection {
            application_identification_uris: Some(vec![
                "http://www.smpte-ra.org/ns/2067-21/2021".to_string()
            ]),
            ..Default::default()
        });
        let validators = registry.resolve_for_cpl(&cpl);
        assert_eq!(validators.len(), 2);
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2020");
        assert_eq!(validators[1].spec_id(), "ST 2067-21:2023 (App2E)");
    }

    // ── ColorSystem Tests ────────────────────────────────────────────────────

    #[test]
    fn color_system_color1_bt601_625() {
        let cs = ColorSystem::from_components(
            &ColorPrimaries::Bt601_625,
            &TransferCharacteristic::Bt709,
            Some(&CodingEquations::Bt601),
        );
        assert_eq!(cs, Some(ColorSystem::Color1));
        assert!(!cs.unwrap().is_hdr());
    }

    #[test]
    fn color_system_color2_bt601_525() {
        let cs = ColorSystem::from_components(
            &ColorPrimaries::Bt601_525,
            &TransferCharacteristic::Bt709,
            Some(&CodingEquations::Bt601),
        );
        assert_eq!(cs, Some(ColorSystem::Color2));
    }

    #[test]
    fn color_system_color3_bt709() {
        let cs = ColorSystem::from_components(
            &ColorPrimaries::Bt709,
            &TransferCharacteristic::Bt709,
            Some(&CodingEquations::Bt709),
        );
        assert_eq!(cs, Some(ColorSystem::Color3));
        assert!(!cs.unwrap().is_hdr());
    }

    #[test]
    fn color_system_color4_xvycc() {
        let cs = ColorSystem::from_components(
            &ColorPrimaries::Bt709,
            &TransferCharacteristic::XvYcc709,
            Some(&CodingEquations::Bt709),
        );
        assert_eq!(cs, Some(ColorSystem::Color4));
    }

    #[test]
    fn color_system_color5_bt2020_sdr() {
        let cs = ColorSystem::from_components(
            &ColorPrimaries::Bt2020,
            &TransferCharacteristic::Bt2020,
            Some(&CodingEquations::Bt2020Ncl),
        );
        assert_eq!(cs, Some(ColorSystem::Color5));
        assert!(!cs.unwrap().is_hdr());
    }

    #[test]
    fn color_system_color6_p3_pq_rgb() {
        let cs = ColorSystem::from_components(
            &ColorPrimaries::P3D65,
            &TransferCharacteristic::PqSt2084,
            None,
        );
        assert_eq!(cs, Some(ColorSystem::Color6));
        assert!(cs.unwrap().is_hdr());
        assert!(cs.unwrap().requires_hdr_metadata());
    }

    #[test]
    fn color_system_color7_bt2020_pq() {
        let cs = ColorSystem::from_components(
            &ColorPrimaries::Bt2020,
            &TransferCharacteristic::PqSt2084,
            Some(&CodingEquations::Bt2020Ncl),
        );
        assert_eq!(cs, Some(ColorSystem::Color7));
        assert!(cs.unwrap().is_hdr());
        assert!(cs.unwrap().requires_hdr_metadata());
    }

    #[test]
    fn color_system_color8_hlg() {
        let cs = ColorSystem::from_components(
            &ColorPrimaries::Bt2020,
            &TransferCharacteristic::Hlg,
            Some(&CodingEquations::Bt2020Ncl),
        );
        assert_eq!(cs, Some(ColorSystem::Color8));
        assert!(cs.unwrap().is_hdr());
        assert!(!cs.unwrap().requires_hdr_metadata());
    }

    #[test]
    fn color_system_invalid_combination_returns_none() {
        // BT.709 primaries + PQ transfer + BT.601 coding = not a valid system
        let cs = ColorSystem::from_components(
            &ColorPrimaries::Bt709,
            &TransferCharacteristic::PqSt2084,
            Some(&CodingEquations::Bt601),
        );
        assert_eq!(cs, None);
    }

    // ── Core Constraints Tests ───────────────────────────────────────────────

    #[test]
    fn core_2020_requires_essence_descriptor_list() {
        let cpl = minimal_cpl(); // no EDL
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        let edl_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.code.contains("EssenceDescriptorList"))
            .collect();
        assert!(
            !edl_issues.is_empty(),
            "Should flag missing EssenceDescriptorList"
        );
    }

    #[test]
    fn core_2013_allows_missing_essence_descriptor_list() {
        let mut cpl = minimal_cpl();
        cpl.namespace = CplNamespace::Smpte2067_3_2013;
        // Add a sequence so no "empty segment" error
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(3),
                track_id: uuid(4),
                resource_list: ResourceList {
                    resources: vec![make_resource(None)],
                },
            });
        let v = CoreConstraints2013;
        let issues = v.validate_cpl(&cpl);
        let edl_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.code.contains("EssenceDescriptorList"))
            .collect();
        assert!(
            edl_issues.is_empty(),
            "2013 should not require EssenceDescriptorList"
        );
    }

    #[test]
    fn core_flags_empty_content_title() {
        let mut cpl = minimal_cpl();
        cpl.content_title.text = "".to_string();
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("ContentTitle")),
            "Should flag empty ContentTitle"
        );
    }

    #[test]
    fn core_flags_empty_segment_list() {
        let mut cpl = minimal_cpl();
        cpl.segment_list.segments.clear();
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("SegmentList")),
            "Should flag empty SegmentList"
        );
    }

    #[test]
    fn core_flags_segment_with_no_sequences() {
        let cpl = minimal_cpl(); // has one segment with empty sequence list
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("Segment")),
            "Should flag segment with no sequences"
        );
    }

    #[test]
    fn core_2020_flags_unresolved_source_encoding() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        // Remove the audio descriptor to create a dangling SourceEncoding reference
        cpl.essence_descriptor_list
            .as_mut()
            .unwrap()
            .essence_descriptors
            .retain(|ed| ed.wave_pcm_descriptor.is_none());
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("SourceEncoding")),
            "Should flag unresolved SourceEncoding"
        );
    }

    // ── App2E Tests ──────────────────────────────────────────────────────────

    #[test]
    fn app2e_validates_valid_color3_hd() {
        let cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        // COLOR.3 with 10-bit J2K should have no App2E errors
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Error || i.severity == Severity::Critical)
            .collect();
        assert!(
            errors.is_empty(),
            "Valid COLOR.3 HD should pass App2E: {:?}",
            errors
        );
    }

    #[test]
    fn app2e_flags_invalid_color_system() {
        // BT.709 primaries + PQ transfer + BT.601 coding = not a valid system
        let cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::PqSt2084,
            CodingEquations::Bt601,
            10,
        );
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("6.2/ColorSystem")),
            "Should flag invalid color system combination"
        );
    }

    #[test]
    fn app2e_flags_non_j2k_codec() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        // Change codec to H.265
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.picture_compression = Some(VideoCodec::H265);
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("6.2.5")),
            "Should flag non-JPEG-2000 codec"
        );
    }

    #[test]
    fn app2e_flags_invalid_bit_depth() {
        let cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            14, // Not an allowed depth
        );
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("6.4/ComponentDepth")),
            "Should flag invalid bit depth"
        );
    }

    #[test]
    fn app2e_notes_missing_hdr_metadata_for_pq() {
        // COLOR.7 (BT.2020 + PQ + BT.2020 NCL) — MaxCLL/MaxFALL are optional (0..1)
        // per ST 2067-21:2023 §7.5, so absence is Info, not Error.
        let cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt2020,
            TransferCharacteristic::PqSt2084,
            CodingEquations::Bt2020Ncl,
            10,
        );
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        let hdr_issues: Vec<_> = issues.iter().filter(|i| i.code.contains("7.5")).collect();
        assert!(
            !hdr_issues.is_empty(),
            "Should note absent MaxCLL/MaxFALL for PQ content"
        );
        assert!(
            hdr_issues.iter().all(|i| i.severity == Severity::Info),
            "Missing MaxCLL/MaxFALL should be Info, not Error (0..1 cardinality per §7.5)"
        );
    }

    #[test]
    fn app2e_passes_hdr_with_metadata() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt2020,
            TransferCharacteristic::PqSt2084,
            CodingEquations::Bt2020Ncl,
            10,
        );
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some("http://www.smpte-ra.org/ns/2067-21/2021".to_string()),
            max_cll: Some(1000),
            max_fall: Some(400),
        });
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("7.5")),
            "Should not flag HDR metadata when MaxCLL/MaxFALL are present"
        );
    }

    #[test]
    fn app2e_hlg_does_not_require_hdr_metadata() {
        // COLOR.8 (HLG) does NOT require MaxCLL/MaxFALL
        let cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt2020,
            TransferCharacteristic::Hlg,
            CodingEquations::Bt2020Ncl,
            10,
        );
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("8.3.3")),
            "HLG should not require MaxCLL/MaxFALL"
        );
    }

    #[test]
    fn app2e_flags_missing_color_primaries() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        // Remove color primaries
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.color_primaries = None;
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.2.1/ColorPrimaries")),
            "Should flag missing ColorPrimaries"
        );
    }

    #[test]
    fn app2e_flags_heterogeneous_color_systems() {
        let ed_id_1 = uuid(10);
        let ed_id_2 = uuid(11);
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![
                // First descriptor: COLOR.3 (BT.709)
                EssenceDescriptor {
                    id: ed_id_1,
                    rgba_descriptor: None,
                    cdci_descriptor: Some(CDCIDescriptor {
                        instance_id: None,
                        stored_width: Some(1920),
                        stored_height: Some(1080),
                        display_width: Some(1920),
                        display_height: Some(1080),
                        sample_rate: None,
                        image_aspect_ratio: None,
                        color_primaries: Some(ColorPrimaries::Bt709),
                        transfer_characteristic: Some(TransferCharacteristic::Bt709),
                        coding_equations: Some(CodingEquations::Bt709),
                        picture_compression: Some(VideoCodec::Jpeg2000),
                        component_depth: Some(10),
                        frame_layout: Some("FullFrame".to_string()),
                        display_f2_offset: None,
                        horizontal_subsampling: Some(2),
                        vertical_subsampling: Some(1),
                        color_siting: Some(0),
                        black_ref_level: Some(64),
                        white_ref_level: Some(940),
                        color_range: Some(897),
                        stored_f2_offset: None,
                        sampled_width: None,
                        sampled_height: None,
                        sampled_x_offset: None,
                        sampled_y_offset: None,
                        alpha_transparency: None,
                        image_alignment_offset: None,
                        image_start_offset: None,
                        image_end_offset: None,
                        field_dominance: None,
                        reversed_byte_order: None,
                        padding_bits: None,
                        alpha_sample_depth: None,
                        linked_track_id: None,
                        active_width: None,
                        active_height: None,
                        sub_descriptors: None,
                    }),
                    wave_pcm_descriptor: None,
                    dc_timed_text_descriptor: None,
                    iab_essence_descriptor: None,
                    isxd_data_essence_descriptor: None,
                },
                // Second descriptor: COLOR.5 (BT.2020 SDR)
                EssenceDescriptor {
                    id: ed_id_2,
                    rgba_descriptor: None,
                    cdci_descriptor: Some(CDCIDescriptor {
                        instance_id: None,
                        stored_width: Some(3840),
                        stored_height: Some(2160),
                        display_width: Some(3840),
                        display_height: Some(2160),
                        sample_rate: None,
                        image_aspect_ratio: None,
                        color_primaries: Some(ColorPrimaries::Bt2020),
                        transfer_characteristic: Some(TransferCharacteristic::Bt2020),
                        coding_equations: Some(CodingEquations::Bt2020Ncl),
                        picture_compression: Some(VideoCodec::Jpeg2000),
                        component_depth: Some(10),
                        frame_layout: Some("FullFrame".to_string()),
                        display_f2_offset: None,
                        horizontal_subsampling: Some(2),
                        vertical_subsampling: Some(1),
                        color_siting: Some(0),
                        black_ref_level: Some(64),
                        white_ref_level: Some(940),
                        color_range: Some(897),
                        stored_f2_offset: None,
                        sampled_width: None,
                        sampled_height: None,
                        sampled_x_offset: None,
                        sampled_y_offset: None,
                        alpha_transparency: None,
                        image_alignment_offset: None,
                        image_start_offset: None,
                        image_end_offset: None,
                        field_dominance: None,
                        reversed_byte_order: None,
                        padding_bits: None,
                        alpha_sample_depth: None,
                        linked_track_id: None,
                        active_width: None,
                        active_height: None,
                        sub_descriptors: None,
                    }),
                    wave_pcm_descriptor: None,
                    dc_timed_text_descriptor: None,
                    iab_essence_descriptor: None,
                    isxd_data_essence_descriptor: None,
                },
            ],
        });
        let mut sl = empty_sequence_list();
        sl.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: uuid(4),
            resource_list: ResourceList {
                resources: vec![make_resource(Some(ed_id_1))],
            },
        });
        cpl.segment_list.segments[0].sequence_list = sl;

        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("7.2/HomogeneousImageEssence")),
            "Should flag heterogeneous color systems"
        );
    }

    // ── Integration: validate_cpl ────────────────────────────────────────────

    #[test]
    fn validate_cpl_dispatches_both_core_and_app2e() {
        let mut cpl = minimal_cpl(); // No EDL → core will flag it
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some("http://www.smpte-ra.org/ns/2067-21/2021".to_string()),
            max_cll: None,
            max_fall: None,
        });
        let validators = get_validators_for_cpl(&cpl);
        assert_eq!(validators.len(), 2, "Should dispatch both core and app2e");
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2020");
        assert_eq!(validators[1].spec_id(), "ST 2067-21:2023 (App2E)");

        // validate_cpl should run both and merge issues
        let issues = validate_cpl(&cpl);
        let has_core = issues.iter().any(|i| i.code.starts_with("ST2067-2:2020:"));
        assert!(
            has_core,
            "Core validator should produce issues for CPL without EDL"
        );
    }

    #[test]
    fn color_system_display() {
        assert_eq!(
            ColorSystem::Color3.to_string(),
            "COLOR.3 (BT.709 / BT.709 / BT.709)"
        );
        assert_eq!(
            ColorSystem::Color7.to_string(),
            "COLOR.7 (BT.2020 / PQ / BT.2020 NCL)"
        );
    }

    #[test]
    fn app2e_validates_j2k_sub_with_all_table14_fields() {
        use crate::cpl::{
            J2CLayout, J2KExtendedCapabilities, JPEG2000SubDescriptor, RGBADescriptor,
            RGBALayoutComponent, VideoSubDescriptors,
        };

        let ed_id = uuid(10);
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: ed_id,
                rgba_descriptor: Some(RGBADescriptor {
                    instance_id: None,
                    display_width: Some(1920),
                    display_height: Some(1080),
                    stored_width: Some(1920),
                    stored_height: Some(1080),
                    sample_rate: Some(EditRate::new(24, 1)),
                    image_aspect_ratio: None,
                    color_primaries: Some(ColorPrimaries::P3D65),
                    transfer_characteristic: Some(TransferCharacteristic::PqSt2084),
                    coding_equations: None,
                    picture_compression: Some(VideoCodec::Jpeg2000Ht),
                    frame_layout: Some("FullFrame".to_string()),
                    display_f2_offset: None,
                    component_max_ref: Some(1023),
                    component_min_ref: Some(0),
                    scanning_direction: Some(
                        "ScanningDirection_LeftToRightTopToBottom".to_string(),
                    ),
                    stored_f2_offset: None,
                    sampled_width: None,
                    sampled_height: None,
                    sampled_x_offset: None,
                    sampled_y_offset: None,
                    alpha_transparency: None,
                    image_alignment_offset: None,
                    image_start_offset: None,
                    image_end_offset: None,
                    field_dominance: None,
                    alpha_max_ref: None,
                    alpha_min_ref: None,
                    palette: None,
                    palette_layout: None,
                    linked_track_id: None,
                    sub_descriptors: Some(VideoSubDescriptors {
                        phdr_metadata_track_sub_descriptor: None,
                        jpeg2000_sub_descriptor: Some(JPEG2000SubDescriptor {
                            instance_id: None,
                            rsiz: Some(16384), // HTJ2K
                            xsiz: Some(1920),
                            ysiz: Some(1080),
                            xo_siz: Some(0),
                            yo_siz: Some(0),
                            xt_siz: Some(1920),
                            yt_siz: Some(1080),
                            xto_siz: Some(0),
                            yto_siz: Some(0),
                            csiz: Some(3),
                            coding_style_default: Some("01020001".to_string()),
                            quantization_default: Some("2060".to_string()),
                            j2c_layout: Some(J2CLayout {
                                components: vec![
                                    RGBALayoutComponent {
                                        code: "CompRed".to_string(),
                                        component_size: 10,
                                    },
                                    RGBALayoutComponent {
                                        code: "CompGreen".to_string(),
                                        component_size: 10,
                                    },
                                    RGBALayoutComponent {
                                        code: "CompBlue".to_string(),
                                        component_size: 10,
                                    },
                                ],
                            }),
                            j2k_extended_capabilities: Some(J2KExtendedCapabilities {
                                pcap: Some(131072),
                            }),
                            picture_component_sizing: None,
                        }),
                    }),
                }),
                cdci_descriptor: None,
                wave_pcm_descriptor: None,
                dc_timed_text_descriptor: None,
                iab_essence_descriptor: None,
                isxd_data_essence_descriptor: None,
            }],
        });
        let mut sl = empty_sequence_list();
        sl.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: uuid(4),
            resource_list: ResourceList {
                resources: vec![make_resource(Some(ed_id))],
            },
        });
        cpl.segment_list.segments[0].sequence_list = sl;

        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == Severity::Error || i.severity == Severity::Critical)
            .collect();
        assert!(
            errors.is_empty(),
            "Valid J2K sub descriptor should pass: {:?}",
            errors
        );
    }

    #[test]
    fn app2e_flags_j2k_missing_coding_style() {
        use crate::cpl::{
            J2CLayout, J2KExtendedCapabilities, JPEG2000SubDescriptor, RGBADescriptor,
            RGBALayoutComponent, VideoSubDescriptors,
        };

        let ed_id = uuid(10);
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: ed_id,
                rgba_descriptor: Some(RGBADescriptor {
                    instance_id: None,
                    display_width: Some(1920),
                    display_height: Some(1080),
                    stored_width: Some(1920),
                    stored_height: Some(1080),
                    sample_rate: Some(EditRate::new(24, 1)),
                    image_aspect_ratio: None,
                    color_primaries: Some(ColorPrimaries::P3D65),
                    transfer_characteristic: Some(TransferCharacteristic::PqSt2084),
                    coding_equations: None,
                    picture_compression: Some(VideoCodec::Jpeg2000Ht),
                    frame_layout: Some("FullFrame".to_string()),
                    display_f2_offset: None,
                    component_max_ref: Some(1023),
                    component_min_ref: Some(0),
                    scanning_direction: Some(
                        "ScanningDirection_LeftToRightTopToBottom".to_string(),
                    ),
                    stored_f2_offset: None,
                    sampled_width: None,
                    sampled_height: None,
                    sampled_x_offset: None,
                    sampled_y_offset: None,
                    alpha_transparency: None,
                    image_alignment_offset: None,
                    image_start_offset: None,
                    image_end_offset: None,
                    field_dominance: None,
                    alpha_max_ref: None,
                    alpha_min_ref: None,
                    palette: None,
                    palette_layout: None,
                    linked_track_id: None,
                    sub_descriptors: Some(VideoSubDescriptors {
                        phdr_metadata_track_sub_descriptor: None,
                        jpeg2000_sub_descriptor: Some(JPEG2000SubDescriptor {
                            instance_id: None,
                            rsiz: Some(16384),
                            xsiz: Some(1920),
                            ysiz: Some(1080),
                            xo_siz: None,
                            yo_siz: None,
                            xt_siz: None,
                            yt_siz: None,
                            xto_siz: None,
                            yto_siz: None,
                            csiz: None,
                            coding_style_default: None, // Missing!
                            quantization_default: None,
                            j2c_layout: Some(J2CLayout {
                                components: vec![RGBALayoutComponent {
                                    code: "CompRed".to_string(),
                                    component_size: 10,
                                }],
                            }),
                            j2k_extended_capabilities: Some(J2KExtendedCapabilities {
                                pcap: Some(131072),
                            }),
                            picture_component_sizing: None,
                        }),
                    }),
                }),
                cdci_descriptor: None,
                wave_pcm_descriptor: None,
                dc_timed_text_descriptor: None,
                iab_essence_descriptor: None,
                isxd_data_essence_descriptor: None,
            }],
        });
        let mut sl = empty_sequence_list();
        sl.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: uuid(4),
            resource_list: ResourceList {
                resources: vec![make_resource(Some(ed_id))],
            },
        });
        cpl.segment_list.segments[0].sequence_list = sl;

        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("6.5.2/CodingStyle")),
            "Should flag missing CodingStyleDefault: {:#?}",
            issues
        );
    }

    #[test]
    fn app2e_flags_j2k_missing_j2c_layout() {
        use crate::cpl::{
            J2KExtendedCapabilities, JPEG2000SubDescriptor, RGBADescriptor, VideoSubDescriptors,
        };

        let ed_id = uuid(10);
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: ed_id,
                rgba_descriptor: Some(RGBADescriptor {
                    instance_id: None,
                    display_width: Some(1920),
                    display_height: Some(1080),
                    stored_width: Some(1920),
                    stored_height: Some(1080),
                    sample_rate: Some(EditRate::new(24, 1)),
                    image_aspect_ratio: None,
                    color_primaries: Some(ColorPrimaries::P3D65),
                    transfer_characteristic: Some(TransferCharacteristic::PqSt2084),
                    coding_equations: None,
                    picture_compression: Some(VideoCodec::Jpeg2000Ht),
                    frame_layout: Some("FullFrame".to_string()),
                    display_f2_offset: None,
                    component_max_ref: Some(1023),
                    component_min_ref: Some(0),
                    scanning_direction: Some(
                        "ScanningDirection_LeftToRightTopToBottom".to_string(),
                    ),
                    stored_f2_offset: None,
                    sampled_width: None,
                    sampled_height: None,
                    sampled_x_offset: None,
                    sampled_y_offset: None,
                    alpha_transparency: None,
                    image_alignment_offset: None,
                    image_start_offset: None,
                    image_end_offset: None,
                    field_dominance: None,
                    alpha_max_ref: None,
                    alpha_min_ref: None,
                    palette: None,
                    palette_layout: None,
                    linked_track_id: None,
                    sub_descriptors: Some(VideoSubDescriptors {
                        phdr_metadata_track_sub_descriptor: None,
                        jpeg2000_sub_descriptor: Some(JPEG2000SubDescriptor {
                            instance_id: None,
                            rsiz: Some(16384),
                            xsiz: Some(1920),
                            ysiz: Some(1080),
                            xo_siz: None,
                            yo_siz: None,
                            xt_siz: None,
                            yt_siz: None,
                            xto_siz: None,
                            yto_siz: None,
                            csiz: None,
                            coding_style_default: Some("01020001".to_string()),
                            quantization_default: None,
                            j2c_layout: None, // Missing!
                            j2k_extended_capabilities: Some(J2KExtendedCapabilities {
                                pcap: Some(131072),
                            }),
                            picture_component_sizing: None,
                        }),
                    }),
                }),
                cdci_descriptor: None,
                wave_pcm_descriptor: None,
                dc_timed_text_descriptor: None,
                iab_essence_descriptor: None,
                isxd_data_essence_descriptor: None,
            }],
        });
        let mut sl = empty_sequence_list();
        sl.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: uuid(4),
            resource_list: ResourceList {
                resources: vec![make_resource(Some(ed_id))],
            },
        });
        cpl.segment_list.segments[0].sequence_list = sl;

        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("6.5.2/J2CLayout")),
            "Should flag missing J2CLayout: {:#?}",
            issues
        );
    }

    #[test]
    fn app2e_flags_j2k_missing_extended_capabilities_for_htj2k() {
        use crate::cpl::{
            J2CLayout, JPEG2000SubDescriptor, RGBADescriptor, RGBALayoutComponent,
            VideoSubDescriptors,
        };

        let ed_id = uuid(10);
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: ed_id,
                rgba_descriptor: Some(RGBADescriptor {
                    instance_id: None,
                    display_width: Some(1920),
                    display_height: Some(1080),
                    stored_width: Some(1920),
                    stored_height: Some(1080),
                    sample_rate: Some(EditRate::new(24, 1)),
                    image_aspect_ratio: None,
                    color_primaries: Some(ColorPrimaries::P3D65),
                    transfer_characteristic: Some(TransferCharacteristic::PqSt2084),
                    coding_equations: None,
                    picture_compression: Some(VideoCodec::Jpeg2000Ht),
                    frame_layout: Some("FullFrame".to_string()),
                    display_f2_offset: None,
                    component_max_ref: Some(1023),
                    component_min_ref: Some(0),
                    scanning_direction: Some(
                        "ScanningDirection_LeftToRightTopToBottom".to_string(),
                    ),
                    stored_f2_offset: None,
                    sampled_width: None,
                    sampled_height: None,
                    sampled_x_offset: None,
                    sampled_y_offset: None,
                    alpha_transparency: None,
                    image_alignment_offset: None,
                    image_start_offset: None,
                    image_end_offset: None,
                    field_dominance: None,
                    alpha_max_ref: None,
                    alpha_min_ref: None,
                    palette: None,
                    palette_layout: None,
                    linked_track_id: None,
                    sub_descriptors: Some(VideoSubDescriptors {
                        phdr_metadata_track_sub_descriptor: None,
                        jpeg2000_sub_descriptor: Some(JPEG2000SubDescriptor {
                            instance_id: None,
                            rsiz: Some(16384), // HTJ2K = bit 14 set
                            xsiz: Some(1920),
                            ysiz: Some(1080),
                            xo_siz: None,
                            yo_siz: None,
                            xt_siz: None,
                            yt_siz: None,
                            xto_siz: None,
                            yto_siz: None,
                            csiz: None,
                            coding_style_default: Some("01020001".to_string()),
                            quantization_default: None,
                            j2c_layout: Some(J2CLayout {
                                components: vec![RGBALayoutComponent {
                                    code: "CompRed".to_string(),
                                    component_size: 10,
                                }],
                            }),
                            j2k_extended_capabilities: None, // Missing for HTJ2K!
                            picture_component_sizing: None,
                        }),
                    }),
                }),
                cdci_descriptor: None,
                wave_pcm_descriptor: None,
                dc_timed_text_descriptor: None,
                iab_essence_descriptor: None,
                isxd_data_essence_descriptor: None,
            }],
        });
        let mut sl = empty_sequence_list();
        sl.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: uuid(4),
            resource_list: ResourceList {
                resources: vec![make_resource(Some(ed_id))],
            },
        });
        cpl.segment_list.segments[0].sequence_list = sl;

        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.5.2/J2KExtendedCapabilities")),
            "Should flag missing J2KExtendedCapabilities for HTJ2K: {:#?}",
            issues
        );
    }

    #[test]
    fn app2e_warns_j2k_sub_descriptor_missing() {
        let cpl = cpl_with_rgba_descriptor(ColorPrimaries::P3D65, TransferCharacteristic::PqSt2084);

        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);

        assert!(
            issues.iter().any(|i| {
                i.code.contains("6.5.2/JPEG2000SubDescriptor") && i.severity == Severity::Warning
            }),
            "Should warn when JPEG2000SubDescriptor is missing: {:#?}",
            issues
        );
    }

    #[test]
    fn app2e_sampled_x_offset_non_zero() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.sampled_x_offset = Some(1);
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.2.1/SampledXOffset")),
            "Should flag SampledXOffset != 0: {:#?}",
            issues
        );
    }

    // ── §7.4 Segment Duration ──────────────────────────────────────────────

    /// Helper: build a CPL with audio at the given sample rate and composition edit rate,
    /// with a single segment containing a main image resource of the given duration.
    fn cpl_with_audio_and_segment_duration(
        audio_sample_rate: EditRate,
        composition_edit_rate: EditRate,
        segment_durations: &[u64],
    ) -> CompositionPlaylist {
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(composition_edit_rate);

        // Audio essence descriptor with the given sample rate.
        let audio_ed_id = uuid(20);
        let video_ed_id = uuid(10);

        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![
                EssenceDescriptor {
                    id: video_ed_id,
                    rgba_descriptor: None,
                    cdci_descriptor: None,
                    wave_pcm_descriptor: None,
                    dc_timed_text_descriptor: None,
                    iab_essence_descriptor: None,
                    isxd_data_essence_descriptor: None,
                },
                EssenceDescriptor {
                    id: audio_ed_id,
                    rgba_descriptor: None,
                    cdci_descriptor: None,
                    wave_pcm_descriptor: Some(WAVEPCMDescriptor {
                        instance_id: None,
                        sample_rate: None,
                        audio_sample_rate: Some(audio_sample_rate),
                        channel_count: Some(2),
                        quantization_bits: Some(24),
                        linked_track_id: None,
                        sub_descriptors: None,
                    }),
                    dc_timed_text_descriptor: None,
                    iab_essence_descriptor: None,
                    isxd_data_essence_descriptor: None,
                },
            ],
        });

        let mut segments = Vec::new();
        for (i, &dur) in segment_durations.iter().enumerate() {
            let mut sl = empty_sequence_list();
            sl.main_image_sequences.push(MainImageSequence {
                id: uuid(30 + i as u8),
                track_id: uuid(40),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(50 + i as u8),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: dur,
                        entry_point: None,
                        source_duration: Some(dur),
                        source_encoding: Some(video_ed_id),
                        track_file_id: Some(uuid(60 + i as u8)),
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
            segments.push(Segment {
                id: uuid(70 + i as u8),
                sequence_list: sl,
            });
        }
        cpl.segment_list = SegmentList { segments };

        cpl
    }

    /// §7.4: 48kHz audio at 30000/1001 fps → 1601.6 samples/edit unit (non-integer).
    /// Segment duration of 7 edit units is not divisible by 5 → should flag.
    #[test]
    fn app2e_flags_segment_duration_not_multiple_of_5() {
        let cpl = cpl_with_audio_and_segment_duration(
            EditRate::new(48000, 1),    // 48kHz audio
            EditRate::new(30000, 1001), // 29.97 fps — 1601.6 samples/EU (non-integer)
            &[7],                       // 7 edit units, not divisible by 5
        );
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("7.4")),
            "Should flag segment duration not multiple of 5: {:#?}",
            issues
        );
    }

    /// §7.4: Same non-integer sample rate, but segment duration is 10 (multiple of 5) → pass.
    #[test]
    fn app2e_allows_segment_duration_multiple_of_5() {
        let cpl = cpl_with_audio_and_segment_duration(
            EditRate::new(48000, 1),    // 48kHz audio
            EditRate::new(30000, 1001), // 29.97 fps — non-integer samples/EU
            &[10],                      // 10 edit units, divisible by 5
        );
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("7.4")),
            "Should NOT flag segment duration that is a multiple of 5: {:#?}",
            issues
        );
    }

    /// §7.4: 48kHz audio at 24 fps → 2000 samples/edit unit (integer).
    /// Constraint does not apply, so any segment duration is fine.
    #[test]
    fn app2e_allows_any_duration_when_integer_samples() {
        let cpl = cpl_with_audio_and_segment_duration(
            EditRate::new(48000, 1), // 48kHz audio
            EditRate::new(24, 1),    // 24 fps — 2000 samples/EU (integer)
            &[7], // 7 edit units, not divisible by 5 — but constraint doesn't apply
        );
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("7.4")),
            "Should NOT flag when audio samples per EU is integer: {:#?}",
            issues
        );
    }

    /// §7.4: Multiple segments — one compliant, one not. Only the non-compliant one is flagged.
    #[test]
    fn app2e_flags_only_non_compliant_segments() {
        let cpl = cpl_with_audio_and_segment_duration(
            EditRate::new(48000, 1),    // 48kHz audio
            EditRate::new(30000, 1001), // 29.97 fps — non-integer
            &[10, 7, 15],               // 10=ok, 7=bad, 15=ok
        );
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        let seg_issues: Vec<_> = issues.iter().filter(|i| i.code.contains("7.4")).collect();
        assert_eq!(
            seg_issues.len(),
            1,
            "Should flag exactly 1 segment: {:#?}",
            seg_issues
        );
        assert!(
            seg_issues[0].message.contains("Segment 2"),
            "Should flag segment 2 (the one with duration 7): {}",
            seg_issues[0].message
        );
    }

    /// §7.4: 48kHz at 24000/1001 fps → 2002 samples/EU (integer). No constraint.
    #[test]
    fn app2e_allows_any_duration_at_23_976fps() {
        let cpl = cpl_with_audio_and_segment_duration(
            EditRate::new(48000, 1),    // 48kHz audio
            EditRate::new(24000, 1001), // 23.976 fps — 2002 samples/EU (integer)
            &[13],                      // Not divisible by 5, but constraint doesn't apply
        );
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("7.4")),
            "Should NOT flag at 23.976 fps (integer samples/EU): {:#?}",
            issues
        );
    }

    // ────────────────────────────────────────────────────────────────────────
    // Removed: fabricated "Table 3" image characteristics tests.
    // ST 2067-21:2023 has no combined resolution+colorimetry+frame-rate table.
    // Individual field constraints (Tables 8-14, §6.2.2-6.2.4) are tested above.
    // ────────────────────────────────────────────────────────────────────────

    // ════════════════════════════════════════════════════════════════════════
    // Core constraints tests — resource & timeline validation (ST 2067-2)
    // ════════════════════════════════════════════════════════════════════════

    #[test]
    fn core_rejects_duplicate_segment_ids() {
        use crate::cpl::Segment;

        let mut cpl = minimal_cpl();
        // Add a second segment with the same ID as the first
        cpl.segment_list.segments.push(Segment {
            id: cpl.segment_list.segments[0].id,
            sequence_list: empty_sequence_list(),
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("UniqueSegmentId")),
            "Duplicate segment IDs should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_rejects_zero_intrinsic_duration() {
        let mut cpl = minimal_cpl();
        let ed_id = uuid(10);
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(3),
                track_id: uuid(4),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(99),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 0, // Invalid!
                        entry_point: None,
                        source_duration: None,
                        source_encoding: Some(ed_id),
                        track_file_id: Some(uuid(50)),
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("IntrinsicDuration")),
            "Zero IntrinsicDuration should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_rejects_entry_plus_duration_exceeds_intrinsic() {
        let mut cpl = minimal_cpl();
        let ed_id = uuid(10);
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(3),
                track_id: uuid(4),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(99),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: Some(50),
                        source_duration: Some(60), // 50 + 60 = 110 > 100
                        source_encoding: Some(ed_id),
                        track_file_id: Some(uuid(50)),
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("ResourceDuration")),
            "EntryPoint + SourceDuration > IntrinsicDuration should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accepts_valid_entry_plus_duration() {
        let mut cpl = minimal_cpl();
        let ed_id = uuid(10);
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(3),
                track_id: uuid(4),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(99),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: Some(50),
                        source_duration: Some(50), // 50 + 50 = 100 == IntrinsicDuration
                        source_encoding: Some(ed_id),
                        track_file_id: Some(uuid(50)),
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("ResourceDuration")),
            "Valid EntryPoint + SourceDuration should not be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_rejects_zero_repeat_count() {
        let mut cpl = minimal_cpl();
        let ed_id = uuid(10);
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(3),
                track_id: uuid(4),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(99),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: None,
                        source_duration: None,
                        source_encoding: Some(ed_id),
                        track_file_id: Some(uuid(50)),
                        repeat_count: Some(0), // Invalid!
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("RepeatCount")),
            "Zero RepeatCount should be flagged: {:#?}",
            issues,
        );
    }

    // ── A: SourceDuration > 0 ────────────────────────────────────────────────

    #[test]
    fn core_rejects_zero_source_duration() {
        let mut cpl = minimal_cpl();
        let ed_id = uuid(10);
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(3),
                track_id: uuid(4),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(99),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: None,
                        source_duration: Some(0), // Invalid
                        source_encoding: Some(ed_id),
                        track_file_id: Some(uuid(50)),
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("SourceDuration")),
            "Zero SourceDuration should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accepts_nonzero_source_duration() {
        let mut cpl = minimal_cpl();
        let ed_id = uuid(10);
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(3),
                track_id: uuid(4),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(99),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: None,
                        source_duration: Some(50),
                        source_encoding: Some(ed_id),
                        track_file_id: Some(uuid(50)),
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("SourceDuration")),
            "Valid SourceDuration should not be flagged: {:#?}",
            issues,
        );
    }

    // ── B: EntryPoint < IntrinsicDuration ───────────────────────────────────

    #[test]
    fn core_rejects_entry_point_gte_intrinsic_duration() {
        let mut cpl = minimal_cpl();
        let ed_id = uuid(10);
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(3),
                track_id: uuid(4),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(99),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: Some(100), // Equal — invalid, must be < 100
                        source_duration: None,
                        source_encoding: Some(ed_id),
                        track_file_id: Some(uuid(50)),
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("EntryPoint")),
            "EntryPoint >= IntrinsicDuration should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accepts_entry_point_less_than_intrinsic() {
        let mut cpl = minimal_cpl();
        let ed_id = uuid(10);
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(3),
                track_id: uuid(4),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(99),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: Some(99), // Valid: 99 < 100
                        source_duration: Some(1),
                        source_encoding: Some(ed_id),
                        track_file_id: Some(uuid(50)),
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("EntryPoint")),
            "Valid EntryPoint should not be flagged: {:#?}",
            issues,
        );
    }

    // ── C: Duplicate TrackId within a segment ───────────────────────────────

    #[test]
    fn core_rejects_duplicate_track_id_within_segment() {
        let mut cpl = minimal_cpl();
        let ed_id = uuid(10);
        let shared_track_id = uuid(4);
        let make_seq = |seq_id: u8, res_id: u8| MainImageSequence {
            id: uuid(seq_id),
            track_id: shared_track_id,
            resource_list: ResourceList {
                resources: vec![Resource {
                    id: uuid(res_id),
                    annotation: None,
                    edit_rate: None,
                    intrinsic_duration: 48,
                    entry_point: None,
                    source_duration: None,
                    source_encoding: Some(ed_id),
                    track_file_id: Some(uuid(50)),
                    repeat_count: None,
                    key_id: None,
                    hash: None,
                    markers: vec![],
                }],
            },
        };
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(make_seq(3, 99));
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(make_seq(5, 98));
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("TrackIdNotUnique")),
            "Duplicate TrackId in same segment should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accepts_same_track_id_in_different_segments() {
        let mut cpl = minimal_cpl();
        let ed_id = uuid(10);
        let shared_track_id = uuid(4);
        let make_res = |res_id: u8| Resource {
            id: uuid(res_id),
            annotation: None,
            edit_rate: None,
            intrinsic_duration: 48,
            entry_point: None,
            source_duration: None,
            source_encoding: Some(ed_id),
            track_file_id: Some(uuid(50)),
            repeat_count: None,
            key_id: None,
            hash: None,
            markers: vec![],
        };
        let mut sl1 = empty_sequence_list();
        sl1.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: shared_track_id,
            resource_list: ResourceList {
                resources: vec![make_res(99)],
            },
        });
        let mut sl2 = empty_sequence_list();
        sl2.main_image_sequences.push(MainImageSequence {
            id: uuid(7),
            track_id: shared_track_id,
            resource_list: ResourceList {
                resources: vec![make_res(98)],
            },
        });
        cpl.segment_list.segments = vec![
            Segment {
                id: uuid(2),
                sequence_list: sl1,
            },
            Segment {
                id: uuid(6),
                sequence_list: sl2,
            },
        ];
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("TrackIdNotUnique")),
            "Same TrackId in different segments (virtual track) should not be flagged: {:#?}",
            issues,
        );
    }

    // ── D: Locale tag validation at core level ───────────────────────────────

    #[test]
    fn core_warns_malformed_locale_language_tag() {
        let mut cpl = minimal_cpl();
        cpl.locale_list = Some(crate::cpl::LocaleList {
            locales: vec![crate::cpl::Locale {
                language_list: Some(crate::cpl::LanguageList {
                    languages: vec![crate::cpl::LanguageTag("123invalid".to_string())],
                }),
                region_list: None,
                content_maturity_rating_list: None,
            }],
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("LocaleLanguageTagInvalid")),
            "Malformed language tag should be flagged at core level: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accepts_valid_locale() {
        let mut cpl = minimal_cpl();
        cpl.locale_list = Some(crate::cpl::LocaleList {
            locales: vec![crate::cpl::Locale {
                language_list: Some(crate::cpl::LanguageList {
                    languages: vec![
                        crate::cpl::LanguageTag("nl".to_string()),
                        crate::cpl::LanguageTag("en".to_string()),
                    ],
                }),
                region_list: Some(crate::cpl::RegionList {
                    regions: vec!["NL".to_string(), "US".to_string()],
                }),
                content_maturity_rating_list: None,
            }],
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("Locale")),
            "Valid locale should not be flagged: {:#?}",
            issues,
        );
    }

    // ── F: core_2016 requires EssenceDescriptorList ─────────────────────────

    #[test]
    fn core_2016_requires_essence_descriptor_list() {
        let mut cpl = minimal_cpl();
        cpl.namespace = CplNamespace::Smpte2067_3_2016;
        cpl.essence_descriptor_list = None; // Missing — 2016 requires it
        let issues = CoreConstraints2016.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("EssenceDescriptorList")),
            "ST 2067-2:2016 should require EssenceDescriptorList: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_2016_accepts_present_essence_descriptor_list() {
        let mut cpl = minimal_cpl();
        cpl.namespace = CplNamespace::Smpte2067_3_2016;
        // EDL must be present AND non-empty per XSD (minOccurs=1 on EssenceDescriptor)
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: uuid(99),
                rgba_descriptor: None,
                cdci_descriptor: None,
                wave_pcm_descriptor: None,
                dc_timed_text_descriptor: None,
                iab_essence_descriptor: None,
                isxd_data_essence_descriptor: None,
            }],
        });
        let issues = CoreConstraints2016.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("EssenceDescriptorList")),
            "Present non-empty EDL should not be flagged: {:#?}",
            issues,
        );
    }

    // ── H: ContentVersion.Id non-empty ──────────────────────────────────────

    #[test]
    fn core_flags_empty_content_version_id() {
        let mut cpl = minimal_cpl();
        cpl.content_version_list = Some(crate::cpl::ContentVersionList {
            content_versions: vec![crate::cpl::ContentVersion {
                id: "".to_string(),
                label_text: Some(crate::cpl::LanguageString {
                    text: "Version 1".to_string(),
                    language: None,
                }),
            }],
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("ContentVersionIdInvalid")),
            "Empty ContentVersion Id should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accepts_nonempty_content_version_id() {
        let mut cpl = minimal_cpl();
        cpl.content_version_list = Some(crate::cpl::ContentVersionList {
            content_versions: vec![crate::cpl::ContentVersion {
                id: "urn:uuid:00000000-0000-0000-0000-000000000001".to_string(),
                label_text: Some(crate::cpl::LanguageString {
                    text: "Version 1".to_string(),
                    language: None,
                }),
            }],
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("ContentVersionIdInvalid")),
            "Non-empty ContentVersion Id should not be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_rejects_missing_track_file_id() {
        let mut cpl = minimal_cpl();
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(3),
                track_id: uuid(4),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(99),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: None,
                        source_duration: None,
                        source_encoding: Some(uuid(10)),
                        track_file_id: None, // Missing!
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("TrackFileId")),
            "Missing TrackFileId should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_detects_virtual_track_discontinuity() {
        use crate::cpl::Segment;

        let mut cpl = minimal_cpl();
        let track_a = uuid(4);
        let track_b = uuid(5);

        // Segment 1: has track_a and track_b
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(10),
                track_id: track_a,
                resource_list: ResourceList {
                    resources: vec![make_resource(Some(uuid(20)))],
                },
            });
        cpl.segment_list.segments[0]
            .sequence_list
            .main_audio_sequences
            .push(MainAudioSequence {
                id: uuid(11),
                track_id: track_b,
                resource_list: ResourceList {
                    resources: vec![make_resource(Some(uuid(21)))],
                },
            });

        // Segment 2: only has track_a (track_b missing = discontinuity)
        cpl.segment_list.segments.push(Segment {
            id: uuid(30),
            sequence_list: SequenceList {
                main_image_sequences: vec![MainImageSequence {
                    id: uuid(12),
                    track_id: track_a,
                    resource_list: ResourceList {
                        resources: vec![make_resource(Some(uuid(22)))],
                    },
                }],
                main_audio_sequences: vec![],
                subtitles_sequences: vec![],
                hearing_impaired_captions_sequences: vec![],
                forced_narrative_sequences: vec![],
                marker_sequences: vec![],
                iab_sequences: vec![],
                isxd_sequences: vec![],
            },
        });

        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("VirtualTrackContinuity")),
            "Missing virtual track in segment 2 should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accepts_continuous_virtual_tracks() {
        use crate::cpl::Segment;

        let mut cpl = minimal_cpl();
        let track_a = uuid(4);

        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(10),
                track_id: track_a,
                resource_list: ResourceList {
                    resources: vec![make_resource(Some(uuid(20)))],
                },
            });

        // Segment 2: same track_a present
        cpl.segment_list.segments.push(Segment {
            id: uuid(30),
            sequence_list: SequenceList {
                main_image_sequences: vec![MainImageSequence {
                    id: uuid(12),
                    track_id: track_a,
                    resource_list: ResourceList {
                        resources: vec![make_resource(Some(uuid(22)))],
                    },
                }],
                main_audio_sequences: vec![],
                subtitles_sequences: vec![],
                hearing_impaired_captions_sequences: vec![],
                forced_narrative_sequences: vec![],
                marker_sequences: vec![],
                iab_sequences: vec![],
                isxd_sequences: vec![],
            },
        });

        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("VirtualTrackContinuity")),
            "Continuous virtual tracks should not be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_detects_edit_rate_mismatch_in_virtual_track() {
        use crate::cpl::Segment;

        let mut cpl = minimal_cpl();
        let track_a = uuid(4);

        // Segment 1: resource at 24fps
        let mut res1 = make_resource(Some(uuid(20)));
        res1.id = uuid(91);
        res1.edit_rate = Some(EditRate::new(24, 1));

        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(10),
                track_id: track_a,
                resource_list: ResourceList {
                    resources: vec![res1],
                },
            });

        // Segment 2: same track, resource at 25fps (mismatch!)
        let mut res2 = make_resource(Some(uuid(22)));
        res2.id = uuid(92);
        res2.edit_rate = Some(EditRate::new(25, 1));

        cpl.segment_list.segments.push(Segment {
            id: uuid(30),
            sequence_list: SequenceList {
                main_image_sequences: vec![MainImageSequence {
                    id: uuid(12),
                    track_id: track_a,
                    resource_list: ResourceList {
                        resources: vec![res2],
                    },
                }],
                main_audio_sequences: vec![],
                subtitles_sequences: vec![],
                hearing_impaired_captions_sequences: vec![],
                forced_narrative_sequences: vec![],
                marker_sequences: vec![],
                iab_sequences: vec![],
                isxd_sequences: vec![],
            },
        });

        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("VirtualTrackEditRate")),
            "Edit rate mismatch in virtual track should be flagged: {:#?}",
            issues,
        );
    }

    /// ST 2067-3 §6.9.3 / XSD: Resource EditRate is optional (minOccurs="0").
    /// Absent EditRate means "inherit the CPL's EditRate". A resource with no
    /// explicit EditRate in the same virtual track as resources with an explicit
    /// EditRate equal to the CPL's EditRate must NOT be flagged.
    #[test]
    fn core_accepts_absent_edit_rate_when_matches_cpl_rate() {
        use crate::cpl::Segment;

        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        let track_a = uuid(4);

        // Segment 1: resource with explicit 24fps (matches CPL)
        let mut res1 = make_resource(Some(uuid(20)));
        res1.id = uuid(91);
        res1.edit_rate = Some(EditRate::new(24, 1));

        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(10),
                track_id: track_a,
                resource_list: ResourceList {
                    resources: vec![res1],
                },
            });

        // Segment 2: same track, resource with NO explicit EditRate (inherits CPL 24fps)
        let mut res2 = make_resource(Some(uuid(22)));
        res2.id = uuid(92);
        res2.edit_rate = None;

        cpl.segment_list.segments.push(Segment {
            id: uuid(30),
            sequence_list: SequenceList {
                main_image_sequences: vec![MainImageSequence {
                    id: uuid(12),
                    track_id: track_a,
                    resource_list: ResourceList {
                        resources: vec![res2],
                    },
                }],
                main_audio_sequences: vec![],
                subtitles_sequences: vec![],
                hearing_impaired_captions_sequences: vec![],
                forced_narrative_sequences: vec![],
                marker_sequences: vec![],
                iab_sequences: vec![],
                isxd_sequences: vec![],
            },
        });

        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("VirtualTrackEditRate")),
            "Absent EditRate matching CPL rate must not be flagged: {:#?}",
            issues,
        );
    }

    /// SegmentDuration normalization: audio at 48000/1 and video at 24/1 with
    /// the same real-time duration must NOT be flagged.
    ///
    /// ST 2067-2 §7.2.2 requires equal real-time duration across all virtual
    /// tracks in a segment, not equal raw frame counts.
    #[test]
    fn core_accepts_equal_real_time_duration_across_edit_rates() {
        let mut cpl = minimal_cpl();
        let mut sl = empty_sequence_list();
        let er_video = EditRate {
            numerator: 24,
            denominator: 1,
        };
        let er_audio = EditRate {
            numerator: 48_000,
            denominator: 1,
        };
        // Video: 240 frames at 24/1 = 10.0 s
        sl.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: uuid(4),
            resource_list: ResourceList {
                resources: vec![Resource {
                    id: uuid(20),
                    annotation: None,
                    edit_rate: Some(er_video),
                    intrinsic_duration: 240,
                    entry_point: None,
                    source_duration: None,
                    source_encoding: None,
                    track_file_id: Some(uuid(50)),
                    repeat_count: None,
                    key_id: None,
                    hash: None,
                    markers: vec![],
                }],
            },
        });
        // Audio: 480000 frames at 48000/1 = 10.0 s (same real time)
        sl.main_audio_sequences.push(MainAudioSequence {
            id: uuid(5),
            track_id: uuid(6),
            resource_list: ResourceList {
                resources: vec![Resource {
                    id: uuid(21),
                    annotation: None,
                    edit_rate: Some(er_audio),
                    intrinsic_duration: 480_000,
                    entry_point: None,
                    source_duration: None,
                    source_encoding: None,
                    track_file_id: Some(uuid(51)),
                    repeat_count: None,
                    key_id: None,
                    hash: None,
                    markers: vec![],
                }],
            },
        });
        cpl.segment_list.segments[0].sequence_list = sl;

        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("SegmentDuration")),
            "Equal real-time duration across different edit rates must not be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_rejects_marker_offset_beyond_duration() {
        use crate::cpl::MarkerLabel;
        use crate::cpl::{MarkerInfo, MarkerLabelElement, MarkerSequence};

        let mut cpl = minimal_cpl();
        cpl.segment_list.segments[0]
            .sequence_list
            .marker_sequences
            .push(MarkerSequence {
                id: uuid(60),
                track_id: uuid(61),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(62),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: Some(10),
                        source_duration: Some(50), // effective duration = 50
                        source_encoding: None,
                        track_file_id: None,
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![MarkerInfo {
                            annotation: None,
                            label: MarkerLabelElement::from(MarkerLabel::Ffoc),
                            offset: 60, // 60 >= 50 → out of bounds
                        }],
                    }],
                },
            });

        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("MarkerOffset")),
            "Marker offset beyond resource duration should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accepts_marker_at_valid_offset() {
        use crate::cpl::MarkerLabel;
        use crate::cpl::{MarkerInfo, MarkerLabelElement, MarkerSequence};

        let mut cpl = minimal_cpl();
        cpl.segment_list.segments[0]
            .sequence_list
            .marker_sequences
            .push(MarkerSequence {
                id: uuid(60),
                track_id: uuid(61),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(62),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: None,
                        source_duration: Some(100),
                        source_encoding: None,
                        track_file_id: None,
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![MarkerInfo {
                            annotation: None,
                            label: MarkerLabelElement::from(MarkerLabel::Ffoc),
                            offset: 0, // First frame — valid
                        }],
                    }],
                },
            });

        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("MarkerOffset")),
            "Marker at offset 0 should not be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_rejects_duplicate_essence_descriptor_ids() {
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![
                EssenceDescriptor {
                    id: uuid(10),
                    rgba_descriptor: None,
                    cdci_descriptor: None,
                    wave_pcm_descriptor: None,
                    dc_timed_text_descriptor: None,
                    iab_essence_descriptor: None,
                    isxd_data_essence_descriptor: None,
                },
                EssenceDescriptor {
                    id: uuid(10), // Duplicate!
                    rgba_descriptor: None,
                    cdci_descriptor: None,
                    wave_pcm_descriptor: None,
                    dc_timed_text_descriptor: None,
                    iab_essence_descriptor: None,
                    isxd_data_essence_descriptor: None,
                },
            ],
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("UniqueEssenceDescriptorId")),
            "Duplicate EssenceDescriptor IDs should be flagged: {:#?}",
            issues,
        );
    }

    // ════════════════════════════════════════════════════════════════════════
    // Segment track duration consistency tests
    // ════════════════════════════════════════════════════════════════════════

    #[test]
    fn core_accepts_equal_track_durations_in_segment() {
        let mut cpl = minimal_cpl();
        let mut sl = empty_sequence_list();
        // Video: 100 frames
        sl.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: uuid(4),
            resource_list: ResourceList {
                resources: vec![Resource {
                    id: uuid(20),
                    annotation: None,
                    edit_rate: None,
                    intrinsic_duration: 100,
                    entry_point: None,
                    source_duration: None,
                    source_encoding: None,
                    track_file_id: Some(uuid(50)),
                    repeat_count: None,
                    key_id: None,
                    hash: None,
                    markers: vec![],
                }],
            },
        });
        // Audio: also 100 frames
        sl.main_audio_sequences.push(MainAudioSequence {
            id: uuid(5),
            track_id: uuid(6),
            resource_list: ResourceList {
                resources: vec![Resource {
                    id: uuid(21),
                    annotation: None,
                    edit_rate: None,
                    intrinsic_duration: 100,
                    entry_point: None,
                    source_duration: None,
                    source_encoding: None,
                    track_file_id: Some(uuid(51)),
                    repeat_count: None,
                    key_id: None,
                    hash: None,
                    markers: vec![],
                }],
            },
        });
        cpl.segment_list.segments[0].sequence_list = sl;

        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("SegmentDuration")),
            "Equal durations should not produce duration mismatch: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_detects_mismatched_track_durations_in_segment() {
        let mut cpl = minimal_cpl();
        let mut sl = empty_sequence_list();
        let er_video = EditRate {
            numerator: 24,
            denominator: 1,
        };
        let er_audio = EditRate {
            numerator: 48000,
            denominator: 1,
        };
        // Video: 100 frames at 24/1 = ~4.167 s
        sl.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: uuid(4),
            resource_list: ResourceList {
                resources: vec![Resource {
                    id: uuid(20),
                    annotation: None,
                    edit_rate: Some(er_video),
                    intrinsic_duration: 100,
                    entry_point: None,
                    source_duration: None,
                    source_encoding: None,
                    track_file_id: Some(uuid(50)),
                    repeat_count: None,
                    key_id: None,
                    hash: None,
                    markers: vec![],
                }],
            },
        });
        // Audio: 96000 frames at 48000/1 = 2.0 s — genuinely different real-time duration
        sl.main_audio_sequences.push(MainAudioSequence {
            id: uuid(5),
            track_id: uuid(6),
            resource_list: ResourceList {
                resources: vec![Resource {
                    id: uuid(21),
                    annotation: None,
                    edit_rate: Some(er_audio),
                    intrinsic_duration: 96_000,
                    entry_point: None,
                    source_duration: None,
                    source_encoding: None,
                    track_file_id: Some(uuid(51)),
                    repeat_count: None,
                    key_id: None,
                    hash: None,
                    markers: vec![],
                }],
            },
        });
        cpl.segment_list.segments[0].sequence_list = sl;

        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("SegmentDuration")),
            "Mismatched durations should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accounts_for_entry_point_and_source_duration() {
        let mut cpl = minimal_cpl();
        let mut sl = empty_sequence_list();
        // Video: intrinsic=200, entry=50, source=100 → effective = 100
        sl.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: uuid(4),
            resource_list: ResourceList {
                resources: vec![Resource {
                    id: uuid(20),
                    annotation: None,
                    edit_rate: None,
                    intrinsic_duration: 200,
                    entry_point: Some(50),
                    source_duration: Some(100),
                    source_encoding: None,
                    track_file_id: Some(uuid(50)),
                    repeat_count: None,
                    key_id: None,
                    hash: None,
                    markers: vec![],
                }],
            },
        });
        // Audio: intrinsic=100 → effective = 100 (matches)
        sl.main_audio_sequences.push(MainAudioSequence {
            id: uuid(5),
            track_id: uuid(6),
            resource_list: ResourceList {
                resources: vec![Resource {
                    id: uuid(21),
                    annotation: None,
                    edit_rate: None,
                    intrinsic_duration: 100,
                    entry_point: None,
                    source_duration: None,
                    source_encoding: None,
                    track_file_id: Some(uuid(51)),
                    repeat_count: None,
                    key_id: None,
                    hash: None,
                    markers: vec![],
                }],
            },
        });
        cpl.segment_list.segments[0].sequence_list = sl;

        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("SegmentDuration")),
            "Entry point + source duration should compute correct effective duration: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accounts_for_repeat_count_in_duration() {
        let mut cpl = minimal_cpl();
        let mut sl = empty_sequence_list();
        // Video: 50 × repeat 2 = 100
        sl.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: uuid(4),
            resource_list: ResourceList {
                resources: vec![Resource {
                    id: uuid(20),
                    annotation: None,
                    edit_rate: None,
                    intrinsic_duration: 50,
                    entry_point: None,
                    source_duration: None,
                    source_encoding: None,
                    track_file_id: Some(uuid(50)),
                    repeat_count: Some(2),
                    key_id: None,
                    hash: None,
                    markers: vec![],
                }],
            },
        });
        // Audio: 100 (matches 50 × 2)
        sl.main_audio_sequences.push(MainAudioSequence {
            id: uuid(5),
            track_id: uuid(6),
            resource_list: ResourceList {
                resources: vec![Resource {
                    id: uuid(21),
                    annotation: None,
                    edit_rate: None,
                    intrinsic_duration: 100,
                    entry_point: None,
                    source_duration: None,
                    source_encoding: None,
                    track_file_id: Some(uuid(51)),
                    repeat_count: None,
                    key_id: None,
                    hash: None,
                    markers: vec![],
                }],
            },
        });
        cpl.segment_list.segments[0].sequence_list = sl;

        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("SegmentDuration")),
            "RepeatCount should be factored into duration: {:#?}",
            issues,
        );
    }

    // ════════════════════════════════════════════════════════════════════════
    // Audio MCA validation tests
    // ════════════════════════════════════════════════════════════════════════

    fn cpl_with_audio(wave: WAVEPCMDescriptor) -> CompositionPlaylist {
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: uuid(10),
                rgba_descriptor: None,
                cdci_descriptor: None,
                wave_pcm_descriptor: Some(wave),
                dc_timed_text_descriptor: None,
                iab_essence_descriptor: None,
                isxd_data_essence_descriptor: None,
            }],
        });
        cpl
    }

    #[test]
    fn audio_warns_missing_mca_sub_descriptors() {
        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: Some(6),
            quantization_bits: Some(24),
            linked_track_id: None,
            sub_descriptors: None, // Missing!
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("MCASubDescriptors")),
            "Missing MCA sub-descriptors should produce warning: {:#?}",
            issues,
        );
    }

    #[test]
    fn audio_warns_missing_soundfield_group() {
        use crate::cpl::AudioSubDescriptors;

        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: Some(6),
            quantization_bits: Some(24),
            linked_track_id: None,
            sub_descriptors: Some(AudioSubDescriptors {
                soundfield_group_label_sub_descriptor: None, // Missing!
            }),
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("SoundfieldGroup")),
            "Missing soundfield group should produce warning: {:#?}",
            issues,
        );
    }

    #[test]
    fn audio_flags_channel_count_mismatch() {
        use crate::cpl::McaTagSymbol;
        use crate::cpl::{AudioSubDescriptors, SoundfieldGroupLabelSubDescriptor};

        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: Some(2), // 2 channels but 5.1 soundfield!
            quantization_bits: Some(24),
            linked_track_id: None,
            sub_descriptors: Some(AudioSubDescriptors {
                soundfield_group_label_sub_descriptor: Some(SoundfieldGroupLabelSubDescriptor {
                    mca_tag_symbol: Some(McaTagSymbol::Sg51), // 5.1 expects 6 channels
                    mca_tag_name: Some("5.1".to_string()),
                    mca_audio_content_kind: None,
                    rfc5646_spoken_language: None,
                }),
            }),
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("SoundfieldChannelCount")),
            "Channel count mismatch with soundfield should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn audio_accepts_correct_51_channel_count() {
        use crate::cpl::McaTagSymbol;
        use crate::cpl::{AudioSubDescriptors, SoundfieldGroupLabelSubDescriptor};

        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: Some(6), // Correct for 5.1
            quantization_bits: Some(24),
            linked_track_id: None,
            sub_descriptors: Some(AudioSubDescriptors {
                soundfield_group_label_sub_descriptor: Some(SoundfieldGroupLabelSubDescriptor {
                    mca_tag_symbol: Some(McaTagSymbol::Sg51),
                    mca_tag_name: Some("5.1".to_string()),
                    mca_audio_content_kind: None,
                    rfc5646_spoken_language: None,
                }),
            }),
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("SoundfieldChannelCount")),
            "Correct 5.1 channel count should not be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn audio_flags_zero_channel_count() {
        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: Some(0), // Invalid
            quantization_bits: Some(24),
            linked_track_id: None,
            sub_descriptors: None,
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("ChannelCount") && i.severity == Severity::Error),
            "Zero ChannelCount should be an error: {:#?}",
            issues,
        );
    }

    #[test]
    fn audio_accepts_stereo_with_correct_channel_count() {
        use crate::cpl::McaTagSymbol;
        use crate::cpl::{AudioSubDescriptors, SoundfieldGroupLabelSubDescriptor};

        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: Some(2),
            quantization_bits: Some(24),
            linked_track_id: None,
            sub_descriptors: Some(AudioSubDescriptors {
                soundfield_group_label_sub_descriptor: Some(SoundfieldGroupLabelSubDescriptor {
                    mca_tag_symbol: Some(McaTagSymbol::SgSt),
                    mca_tag_name: Some("Stereo".to_string()),
                    mca_audio_content_kind: None,
                    rfc5646_spoken_language: None,
                }),
            }),
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("SoundfieldChannelCount")),
            "Correct Stereo channel count should not be flagged: {:#?}",
            issues,
        );
    }

    // ════════════════════════════════════════════════════════════════════════
    // App2E QuantizationBits tests
    // ════════════════════════════════════════════════════════════════════════

    #[test]
    fn app2e_accepts_24bit_audio() {
        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: Some(2),
            quantization_bits: Some(24),
            linked_track_id: None,
            sub_descriptors: None,
        });
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("QuantizationBits")),
            "24-bit audio should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_16bit_audio() {
        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: Some(2),
            quantization_bits: Some(16),
            linked_track_id: None,
            sub_descriptors: None,
        });
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("QuantizationBits")),
            "16-bit audio should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_rejects_8bit_audio() {
        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: Some(2),
            quantization_bits: Some(8), // Not allowed by ST 2067-21
            linked_track_id: None,
            sub_descriptors: None,
        });
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("QuantizationBits")),
            "8-bit audio should be rejected: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_rejects_32bit_audio() {
        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: Some(2),
            quantization_bits: Some(32), // Not allowed by ST 2067-21
            linked_track_id: None,
            sub_descriptors: None,
        });
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("QuantizationBits")),
            "32-bit audio should be rejected: {:#?}",
            issues,
        );
    }

    // ── Image resolution validation (Tables 4-6) ────────────────────────────

    fn cpl_with_image_resolution(
        width: u32,
        height: u32,
        rate: Option<EditRate>,
    ) -> CompositionPlaylist {
        let ed_id = uuid(10);
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: ed_id,
                rgba_descriptor: None,
                cdci_descriptor: Some(CDCIDescriptor {
                    instance_id: None,
                    stored_width: Some(width),
                    stored_height: Some(height),
                    display_width: Some(width),
                    display_height: Some(height),
                    sample_rate: rate,
                    image_aspect_ratio: None,
                    color_primaries: Some(ColorPrimaries::Bt709),
                    transfer_characteristic: Some(TransferCharacteristic::Bt709),
                    coding_equations: Some(CodingEquations::Bt709),
                    picture_compression: Some(VideoCodec::Jpeg2000),
                    component_depth: Some(10),
                    frame_layout: Some("FullFrame".to_string()),
                    display_f2_offset: None,
                    horizontal_subsampling: Some(2),
                    vertical_subsampling: Some(1),
                    color_siting: Some(0),
                    black_ref_level: Some(64),
                    white_ref_level: Some(940),
                    color_range: Some(897),
                    stored_f2_offset: None,
                    sampled_width: None,
                    sampled_height: None,
                    sampled_x_offset: None,
                    sampled_y_offset: None,
                    alpha_transparency: None,
                    image_alignment_offset: None,
                    image_start_offset: None,
                    image_end_offset: None,
                    field_dominance: None,
                    reversed_byte_order: None,
                    padding_bits: None,
                    alpha_sample_depth: None,
                    linked_track_id: None,
                    active_width: None,
                    active_height: None,
                    sub_descriptors: None,
                }),
                wave_pcm_descriptor: None,
                dc_timed_text_descriptor: None,
                iab_essence_descriptor: None,
                isxd_data_essence_descriptor: None,
            }],
        });
        let mut sl = empty_sequence_list();
        sl.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: uuid(4),
            resource_list: ResourceList {
                resources: vec![make_resource(Some(ed_id))],
            },
        });
        cpl.segment_list.segments[0].sequence_list = sl;
        cpl
    }

    #[test]
    fn app2e_accepts_1920x1080() {
        let cpl = cpl_with_image_resolution(1920, 1080, Some(EditRate::new(24, 1)));
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("Resolution")),
            "1920x1080 should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_2048x1080() {
        let cpl = cpl_with_image_resolution(2048, 1080, Some(EditRate::new(25, 1)));
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("Resolution")),
            "2048x1080 should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_3840x2160() {
        let cpl = cpl_with_image_resolution(3840, 2160, Some(EditRate::new(24, 1)));
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("Resolution")),
            "3840x2160 should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_4096x2160() {
        let cpl = cpl_with_image_resolution(4096, 2160, Some(EditRate::new(24000, 1001)));
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("Resolution")),
            "4096x2160 should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_7680x4320() {
        let cpl = cpl_with_image_resolution(7680, 4320, Some(EditRate::new(24, 1)));
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("Resolution")),
            "7680x4320 should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_rejects_1280x720() {
        let cpl = cpl_with_image_resolution(1280, 720, Some(EditRate::new(24, 1)));
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("Resolution")),
            "1280x720 should be rejected: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_rejects_1920x800() {
        let cpl = cpl_with_image_resolution(1920, 800, Some(EditRate::new(24, 1)));
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("Resolution")),
            "1920x800 should be rejected: {:#?}",
            issues,
        );
    }

    // ── Image frame rate validation (Tables 4-6) ────────────────────────────

    #[test]
    fn app2e_accepts_24fps() {
        let cpl = cpl_with_image_resolution(1920, 1080, Some(EditRate::new(24, 1)));
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("FrameRate")),
            "24 fps should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_23976fps() {
        let cpl = cpl_with_image_resolution(1920, 1080, Some(EditRate::new(24000, 1001)));
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("FrameRate")),
            "23.976 fps should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_25fps() {
        let cpl = cpl_with_image_resolution(1920, 1080, Some(EditRate::new(25, 1)));
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("FrameRate")),
            "25 fps should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_60fps() {
        let cpl = cpl_with_image_resolution(3840, 2160, Some(EditRate::new(60, 1)));
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("FrameRate")),
            "60 fps should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_5994fps() {
        let cpl = cpl_with_image_resolution(3840, 2160, Some(EditRate::new(60000, 1001)));
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("FrameRate")),
            "59.94 fps should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_rejects_120fps() {
        let cpl = cpl_with_image_resolution(1920, 1080, Some(EditRate::new(120, 1)));
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("FrameRate")),
            "120 fps should be rejected: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_rejects_15fps() {
        let cpl = cpl_with_image_resolution(1920, 1080, Some(EditRate::new(15, 1)));
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("FrameRate")),
            "15 fps should be rejected: {:#?}",
            issues,
        );
    }

    // ── Audio sample rate validation (§6.5) ────────────────────────────────

    #[test]
    fn app2e_accepts_48000hz_audio() {
        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: Some(2),
            quantization_bits: Some(24),
            linked_track_id: None,
            sub_descriptors: None,
        });
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("AudioSampleRate")),
            "48000 Hz should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_rejects_44100hz_audio() {
        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(44100, 1)),
            channel_count: Some(2),
            quantization_bits: Some(24),
            linked_track_id: None,
            sub_descriptors: None,
        });
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("AudioSampleRate")),
            "44100 Hz should be rejected: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_rejects_96000hz_audio() {
        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(96000, 1)),
            channel_count: Some(2),
            quantization_bits: Some(24),
            linked_track_id: None,
            sub_descriptors: None,
        });
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("AudioSampleRate")),
            "96000 Hz should be rejected: {:#?}",
            issues,
        );
    }

    // ════════════════════════════════════════════════════════════════════════
    // Essence descriptor completeness (§6.2)
    // ════════════════════════════════════════════════════════════════════════

    #[test]
    fn app2e_flags_cdci_missing_sample_rate() {
        let ed_id = uuid(10);
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: ed_id,
                rgba_descriptor: None,
                cdci_descriptor: Some(CDCIDescriptor {
                    instance_id: None,
                    stored_width: Some(1920),
                    stored_height: Some(1080),
                    sample_rate: None, // Missing!
                    image_aspect_ratio: None,
                    color_primaries: Some(ColorPrimaries::Bt709),
                    transfer_characteristic: Some(TransferCharacteristic::Bt709),
                    coding_equations: Some(CodingEquations::Bt709),
                    picture_compression: Some(VideoCodec::Jpeg2000),
                    component_depth: Some(10),
                    frame_layout: Some("FullFrame".to_string()),
                    display_width: None,
                    display_height: None,
                    display_f2_offset: None,
                    horizontal_subsampling: Some(2),
                    vertical_subsampling: Some(1),
                    color_siting: Some(0),
                    black_ref_level: Some(64),
                    white_ref_level: Some(940),
                    color_range: Some(897),
                    stored_f2_offset: None,
                    sampled_width: None,
                    sampled_height: None,
                    sampled_x_offset: None,
                    sampled_y_offset: None,
                    alpha_transparency: None,
                    image_alignment_offset: None,
                    image_start_offset: None,
                    image_end_offset: None,
                    field_dominance: None,
                    reversed_byte_order: None,
                    padding_bits: None,
                    alpha_sample_depth: None,
                    linked_track_id: None,
                    active_width: None,
                    active_height: None,
                    sub_descriptors: None,
                }),
                wave_pcm_descriptor: None,
                dc_timed_text_descriptor: None,
                iab_essence_descriptor: None,
                isxd_data_essence_descriptor: None,
            }],
        });

        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code == St2067_21_2023::RequiredSampleRate.code()),
            "Missing SampleRate should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_flags_wave_missing_channel_count() {
        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: None, // Missing!
            quantization_bits: Some(24),
            linked_track_id: None,
            sub_descriptors: None,
        });
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code == St2067_21_2023::RequiredChannelCount.code()),
            "Missing ChannelCount should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_flags_wave_missing_quantization_bits() {
        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: Some(2),
            quantization_bits: None, // Missing!
            linked_track_id: None,
            sub_descriptors: None,
        });
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code == St2067_21_2023::RequiredQuantizationBits.code()),
            "Missing QuantizationBits should be flagged: {:#?}",
            issues,
        );
    }

    // ── XSD structural validation tests ─────────────────────────────────────

    /// XSD: is_valid_xs_datetime accepts standard formats.
    #[test]
    fn xs_datetime_valid_formats() {
        assert!(is_valid_xs_datetime("2024-01-01T00:00:00Z"));
        assert!(is_valid_xs_datetime("2024-01-01T00:00:00"));
        assert!(is_valid_xs_datetime("2024-01-01T12:30:45.123Z"));
        assert!(is_valid_xs_datetime("2024-01-01T12:30:45+05:30"));
    }

    #[test]
    fn xs_datetime_invalid_formats() {
        assert!(!is_valid_xs_datetime(""));
        assert!(!is_valid_xs_datetime("2024"));
        assert!(!is_valid_xs_datetime("not-a-date"));
        assert!(!is_valid_xs_datetime("01-01-2024T00:00:00"));
    }

    /// XSD §88: EditRate is required on the composition.
    #[test]
    fn core_flags_missing_edit_rate() {
        let cpl = minimal_cpl(); // edit_rate is None
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("EditRate")),
            "Missing EditRate should be flagged: {:#?}",
            issues,
        );
    }

    /// XSD §88: EditRate present passes.
    #[test]
    fn core_accepts_present_edit_rate() {
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        // Add a sequence so no "empty segment" error
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(3),
                track_id: uuid(4),
                resource_list: ResourceList {
                    resources: vec![make_resource(None)],
                },
            });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("-EditRate")),
            "Present EditRate should not be flagged: {:#?}",
            issues,
        );
    }

    /// XSD: IssueDate format validation.
    #[test]
    fn core_warns_invalid_issue_date_format() {
        let mut cpl = minimal_cpl();
        cpl.issue_date = "not-a-date".to_string();
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("IssueDate-Format")),
            "Invalid IssueDate format should produce warning: {:#?}",
            issues,
        );
    }

    /// XSD: Empty IssueDate flags error.
    #[test]
    fn core_flags_empty_issue_date() {
        let mut cpl = minimal_cpl();
        cpl.issue_date = "".to_string();
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("IssueDate") && i.severity == Severity::Error),
            "Empty IssueDate should be an error: {:#?}",
            issues,
        );
    }

    /// XSD §121-127: CompositionTimecode completeness.
    #[test]
    fn core_flags_incomplete_composition_timecode() {
        use crate::cpl::CompositionTimecode;

        let mut cpl = minimal_cpl();
        cpl.composition_timecode = Some(CompositionTimecode {
            timecode_drop_frame: None, // Missing!
            timecode_rate: Some(24),
            timecode_start_address: None, // Missing!
        });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("CompositionTimecode-DropFrame")),
            "Missing TimecodeDropFrame should be flagged: {:#?}",
            issues,
        );
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("CompositionTimecode-StartAddress")),
            "Missing TimecodeStartAddress should be flagged: {:#?}",
            issues,
        );
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("CompositionTimecode-Rate")),
            "Present TimecodeRate should not be flagged: {:#?}",
            issues,
        );
    }

    /// XSD SequenceType: ResourceList must not be empty.
    #[test]
    fn core_flags_empty_resource_list() {
        let mut cpl = minimal_cpl();
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(3),
                track_id: uuid(4),
                resource_list: ResourceList {
                    resources: vec![], // Empty!
                },
            });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("ResourceList-Empty")),
            "Empty ResourceList should be flagged: {:#?}",
            issues,
        );
    }

    // ── #49: ContentVersion.LabelText required validation ────────────────

    /// ContentVersion with LabelText present should not be flagged.
    #[test]
    fn core_accepts_content_version_with_label_text() {
        let mut cpl = minimal_cpl();
        cpl.content_version_list = Some(ContentVersionList {
            content_versions: vec![ContentVersion {
                id: "urn:uuid:00000000-0000-0000-0000-000000000099".to_string(),
                label_text: Some(LanguageString {
                    text: "Version 1".to_string(),
                    language: Some(LanguageTag("en".to_string())),
                }),
            }],
        });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("ContentVersionLabelTextMissing")),
            "ContentVersion with LabelText should not be flagged: {:#?}",
            issues,
        );
    }

    /// ContentVersion missing LabelText should be flagged.
    #[test]
    fn core_flags_content_version_missing_label_text() {
        let mut cpl = minimal_cpl();
        cpl.content_version_list = Some(ContentVersionList {
            content_versions: vec![ContentVersion {
                id: "urn:uuid:00000000-0000-0000-0000-000000000099".to_string(),
                label_text: None,
            }],
        });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("ContentVersionLabelTextMissing")),
            "Missing LabelText should be flagged: {:#?}",
            issues,
        );
    }

    // ── #52: Marker label vocabulary validation ──────────────────────────

    /// Recognized SMPTE marker labels (FFOC, LFOC) should not be flagged.
    #[test]
    fn core_accepts_recognized_marker_labels() {
        let mut cpl = minimal_cpl();
        cpl.segment_list.segments[0]
            .sequence_list
            .marker_sequences
            .push(MarkerSequence {
                id: uuid(20),
                track_id: uuid(21),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(22),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: None,
                        source_duration: None,
                        source_encoding: None,
                        track_file_id: None,
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![
                            MarkerInfo {
                                annotation: None,
                                label: MarkerLabelElement::from(MarkerLabel::Ffoc),
                                offset: 0,
                            },
                            MarkerInfo {
                                annotation: None,
                                label: MarkerLabelElement::from(MarkerLabel::Lfoc),
                                offset: 99,
                            },
                        ],
                    }],
                },
            });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("MarkerLabelUnknown")),
            "Recognized marker labels should not be flagged: {:#?}",
            issues,
        );
    }

    /// Unrecognized marker label under SMPTE scope should be flagged.
    #[test]
    fn core_flags_unrecognized_marker_label_under_smpte_scope() {
        let mut cpl = minimal_cpl();
        cpl.segment_list.segments[0]
            .sequence_list
            .marker_sequences
            .push(MarkerSequence {
                id: uuid(20),
                track_id: uuid(21),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(22),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: None,
                        source_duration: None,
                        source_encoding: None,
                        track_file_id: None,
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![MarkerInfo {
                            annotation: None,
                            label: MarkerLabelElement::from(MarkerLabel::Other(
                                "CUSTOM_MARKER".to_string(),
                            )),
                            offset: 0,
                        }],
                    }],
                },
            });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("MarkerLabelUnknown")),
            "Unrecognized marker label under SMPTE scope should be flagged: {:#?}",
            issues,
        );
    }

    /// Custom marker label under custom (non-SMPTE) scope should not be flagged.
    #[test]
    fn core_accepts_custom_marker_label_under_custom_scope() {
        let mut cpl = minimal_cpl();
        cpl.segment_list.segments[0]
            .sequence_list
            .marker_sequences
            .push(MarkerSequence {
                id: uuid(20),
                track_id: uuid(21),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(22),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: None,
                        source_duration: None,
                        source_encoding: None,
                        track_file_id: None,
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![MarkerInfo {
                            annotation: None,
                            label: MarkerLabelElement {
                                label: MarkerLabel::Other("VENDOR_MARKER".to_string()),
                                scope: Some("http://example.com/markers".to_string()),
                            },
                            offset: 0,
                        }],
                    }],
                },
            });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("MarkerLabelUnknown")),
            "Custom marker under custom scope should not be flagged: {:#?}",
            issues,
        );
    }

    // ── #55: FrameLayout progressive constraint ──────────────────────────

    /// App2E: FullFrame (progressive) FrameLayout should be accepted.
    #[test]
    fn app2e_accepts_progressive_frame_layout() {
        let cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code == St2067_21_2023::FrameLayoutInterlaced.code()),
            "FullFrame FrameLayout should not be flagged: {:#?}",
            issues,
        );
    }

    /// App2E: SeparateFields (interlaced) FrameLayout should be flagged.
    #[test]
    fn app2e_flags_interlaced_frame_layout() {
        let ed_id = uuid(10);
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some(APP2E_APPLICATION_IDENTIFICATION.to_string()),
            max_cll: None,
            max_fall: None,
        });
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: ed_id,
                rgba_descriptor: None,
                cdci_descriptor: Some(CDCIDescriptor {
                    instance_id: None,
                    stored_width: Some(1920),
                    stored_height: Some(1080),
                    display_width: Some(1920),
                    display_height: Some(1080),
                    sample_rate: Some(EditRate::new(24, 1)),
                    image_aspect_ratio: None,
                    color_primaries: Some(ColorPrimaries::Bt709),
                    transfer_characteristic: Some(TransferCharacteristic::Bt709),
                    coding_equations: Some(CodingEquations::Bt709),
                    picture_compression: Some(VideoCodec::Jpeg2000),
                    component_depth: Some(10),
                    frame_layout: Some("SeparateFields".to_string()),
                    display_f2_offset: None,
                    horizontal_subsampling: Some(2),
                    vertical_subsampling: Some(1),
                    color_siting: Some(0),
                    black_ref_level: Some(64),
                    white_ref_level: Some(940),
                    color_range: Some(897),
                    stored_f2_offset: None,
                    sampled_width: None,
                    sampled_height: None,
                    sampled_x_offset: None,
                    sampled_y_offset: None,
                    alpha_transparency: None,
                    image_alignment_offset: None,
                    image_start_offset: None,
                    image_end_offset: None,
                    field_dominance: Some(1),
                    reversed_byte_order: None,
                    padding_bits: None,
                    alpha_sample_depth: None,
                    linked_track_id: None,
                    active_width: None,
                    active_height: None,
                    sub_descriptors: None,
                }),
                wave_pcm_descriptor: None,
                iab_essence_descriptor: None,
                dc_timed_text_descriptor: None,
                isxd_data_essence_descriptor: None,
            }],
        });
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code == St2067_21_2023::FrameLayoutInterlaced.code()),
            "SeparateFields FrameLayout should be flagged for App2E: {:#?}",
            issues,
        );
    }

    // ── #56: CompositionTimecode rate cross-validation ───────────────────

    /// CompositionTimecode.TimecodeRate matching EditRate should not be flagged.
    #[test]
    fn core_accepts_matching_timecode_rate() {
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        cpl.composition_timecode = Some(CompositionTimecode {
            timecode_drop_frame: Some(false),
            timecode_rate: Some(24),
            timecode_start_address: Some("00:00:00:00".to_string()),
        });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("RateMismatch")),
            "Matching TimecodeRate and EditRate should not be flagged: {:#?}",
            issues,
        );
    }

    /// CompositionTimecode.TimecodeRate not matching EditRate should be flagged.
    #[test]
    fn core_flags_mismatched_timecode_rate() {
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        cpl.composition_timecode = Some(CompositionTimecode {
            timecode_drop_frame: Some(false),
            timecode_rate: Some(25), // Mismatched!
            timecode_start_address: Some("00:00:00:00".to_string()),
        });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("RateMismatch")),
            "Mismatched TimecodeRate and EditRate should be flagged: {:#?}",
            issues,
        );
    }

    /// CompositionTimecode.TimecodeRate matching non-integer EditRate (23.976).
    #[test]
    fn core_accepts_timecode_rate_for_23976fps() {
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24000, 1001)); // 23.976 fps
        cpl.composition_timecode = Some(CompositionTimecode {
            timecode_drop_frame: Some(true),
            timecode_rate: Some(24), // Rounds to 24
            timecode_start_address: Some("00:00:00;00".to_string()),
        });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("RateMismatch")),
            "TimecodeRate 24 should match EditRate 24000/1001 (23.976 fps): {:#?}",
            issues,
        );
    }

    // ── #44: SourceEncoding cross-reference validation ───────────────────

    /// SourceEncoding referencing a valid EssenceDescriptor should not be flagged.
    #[test]
    fn core_accepts_valid_source_encoding_ref() {
        let ed_id = uuid(10);
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: ed_id,
                rgba_descriptor: None,
                cdci_descriptor: None,
                wave_pcm_descriptor: None,
                iab_essence_descriptor: None,
                dc_timed_text_descriptor: Some(DCTimedTextDescriptor {
                    instance_id: None,
                    linked_track_id: None,
                    sample_rate: None,
                    namespace_uri: Some("http://www.w3.org/ns/ttml".to_string()),
                    rfc5646_language_tag_list: vec![],
                }),
                isxd_data_essence_descriptor: None,
            }],
        });
        cpl.segment_list.segments[0]
            .sequence_list
            .subtitles_sequences
            .push(SubtitlesSequence {
                id: uuid(20),
                track_id: uuid(21),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(22),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: None,
                        source_duration: None,
                        source_encoding: Some(ed_id),
                        track_file_id: Some(uuid(50)),
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("SourceEncoding")),
            "Valid SourceEncoding ref should not be flagged: {:#?}",
            issues,
        );
    }

    /// SourceEncoding referencing a non-existent EssenceDescriptor should be flagged.
    #[test]
    fn core_flags_unresolved_source_encoding_ref() {
        let ed_id = uuid(10);
        let bad_ref = uuid(99); // Does not exist in EDL
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: ed_id,
                rgba_descriptor: None,
                cdci_descriptor: None,
                wave_pcm_descriptor: None,
                iab_essence_descriptor: None,
                dc_timed_text_descriptor: None,
                isxd_data_essence_descriptor: None,
            }],
        });
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .push(MainImageSequence {
                id: uuid(20),
                track_id: uuid(21),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(22),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: None,
                        source_duration: None,
                        source_encoding: Some(bad_ref),
                        track_file_id: Some(uuid(50)),
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("SourceEncodingUnresolved")),
            "Unresolved SourceEncoding should be flagged: {:#?}",
            issues,
        );
    }

    // ── #54: ContentKind vocabulary validation ───────────────────────────

    /// Known ContentKind under SMPTE scope should not be flagged.
    #[test]
    fn core_accepts_known_content_kind() {
        let mut cpl = minimal_cpl();
        cpl.content_kind = ContentKindElement::from(ContentKind::Feature);
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("ContentKindUnknown")),
            "Known ContentKind should not be flagged: {:#?}",
            issues,
        );
    }

    /// Unknown ContentKind under SMPTE scope should be flagged.
    #[test]
    fn core_flags_unknown_content_kind_under_smpte_scope() {
        let mut cpl = minimal_cpl();
        cpl.content_kind =
            ContentKindElement::from(ContentKind::Other("SomeFutureKind".to_string()));
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("ContentKindUnknown")),
            "Unknown ContentKind under SMPTE scope should be flagged: {:#?}",
            issues,
        );
    }

    /// Unknown ContentKind under custom scope should NOT be flagged.
    #[test]
    fn core_accepts_custom_content_kind_under_custom_scope() {
        let mut cpl = minimal_cpl();
        cpl.content_kind = ContentKindElement {
            kind: ContentKind::Other("VendorSpecific".to_string()),
            scope: Some("http://example.com/content-kinds".to_string()),
        };
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("ContentKindUnknown")),
            "Custom ContentKind under custom scope should not be flagged: {:#?}",
            issues,
        );
    }

    // ── #50: Audio channel homogeneity ───────────────────────────────────

    /// All audio descriptors with same channel count should not be flagged.
    #[test]
    fn app2e_accepts_homogeneous_audio_channels() {
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some(APP2E_APPLICATION_IDENTIFICATION.to_string()),
            max_cll: None,
            max_fall: None,
        });
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![
                EssenceDescriptor {
                    id: uuid(10),
                    rgba_descriptor: None,
                    cdci_descriptor: None,
                    wave_pcm_descriptor: Some(WAVEPCMDescriptor {
                        instance_id: None,
                        sample_rate: Some(EditRate::new(48000, 1)),
                        audio_sample_rate: None,
                        quantization_bits: Some(24),
                        channel_count: Some(6),
                        linked_track_id: None,
                        sub_descriptors: None,
                    }),
                    iab_essence_descriptor: None,
                    dc_timed_text_descriptor: None,
                    isxd_data_essence_descriptor: None,
                },
                EssenceDescriptor {
                    id: uuid(11),
                    rgba_descriptor: None,
                    cdci_descriptor: None,
                    wave_pcm_descriptor: Some(WAVEPCMDescriptor {
                        instance_id: None,
                        sample_rate: Some(EditRate::new(48000, 1)),
                        audio_sample_rate: None,
                        quantization_bits: Some(24),
                        channel_count: Some(6),
                        linked_track_id: None,
                        sub_descriptors: None,
                    }),
                    iab_essence_descriptor: None,
                    dc_timed_text_descriptor: None,
                    isxd_data_essence_descriptor: None,
                },
            ],
        });
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("AudioHomogeneity")),
            "Homogeneous audio channels should not be flagged: {:#?}",
            issues,
        );
    }

    /// A CPL with separate 5.1 and stereo audio tracks must NOT be flagged.
    /// §7.3 homogeneity applies within a virtual track, not across tracks.
    #[test]
    fn app2e_accepts_mixed_channel_count_across_tracks() {
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some(APP2E_APPLICATION_IDENTIFICATION.to_string()),
            max_cll: None,
            max_fall: None,
        });
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![
                EssenceDescriptor {
                    id: uuid(10),
                    rgba_descriptor: None,
                    cdci_descriptor: None,
                    wave_pcm_descriptor: Some(WAVEPCMDescriptor {
                        instance_id: None,
                        sample_rate: Some(EditRate::new(48000, 1)),
                        audio_sample_rate: None,
                        quantization_bits: Some(24),
                        channel_count: Some(6), // 5.1
                        linked_track_id: None,
                        sub_descriptors: None,
                    }),
                    iab_essence_descriptor: None,
                    dc_timed_text_descriptor: None,
                    isxd_data_essence_descriptor: None,
                },
                EssenceDescriptor {
                    id: uuid(11),
                    rgba_descriptor: None,
                    cdci_descriptor: None,
                    wave_pcm_descriptor: Some(WAVEPCMDescriptor {
                        instance_id: None,
                        sample_rate: Some(EditRate::new(48000, 1)),
                        audio_sample_rate: None,
                        quantization_bits: Some(24),
                        channel_count: Some(2), // Stereo — different!
                        linked_track_id: None,
                        sub_descriptors: None,
                    }),
                    iab_essence_descriptor: None,
                    dc_timed_text_descriptor: None,
                    isxd_data_essence_descriptor: None,
                },
            ],
        });
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("AudioHomogeneity")),
            "Mixed channel counts across separate audio tracks must not be flagged: {:#?}",
            issues,
        );
    }

    // ── #51: HIC/FN caption constraints ─────────────────────────────────

    /// HearingImpaired captions referencing timed text descriptor should be accepted.
    #[test]
    fn app2e_accepts_hic_with_timed_text() {
        let ed_id = uuid(10);
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some(APP2E_APPLICATION_IDENTIFICATION.to_string()),
            max_cll: None,
            max_fall: None,
        });
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: ed_id,
                rgba_descriptor: None,
                cdci_descriptor: None,
                wave_pcm_descriptor: None,
                iab_essence_descriptor: None,
                dc_timed_text_descriptor: Some(DCTimedTextDescriptor {
                    instance_id: None,
                    linked_track_id: None,
                    sample_rate: None,
                    namespace_uri: Some("http://www.w3.org/ns/ttml".to_string()),
                    rfc5646_language_tag_list: vec![LanguageTag("en".to_string())],
                }),
                isxd_data_essence_descriptor: None,
            }],
        });
        cpl.segment_list.segments[0]
            .sequence_list
            .hearing_impaired_captions_sequences
            .push(HearingImpairedCaptionsSequence {
                id: uuid(20),
                track_id: uuid(21),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(22),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: None,
                        source_duration: None,
                        source_encoding: Some(ed_id),
                        track_file_id: Some(uuid(50)),
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("5.6-HIC")),
            "HIC with timed text should not be flagged: {:#?}",
            issues,
        );
    }

    /// HearingImpaired captions referencing non-timed-text descriptor should be flagged.
    #[test]
    fn app2e_flags_hic_without_timed_text() {
        let ed_id = uuid(10);
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some(APP2E_APPLICATION_IDENTIFICATION.to_string()),
            max_cll: None,
            max_fall: None,
        });
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: ed_id,
                rgba_descriptor: None,
                cdci_descriptor: None,
                wave_pcm_descriptor: Some(WAVEPCMDescriptor {
                    instance_id: None,
                    sample_rate: Some(EditRate::new(48000, 1)),
                    quantization_bits: Some(24),
                    channel_count: Some(2),
                    audio_sample_rate: None,
                    linked_track_id: None,
                    sub_descriptors: None,
                }),
                iab_essence_descriptor: None,
                dc_timed_text_descriptor: None, // No timed text!
                isxd_data_essence_descriptor: None,
            }],
        });
        cpl.segment_list.segments[0]
            .sequence_list
            .hearing_impaired_captions_sequences
            .push(HearingImpairedCaptionsSequence {
                id: uuid(20),
                track_id: uuid(21),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(22),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: None,
                        source_duration: None,
                        source_encoding: Some(ed_id),
                        track_file_id: Some(uuid(50)),
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("5.6/HICTimedText")),
            "HIC without timed text should be flagged: {:#?}",
            issues,
        );
    }

    /// ForcedNarrative referencing non-timed-text descriptor should be flagged.
    #[test]
    fn app2e_flags_fn_without_timed_text() {
        let ed_id = uuid(10);
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some(APP2E_APPLICATION_IDENTIFICATION.to_string()),
            max_cll: None,
            max_fall: None,
        });
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: ed_id,
                rgba_descriptor: None,
                cdci_descriptor: None,
                wave_pcm_descriptor: Some(WAVEPCMDescriptor {
                    instance_id: None,
                    sample_rate: Some(EditRate::new(48000, 1)),
                    quantization_bits: Some(24),
                    channel_count: Some(2),
                    linked_track_id: None,
                    sub_descriptors: None,
                    audio_sample_rate: None,
                }),
                iab_essence_descriptor: None,
                dc_timed_text_descriptor: None,
                isxd_data_essence_descriptor: None,
            }],
        });
        cpl.segment_list.segments[0]
            .sequence_list
            .forced_narrative_sequences
            .push(ForcedNarrativeSequence {
                id: uuid(20),
                track_id: uuid(21),
                resource_list: ResourceList {
                    resources: vec![Resource {
                        id: uuid(22),
                        annotation: None,
                        edit_rate: None,
                        intrinsic_duration: 100,
                        entry_point: None,
                        source_duration: None,
                        source_encoding: Some(ed_id),
                        track_file_id: Some(uuid(50)),
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![],
                    }],
                },
            });
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("5.6/FNTimedText")),
            "ForcedNarrative without timed text should be flagged: {:#?}",
            issues,
        );
    }

    // ── #46: ContentMaturityRating / ApplicationIdentification ───────────

    /// App2E: Missing ApplicationIdentification should be flagged.
    #[test]
    fn app2e_flags_missing_application_identification() {
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: None,
            max_cll: None,
            max_fall: None,
        });
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("7.1/ApplicationIdentification")),
            "Missing ApplicationIdentification should be flagged: {:#?}",
            issues,
        );
    }

    // ── #47: LocaleList / RFC 5646 / ISO 3166-1 validation ──────────────

    /// Valid LocaleList with proper language tags and region codes should pass.
    #[test]
    fn app2e_accepts_valid_locale_list() {
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some(APP2E_APPLICATION_IDENTIFICATION.to_string()),
            max_cll: None,
            max_fall: None,
        });
        cpl.locale_list = Some(LocaleList {
            locales: vec![Locale {
                language_list: Some(LanguageList {
                    languages: vec![LanguageTag("en".to_string()), LanguageTag("fr".to_string())],
                }),
                content_maturity_rating_list: None,
                region_list: Some(RegionList {
                    regions: vec!["US".to_string(), "FR".to_string()],
                }),
            }],
        });
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("5.3/")),
            "Valid locale should not be flagged: {:#?}",
            issues,
        );
    }

    /// Invalid region code (lowercase) should be flagged.
    #[test]
    fn app2e_flags_invalid_region_code() {
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some(APP2E_APPLICATION_IDENTIFICATION.to_string()),
            max_cll: None,
            max_fall: None,
        });
        cpl.locale_list = Some(LocaleList {
            locales: vec![Locale {
                language_list: None,
                region_list: Some(RegionList {
                    regions: vec!["us".to_string()], // Lowercase = invalid
                }),
                content_maturity_rating_list: None,
            }],
        });
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("5.3/RegionCode")),
            "Invalid region code should be flagged: {:#?}",
            issues,
        );
    }

    /// Empty language tag should be flagged.
    #[test]
    fn app2e_flags_empty_locale_language_tag() {
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some(APP2E_APPLICATION_IDENTIFICATION.to_string()),
            max_cll: None,
            max_fall: None,
        });
        cpl.locale_list = Some(LocaleList {
            locales: vec![Locale {
                language_list: Some(LanguageList {
                    languages: vec![LanguageTag("".to_string())],
                }),
                region_list: None,
                content_maturity_rating_list: None,
            }],
        });
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("5.3/EmptyLanguageTag")),
            "Empty language tag should be flagged: {:#?}",
            issues,
        );
    }

    // ── #48: Timed text extended validation ──────────────────────────────

    /// Timed text descriptor missing SampleRate should be warned.
    #[test]
    fn core_warns_timed_text_missing_sample_rate() {
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: uuid(10),
                rgba_descriptor: None,
                cdci_descriptor: None,
                wave_pcm_descriptor: None,
                iab_essence_descriptor: None,
                dc_timed_text_descriptor: Some(DCTimedTextDescriptor {
                    instance_id: None,
                    linked_track_id: None,
                    sample_rate: None, // Missing!
                    namespace_uri: Some("http://www.w3.org/ns/ttml".to_string()),
                    rfc5646_language_tag_list: vec![LanguageTag("en".to_string())],
                }),
                isxd_data_essence_descriptor: None,
            }],
        });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("TimedText-SampleRate")),
            "Missing timed text SampleRate should be warned: {:#?}",
            issues,
        );
    }

    /// Timed text descriptor with valid SampleRate should not be warned.
    #[test]
    fn core_accepts_timed_text_with_sample_rate() {
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: uuid(10),
                rgba_descriptor: None,
                cdci_descriptor: None,
                wave_pcm_descriptor: None,
                iab_essence_descriptor: None,
                dc_timed_text_descriptor: Some(DCTimedTextDescriptor {
                    instance_id: None,
                    linked_track_id: None,
                    sample_rate: Some(EditRate::new(24, 1)),
                    namespace_uri: Some("http://www.w3.org/ns/ttml".to_string()),
                    rfc5646_language_tag_list: vec![LanguageTag("en".to_string())],
                }),
                isxd_data_essence_descriptor: None,
            }],
        });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("TimedText-SampleRate")),
            "Valid timed text SampleRate should not be warned: {:#?}",
            issues,
        );
    }

    /// Empty language tag in timed text should be warned.
    #[test]
    fn core_warns_timed_text_empty_language_tag() {
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![EssenceDescriptor {
                id: uuid(10),
                rgba_descriptor: None,
                cdci_descriptor: None,
                wave_pcm_descriptor: None,
                iab_essence_descriptor: None,
                dc_timed_text_descriptor: Some(DCTimedTextDescriptor {
                    instance_id: None,
                    linked_track_id: None,
                    sample_rate: Some(EditRate::new(24, 1)),
                    namespace_uri: Some("http://www.w3.org/ns/ttml".to_string()),
                    rfc5646_language_tag_list: vec![LanguageTag("".to_string())],
                }),
                isxd_data_essence_descriptor: None,
            }],
        });
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("TimedText-EmptyLanguageTag")),
            "Empty language tag should be warned: {:#?}",
            issues,
        );
    }

    // ── #43: Digital Signatures notice ───────────────────────────────────

    /// CPL with 2020 namespace should emit digital signature info notice.
    #[test]
    fn core_emits_digital_signature_notice_for_2020_cpl() {
        let cpl = minimal_cpl(); // Uses Smpte2067_3_2020
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("DigitalSignature") && i.severity == Severity::Info),
            "2020 CPL should emit digital signature info notice: {:#?}",
            issues,
        );
    }

    /// CPL with 2013 namespace should not emit digital signature notice.
    #[test]
    fn core_no_digital_signature_notice_for_2013_cpl() {
        let mut cpl = minimal_cpl();
        cpl.namespace = CplNamespace::Smpte2067_3_2013;
        let v = CoreConstraints2013;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("DigitalSignature")),
            "2013 CPL should not emit digital signature notice: {:#?}",
            issues,
        );
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Normative-claim gap closure: CDCI Table 8
    // ═════════════════════════════════════════════════════════════════════════

    /// Table 8: SampledHeight ≠ StoredHeight shall be flagged.
    #[test]
    fn app2e_flags_cdci_sampled_height_mismatch() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.sampled_height = Some(720); // ≠ stored_height (1080)
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.2.1/SampledHeight")),
            "Should flag SampledHeight ≠ StoredHeight: {:#?}",
            issues,
        );
    }

    /// Table 8: SampledYOffset non-zero shall be flagged.
    #[test]
    fn app2e_flags_cdci_sampled_y_offset_nonzero() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.sampled_y_offset = Some(1);
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.2.1/SampledYOffset")),
            "Should flag SampledYOffset ≠ 0: {:#?}",
            issues,
        );
    }

    /// Table 8: SampledXOffset non-zero shall be flagged.
    #[test]
    fn app2e_flags_cdci_sampled_x_offset_nonzero() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.sampled_x_offset = Some(5);
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.2.1/SampledXOffset")),
            "Should flag SampledXOffset ≠ 0: {:#?}",
            issues,
        );
    }

    /// Table 8: Missing CodingEquations shall be flagged.
    #[test]
    fn app2e_flags_cdci_coding_equations_missing() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.coding_equations = None;
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.2.1/CodingEquations")),
            "Should flag missing CodingEquations: {:#?}",
            issues,
        );
    }

    /// §6.2.3: Unrecognized CodingEquations UL shall be flagged.
    #[test]
    fn app2e_flags_cdci_coding_equations_unknown() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.coding_equations = Some(CodingEquations::Unknown(
                        "06.0e.2b.34.04.01.01.0d.04.01.01.01.ff.00.00.00".to_string(),
                    ));
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("6.2.3")),
            "Should flag unknown CodingEquations UL: {:#?}",
            issues,
        );
    }

    /// Table 8: Missing TransferCharacteristic shall be flagged.
    #[test]
    fn app2e_flags_cdci_transfer_characteristic_missing() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.transfer_characteristic = None;
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.2.1/TransferCharacteristic")),
            "Should flag missing TransferCharacteristic: {:#?}",
            issues,
        );
    }

    /// §6.2.2: Unrecognized TransferCharacteristic UL shall be flagged.
    #[test]
    fn app2e_flags_cdci_transfer_characteristic_unknown() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.transfer_characteristic = Some(TransferCharacteristic::Unknown(
                        "06.0e.2b.34.04.01.01.0d.04.01.01.01.ff.00.00.00".to_string(),
                    ));
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("6.2.2")),
            "Should flag unknown TransferCharacteristic UL: {:#?}",
            issues,
        );
    }

    /// Table 8: Missing ColorPrimaries shall be flagged.
    #[test]
    fn app2e_flags_cdci_color_primaries_missing() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.color_primaries = None;
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.2.1/ColorPrimaries")),
            "Should flag missing ColorPrimaries: {:#?}",
            issues,
        );
    }

    /// §6.2.4: Unrecognized ColorPrimaries UL shall be flagged.
    #[test]
    fn app2e_flags_cdci_color_primaries_unknown() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.color_primaries = Some(ColorPrimaries::Unknown(
                        "06.0e.2b.34.04.01.01.0d.04.01.01.01.ff.00.00.00".to_string(),
                    ));
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("6.2.4")),
            "Should flag unknown ColorPrimaries UL: {:#?}",
            issues,
        );
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Normative-claim gap closure: CDCI Table 12
    // ═════════════════════════════════════════════════════════════════════════

    /// Table 12: HorizontalSubsampling ≠ {1,2} shall be flagged.
    #[test]
    fn app2e_flags_cdci_horizontal_subsampling_invalid() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.horizontal_subsampling = Some(3);
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.4/HorizontalSubsampling")),
            "Should flag HorizontalSubsampling=3: {:#?}",
            issues,
        );
    }

    /// Table 12: Missing HorizontalSubsampling shall be flagged.
    #[test]
    fn app2e_flags_cdci_horizontal_subsampling_missing() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.horizontal_subsampling = None;
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.4/HorizontalSubsampling")),
            "Should flag missing HorizontalSubsampling: {:#?}",
            issues,
        );
    }

    /// Table 12: ColorSiting non-zero shall be flagged.
    #[test]
    fn app2e_flags_cdci_color_siting_nonzero() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.color_siting = Some(3);
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("6.4/ColorSiting")),
            "Should flag ColorSiting ≠ 0: {:#?}",
            issues,
        );
    }

    /// Table 12: Missing ColorSiting shall be flagged.
    #[test]
    fn app2e_flags_cdci_color_siting_missing() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.color_siting = None;
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("6.4/ColorSiting")),
            "Should flag missing ColorSiting: {:#?}",
            issues,
        );
    }

    /// Table 12: VerticalSubsampling ≠ 1 shall be flagged.
    #[test]
    fn app2e_flags_cdci_vertical_subsampling_invalid() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.vertical_subsampling = Some(2);
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.4/VerticalSubsampling")),
            "Should flag VerticalSubsampling ≠ 1: {:#?}",
            issues,
        );
    }

    /// Table 12: Missing VerticalSubsampling shall be flagged.
    #[test]
    fn app2e_flags_cdci_vertical_subsampling_missing() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.vertical_subsampling = None;
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.4/VerticalSubsampling")),
            "Should flag missing VerticalSubsampling: {:#?}",
            issues,
        );
    }

    /// Table 12: ComponentDepth not in {8,10,12,16} shall be flagged.
    #[test]
    fn app2e_flags_cdci_component_depth_invalid() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.component_depth = Some(14);
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("6.4/ComponentDepth")),
            "Should flag ComponentDepth=14: {:#?}",
            issues,
        );
    }

    /// Table 12: Missing ComponentDepth shall be flagged.
    #[test]
    fn app2e_flags_cdci_component_depth_missing() {
        let mut cpl = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        );
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut cdci) = ed.cdci_descriptor {
                    cdci.component_depth = None;
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("6.4/ComponentDepth")),
            "Should flag missing ComponentDepth: {:#?}",
            issues,
        );
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Normative-claim gap closure: RGBA Table 10/11
    // ═════════════════════════════════════════════════════════════════════════

    /// Table 10: Missing ComponentMaxRef shall be flagged.
    #[test]
    fn app2e_flags_rgba_component_max_ref_missing() {
        let mut cpl =
            cpl_with_rgba_descriptor(ColorPrimaries::P3D65, TransferCharacteristic::PqSt2084);
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut rgba) = ed.rgba_descriptor {
                    rgba.component_max_ref = None;
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.3/ComponentMaxRef")),
            "Should flag missing ComponentMaxRef: {:#?}",
            issues,
        );
    }

    /// Table 10: Missing ComponentMinRef shall be flagged.
    #[test]
    fn app2e_flags_rgba_component_min_ref_missing() {
        let mut cpl =
            cpl_with_rgba_descriptor(ColorPrimaries::P3D65, TransferCharacteristic::PqSt2084);
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut rgba) = ed.rgba_descriptor {
                    rgba.component_min_ref = None;
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.3/ComponentMinRef")),
            "Should flag missing ComponentMinRef: {:#?}",
            issues,
        );
    }

    /// Table 10: Missing ScanningDirection shall be flagged.
    #[test]
    fn app2e_flags_rgba_scanning_direction_missing() {
        let mut cpl =
            cpl_with_rgba_descriptor(ColorPrimaries::P3D65, TransferCharacteristic::PqSt2084);
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut rgba) = ed.rgba_descriptor {
                    rgba.scanning_direction = None;
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.3/ScanningDirection")),
            "Should flag missing ScanningDirection: {:#?}",
            issues,
        );
    }

    /// Table 10: Wrong ScanningDirection shall be flagged.
    #[test]
    fn app2e_flags_rgba_scanning_direction_wrong() {
        let mut cpl =
            cpl_with_rgba_descriptor(ColorPrimaries::P3D65, TransferCharacteristic::PqSt2084);
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut rgba) = ed.rgba_descriptor {
                    rgba.scanning_direction =
                        Some("ScanningDirection_RightToLeftBottomToTop".to_string());
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("6.3/ScanningDirection")),
            "Should flag wrong ScanningDirection: {:#?}",
            issues,
        );
    }

    /// Table 11: ComponentMinRef/MaxRef mismatch for QE.1 / QE.2 shall be flagged.
    #[test]
    fn app2e_flags_rgba_table11_ref_mismatch() {
        let mut cpl =
            cpl_with_rgba_descriptor(ColorPrimaries::Bt709, TransferCharacteristic::Bt709);
        if let Some(ref mut edl) = cpl.essence_descriptor_list {
            for ed in &mut edl.essence_descriptors {
                if let Some(ref mut rgba) = ed.rgba_descriptor {
                    // min_ref=0 → QE.2 at 10-bit expects max=1023,
                    // but set max=940 which is QE.1 → mismatch.
                    rgba.component_min_ref = Some(0);
                    rgba.component_max_ref = Some(940);
                }
            }
        }
        let v = App2E2021;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("6.3.2/")),
            "Should flag Table11 ref mismatch: {:#?}",
            issues,
        );
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Normative-claim gap closure: UniqueResourceId
    // ═════════════════════════════════════════════════════════════════════════

    /// Duplicate Resource Id across sequences shall be flagged.
    #[test]
    fn core_flags_duplicate_resource_id() {
        let mut cpl = minimal_cpl();
        let dup_id = uuid(42);
        let mut sl = empty_sequence_list();
        sl.main_image_sequences.push(MainImageSequence {
            id: uuid(3),
            track_id: uuid(4),
            resource_list: ResourceList {
                resources: vec![Resource {
                    id: dup_id,
                    annotation: None,
                    edit_rate: None,
                    intrinsic_duration: 100,
                    entry_point: None,
                    source_duration: None,
                    source_encoding: Some(uuid(10)),
                    track_file_id: Some(uuid(50)),
                    repeat_count: None,
                    key_id: None,
                    hash: None,
                    markers: vec![],
                }],
            },
        });
        sl.main_audio_sequences.push(MainAudioSequence {
            id: uuid(5),
            track_id: uuid(6),
            resource_list: ResourceList {
                resources: vec![Resource {
                    id: dup_id, // Same ID as above → duplicate
                    annotation: None,
                    edit_rate: None,
                    intrinsic_duration: 100,
                    entry_point: None,
                    source_duration: None,
                    source_encoding: Some(uuid(11)),
                    track_file_id: Some(uuid(51)),
                    repeat_count: None,
                    key_id: None,
                    hash: None,
                    markers: vec![],
                }],
            },
        });
        cpl.segment_list.segments[0].sequence_list = sl;
        let v = CoreConstraints2020;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("UniqueResourceId")),
            "Should flag duplicate resource ID: {:#?}",
            issues,
        );
    }

    /// Regression guard: emitted validator codes should never fall back to :General paragraph.
    #[test]
    fn emitted_codes_do_not_use_general_fallback() {
        let mut cpl = minimal_cpl();
        cpl.edit_rate = None;
        cpl.issue_date = "invalid-date".to_string();

        let core_issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !core_issues.iter().any(|i| i.code.contains(":General/")),
            "Core validator emitted :General fallback codes: {:#?}",
            core_issues,
        );

        let app2e_issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !app2e_issues.iter().any(|i| i.code.contains(":General/")),
            "App2E validator emitted :General fallback codes: {:#?}",
            app2e_issues,
        );
    }

    // ── XSD constraint helpers ────────────────────────────────────────────────

    #[test]
    fn helper_timecode_address_valid() {
        assert!(is_valid_timecode_address("00:00:00:00"));
        assert!(is_valid_timecode_address("23:59:59:29"));
        assert!(is_valid_timecode_address("10;00;00;00")); // semicolons (drop-frame)
        assert!(is_valid_timecode_address("01/02/03/04"));
    }

    #[test]
    fn helper_timecode_address_invalid() {
        assert!(!is_valid_timecode_address("00:00:00")); // too short (HH:MM:SS)
        assert!(!is_valid_timecode_address("00:00:00:00:00")); // too long
        assert!(!is_valid_timecode_address("30:00:00:00")); // hour > 29
        assert!(!is_valid_timecode_address("00:60:00:00")); // minute > 59
        assert!(!is_valid_timecode_address("00:00:60:00")); // second > 59
        assert!(!is_valid_timecode_address("ab:cd:ef:gh")); // non-digit
    }

    #[test]
    fn helper_total_running_time_valid() {
        assert!(is_valid_total_running_time("00:00:00"));
        assert!(is_valid_total_running_time("99:59:59"));
        assert!(is_valid_total_running_time("02:30:00"));
    }

    #[test]
    fn helper_total_running_time_invalid() {
        assert!(!is_valid_total_running_time("2:30:00")); // wrong digit count
        assert!(!is_valid_total_running_time("02:60:00")); // minute > 59
        assert!(!is_valid_total_running_time("02:30:60")); // second > 59
        assert!(!is_valid_total_running_time("02:30:00:00")); // too long
    }

    #[test]
    fn helper_any_uri_valid() {
        assert!(is_valid_any_uri("http://www.movielabs.com/md/ratings"));
        assert!(is_valid_any_uri("urn:smpte:2067-3:ag"));
        assert!(is_valid_any_uri("https://example.com/agency"));
        assert!(is_valid_any_uri("relative/path"));
    }

    #[test]
    fn helper_any_uri_invalid() {
        assert!(!is_valid_any_uri("http://example.com with space"));
        assert!(!is_valid_any_uri("has\ttab"));
        assert!(!is_valid_any_uri("has\nnewline"));
    }

    // ── XSD: TotalRunningTime pattern ────────────────────────────────────────

    #[test]
    fn core_flags_invalid_total_running_time() {
        let mut cpl = minimal_cpl();
        cpl.total_running_time = Some("2:30:00".to_string()); // missing leading zero
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("TotalRunningTime-Format")),
            "Should flag invalid TotalRunningTime format: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accepts_valid_total_running_time() {
        let mut cpl = minimal_cpl();
        cpl.total_running_time = Some("02:30:00".to_string());
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("TotalRunningTime")),
            "Valid TotalRunningTime should be accepted: {:#?}",
            issues,
        );
    }

    // ── XSD: TimecodeRate > 0 ────────────────────────────────────────────────

    #[test]
    fn core_flags_timecode_rate_zero() {
        let mut cpl = minimal_cpl();
        cpl.composition_timecode = Some(CompositionTimecode {
            timecode_drop_frame: Some(false),
            timecode_rate: Some(0),
            timecode_start_address: Some("00:00:00:00".to_string()),
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("CompositionTimecode-Rate-Zero")),
            "TimecodeRate of 0 should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accepts_positive_timecode_rate() {
        let mut cpl = minimal_cpl();
        cpl.composition_timecode = Some(CompositionTimecode {
            timecode_drop_frame: Some(false),
            timecode_rate: Some(24),
            timecode_start_address: Some("00:00:00:00".to_string()),
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("CompositionTimecode-Rate-Zero")),
            "Positive TimecodeRate should be accepted: {:#?}",
            issues,
        );
    }

    // ── XSD: TimecodeStartAddress format ─────────────────────────────────────

    #[test]
    fn core_flags_invalid_timecode_start_address() {
        let mut cpl = minimal_cpl();
        cpl.composition_timecode = Some(CompositionTimecode {
            timecode_drop_frame: Some(false),
            timecode_rate: Some(24),
            timecode_start_address: Some("10:00:00".to_string()), // missing frame field
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("CompositionTimecode-StartAddress-Format")),
            "Invalid TimecodeStartAddress format should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accepts_valid_timecode_start_address() {
        let mut cpl = minimal_cpl();
        cpl.composition_timecode = Some(CompositionTimecode {
            timecode_drop_frame: Some(false),
            timecode_rate: Some(24),
            timecode_start_address: Some("10:00:00:00".to_string()),
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("CompositionTimecode-StartAddress-Format")),
            "Valid TimecodeStartAddress should be accepted: {:#?}",
            issues,
        );
    }

    // ── XSD: ContentVersionList non-empty ────────────────────────────────────

    #[test]
    fn core_flags_empty_content_version_list() {
        let mut cpl = minimal_cpl();
        cpl.content_version_list = Some(ContentVersionList {
            content_versions: vec![],
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("ContentVersionListEmpty")),
            "Empty ContentVersionList should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accepts_non_empty_content_version_list() {
        let mut cpl = minimal_cpl();
        cpl.content_version_list = Some(ContentVersionList {
            content_versions: vec![ContentVersion {
                id: "urn:uuid:00000000-0000-0000-0000-000000000001".to_string(),
                label_text: Some(LanguageString {
                    text: "v1".to_string(),
                    language: None,
                }),
            }],
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("ContentVersionListEmpty")),
            "Non-empty ContentVersionList should be accepted: {:#?}",
            issues,
        );
    }

    // ── XSD: LocaleList non-empty ─────────────────────────────────────────────

    #[test]
    fn core_flags_empty_locale_list() {
        let mut cpl = minimal_cpl();
        cpl.locale_list = Some(LocaleList { locales: vec![] });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("LocaleList-NonEmpty")),
            "Empty LocaleList should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn core_accepts_non_empty_locale_list() {
        let mut cpl = minimal_cpl();
        cpl.locale_list = Some(LocaleList {
            locales: vec![Locale {
                language_list: None,
                region_list: None,
                content_maturity_rating_list: None,
            }],
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("LocaleList-NonEmpty")),
            "Non-empty LocaleList should be accepted: {:#?}",
            issues,
        );
    }

    // ── XSD: EssenceDescriptorList non-empty ─────────────────────────────────

    #[test]
    fn core_flags_empty_essence_descriptor_list() {
        let mut cpl = minimal_cpl();
        cpl.essence_descriptor_list = Some(EssenceDescriptorList {
            essence_descriptors: vec![],
        });
        let issues = CoreConstraints2020.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("EssenceDescriptorListEmpty")),
            "Empty EssenceDescriptorList should be flagged: {:#?}",
            issues,
        );
    }

    // ── XSD: ContentMaturityRating.Agency as xs:anyURI ───────────────────────

    #[test]
    fn app2e_flags_agency_with_whitespace() {
        let mut cpl = minimal_cpl();
        cpl.locale_list = Some(LocaleList {
            locales: vec![Locale {
                language_list: None,
                region_list: None,
                content_maturity_rating_list: Some(ContentMaturityRatingList {
                    ratings: vec![ContentMaturityRating {
                        agency: "http://www.example.com bad uri".to_string(),
                        rating: Some("PG".to_string()),
                        audience: None,
                    }],
                }),
            }],
        });
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("ContentMaturityRating-Agency-URI")),
            "Agency with whitespace should be flagged: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_valid_agency_uri() {
        let mut cpl = minimal_cpl();
        cpl.locale_list = Some(LocaleList {
            locales: vec![Locale {
                language_list: None,
                region_list: None,
                content_maturity_rating_list: Some(ContentMaturityRatingList {
                    ratings: vec![ContentMaturityRating {
                        agency: "http://www.movielabs.com/md/ratings".to_string(),
                        rating: Some("PG-13".to_string()),
                        audience: None,
                    }],
                }),
            }],
        });
        let issues = App2E2021.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("ContentMaturityRating-Agency-URI")),
            "Valid Agency URI should be accepted: {:#?}",
            issues,
        );
    }
}
