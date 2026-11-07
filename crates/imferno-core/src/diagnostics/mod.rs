//! IMF validation finding model and report types.
//!
//! Cross-cutting types used by all spec modules to return findings.

use crate::assetmap::ImfUuid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// Severity level of validation issues
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational - Best practice suggestions
    Info,
    /// Warning - Should fix but not critical
    Warning,
    /// Error - Must fix for compliance
    Error,
    /// Critical - Prevents package from being usable
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Error => write!(f, "ERROR"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Category of validation issue
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    /// XML structure and syntax issues
    Structure,
    /// SMPTE schema compliance
    Schema,
    /// UUID and reference resolution
    Reference,
    /// Asset file availability and integrity
    Asset,
    /// Timing, frame rate, and duration issues
    Timing,
    /// Codec-level encoding issues (J2K profiles, JPEG-XS, AAC params,
    /// PCM bit depth, etc.). For *container*-level concerns (MXF
    /// wrapping, partition layout) use `Container`.
    Encoding,
    /// MXF / wrapping container constraints — distinct from `Encoding`
    /// which covers the codec carried inside the container.
    Container,
    /// Audio configuration issues
    Audio,
    /// Video configuration issues
    Video,
    /// Subtitle and caption issues
    Subtitle,
    /// Data essence (ISXD, dynamic metadata sidecars, ancillary data
    /// tracks). NOT for *metadata about* essence — that's `Metadata`.
    Data,
    /// Metadata and labeling issues
    Metadata,
    /// Security and DRM issues
    Security,
    /// Studio-specific requirements
    StudioSpecific(String),
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Structure => write!(f, "Structure"),
            Category::Schema => write!(f, "Schema"),
            Category::Reference => write!(f, "Reference"),
            Category::Asset => write!(f, "Asset"),
            Category::Timing => write!(f, "Timing"),
            Category::Encoding => write!(f, "Encoding"),
            Category::Container => write!(f, "Container"),
            Category::Audio => write!(f, "Audio"),
            Category::Video => write!(f, "Video"),
            Category::Subtitle => write!(f, "Subtitle"),
            Category::Data => write!(f, "Data"),
            Category::Metadata => write!(f, "Metadata"),
            Category::Security => write!(f, "Security"),
            Category::StudioSpecific(studio) => write!(f, "{} Specific", studio),
        }
    }
}

/// Which authority produced a validation finding.
///
/// Encodes the "prose vs XSD" provenance distinction that SMPTE ST 2067-3
/// §5.1 calls out ("the prose document takes precedence on conflict"),
/// so callers can filter, group, or apply per-source severity overrides
/// without inspecting the code string.
///
/// Derived from the rule's spec ([`Spec::source`]): `XSD` rules are
/// `XsdLayer`, `IMFERNO` rules `EngineInternal`, SMPTE specs `ProseRule`.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueSource {
    /// Came from the runtime XSD validator (uppsala). Schema-layer:
    /// the structural subset of the spec that the XSD DSL can express.
    XsdLayer,
    /// Came from a hand-rolled prose-cited rule. Semantic/cross-field/
    /// value-set checks that XSD can't express; the rule's code
    /// typically cites a SMPTE prose section (e.g. `ST2067-2:2020:6.4.2`).
    ProseRule,
    /// Came from imferno engine internals — parse failures, package
    /// structure checks, manifest issues — not directly traceable to
    /// a single SMPTE spec section.
    EngineInternal,
}

impl fmt::Display for IssueSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueSource::XsdLayer => write!(f, "XSD"),
            IssueSource::ProseRule => write!(f, "Prose"),
            IssueSource::EngineInternal => write!(f, "Engine"),
        }
    }
}

/// Location where the issue was found
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Location {
    /// File path if applicable
    pub file: Option<PathBuf>,
    /// CPL UUID if applicable
    pub cpl_id: Option<ImfUuid>,
    /// CPL filename if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpl_filename: Option<String>,
    /// CPL content title if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpl_title: Option<String>,
    /// Segment index (0-based)
    pub segment: Option<usize>,
    /// Segment Id (`urn:uuid:…`) if applicable
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment_id: Option<String>,
    /// Sequence UUID if applicable
    pub sequence_id: Option<String>,
    /// Resource UUID if applicable
    pub resource_id: Option<String>,
    /// Timecode if applicable
    pub timecode: Option<String>,
    /// Line number in XML file
    pub line: Option<usize>,
    /// XPath or field path
    pub path: Option<String>,
}

