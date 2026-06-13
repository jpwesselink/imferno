//! Typed validation-code catalogue for SMPTE ST 429-9 (VOLINDEX).

use crate::diagnostics::codes::ValidationCode;
use crate::diagnostics::{Category, Severity};

/// Validation codes defined by SMPTE ST 429-9:2014 (Volume Index & AssetMap).
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum St429_9_2014 {
    /// No volume-index document found in the package root.
    VolindexMissing,
    /// The VOLINDEX.xml document is not well-formed XML (ST 429-9:2014 §7).
    MalformedXml,
}

impl ValidationCode for St429_9_2014 {
    fn code(&self) -> &'static str {
        match self {
            Self::VolindexMissing => "ST429-9:2014:7/VolindexMissing",
            Self::MalformedXml => "ST429-9:2014:7/MalformedXml",
        }
    }
    fn description(&self) -> &'static str {
        match self {
            Self::VolindexMissing => "No volume-index document found in the package root.",
            Self::MalformedXml => "The VOLINDEX.xml document is not well-formed XML.",
        }
    }
    fn default_severity(&self) -> Severity {
        match self {
            Self::VolindexMissing => Severity::Info,
            Self::MalformedXml => Severity::Error,
        }
    }
    fn category(&self) -> Category {
        Category::Structure
    }
    fn example(&self) -> Option<&'static str> {
        Some(match self {
            Self::VolindexMissing => {
                "Package root contains ASSETMAP.xml and CPL/PKL XMLs but no VOLINDEX.xml."
            }
            Self::MalformedXml => {
                "<VolumeIndex>1<!-- truncated --> — VOLINDEX.xml ends mid-element."
            }
        })
    }
}

impl St429_9_2014 {
    pub const ALL: &'static [Self] = &[Self::VolindexMissing, Self::MalformedXml];
}

impl From<St429_9_2014> for String {
    fn from(c: St429_9_2014) -> String {
        c.code().to_string()
    }
}
