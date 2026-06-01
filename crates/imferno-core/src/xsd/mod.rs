//! Runtime XSD validation via uppsala — pure-Rust XSD 1.1 validator.
//!
//! This module wraps uppsala's structural diagnostics into the imferno
//! catalogue surface. Activated by the `xsd-runtime` feature.
//!
//! ## What it covers
//!
//! Every constraint expressible in XSD 1.0/1.1: element presence,
//! cardinality, ordering (`xs:sequence`), type validation (built-in
//! types + restrictions), enumeration facets, pattern facets,
//! unexpected-element detection. This is the "schema" half of the
//! spec/XSD split documented in `specs/comparisons/*.md`.
//!
//! ## What it doesn't cover
//!
//! Anything the XSD grammar can't express: value-set membership
//! against external sources (BCP-47, UL registries), cross-field
//! invariants, conditional cardinality, cross-document refs,
//! computed values. Those stay hand-rolled in `validation/mod.rs`
//! and the per-spec catalogue files.
//!
//! ## Diagnostic mapping
//!
//! uppsala returns `Vec<ValidationError>` with `{ message, line, column }`.
//! `translate()` classifies each error into one of 5 catalogue codes
//! (`XSD/PatternInvalid`, `XSD/ElementMissing`, `XSD/TypeInvalid`,
//! `XSD/UnexpectedElement`, `XSD/SchemaConstraintFailed`) and wraps
//! it as a `ValidationIssue` carrying the original uppsala message
//! as the diagnostic body.
//!
//! ## Schema composition limitation
//!
//! SMPTE XSDs use `<xs:import namespace="..."/>` with no
//! `schemaLocation` attribute. uppsala (like every other XSD validator)
//! skips namespace-only imports — lax-validating anything typed
//! against the unresolved namespace. For full SMPTE coverage, callers
//! need to either compose schemas inline (see
//! `validate_against_composite_schema`) or accept the lax-validation
//! gap for elements whose types come from the unresolved import.

use crate::diagnostics::codes::ValidationCode;
use crate::diagnostics::{Category, Location, Severity, ValidationIssue};

pub mod codes;

use codes::XsdConstraintCode;

/// Validate an XML instance against an XSD schema.
///
/// Returns a `Vec<ValidationIssue>` — one per uppsala diagnostic, each
/// wrapped in an `XSD/...` catalogue code chosen by classifying the
/// uppsala message.
///
/// If either the schema or the instance fails to parse as XML, a single
/// `IMFERNO:Package/ParseError`-style issue is returned describing the
/// failure (parse failures are not the validator's job to report, but
/// we surface them so callers don't have to handle every error type
/// separately).
///
/// `cpl_id` is optional — when provided, every diagnostic gets a
/// `Location` carrying it so downstream reports can group by CPL.
pub fn validate_against_schema(
    instance_xml: &str,
    schema_xml: &str,
    cpl_id: Option<crate::assetmap::ImfUuid>,
) -> Vec<ValidationIssue> {
    let schema_doc = match uppsala::parse(schema_xml) {
        Ok(d) => d,
        Err(e) => {
            return vec![parse_failure_issue("xsd-schema", e, cpl_id)];
        }
    };
    let validator = match uppsala::XsdValidator::from_schema(&schema_doc) {
        Ok(v) => v,
        Err(e) => {
            return vec![schema_build_failure_issue(e, cpl_id)];
        }
    };
    let instance_doc = match uppsala::parse(instance_xml) {
        Ok(d) => d,
        Err(e) => {
            return vec![parse_failure_issue("xml-instance", e, cpl_id)];
        }
    };

    validator
        .validate(&instance_doc)
        .into_iter()
        .map(|err| translate(err, cpl_id))
        .collect()
}

/// Map a single uppsala diagnostic to a catalogue `ValidationIssue`.
///
/// Classification is by substring match on uppsala's message text — see
/// the comparison rows in `xsd_validate_spike_uppsala.rs` for the
/// canonical message shapes. Unrecognized messages fall through to
/// `SchemaConstraintFailed` so nothing is silently dropped.
pub fn translate(
    err: uppsala::ValidationError,
    cpl_id: Option<crate::assetmap::ImfUuid>,
) -> ValidationIssue {
    let kind = classify(&err.message);
    let mut loc = Location::new();
    if let Some(id) = cpl_id {
        loc = loc.with_cpl(id);
    }
    // Until `Location` grows line/column fields, we fold the position
    // into the human-readable message body so the information isn't lost.
    let message = match (err.line, err.column) {
        (Some(line), Some(col)) => format!("{} (at line {line}, column {col})", err.message),
        (Some(line), None) => format!("{} (at line {line})", err.message),
        _ => err.message,
    };
    ValidationIssue::new(kind.default_severity(), kind.category(), kind.code(), message)
        .with_location(loc)
}