impl Location {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    pub fn with_cpl(mut self, cpl_id: ImfUuid) -> Self {
        self.cpl_id = Some(cpl_id);
        self
    }

    pub fn with_cpl_filename(mut self, filename: impl Into<String>) -> Self {
        self.cpl_filename = Some(filename.into());
        self
    }

    pub fn with_cpl_title(mut self, title: impl Into<String>) -> Self {
        self.cpl_title = Some(title.into());
        self
    }

    pub fn with_segment(mut self, segment: usize) -> Self {
        self.segment = Some(segment);
        self
    }

    pub fn with_segment_id(mut self, segment_id: impl Into<String>) -> Self {
        self.segment_id = Some(segment_id.into());
        self
    }

    pub fn with_resource(mut self, resource: usize) -> Self {
        self.resource_id = Some(resource.to_string());
        self
    }

    pub fn with_sequence(mut self, sequence_id: String) -> Self {
        self.sequence_id = Some(sequence_id);
        self
    }

    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        if let Some(ref file) = self.file {
            parts.push(format!("{}", file.display()));
        }
        if let Some(ref cpl_id) = self.cpl_id {
            let s = cpl_id.to_string();
            parts.push(format!("CPL:{}", &s[..8.min(s.len())]));
        }
        if let Some(segment) = self.segment {
            parts.push(format!("Segment:{}", segment + 1));
        }
        if let Some(ref sequence_id) = self.sequence_id {
            parts.push(format!("Seq:{}", &sequence_id[..8.min(sequence_id.len())]));
        }
        if let Some(line) = self.line {
            parts.push(format!("Line:{}", line));
        }
        if let Some(ref path) = self.path {
            parts.push(path.to_string());
        }

        write!(f, "{}", parts.join(", "))
    }
}

/// A single validation finding.
///
/// Created only through [`ValidationIssue::from_rule`]: severity, category,
/// clause and display code all come from the rule table row of the edition
/// that was applied.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Severity level (the table default, possibly remapped by a rules config).
    pub severity: Severity,
    /// Category of issue
    pub category: Category,
    /// Authority that emitted this issue (XSD layer, prose rule, engine
    /// internal), derived from the rule's spec.
    pub source: IssueSource,
    /// Stable, edition-free rule id (`ST2067-3/SegmentDuration`).
    pub rule: RuleId,
    /// Spec the rule belongs to.
    pub spec: Spec,
    /// Edition of the spec that was applied (`"2016"`; empty for
    /// single-edition pseudo-specs).
    pub edition: String,
    /// Clause of that edition the rule comes from (empty for pseudo-specs
    /// without clauses).
    pub clause: String,
    /// Normative strength of the rule in that edition.
    pub strength: Strength,
    /// Location where issue was found
    pub location: Location,
    /// Display code: `SPEC:EDITION:CLAUSE/Name`, with the edition actually
    /// applied (e.g. `ST2067-3:2016:7.2.2/SegmentDuration`).
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Suggestion for how to fix
    pub suggestion: Option<String>,
    /// Additional context
    pub context: HashMap<String, String>,
    /// Other `Location`s when this issue represents an aggregation of
    /// multiple identical-code occurrences (see
    /// [`ValidationReport::aggregate`]). Empty for fresh, un-aggregated
    /// issues. Skipped during serialisation when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_instances: Vec<Location>,
}

