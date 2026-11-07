//! ESLint-style per-rule severity configuration.
//!
//! A [`RulesConfig`] maps keys to [`RuleSeverity`] overrides. When several
//! keys match a finding, the most specific one wins:
//!
//! 1. `ST2067-3:2016/SegmentDuration` — the rule in one edition. The full
//!    display code of a finding (`ST2067-3:2016:7.2.2/SegmentDuration`) is
//!    accepted too, so a code copied from a report works as a key.
//! 2. `ST2067-3/SegmentDuration` — the rule in every edition (the rule id).
//! 3. `SegmentDuration` — every rule with that name, in any spec.
//! 4. `ST2067-3:2016:*` — every rule of one spec edition.
//! 5. `ST2067-3:*` — every rule of a spec.
//! 6. `source:XsdLayer` / `source:ProseRule` / `source:EngineInternal` —
//!    every finding of that [`IssueSource`].
//!
//! Keys are matched exactly (no other wildcards). [`RulesConfig::validate`]
//! reports keys that match no rule of the catalogue.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::diagnostics::rule::display_code;
use crate::diagnostics::{Edition, IssueSource, Rule, ValidationIssue};
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
/// config-load time to catch typos before any validation work runs.
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
    /// The key matches no rule, edition or spec of the catalogue.
    MatchesNothing,
    /// The key uses syntax the matcher doesn't support (e.g. `XSD/*`).
    UnsupportedPattern { hint: String },
}

/// ESLint-style per-rule severity overrides (see the module docs for the
/// key forms and their precedence). An empty map (the default) is a no-op.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RulesConfig(HashMap<String, RuleSeverity>);

/// How specific a matching key is; greater wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Specificity {
    Source,
    Spec,
    SpecEdition,
    Name,
    Rule,
    RuleEdition,
}

/// Every key form that selects `issue`, most specific first.
fn candidate_keys(issue: &ValidationIssue) -> Vec<(Specificity, String)> {
    let spec = issue.spec.id();
    let name = issue.rule_name();
    let mut keys = Vec::with_capacity(7);
    if !issue.edition.is_empty() {
        keys.push((
            Specificity::RuleEdition,
            format!("{spec}:{}/{name}", issue.edition),
        ));
    }
    keys.push((Specificity::RuleEdition, issue.code.clone()));
    keys.push((Specificity::Rule, issue.rule.0.clone()));
    keys.push((Specificity::Name, name.to_string()));
    if !issue.edition.is_empty() {
        keys.push((
            Specificity::SpecEdition,
            format!("{spec}:{}:*", issue.edition),
        ));
    }
    keys.push((Specificity::Spec, format!("{spec}:*")));
    keys
}

impl RulesConfig {
    /// Set the severity of `rule` in every edition (key `SPEC/Name`).
    pub fn set<E: Edition>(&mut self, rule: &Rule<E>, severity: RuleSeverity) {
        self.0.insert(rule.id().0, severity);
    }

    /// Set the severity of `rule` in one edition (key `SPEC:EDITION/Name`).
    pub fn set_in_edition<E: Edition>(
        &mut self,
        rule: &Rule<E>,
        edition: E,
        severity: RuleSeverity,
    ) {
        let key = if edition.label().is_empty() {
            rule.id().0
        } else {
            format!("{}:{}/{}", E::SPEC.id(), edition.label(), rule.name)
        };
        self.0.insert(key, severity);
    }

    /// Set severity by raw string key (any of the key forms).
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

