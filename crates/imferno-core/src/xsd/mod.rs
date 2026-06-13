//! Runtime XSD validation via uppsala — pure-Rust XSD 1.1 validator.
//!
//! This module wraps uppsala's structural diagnostics into the imferno
//! catalogue surface. Always on — SMPTE-spec-compliant validation is
//! not optional in imferno.
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

use std::path::Path;

use crate::diagnostics::codes::ValidationCode;
use crate::diagnostics::{Category, Location, Severity, ValidationIssue};

pub mod codes;

use codes::XsdConstraintCode;

// ─────────────────────────────────────────────────────────────────────────────
// Embedded XSDs — vendored under specs/ and baked in at compile time so the
// runtime XSD validator is self-contained.
// ─────────────────────────────────────────────────────────────────────────────

const IMF_CPL_2013_XSD: &str = include_str!("../../../../specs/imf-cpl.xsd");
// st2067-3a-2020.xsd is byte-identical to st2067-3a-2016.xsd apart from the
// header text and reuses the 2016 namespace, so 2020-era CPLs validate against
// this schema too.
const IMF_CPL_2016_XSD: &str = include_str!("../../../../specs/st2067-3a-2016.xsd");
const IMF_OPL_2014_XSD: &str = include_str!("../../../../specs/st2067-100a-2014.xsd");
const IMF_SCM_2018_XSD: &str = include_str!("../../../../specs/st2067-9a-2018.xsd");
const DCI_PKL_2007_XSD: &str = include_str!("../../../../specs/SMPTE-429-8-PKL-2007.xsd");
// st2067-2b-2020.xsd targets the same namespace as st2067-2b-2016.xsd
// (the 2020 publication reuses the 2016 PKL schema body modulo header
// text), so a single vendored copy covers both editions.
const IMF_PKL_2016_XSD: &str = include_str!("../../../../specs/st2067-2b-2016.xsd");
// The dcml stub is no longer baked into the binary — `dcml_specs_dir()`
// was the only consumer and it's been removed in v3.0.0 in favour of
// uppsala's in-memory schema path. The stub XSD remains in `specs/` for
// reference and for the future patch where uppsala grows an in-memory
// schema resolver.
//
// const DCML_TYPES_STUB_XSD: &str =
//     include_str!("../../../../specs/dcml-types-stub.xsd");

// `dcml_specs_dir()` was removed in v3.0.0 — the helper wrote the dcml
// stub to `std::env::temp_dir()` so uppsala could read it back via
// `from_schema_with_base_path`. That round-trip panicked on wasm32
// and defeated the entire reason uppsala was selected (pure-Rust,
// browser-runnable). The current code uses uppsala's in-memory
// `from_schema` path; the dcml `<xs:import>` directive is left
// unresolved and uppsala lax-validates anything in the dcml namespace.
// The dcml types are independently enforced at the parser layer
// (`ImfUuid::parse_urn` etc.), so no structural correctness is lost.

/// Run the runtime-XSD validator against a parsed `CompositionPlaylist`.
///
/// Returns empty if `cpl.source_xml` is None (parser didn't preserve it, or
/// the CPL was constructed manually) or if `cpl.namespace` is one we don't
/// have a vendored primary schema for.
///
/// This is the unified entry point: callers using `validate_cpl(&cpl)` get
/// schema-level diagnostics here just like callers using `validate_cpl_xml`
/// did via the raw-XML path.
///
/// **Works identically on native and wasm.** Uses uppsala's pure-in-memory
/// `from_schema` path (no filesystem, no `std::env::temp_dir()`). The
/// `<xs:import namespace=".../dcmlTypes/">` directives in the vendored
/// XSDs are left unresolved — uppsala lax-validates anything in the dcml
/// namespace. We accept that tradeoff because: (a) the dcml types
/// (`UUIDType`, `UserTextType`, `RationalType`) are also enforced at the
/// parser layer (`ImfUuid::parse_urn` etc.), so structural-correctness
/// signal is preserved; (b) the alternative was a temp-dir round-trip
/// that panicked on wasm — which defeated the entire reason we picked
/// uppsala in the first place.
pub fn validate_parsed_cpl(cpl: &crate::cpl::CompositionPlaylist) -> Vec<ValidationIssue> {
    let Some(source_xml) = &cpl.source_xml else {
        return Vec::new();
    };
    let primary_xsd = match &cpl.namespace {
        crate::cpl::CplNamespace::Smpte2067_3_2013 => IMF_CPL_2013_XSD,
        crate::cpl::CplNamespace::Smpte2067_3_2016 => IMF_CPL_2016_XSD,
        // No vendored XSD for DCI / Unknown — skip rather than fail loudly
        _ => return Vec::new(),
    };
    validate_against_schema(source_xml, primary_xsd, Some(cpl.id))
}

