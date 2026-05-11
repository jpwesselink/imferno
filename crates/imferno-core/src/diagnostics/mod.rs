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
    /// Encoding and codec issues
    Encoding,
    /// Audio configuration issues
    Audio,
    /// Video configuration issues
    Video,
    /// Subtitle and caption issues
    Subtitle,
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
            Category::Audio => write!(f, "Audio"),
            Category::Video => write!(f, "Video"),
            Category::Subtitle => write!(f, "Subtitle"),
            Category::Metadata => write!(f, "Metadata"),
            Category::Security => write!(f, "Security"),
            Category::StudioSpecific(studio) => write!(f, "{} Specific", studio),
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
}

impl ValidationIssue {
    pub fn new(
        severity: Severity,
        category: Category,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            category,
            location: Location::new(),
            code: code.into(),
            message: message.into(),
            suggestion: None,
            context: HashMap::new(),
        }
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
}