fn classify(message: &str) -> XsdConstraintCode {
    // Patterns are observed from uppsala v0.4.0 — see the spike's
    // boundary probe + synthetic-broken-CPL output for the message
    // shapes these match. New uppsala versions may add new shapes;
    // anything unrecognized falls through to SchemaConstraintFailed.
    if message.contains("Expected at least") && message.contains("occurrence") {
        XsdConstraintCode::ElementMissing
    } else if message.contains("Unexpected element") {
        XsdConstraintCode::UnexpectedElement
    } else if message.contains("not match pattern") || message.contains("does not match") {
        XsdConstraintCode::PatternInvalid
    } else if message.contains("is not a valid") {
        XsdConstraintCode::TypeInvalid
    } else {
        XsdConstraintCode::SchemaConstraintFailed
    }
}

fn parse_failure_issue(
    role: &'static str,
    err: impl std::fmt::Debug,
    cpl_id: Option<crate::assetmap::ImfUuid>,
) -> ValidationIssue {
    let mut loc = Location::new();
    if let Some(id) = cpl_id {
        loc = loc.with_cpl(id);
    }
    ValidationIssue::new(
        Severity::Critical,
        Category::Schema,
        XsdConstraintCode::SchemaConstraintFailed.code(),
        format!("XSD validation aborted: failed to parse {role}: {err:?}"),
    )
    .with_location(loc)
}

fn schema_build_failure_issue(
    err: impl std::fmt::Debug,
    cpl_id: Option<crate::assetmap::ImfUuid>,
) -> ValidationIssue {
    let mut loc = Location::new();
    if let Some(id) = cpl_id {
        loc = loc.with_cpl(id);
    }
    ValidationIssue::new(
        Severity::Critical,
        Category::Schema,
        XsdConstraintCode::SchemaConstraintFailed.code(),
        format!("XSD validation aborted: schema parsed but XsdValidator construction failed: {err:?}"),
    )
    .with_location(loc)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINI_XSD: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
      <xs:element name="thing">
        <xs:complexType>
          <xs:sequence>
            <xs:element name="name" type="xs:string"/>
            <xs:element name="count" type="xs:positiveInteger"/>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
    </xs:schema>"#;

    #[test]
    fn valid_doc_yields_no_issues() {
        let xml = "<thing><name>x</name><count>5</count></thing>";
        let issues = validate_against_schema(xml, MINI_XSD, None);
        assert!(issues.is_empty(), "expected no issues, got: {issues:#?}");
    }

    #[test]
    fn missing_required_classifies_as_element_missing() {
        let xml = "<thing><name>x</name></thing>";
        let issues = validate_against_schema(xml, MINI_XSD, None);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.code.contains("ElementMissing")),
            "expected XSD/ElementMissing: {issues:#?}");
    }

    #[test]
    fn unknown_element_classifies_as_unexpected_element() {
        let xml = "<thing><name>x</name><count>5</count><unknown/></thing>";
        let issues = validate_against_schema(xml, MINI_XSD, None);
        assert!(issues.iter().any(|i| i.code.contains("UnexpectedElement")),
            "expected XSD/UnexpectedElement: {issues:#?}");
    }

    #[test]
    fn invalid_type_classifies_as_type_invalid() {
        let xml = "<thing><name>x</name><count>not-a-number</count></thing>";
        let issues = validate_against_schema(xml, MINI_XSD, None);
        assert!(issues.iter().any(|i| i.code.contains("TypeInvalid")),
            "expected XSD/TypeInvalid: {issues:#?}");
    }

    #[test]
    fn negative_for_positive_classifies_as_type_invalid() {
        let xml = "<thing><name>x</name><count>-1</count></thing>";
        let issues = validate_against_schema(xml, MINI_XSD, None);
        assert!(issues.iter().any(|i| i.code.contains("TypeInvalid")),
            "expected XSD/TypeInvalid for negative-positive: {issues:#?}");
    }

    #[test]
    fn malformed_schema_aborts_with_critical() {
        // Genuinely malformed XML — trips the schema-side parse path.
        let issues = validate_against_schema("<x/>", "<broken schema", None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Critical);
    }

    #[test]
    fn malformed_instance_aborts_with_critical() {
        let issues = validate_against_schema("<not closed", MINI_XSD, None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Critical);
    }
}