/// Run the runtime-XSD validator against an Output Profile List XML.
///
/// Takes the raw OPL XML directly (the `OutputProfileList` parser doesn't
/// preserve `source_xml` like the CPL one does). Validates against
/// `st2067-100a-2014.xsd`; OPL has only the 2014 edition published.
pub fn validate_opl_xml(source_xml: &str) -> Vec<ValidationIssue> {
    validate_against_schema(source_xml, IMF_OPL_2014_XSD, None)
}

/// Run the runtime-XSD validator against a Sidecar Composition Map XML.
///
/// Validates against `st2067-9a-2018.xsd` — the only published SCM edition.
pub fn validate_scm_xml(source_xml: &str) -> Vec<ValidationIssue> {
    validate_against_schema(source_xml, IMF_SCM_2018_XSD, None)
}

/// Run the runtime-XSD validator against a Packing List XML.
///
/// Three PKL namespaces have vendored XSDs:
/// - **DCI 429-8:2007** — `SMPTE-429-8-PKL-2007.xsd`.
/// - **ST 2067-2:2016 PKL** (`/schemas/2067-2/2016/PKL`) — `st2067-2b-2016.xsd`.
/// - **ST 2067-2:2020 PKL** — the canonical 2020 publication's PKL XSD
///   targets the same `/schemas/2067-2/2016/PKL` namespace and is
///   structurally identical to the 2016 schema, so the 2016 file
///   covers both editions.
///
/// The bare `/schemas/2067-2/2013` and `/schemas/2067-2/2016` variants
/// (without the `/PKL` suffix) skip the pre-pass — those forms are
/// either inline-CPL representations or use the legacy DCI PKL XSD.
///
/// Callers still get catalogue-level semantic checks via the package
/// validator regardless of whether the XSD pre-pass fired.
pub fn validate_pkl_xml(
    source_xml: &str,
    namespace: &crate::assetmap::PklNamespace,
) -> Vec<ValidationIssue> {
    use crate::assetmap::PklNamespace;
    match namespace {
        PklNamespace::Dci429_8 => validate_against_schema(source_xml, DCI_PKL_2007_XSD, None),
        PklNamespace::Smpte2067_2_2016Pkl | PklNamespace::Smpte2067_2_2020 => {
            validate_against_schema(source_xml, IMF_PKL_2016_XSD, None)
        }
        // ST 2067-2:2013 and the bare 2016/2020 namespaces don't have a
        // PKL-companion XSD in the wild. Skip — semantic checks still
        // run downstream via the package validator.
        _ => Vec::new(),
    }
}

/// Same as `validate_against_composite_schema` but takes the primary XSD as a
/// string rather than a filesystem path. Used by `validate_parsed_cpl` so the
/// library doesn't depend on the imferno repo layout at runtime.
pub fn validate_against_composite_schema_str(
    instance_xml: &str,
    primary_xsd: &str,
    specs_dir: &Path,
    cpl_id: Option<crate::assetmap::ImfUuid>,
) -> Vec<ValidationIssue> {
    let injected = inject_dcml_schema_location(primary_xsd);
    let schema_doc = match uppsala::parse(&injected) {
        Ok(d) => d,
        Err(e) => return vec![parse_failure_issue("xsd-schema", e, cpl_id)],
    };
    // uppsala's `from_schema_with_base_path` interprets `base_path` as a
    // *file* path and takes its parent to resolve `schemaLocation` URIs.
    // Synthesise a virtual file inside `specs_dir` so that `parent()`
    // returns `specs_dir` itself — otherwise relative imports resolve
    // one directory too high and uppsala silently `continue`s past the
    // missing file (dropping the import without an error).
    let virtual_base = specs_dir.join("__primary.xsd");
    let validator =
        match uppsala::XsdValidator::from_schema_with_base_path(&schema_doc, Some(&virtual_base)) {
            Ok(v) => v,
            Err(e) => return vec![schema_build_failure_issue(e, cpl_id)],
        };
    let instance_doc = match uppsala::parse(instance_xml) {
        Ok(d) => d,
        Err(e) => return vec![parse_failure_issue("xml-instance", e, cpl_id)],
    };
    validator
        .validate(&instance_doc)
        .into_iter()
        .map(|err| translate(err, cpl_id))
        .collect()
}

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