impl ValidationIssue {
    /// Report `rule` against `edition`, the edition of the rule's spec that
    /// was actually applied. Severity, category, clause, strength and the
    /// display code are read from the rule's table row for that edition.
    ///
    /// Reporting a rule for an edition whose row says the edition does not
    /// contain it is a bug: it panics in debug builds; release builds drop
    /// the finding and report `IMFERNO:Engine/RuleNotInEdition` instead.
    pub fn from_rule<E: Edition>(
        rule: &'static Rule<E>,
        edition: E,
        message: impl Into<String>,
    ) -> Self {
        let message = message.into();
        let Some((entry, clause)) = rule.entry(edition).and_then(|e| e.clause.map(|c| (e, c)))
        else {
            debug_assert!(
                false,
                "{} reported for {} {}, whose table row says the edition does not contain it",
                rule.id(),
                E::SPEC,
                edition.label()
            );
            return Self::from_rule(
                &crate::catalogue::imferno::RuleNotInEdition,
                crate::catalogue::imferno::ImfernoEdition::Current,
                format!(
                    "{} was reported for {} {}, which does not contain it (dropped finding: {})",
                    rule.id(),
                    E::SPEC,
                    edition.label(),
                    message
                ),
            );
        };
        Self {
            severity: entry.default_severity,
            category: rule.category.clone(),
            source: E::SPEC.source(),
            rule: rule.id(),
            spec: E::SPEC,
            edition: edition.label().to_string(),
            clause: clause.to_string(),
            strength: entry.strength,
            location: Location::new(),
            code: rule::display_code(E::SPEC, edition.label(), clause, rule.name),
            message,
            suggestion: None,
            context: HashMap::new(),
            additional_instances: Vec::new(),
        }
    }

    /// The rule's name (the part of [`Self::rule`] after the spec).
    pub fn rule_name(&self) -> &str {
        self.rule
            .0
            .split_once('/')
            .map(|(_, n)| n)
            .unwrap_or(&self.rule.0)
    }

    /// Whether this finding reports `rule` (any edition).
    pub fn is<E: Edition>(&self, rule: &Rule<E>) -> bool {
        self.spec == E::SPEC && self.rule_name() == rule.name
    }

    /// Number of occurrences this issue represents. `1` for a fresh
    /// issue, `1 + additional_instances.len()` after aggregation.
    pub fn instance_count(&self) -> usize {
        1 + self.additional_instances.len()
    }

    pub fn with_location(mut self, location: Location) -> Self {
        self.location = location;
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} ({}): {}",
            self.severity, self.category, self.code, self.message
        )?;

        if !self.location.to_string().is_empty() {
            write!(f, "\n  Location: {}", self.location)?;
        }

        if let Some(ref suggestion) = self.suggestion {
            write!(f, "\n  Suggestion: {}", suggestion)?;
        }

        let count = self.instance_count();
        if count > 1 {
            write!(f, "\n  Occurrences: {}", count)?;
        }

        Ok(())
    }
}

/// Comprehensive validation report
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Critical issues that prevent usage
    pub critical: Vec<ValidationIssue>,
    /// Errors that must be fixed for compliance
    pub errors: Vec<ValidationIssue>,
    /// Warnings that should be addressed
    pub warnings: Vec<ValidationIssue>,
    /// Informational issues
    pub info: Vec<ValidationIssue>,
    /// Issues that were suppressed by a `RuleSeverity::Off` override.
    /// These are not counted toward `is_playable`/`is_compliant` or
    /// surfaced by `has_errors`/`summary` — they exist so operators can
    /// debug their `RulesConfig` (`--show-suppressed`) without re-running
    /// validation. Each carries a `context["suppressed_by"]` annotation
    /// naming the rule key that matched.
    ///
    /// Skipped during serialisation when empty so reports without
    /// suppressed rules keep the previous on-wire shape.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suppressed: Vec<ValidationIssue>,
    /// Whether the package is playable despite issues
    pub is_playable: bool,
    /// Whether the package is compliant with base SMPTE standards
    pub is_compliant: bool,
    /// Validation profile used
    pub profile: ValidationProfile,
    /// Timestamp of validation
    pub timestamp: String,
}

