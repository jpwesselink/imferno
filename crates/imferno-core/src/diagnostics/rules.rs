//! ESLint-style per-rule severity configuration.
//!
//! A [`RulesConfig`] maps rule identifiers to [`RuleSeverity`] overrides.
//! Rules can be identified by their full normalised code
//! (`"ST2067-3:2020:7.2.2/SegmentDuration"`) or by their short
//! suffix (`"SegmentDuration"`).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::diagnostics::codes::ValidationCode;
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
}

fn rule_matches(code: &str, key: &str) -> bool {
    // Full normalised code match first, then suffix match.
    code == key || code.rsplit('/').next() == Some(key)
}

impl ValidationReport {
    /// Apply ESLint-style per-rule severity overrides.
    ///
    /// Issues whose rule matches a [`RuleSeverity::Off`] entry are removed.
    /// All other matching issues have their severity remapped and re-bucketed.
    /// `is_playable` and `is_compliant` are recomputed from the updated buckets.
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
            let override_sev = rules
                .iter()
                .find(|(k, _)| rule_matches(&issue.code, k))
                .map(|(_, v)| v);

            match override_sev {
                Some(RuleSeverity::Off) => {} // drop
                Some(RuleSeverity::Info) => {
                    issue.severity = Severity::Info;
                    self.info.push(issue);
                }
                Some(RuleSeverity::Warn) => {
                    issue.severity = Severity::Warning;
                    self.warnings.push(issue);
                }
                Some(RuleSeverity::Error) => {
                    issue.severity = Severity::Error;
                    self.errors.push(issue);
                }
                Some(RuleSeverity::Critical) => {
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