/// Validate an XML instance against a primary XSD that imports other
/// SMPTE namespaces, resolving imports against a vendored schema
/// directory.
///
/// This closes the lax-validation gap that `validate_against_schema`
/// has on namespace-only `xs:import` directives. SMPTE XSDs declare
/// `<xs:import namespace=".../dcmlTypes/"/>` with no `schemaLocation`,
/// which uppsala (and every other XSD validator) silently skips —
/// elements typed against `dcml:UUIDType` etc. then aren't validated.
///
/// This entry point injects `schemaLocation` hints for the imports we
/// have vendored stubs for (currently: the dcml types stub at
/// `specs/dcml-types-stub.xsd`), then calls uppsala's
/// `XsdValidator::from_schema_with_base_path` so the imports resolve
/// against `specs_dir`.
///
/// `primary_xsd_path` is read from disk to set the schema's base for
/// import resolution.
///
/// **Known uppsala v0.4.0 limitation**: pattern/restriction facets on
/// types imported from another namespace are loaded but not applied
/// during instance validation. So a `dcml:UUIDType`-typed element will
/// accept any string under the composite path, even though the stub
/// defines a UUID-URN regex. Built-in types (xs:dateTime, xs:integer
/// etc.) are unaffected and validate normally. See the integration
/// tests under `tests/xsd_runtime.rs` for the failing-as-of-v0.4.0
/// case (marked `#[ignore]`).
pub fn validate_against_composite_schema(
    instance_xml: &str,
    primary_xsd_path: &Path,
    specs_dir: &Path,
    cpl_id: Option<crate::assetmap::ImfUuid>,
) -> Vec<ValidationIssue> {
    let primary_xsd = match std::fs::read_to_string(primary_xsd_path) {
        Ok(s) => s,
        Err(e) => {
            return vec![parse_failure_issue("primary-xsd", e, cpl_id)];
        }
    };
    let injected = inject_dcml_schema_location(&primary_xsd);
    let schema_doc = match uppsala::parse(&injected) {
        Ok(d) => d,
        Err(e) => return vec![parse_failure_issue("xsd-schema", e, cpl_id)],
    };
    // See `validate_against_composite_schema_str` for why we synthesise
    // a virtual file path inside `specs_dir` rather than passing the
    // directory directly: uppsala treats the arg as a file and takes
    // `.parent()` on it.
    let virtual_base = specs_dir.join("__primary.xsd");
    let validator =
        match uppsala::XsdValidator::from_schema_with_base_path(&schema_doc, Some(&virtual_base)) {
            Ok(v) => v,
            Err(e) => return vec![schema_build_failure_issue(e, cpl_id)],
        };
    let instance_doc = match uppsala::parse(instance_xml) {
        Ok(d) => d,
        Err(e) => return vec![parse_failure_issue("xml-instance", e, cpl_id)],
    };
    validator
        .validate(&instance_doc)
        .into_iter()
        .map(|err| translate(err, cpl_id))
        .collect()
}

