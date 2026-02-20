//! imferno-core — SMPTE ST 2067 IMF parser and validator.

pub mod diagnostics;
pub mod assetmap;
pub mod mxf;
pub mod cpl;
pub mod scm;
pub mod validation;
pub mod package;

// Re-export the most-used diagnostic types at crate root so that
// `crate::Severity`, `crate::ValidationReport`, etc. resolve correctly
// (required by diagnostics/rules.rs which uses `use crate::{Severity, ValidationReport}`).
pub use diagnostics::{
    Category,
    CriticalError,
    Location,
    ParseResult,
    Severity,
    ValidationIssue,
    ValidationProfile,
    ValidationReport,
};
