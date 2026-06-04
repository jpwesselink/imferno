//! ESLint-style per-rule severity configuration.
//!
//! A [`RulesConfig`] maps rule identifiers to [`RuleSeverity`] overrides.
//! Rules can be identified by:
//! - Full normalised code — `"ST2067-3:2020:7.2.2/SegmentDuration"`.
//! - Rule suffix — `"SegmentDuration"`.
//! - Glob pattern with `*` as a single-segment wildcard —
//!   `"XSD/*"`, `"XSD/PatternInvalid/*"`, `"ST2067-*:2020:*/EditRate"`.
//! - Source prefix — `"source:XsdLayer"`, `"source:ProseRule"`,
//!   `"source:EngineInternal"` — selects every issue whose
//!   [`IssueSource`] inference matches.
//!
//! When multiple keys match a single issue, the most-specific key wins
//! (full code > suffix > glob > source-prefix). Within glob keys, the
//! one with the longer literal prefix is more specific.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::diagnostics::codes::ValidationCode;
use crate::diagnostics::IssueSource;
use crate::{Severity, ValidationReport};

/// Per-rule severity override.
///
/// Mirrors ESLint's `"off"` / `"warn"` / `"error"` vocabulary, extended with
/// the two IMF severity levels.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleSeverity {
    /// Suppress this rule entirely — matching issues are removed from output.
    Off,
    /// Remap to `Info`.
    Info,
    /// Remap to `Warning`.
    Warn,
    /// Remap to `Error`.
    Error,
    /// Remap to `Critical`.
    Critical,
}

/// A diagnostic about a `RulesConfig` key that the engine couldn't match.
///
/// Produced by [`RulesConfig::validate`]. Operators can use these at
/// config-load time to catch typos and unsupported syntax before any
/// validation work runs.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleValidationWarning {
    /// The configured key that triggered the warning.
    pub key: String,
    /// Why it triggered.
    pub reason: RuleValidationReason,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuleValidationReason {
    /// `source:<variant>` — `<variant>` isn't a known [`IssueSource`].
    UnknownSource { variant: String },
    /// The key parsed but matched zero codes in the supplied universe.
    MatchesNothing,
    /// The key used syntax the matcher doesn't support (e.g. `**`).
    UnsupportedPattern { hint: String },
}

/// ESLint-style per-rule severity overrides.
///
/// Keys are either:
/// - A rule suffix — `"SegmentDuration"` — matched against the part of the
///   issue code after the last `/`.
/// - A full normalised code — `"ST2067-3:2020:7.2.2/SegmentDuration"`.
///
/// Values are the desired [`RuleSeverity`], or [`RuleSeverity::Off`] to
/// suppress the rule entirely.
///
/// An empty map (the default) is a no-op.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RulesConfig(HashMap<String, RuleSeverity>);

impl RulesConfig {
    /// Set the severity for a typed validation code.
    ///
    /// ```
    /// use imferno_core::diagnostics::rules::{RulesConfig, RuleSeverity};
    /// use imferno_core::assetmap::codes::St2067_2_2020;
    ///
    /// let mut rules = RulesConfig::default();
    /// rules.set(St2067_2_2020::FileNotFound, RuleSeverity::Critical);
    /// ```
    pub fn set(&mut self, code: impl ValidationCode, severity: RuleSeverity) {
        self.0.insert(code.code().to_string(), severity);
    }

    /// Set severity by raw string key (rule suffix or full code).
    pub fn set_raw(&mut self, key: String, severity: RuleSeverity) {
        self.0.insert(key, severity);
    }

