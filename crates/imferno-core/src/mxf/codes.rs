//! Typed validation-code catalogue for SMPTE ST 377-1 (MXF).

use crate::diagnostics::codes::ValidationCode;
use crate::diagnostics::{Category, Severity};

/// Validation codes defined by SMPTE ST 377-1:2011 (MXF File Format).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum St377_1_2011 {
    /// File is not a valid MXF container.
    NotMxf,
    /// MXF file could not be parsed; may be truncated or corrupt.
    ParseError,
    /// MXF file contains no essence containers.
    NoEssenceContainers,
    /// Operational pattern is not OP1a as required by IMF.
    Op1a,
}

impl ValidationCode for St377_1_2011 {
    fn code(&self) -> &'static str {
        match self {
            Self::NotMxf => "ST377-1:2011:5/NotMxf",
            Self::ParseError => "ST377-1:2011:5/ParseError",
            Self::NoEssenceContainers => "ST377-1:2011:11/NoEssenceContainers",
            Self::Op1a => "ST377-1:2011:7/OP1a",
        }
    }
    fn description(&self) -> &'static str {
        match self {
            Self::NotMxf => "File is not a valid MXF container.",
            Self::ParseError => "MXF file could not be parsed; it may be truncated or corrupt.",
            Self::NoEssenceContainers => "MXF file contains no essence containers.",
            Self::Op1a => "MXF operational pattern must be OP1a for IMF packages.",
        }
    }
    fn default_severity(&self) -> Severity {
        match self {
            Self::NotMxf | Self::ParseError | Self::NoEssenceContainers => Severity::Warning,
            Self::Op1a => Severity::Error,
        }
    }
    fn category(&self) -> Category {
        match self {
            Self::NotMxf | Self::ParseError => Category::Asset,
            Self::NoEssenceContainers | Self::Op1a => Category::Encoding,
        }
    }
}

impl St377_1_2011 {
    pub const ALL: &'static [Self] = &[
        Self::NotMxf,
        Self::ParseError,
        Self::NoEssenceContainers,
        Self::Op1a,
    ];
}

impl From<St377_1_2011> for String {
    fn from(c: St377_1_2011) -> String {
        c.code().to_string()
    }
}
