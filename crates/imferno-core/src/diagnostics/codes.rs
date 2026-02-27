//! Shared validation-code infrastructure.
//!
//! This module defines the [`ValidationCode`] trait that every per-spec enum implements.
//!
//! Codes live in the crate that owns their spec:
//! - [`st429_9::codes`]   — SMPTE ST 429-9 (VOLINDEX / AssetMap)
//! - [`st2067_2::codes`]  — SMPTE ST 2067-2 (PKL / package-level)
//! - [`st2067_3::codes`]  — SMPTE ST 2067-3 (Composition Playlist)
//! - [`st2067_21::codes`] — SMPTE ST 2067-21 (Application Profile #2E)
//! - [`imf_parser::codes`]  — re-exports of the above plus ST 377-1

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
}
