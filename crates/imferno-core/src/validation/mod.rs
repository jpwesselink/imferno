//! SMPTE ST 2067 Constraints Validation Framework
//!
//! Implements the factory + trait pattern for normative IMF constraints validation:
//! one validator struct per SMPTE spec edition, dispatched by namespace URI.
//!
//! Each SMPTE spec version gets its own validator struct. The factory dispatches
//! by namespace URI. Multiple validators run per CPL — typically one for core
//! constraints and one (or more) for application profiles.
//!
//! ## Validation Architecture
//!
//! ```text
//! CPL namespaces ──→ Factory ──→ [CoreConstraints2020, App2E2023, ...]
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
//! - **ST 2067-21:2014/2016/2020 (+Am1)/2023/2026** Application Profile #2E
//!   (`App2E2014`, `App2E2016`, `App2E2020`, `App2E2023`, `App2E2026`), each
//!   against its own edition's text
//! - **ST 2067-201:2019/2021** IAB Level 0 Plug-in (`AppIabPlugin2019`, `AppIabPlugin2021`)
//! - **ST 2067-202:2023** ISXD Plug-in (`AppIsxdPlugin2023`)

mod app2e;
pub mod codes;
pub mod iab;
pub mod iab_codes;
pub mod isxd;
pub mod isxd_codes;

pub use app2e::{App2E, App2E2014, App2E2016, App2E2020, App2E2023, App2E2026, App2EEdition};
pub use iab::{AppIabPlugin2019, AppIabPlugin2021, AppIabPlugin2026, URI_2019, URI_2019_SCHEMAS};
pub use isxd::{AppIsxdPlugin2023, URI_2022};

use std::collections::{HashMap, HashSet};

use crate::assetmap::codes::CoreConstraintsCode;
use crate::cpl::codes::{St2067_3Code, St2067_3_2013, St2067_3_2016};
use crate::cpl::CompositionPlaylist;
use crate::cpl::{CodingEquations, ColorPrimaries, CplNamespace, EditRate, TransferCharacteristic};
use crate::diagnostics::{Category, Location, Severity, ValidationIssue};

// ═════════════════════════════════════════════════════════════════════════════
// Trait
// ═════════════════════════════════════════════════════════════════════════════

/// A normative constraints validator, dispatched by namespace URI.
///
/// Each SMPTE spec version implements this trait. Multiple validators
/// run per CPL — one for core constraints, one for CPL schema, and one
/// (or more) for application profiles.
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
            _ => None,
        };
        if let Some(ns) = core_ns {
            if let Some(v) = self.resolve_namespace(ns) {
                validators.push(v);
            }
        }

        push_application_validators(cpl, |uri| self.resolve_namespace(uri), &mut validators);

        push_sequence_presence_plugins(cpl, &mut validators);

        validators
    }
}

/// Resolve the CPL's ApplicationIdentification URIs (an `xs:list`) to
/// validators. When the element is absent, a URI does not resolve, or no
/// URI is an application profile (plug-in URIs such as IAB or ISXD only),
/// a validator emitting `IMFERNO:Package/ApplicationProfileNotValidated`
/// is added, so a CPL whose application profile was not checked is not
/// reported as if it passed (https://github.com/Netflix/photon/issues/407).
/// Plug-in validators still run.
fn push_application_validators(
    cpl: &CompositionPlaylist,
    resolve: impl Fn(&str) -> Option<Box<dyn ConstraintsValidator>>,
    validators: &mut Vec<Box<dyn ConstraintsValidator>>,
) {
    let uris: Vec<&str> = cpl
        .extension_properties
        .as_ref()
        .and_then(|ext| ext.application_identification.as_deref())
        .map(|ids| ids.split_whitespace().collect())
        .unwrap_or_default();
    let mut unrecognised = Vec::new();
    // URIs that resolve to a plug-in or core-constraints validator: those
    // run, but they are not an application profile.
    let mut not_app_profile = Vec::new();
    let mut any_app_profile = false;
    for uri in &uris {
        match resolve(uri) {
            Some(v) => {
                if App2EEdition::from_application_identification(uri).is_some() {
                    any_app_profile = true;
                } else {
                    not_app_profile.push(uri.to_string());
                }
                validators.push(v);
            }
            None => unrecognised.push(uri.to_string()),
        }
    }
    if uris.is_empty() || !any_app_profile || !unrecognised.is_empty() {
        validators.push(Box::new(ApplicationProfileNotValidated {
            any_validated: any_app_profile,
            unrecognised,
            not_app_profile,
        }));
    }
}

/// Emits `IMFERNO:Package/ApplicationProfileNotValidated` (see
/// [`push_application_validators`]).
struct ApplicationProfileNotValidated {
    /// An application-profile validator ran for some URI.
    any_validated: bool,
    /// URIs with no validator.
    unrecognised: Vec<String>,
    /// URIs whose (plug-in or core) validator ran but which are not an
    /// application profile.
    not_app_profile: Vec<String>,
}

impl ConstraintsValidator for ApplicationProfileNotValidated {
    fn spec_id(&self) -> &str {
        "imferno application-profile dispatch"
    }

    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        use crate::package::codes::ImfernoCode;
        let message = if self.unrecognised.is_empty() && self.not_app_profile.is_empty() {
            "CPL has no ApplicationIdentification, so no application-profile \
             validation was performed"
                .to_string()
        } else if !self.any_validated && self.unrecognised.is_empty() {
            format!(
                "CPL ApplicationIdentification lists only plug-in or core-constraints \
                 URI(s) ({}), which were validated; no application-profile validation \
                 was performed",
                self.not_app_profile.join(" ")
            )
        } else if self.any_validated {
            format!(
                "CPL ApplicationIdentification lists URI(s) imferno has no validator for: {}; \
                 those application profiles were not validated",
                self.unrecognised.join(" ")
            )
        } else {
            format!(
                "CPL ApplicationIdentification lists no application profile imferno has a \
                 validator for (no validator: {}{}), so no application-profile validation \
                 was performed",
                self.unrecognised.join(" "),
                if self.not_app_profile.is_empty() {
                    String::new()
                } else {
                    format!("; plug-in/core only: {}", self.not_app_profile.join(" "))
                }
            )
        };
        vec![
            ValidationIssue::from_code(ImfernoCode::ApplicationProfileNotValidated, message)
                .with_location(crate::diagnostics::Location::new().with_cpl(cpl.id)),
        ]
    }
}

/// Add plug-in validators signaled by sequence presence rather than by
/// `ApplicationIdentification`.
///
/// ST 2067-201 (IAB) and ST 2067-202 (ISXD) are *plug-ins* on top of an
/// application profile: real-world CPLs declare the application (e.g.
/// App2E `.../2067-21/2016`) in `ApplicationIdentification` and signal
/// plug-in usage by the presence of `iab:IABSequence` / `isxd:ISXDSequence`
/// elements in the segment sequence lists — the plug-in namespace often
/// never appears in `ApplicationIdentification` at all (this is exactly
/// how the IAB/ISXD corpus fixtures are authored). Before this dispatch
/// existed, the IAB/ISXD plugins were only reachable by direct
/// construction — `imferno validate` never ran them.
///
/// Skips a plug-in if a validator for its spec was already selected via
/// URI resolution, so a CPL that both declares the URI and carries the
/// sequences doesn't get validated twice.
fn push_sequence_presence_plugins(
    cpl: &CompositionPlaylist,
    validators: &mut Vec<Box<dyn ConstraintsValidator>>,
) {
    let has_iab = cpl
        .segment_list
        .segments
        .iter()
        .any(|s| !s.sequence_list.iab_sequences.is_empty());
    let has_isxd = cpl
        .segment_list
        .segments
        .iter()
        .any(|s| !s.sequence_list.isxd_sequences.is_empty());

    let already = |needle: &str, validators: &[Box<dyn ConstraintsValidator>]| {
        validators.iter().any(|v| v.spec_id().contains(needle))
    };

    // Every published ST 2067-201 edition (2019/2021/2026) reuses the 2019
    // namespace URI, so the sequence itself can't tell editions apart.
    // Default to the 2021 rules: 2019's stricter ChannelCount==0 requirement
    // was relaxed in 2021 (the corpus has a `valid_non_zero_essence_
    // channelcount` fixture that 2019 rules would false-positive), and
    // 2026 only adds an Annex E recommendation that pre-2026 content
    // shouldn't be warned about by default. Pin a different edition via
    // `ValidatorSelection::app_specs`.
    if has_iab && !already("2067-201", validators) {
        validators.push(Box::new(AppIabPlugin2021));
    }
    if has_isxd && !already("2067-202", validators) {
        validators.push(Box::new(AppIsxdPlugin2023));
    }
}

/// Built-in namespace/profile resolver for supported SMPTE validators.
pub struct BuiltinValidatorRegistry;

impl ValidatorRegistry for BuiltinValidatorRegistry {
    fn resolve_namespace(&self, namespace_uri: &str) -> Option<Box<dyn ConstraintsValidator>> {
        // ST 2067-21: each edition's own ApplicationIdentification value
        // (2014 §7 Table 5, 2016 §7.1 Table 8, 2020 and 2023 §7.1 Table 15,
        // 2026 §8.1 Figure 2) selects that edition. The 2023 edition's value
        // is `.../ns/2067-21/2021`; no edition defines `.../2067-21/2015` or
        // `.../ns/2067-21/2023`.
        if let Some(edition) = App2EEdition::from_application_identification(namespace_uri) {
            return Some(app2e_validator(edition));
        }
        match namespace_uri {
            "http://www.smpte-ra.org/schemas/2067-2/2013" => Some(Box::new(CoreConstraints2013)),
            "http://www.smpte-ra.org/schemas/2067-2/2016" => Some(Box::new(CoreConstraints2016)),
            "http://www.smpte-ra.org/ns/2067-2/2020" => Some(Box::new(CoreConstraints2020)),
            // ST 2067-201 IAB Level 0 plug-in. Every published edition
            // (2019/2021/2026) reuses the 2019 namespace URI — see the
            // note above `iab::URI_2019` — so the URI resolves to the
            // 2021 rules (the current unconditional edition; 2026 adds
            // only an opt-in Annex E recommendation).
            iab::URI_2019 | iab::URI_2019_SCHEMAS => Some(Box::new(AppIabPlugin2021)),
            // ST 2067-202 ISXD plug-in.
            isxd::URI_2022 => Some(Box::new(AppIsxdPlugin2023)),
            _ => None,
        }
    }
}

/// The App #2E validator of `edition`.
pub fn app2e_validator(edition: App2EEdition) -> Box<dyn ConstraintsValidator> {
    match edition {
        App2EEdition::V2014 => Box::new(App2E2014),
        App2EEdition::V2016 => Box::new(App2E2016),
        App2EEdition::V2020 => Box::new(App2E2020),
        App2EEdition::V2023 => Box::new(App2E2023),
        App2EEdition::V2026 => Box::new(App2E2026),
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

impl std::str::FromStr for CoreSpecTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v2013" => Ok(Self::St2067_2_2013),
            "v2016" => Ok(Self::St2067_2_2016),
            "v2020" => Ok(Self::St2067_2_2020),
            other => Err(format!(
                "Unsupported coreSpec '{}'. Use auto|v2013|v2016|v2020",
                other
            )),
        }
    }
}

/// An ST 2067-21 (App #2E) edition to validate against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppSpecTarget {
    St2067_21_2014,
    St2067_21_2016,
    /// ST 2067-21:2020 including Amendment 1:2020.
    St2067_21_2020,
    St2067_21_2023,
    St2067_21_2026,
}

impl AppSpecTarget {
    /// The edition this target selects.
    pub fn edition(&self) -> App2EEdition {
        match self {
            Self::St2067_21_2014 => App2EEdition::V2014,
            Self::St2067_21_2016 => App2EEdition::V2016,
            Self::St2067_21_2020 => App2EEdition::V2020,
            Self::St2067_21_2023 => App2EEdition::V2023,
            Self::St2067_21_2026 => App2EEdition::V2026,
        }
    }

    /// The ApplicationIdentification value the edition defines.
    pub fn application_identification_uri(&self) -> &'static str {
        self.edition().application_identification_uri()
    }
}

impl std::str::FromStr for AppSpecTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v2014" => Ok(Self::St2067_21_2014),
            "v2016" => Ok(Self::St2067_21_2016),
            "v2020" => Ok(Self::St2067_21_2020),
            "v2023" => Ok(Self::St2067_21_2023),
            "v2026" => Ok(Self::St2067_21_2026),
            other => Err(format!(
                "Unsupported app2eSpec '{}'. Use auto|none|v2014|v2016|v2020|v2023|v2026",
                other
            )),
        }
    }
}

/// Parse a core spec string, handling `"auto"` as `None` (auto-detect).
pub fn parse_core_spec_target(s: &str) -> Result<Option<CoreSpecTarget>, String> {
    match s {
        "auto" => Ok(None),
        other => Ok(Some(other.parse()?)),
    }
}

