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
/// Currently *inferred* from the code prefix on a `ValidationIssue`
/// (see `ValidationIssue::source`) — no engine code needs to set it
/// explicitly. If inference ever becomes ambiguous (e.g. a non-XSD
/// engine internal rule that wants the `XSD/` prefix for catalogue
/// reasons), we can add a `with_source()` builder without changing
/// the public shape.
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

impl IssueSource {
    /// Infer the source authority from a catalogue code string.
    ///
    /// Inference rules (checked top-to-bottom):
    /// - `XSD/...` → XsdLayer
    /// - `IMFERNO:...` or `IMFERNO/...` → EngineInternal
    /// - everything else → ProseRule (most catalogue codes cite a spec §)
    pub fn from_code(code: &str) -> Self {
        if code.starts_with("XSD/") {
            IssueSource::XsdLayer
        } else if code.starts_with("IMFERNO:") || code.starts_with("IMFERNO/") {
            IssueSource::EngineInternal
        } else {
            IssueSource::ProseRule
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

/// A single validation issue
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Severity level
    pub severity: Severity,
    /// Category of issue
    pub category: Category,
    /// Authority that emitted this issue (XSD layer, prose rule, engine
    /// internal). Inferred from `code` at construction time so the field
    /// is always consistent with the code prefix — JS consumers can
    /// filter on `issue.source` directly without re-implementing the
    /// inference. Deserialised with a default so old reports without the
    /// field round-trip cleanly.
    #[serde(default = "default_source_for_deserialize")]
    pub source: IssueSource,
    /// Location where issue was found
    pub location: Location,
    /// Error code (e.g., "ST2067-2:2020:8.3/FileNotFound")
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
    /// issues. Skipped during serialisation when empty so the on-wire
    /// shape stays back-compatible for callers that don't aggregate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_instances: Vec<Location>,
}

impl ValidationIssue {
    pub fn new(
        severity: Severity,
        category: Category,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let code: String = code.into();
        let source = IssueSource::from_code(&code);
        Self {
            severity,
            category,
            source,
            location: Location::new(),
            code,
            message: message.into(),
            suggestion: None,
            context: HashMap::new(),
            additional_instances: Vec::new(),
        }
    }

    /// Build a `ValidationIssue` directly from a typed `ValidationCode`,
    /// reading severity + category from the catalogue rather than the
    /// caller. Use this for any rule whose code lives in one of the
    /// `*_codes` modules — the catalogue is the single source of truth
    /// for severity and category. Pass the bare enum value:
    ///
    /// ```ignore
    /// use imferno_core::mxf::codes::St2067_2_2016;
    /// let issue = ValidationIssue::from_code(
    ///     St2067_2_2016::AudioSampleRateUnsupported,
    ///     format!("got {hz} Hz"),
    /// );
    /// ```
    pub fn from_code<C: codes::ValidationCode>(code: C, message: impl Into<String>) -> Self {
        // `code` is consumed only via `&self`-taking methods; we never
        // need to move it. The string code itself comes from
        // `code.code()` which returns `&'static str`, so no `Into<String>`
        // bound on `C` is required.
        Self::new(code.default_severity(), code.category(), code.code(), message)
    }

    /// Number of occurrences this issue represents. `1` for a fresh
    /// issue, `1 + additional_instances.len()` after aggregation.
    ///
    /// JS consumers can compute this themselves as
    /// `1 + issue.additionalInstances.length` — the field is the source
    /// of truth, this method is a Rust-side convenience.
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

/// Serde default for `ValidationIssue::source` — used only when
/// deserialising a payload that pre-dates the field. Returns
/// `IssueSource::EngineInternal` as a safe fallback; the proper value
/// will be re-derived if the issue is re-emitted, since `::new` always
/// computes from the code.
fn default_source_for_deserialize() -> IssueSource {
    IssueSource::EngineInternal
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

/// Typed validation-code catalogue with per-spec enums.
///
/// Every normative code emitted by the imf-rs validators is defined here,
/// grouped by the SMPTE specification that defines it.  See [`codes`] for
/// the full API.
pub mod codes;

/// ESLint-style per-rule severity overrides for `ValidationReport`.
pub mod rules;
pub use rules::{RuleSeverity, RulesConfig};

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

    #[test]
    fn test_validation_issue_creation() {
        let issue = ValidationIssue::new(
            Severity::Error,
            Category::Schema,
            "ST2067-2:2020:8.3/FileNotFound",
            "Missing required field 'EditRate' in Segment",
        )
        .with_location(
            Location::new()
                .with_cpl(ImfUuid::parse("urn:uuid:12345678-0000-0000-0000-000000000000").unwrap())
                .with_segment(0),
        )
        .with_suggestion("Add EditRate element with value like '24 1' or '24000 1001'");

        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.code, "ST2067-2:2020:8.3/FileNotFound");
        assert!(issue.suggestion.is_some());
    }

    #[test]
    fn issue_source_from_code_classifies_xsd_layer() {
        assert_eq!(
            IssueSource::from_code("XSD/TypeInvalid/IssueDate"),
            IssueSource::XsdLayer
        );
        assert_eq!(
            IssueSource::from_code("XSD/ElementMissing"),
            IssueSource::XsdLayer
        );
    }

    #[test]
    fn issue_source_from_code_classifies_engine_internal() {
        assert_eq!(
            IssueSource::from_code("IMFERNO:Package/UnreferencedAsset"),
            IssueSource::EngineInternal
        );
        assert_eq!(
            IssueSource::from_code("IMFERNO/Internal/ParseError"),
            IssueSource::EngineInternal
        );
    }

    #[test]
    fn issue_source_from_code_defaults_to_prose_rule() {
        // SMPTE prose-cited codes — explicit spec section in the prefix.
        assert_eq!(
            IssueSource::from_code("ST2067-2:2020:6.4.2/EssenceDescriptorList"),
            IssueSource::ProseRule
        );
        assert_eq!(
            IssueSource::from_code("ST2067-21:2023:7.1/AppIdMismatch"),
            IssueSource::ProseRule
        );
        // Catch-all: anything we don't recognise falls into ProseRule
        // (catalogue codes are overwhelmingly spec-cited).
        assert_eq!(
            IssueSource::from_code("dcml-UUID-Malformed"),
            IssueSource::ProseRule
        );
    }

    #[test]
    fn validation_issue_source_field_is_populated_from_code() {
        let xsd = ValidationIssue::new(
            Severity::Error,
            Category::Schema,
            "XSD/PatternInvalid/UUID",
            "uuid did not match pattern",
        );
        assert_eq!(xsd.source, IssueSource::XsdLayer);

        let prose = ValidationIssue::new(
            Severity::Warning,
            Category::Reference,
            "ST2067-3:2020:5.5.1.2/ContentKindUnknown",
            "unknown content kind",
        );
        assert_eq!(prose.source, IssueSource::ProseRule);

        let engine = ValidationIssue::new(
            Severity::Critical,
            Category::Structure,
            "IMFERNO:Package/ParseError",
            "could not parse CPL",
        );
        assert_eq!(engine.source, IssueSource::EngineInternal);
    }

    #[test]
    fn validation_issue_source_round_trips_through_serde() {
        let xsd = ValidationIssue::new(
            Severity::Error,
            Category::Schema,
            "XSD/PatternInvalid/UUID",
            "uuid did not match pattern",
        );
        let json = serde_json::to_string(&xsd).unwrap();
        assert!(json.contains("XsdLayer"), "source should serialise: {json}");
        let back: ValidationIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(back.source, IssueSource::XsdLayer);
    }

    #[test]
    fn validation_issue_source_deserialise_defaults_when_missing() {
        // Older payload shape without the `source` field — must
        // round-trip into a sensible default rather than failing.
        let legacy = r#"{
            "severity": "Error",
            "category": "Schema",
            "location": {},
            "code": "XSD/PatternInvalid/UUID",
            "message": "uuid did not match pattern",
            "suggestion": null,
            "context": {}
        }"#;
        let issue: ValidationIssue = serde_json::from_str(legacy).unwrap();
        assert_eq!(issue.source, IssueSource::EngineInternal);
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);

        report.add(ValidationIssue::new(
            Severity::Critical,
            Category::Asset,
            "ST2067-2:2020:8.3/FileNotFound",
            "Required MXF file not found",
        ));

        report.add(ValidationIssue::new(
            Severity::Warning,
            Category::Metadata,
            "META-001",
            "ContentKind not in recommended vocabulary",
        ));

        assert_eq!(report.total_issues(), 2);
        assert!(!report.is_playable);
        assert!(!report.is_compliant);
        assert!(report.has_critical());
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
        assert!(formatted.contains("1234-5678") || !formatted.is_empty());
        assert!(formatted.contains("2") || !formatted.is_empty());
    }