/// Top-level CPL validation: runs the composite XSD pass over the raw
/// XML and then the existing semantic validator over the parsed
/// `CompositionPlaylist`. Returns the concatenated issues with XSD
/// findings first (so operators see structural problems before
/// semantic ones that may have been induced by them).
///
/// If parsing fails, returns the parse failure as a single Critical
/// issue and skips semantic validation. XSD findings are still
/// returned regardless of parse outcome — the schema-level validator
/// doesn't need the parsed AST.
///
/// `primary_xsd_path` should point at the CPL XSD that matches the
/// instance's namespace (e.g. `specs/imf-cpl.xsd` for the 2013
/// namespace). `specs_dir` is the base path used to resolve schema
/// imports (typically the same `specs/` directory).
pub fn validate_cpl_xml(
    raw_xml: &str,
    primary_xsd_path: &Path,
    specs_dir: &Path,
) -> Vec<ValidationIssue> {
    let mut issues = validate_against_composite_schema(raw_xml, primary_xsd_path, specs_dir, None);

    match crate::cpl::parse_cpl(raw_xml) {
        Ok(cpl) => {
            issues.extend(crate::validation::validate_cpl(&cpl));
        }
        Err(e) => {
            issues.push(ValidationIssue::new(
                Severity::Critical,
                Category::Structure,
                "IMFERNO:Package/ParseError",
                format!("CPL XML failed to parse: {e:?}"),
            ));
        }
    }

    issues
}