impl ValidationReport {
    pub fn new(profile: ValidationProfile) -> Self {
        Self {
            critical: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            suppressed: Vec::new(),
            is_playable: true,
            is_compliant: true,
            profile,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn add(&mut self, issue: ValidationIssue) {
        match issue.severity {
            Severity::Critical => {
                self.critical.push(issue);
                self.is_playable = false;
                self.is_compliant = false;
            }
            Severity::Error => {
                self.errors.push(issue);
                self.is_compliant = false;
            }
            Severity::Warning => self.warnings.push(issue),
            Severity::Info => self.info.push(issue),
        }
    }

    /// Collapse repeat-offender diagnostics within each severity bucket.
    ///
    /// When the same `code` fires N times, the first-encountered issue
    /// is kept as the canonical instance and the rest contribute only
    /// their `Location` to its [`ValidationIssue::additional_instances`].
    /// All other fields (`message`, `suggestion`, `context`, `severity`,
    /// `category`) are taken from the first occurrence; later instances'
    /// `additional_instances` (if any — handles repeated aggregation
    /// idempotently) are merged in.
    ///
    /// Aggregation stays within severity buckets, since rules emit at
    /// a fixed severity per code. Order across the report is preserved
    /// (first-seen wins) so output remains reproducible.
    ///
    /// Idempotent and order-independent with [`Self::apply_rules`] —
    /// safe to call either before or after.
    pub fn aggregate(mut self) -> Self {
        fn collapse(bucket: &mut Vec<ValidationIssue>) {
            if bucket.len() < 2 {
                return;
            }
            let mut seen: HashMap<String, usize> = HashMap::with_capacity(bucket.len());
            let mut out: Vec<ValidationIssue> = Vec::with_capacity(bucket.len());
            for issue in bucket.drain(..) {
                match seen.get(&issue.code) {
                    Some(&i) => {
                        out[i].additional_instances.push(issue.location);
                        out[i]
                            .additional_instances
                            .extend(issue.additional_instances);
                    }
                    None => {
                        seen.insert(issue.code.clone(), out.len());
                        out.push(issue);
                    }
                }
            }
            *bucket = out;
        }
        collapse(&mut self.critical);
        collapse(&mut self.errors);
        collapse(&mut self.warnings);
        collapse(&mut self.info);
        self
    }

    /// Merge another report's issues into this one.
    ///
    /// The source report's `profile` and `timestamp` are discarded;
    /// only `self`'s values are retained. The `is_playable` and
    /// `is_compliant` flags are combined via logical AND.
    pub fn merge(&mut self, other: ValidationReport) {
        self.critical.extend(other.critical);
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.info.extend(other.info);
        self.suppressed.extend(other.suppressed);
        self.is_playable = self.is_playable && other.is_playable;
        self.is_compliant = self.is_compliant && other.is_compliant;
    }

    pub fn total_issues(&self) -> usize {
        self.critical.len() + self.errors.len() + self.warnings.len() + self.info.len()
    }

    pub fn has_critical(&self) -> bool {
        !self.critical.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn summary(&self) -> String {
        format!(
            "Validation Report: {} critical, {} errors, {} warnings, {} info",
            self.critical.len(),
            self.errors.len(),
            self.warnings.len(),
            self.info.len()
        )
    }
}

impl fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "IMF Package Validation Report")?;
        writeln!(f, "=============================")?;
        writeln!(f, "Profile: {}", self.profile)?;
        writeln!(f, "Timestamp: {}", self.timestamp)?;
        writeln!(
            f,
            "Playable: {}",
            if self.is_playable {
                "✅ YES"
            } else {
                "❌ NO"
            }
        )?;
        writeln!(
            f,
            "Compliant: {}",
            if self.is_compliant {
                "✅ YES"
            } else {
                "❌ NO"
            }
        )?;
        writeln!(f)?;

        if !self.critical.is_empty() {
            writeln!(f, "CRITICAL ISSUES ({}):", self.critical.len())?;
            for issue in &self.critical {
                writeln!(f, "  • {}", issue)?;
            }
            writeln!(f)?;
        }

        if !self.errors.is_empty() {
            writeln!(f, "ERRORS ({}):", self.errors.len())?;
            for issue in &self.errors {
                writeln!(f, "  • {}", issue)?;
            }
            writeln!(f)?;
        }

        if !self.warnings.is_empty() {
            writeln!(f, "WARNINGS ({}):", self.warnings.len())?;
            for issue in &self.warnings {
                writeln!(f, "  • {}", issue)?;
            }
            writeln!(f)?;
        }

        if !self.info.is_empty() {
            writeln!(f, "INFO ({}):", self.info.len())?;
            for issue in &self.info {
                writeln!(f, "  • {}", issue)?;
            }
        }

        Ok(())
    }
}