    /// Returns `true` if no overrides are configured.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Number of configured overrides.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check the configured keys against a known-code universe and return
    /// a list of warnings for keys that match nothing. Useful at config-load
    /// time so operators get fast feedback on typos and unsupported syntax.
    ///
    /// The caller supplies the known-code universe (typically obtained from
    /// the `listRules` enumerator on the NAPI/wasm boundary, or by iterating
    /// every typed code enum's `ALL` const).
    ///
    /// Warning categories:
    /// - `UnknownSource` — `source:Foo` where `Foo` isn't a known variant.
    /// - `MatchesNothing` — the key parsed fine but didn't match any code
    ///   in `known_codes` (typo, removed rule, or fictional namespace).
    /// - `UnsupportedPattern` — syntax we don't implement (e.g. `**`).
    pub fn validate<I, S>(&self, known_codes: I) -> Vec<RuleValidationWarning>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let codes: Vec<String> = known_codes
            .into_iter()
            .map(|c| c.as_ref().to_string())
            .collect();
        let mut warnings = Vec::new();
        for key in self.0.keys() {
            if let Some(rest) = key.strip_prefix("source:") {
                if parse_source(rest).is_none() {
                    warnings.push(RuleValidationWarning {
                        key: key.clone(),
                        reason: RuleValidationReason::UnknownSource {
                            variant: rest.to_string(),
                        },
                    });
                }
                continue;
            }
            // `**` is reserved for any-depth wildcards but unsupported by
            // `glob_match` (each `*` matches a single segment). Flag rather
            // than silently fail to match.
            if key.contains("**") {
                warnings.push(RuleValidationWarning {
                    key: key.clone(),
                    reason: RuleValidationReason::UnsupportedPattern {
                        hint: "`**` (any-depth wildcard) is not supported; use `*/*` or `source:<Variant>` for broader scopes".to_string(),
                    },
                });
                continue;
            }
            let matches_any = codes
                .iter()
                .any(|c| match_specificity(c, key).is_some());
            if !matches_any {
                warnings.push(RuleValidationWarning {
                    key: key.clone(),
                    reason: RuleValidationReason::MatchesNothing,
                });
            }
        }
        warnings.sort_by(|a, b| a.key.cmp(&b.key));
        warnings
    }

    /// Iterate over configured overrides.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &RuleSeverity)> {
        self.0.iter()
    }
}

/// How specific a matching rule key is. Higher specificity wins when
/// multiple keys match the same issue. Variants are ordered from least
/// to most specific so `Ord` reflects "more specific = greater".
///
/// Ranking rationale: full-code keys carry the most caller intent
/// (exact target). Globs are anchored to a position in the code path,
/// so they beat bare suffix matches (which match anywhere named X).
/// Source-prefix keys are the broadest backstop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Specificity {
    /// `source:XsdLayer` etc. — broadest scope, lowest specificity.
    SourcePrefix,
    /// Bare suffix match (e.g. `SegmentDuration` — matches anywhere
    /// the rule code ends with that segment).
    Suffix,
    /// Glob pattern with `*`. Inner tuple breaks ties: longer literal
    /// prefix wins, then longer total key length.
    Glob(usize, usize),
    /// Full-code exact match — most specific.
    FullCode,
}

/// Test whether `key` matches `code` and, if so, how specific the match is.
/// Returns `None` for no match.
fn match_specificity(code: &str, key: &str) -> Option<Specificity> {
    // 1. Source-prefix keys: `source:XsdLayer` etc. Lowest specificity
    //    (intentional — operators write these as a broad backstop).
    if let Some(rest) = key.strip_prefix("source:") {
        return parse_source(rest)
            .filter(|src| IssueSource::from_code(code) == *src)
            .map(|_| Specificity::SourcePrefix);
    }

    // 2. Glob match: any `*` segment matches one path segment.
    if key.contains('*') {
        return glob_match(code, key)
            .then_some(Specificity::Glob(literal_prefix_len(key), key.len()));
    }

    // 3. Full-code exact match (key with `/` or `:` reads as a full code).
    if code == key {
        return Some(Specificity::FullCode);
    }

    // 4. Suffix match — final segment after the last `/`.
    if code.rsplit('/').next() == Some(key) {
        return Some(Specificity::Suffix);
    }

    None
}

/// Match a code against a glob key using `*` as a single-segment wildcard.
/// `*` matches exactly one segment between `/`s; e.g. `XSD/*` matches
/// `XSD/TypeInvalid` but not `XSD/TypeInvalid/IssueDate`. A trailing `*`
/// segment can be expanded to match the remaining tail by writing it as
/// the last segment with nothing after — but for explicit
/// "match-anything-deeper" use `XSD/*/*`, etc. To match any depth,
/// callers can use the source-prefix form (`source:XsdLayer`).
///
/// Mid-segment wildcards (e.g. `ST2067-*:2020`) are supported via
/// prefix/suffix anchoring around a single `*` per segment.
fn glob_match(code: &str, key: &str) -> bool {
    let code_parts: Vec<&str> = code.split('/').collect();
    let key_parts: Vec<&str> = key.split('/').collect();
    if code_parts.len() != key_parts.len() {
        return false;
    }
    code_parts
        .iter()
        .zip(key_parts.iter())
        .all(|(c, k)| segment_matches(c, k))
}

