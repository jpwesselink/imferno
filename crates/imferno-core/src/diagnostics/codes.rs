//! Shared validation-code infrastructure.
//!
//! This module defines the [`ValidationCode`] trait that every per-spec enum implements.
//!
//! Codes live in their spec's module:
//! - [`crate::assetmap::volindex_codes`] — SMPTE ST 429-9 (VOLINDEX)
//! - [`crate::assetmap::codes`]          — SMPTE ST 2067-2 (AssetMap / PKL)
//! - [`crate::cpl::codes`]               — SMPTE ST 2067-3 (CPL)
//! - [`crate::validation::codes`]        — SMPTE ST 2067-21 (App2E)
//! - [`crate::mxf::codes`]               — SMPTE ST 377-1 (MXF)

use super::{Category, Severity};

// ─────────────────────────────────────────────────────────────────────────────
// ValidationCode trait
// ─────────────────────────────────────────────────────────────────────────────

/// Metadata for a typed validation code.
///
/// Implement this trait on a per-spec enum to get a typed, iterable catalogue
/// of all codes a spec emits, each with a canonical code string, description,
/// default severity, and category.
pub trait ValidationCode {
    /// The canonical string written into [`crate::ValidationIssue::code`].
    fn code(&self) -> &'static str;
    /// A concise English description of what the constraint checks.
    fn description(&self) -> &'static str;
    /// The severity level assigned by the validator by default.
    fn default_severity(&self) -> Severity;
    /// The category bucket this code belongs to.
    fn category(&self) -> Category;

    /// One-line snippet illustrating what a violation looks like in
    /// the source artefact. Defaults to `None`; per-code
    /// implementations override.
    ///
    /// Authoring guidance: prefer a minimal XML / value fragment over
    /// prose, so operators can scan and recognize the pattern.
    fn example(&self) -> Option<&'static str> {
        None
    }

    /// Concrete fix instruction. Defaults to `None`; per-code
    /// implementations override.
    ///
    /// Authoring guidance: imperative voice, single action when
    /// possible ("Add a `CompositionTimecode/TimecodeStartAddress`
    /// element matching format `HH:MM:SS:FF`"), spec section reference
    /// in parentheses for the operator to look up.
    fn remediation(&self) -> Option<&'static str> {
        None
    }
}