/// Validation profile determining strictness
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ValidationProfile {
    /// Minimal validation - just check if playable
    Minimal,
    /// Standard SMPTE compliance
    #[default]
    SMPTE,
    /// Custom profile with specific rules
    Custom,
}

impl fmt::Display for ValidationProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationProfile::Minimal => write!(f, "Minimal"),
            ValidationProfile::SMPTE => write!(f, "SMPTE"),
            ValidationProfile::Custom => write!(f, "Custom"),
        }
    }
}

/// Per-edition rule model (rules, edition tables, strength).
pub mod rule;
pub use rule::{Edition, Rule, RuleId, Spec, Strength};

/// ESLint-style per-rule severity overrides for `ValidationReport`.
pub mod rules;
pub use rules::{RuleSeverity, RulesConfig};

/// SARIF 2.1.0 output.
pub mod sarif;
pub use sarif::{sarif_from_json, to_sarif, to_sarif_json};

/// Result type for parsing operations that can accumulate errors
pub type ParseResult<T> = Result<(T, ValidationReport), CriticalError>;

/// Critical errors that prevent any further processing
#[derive(Debug)]
pub struct CriticalError {
    pub message: String,
    pub cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for CriticalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Critical Error: {}", self.message)?;
        if let Some(ref cause) = self.cause {
            write!(f, "\nCaused by: {}", cause)?;
        }
        Ok(())
    }
}

impl std::error::Error for CriticalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause
            .as_ref()
            .map(|e| &**e as &(dyn std::error::Error + 'static))
    }
}