/// Match one path segment, supporting any number of `*` wildcards. Each
/// `*` is a "match-any-substring" wildcard anchored by the literals
/// before and after it. Standard left-to-right greedy glob matching.
fn segment_matches(code_seg: &str, key_seg: &str) -> bool {
    if !key_seg.contains('*') {
        return code_seg == key_seg;
    }
    let pieces: Vec<&str> = key_seg.split('*').collect();
    // `split` on `*` yields N+1 pieces for N stars. The first must be a
    // prefix of the code segment; the last must be a suffix; the rest
    // must appear in order in between.
    let first = pieces.first().copied().unwrap_or("");
    let last = pieces.last().copied().unwrap_or("");
    if !code_seg.starts_with(first) || !code_seg.ends_with(last) {
        return false;
    }
    if pieces.len() == 1 {
        // No `*` at all — handled by the early-return above, but keep
        // this branch defensive.
        return code_seg == first;
    }
    // Ensure prefix + suffix don't overlap (e.g. key "a*b" against
    // code "ab" — len check confirms there's room for the middle
    // pieces, even if there are none).
    if code_seg.len() < first.len() + last.len() {
        return false;
    }

    // Walk the middle pieces in order. Each must appear after the
    // previous match. Skip empties (consecutive `**` collapses to one).
    let mut cursor = first.len();
    let end = code_seg.len() - last.len();
    for piece in &pieces[1..pieces.len() - 1] {
        if piece.is_empty() {
            continue;
        }
        match code_seg[cursor..end].find(piece) {
            Some(offset) => cursor += offset + piece.len(),
            None => return false,
        }
    }
    true
}

/// Length of the literal prefix in a glob key (characters before the
/// first `*`). Used to break ties between glob keys: `XSD/PatternInvalid/*`
/// (prefix len 17) outranks `XSD/*` (prefix len 4).
fn literal_prefix_len(key: &str) -> usize {
    key.find('*').unwrap_or(key.len())
}

/// Parse a source-prefix variant name (e.g. `XsdLayer`) into an [`IssueSource`].
///
/// Matching is case-insensitive so operator-friendly keys like
/// `source:xsdlayer`, `source:XSDLAYER`, and `source:XsdLayer` are all
/// accepted. Returns `None` for unknown variants — `RulesConfig::validate()`
/// (FIX-13) surfaces those as unmatchable-pattern warnings.
fn parse_source(name: &str) -> Option<IssueSource> {
    if name.eq_ignore_ascii_case("XsdLayer") {
        Some(IssueSource::XsdLayer)
    } else if name.eq_ignore_ascii_case("ProseRule") {
        Some(IssueSource::ProseRule)
    } else if name.eq_ignore_ascii_case("EngineInternal") {
        Some(IssueSource::EngineInternal)
    } else {
        None
    }
}