    /// Iterate over configured overrides.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &RuleSeverity)> {
        self.0.iter()
    }

    /// The override that applies to `issue` and the key it came from.
    fn lookup(&self, issue: &ValidationIssue) -> Option<(&str, &RuleSeverity)> {
        let mut best: Option<(Specificity, &str, &RuleSeverity)> = None;
        for (spec, key) in candidate_keys(issue) {
            if let Some((k, v)) = self.0.get_key_value(&key) {
                if best.is_none_or(|(s, _, _)| spec > s) {
                    best = Some((spec, k.as_str(), v));
                }
            }
        }
        if best.is_none() {
            // Source keys: lowest precedence, case-insensitive variant.
            let mut sources: Vec<_> = self
                .0
                .iter()
                .filter(|(k, _)| {
                    k.strip_prefix("source:")
                        .and_then(parse_source)
                        .is_some_and(|s| s == issue.source)
                })
                .collect();
            sources.sort_by(|a, b| a.0.cmp(b.0));
            if let Some((k, v)) = sources.into_iter().next() {
                best = Some((Specificity::Source, k.as_str(), v));
            }
        }
        best.map(|(_, k, v)| (k, v))
    }

    /// Check the configured keys against the rule catalogue and return a
    /// warning for every key that selects nothing. Useful at config-load
    /// time so operators get fast feedback on typos.
    pub fn validate(&self) -> Vec<RuleValidationWarning> {
        let known = known_keys();
        let mut warnings = Vec::new();
        for key in self.0.keys() {
            let reason = if let Some(rest) = key.strip_prefix("source:") {
                parse_source(rest)
                    .is_none()
                    .then(|| RuleValidationReason::UnknownSource {
                        variant: rest.to_string(),
                    })
            } else if known.contains(key.as_str()) {
                None
            } else if key.contains('*') && !key.ends_with(":*") {
                Some(RuleValidationReason::UnsupportedPattern {
                    hint: "only `SPEC:*` and `SPEC:EDITION:*` wildcards are supported".to_string(),
                })
            } else {
                Some(RuleValidationReason::MatchesNothing)
            };
            if let Some(reason) = reason {
                warnings.push(RuleValidationWarning {
                    key: key.clone(),
                    reason,
                });
            }
        }
        warnings.sort_by(|a, b| a.key.cmp(&b.key));
        warnings
    }
}

/// Every key that selects at least one (rule, edition) of the catalogue.
fn known_keys() -> HashSet<String> {
    let mut keys = HashSet::new();
    for rule in crate::catalogue::all() {
        let spec = rule.spec().id();
        let name = rule.name();
        keys.insert(rule.id().0);
        keys.insert(name.to_string());
        keys.insert(format!("{spec}:*"));
        for row in rule.edition_rows() {
            let Some(clause) = &row.clause else { continue };
            keys.insert(display_code(rule.spec(), &row.edition, clause, name));
            if !row.edition.is_empty() {
                keys.insert(format!("{spec}:{}/{name}", row.edition));
                keys.insert(format!("{spec}:{}:*", row.edition));
            }
        }
    }
    keys
}