impl From<std::io::Error> for CriticalError {
    fn from(err: std::io::Error) -> Self {
        CriticalError {
            message: format!("IO Error: {}", err),
            cause: Some(Box::new(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogue::imferno::{self as imf, ImfernoEdition};
    use crate::catalogue::st2067_2::{self, St2067_2Edition};
    use crate::catalogue::st2067_21::{self, App2EEdition};
    use crate::catalogue::st2067_3::{self, St2067_3Edition};
    use crate::catalogue::xsd::{self, XsdSchema};

    fn file_not_found() -> ValidationIssue {
        ValidationIssue::from_rule(&st2067_2::FileNotFound, St2067_2Edition::V2020, "missing")
    }

    #[test]
    fn from_rule_reads_severity_category_and_clause_from_the_table() {
        let issue = ValidationIssue::from_rule(
            &st2067_3::SegmentDuration,
            St2067_3Edition::V2013,
            "durations differ",
        );
        assert_eq!(issue.code, "ST2067-3:2013:7.2.2/SegmentDuration");
        assert_eq!(issue.rule.0, "ST2067-3/SegmentDuration");
        assert_eq!(issue.rule_name(), "SegmentDuration");
        assert_eq!(issue.spec, Spec::St2067_3);
        assert_eq!(issue.edition, "2013");
        assert_eq!(issue.clause, "7.2.2");
        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.strength, Strength::Shall);
        assert_eq!(issue.category, Category::Timing);
        assert_eq!(issue.source, IssueSource::ProseRule);
        assert!(issue.is(&st2067_3::SegmentDuration));
    }

    #[test]
    fn from_rule_reports_the_edition_applied() {
        // The same rule reported against two editions carries each
        // edition's own label and clause.
        let e2016 =
            ValidationIssue::from_rule(&st2067_2::FileNotFound, St2067_2Edition::V2016, "x");
        let e2020 = file_not_found();
        assert_eq!(e2016.code, "ST2067-2:2016:8.3/FileNotFound");
        assert_eq!(e2020.code, "ST2067-2:2020:8.3/FileNotFound");
        assert_eq!(e2016.rule, e2020.rule);
    }

    #[test]
    fn pseudo_specs_drop_empty_code_parts() {
        let engine = ValidationIssue::from_rule(&imf::ParseError, ImfernoEdition::Current, "x");
        assert_eq!(engine.code, "IMFERNO:Package/ParseError");
        assert_eq!(engine.source, IssueSource::EngineInternal);
        assert_eq!(engine.severity, Severity::Critical);
        let schema = ValidationIssue::from_rule(&xsd::TypeInvalid, XsdSchema::Cpl2016, "x");
        assert_eq!(schema.code, "XSD:CPL-2016/TypeInvalid");
        assert_eq!(schema.source, IssueSource::XsdLayer);
    }

    #[test]
    #[should_panic(expected = "does not contain it")]
    fn reporting_a_rule_outside_its_editions_panics_in_debug() {
        // ColorSystem is in every edition; HtIrvParameterBMax only in 2026.
        let _ =
            ValidationIssue::from_rule(&st2067_21::HtIrvParameterBMax, App2EEdition::V2014, "x");
    }

    #[test]
    fn rule_not_in_edition_is_an_engine_rule() {
        let rows = imf::RuleNotInEdition.editions;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].clause, Some("Engine"));
    }

    #[test]
    fn validation_issue_serde_round_trip() {
        let uuid = ImfUuid::parse("urn:uuid:abcdef00-1234-5678-9abc-def012345678").unwrap();
        let issue = file_not_found().with_location(Location::new().with_cpl(uuid));
        let json = serde_json::to_string(&issue).unwrap();
        assert!(json.contains(r#""rule":"ST2067-2/FileNotFound""#), "{json}");
        assert!(json.contains(r#""spec":"ST2067-2""#), "{json}");
        assert!(json.contains(r#""strength":"shall""#), "{json}");
        let back: ValidationIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(back.severity, Severity::Error);
        assert_eq!(back.location.cpl_id, Some(uuid));
        assert_eq!(back.rule, issue.rule);
        assert_eq!(back.edition, "2020");
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        report.add(ValidationIssue::from_rule(
            &imf::ParseError,
            ImfernoEdition::Current,
            "x",
        ));
        report.add(ValidationIssue::from_rule(
            &st2067_3::ContentKindUnknown,
            St2067_3Edition::V2016,
            "x",
        ));
        assert_eq!(report.total_issues(), 2);
        assert!(!report.is_playable);
        assert!(!report.is_compliant);
        assert!(report.has_critical());
        assert!(report.summary().contains("1 critical"));
        let display = format!("{report}");
        assert!(display.contains("IMFERNO:Package/ParseError"));
    }

    #[test]
    fn test_location_formatting() {
        let location = Location::new()
            .with_file(std::path::PathBuf::from("ASSETMAP.xml"))
            .with_cpl(ImfUuid::parse("urn:uuid:12345678-0000-0000-0000-000000000000").unwrap())
            .with_segment(2)
            .with_path("/path/to/package".to_string());
        let formatted = format!("{}", location);
        assert!(formatted.contains("ASSETMAP.xml"));
        assert!(formatted.contains("Segment:3"));
        assert!(Location::new().to_string().is_empty());
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::Error);
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
    }

    #[test]
    fn test_category_display() {
        assert_eq!(format!("{}", Category::Schema), "Schema");
        assert_eq!(format!("{}", Category::Timing), "Timing");
    }

    #[test]
    fn test_validation_report_merge() {
        let mut report1 = ValidationReport::new(ValidationProfile::SMPTE);
        report1.add(file_not_found());
        let mut report2 = ValidationReport::new(ValidationProfile::SMPTE);
        report2.add(ValidationIssue::from_rule(
            &st2067_3::ContentKindUnknown,
            St2067_3Edition::V2016,
            "x",
        ));
        report1.merge(report2);
        assert_eq!(report1.total_issues(), 2);
        assert!(report1.has_errors());
    }

    #[test]
    fn test_error_display() {
        let error = CriticalError {
            message: "Package not found".to_string(),
            cause: None,
        };
        assert!(format!("{}", error).contains("Package not found"));
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let critical_error = CriticalError::from(io_error);
        assert!(critical_error.message.contains("IO Error"));
        assert!(critical_error.cause.is_some());
    }

    #[test]
    fn test_validation_issue_chaining() {
        let issue = file_not_found()
            .with_location(Location::new().with_segment(5))
            .with_suggestion("Check segment timing")
            .with_context("phase", "During composition validation");
        assert!(issue.location.segment.is_some());
        assert!(issue.suggestion.is_some());
        assert!(!issue.context.is_empty());
    }

    /// FIX-12 regression: a ValidationReport carrying both a populated
    /// `suppressed` bucket and aggregated issues with non-empty
    /// `additional_instances` survives a serde JSON round-trip.
    #[test]
    fn validation_report_serde_round_trip_with_suppressed_and_aggregate() {
        let uuid1 = ImfUuid::parse("urn:uuid:00000000-0000-0000-0000-000000000001").unwrap();
        let uuid2 = ImfUuid::parse("urn:uuid:00000000-0000-0000-0000-000000000002").unwrap();
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        let mut aggregated = file_not_found().with_location(Location::new().with_cpl(uuid1));
        aggregated
            .additional_instances
            .push(Location::new().with_cpl(uuid2));
        report.errors.push(aggregated);
        let mut suppressed = file_not_found();
        suppressed
            .context
            .insert("suppressed_by".to_string(), "source:ProseRule".to_string());
        report.suppressed.push(suppressed);

        let json = serde_json::to_string(&report).expect("serialise");
        let back: ValidationReport = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(back.errors[0].instance_count(), 2);
        assert_eq!(back.errors[0].additional_instances[0].cpl_id, Some(uuid2));
        assert_eq!(
            back.suppressed[0]
                .context
                .get("suppressed_by")
                .map(String::as_str),
            Some("source:ProseRule")
        );
    }

    fn agg_issue<E: Edition>(rule: &'static Rule<E>, edition: E, cpl_byte: u8) -> ValidationIssue {
        let uuid = ImfUuid::parse(&format!(
            "urn:uuid:00000000-0000-0000-0000-0000000000{:02x}",
            cpl_byte
        ))
        .unwrap();
        ValidationIssue::from_rule(rule, edition, "test")
            .with_location(Location::new().with_cpl(uuid))
    }

    #[test]
    fn aggregate_collapses_repeat_codes_within_a_bucket() {
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        for b in 1..=3 {
            report.add(agg_issue(
                &st2067_2::FileNotFound,
                St2067_2Edition::V2020,
                b,
            ));
        }
        report.add(agg_issue(
            &st2067_2::SizeMismatch,
            St2067_2Edition::V2020,
            4,
        ));
        let out = report.aggregate();
        assert_eq!(out.errors.len(), 2);
        let agg = out
            .errors
            .iter()
            .find(|i| i.is(&st2067_2::FileNotFound))
            .unwrap();
        assert_eq!(agg.instance_count(), 3);
        let solo = out
            .errors
            .iter()
            .find(|i| i.is(&st2067_2::SizeMismatch))
            .unwrap();
        assert_eq!(solo.instance_count(), 1);
    }

    #[test]
    fn aggregate_keeps_editions_apart() {
        // Same rule, different editions → different codes → not merged.
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        report.add(agg_issue(
            &st2067_2::FileNotFound,
            St2067_2Edition::V2016,
            1,
        ));
        report.add(agg_issue(
            &st2067_2::FileNotFound,
            St2067_2Edition::V2020,
            2,
        ));
        assert_eq!(report.aggregate().errors.len(), 2);
    }

    #[test]
    fn aggregate_preserves_first_message_and_order() {
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        report.add(
            ValidationIssue::from_rule(&st2067_2::SizeMismatch, St2067_2Edition::V2020, "first")
                .with_suggestion("first suggestion"),
        );
        report.add(file_not_found());
        report.add(ValidationIssue::from_rule(
            &st2067_2::SizeMismatch,
            St2067_2Edition::V2020,
            "second",
        ));
        let out = report.aggregate();
        assert_eq!(out.errors.len(), 2);
        assert_eq!(out.errors[0].message, "first");
        assert_eq!(
            out.errors[0].suggestion.as_deref(),
            Some("first suggestion")
        );
        assert!(out.errors[1].is(&st2067_2::FileNotFound));
    }

    #[test]
    fn aggregate_is_idempotent_and_shows_count() {
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        for b in 1..=3 {
            report.add(agg_issue(
                &st2067_2::FileNotFound,
                St2067_2Edition::V2020,
                b,
            ));
        }
        let once = report.aggregate();
        let twice = once.clone().aggregate();
        assert_eq!(twice.errors.len(), 1);
        assert_eq!(twice.errors[0].instance_count(), 3);
        assert!(format!("{}", twice.errors[0]).contains("Occurrences: 3"));
    }
}