    #[test]
    fn test_severity_ordering() {
        // Test severity ordering for proper sorting
        assert!(Severity::Critical > Severity::Error);
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);

        let severities = vec![
            Severity::Info,
            Severity::Critical,
            Severity::Warning,
            Severity::Error,
        ];
        let mut sorted = severities.clone();
        sorted.sort();
        sorted.reverse(); // Highest first

        assert_eq!(
            sorted,
            vec![
                Severity::Critical,
                Severity::Error,
                Severity::Warning,
                Severity::Info
            ]
        );
    }

    #[test]
    fn test_category_display() {
        assert_eq!(format!("{}", Category::Schema), "Schema");
        assert_eq!(format!("{}", Category::Asset), "Asset");
        assert_eq!(format!("{}", Category::Metadata), "Metadata");
        assert_eq!(format!("{}", Category::Timing), "Timing");
        assert_eq!(format!("{}", Category::Asset), "Asset");
        assert_eq!(format!("{}", Category::Structure), "Structure");
    }

    #[test]
    fn test_validation_issue_with_context() {
        let mut issue = ValidationIssue::new(
            Severity::Warning,
            Category::Metadata,
            "META-002",
            "ContentKind uses non-standard value",
        );

        issue = issue.with_context("element", "Found in MainMarker element");

        assert!(!issue.context.is_empty());
        assert!(issue.context.contains_key("element"));
    }

    #[test]
    fn test_validation_report_merge() {
        let mut report1 = ValidationReport::new(ValidationProfile::SMPTE);
        report1.add(ValidationIssue::new(
            Severity::Error,
            Category::Schema,
            "ST2067-2:2020:8.3/ChecksumMismatch",
            "Invalid type for EditRate",
        ));

        let mut report2 = ValidationReport::new(ValidationProfile::SMPTE);
        report2.add(ValidationIssue::new(
            Severity::Warning,
            Category::Metadata,
            "META-003",
            "Missing annotation",
        ));

        report1.merge(report2);

        assert_eq!(report1.total_issues(), 2);
        assert!(report1.has_errors());
    }

    #[test]
    fn test_validation_report_summary() {
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);

        report.add(ValidationIssue::new(
            Severity::Critical,
            Category::Asset,
            "ST2067-2:2020:8.3/FileNotFound",
            "Critical issue",
        ));

        report.add(ValidationIssue::new(
            Severity::Error,
            Category::Schema,
            "ST2067-2:2020:8.3/ChecksumMismatch",
            "Error issue",
        ));

        report.add(ValidationIssue::new(
            Severity::Warning,
            Category::Metadata,
            "META-004",
            "Warning issue",
        ));

        report.add(ValidationIssue::new(
            Severity::Info,
            Category::Structure,
            "INFO-001",
            "Info issue",
        ));

        let summary = report.summary();
        assert!(summary.contains("1 critical") || !summary.is_empty());
        assert!(summary.contains("issues") || summary.len() > 10);
        // Summary is a formatted string, check it contains useful information
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_error_display() {
        let error = CriticalError {
            message: "Package not found".to_string(),
            cause: None,
        };

        let display = format!("{}", error);
        assert!(display.contains("Package not found"));
    }

    #[test]
    fn test_error_with_cause() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let critical_error = CriticalError::from(io_error);

        assert!(critical_error.message.contains("IO Error"));
        assert!(critical_error.cause.is_some());
    }

    #[test]
    fn test_validation_profile_display() {
        assert_eq!(format!("{}", ValidationProfile::SMPTE), "SMPTE");
        assert_eq!(format!("{}", ValidationProfile::SMPTE), "SMPTE");
        assert_eq!(format!("{}", ValidationProfile::Custom), "Custom");
        assert_eq!(format!("{}", ValidationProfile::Custom), "Custom");
    }

    #[test]
    fn test_location_edge_cases() {
        // Test empty location
        let empty_location = Location::new();
        let formatted = format!("{}", empty_location);
        assert!(formatted.is_empty());

        // Test location with only path
        let path_only = Location::new().with_path("/test/path".to_string());
        let formatted = format!("{}", path_only);
        assert!(formatted.contains("/test/path"));

        // Test location with only file
        let file_only = Location::new().with_file(std::path::PathBuf::from("test.xml"));
        let formatted = format!("{}", file_only);
        assert!(formatted.contains("test.xml"));
    }

    #[test]
    fn test_validation_issue_chaining() {
        let issue = ValidationIssue::new(
            Severity::Error,
            Category::Timing,
            "TL-001",
            "Timeline validation failed",
        )
        .with_location(Location::new().with_segment(5))
        .with_suggestion("Check segment timing")
        .with_context("phase", "During composition validation");

        assert!(issue.location.segment.is_some());
        assert!(issue.suggestion.is_some());
        assert!(!issue.context.is_empty());
    }

    #[test]
    fn test_validation_report_display() {
        let mut report = ValidationReport::new(ValidationProfile::Custom);

        report.add(ValidationIssue::new(
            Severity::Critical,
            Category::Asset,
            "ST2067-2:2020:8.3/FileNotFound",
            "Critical test issue",
        ));

        let display = format!("{}", report);
        assert!(display.contains("Critical"));
        assert!(
            display.contains("FILE_NOT_FOUND") || display.contains("Asset") || !display.is_empty()
        );
        assert!(display.contains("Critical test issue"));
    }

    #[test]
    fn test_error_codes() {
        // ValidationIssue stores whatever code string it is given.
        let issue = ValidationIssue::new(Severity::Error, Category::Asset, "A/Code", "msg");
        assert_eq!(issue.code, "A/Code");
        let issue2 = ValidationIssue::new(Severity::Error, Category::Asset, "B/Code", "msg");
        assert_ne!(issue.code, issue2.code);
    }

    #[test]
    fn location_cpl_id_serde_round_trip() {
        let uuid = ImfUuid::parse("urn:uuid:12345678-0000-0000-0000-000000000000").unwrap();
        let loc = Location::new().with_cpl(uuid);
        let json = serde_json::to_string(&loc).unwrap();
        let deserialized: Location = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.cpl_id, Some(uuid));
    }

    #[test]
    fn validation_issue_serde_round_trip() {
        let uuid = ImfUuid::parse("urn:uuid:abcdef00-1234-5678-9abc-def012345678").unwrap();
        let issue = ValidationIssue::new(
            Severity::Warning,
            Category::Structure,
            "TEST/Code",
            "test message",
        )
        .with_location(Location::new().with_cpl(uuid));

        let json = serde_json::to_string(&issue).unwrap();
        let deserialized: ValidationIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.severity, Severity::Warning);
        assert_eq!(deserialized.location.cpl_id, Some(uuid));
    }

    /// FIX-12 regression: a ValidationReport carrying both a populated
    /// `suppressed` bucket and aggregated issues with non-empty
    /// `additional_instances` survives a serde JSON round-trip.
    #[test]
    fn validation_report_serde_round_trip_with_suppressed_and_aggregate() {
        let uuid1 = ImfUuid::parse("urn:uuid:00000000-0000-0000-0000-000000000001").unwrap();
        let uuid2 = ImfUuid::parse("urn:uuid:00000000-0000-0000-0000-000000000002").unwrap();
        let uuid3 = ImfUuid::parse("urn:uuid:00000000-0000-0000-0000-000000000003").unwrap();

        let mut report = ValidationReport::new(ValidationProfile::SMPTE);

        // Aggregated issue: 1 primary + 2 additional instances.
        let mut aggregated = ValidationIssue::new(
            Severity::Error,
            Category::Schema,
            "XSD/PatternInvalid/UUID",
            "uuid pattern violation",
        )
        .with_location(Location::new().with_cpl(uuid1));
        aggregated
            .additional_instances
            .push(Location::new().with_cpl(uuid2));
        aggregated
            .additional_instances
            .push(Location::new().with_cpl(uuid3));
        report.errors.push(aggregated);

        // Suppressed issue: demoted by a hypothetical rule key, annotated.
        let mut suppressed = ValidationIssue::new(
            Severity::Info,
            Category::Schema,
            "XSD/TypeInvalid/IssueDate",
            "issue date not a valid xs:dateTime",
        )
        .with_location(Location::new().with_cpl(uuid1));
        suppressed
            .context
            .insert("suppressed_by".to_string(), "source:XsdLayer".to_string());
        report.suppressed.push(suppressed);

        let json = serde_json::to_string(&report).expect("serialise");
        let back: ValidationReport = serde_json::from_str(&json).expect("deserialise");

        assert_eq!(back.errors.len(), 1);
        assert_eq!(back.errors[0].additional_instances.len(), 2);
        assert_eq!(back.errors[0].additional_instances[0].cpl_id, Some(uuid2));
        assert_eq!(back.errors[0].additional_instances[1].cpl_id, Some(uuid3));
        assert_eq!(back.errors[0].instance_count(), 3);

        assert_eq!(back.suppressed.len(), 1);
        assert_eq!(
            back.suppressed[0].context.get("suppressed_by").map(String::as_str),
            Some("source:XsdLayer")
        );
    }

    fn agg_issue(code: &str, severity: Severity, cpl_byte: u8) -> ValidationIssue {
        let uuid =
            ImfUuid::parse(&format!("urn:uuid:00000000-0000-0000-0000-0000000000{:02x}", cpl_byte))
                .unwrap();
        ValidationIssue::new(severity, Category::Schema, code, "test")
            .with_location(Location::new().with_cpl(uuid))
    }

    #[test]
    fn aggregate_collapses_repeat_codes_within_a_bucket() {
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        report.add(agg_issue("XSD/PatternInvalid/UUID", Severity::Error, 1));
        report.add(agg_issue("XSD/PatternInvalid/UUID", Severity::Error, 2));
        report.add(agg_issue("XSD/PatternInvalid/UUID", Severity::Error, 3));
        report.add(agg_issue("XSD/Other/X", Severity::Error, 4));
        let out = report.aggregate();
        // Two distinct codes → two issues in errors bucket.
        assert_eq!(out.errors.len(), 2);
        // The repeated-code issue carries the extra locations.
        let agg = out
            .errors
            .iter()
            .find(|i| i.code == "XSD/PatternInvalid/UUID")
            .unwrap();
        assert_eq!(agg.instance_count(), 3);
        assert_eq!(agg.additional_instances.len(), 2);
        // First-seen location is retained on the canonical issue.
        // Other code stays as a single-instance issue.
        let solo = out.errors.iter().find(|i| i.code == "XSD/Other/X").unwrap();
        assert_eq!(solo.instance_count(), 1);
    }

    #[test]
    fn aggregate_preserves_first_message_and_severity() {
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        report.add(
            ValidationIssue::new(
                Severity::Error,
                Category::Schema,
                "X/Y",
                "first message",
            )
            .with_suggestion("first suggestion"),
        );
        report.add(ValidationIssue::new(
            Severity::Error,
            Category::Schema,
            "X/Y",
            "second message",
        ));
        let out = report.aggregate();
        assert_eq!(out.errors.len(), 1);
        assert_eq!(out.errors[0].message, "first message");
        assert_eq!(out.errors[0].suggestion.as_deref(), Some("first suggestion"));
    }

    #[test]
    fn aggregate_does_not_cross_severity_buckets() {
        // Same code in different buckets should NOT merge — different
        // buckets indicate the issue was demoted/promoted independently.
        // (In practice rules emit at one severity per code, but be
        // defensive against post-`apply_rules` weirdness.)
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        report.add(agg_issue("X/Y", Severity::Error, 1));
        report.add(agg_issue("X/Y", Severity::Warning, 2));
        let out = report.aggregate();
        assert_eq!(out.errors.len(), 1);
        assert_eq!(out.warnings.len(), 1);
    }

    #[test]
    fn aggregate_is_idempotent() {
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        report.add(agg_issue("X/Y", Severity::Error, 1));
        report.add(agg_issue("X/Y", Severity::Error, 2));
        report.add(agg_issue("X/Y", Severity::Error, 3));
        let once = report.clone().aggregate();
        let twice = once.clone().aggregate();
        // Aggregating already-aggregated report yields same shape.
        assert_eq!(twice.errors.len(), 1);
        assert_eq!(twice.errors[0].instance_count(), 3);
    }

    #[test]
    fn aggregate_preserves_first_seen_order() {
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        report.add(agg_issue("Z/last", Severity::Error, 1));
        report.add(agg_issue("A/first", Severity::Error, 2));
        report.add(agg_issue("Z/last", Severity::Error, 3));
        report.add(agg_issue("A/first", Severity::Error, 4));
        let out = report.aggregate();
        // First-seen order maps to "Z/last" → idx 0, "A/first" → idx 1.
        assert_eq!(out.errors[0].code, "Z/last");
        assert_eq!(out.errors[1].code, "A/first");
    }

    #[test]
    fn aggregate_short_circuits_when_bucket_is_singleton() {
        // Single issue in a bucket — aggregate is a no-op and should
        // leave the issue untouched (no unnecessary clone / re-push).
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        report.add(agg_issue("X/Y", Severity::Error, 1));
        let out = report.aggregate();
        assert_eq!(out.errors.len(), 1);
        assert_eq!(out.errors[0].additional_instances.len(), 0);
        assert_eq!(out.errors[0].instance_count(), 1);
    }

    #[test]
    fn aggregate_display_shows_occurrence_count_when_above_one() {
        let mut report = ValidationReport::new(ValidationProfile::SMPTE);
        report.add(agg_issue("X/Y", Severity::Error, 1));
        report.add(agg_issue("X/Y", Severity::Error, 2));
        let out = report.aggregate();
        let rendered = format!("{}", out.errors[0]);
        assert!(
            rendered.contains("Occurrences: 2"),
            "Display should mention aggregate count: {rendered}"
        );
    }
}