/// Parse a source-prefix variant name (e.g. `XsdLayer`), case-insensitively.
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
    /// Issues whose most specific matching key is [`RuleSeverity::Off`] move
    /// to the `suppressed` bucket (annotated with `context["suppressed_by"]`).
    /// All other matching issues have their severity remapped and re-bucketed.
    /// `is_playable` and `is_compliant` are recomputed from the updated
    /// buckets: a package is compliant when no finding has an effective
    /// severity of Error or above.
    ///
    /// An empty [`RulesConfig`] is a no-op.
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
            let matched = rules.lookup(&issue).map(|(k, v)| (k.to_string(), v));
            match matched {
                Some((key, RuleSeverity::Off)) => {
                    issue.context.insert(
                        "original_severity".to_string(),
                        format!("{:?}", issue.severity),
                    );
                    issue.context.insert("suppressed_by".to_string(), key);
                    issue.severity = Severity::Info;
                    self.suppressed.push(issue);
                    continue;
                }
                Some((_, RuleSeverity::Info)) => issue.severity = Severity::Info,
                Some((_, RuleSeverity::Warn)) => issue.severity = Severity::Warning,
                Some((_, RuleSeverity::Error)) => issue.severity = Severity::Error,
                Some((_, RuleSeverity::Critical)) => issue.severity = Severity::Critical,
                None => {}
            }
            match issue.severity {
                Severity::Critical => self.critical.push(issue),
                Severity::Error => self.errors.push(issue),
                Severity::Warning => self.warnings.push(issue),
                Severity::Info => self.info.push(issue),
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
    use crate::catalogue::imferno::{self, ImfernoEdition};
    use crate::catalogue::st2067_2::{self, St2067_2Edition};
    use crate::catalogue::st2067_3::{self, St2067_3Edition};
    use crate::catalogue::xsd::{self, XsdSchema};
    use crate::diagnostics::ValidationProfile;

    fn seg(edition: St2067_3Edition) -> ValidationIssue {
        ValidationIssue::from_rule(&st2067_3::SegmentDuration, edition, "x")
    }

    fn one(issue: ValidationIssue, rules: &RulesConfig) -> ValidationReport {
        let mut r = ValidationReport::new(ValidationProfile::SMPTE);
        r.add(issue);
        r.apply_rules(rules)
    }

    fn rules(pairs: &[(&str, RuleSeverity)]) -> RulesConfig {
        let mut r = RulesConfig::default();
        for (k, v) in pairs {
            r.set_raw(k.to_string(), v.clone());
        }
        r
    }

    fn severity(issue: ValidationIssue, rules: &RulesConfig) -> Option<Severity> {
        let r = one(issue, rules);
        [
            (Severity::Critical, r.critical.len()),
            (Severity::Error, r.errors.len()),
            (Severity::Warning, r.warnings.len()),
            (Severity::Info, r.info.len()),
        ]
        .into_iter()
        .find(|(_, n)| *n > 0)
        .map(|(s, _)| s)
    }

    // ── one test per precedence level ─────────────────────────────────

    #[test]
    fn level1_rule_in_one_edition() {
        let r = rules(&[("ST2067-3:2016/SegmentDuration", RuleSeverity::Warn)]);
        assert_eq!(
            severity(seg(St2067_3Edition::V2016), &r),
            Some(Severity::Warning)
        );
        // Other edition untouched.
        assert_eq!(
            severity(seg(St2067_3Edition::V2013), &r),
            Some(Severity::Error)
        );
    }

    #[test]
    fn level1_accepts_the_full_display_code() {
        let r = rules(&[("ST2067-3:2016:7.2.2/SegmentDuration", RuleSeverity::Warn)]);
        assert_eq!(
            severity(seg(St2067_3Edition::V2016), &r),
            Some(Severity::Warning)
        );
        assert_eq!(
            severity(seg(St2067_3Edition::V2013), &r),
            Some(Severity::Error)
        );
    }

    #[test]
    fn level2_rule_in_every_edition() {
        let r = rules(&[("ST2067-3/SegmentDuration", RuleSeverity::Info)]);
        assert_eq!(
            severity(seg(St2067_3Edition::V2013), &r),
            Some(Severity::Info)
        );
        assert_eq!(
            severity(seg(St2067_3Edition::V2016), &r),
            Some(Severity::Info)
        );
    }

    #[test]
    fn level3_name_in_any_spec() {
        // `ParseError` exists in ST 377-1 and IMFERNO.
        let r = rules(&[("ParseError", RuleSeverity::Info)]);
        let engine = ValidationIssue::from_rule(&imferno::ParseError, ImfernoEdition::Current, "x");
        assert_eq!(severity(engine, &r), Some(Severity::Info));
        let mxf = ValidationIssue::from_rule(
            &crate::catalogue::st377_1::ParseError,
            crate::catalogue::st377_1::St377_1Edition::V2011,
            "x",
        );
        assert_eq!(severity(mxf, &r), Some(Severity::Info));
    }

    #[test]
    fn level4_whole_spec_edition() {
        let r = rules(&[("ST2067-3:2016:*", RuleSeverity::Warn)]);
        assert_eq!(
            severity(seg(St2067_3Edition::V2016), &r),
            Some(Severity::Warning)
        );
        assert_eq!(
            severity(seg(St2067_3Edition::V2013), &r),
            Some(Severity::Error)
        );
    }

    #[test]
    fn level5_whole_spec() {
        let r = rules(&[("ST2067-3:*", RuleSeverity::Warn)]);
        assert_eq!(
            severity(seg(St2067_3Edition::V2013), &r),
            Some(Severity::Warning)
        );
        let other =
            ValidationIssue::from_rule(&st2067_2::FileNotFound, St2067_2Edition::V2020, "x");
        assert_eq!(severity(other, &r), Some(Severity::Error));
    }

    #[test]
    fn level6_source() {
        let r = rules(&[("source:xsdlayer", RuleSeverity::Off)]);
        let out = one(
            ValidationIssue::from_rule(&xsd::TypeInvalid, XsdSchema::Cpl2016, "x"),
            &r,
        );
        assert_eq!(out.suppressed.len(), 1);
        assert_eq!(
            out.suppressed[0]
                .context
                .get("suppressed_by")
                .map(String::as_str),
            Some("source:xsdlayer")
        );
        assert!(out.is_compliant);
    }

    #[test]
    fn most_specific_key_wins() {
        let r = rules(&[
            ("source:ProseRule", RuleSeverity::Off),
            ("ST2067-3:*", RuleSeverity::Critical),
            ("ST2067-3:2016:*", RuleSeverity::Error),
            ("SegmentDuration", RuleSeverity::Info),
            ("ST2067-3/SegmentDuration", RuleSeverity::Warn),
            ("ST2067-3:2016/SegmentDuration", RuleSeverity::Critical),
        ]);
        assert_eq!(
            severity(seg(St2067_3Edition::V2016), &r),
            Some(Severity::Critical)
        );
        // 2013: level 2 is the most specific match.
        assert_eq!(
            severity(seg(St2067_3Edition::V2013), &r),
            Some(Severity::Warning)
        );
        // Each level, removed in turn, falls back to the next one.
        let r = rules(&[
            ("ST2067-3:*", RuleSeverity::Critical),
            ("ST2067-3:2016:*", RuleSeverity::Info),
            ("SegmentDuration", RuleSeverity::Warn),
        ]);
        assert_eq!(
            severity(seg(St2067_3Edition::V2016), &r),
            Some(Severity::Warning)
        );
        let r = rules(&[
            ("ST2067-3:*", RuleSeverity::Critical),
            ("ST2067-3:2016:*", RuleSeverity::Info),
        ]);
        assert_eq!(
            severity(seg(St2067_3Edition::V2016), &r),
            Some(Severity::Info)
        );
    }

    #[test]
    fn off_moves_to_suppressed_and_keeps_compliance() {
        let r = rules(&[("ST2067-3/SegmentDuration", RuleSeverity::Off)]);
        let out = one(seg(St2067_3Edition::V2016), &r);
        assert!(out.errors.is_empty());
        assert_eq!(out.suppressed.len(), 1);
        assert_eq!(out.suppressed[0].severity, Severity::Info);
        assert!(out.is_compliant && out.is_playable);
    }

    #[test]
    fn set_helpers_write_level_1_and_2_keys() {
        let mut r = RulesConfig::default();
        r.set(&st2067_3::SegmentDuration, RuleSeverity::Off);
        r.set_in_edition(
            &st2067_3::SegmentDuration,
            St2067_3Edition::V2013,
            RuleSeverity::Warn,
        );
        let keys: HashSet<&String> = r.iter().map(|(k, _)| k).collect();
        assert!(keys.contains(&"ST2067-3/SegmentDuration".to_string()));
        assert!(keys.contains(&"ST2067-3:2013/SegmentDuration".to_string()));
        let json = serde_json::to_string(&r).unwrap();
        let back: RulesConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 2);
    }

    #[test]
    fn validate_accepts_every_key_form() {
        let r = rules(&[
            ("ST2067-3:2016/SegmentDuration", RuleSeverity::Warn),
            ("ST2067-3:2016:7.2.2/SegmentDuration", RuleSeverity::Warn),
            ("ST2067-3/SegmentDuration", RuleSeverity::Warn),
            ("SegmentDuration", RuleSeverity::Warn),
            ("ST2067-3:2016:*", RuleSeverity::Warn),
            ("ST2067-3:*", RuleSeverity::Warn),
            ("IMFERNO/ParseError", RuleSeverity::Warn),
            ("IMFERNO:Package/ParseError", RuleSeverity::Warn),
            ("XSD:CPL-2016/TypeInvalid", RuleSeverity::Warn),
            ("source:XsdLayer", RuleSeverity::Off),
        ]);
        assert_eq!(r.validate(), vec![]);
    }

    #[test]
    fn validate_flags_unknown_keys() {
        let r = rules(&[
            ("source:NotAVariant", RuleSeverity::Off),
            ("Doesnotexist", RuleSeverity::Warn),
            // An edition the rule does not have.
            ("ST2067-3:2020/SegmentDuration", RuleSeverity::Warn),
            ("XSD/*", RuleSeverity::Off),
        ]);
        let w = r.validate();
        assert_eq!(w.len(), 4, "{w:#?}");
        assert_eq!(w[0].key, "Doesnotexist");
        assert_eq!(w[0].reason, RuleValidationReason::MatchesNothing);
        assert_eq!(w[1].reason, RuleValidationReason::MatchesNothing);
        assert!(matches!(
            w[2].reason,
            RuleValidationReason::UnsupportedPattern { .. }
        ));
        assert_eq!(
            w[3].reason,
            RuleValidationReason::UnknownSource {
                variant: "NotAVariant".to_string()
            }
        );
    }
}
