//! imferno-core — SMPTE ST 2067 IMF parser and validator.

pub mod assetmap;
pub mod cpl;
pub mod diagnostics;
pub mod mxf;
pub mod package;
pub mod scm;
pub mod validation;

// Re-export the most-used diagnostic types at crate root so that
// `crate::Severity`, `crate::ValidationReport`, etc. resolve correctly
// (required by diagnostics/rules.rs which uses `use crate::{Severity, ValidationReport}`).
pub use diagnostics::{
    Category, CriticalError, Location, ParseResult, Severity, ValidationIssue, ValidationProfile,
    ValidationReport,
};