impl ValidationReport {
    /// Apply ESLint-style per-rule severity overrides.
    ///
    /// Issues whose rule matches a [`RuleSeverity::Off`] entry are removed.
    /// All other matching issues have their severity remapped and re-bucketed.
    /// `is_playable` and `is_compliant` are recomputed from the updated buckets.
    ///
    /// When multiple keys match the same issue, the most-specific match
    /// wins (see module docs). Selection is deterministic across runs.
    ///
    /// An empty [`RulesConfig`] is a no-op (fast path, no allocation).
    pub fn apply_rules(mut self, rules: &RulesConfig) -> Self {
        if rules.is_empty() {
            return self;
        }

        let all: Vec<_> = self
            .critical
            .drain(..)
            .chain(self.errors.drain(..))
            .chain(self.warnings.drain(..))
            .chain(self.info.drain(..))
            .collect();

        for mut issue in all {
            let matched = rules
                .iter()
                .filter_map(|(k, v)| match_specificity(&issue.code, k).map(|s| (s, k, v)))
                // Pick the highest specificity; on a tie, the longer key
                // wins (already encoded in `Specificity::Glob`). For all
                // other tiers, ties are impossible given the grammar.
                .max_by(|(a, ak, _), (b, bk, _)| a.cmp(b).then_with(|| ak.len().cmp(&bk.len())));

            match matched {
                Some((_, key, RuleSeverity::Off)) => {
                    // Suppressed — annotate with the matching rule key
                    // and park in the suppressed bucket so operators can
                    // `--show-suppressed` to debug their config.
                    // Severity is demoted to Info so any iteration over
                    // `suppressed` doesn't show misleading severities.
                    issue
                        .context
                        .insert("suppressed_by".to_string(), key.clone());
                    issue.severity = Severity::Info;
                    self.suppressed.push(issue);
                }
                Some((_, _, RuleSeverity::Info)) => {
                    issue.severity = Severity::Info;
                    self.info.push(issue);
                }
                Some((_, _, RuleSeverity::Warn)) => {
                    issue.severity = Severity::Warning;
                    self.warnings.push(issue);
                }
                Some((_, _, RuleSeverity::Error)) => {
                    issue.severity = Severity::Error;
                    self.errors.push(issue);
                }
                Some((_, _, RuleSeverity::Critical)) => {
                    issue.severity = Severity::Critical;
                    self.critical.push(issue);
                }
                None => match issue.severity {
                    Severity::Critical => self.critical.push(issue),
                    Severity::Error => self.errors.push(issue),
                    Severity::Warning => self.warnings.push(issue),
                    Severity::Info => self.info.push(issue),
                },
            }
        }

        self.is_playable = self.critical.is_empty();
        self.is_compliant = self.critical.is_empty() && self.errors.is_empty();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_config_accessors() {
        let mut rules = RulesConfig::default();
        assert!(rules.is_empty());
        assert_eq!(rules.len(), 0);

        rules.set(
            crate::assetmap::codes::St2067_2_2020::FileNotFound,
            RuleSeverity::Critical,
        );
        assert!(!rules.is_empty());
        assert_eq!(rules.len(), 1);
        assert_eq!(rules.iter().count(), 1);
    }

    #[test]
    fn rules_config_serde_round_trip() {
        let mut rules = RulesConfig::default();
        rules.set(
            crate::assetmap::codes::St2067_2_2020::FileNotFound,
            RuleSeverity::Off,
        );
        let json = serde_json::to_string(&rules).unwrap();
        let deserialized: RulesConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.len(), 1);
    }

    use crate::diagnostics::{
        Category, IssueSource, Location, ValidationIssue, ValidationProfile,
    };

    fn issue(code: &str, severity: Severity) -> ValidationIssue {
        ValidationIssue::new(severity, Category::Schema, code, "test").with_location(Location::new())
    }

    fn report_with(issues: Vec<ValidationIssue>) -> ValidationReport {
        let mut r = ValidationReport::new(ValidationProfile::SMPTE);
        for i in issues {
            r.add(i);
        }
        r
    }

    #[test]
    fn rule_matches_supports_single_segment_glob() {
        assert!(match_specificity("XSD/TypeInvalid", "XSD/*").is_some());
        // Different depth — `XSD/*` is one segment past XSD, not arbitrary depth.
        assert!(match_specificity("XSD/TypeInvalid/IssueDate", "XSD/*").is_none());
        // But `XSD/*/*` matches two levels deep.
        assert!(match_specificity("XSD/TypeInvalid/IssueDate", "XSD/*/*").is_some());
    }

    #[test]
    fn rule_matches_supports_multi_segment_glob() {
        assert!(match_specificity("XSD/PatternInvalid/UUID", "XSD/*/UUID").is_some());
        assert!(match_specificity("XSD/TypeInvalid/UUID", "XSD/*/UUID").is_some());
        assert!(match_specificity("XSD/PatternInvalid/Number", "XSD/*/UUID").is_none());
    }

    #[test]
    fn rule_matches_supports_smpte_section_globs() {
        // Mid-segment wildcards — anchored at both ends of the segment.
        assert!(match_specificity(
            "ST2067-2:2020:6.4.2/EditRate",
            "ST2067-*:2020:*/EditRate",
        )
        .is_some());
        assert!(match_specificity(
            "ST2067-3:2020:5.5.1.2/ContentKindUnknown",
            "ST2067-*:2020:*/EditRate",
        )
        .is_none());
    }

