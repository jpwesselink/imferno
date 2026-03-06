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

/// Barrel re-export of every validation-code enum.
///
/// ```rust
/// use imferno_core::codes::*;
/// ```
pub mod codes {
    pub use crate::assetmap::codes::{
        St2067_2_2013_Core, St2067_2_2016_Core, St2067_2_2020, St2067_2_2020_Core, St429_9_2014,
    };
    pub use crate::cpl::codes::{St2067_3_2013, St2067_3_2016, St2067_3_2020};
    pub use crate::diagnostics::codes::ValidationCode;
    pub use crate::mxf::codes::St377_1_2011;
    pub use crate::package::codes::ImfernoCode;
    pub use crate::scm::codes::St2067_9_2018;
    pub use crate::validation::codes::{St2067_21_2020, St2067_21_2023, St2067_21_2025};
    pub use crate::validation::iab_codes::{St2067_201_2019, St2067_201_2021};
    pub use crate::validation::isxd_codes::St2067_202_2022;
}