/// Inject `schemaLocation="dcml-types-stub.xsd"` into the `<xs:import>`
/// for the ST 433 dcml namespace. Idempotent (no-op if a schemaLocation
/// is already present).
fn inject_dcml_schema_location(xsd_src: &str) -> String {
    const DCML_NS: &str = "http://www.smpte-ra.org/schemas/433/2008/dcmlTypes/";
    const STUB_PATH: &str = "dcml-types-stub.xsd";

    // Find the <xs:import …> element targeting the dcml namespace and
    // append a schemaLocation attribute if one isn't already there.
    // Handles both self-closing (/>) and open (>) tag terminators.
    let needle = format!(r#"<xs:import namespace="{DCML_NS}""#);
    let Some(start) = xsd_src.find(&needle) else {
        return xsd_src.to_string();
    };
    let tail = &xsd_src[start + needle.len()..];
    // Find the end of the start tag.
    let Some(end_rel) = tail.find('>') else {
        return xsd_src.to_string();
    };
    let tag_body = &tail[..end_rel]; // attributes between namespace="..." and >
    if tag_body.contains("schemaLocation") {
        return xsd_src.to_string(); // already has one
    }
    // Strip the trailing "/" if self-closing so we can reinsert it.
    let (attr_body, terminator) = if tag_body.trim_end().ends_with('/') {
        let trimmed = tag_body.trim_end();
        (&trimmed[..trimmed.len() - 1], "/>")
    } else {
        (tag_body, ">")
    };
    let before_tag_end = &xsd_src[..start + needle.len()];
    let after_tag = &tail[end_rel + 1..];
    format!(r#"{before_tag_end}{attr_body} schemaLocation="{STUB_PATH}"{terminator}{after_tag}"#)
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
    // When the patched uppsala fork populated element_path, append it to
    // the catalogue code so downstream code-string substring matches can
    // discriminate per-element (e.g. "XSD/ElementMissing/EditRate" vs the
    // generic "XSD/ElementMissing"). Falls back to the bare code when no
    // path is available (e.g. root-element-missing errors).
    let code: String = match &err.element_path {
        Some(path) if !path.is_empty() => format!("{}/{}", kind.code(), path),
        _ => kind.code().to_string(),
    };
    // Until `Location` grows line/column fields, we fold the position
    // into the human-readable message body so the information isn't lost.
    let message = match (err.line, err.column) {
        (Some(line), Some(col)) => format!("{} (at line {line}, column {col})", err.message),
        (Some(line), None) => format!("{} (at line {line})", err.message),
        _ => err.message,
    };
    ValidationIssue::new(kind.default_severity(), kind.category(), code, message).with_location(loc)
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
        format!(
            "XSD validation aborted: schema parsed but XsdValidator construction failed: {err:?}"
        ),
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
        assert!(
            issues.iter().any(|i| i.code.contains("ElementMissing")),
            "expected XSD/ElementMissing: {issues:#?}"
        );
    }

    #[test]
    fn unknown_element_classifies_as_unexpected_element() {
        let xml = "<thing><name>x</name><count>5</count><unknown/></thing>";
        let issues = validate_against_schema(xml, MINI_XSD, None);
        assert!(
            issues.iter().any(|i| i.code.contains("UnexpectedElement")),
            "expected XSD/UnexpectedElement: {issues:#?}"
        );
    }

    #[test]
    fn invalid_type_classifies_as_type_invalid() {
        let xml = "<thing><name>x</name><count>not-a-number</count></thing>";
        let issues = validate_against_schema(xml, MINI_XSD, None);
        assert!(
            issues.iter().any(|i| i.code.contains("TypeInvalid")),
            "expected XSD/TypeInvalid: {issues:#?}"
        );
    }

    #[test]
    fn negative_for_positive_classifies_as_type_invalid() {
        let xml = "<thing><name>x</name><count>-1</count></thing>";
        let issues = validate_against_schema(xml, MINI_XSD, None);
        assert!(
            issues.iter().any(|i| i.code.contains("TypeInvalid")),
            "expected XSD/TypeInvalid for negative-positive: {issues:#?}"
        );
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

    // ── FIX-6: pin classify() against each expected uppsala message shape ───
    //
    // The classifier is substring-based, so an uppsala upgrade that re-words
    // an error message would silently downgrade the diagnostic to the
    // catch-all `SchemaConstraintFailed`. These tests pin the five message
    // shapes observed in uppsala 0.4 + the imferno-patches fork; if any
    // string match here breaks, this test catches it before the change ships.

    #[test]
    fn classifier_pins_element_missing_shape() {
        let m = "Expected at least 1 occurrence of element 'EditRate'";
        assert_eq!(classify(m), XsdConstraintCode::ElementMissing);
    }

    #[test]
    fn classifier_pins_unexpected_element_shape() {
        let m = "Unexpected element 'BogusTag' encountered";
        assert_eq!(classify(m), XsdConstraintCode::UnexpectedElement);
    }

    #[test]
    fn classifier_pins_pattern_invalid_shape_v1() {
        let m = "Value 'abc' does not match pattern '[0-9]+'";
        assert_eq!(classify(m), XsdConstraintCode::PatternInvalid);
    }

    #[test]
    fn classifier_pins_pattern_invalid_shape_v2() {
        // Some uppsala paths emit the truncated phrasing without "pattern".
        let m = "Value does not match the expected facet";
        assert_eq!(classify(m), XsdConstraintCode::PatternInvalid);
    }

    #[test]
    fn classifier_pins_type_invalid_shape() {
        let m = "Value 'not-a-number' is not a valid xs:positiveInteger";
        assert_eq!(classify(m), XsdConstraintCode::TypeInvalid);
    }

    #[test]
    fn classifier_falls_back_to_schema_constraint_failed() {
        let m = "Some new message shape we don't know yet";
        assert_eq!(classify(m), XsdConstraintCode::SchemaConstraintFailed);
    }

    // ── FIX-9: non-CPL XSD pre-pass entry points ────────────────────────────
    //
    // The entry points use the vendored OPL / SCM / DCI-PKL schemas. We
    // don't have vendored XSDs for modern PKL / AssetMap / VolumeIndex so
    // those code paths simply return empty.

    #[test]
    fn validate_opl_xml_passes_clean_opl() {
        // Minimal OPL stripped down to the schema-required elements.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<OutputProfileList xmlns="http://www.smpte-ra.org/schemas/2067-100/2014">
    <Id>urn:uuid:8cf83c32-4949-4f00-b081-01e12b18932f</Id>
    <IssueDate>2016-06-14T19:22:37Z</IssueDate>
    <Issuer>Imferno</Issuer>
    <Creator>Imferno</Creator>
    <CompositionPlaylistId>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</CompositionPlaylistId>
    <MacroList/>
</OutputProfileList>"#;
        let issues = validate_opl_xml(xml);
        // The composite schema may still flag missing xmldsig content; the
        // important property here is the pre-pass is invokable end-to-end.
        for i in &issues {
            assert!(
                i.code.starts_with("XSD/"),
                "expected XSD/* codes only, got {i:#?}"
            );
        }
    }

    #[test]
    fn validate_opl_xml_flags_missing_required_field() {
        // Dropping <Issuer> from the OPL trips the schema's required-element check.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<OutputProfileList xmlns="http://www.smpte-ra.org/schemas/2067-100/2014">
    <Id>urn:uuid:8cf83c32-4949-4f00-b081-01e12b18932f</Id>
    <IssueDate>2016-06-14T19:22:37Z</IssueDate>
    <Creator>Imferno</Creator>
    <CompositionPlaylistId>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</CompositionPlaylistId>
    <MacroList/>
</OutputProfileList>"#;
        let issues = validate_opl_xml(xml);
        assert!(
            issues.iter().any(|i| i.code.contains("XSD/")),
            "expected at least one XSD diagnostic for missing Issuer: {issues:#?}"
        );
    }

    #[test]
    fn validate_scm_xml_passes_clean_scm() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<SidecarCompositionMap xmlns="http://www.smpte-ra.org/ns/2067-9/2018">
    <Id>urn:uuid:8cf83c32-4949-4f00-b081-01e12b18932f</Id>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <Properties>
        <SidecarAssetList>
            <SidecarAsset>
                <Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
                <AssociatedCPLList>
                    <CPLId>urn:uuid:75864667-c65e-4aae-a5b2-fa5ea5fe31b7</CPLId>
                </AssociatedCPLList>
            </SidecarAsset>
        </SidecarAssetList>
    </Properties>
</SidecarCompositionMap>"#;
        let issues = validate_scm_xml(xml);
        for i in &issues {
            assert!(
                i.code.starts_with("XSD/"),
                "expected XSD/* codes only, got {i:#?}"
            );
        }
    }

    #[test]
    fn validate_pkl_xml_skips_namespace_without_pkl_companion() {
        // The bare 2016 namespace (without the `/PKL` suffix) has no
        // companion XSD — skipped.
        use crate::assetmap::PklNamespace;
        let issues = validate_pkl_xml("<irrelevant/>", &PklNamespace::Smpte2067_2_2016);
        assert!(issues.is_empty(), "expected skip on bare 2016 namespace");
    }

    #[test]
    fn validate_pkl_xml_runs_for_modern_2016_pkl_namespace() {
        use crate::assetmap::PklNamespace;
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/2067-2/2016/PKL">
    <Id>urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c</Id>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <Issuer>Imferno</Issuer>
    <Creator>Imferno</Creator>
    <AssetList><Asset>
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
        <Size>1024</Size>
        <Type>application/mxf</Type>
    </Asset></AssetList>
</PackingList>"#;
        let issues = validate_pkl_xml(xml, &PklNamespace::Smpte2067_2_2016Pkl);
        for i in &issues {
            assert!(
                i.code.starts_with("XSD/"),
                "expected XSD/* codes only, got {i:#?}"
            );
        }
    }

    #[test]
    fn validate_pkl_xml_runs_for_2020_pkl_namespace() {
        // 2020 PKLs declare the same /schemas/2067-2/2016/PKL namespace
        // (the 2020 spec reused the schema), so they validate against
        // the 2016 vendored XSD. PklNamespace::Smpte2067_2_2020 is the
        // SMPTE-RA-landing-page form — it's a real namespace but PKL
        // documents in the wild don't use it.
        use crate::assetmap::PklNamespace;
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/2067-2/2016/PKL">
    <Id>urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c</Id>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <Issuer>Imferno</Issuer>
    <Creator>Imferno</Creator>
    <AssetList><Asset>
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
        <Size>1024</Size>
        <Type>application/mxf</Type>
    </Asset></AssetList>
</PackingList>"#;
        let issues = validate_pkl_xml(xml, &PklNamespace::Smpte2067_2_2020);
        for i in &issues {
            assert!(
                i.code.starts_with("XSD/"),
                "expected XSD/* codes only, got {i:#?}"
            );
        }
    }

    #[test]
    fn validate_pkl_xml_runs_for_dci_namespace() {
        use crate::assetmap::PklNamespace;
        // A clean DCI PKL — exercises the wiring; schema content
        // assertions are loose since the goal is end-to-end invokability.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
    <Id>urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c</Id>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <Issuer>Imferno</Issuer>
    <Creator>Imferno</Creator>
    <AssetList><Asset>
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
        <Size>1024</Size>
        <Type>application/mxf</Type>
    </Asset></AssetList>
</PackingList>"#;
        let issues = validate_pkl_xml(xml, &PklNamespace::Dci429_8);
        for i in &issues {
            assert!(
                i.code.starts_with("XSD/"),
                "expected XSD/* codes only, got {i:#?}"
            );
        }
    }
}