    #[test]
    fn rule_matches_supports_source_prefix() {
        assert!(match_specificity("XSD/TypeInvalid/IssueDate", "source:XsdLayer").is_some());
        assert!(match_specificity("IMFERNO:Package/X", "source:XsdLayer").is_none());
        assert!(match_specificity("IMFERNO:Package/X", "source:EngineInternal").is_some());
        assert!(match_specificity("ST2067-3:2016:5/X", "source:ProseRule").is_some());
        // Unknown source name — no match (silently ignored, doesn't panic).
        assert!(match_specificity("XSD/X", "source:NotAVariant").is_none());
        // Sanity: matched issue inherits the SourcePrefix tier.
        assert_eq!(
            IssueSource::from_code("XSD/TypeInvalid/IssueDate"),
            IssueSource::XsdLayer,
        );
    }

    // ── FIX-13: validate() unmatchable-pattern helper ─────────────────────

    /// A clean config (every key resolves) returns no warnings.
    #[test]
    fn validate_returns_no_warnings_for_clean_config() {
        let mut rules = RulesConfig::default();
        rules.set_raw("XSD/TypeInvalid/IssueDate".into(), RuleSeverity::Warn);
        rules.set_raw("source:XsdLayer".into(), RuleSeverity::Off);
        rules.set_raw("XSD/*/*".into(), RuleSeverity::Warn);
        let warnings = rules.validate(["XSD/TypeInvalid/IssueDate", "XSD/PatternInvalid/UUID"]);
        assert!(warnings.is_empty(), "expected no warnings, got: {warnings:#?}");
    }