/// Parse an app2e spec string, handling `"auto"` as `None` (auto-detect)
/// and `"none"` as an empty list (skip app validation).
pub fn parse_app_spec_targets(s: &str) -> Result<Option<Vec<AppSpecTarget>>, String> {
    match s {
        "auto" => Ok(None),
        "none" => Ok(Some(vec![])),
        other => Ok(Some(vec![other.parse()?])),
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
        } else {
            // Auto mode — no caller-pinned app selection. Resolve from the
            // CPL's declared ApplicationIdentification, then add plug-ins
            // signaled by sequence presence (IAB/ISXD). When the caller
            // pinned app_specs/app_uris above, we respect that pin and do
            // NOT auto-add plug-ins.
            push_application_validators(cpl, |uri| self.resolve_namespace(uri), &mut validators);
            push_sequence_presence_plugins(cpl, &mut validators);
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
/// This is designed to plug directly into the package validator's
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
// Colorimetry — ST 2067-21 colorimetry systems (2020/2023 §5.3 Table 4)
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

    /// The system's designator, e.g. `"COLOR.3"`.
    pub fn designator(&self) -> &'static str {
        match self {
            Self::Color1 => "COLOR.1",
            Self::Color2 => "COLOR.2",
            Self::Color3 => "COLOR.3",
            Self::Color4 => "COLOR.4",
            Self::Color5 => "COLOR.5",
            Self::Color6 => "COLOR.6",
            Self::Color7 => "COLOR.7",
            Self::Color8 => "COLOR.8",
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
// ST 2067-2 core constraints — shared abstract base for 2013/2016/2020 editions.
// ═════════════════════════════════════════════════════════════════════════════

// xs:dateTime, TimecodeType pattern, and TotalRunningTime regex helpers were
// gutted along with their XSD-overlap emission sites — see the runtime-XSD
// architecture spike (uppsala-based) for the replacement path.

// `validate_resource_list_non_empty` was gutted — XSD-overlap check
// (CoreConstraintsCode::ResourceListEmpty mirrors xs:sequence + minOccurs=1
// on the Resource element). Replacement comes via runtime-XSD validation.

/// Shared core structure checks applied by all spec versions.
fn validate_core_structure(
    cpl: &CompositionPlaylist,
    code: fn(CoreConstraintsCode) -> &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    // Runtime XSD pre-pass: run the schema-level validator before the
    // semantic checks so structural diagnostics fire first (later
    // checks may have been induced by them). No-op only when
    // source_xml is None (e.g., the CPL was constructed manually in
    // tests rather than parsed from XML).
    issues.extend(crate::xsd::validate_parsed_cpl(cpl));

    let loc = Location::new().with_cpl(cpl.id);

    // ───────────────────────────────────────────────────────────────────────────
    // XSD-overlap structural checks (ContentTitle, TotalRunningTime, SegmentList,
    // Segment-has-sequences, EditRate, IssueDate, IssueDate format,
    // CompositionTimecode completeness + TimecodeRate>0 + TimecodeStartAddress
    // format) were gutted. They mirrored constraints already expressed in the
    // SMPTE XSDs and will be re-emitted by the runtime-XSD validation path
    // (see spike branch) via a translator that maps schema diagnostics back
    // into the CoreConstraintsCode catalogue.
    // ───────────────────────────────────────────────────────────────────────────

    // Cross-field semantic check: TimecodeRate must equal rounded CPL EditRate.
    // KEPT — XSD cannot express this; it's prose-only (cross-field invariant).
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

    // LocaleListNonEmpty + ResourceListEmpty were gutted — both are XSD-overlap
    // (xs:sequence + minOccurs=1 on Locale / Resource elements respectively).

    // ST 2067-3 constraints delegated to st2067-3 crate (§5.5.1.2, §6.4.2, §6.11, §6.12, §7.3, §7.4)
    issues.extend(crate::cpl::validate_cpl_constraints(cpl));

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
    let cpl_loc = Location::new().with_cpl(cpl.id);

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
                        .with_cpl(cpl.id)
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

                    // A: EntryPoint + SourceDuration <= IntrinsicDuration
                    // (ST 2067-3 §6.11.6). The lower bound is 0, inclusive:
                    // a SourceDuration of 0 is allowed and not flagged.
                    if let Some(source_duration) = resource.source_duration {
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
                    .with_location(Location::new().with_cpl(cpl.id).with_segment(seg_idx)),
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
/// Applies to image, audio, IAB and ISXD virtual tracks. Timed Text virtual
/// tracks are excluded (ST 2067-2:2020 §6.3.3: any edit rate).
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
                                        .with_cpl(cpl.id)
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
        // Timed Text virtual tracks (Subtitles, HearingImpairedCaptions,
        // ForcedNarrative) are not checked: ST 2067-2:2020 §6.3.3 lets their
        // Resource edit rate be any value, and 2013/2016 §6.3.3 set no
        // edit-rate constraint on these tracks.
        for seq in &sl.iab_sequences {
            check_sequence(seq, "IABSequence", issues, &mut track_edit_rates);
        }
        // AUDIT-21: ISXD sequences were missing from this loop, so
        // ST 2067-202 §6 ("The Edit Rate of an ISXD Virtual Track shall
        // be equal to the Edit Rate of the Main Image Virtual Track")
        // violations went undetected at the generic per-track level.
        for seq in &sl.isxd_sequences {
            check_sequence(seq, "ISXDSequence", issues, &mut track_edit_rates);
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
            .with_cpl(cpl.id)
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
            let s = tag.as_str();
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
/// - WAVEPCMDescriptor.ChannelCount should not exceed the soundfield group size
///   (a subset is allowed, ST 2067-2 §5.3.6.6)
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
            .with_cpl(cpl.id)
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

        // Channel count vs soundfield group. ST 2067-2 §5.3.6.6: not all
        // channels of the soundfield group need be present in the track
        // file, so only a ChannelCount above the group size is flagged.
        if let Some(expected) = expected_channel_count(tag) {
            if channel_count > expected {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        Category::Audio,
                        code(CoreConstraintsCode::SoundfieldChannelCount),
                        format!(
                            "WAVEPCMDescriptor {} has ChannelCount {} but MCATagSymbol '{}' \
                             defines only {} channels",
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
        CplNamespace::Smpte2067_3_2016 | CplNamespace::Unknown(_) => {
            St2067_3_2016::for_code(St2067_3Code::SegmentDuration)
        }
    };

    for (seg_idx, segment) in cpl.segment_list.segments.iter().enumerate() {
        let sl = &segment.sequence_list;
        let seg_loc = Location::new().with_cpl(cpl.id).with_segment(seg_idx);

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
    let supports_signatures = matches!(cpl.namespace, CplNamespace::Smpte2067_3_2016);
    if supports_signatures {
        issues.push(
            ValidationIssue::new(
                Severity::Info,
                Category::Security,
                code(CoreConstraintsCode::DigitalSignature),
                "Digital signature validation (ST 2067-2 §8) is not currently performed; \
                 Signer/Signature XML elements are not parsed",
            )
            .with_location(Location::new().with_cpl(cpl.id)),
        );
    }
}

/// ST 2067-2 §6.4.2: Every EssenceDescriptor in the EssenceDescriptorList shall be
/// referenced by at least one Resource's SourceEncoding element.
///
/// An ED that exists in the EDL but is not referenced by any Resource is a
/// "dangling" descriptor — it occupies space and implies metadata but has no
/// corresponding essence in the composition. This is rejected by ST 2067-2.
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
                .with_location(Location::new().with_cpl(cpl.id)),
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
        let loc = Location::new().with_cpl(cpl.id);

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
                .with_location(Location::new().with_cpl(cpl.id)),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

// Application Profile #2E (ST 2067-21) lives in `app2e.rs`.

// ═════════════════════════════════════════════════════════════════════════════
// Tests
// ═════════════════════════════════════════════════════════════════════════════

// ── FromStr / parse helper tests ─────────────────────────────────────────────

#[cfg(test)]
mod spec_target_tests {
    use super::*;

    #[test]
    fn core_spec_target_from_str_valid() {
        assert_eq!(
            "v2013".parse::<CoreSpecTarget>().unwrap(),
            CoreSpecTarget::St2067_2_2013
        );
        assert_eq!(
            "v2016".parse::<CoreSpecTarget>().unwrap(),
            CoreSpecTarget::St2067_2_2016
        );
        assert_eq!(
            "v2020".parse::<CoreSpecTarget>().unwrap(),
            CoreSpecTarget::St2067_2_2020
        );
    }

    #[test]
    fn core_spec_target_from_str_invalid() {
        assert!("v2099".parse::<CoreSpecTarget>().is_err());
        assert!("auto".parse::<CoreSpecTarget>().is_err());
        assert!("".parse::<CoreSpecTarget>().is_err());
    }

    #[test]
    fn app_spec_target_from_str_valid() {
        assert_eq!(
            "v2020".parse::<AppSpecTarget>().unwrap(),
            AppSpecTarget::St2067_21_2020
        );
        assert_eq!(
            "v2014".parse::<AppSpecTarget>().unwrap(),
            AppSpecTarget::St2067_21_2014
        );
        assert_eq!(
            "v2016".parse::<AppSpecTarget>().unwrap(),
            AppSpecTarget::St2067_21_2016
        );
        assert_eq!(
            "v2023".parse::<AppSpecTarget>().unwrap(),
            AppSpecTarget::St2067_21_2023
        );
        assert_eq!(
            "v2026".parse::<AppSpecTarget>().unwrap(),
            AppSpecTarget::St2067_21_2026
        );
    }

    #[test]
    fn app_spec_target_from_str_invalid() {
        assert!("none".parse::<AppSpecTarget>().is_err());
        assert!("garbage".parse::<AppSpecTarget>().is_err());
        // There is no 2021 edition of ST 2067-21 (the 2023 edition's
        // ApplicationIdentification value merely ends in /2021).
        assert!("v2021".parse::<AppSpecTarget>().is_err());
    }

    #[test]
    fn parse_core_spec_target_auto() {
        assert_eq!(parse_core_spec_target("auto").unwrap(), None);
    }

    #[test]
    fn parse_core_spec_target_specific() {
        assert_eq!(
            parse_core_spec_target("v2020").unwrap(),
            Some(CoreSpecTarget::St2067_2_2020),
        );
    }

    #[test]
    fn parse_app_spec_targets_auto() {
        assert_eq!(parse_app_spec_targets("auto").unwrap(), None);
    }

    #[test]
    fn parse_app_spec_targets_none() {
        assert_eq!(parse_app_spec_targets("none").unwrap(), Some(vec![]));
    }

    #[test]
    fn parse_app_spec_targets_specific() {
        assert_eq!(
            parse_app_spec_targets("v2023").unwrap(),
            Some(vec![AppSpecTarget::St2067_21_2023]),
        );
    }

    #[test]
    fn parse_app_spec_targets_invalid() {
        assert!(parse_app_spec_targets("garbage").is_err());
    }
}

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
    use crate::validation::codes::{St2067_21_2020, St2067_21_2023};

    /// ST 2067-21:2023 §7.1 Table 15 value.
    const APP2E_APPLICATION_IDENTIFICATION: &str = "http://www.smpte-ra.org/ns/2067-21/2021";

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
            namespace: CplNamespace::Smpte2067_3_2016,
            id: uuid(1),
            annotation: None,
            issue_date: "2024-01-01T00:00:00Z".to_string(),
            issuer: None,
            creator: None,
            content_originator: None,
            content_title: LanguageString {
                text: "Test".to_string(),
                language: Some(LanguageTag::new("en")),
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
            source_xml: None,
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

    /// Each edition's own ApplicationIdentification value selects that
    /// edition: 2014 §7 Table 5 (p6), 2016 §7.1 Table 8 (p10), 2020 §7.1
    /// Table 15 (p17), 2023 §7.1 Table 15 (p20), 2026 §8.1 Figure 2.
    #[test]
    fn factory_routes_each_app2e_edition_uri_to_its_own_validator() {
        for (uri, spec_id) in [
            (
                "http://www.smpte-ra.org/schemas/2067-21/2014",
                "ST 2067-21:2014 (App2E)",
            ),
            (
                "http://www.smpte-ra.org/schemas/2067-21/2016",
                "ST 2067-21:2016 (App2E)",
            ),
            (
                "http://www.smpte-ra.org/ns/2067-21/2020",
                "ST 2067-21:2020 (App2E)",
            ),
            (
                "http://www.smpte-ra.org/ns/2067-21/2021",
                "ST 2067-21:2023 (App2E)",
            ),
            (
                "http://www.smpte-ra.org/ns/2067-21/5ED",
                "ST 2067-21:2026 (App2E)",
            ),
        ] {
            let v = get_validator(uri).unwrap_or_else(|| panic!("{uri} not routed"));
            assert_eq!(v.spec_id(), spec_id, "{uri}");
        }
    }

    /// No edition defines `.../schemas/2067-21/2015` or
    /// `.../ns/2067-21/2023`, so neither selects an App #2E validator.
    #[test]
    fn factory_does_not_route_app2e_uris_no_edition_defines() {
        for uri in [
            "http://www.smpte-ra.org/schemas/2067-21/2015",
            "http://www.smpte-ra.org/ns/2067-21/2023",
        ] {
            assert!(get_validator(uri).is_none(), "{uri} must not be routed");
        }
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

    // `get_validators_for_cpl_returns_core_2020` was removed alongside
    // `CplNamespace::Smpte2067_3_2020`; the 2016 equivalent below now covers
    // the same dispatch path (minimal_cpl uses Smpte2067_3_2016).

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
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2016");
        assert_eq!(validators[1].spec_id(), "ST 2067-21:2023 (App2E)");
    }

    fn not_validated_issues(cpl: &CompositionPlaylist) -> Vec<ValidationIssue> {
        validate_cpl(cpl)
            .into_iter()
            .filter(|i| i.code == "IMFERNO:Package/ApplicationProfileNotValidated")
            .collect()
    }

    fn with_app_id(app_id: Option<&str>) -> CompositionPlaylist {
        let mut cpl = minimal_cpl();
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: app_id.map(str::to_string),
            max_cll: None,
            max_fall: None,
        });
        cpl
    }

    /// https://github.com/Netflix/photon/issues/407: a CPL whose application
    /// profile was not validated must say so instead of passing silently.
    #[test]
    fn missing_application_identification_is_reported() {
        for cpl in [minimal_cpl(), with_app_id(None), with_app_id(Some("  "))] {
            let issues = not_validated_issues(&cpl);
            assert_eq!(issues.len(), 1, "{issues:#?}");
            assert_eq!(issues[0].severity, Severity::Warning);
            assert!(issues[0].message.contains("no ApplicationIdentification"));
        }
    }

    #[test]
    fn unrecognised_application_identification_is_reported_with_uri() {
        let issues = not_validated_issues(&with_app_id(Some("http://example.com/unknown-app")));
        assert_eq!(issues.len(), 1, "{issues:#?}");
        assert!(issues[0].message.contains("http://example.com/unknown-app"));
        assert!(issues[0]
            .message
            .contains("no application-profile validation"));

        // Partly recognised: the App #2E part runs, the rest is named.
        let issues = not_validated_issues(&with_app_id(Some(
            "http://www.smpte-ra.org/ns/2067-21/2021 http://example.com/unknown-app",
        )));
        assert_eq!(issues.len(), 1, "{issues:#?}");
        assert!(issues[0].message.contains("http://example.com/unknown-app"));
        assert!(!issues[0].message.contains("2067-21/2021"));
    }

    #[test]
    fn recognised_application_identification_is_not_reported() {
        let cpl = with_app_id(Some("http://www.smpte-ra.org/ns/2067-21/2021"));
        assert!(not_validated_issues(&cpl).is_empty());
    }

    /// Plug-in URIs (IAB, ISXD) are not application profiles: the plug-in
    /// validator runs, but the CPL still gets the notice.
    #[test]
    fn plugin_only_application_identification_is_reported() {
        for uri in [isxd::URI_2022, iab::URI_2019] {
            let cpl = with_app_id(Some(uri));
            let issues = not_validated_issues(&cpl);
            assert_eq!(issues.len(), 1, "{uri}: {issues:#?}");
            assert!(issues[0].message.contains(uri), "{}", issues[0].message);
            let ids: Vec<_> = get_validators_for_cpl(&cpl)
                .iter()
                .map(|v| v.spec_id().to_string())
                .collect();
            assert!(
                ids.iter()
                    .any(|id| id.contains("Plug-in") || id.contains("IAB")),
                "plug-in validator still runs: {ids:?}"
            );
        }
        // App profile plus plug-in: nothing to report.
        let cpl = with_app_id(Some(&format!(
            "http://www.smpte-ra.org/ns/2067-21/2021 {}",
            isxd::URI_2022
        )));
        assert!(not_validated_issues(&cpl).is_empty());
    }

    #[test]
    fn pinned_app_selection_does_not_report() {
        // The caller chose the app profiles (here: none), so nothing is
        // left unvalidated by imferno's choice.
        let registry = ConfigurableValidatorRegistry::new(ValidatorSelection {
            app_specs: Some(vec![]),
            ..Default::default()
        });
        let issues = validate_cpl_with_registry(&minimal_cpl(), &registry);
        assert!(!issues
            .iter()
            .any(|i| i.code == "IMFERNO:Package/ApplicationProfileNotValidated"));
        // Auto mode reports.
        let auto = ConfigurableValidatorRegistry::new(ValidatorSelection::default());
        let issues = validate_cpl_with_registry(&minimal_cpl(), &auto);
        assert!(issues
            .iter()
            .any(|i| i.code == "IMFERNO:Package/ApplicationProfileNotValidated"));
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
        assert_eq!(validators.len(), 3);
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2016");
        assert_eq!(validators[1].spec_id(), "ST 2067-21:2023 (App2E)");
        assert_eq!(
            validators[2].spec_id(),
            "imferno application-profile dispatch"
        );
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
        assert_eq!(validators.len(), 2); // + no-AppId notice (photon#407)
        assert_eq!(
            validators[1].spec_id(),
            "imferno application-profile dispatch"
        );
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2016");
    }

    #[test]
    fn configurable_registry_overrides_application_identification_uris() {
        let cpl = minimal_cpl();
        let registry = ConfigurableValidatorRegistry::new(ValidatorSelection {
            app_specs: Some(vec![AppSpecTarget::St2067_21_2023]),
            ..Default::default()
        });

        let validators = registry.resolve_for_cpl(&cpl);
        assert_eq!(validators.len(), 2);
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2016");
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
        assert_eq!(validators.len(), 2); // + no-AppId notice (photon#407)
        assert_eq!(
            validators[1].spec_id(),
            "imferno application-profile dispatch"
        );
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2016");
    }

    #[test]
    fn get_validators_for_cpl_returns_core_2013_for_2013_namespace() {
        let mut cpl = minimal_cpl();
        cpl.namespace = CplNamespace::Smpte2067_3_2013;
        let validators = get_validators_for_cpl(&cpl);
        assert_eq!(validators.len(), 2); // + no-AppId notice (photon#407)
        assert_eq!(
            validators[1].spec_id(),
            "imferno application-profile dispatch"
        );
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2013");
    }

    #[test]
    fn get_validators_for_cpl_returns_empty_for_dci_legacy_namespace() {
        // DCI-era CPLs (ST 429-7:2006) have no matching core validator.
        let mut cpl = minimal_cpl();
        cpl.namespace = CplNamespace::Dci429_7;
        let validators = get_validators_for_cpl(&cpl);
        // Only the "no application profile validated" notice (photon#407).
        let ids: Vec<_> = validators.iter().map(|v| v.spec_id()).collect();
        assert_eq!(
            ids,
            vec!["imferno application-profile dispatch"],
            "DCI namespace should yield no core validator"
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
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2016");
        assert_eq!(validators[1].spec_id(), "ST 2067-21:2016 (App2E)");
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
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2016");
    }

    #[test]
    fn configurable_registry_raw_string_core_uri_override() {
        // Raw string URI override for callers that need to target an unlisted namespace.
        let mut cpl = minimal_cpl();
        cpl.namespace = CplNamespace::Smpte2067_3_2016;
        let registry = ConfigurableValidatorRegistry::new(ValidatorSelection {
            core_namespace_uri: Some("http://www.smpte-ra.org/schemas/2067-2/2016".to_string()),
            ..Default::default()
        });
        let validators = registry.resolve_for_cpl(&cpl);
        assert_eq!(validators.len(), 2); // + no-AppId notice (photon#407)
        assert_eq!(
            validators[1].spec_id(),
            "imferno application-profile dispatch"
        );
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
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2016");
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

    /// XSD: ContentTitle is `dcml:UserTextType` (xs:string-derived). The
    /// XSD grammar permits the empty string, so this is NOT a structural
    /// constraint — it has to live as a semantic check if we want it at
    /// all. The original "gutted" claim was inaccurate; left ignored
    /// pending a deliberate decision on whether to re-add semantically.
    #[test]
    #[ignore = "ContentTitle non-empty is not XSD-expressible; needs semantic check if desired"]
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

    /// XSD §57: `SegmentList` requires `Segment maxOccurs=unbounded`
    /// with default `minOccurs=1` — an empty SegmentList trips the
    /// schema-level validator (`ElementMissing/Segment`).
    #[test]
    fn core_flags_empty_segment_list() {
        let xml = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
            <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
            <IssueDate>2024-01-01T00:00:00Z</IssueDate>
            <ContentTitle>Test</ContentTitle>
            <EditRate>24 1</EditRate>
            <SegmentList/>
        </CompositionPlaylist>"#;
        let cpl = crate::cpl::parse_cpl(xml).expect("should parse despite empty SegmentList");
        let issues = CoreConstraints2013.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("Segment")),
            "Empty SegmentList should be flagged: {:#?}",
            issues,
        );
    }

    /// XSD §150-156: `SequenceList` contains an optional `MarkerSequence`
    /// followed by `xs:any namespace="##other" minOccurs="0"` — the
    /// schema explicitly *allows* an empty SequenceList. Catching this
    /// requires semantic logic ("a Segment with zero sequences is
    /// useless") and is not XSD-expressible.
    #[test]
    #[ignore = "Empty SequenceList is schema-valid (xs:any minOccurs=0); needs semantic check"]
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

    /// Add the ST 2067-21:2023 Table 15 ApplicationIdentification, which
    /// ST 2067-2 §6.1 requires, so "no errors" fixtures are conforming.
    fn with_app2e_app_id(mut cpl: CompositionPlaylist) -> CompositionPlaylist {
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some(APP2E_APPLICATION_IDENTIFICATION.to_string()),
            max_cll: None,
            max_fall: None,
        });
        cpl
    }

    #[test]
    fn app2e_validates_valid_color3_hd() {
        let cpl = with_app2e_app_id(cpl_with_cdci_descriptor(
            ColorPrimaries::Bt709,
            TransferCharacteristic::Bt709,
            CodingEquations::Bt709,
            10,
        ));
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("8.3.3")),
            "HLG should not require MaxCLL/MaxFALL"
        );
    }

    /// COLOR.8 (BT.2020 / HLG) is added to ST 2067-21:2020 by Amendment
    /// 1:2020 (§3.2 Table 3, §3.3 6.2.2–6.2.4, p3–4) and is in the 2023
    /// Table 3 (p7). The 2020 validator applies the amendment, so both
    /// accept it.
    #[test]
    fn color8_is_accepted_by_app2e_2020_with_am1_and_2023() {
        let cdci = cpl_with_cdci_descriptor(
            ColorPrimaries::Bt2020,
            TransferCharacteristic::Hlg,
            CodingEquations::Bt2020Ncl,
            10,
        );
        let rgba = cpl_with_rgba_descriptor(ColorPrimaries::Bt2020, TransferCharacteristic::Hlg);
        for cpl in [&cdci, &rgba] {
            for issues in [App2E2020.validate_cpl(cpl), App2E2023.validate_cpl(cpl)] {
                assert!(
                    !issues.iter().any(|i| i.code.ends_with("/ColorSystem")),
                    "COLOR.8 is permitted: {issues:#?}",
                );
            }
        }
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
        let v = App2E2023;
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

        let v = App2E2023;
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
        assert_eq!(validators[0].spec_id(), "ST 2067-2:2016");
        assert_eq!(validators[1].spec_id(), "ST 2067-21:2023 (App2E)");

        // validate_cpl should run both and merge issues
        let issues = validate_cpl(&cpl);
        let has_core = issues.iter().any(|i| i.code.starts_with("ST2067-2:2016:"));
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
        let mut cpl = with_app2e_app_id(minimal_cpl());
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
                                ccapi: None,
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

        let v = App2E2023;
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
                                ccapi: None,
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

        let v = App2E2023;
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
                                ccapi: None,
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

        let v = App2E2023;
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

        let v = App2E2023;
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

        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("7.4")),
            "Should flag segment duration not multiple of 5: {:#?}",
            issues
        );
    }

    /// Segment duration is the Sequence duration: per resource SourceDuration
    /// (default IntrinsicDuration - EntryPoint) x RepeatCount, summed
    /// (https://github.com/Netflix/photon/issues/160, /209).
    fn segment_issues_for_resource(
        intrinsic: u64,
        entry_point: Option<u64>,
        source_duration: Option<u64>,
        repeat_count: Option<u64>,
    ) -> usize {
        let mut cpl = cpl_with_audio_and_segment_duration(
            EditRate::new(48000, 1),
            EditRate::new(30000, 1001),
            &[intrinsic],
        );
        let r = &mut cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences[0]
            .resource_list
            .resources[0];
        r.entry_point = entry_point;
        r.source_duration = source_duration;
        r.repeat_count = repeat_count;
        App2E2023
            .validate_cpl(&cpl)
            .iter()
            .filter(|i| i.code.contains("SegmentDurationMultiple"))
            .count()
    }

    /// Hand check for imferno issue 88: 48 kHz at 24000/1001 is 2002
    /// samples per edit unit (an integer), so §7.4 does not apply and 3761
    /// or 2433 edit units are fine. At 30000/1001 (1601.6 samples per edit
    /// unit, 5 x 1601.6 = 8008) neither is a multiple of 5.
    #[test]
    fn app2e_segment_duration_hand_checks() {
        for (rate, durations, expected) in [
            (EditRate::new(24000, 1001), [3761, 2433], 0),
            (EditRate::new(30000, 1001), [3761, 2433], 2),
            (EditRate::new(30000, 1001), [3760, 2435], 0),
        ] {
            let cpl =
                cpl_with_audio_and_segment_duration(EditRate::new(48000, 1), rate, &durations);
            let issues: Vec<_> = App2E2023
                .validate_cpl(&cpl)
                .into_iter()
                .filter(|i| i.code.contains("SegmentDurationMultiple"))
                .collect();
            assert_eq!(
                issues.len(),
                expected,
                "{rate:?} {durations:?}: {issues:#?}"
            );
            if let Some(i) = issues.first() {
                assert!(
                    i.message.contains("1601.600 samples per edit unit"),
                    "{}",
                    i.message
                );
            }
        }
    }

    #[test]
    fn app2e_segment_duration_subtracts_entry_point() {
        // Intrinsic 12, EntryPoint 2, no SourceDuration: 10 edit units.
        assert_eq!(segment_issues_for_resource(12, Some(2), None, None), 0);
        // Intrinsic 10, EntryPoint 2: 8 edit units.
        assert_eq!(segment_issues_for_resource(10, Some(2), None, None), 1);
    }

    #[test]
    fn app2e_segment_duration_multiplies_repeat_count() {
        // SourceDuration 3 x RepeatCount 5 = 15.
        assert_eq!(segment_issues_for_resource(3, None, Some(3), Some(5)), 0);
        // SourceDuration 5 x RepeatCount 2 = 10, EntryPoint ignored when
        // SourceDuration is present.
        assert_eq!(segment_issues_for_resource(9, Some(4), Some(5), Some(2)), 0);
        // SourceDuration 4 x RepeatCount 2 = 8.
        assert_eq!(segment_issues_for_resource(4, None, Some(4), Some(2)), 1);
    }

    #[test]
    fn app2e_segment_duration_uses_one_sequence_not_the_sum() {
        // ST 2067-3: Sequences in a Segment share its duration, so the
        // Segment duration is one Sequence's, not the sum over Sequences.
        // Sequences of 6 and 4 sum to 10, but the Segment is 6 long.
        let mut cpl = cpl_with_audio_and_segment_duration(
            EditRate::new(48000, 1),
            EditRate::new(30000, 1001),
            &[6, 4],
        );
        let second = cpl.segment_list.segments.remove(1);
        let seq = second.sequence_list.main_image_sequences.into_iter().next();
        cpl.segment_list.segments[0]
            .sequence_list
            .main_image_sequences
            .extend(seq);
        let n = App2E2023
            .validate_cpl(&cpl)
            .iter()
            .filter(|i| i.code.contains("SegmentDurationMultiple"))
            .count();
        assert_eq!(n, 1);
    }

    /// §7.4: Same non-integer sample rate, but segment duration is 10 (multiple of 5) → pass.
    #[test]
    fn app2e_allows_segment_duration_multiple_of_5() {
        let cpl = cpl_with_audio_and_segment_duration(
            EditRate::new(48000, 1),    // 48kHz audio
            EditRate::new(30000, 1001), // 29.97 fps — non-integer samples/EU
            &[10],                      // 10 edit units, divisible by 5
        );
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("7.4")),
            "Should NOT flag at 23.976 fps (integer samples/EU): {:#?}",
            issues
        );
    }

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

    // ── A: SourceDuration 0 is permitted ─────────────────────────────────────

    /// ST 2067-3 §6.11.6 (2013, 2016 and 2020 alike) allows SourceDuration
    /// from 0 (zero) up to IntrinsicDuration - EntryPoint, and the XSD types
    /// it as xs:nonNegativeInteger. ST 2067-2 adds no lower bound. A
    /// zero-length Resource is therefore legal and must not be flagged.
    /// https://github.com/jpwesselink/imferno/issues/89
    #[test]
    fn core_accepts_zero_source_duration() {
        for edition in [
            &CoreConstraints2013 as &dyn ConstraintsValidator,
            &CoreConstraints2016,
            &CoreConstraints2020,
        ] {
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
                            intrinsic_duration: 24,
                            entry_point: None,
                            source_duration: Some(0),
                            source_encoding: Some(ed_id),
                            track_file_id: Some(uuid(50)),
                            repeat_count: None,
                            key_id: None,
                            hash: None,
                            markers: vec![],
                        }],
                    },
                });
            let issues = edition.validate_cpl(&cpl);
            assert!(
                !issues
                    .iter()
                    .any(|i| i.code.contains("SourceDuration")
                        || i.code.contains("ResourceDuration")),
                "{}: SourceDuration 0 is allowed by ST 2067-3 6.11.6: {:#?}",
                edition.spec_id(),
                issues,
            );
        }
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
                    languages: vec![crate::cpl::LanguageTag::new("123invalid")],
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
                        crate::cpl::LanguageTag::new("nl"),
                        crate::cpl::LanguageTag::new("en"),
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

    /// ST 2067-2:2020 §6.3.3: Timed Text resource edit rates may be any
    /// value (2013/2016 §6.3.3 set no edit-rate constraint on these tracks
    /// either), so mixed edit rates in a Subtitles / HearingImpairedCaptions /
    /// ForcedNarrative virtual track are not flagged.
    #[test]
    fn core_accepts_mixed_edit_rates_in_timed_text_virtual_track() {
        use crate::cpl::{
            ForcedNarrativeSequence, HearingImpairedCaptionsSequence, Segment, SubtitlesSequence,
        };

        let resource = |id: u8, er: u32| {
            let mut r = make_resource(None);
            r.id = uuid(id);
            r.edit_rate = Some(EditRate::new(er, 1));
            r
        };
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        let mut sl1 = empty_sequence_list();
        let mut sl2 = empty_sequence_list();
        sl1.subtitles_sequences.push(SubtitlesSequence {
            id: uuid(10),
            track_id: uuid(40),
            resource_list: ResourceList {
                resources: vec![resource(90, 24), resource(91, 1000)],
            },
        });
        sl2.subtitles_sequences.push(SubtitlesSequence {
            id: uuid(11),
            track_id: uuid(40),
            resource_list: ResourceList {
                resources: vec![resource(92, 25)],
            },
        });
        sl1.hearing_impaired_captions_sequences
            .push(HearingImpairedCaptionsSequence {
                id: uuid(12),
                track_id: uuid(41),
                resource_list: ResourceList {
                    resources: vec![resource(93, 24), resource(94, 30)],
                },
            });
        sl1.forced_narrative_sequences
            .push(ForcedNarrativeSequence {
                id: uuid(13),
                track_id: uuid(42),
                resource_list: ResourceList {
                    resources: vec![resource(95, 24), resource(96, 60)],
                },
            });
        cpl.segment_list.segments = vec![
            Segment {
                id: uuid(30),
                sequence_list: sl1,
            },
            Segment {
                id: uuid(31),
                sequence_list: sl2,
            },
        ];

        for issues in [
            CoreConstraints2013.validate_cpl(&cpl),
            CoreConstraints2016.validate_cpl(&cpl),
            CoreConstraints2020.validate_cpl(&cpl),
        ] {
            assert!(
                !issues
                    .iter()
                    .any(|i| i.code.contains("VirtualTrackEditRate")),
                "Timed text edit rates must not be flagged: {:#?}",
                issues,
            );
        }
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
                        source_duration: Some(50),
                        source_encoding: None,
                        track_file_id: None,
                        repeat_count: None,
                        key_id: None,
                        hash: None,
                        markers: vec![MarkerInfo {
                            annotation: None,
                            label: MarkerLabelElement::from(MarkerLabel::Ffoc),
                            // ST 2067-3 §6.13 bounds Offset by IntrinsicDuration
                            // (100), not by SourceDuration.
                            offset: 120,
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

    /// ST 2067-2 §5.3.6.6 (2013, 2016, 2020): not all channels of a
    /// soundfield group need be present in an Audio Track File, so fewer
    /// channels than the soundfield implies is not an error.
    #[test]
    fn audio_accepts_partial_soundfield_channel_count() {
        use crate::cpl::McaTagSymbol;
        use crate::cpl::{AudioSubDescriptors, SoundfieldGroupLabelSubDescriptor};

        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: Some(2), // subset of a 5.1 soundfield
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
            !issues
                .iter()
                .any(|i| i.code.contains("SoundfieldChannelCount")),
            "Partial soundfield (fewer channels) should not be flagged: {:#?}",
            issues,
        );
    }

    /// More channels than the soundfield group defines is still flagged.
    #[test]
    fn audio_flags_channel_count_exceeding_soundfield() {
        use crate::cpl::McaTagSymbol;
        use crate::cpl::{AudioSubDescriptors, SoundfieldGroupLabelSubDescriptor};

        let cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: None,
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            channel_count: Some(8),
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
            issues
                .iter()
                .any(|i| i.code.contains("SoundfieldChannelCount")),
            "8 channels under a 5.1 soundfield should be flagged: {:#?}",
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

    // ── Image resolution validation (Table 3) ───────────────────────────────

    /// HT-J2K (ISO/IEC 15444-15) is permitted by ST 2067-21:2023
    /// (§5.5.2 Table 6, Annex I) but not by the 2020 edition, whose §5.5.2
    /// lists only ISO/IEC 15444-1 profiles. J2K-HT-Not-Allowed must only
    /// fire under the 2020 validator.
    #[test]
    fn j2k_ht_not_allowed_only_fires_for_app2e_2020() {
        let mut cpl = cpl_with_image_resolution(1920, 1080, Some(EditRate::new(24, 1)));
        cpl.essence_descriptor_list
            .as_mut()
            .unwrap()
            .essence_descriptors[0]
            .cdci_descriptor
            .as_mut()
            .unwrap()
            .picture_compression = Some(VideoCodec::Jpeg2000Ht);
        let ht = St2067_21_2020::J2KHtNotAllowed.code();

        let issues_2023 = App2E2023.validate_cpl(&cpl);
        assert!(
            !issues_2023
                .iter()
                .any(|i| i.code.contains("J2K-HT-Not-Allowed")),
            "HT-J2K must be accepted by App2E 2023: {issues_2023:#?}",
        );

        let issues_2020 = App2E2020.validate_cpl(&cpl);
        assert!(
            issues_2020.iter().any(|i| i.code == ht),
            "HT-J2K must be rejected by App2E 2020: {issues_2020:#?}",
        );
    }

    /// ST 2067-2:2020 §6.1 Table 10: ApplicationIdentification is a list of
    /// URIs; ST 2067-21 §7.1 (2020, 2023) only requires it to include the
    /// Table 15 value. A list containing it must not raise AppIdMismatch.
    #[test]
    fn app_id_mismatch_accepts_list_including_table_15_value() {
        let with_app_id = |app_id: &str| {
            let mut cpl = minimal_cpl();
            cpl.extension_properties = Some(ExtensionProperties {
                application_identification: Some(app_id.to_string()),
                max_cll: None,
                max_fall: None,
            });
            cpl
        };
        let mismatch =
            |issues: &[ValidationIssue], code: &str| issues.iter().any(|i| i.code == code);
        let code_2023 = St2067_21_2023::AppIdMismatch.code();
        let code_2020 = crate::validation::codes::St2067_21_2020::AppIdMismatch.code();

        let both = "http://example.com/other-app \
                    http://www.smpte-ra.org/ns/2067-21/2020\n\
                    http://www.smpte-ra.org/ns/2067-21/2021";
        assert!(!mismatch(
            &App2E2023.validate_cpl(&with_app_id(both)),
            code_2023
        ));
        assert!(!mismatch(
            &App2E2020.validate_cpl(&with_app_id(both)),
            code_2020
        ));

        // A list that does not include the Table 15 value is still flagged.
        let other = "http://example.com/other-app http://www.smpte-ra.org/ns/2067-21/2020";
        assert!(mismatch(
            &App2E2023.validate_cpl(&with_app_id(other)),
            code_2023
        ));
        let other = "http://example.com/other-app http://www.smpte-ra.org/ns/2067-21/2021";
        assert!(mismatch(
            &App2E2020.validate_cpl(&with_app_id(other)),
            code_2020
        ));
    }

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

    /// A conforming 1080i CDCI descriptor: ST 2067-21 Table 9 stores each
    /// field, so StoredWidth×StoredHeight = 1920×540 for a 1920×1080 frame;
    /// Table 3 interlaced column: 25 / 30 / 30000/1001, 4:2:2 Y'C'BC'R,
    /// COLOR.1-3, 8 or 10 bit. FieldDominance and DisplayF2Offset are
    /// present as Table 8 requires for interlaced.
    fn cpl_1080i(rate: EditRate) -> CompositionPlaylist {
        let mut cpl = cpl_with_image_resolution(1920, 540, Some(rate));
        let cdci = cpl
            .essence_descriptor_list
            .as_mut()
            .unwrap()
            .essence_descriptors[0]
            .cdci_descriptor
            .as_mut()
            .unwrap();
        cdci.frame_layout = Some("SeparateFields".to_string());
        cdci.field_dominance = Some(1);
        cdci.display_f2_offset = Some(0);
        cpl
    }

    fn cdci_mut(cpl: &mut CompositionPlaylist) -> &mut CDCIDescriptor {
        cpl.essence_descriptor_list
            .as_mut()
            .unwrap()
            .essence_descriptors[0]
            .cdci_descriptor
            .as_mut()
            .unwrap()
    }

    /// ST 2067-21 §6.2.1 Table 9 (2020 p13, 2023 p15): for interlaced
    /// content StoredHeight is the frame height / 2, so the image frame is
    /// 1920×1080 and must pass the resolution rule.
    #[test]
    fn app2e_resolution_accepts_1080i_stored_per_field() {
        for rate in [
            EditRate::new(25, 1),
            EditRate::new(30, 1),
            EditRate::new(30000, 1001),
        ] {
            let issues = App2E2023.validate_cpl(&cpl_1080i(rate));
            assert!(
                !issues
                    .iter()
                    .any(|i| i.code == St2067_21_2023::Resolution.code()),
                "1080i (stored 1920x540) should pass the resolution rule: {issues:#?}",
            );
        }
    }

    #[test]
    fn app2e_accepts_1920x1080() {
        let cpl = cpl_with_image_resolution(1920, 1080, Some(EditRate::new(24, 1)));
        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("Resolution")),
            "1920x1080 should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_2048x1080() {
        let cpl = cpl_with_image_resolution(2048, 1080, Some(EditRate::new(25, 1)));
        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("Resolution")),
            "2048x1080 should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_3840x2160() {
        let cpl = cpl_with_image_resolution(3840, 2160, Some(EditRate::new(24, 1)));
        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("Resolution")),
            "3840x2160 should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_4096x2160() {
        // Table 3: sizes above 3840x2160 exist only in the R'G'B' column.
        let cpl = cpl_with_rgba_size(4096, 2160);
        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("Resolution")),
            "4096x2160 R'G'B' should be accepted: {:#?}",
            issues,
        );
    }

    /// ST 2067-21 §5.2 Table 3 (2020 p6, 2023 p7): the largest image frame
    /// is 4096x3112; 7680x4320 is outside every column.
    #[test]
    fn app2e_rejects_7680x4320() {
        let cpl = cpl_with_image_resolution(7680, 4320, Some(EditRate::new(24, 1)));
        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("Resolution")),
            "7680x4320 should be rejected: {:#?}",
            issues,
        );
    }

    /// Table 3 gives ranges ("a..b ... including a and b"), not exact sizes:
    /// 1280x720 is within 1..1920 x 1..1080.
    #[test]
    fn app2e_accepts_1280x720() {
        let cpl = cpl_with_image_resolution(1280, 720, Some(EditRate::new(24, 1)));
        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("Resolution")),
            "1280x720 should be accepted: {:#?}",
            issues,
        );
    }

    /// 1920x800 is within 1..1920 x 1..1080 (Table 3 ranges).
    #[test]
    fn app2e_accepts_1920x800() {
        let cpl = cpl_with_image_resolution(1920, 800, Some(EditRate::new(24, 1)));
        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("Resolution")),
            "1920x800 should be accepted: {:#?}",
            issues,
        );
    }

    /// Table 3 range edges: 4096x3112 is the largest frame; one more pixel
    /// in either direction is outside every column. Same in 2020 and 2023.
    #[test]
    fn app2e_resolution_uses_table_3_range_edges() {
        let flagged = |cpl: CompositionPlaylist| {
            let a = App2E2023
                .validate_cpl(&cpl)
                .iter()
                .any(|i| i.code == St2067_21_2023::Resolution.code());
            let b = App2E2020
                .validate_cpl(&cpl)
                .iter()
                .any(|i| i.code == St2067_21_2020::Resolution.code());
            assert_eq!(a, b, "2020 and 2023 Table 3 are identical");
            a
        };
        let cdci = |w, h| cpl_with_image_resolution(w, h, Some(EditRate::new(24, 1)));

        // R'G'B' (RGBA): up to 1..4096 x 1..3112 (Table 3 R'G'B' column).
        assert!(!flagged(cpl_with_rgba_size(4096, 3112)));
        assert!(!flagged(cpl_with_rgba_size(3996, 2160)));
        assert!(flagged(cpl_with_rgba_size(4097, 2160)));
        assert!(flagged(cpl_with_rgba_size(4096, 3113)));

        // Y'C'BC'R (CDCI): every column allowing it stops at 3840x2160.
        assert!(!flagged(cdci(1, 1)));
        assert!(!flagged(cdci(3840, 2160)));
        assert!(flagged(cdci(3841, 2160)));
        assert!(flagged(cdci(3840, 2161)));
        assert!(flagged(cdci(3996, 2160)));
        assert!(flagged(cdci(4096, 2160)));
        assert!(flagged(cdci(4096, 3112)));
    }

    /// RGBA (R'G'B') COLOR.3 descriptor with the given stored size.
    fn cpl_with_rgba_size(width: u32, height: u32) -> CompositionPlaylist {
        let mut cpl =
            cpl_with_rgba_descriptor(ColorPrimaries::Bt709, TransferCharacteristic::Bt709);
        let rgba = cpl
            .essence_descriptor_list
            .as_mut()
            .unwrap()
            .essence_descriptors[0]
            .rgba_descriptor
            .as_mut()
            .unwrap();
        rgba.stored_width = Some(width);
        rgba.stored_height = Some(height);
        rgba.display_width = Some(width);
        rgba.display_height = Some(height);
        cpl
    }

    // ── Image frame rate validation (Table 3) ───────────────────────────────

    #[test]
    fn app2e_accepts_24fps() {
        let cpl = cpl_with_image_resolution(1920, 1080, Some(EditRate::new(24, 1)));
        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("FrameRate")),
            "24 fps should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_23976fps() {
        let cpl = cpl_with_image_resolution(1920, 1080, Some(EditRate::new(24000, 1001)));
        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("FrameRate")),
            "23.976 fps should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_25fps() {
        let cpl = cpl_with_image_resolution(1920, 1080, Some(EditRate::new(25, 1)));
        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("FrameRate")),
            "25 fps should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_60fps() {
        let cpl = cpl_with_image_resolution(3840, 2160, Some(EditRate::new(60, 1)));
        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("FrameRate")),
            "60 fps should be accepted: {:#?}",
            issues,
        );
    }

    #[test]
    fn app2e_accepts_5994fps() {
        let cpl = cpl_with_image_resolution(3840, 2160, Some(EditRate::new(60000, 1001)));
        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("FrameRate")),
            "59.94 fps should be accepted: {:#?}",
            issues,
        );
    }

    /// Table 3: 120 fps is only in the R'G'B' column, so a Y'C'BC'R (CDCI)
    /// descriptor at 120 fps is rejected.
    #[test]
    fn app2e_rejects_120fps() {
        let cpl = cpl_with_image_resolution(1920, 1080, Some(EditRate::new(120, 1)));
        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("FrameRate")),
            "120 fps should be rejected: {:#?}",
            issues,
        );
    }

    /// ST 2067-21 §5.2 Table 3 (2020 p6, 2023 p7): no column lists 48 or
    /// 48000/1001.
    #[test]
    fn app2e_rejects_48fps() {
        for rate in [EditRate::new(48, 1), EditRate::new(48000, 1001)] {
            let cpl = cpl_with_image_resolution(1920, 1080, Some(rate));
            for (issues, code) in [
                (
                    App2E2023.validate_cpl(&cpl),
                    St2067_21_2023::FrameRate.code(),
                ),
                (
                    App2E2020.validate_cpl(&cpl),
                    St2067_21_2020::FrameRate.code(),
                ),
            ] {
                assert!(
                    issues.iter().any(|i| i.code == code),
                    "{rate:?} should be rejected: {issues:#?}",
                );
            }
        }
    }

    /// Table 3: 120 fps is listed only in the 1..4096 x 1..3112 R'G'B'
    /// 4:4:4 column, so it is accepted on an RGBA descriptor.
    #[test]
    fn app2e_accepts_120fps_rgb() {
        let mut cpl =
            cpl_with_rgba_descriptor(ColorPrimaries::Bt709, TransferCharacteristic::Bt709);
        cpl.essence_descriptor_list
            .as_mut()
            .unwrap()
            .essence_descriptors[0]
            .rgba_descriptor
            .as_mut()
            .unwrap()
            .sample_rate = Some(EditRate::new(120, 1));
        for issues in [App2E2023.validate_cpl(&cpl), App2E2020.validate_cpl(&cpl)] {
            assert!(
                !issues
                    .iter()
                    .any(|i| i.code == St2067_21_2023::FrameRate.code()),
                "120 fps R'G'B' should be accepted: {issues:#?}",
            );
        }
    }

    #[test]
    fn app2e_rejects_15fps() {
        let cpl = cpl_with_image_resolution(1920, 1080, Some(EditRate::new(15, 1)));
        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("FrameRate")),
            "15 fps should be rejected: {:#?}",
            issues,
        );
    }

    // ── Audio sample rate validation (§6.5) ────────────────────────────────

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

        let issues = App2E2023.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code == St2067_21_2023::RequiredSampleRate.code()),
            "Missing SampleRate should be flagged: {:#?}",
            issues,
        );
    }

    // ── XSD structural validation tests ─────────────────────────────────────
    // Helper-direct tests (xs_datetime_valid/invalid_formats, timecode_address_*,
    // total_running_time_*) were deleted along with the helper functions
    // they exercised — these were pure XSD-overlap checks now slated for
    // runtime-XSD validation.

    /// XSD §35: EditRate is required on the composition (no minOccurs="0").
    ///
    /// Uses the 2013 namespace because the vendored 2020 XSD's
    /// `targetNamespace` is still `schemas/2067-3/2016` (SMPTE kept the
    /// 2016 URI for the 2020 edition); the parser-internal `ns/2067-3/2020`
    /// URI doesn't match any vendored XSD. 2013 is the most self-consistent
    /// edition for XSD-pre-pass tests.
    #[test]
    fn core_flags_missing_edit_rate() {
        let xml = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
            <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
            <IssueDate>2024-01-01T00:00:00Z</IssueDate>
            <ContentTitle>Test</ContentTitle>
            <SegmentList><Segment>
                <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
                <SequenceList/>
            </Segment></SegmentList>
        </CompositionPlaylist>"#;
        let cpl = crate::cpl::parse_cpl(xml).expect("should parse despite missing EditRate");
        let issues = CoreConstraints2013.validate_cpl(&cpl);
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

    /// XSD §13: IssueDate is `xs:dateTime` — "not-a-date" fails the
    /// built-in lexical-space check and surfaces as `XSD/TypeInvalid`.
    #[test]
    fn core_warns_invalid_issue_date_format() {
        let xml = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
            <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
            <IssueDate>not-a-date</IssueDate>
            <ContentTitle>Test</ContentTitle>
            <EditRate>24 1</EditRate>
            <SegmentList><Segment>
                <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
                <SequenceList/>
            </Segment></SegmentList>
        </CompositionPlaylist>"#;
        let cpl = crate::cpl::parse_cpl(xml).expect("should parse despite invalid IssueDate");
        let issues = CoreConstraints2013.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("IssueDate")),
            "Invalid IssueDate format should be flagged: {:#?}",
            issues,
        );
    }

    /// XSD §13: Empty IssueDate also fails the `xs:dateTime` lexical check.
    #[test]
    fn core_flags_empty_issue_date() {
        let xml = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
            <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
            <IssueDate></IssueDate>
            <ContentTitle>Test</ContentTitle>
            <EditRate>24 1</EditRate>
            <SegmentList><Segment>
                <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
                <SequenceList/>
            </Segment></SegmentList>
        </CompositionPlaylist>"#;
        let cpl = crate::cpl::parse_cpl(xml).expect("should parse despite empty IssueDate");
        let issues = CoreConstraints2013.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("IssueDate")),
            "Empty IssueDate should be flagged: {:#?}",
            issues,
        );
    }

    /// XSD §68-74: `CompositionTimecodeType` requires
    /// `TimecodeDropFrame`, `TimecodeRate`, and `TimecodeStartAddress`
    /// (none with `minOccurs="0"`). Missing the drop-frame and
    /// start-address fields should trip element-missing diagnostics
    /// for each.
    #[test]
    fn core_flags_incomplete_composition_timecode() {
        let xml = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
            <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
            <IssueDate>2024-01-01T00:00:00Z</IssueDate>
            <ContentTitle>Test</ContentTitle>
            <CompositionTimecode>
                <TimecodeRate>24</TimecodeRate>
            </CompositionTimecode>
            <EditRate>24 1</EditRate>
            <SegmentList><Segment>
                <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
                <SequenceList/>
            </Segment></SegmentList>
        </CompositionPlaylist>"#;
        let cpl = crate::cpl::parse_cpl(xml)
            .expect("should parse despite incomplete CompositionTimecode");
        let issues = CoreConstraints2013.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("TimecodeDropFrame")
                    || i.code.contains("TimecodeStartAddress")),
            "Missing TimecodeDropFrame/StartAddress should be flagged: {:#?}",
            issues,
        );
    }

    /// XSD §164-170: `ResourceList` declares
    /// `Resource maxOccurs=unbounded` with default `minOccurs=1`, so
    /// an empty ResourceList inside a `MarkerSequence` (the only
    /// sequence type the CPL XSD knows directly) trips
    /// `ElementMissing/Resource`.
    #[test]
    fn core_flags_empty_resource_list() {
        let xml = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
            <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
            <IssueDate>2024-01-01T00:00:00Z</IssueDate>
            <ContentTitle>Test</ContentTitle>
            <EditRate>24 1</EditRate>
            <SegmentList><Segment>
                <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
                <SequenceList>
                    <MarkerSequence>
                        <Id>urn:uuid:00000000-0000-0000-0000-000000000003</Id>
                        <TrackId>urn:uuid:00000000-0000-0000-0000-000000000004</TrackId>
                        <ResourceList/>
                    </MarkerSequence>
                </SequenceList>
            </Segment></SegmentList>
        </CompositionPlaylist>"#;
        let cpl = crate::cpl::parse_cpl(xml).expect("should parse despite empty ResourceList");
        let issues = CoreConstraints2013.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("Resource")),
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
                    language: Some(LanguageTag::new("en")),
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
        let v = App2E2023;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues
                .iter()
                .any(|i| i.code == St2067_21_2023::FrameLayoutInterlaced.code()),
            "FullFrame FrameLayout should not be flagged: {:#?}",
            issues,
        );
    }

    /// App2E: a conforming 1080i image is accepted. ST 2067-21 §5.2.2,
    /// §5.2 Table 3 interlaced column (2020 p6, 2023 p7), §6.2.1 Table 9 and
    /// the Frame Layout clause (01h if interlaced).
    #[test]
    fn app2e_accepts_interlaced_frame_layout() {
        let v = App2E2023;
        for rate in [
            EditRate::new(25, 1),
            EditRate::new(30, 1),
            EditRate::new(30000, 1001),
        ] {
            let issues = v.validate_cpl(&with_app2e_app_id(cpl_1080i(rate)));
            let errors: Vec<_> = issues
                .iter()
                .filter(|i| matches!(i.severity, Severity::Error | Severity::Critical))
                .collect();
            assert!(
                errors.is_empty(),
                "conforming 1080i at {rate:?} should have no errors: {errors:#?}",
            );
        }
        let mut cpl = cpl_1080i(EditRate::new(25, 1));

        // A layout outside {FullFrame, SeparateFields} is still rejected by
        // the §6.2.1.3 FrameLayout rule.
        cpl.essence_descriptor_list
            .as_mut()
            .unwrap()
            .essence_descriptors[0]
            .cdci_descriptor
            .as_mut()
            .unwrap()
            .frame_layout = Some("MixedFields".to_string());
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code == St2067_21_2023::FrameLayout.code()),
            "MixedFields FrameLayout should be flagged for App2E: {:#?}",
            issues,
        );
    }

    /// ST 2067-21 §5.2 Table 3 (2020 p6, 2023 p7): the only interlaced
    /// column is <=1920x1080, COLOR.1/2/3, 8 or 10 bit, 25 / 30 / 30000/1001,
    /// 4:2:2 Y'C'BC'R. Interlaced content outside it fits no column; the
    /// differing characteristic is reported against the interlaced column
    /// (closest, frame structure preferred on ties) under its own code.
    #[test]
    fn app2e_rejects_interlaced_outside_table_3_column() {
        let flagged_with = |cpl: &CompositionPlaylist, suffix: &str| {
            App2E2023
                .validate_cpl(cpl)
                .iter()
                .any(|i| i.code.starts_with("ST2067-21:2023:") && i.code.ends_with(suffix))
                && App2E2020
                    .validate_cpl(cpl)
                    .iter()
                    .any(|i| i.code.starts_with("ST2067-21:2020:") && i.code.ends_with(suffix))
        };
        let table_3_errors = |cpl: &CompositionPlaylist| {
            App2E2023
                .validate_cpl(cpl)
                .into_iter()
                .filter(|i| i.message.contains("Table 3"))
                .collect::<Vec<_>>()
        };
        let base = || cpl_1080i(EditRate::new(25, 1));
        assert!(
            table_3_errors(&base()).is_empty(),
            "conforming 1080i must not be flagged"
        );

        type Mutation = Box<dyn Fn(&mut CDCIDescriptor)>;
        let cases: Vec<(&str, &str, Mutation)> = vec![
            (
                "24 fps",
                "/FrameRate",
                Box::new(|c| c.sample_rate = Some(EditRate::new(24, 1))),
            ),
            (
                "50 fps",
                "/FrameRate",
                Box::new(|c| c.sample_rate = Some(EditRate::new(50, 1))),
            ),
            (
                "frame height 2160 (stored 1080 per field)",
                "/Resolution",
                Box::new(|c| c.stored_height = Some(1080)),
            ),
            (
                "width 3840",
                "/Resolution",
                Box::new(|c| c.stored_width = Some(3840)),
            ),
            (
                "12-bit",
                "/ComponentDepth",
                Box::new(|c| c.component_depth = Some(12)),
            ),
            (
                "4:4:4",
                "/HorizontalSubsampling",
                Box::new(|c| c.horizontal_subsampling = Some(1)),
            ),
            (
                "COLOR.5",
                "/ColorSystem",
                Box::new(|c| {
                    c.color_primaries = Some(ColorPrimaries::Bt2020);
                    c.transfer_characteristic = Some(TransferCharacteristic::Bt2020);
                    c.coding_equations = Some(CodingEquations::Bt2020Ncl);
                }),
            ),
        ];
        for (name, suffix, mutate) in cases {
            let mut cpl = base();
            mutate(cdci_mut(&mut cpl));
            assert!(
                flagged_with(&cpl, suffix),
                "interlaced {name} must be flagged with {suffix}: {:#?}",
                table_3_errors(&cpl)
            );
        }

        // R'G'B' (RGBA descriptor) is never permitted with interlaced: the
        // closest column is the 1..1920 progressive 4:4:4 one.
        let mut cpl =
            cpl_with_rgba_descriptor(ColorPrimaries::Bt709, TransferCharacteristic::Bt709);
        let rgba = cpl
            .essence_descriptor_list
            .as_mut()
            .unwrap()
            .essence_descriptors[0]
            .rgba_descriptor
            .as_mut()
            .unwrap();
        rgba.frame_layout = Some("SeparateFields".to_string());
        rgba.stored_height = Some(540);
        rgba.sample_rate = Some(EditRate::new(25, 1));
        rgba.field_dominance = Some(1);
        assert!(
            flagged_with(&cpl, "/FrameLayoutInterlaced"),
            "RGBA + SeparateFields must be flagged: {:#?}",
            table_3_errors(&cpl)
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
            !issues.iter().any(|i| i.code.contains("Rate-Mismatch")),
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
            issues.iter().any(|i| i.code.contains("Rate-Mismatch")),
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
            !issues.iter().any(|i| i.code.contains("Rate-Mismatch")),
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
        let v = App2E2023;
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
        let v = App2E2023;
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
                    rfc5646_language_tag_list: vec![LanguageTag::new("en")],
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
        let v = App2E2023;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("5.6-HIC")),
            "HIC with timed text should not be flagged: {:#?}",
            issues,
        );
    }

    /// The HearingImpairedCaptions / ForcedNarrative timed-text requirement
    /// is ST 2067-2 §6.3.3 (2020 p20), not an ST 2067-21 provision: no
    /// edition of ST 2067-21 (2014–2026) mentions these sequences. The App
    /// #2E validator therefore does not report it.
    #[test]
    fn app2e_does_not_report_hic_timed_text_rule_from_st_2067_2() {
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
        let v = App2E2023;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("TimedText")),
            "not an ST 2067-21 rule: {:#?}",
            issues,
        );
    }

    /// See `app2e_does_not_report_hic_timed_text_rule_from_st_2067_2`.
    #[test]
    fn app2e_does_not_report_fn_timed_text_rule_from_st_2067_2() {
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
        let v = App2E2023;
        let issues = v.validate_cpl(&cpl);
        assert!(
            !issues.iter().any(|i| i.code.contains("TimedText")),
            "not an ST 2067-21 rule: {:#?}",
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
        let v = App2E2023;
        let issues = v.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("7.1/ApplicationIdentification")),
            "Missing ApplicationIdentification should be flagged: {:#?}",
            issues,
        );
    }

    /// ST 2067-2:2020 §6.1 (p18): ExtensionProperties "shall include a
    /// single instance" of ApplicationIdentification, and ST 2067-21 §7.1
    /// requires it to include the Table 15 value. A CPL with no
    /// ExtensionProperties at all therefore lacks it too.
    #[test]
    fn app2e_flags_missing_extension_properties() {
        let mut cpl = minimal_cpl();
        cpl.edit_rate = Some(EditRate::new(24, 1));
        cpl.extension_properties = None;
        for (issues, code) in [
            (
                App2E2023.validate_cpl(&cpl),
                St2067_21_2023::ApplicationIdentification.code(),
            ),
            (
                App2E2020.validate_cpl(&cpl),
                St2067_21_2020::ApplicationIdentification.code(),
            ),
        ] {
            assert!(
                issues.iter().any(|i| i.code == code),
                "Missing ExtensionProperties/ApplicationIdentification should be flagged: {issues:#?}",
            );
        }
    }

    // ── #47: LocaleList / RFC 5646 / ISO 3166-1 validation ──────────────

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
                    rfc5646_language_tag_list: vec![LanguageTag::new("en")],
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
                    rfc5646_language_tag_list: vec![LanguageTag::new("en")],
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
                    rfc5646_language_tag_list: vec![LanguageTag::new("")],
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

    /// CPL with 2016+ namespace should emit digital signature info notice.
    #[test]
    fn core_emits_digital_signature_notice_for_modern_cpl() {
        let cpl = minimal_cpl(); // Uses Smpte2067_3_2016
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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
        let v = App2E2023;
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

        let app2e_issues = App2E2023.validate_cpl(&cpl);
        assert!(
            !app2e_issues.iter().any(|i| i.code.contains(":General/")),
            "App2E validator emitted :General fallback codes: {:#?}",
            app2e_issues,
        );
    }

    // ── XSD constraint helpers ────────────────────────────────────────────────
    // helper_timecode_address_* and helper_total_running_time_* were deleted
    // along with the helper functions they exercised (gutted as part of the
    // runtime-XSD migration).

    // ── XSD: TotalRunningTime pattern ────────────────────────────────────────

    /// XSD §37-41: `TotalRunningTime` carries pattern
    /// `[0-9][0-9]:[0-5][0-9]:[0-5][0-9]` — "2:30:00" fails it (missing
    /// leading zero on the hours field).
    #[test]
    fn core_flags_invalid_total_running_time() {
        let xml = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
            <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
            <IssueDate>2024-01-01T00:00:00Z</IssueDate>
            <ContentTitle>Test</ContentTitle>
            <EditRate>24 1</EditRate>
            <TotalRunningTime>2:30:00</TotalRunningTime>
            <SegmentList><Segment>
                <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
                <SequenceList/>
            </Segment></SegmentList>
        </CompositionPlaylist>"#;
        let cpl =
            crate::cpl::parse_cpl(xml).expect("should parse despite invalid TotalRunningTime");
        let issues = CoreConstraints2013.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("TotalRunningTime")),
            "Invalid TotalRunningTime format should be flagged: {:#?}",
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

    /// XSD §71: `TimecodeRate` is `xs:positiveInteger` — zero fails the
    /// built-in derived-type lower bound (`positive` = ≥ 1).
    #[test]
    fn core_flags_timecode_rate_zero() {
        let xml = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
            <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
            <IssueDate>2024-01-01T00:00:00Z</IssueDate>
            <ContentTitle>Test</ContentTitle>
            <CompositionTimecode>
                <TimecodeDropFrame>false</TimecodeDropFrame>
                <TimecodeRate>0</TimecodeRate>
                <TimecodeStartAddress>00:00:00:00</TimecodeStartAddress>
            </CompositionTimecode>
            <EditRate>24 1</EditRate>
            <SegmentList><Segment>
                <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
                <SequenceList/>
            </Segment></SegmentList>
        </CompositionPlaylist>"#;
        let cpl = crate::cpl::parse_cpl(xml).expect("should parse despite zero TimecodeRate");
        let issues = CoreConstraints2013.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("TimecodeRate")),
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
                .any(|i| i.code.contains("CompositionTimecode/Rate-Zero")),
            "Positive TimecodeRate should be accepted: {:#?}",
            issues,
        );
    }

    // ── XSD: TimecodeStartAddress format ─────────────────────────────────────

    /// XSD §72/75-81: `TimecodeStartAddress` is `cpl:TimecodeType` —
    /// a `xs:string` restricted to a four-field timecode pattern.
    /// "10:00:00" lacks the fourth field and fails the pattern facet.
    #[test]
    fn core_flags_invalid_timecode_start_address() {
        let xml = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
            <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
            <IssueDate>2024-01-01T00:00:00Z</IssueDate>
            <ContentTitle>Test</ContentTitle>
            <CompositionTimecode>
                <TimecodeDropFrame>false</TimecodeDropFrame>
                <TimecodeRate>24</TimecodeRate>
                <TimecodeStartAddress>10:00:00</TimecodeStartAddress>
            </CompositionTimecode>
            <EditRate>24 1</EditRate>
            <SegmentList><Segment>
                <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
                <SequenceList/>
            </Segment></SegmentList>
        </CompositionPlaylist>"#;
        let cpl =
            crate::cpl::parse_cpl(xml).expect("should parse despite invalid TimecodeStartAddress");
        let issues = CoreConstraints2013.validate_cpl(&cpl);
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("TimecodeStartAddress")),
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
                .any(|i| i.code.contains("CompositionTimecode/StartAddress-Format")),
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

    /// XSD §43-49: `LocaleList` requires `Locale maxOccurs=unbounded`
    /// with default `minOccurs=1`, so an empty LocaleList trips
    /// `ElementMissing/Locale` at the schema level.
    #[test]
    fn core_flags_empty_locale_list() {
        let xml = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
            <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
            <IssueDate>2024-01-01T00:00:00Z</IssueDate>
            <ContentTitle>Test</ContentTitle>
            <EditRate>24 1</EditRate>
            <LocaleList/>
            <SegmentList><Segment>
                <Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id>
                <SequenceList/>
            </Segment></SegmentList>
        </CompositionPlaylist>"#;
        let cpl = crate::cpl::parse_cpl(xml).expect("should parse despite empty LocaleList");
        let issues = CoreConstraints2013.validate_cpl(&cpl);
        assert!(
            issues.iter().any(|i| i.code.contains("Locale")),
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

    // ── Per-edition App #2E: image-characteristics columns ───────────────
    //
    // Fixture values below are taken from the spec tables, not from the
    // validator's constants: ST 2067-21:2020 Table 3 (p6) + Am1:2020 Table 3
    // (p4), ST 2067-21:2023 Table 3 (p7), ST 2067-21:2026 Table 2 (§6.2.1).

    const C1: (ColorPrimaries, TransferCharacteristic, CodingEquations) = (
        ColorPrimaries::Bt601_625,
        TransferCharacteristic::Bt709,
        CodingEquations::Bt601,
    );
    const C2: (ColorPrimaries, TransferCharacteristic, CodingEquations) = (
        ColorPrimaries::Bt601_525,
        TransferCharacteristic::Bt709,
        CodingEquations::Bt601,
    );
    const C3: (ColorPrimaries, TransferCharacteristic, CodingEquations) = (
        ColorPrimaries::Bt709,
        TransferCharacteristic::Bt709,
        CodingEquations::Bt709,
    );
    const C4: (ColorPrimaries, TransferCharacteristic, CodingEquations) = (
        ColorPrimaries::Bt709,
        TransferCharacteristic::XvYcc709,
        CodingEquations::Bt709,
    );
    const C5: (ColorPrimaries, TransferCharacteristic, CodingEquations) = (
        ColorPrimaries::Bt2020,
        TransferCharacteristic::Bt2020,
        CodingEquations::Bt2020Ncl,
    );
    const C7: (ColorPrimaries, TransferCharacteristic, CodingEquations) = (
        ColorPrimaries::Bt2020,
        TransferCharacteristic::PqSt2084,
        CodingEquations::Bt2020Ncl,
    );
    const C8: (ColorPrimaries, TransferCharacteristic, CodingEquations) = (
        ColorPrimaries::Bt2020,
        TransferCharacteristic::Hlg,
        CodingEquations::Bt2020Ncl,
    );

    /// A CDCI descriptor. `height` is the image frame height; for
    /// interlaced the stored height is one field (Table 9).
    #[allow(clippy::too_many_arguments)]
    fn cdci_frame(
        width: u32,
        height: u32,
        interlaced: bool,
        rate: (u32, u32),
        color: (ColorPrimaries, TransferCharacteristic, CodingEquations),
        depth: u32,
        h_sub: u32,
    ) -> CompositionPlaylist {
        let stored_h = if interlaced { height / 2 } else { height };
        let mut cpl =
            cpl_with_image_resolution(width, stored_h, Some(EditRate::new(rate.0, rate.1)));
        let c = cdci_mut(&mut cpl);
        c.frame_layout = Some(
            if interlaced {
                "SeparateFields"
            } else {
                "FullFrame"
            }
            .to_string(),
        );
        if interlaced {
            c.field_dominance = Some(1);
        }
        c.color_primaries = Some(color.0);
        c.transfer_characteristic = Some(color.1);
        c.coding_equations = Some(color.2);
        c.component_depth = Some(depth);
        c.horizontal_subsampling = Some(h_sub);
        c.picture_compression = Some(VideoCodec::Jpeg2000);
        cpl
    }

    /// An RGBA descriptor; `refs` is (ComponentMinRef, ComponentMaxRef).
    fn rgba_frame(
        width: u32,
        height: u32,
        rate: (u32, u32),
        color: (ColorPrimaries, TransferCharacteristic),
        refs: (u32, u32),
    ) -> CompositionPlaylist {
        let mut cpl = cpl_with_rgba_descriptor(color.0, color.1);
        let r = cpl
            .essence_descriptor_list
            .as_mut()
            .unwrap()
            .essence_descriptors[0]
            .rgba_descriptor
            .as_mut()
            .unwrap();
        r.stored_width = Some(width);
        r.stored_height = Some(height);
        r.sample_rate = Some(EditRate::new(rate.0, rate.1));
        r.component_min_ref = Some(refs.0);
        r.component_max_ref = Some(refs.1);
        r.picture_compression = Some(VideoCodec::Jpeg2000);
        cpl
    }

    fn codes_of(issues: Vec<ValidationIssue>) -> Vec<String> {
        issues.into_iter().map(|i| i.code).collect()
    }

    /// Codes of the image-characteristics findings (all columns differ).
    fn column_findings(v: &dyn ConstraintsValidator, cpl: &CompositionPlaylist) -> Vec<String> {
        v.validate_cpl(cpl)
            .into_iter()
            .filter(|i| i.message.contains("fit no column"))
            .map(|i| i.code)
            .collect()
    }

    /// One frame per column of the 2023 Table 3 (and of the 2020 table with
    /// Am1): each fits its column, under both editions.
    #[test]
    fn app2e_table_3_every_column_accepts_a_conforming_frame() {
        let frames = [
            // 1..1920 x 1..1080, interlaced, COLOR.1/2/3, 8/10, 25/30/30000/1001, 4:2:2
            cdci_frame(1920, 1080, true, (30000, 1001), C2, 8, 2),
            cdci_frame(720, 576, true, (25, 1), C1, 10, 2),
            // 1..1920 x 1..1080, progressive, COLOR.1/2/3, 8/10, 24..60, 4:4:4 Y'C'BC'R
            cdci_frame(1920, 1080, false, (50, 1), C1, 10, 1),
            // ... or R'G'B' (QE.1)
            rgba_frame(1920, 1080, (60000, 1001), (C3.0, C3.1), (16, 235)),
            // 1..3840 x 1..2160 4:2:2: COLOR.4 8/10, COLOR.3 8..16, COLOR.5/8 10/12, COLOR.7 10/12/16
            cdci_frame(3840, 2160, false, (24, 1), C4, 10, 2),
            cdci_frame(3840, 2160, false, (60, 1), C3, 16, 2),
            cdci_frame(3840, 2160, false, (25, 1), C5, 12, 2),
            cdci_frame(3840, 2160, false, (24000, 1001), C8, 10, 2),
            cdci_frame(3840, 2160, false, (30, 1), C7, 16, 2),
            // 1..4096 x 1..3112 R'G'B' 4:4:4: COLOR.3 8..16, COLOR.5/8 10/12,
            // COLOR.6/7 10/12/16, 24..60 and 120, QE.1/QE.2
            rgba_frame(4096, 3112, (120, 1), (C3.0, C3.1), (0, 255)),
            rgba_frame(4096, 2160, (24, 1), (C5.0, C5.1), (0, 4095)),
            rgba_frame(
                4096,
                2160,
                (24, 1),
                (ColorPrimaries::Bt2020, TransferCharacteristic::Hlg),
                (64, 940),
            ),
            rgba_frame(
                3996,
                2160,
                (48, 2),
                (ColorPrimaries::P3D65, TransferCharacteristic::PqSt2084),
                (0, 65535),
            ),
            rgba_frame(4096, 1716, (120, 1), (C7.0, C7.1), (256, 3760)),
        ];
        for cpl in &frames {
            assert_eq!(column_findings(&App2E2023, cpl), Vec::<String>::new());
            assert_eq!(column_findings(&App2E2020, cpl), Vec::<String>::new());
        }
    }

    /// Characteristics each allowed somewhere in Table 3 but not together in
    /// one column are rejected, with the differing characteristic's code.
    #[test]
    fn app2e_table_3_rejects_combinations_that_fit_no_single_column() {
        let cases = [
            // COLOR.4 is 8/10 bit only.
            (
                cdci_frame(3840, 2160, false, (24, 1), C4, 12, 2),
                "6.4/ComponentDepth",
            ),
            // COLOR.5 has no 8-bit column.
            (
                cdci_frame(3840, 2160, false, (24, 1), C5, 8, 2),
                "6.4/ComponentDepth",
            ),
            // Progressive 4:2:2 with COLOR.1 (only COLOR.3 has a 4:2:2
            // progressive column among COLOR.1/2/3).
            (
                cdci_frame(1920, 1080, false, (25, 1), C1, 10, 2),
                "6.4/HorizontalSubsampling",
            ),
            // 120 fps exists only for R'G'B'.
            (
                cdci_frame(3840, 2160, false, (120, 1), C3, 10, 2),
                "5.2/FrameRate",
            ),
            // R'G'B' COLOR.1 is QE.1 only (1..1920 column).
            (
                rgba_frame(1920, 1080, (24, 1), (C1.0, C1.1), (0, 1023)),
                "6.3.2/ComponentRefValues",
            ),
            // R'G'B' COLOR.4: no R'G'B' column lists COLOR.4.
            (
                rgba_frame(1920, 1080, (24, 1), (C4.0, C4.1), (64, 940)),
                "6.2/ColorSystem",
            ),
            // Y'C'BC'R 4:4:4 exists only for COLOR.1/2/3 (1..1920 column).
            (
                cdci_frame(1920, 1080, false, (24, 1), C5, 10, 1),
                "6.4/HorizontalSubsampling",
            ),
        ];
        for (cpl, suffix) in &cases {
            for (v, label) in [
                (&App2E2023 as &dyn ConstraintsValidator, "ST2067-21:2023:"),
                (&App2E2020, "ST2067-21:2020:"),
            ] {
                let found = column_findings(v, cpl);
                assert!(
                    found.iter().any(|c| *c == format!("{label}{suffix}")),
                    "expected {label}{suffix}, got {found:?}",
                );
            }
        }
    }

    /// ST 2067-21:2026 Table 2 (§6.2.1): 4:2:2 Y'C'BC'R COLOR.3/5/7/8 up to
    /// 1..7680 x 1..4320, R'G'B' up to 1..8192 x 1..6224, COLOR.4 still
    /// 1..3840 x 1..2160, and a 1..1920 progressive 4:2:2 COLOR.1/2/3 column.
    /// None of these fit the 2023 table.
    #[test]
    fn app2e_2026_table_2_columns_differ_from_2023() {
        let only_2026 = [
            cdci_frame(7680, 4320, false, (24, 1), C7, 12, 2),
            cdci_frame(7680, 4320, false, (60, 1), C3, 8, 2),
            rgba_frame(
                8192,
                6224,
                (120, 1),
                (ColorPrimaries::P3D65, TransferCharacteristic::PqSt2084),
                (0, 65535),
            ),
            cdci_frame(1920, 1080, false, (25, 1), C1, 10, 2),
        ];
        for cpl in &only_2026 {
            assert_eq!(column_findings(&App2E2026, cpl), Vec::<String>::new());
            assert!(!column_findings(&App2E2023, cpl).is_empty());
        }
        let c4_8k = cdci_frame(7680, 4320, false, (24, 1), C4, 10, 2);
        assert!(column_findings(&App2E2026, &c4_8k)
            .contains(&"ST2067-21:2026:6.2.1/Resolution".to_string()));
        let too_big = cdci_frame(7681, 4320, false, (24, 1), C3, 10, 2);
        assert!(column_findings(&App2E2026, &too_big)
            .contains(&"ST2067-21:2026:6.2.1/Resolution".to_string()));
    }

    /// 2026 Table 7 (§7.2.1.1): Alpha Transparency "Shall be ignored";
    /// 2020/2023 Table 8: "Shall not be present".
    #[test]
    fn app2e_alpha_transparency_is_ignored_in_2026_only() {
        let mut cpl = cdci_frame(1920, 1080, false, (24, 1), C3, 10, 2);
        cdci_mut(&mut cpl).alpha_transparency = Some("0".to_string());
        let has = |issues: Vec<ValidationIssue>| {
            issues
                .iter()
                .any(|i| i.code.ends_with("/AlphaTransparency"))
        };
        assert!(has(App2E2023.validate_cpl(&cpl)));
        assert!(has(App2E2020.validate_cpl(&cpl)));
        assert!(!has(App2E2026.validate_cpl(&cpl)));
    }

    /// Every code a validator emits carries its own edition's label; rules
    /// that are not in an edition's text are not reported under it.
    #[test]
    fn app2e_validators_emit_only_their_own_edition_codes() {
        let mut broken = cdci_frame(9000, 9000, true, (48, 1), C5, 9, 3);
        {
            let c = cdci_mut(&mut broken);
            c.stored_f2_offset = Some(1);
            c.alpha_transparency = Some("1".to_string());
            c.black_ref_level = Some(1);
            c.picture_compression = Some(VideoCodec::Jpeg2000Ht);
        }
        let mut broken_rgba = rgba_frame(9000, 9000, (48, 1), (C4.0, C4.1), (1, 2));
        broken_rgba
            .essence_descriptor_list
            .as_mut()
            .unwrap()
            .essence_descriptors[0]
            .rgba_descriptor
            .as_mut()
            .unwrap()
            .palette = Some("x".to_string());
        broken.locale_list = Some(LocaleList {
            locales: vec![Locale {
                language_list: Some(LanguageList {
                    languages: vec![LanguageTag::new("")],
                }),
                content_maturity_rating_list: None,
                region_list: Some(RegionList {
                    regions: vec!["usa".to_string()],
                }),
            }],
        });
        for (v, label) in [
            (&App2E2014 as &dyn ConstraintsValidator, "ST2067-21:2014:"),
            (&App2E2016, "ST2067-21:2016:"),
            (&App2E2020, "ST2067-21:2020:"),
            (&App2E2023, "ST2067-21:2023:"),
            (&App2E2026, "ST2067-21:2026:"),
        ] {
            for cpl in [&broken, &broken_rgba, &minimal_cpl()] {
                let issues = v.validate_cpl(cpl);
                assert!(!issues.is_empty());
                for i in &issues {
                    assert!(
                        i.code.starts_with(label),
                        "{} emitted {}",
                        v.spec_id(),
                        i.code
                    );
                }
            }
        }
    }

    // ── ST 2067-21:2014 / 2016 ───────────────────────────────────────────

    fn with_app_id_uri(mut cpl: CompositionPlaylist, uri: &str) -> CompositionPlaylist {
        cpl.extension_properties = Some(ExtensionProperties {
            application_identification: Some(uri.to_string()),
            max_cll: None,
            max_fall: None,
        });
        cpl
    }

    /// ST 2067-21:2014 §6.1.2 (p6): Coding Equations shall be the BT.709 UL
    /// (…02.02.00.00) if COLOR.4 or COLOR.5 is used; §6.1.4 Table 4: COLOR.5
    /// at 10 bit is 64 / 940 / 897.
    #[test]
    fn app2e_2014_color5_uses_bt709_coding_equations() {
        let uri = "http://www.smpte-ra.org/schemas/2067-21/2014";
        let bt709_ce = (
            ColorPrimaries::Bt2020,
            TransferCharacteristic::Bt2020,
            CodingEquations::Bt709,
        );
        let ok = with_app_id_uri(
            cdci_frame(3840, 2160, false, (24, 1), bt709_ce.clone(), 10, 2),
            uri,
        );
        assert_eq!(codes_of(App2E2014.validate_cpl(&ok)), Vec::<String>::new());

        // The BT.2020 NCL coding equations of later editions are not the
        // 2014 COLOR.5 value.
        let ncl = with_app_id_uri(cdci_frame(3840, 2160, false, (24, 1), C5, 10, 2), uri);
        let issues = App2E2014.validate_cpl(&ncl);
        assert_eq!(
            issues.iter().map(|i| i.code.as_str()).collect::<Vec<_>>(),
            vec!["ST2067-21:2014:6.1/ColorSystem"]
        );

        let mut wrong_range = with_app_id_uri(
            cdci_frame(3840, 2160, false, (24, 1), bt709_ce.clone(), 10, 2),
            uri,
        );
        cdci_mut(&mut wrong_range).color_range = Some(1013);
        assert_eq!(
            App2E2014
                .validate_cpl(&wrong_range)
                .iter()
                .map(|i| i.code.as_str())
                .collect::<Vec<_>>(),
            vec!["ST2067-21:2014:6.1.4/ColorRange"]
        );
    }

    /// The 2014 edition only constrains COLOR.4/5 descriptors; COLOR.3 and
    /// descriptor items governed by ST 2067-20 are not checked under it.
    #[test]
    fn app2e_2014_does_not_apply_later_edition_rules() {
        let uri = "http://www.smpte-ra.org/schemas/2067-21/2014";
        let mut cpl = with_app_id_uri(cdci_frame(4096, 3112, true, (48, 1), C3, 12, 1), uri);
        cdci_mut(&mut cpl).stored_f2_offset = Some(3);
        cdci_mut(&mut cpl).picture_compression = Some(VideoCodec::Mpeg2);
        assert_eq!(codes_of(App2E2014.validate_cpl(&cpl)), Vec::<String>::new());
    }

    /// ST 2067-21:2016 §6.1.1–6.1.3 (p8–9): COLOR.5 and COLOR.7 use the
    /// BT.2020 NCL coding equations; COLOR.6 uses P3D65 primaries with the
    /// ST 2084 transfer characteristic. §6.3 Table 7 (p10): COLOR.5/7 at
    /// 12 bit are 256 / 3760 / 3585.
    #[test]
    fn app2e_2016_colorimetry_items() {
        let uri = "http://www.smpte-ra.org/schemas/2067-21/2016";
        let color7_12_bit = || {
            let mut c7 = with_app_id_uri(cdci_frame(3840, 2160, false, (24, 1), C7, 12, 2), uri);
            {
                let c = cdci_mut(&mut c7);
                c.black_ref_level = Some(256);
                c.white_ref_level = Some(3760);
                c.color_range = Some(3585);
            }
            c7.extension_properties.as_mut().unwrap().max_cll = Some(1000);
            c7.extension_properties.as_mut().unwrap().max_fall = Some(400);
            c7
        };
        assert_eq!(
            codes_of(App2E2016.validate_cpl(&color7_12_bit())),
            Vec::<String>::new()
        );

        let bt709_ce = (
            ColorPrimaries::Bt2020,
            TransferCharacteristic::PqSt2084,
            CodingEquations::Bt709,
        );
        let wrong = with_app_id_uri(cdci_frame(3840, 2160, false, (24, 1), bt709_ce, 12, 2), uri);
        assert!(App2E2016
            .validate_cpl(&wrong)
            .iter()
            .any(|i| i.code == "ST2067-21:2016:6.1/ColorSystem"));

        let mut wrong_white = color7_12_bit();
        cdci_mut(&mut wrong_white).white_ref_level = Some(940);
        assert!(App2E2016
            .validate_cpl(&wrong_white)
            .iter()
            .any(|i| i.code == "ST2067-21:2016:6.3/WhiteRefLevel"));

        let c6 = with_app_id_uri(
            rgba_frame(
                4096,
                2160,
                (24, 1),
                (ColorPrimaries::P3D65, TransferCharacteristic::PqSt2084),
                (256, 3760),
            ),
            uri,
        );
        let issues = App2E2016.validate_cpl(&c6);
        assert!(
            !issues.iter().any(|i| i.code.ends_with("/ColorSystem")),
            "{issues:#?}"
        );
        // §7.2: MaxCLL / MaxFALL are 0..1 for COLOR.6/7 — informational.
        assert!(issues
            .iter()
            .any(|i| i.code == "ST2067-21:2016:7.2/MaxCLLMaxFALL" && i.severity == Severity::Info));
    }

    /// ST 2067-21:2016 §6.2 Table 6 (p10): 12-bit R'G'B' is 256/3760 (QE.1)
    /// or 0/4095 (QE.2).
    #[test]
    fn app2e_2016_component_ref_values_12_bit() {
        let uri = "http://www.smpte-ra.org/schemas/2067-21/2016";
        let c = (ColorPrimaries::P3D65, TransferCharacteristic::PqSt2084);
        for refs in [(256, 3760), (0, 4095)] {
            let cpl = with_app_id_uri(rgba_frame(4096, 2160, (24, 1), c.clone(), refs), uri);
            assert!(!App2E2016
                .validate_cpl(&cpl)
                .iter()
                .any(|i| i.code.ends_with("/ComponentRefValues")));
        }
        let cpl = with_app_id_uri(rgba_frame(4096, 2160, (24, 1), c, (0, 3760)), uri);
        assert!(App2E2016
            .validate_cpl(&cpl)
            .iter()
            .any(|i| i.code == "ST2067-21:2016:6.2/ComponentRefValues"));
    }

    /// Pinning an edition against a CPL that declares another edition's
    /// value reports that edition's own clause and value.
    #[test]
    fn app2e_app_id_mismatch_names_the_validated_edition() {
        let cpl = with_app_id_uri(
            cdci_frame(1920, 1080, false, (24, 1), C3, 10, 2),
            "http://www.smpte-ra.org/ns/2067-21/2021",
        );
        let issues = App2E2026.validate_cpl(&cpl);
        let mismatch = issues
            .iter()
            .find(|i| i.code == "ST2067-21:2026:8.1/AppIdMismatch")
            .expect("2026 AppIdMismatch");
        assert!(mismatch
            .message
            .contains("http://www.smpte-ra.org/ns/2067-21/5ED"));
        assert!(App2E2014
            .validate_cpl(&cpl)
            .iter()
            .any(|i| i.code == "ST2067-21:2014:7/AppIdMismatch"));
    }

    /// ST 2067-21:2026 §6.5.2 Table 5 adds a 4097..8192 x 1..6224 column
    /// with the 8K IMF Single/Multi Tile Reversible (Sublevel 0 only) and
    /// 8K IMF Single Tile Lossy (no Sublevel 0) profiles, Mainlevels 1–10.
    /// ULs from the vendored SMPTE Labels register (Labels.xml):
    /// `…04010202.03010403` "8K IMF Single Tile Lossy Profile (Mainlevel 1
    /// Sublevel 1)", `…04010202.03010702` "8K IMF Single/Multi-Tile
    /// Reversible Profile (Mainlevel 1 Sublevel 0)". The 2023 Table 6 has no
    /// 8K column.
    #[test]
    fn app2e_8k_imf_profiles_are_2026_only() {
        for ul in [
            "urn:smpte:ul:060e2b34.0401010d.04010202.03010403",
            "urn:smpte:ul:060e2b34.0401010d.04010202.03010729",
            "urn:smpte:ul:060e2b34.0401010d.04010202.03010702",
        ] {
            let mut cpl = cdci_frame(7680, 4320, false, (24, 1), C3, 10, 2);
            cdci_mut(&mut cpl).picture_compression = Some(VideoCodec::from_ul(ul));
            let j2k = |issues: Vec<ValidationIssue>| {
                issues
                    .into_iter()
                    .filter(|i| i.code.contains("/J2K"))
                    .map(|i| i.code)
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                j2k(App2E2026.validate_cpl(&cpl)),
                Vec::<String>::new(),
                "{ul} at 7680x4320 is a 2026 Table 5 profile"
            );
            assert_eq!(
                j2k(App2E2023.validate_cpl(&cpl)),
                vec!["ST2067-21:2023:6.2.5/J2K-Profile-Not-Allowed".to_string()],
                "{ul}: no 8K profile in 2023 Table 6"
            );

            // 8K profiles cover 4097..8192 x 1..6224 only.
            let mut small = cdci_frame(3840, 2160, false, (24, 1), C3, 10, 2);
            cdci_mut(&mut small).picture_compression = Some(VideoCodec::from_ul(ul));
            assert_eq!(
                j2k(App2E2026.validate_cpl(&small)),
                vec!["ST2067-21:2026:7.2.5/J2K-8K-Resolution".to_string()],
            );
        }
    }

    /// Language tags, region codes, ContentMaturityRating and audio sample
    /// rate / quantization are not provisions of ST 2067-21:2023: §5.3 is
    /// Colorimetry (p9), §6.5 the JPEG 2000 Picture Sub Descriptor (p19) and
    /// §7.1 ApplicationIdentification (p20). The App #2E validator does not
    /// report them under any edition.
    #[test]
    fn app2e_does_not_report_rules_from_other_documents() {
        let mut cpl = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: Some(EditRate::new(44100, 1)),
            quantization_bits: None,
            channel_count: None,
            audio_sample_rate: Some(EditRate::new(44100, 1)),
            linked_track_id: None,
            sub_descriptors: None,
        });
        let mut wave16 = cpl_with_audio(WAVEPCMDescriptor {
            instance_id: None,
            sample_rate: Some(EditRate::new(48000, 1)),
            quantization_bits: Some(16),
            channel_count: Some(2),
            audio_sample_rate: Some(EditRate::new(48000, 1)),
            linked_track_id: None,
            sub_descriptors: None,
        });
        for c in [&mut cpl, &mut wave16] {
            c.locale_list = Some(LocaleList {
                locales: vec![Locale {
                    language_list: Some(LanguageList {
                        languages: vec![LanguageTag::new(""), LanguageTag::new("1en")],
                    }),
                    content_maturity_rating_list: Some(ContentMaturityRatingList {
                        ratings: vec![
                            ContentMaturityRating {
                                agency: String::new(),
                                rating: Some("R".to_string()),
                                audience: None,
                            },
                            ContentMaturityRating {
                                agency: "not a uri".to_string(),
                                rating: Some("R".to_string()),
                                audience: None,
                            },
                        ],
                    }),
                    region_list: Some(RegionList {
                        regions: vec!["usa".to_string()],
                    }),
                }],
            });
        }
        let gone = [
            "/EmptyLanguageTag",
            "/MalformedLanguageTag",
            "/RegionCode",
            "/AudioSampleRate",
            "/QuantizationBits",
            "/Required-ChannelCount",
            "/Required-QuantizationBits",
            "/ContentMaturityRating-Agency",
            "/ContentMaturityRating-Agency-URI",
        ];
        for v in [
            &App2E2014 as &dyn ConstraintsValidator,
            &App2E2016,
            &App2E2020,
            &App2E2023,
            &App2E2026,
        ] {
            for c in [&cpl, &wave16] {
                for i in v.validate_cpl(c) {
                    assert!(
                        !gone.iter().any(|g| i.code.ends_with(g)),
                        "{} emitted {}",
                        v.spec_id(),
                        i.code
                    );
                }
            }
        }
        for g in gone {
            assert!(
                !St2067_21_2023::ALL.iter().any(|c| c.code().ends_with(g)),
                "{g} still in the 2023 code set"
            );
        }
    }
}
