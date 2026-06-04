//! Catalogue codes emitted by the runtime XSD validator.
//!
//! Five buckets cover the universe of XSD constraint violations. The
//! translator (`super::translate`) classifies each uppsala diagnostic
//! into one of these by message-pattern match; operators see stable
//! catalogue codes and can override severity per code.

use crate::diagnostics::codes::ValidationCode;
use crate::diagnostics::{Category, Severity};

/// Catalogue codes emitted by `xsd::validate_against_schema`.
///
/// Five broad buckets covering the universe of XSD constraint
/// classes. Per-element refinement (e.g. mapping to a specific
/// `CoreConstraintsCode::IssueDate` when the schema violation is on
/// `<IssueDate>`) is a follow-up; the first cut keeps the translator
/// simple and stable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum XsdConstraintCode {
    /// A required element is absent (XSD `minOccurs >= 1` violated).
    ElementMissing,
    /// An element appears outside its declared `xs:sequence` /
    /// `xs:choice` content model.
    UnexpectedElement,
    /// A value violates a `xs:pattern` facet (regex restriction).
    PatternInvalid,
    /// A value violates its declared type (built-in or restriction).
    TypeInvalid,
    /// Any other XSD constraint failure; fallback bucket so
    /// unrecognized uppsala messages still surface as a catalogue code
    /// rather than being silently dropped.
    SchemaConstraintFailed,
}

impl ValidationCode for XsdConstraintCode {
    fn code(&self) -> &'static str {
        match self {
            Self::ElementMissing => "XSD/ElementMissing",
            Self::UnexpectedElement => "XSD/UnexpectedElement",
            Self::PatternInvalid => "XSD/PatternInvalid",
            Self::TypeInvalid => "XSD/TypeInvalid",
            Self::SchemaConstraintFailed => "XSD/SchemaConstraintFailed",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::ElementMissing =>
                "A required element is absent (XSD minOccurs violated).",
            Self::UnexpectedElement =>
                "An element appears outside its declared content model.",
            Self::PatternInvalid =>
                "A value violates an xs:pattern facet on its type.",
            Self::TypeInvalid =>
                "A value violates its declared XSD type (built-in or restriction).",
            Self::SchemaConstraintFailed =>
                "An XSD constraint failed; see message for details.",
        }
    }

    fn default_severity(&self) -> Severity {
        // Every XSD violation is at least an error per the schema. Operators
        // can downgrade individual codes via per-code severity overrides.
        Severity::Error
    }

    fn category(&self) -> Category {
        Category::Schema
    }

    fn example(&self) -> Option<&'static str> {
        Some(match self {
            Self::ElementMissing =>
                "<CompositionPlaylist>…</CompositionPlaylist>  <!-- missing required <EditRate> -->",
            Self::UnexpectedElement =>
                "<CompositionPlaylist><BogusTag/>…</CompositionPlaylist>  <!-- not in the schema sequence -->",
            Self::PatternInvalid =>
                "<Id>not-a-uuid</Id>  <!-- dcml:UUIDType pattern expects 'urn:uuid:...' -->",
            Self::TypeInvalid =>
                "<Size>not-a-number</Size>  <!-- declared xs:positiveInteger -->",
            Self::SchemaConstraintFailed =>
                "<EditRate>24 0</EditRate>  <!-- denominator violates a derived restriction -->",
        })
    }
}

impl XsdConstraintCode {
    pub const ALL: &'static [Self] = &[
        Self::ElementMissing,
        Self::UnexpectedElement,
        Self::PatternInvalid,
        Self::TypeInvalid,
        Self::SchemaConstraintFailed,
    ];
}

impl From<XsdConstraintCode> for String {
    fn from(c: XsdConstraintCode) -> String {
        c.code().to_string()
    }
}