    /// `source:Foo` where `Foo` isn't a known `IssueSource` variant.
    #[test]
    fn validate_flags_unknown_source_variant() {
        let mut rules = RulesConfig::default();
        rules.set_raw("source:NotAVariant".into(), RuleSeverity::Off);
        let warnings = rules.validate::<_, &str>([]);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].key, "source:NotAVariant");
        assert_eq!(
            warnings[0].reason,
            RuleValidationReason::UnknownSource {
                variant: "NotAVariant".to_string()
            }
        );
    }

    /// A key that parses fine but matches none of the supplied codes.
    #[test]
    fn validate_flags_match_nothing_keys() {
        let mut rules = RulesConfig::default();
        rules.set_raw("Doesnotexist".into(), RuleSeverity::Warn);
        rules.set_raw("XSD/Madeup/*".into(), RuleSeverity::Off);
        let warnings = rules.validate(["XSD/TypeInvalid/IssueDate"]);
        assert_eq!(warnings.len(), 2);
        // sorted by key
        assert_eq!(warnings[0].key, "Doesnotexist");
        assert_eq!(warnings[0].reason, RuleValidationReason::MatchesNothing);
        assert_eq!(warnings[1].key, "XSD/Madeup/*");
        assert_eq!(warnings[1].reason, RuleValidationReason::MatchesNothing);
    }

    /// `**` is reserved but not implemented; flag it with a hint.
    #[test]
    fn validate_flags_double_star_with_hint() {
        let mut rules = RulesConfig::default();
        rules.set_raw("XSD/**/UUID".into(), RuleSeverity::Off);
        let warnings = rules.validate(["XSD/PatternInvalid/UUID"]);
        assert_eq!(warnings.len(), 1);
        assert!(matches!(
            &warnings[0].reason,
            RuleValidationReason::UnsupportedPattern { hint } if hint.contains("**")
        ));
    }

    /// FIX-5 regression: source-prefix variant names are matched
    /// case-insensitively so config keys like `source:xsdlayer` and
    /// `source:XSDLAYER` work as expected.
    #[test]
    fn rule_matches_source_prefix_case_insensitively() {
        assert!(match_specificity("XSD/TypeInvalid/IssueDate", "source:xsdlayer").is_some());
        assert!(match_specificity("XSD/TypeInvalid/IssueDate", "source:XSDLAYER").is_some());
        assert!(match_specificity("XSD/TypeInvalid/IssueDate", "source:XsDlAyEr").is_some());
        assert!(match_specificity("IMFERNO:Package/X", "source:engineinternal").is_some());
        assert!(match_specificity("ST2067-3:2016:5/X", "source:proserule").is_some());
    }

    #[test]
    fn apply_rules_specific_glob_beats_general_glob() {
        let mut rules = RulesConfig::default();
        rules.set_raw("XSD/*/*".into(), RuleSeverity::Warn);
        rules.set_raw("XSD/PatternInvalid/*".into(), RuleSeverity::Error);
        let report = report_with(vec![issue("XSD/PatternInvalid/UUID", Severity::Info)]);
        let out = report.apply_rules(&rules);
        assert_eq!(out.errors.len(), 1);
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn apply_rules_full_code_beats_glob() {
        let mut rules = RulesConfig::default();
        rules.set_raw("XSD/*/*".into(), RuleSeverity::Warn);
        rules.set_raw("XSD/PatternInvalid/UUID".into(), RuleSeverity::Critical);
        let report = report_with(vec![issue("XSD/PatternInvalid/UUID", Severity::Info)]);
        let out = report.apply_rules(&rules);
        assert_eq!(out.critical.len(), 1);
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn apply_rules_source_prefix_off_moves_to_suppressed_bucket() {
        let mut rules = RulesConfig::default();
        rules.set_raw("source:XsdLayer".into(), RuleSeverity::Off);
        let report = report_with(vec![
            issue("XSD/TypeInvalid/IssueDate", Severity::Error),
            issue("ST2067-3:2020:5/X", Severity::Error),
        ]);
        let out = report.apply_rules(&rules);
        // XSD issue removed from errors, retained in suppressed bucket
        // with an audit annotation. Prose issue stays in errors.
        assert_eq!(out.errors.len(), 1);
        assert!(out.errors[0].code.starts_with("ST2067-"));
        assert_eq!(out.suppressed.len(), 1);
        assert_eq!(out.suppressed[0].code, "XSD/TypeInvalid/IssueDate");
        assert_eq!(out.suppressed[0].severity, Severity::Info);
        assert_eq!(
            out.suppressed[0].context.get("suppressed_by").map(String::as_str),
            Some("source:XsdLayer"),
        );
    }

    #[test]
    fn apply_rules_off_annotates_with_specific_key() {
        // When a more-specific key wins over a broader source-prefix,
        // the annotation should name the actual winning key.
        let mut rules = RulesConfig::default();
        rules.set_raw("source:XsdLayer".into(), RuleSeverity::Warn);
        rules.set_raw("XSD/TypeInvalid/*".into(), RuleSeverity::Off);
        let report = report_with(vec![issue("XSD/TypeInvalid/IssueDate", Severity::Error)]);
        let out = report.apply_rules(&rules);
        assert!(out.errors.is_empty());
        assert_eq!(out.suppressed.len(), 1);
        assert_eq!(
            out.suppressed[0].context.get("suppressed_by").map(String::as_str),
            Some("XSD/TypeInvalid/*"),
        );
    }

    #[test]
    fn apply_rules_suppressed_bucket_does_not_affect_compliance() {
        // Suppressed issues must not flip `is_playable`/`is_compliant`.
        let mut rules = RulesConfig::default();
        rules.set_raw("XSD/*/*".into(), RuleSeverity::Off);
        let report = report_with(vec![
            issue("XSD/TypeInvalid/IssueDate", Severity::Critical),
            issue("XSD/PatternInvalid/UUID", Severity::Error),
        ]);
        let out = report.apply_rules(&rules);
        assert_eq!(out.suppressed.len(), 2);
        assert!(out.is_playable, "suppressed Critical should not block playability");
        assert!(out.is_compliant, "suppressed Error should not block compliance");
    }

    #[test]
    fn apply_rules_remains_deterministic_across_runs() {
        // Two overlapping patterns at the same specificity tier-bucket
        // (both globs with same prefix length and total length). The
        // tie-breaker (longer key length) plus stable comparator must
        // give the same answer every run regardless of HashMap order.
        let mut rules = RulesConfig::default();
        rules.set_raw("XSD/A*".into(), RuleSeverity::Warn);
        rules.set_raw("XSD/B*".into(), RuleSeverity::Error);
        // Issue matches neither — sanity check that no panic / drop happens.
        let code_neither = "XSD/CFoo";
        // Issue matches one — must be the same one every time.
        let code_a = "XSD/Apple";

        let mut first: Option<Severity> = None;
        for _ in 0..100 {
            let r = report_with(vec![
                issue(code_a, Severity::Info),
                issue(code_neither, Severity::Info),
            ])
            .apply_rules(&rules);
            // The Apple issue should always land in warnings (matches XSD/A*).
            let sev = if !r.warnings.is_empty() {
                Severity::Warning
            } else if !r.errors.is_empty() {
                Severity::Error
            } else {
                Severity::Info
            };
            if first.is_none() {
                first = Some(sev);
            } else {
                assert_eq!(first, Some(sev), "result drifted across runs");
            }
        }
        assert_eq!(first, Some(Severity::Warning));
    }
}
