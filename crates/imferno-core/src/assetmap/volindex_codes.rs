//! Typed validation-code catalogue for SMPTE ST 429-9 (VOLINDEX and the
//! Annex A Basic Map Profile v2 path rules).

use crate::diagnostics::codes::ValidationCode;
use crate::diagnostics::{Category, Severity};

/// Validation codes defined by SMPTE ST 429-9:2014 (Volume Index & AssetMap).
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum St429_9_2014 {
    /// No volume-index document found in the package root.
    VolindexMissing,
    /// The VOLINDEX.xml document is not well-formed XML (ST 429-9:2014 §7).
    MalformedXml,
    /// Annex A.2 (Basic Map Profile v2): Path is not a relative-path
    /// reference without query or fragment.
    PathNotRelative,
    /// Annex A.2: a path segment has a character outside a-z, A-Z, 0-9,
    /// "-", "_" and ".".
    PathCharacters,
    /// Annex A.2: the Path exceeds 100 characters, or a segment does.
    PathTooLong,
    /// Annex A.2: the Path has more than 10 segments.
    PathTooManySegments,
    /// Annex A.2: two Paths in the Asset Map are identical regardless of case.
    PathDuplicate,
}

impl ValidationCode for St429_9_2014 {
    fn code(&self) -> &'static str {
        match self {
            Self::VolindexMissing => "ST429-9:2014:7/VolindexMissing",
            Self::MalformedXml => "ST429-9:2014:7/MalformedXml",
            Self::PathNotRelative => "ST429-9:2014:A.2/PathNotRelative",
            Self::PathCharacters => "ST429-9:2014:A.2/PathCharacters",
            Self::PathTooLong => "ST429-9:2014:A.2/PathTooLong",
            Self::PathTooManySegments => "ST429-9:2014:A.2/PathTooManySegments",
            Self::PathDuplicate => "ST429-9:2014:A.2/PathDuplicate",
        }
    }
    fn description(&self) -> &'static str {
        match self {
            Self::VolindexMissing => "No volume-index document found in the package root.",
            Self::MalformedXml => "The VOLINDEX.xml document is not well-formed XML.",
            // Annex A (Basic Map Profile v2) is the profile for packages on
            // a filesystem; the HTTP profile of ST 2067-2 Annex L does not
            // apply there (https://github.com/Netflix/photon/issues/192).
            Self::PathNotRelative => "Asset Map Path is not a relative-path reference without query or fragment (Basic Map Profile v2).",
            Self::PathCharacters => "Asset Map Path segment uses characters other than a-z, A-Z, 0-9, dash, underscore and period (Basic Map Profile v2).",
            Self::PathTooLong => "Asset Map Path or one of its segments is longer than 100 characters (Basic Map Profile v2).",
            Self::PathTooManySegments => "Asset Map Path has more than 10 segments (Basic Map Profile v2).",
            Self::PathDuplicate => "Two Asset Map Paths are identical regardless of case (Basic Map Profile v2).",
        }
    }
    fn default_severity(&self) -> Severity {
        match self {
            Self::VolindexMissing => Severity::Info,
            Self::MalformedXml
            | Self::PathNotRelative
            | Self::PathCharacters
            | Self::PathTooLong
            | Self::PathTooManySegments
            | Self::PathDuplicate => Severity::Error,
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
            Self::PathNotRelative => "<Path>/mnt/imp/video.mxf</Path>",
            Self::PathCharacters => "<Path>my video.mxf</Path>  <!-- space -->",
            Self::PathTooLong => "<Path>…111-character file name….mxf</Path>",
            Self::PathTooManySegments => "<Path>a/b/c/d/e/f/g/h/i/j/k.mxf</Path>",
            Self::PathDuplicate => "<Path>Video.mxf</Path> and <Path>video.mxf</Path>",
        })
    }
}

impl St429_9_2014 {
    pub const ALL: &'static [Self] = &[
        Self::VolindexMissing,
        Self::MalformedXml,
        Self::PathNotRelative,
        Self::PathCharacters,
        Self::PathTooLong,
        Self::PathTooManySegments,
        Self::PathDuplicate,
    ];
}

impl From<St429_9_2014> for String {
    fn from(c: St429_9_2014) -> String {
        c.code().to_string()
    }
}
