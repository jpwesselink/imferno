//! Per-edition rule model.
//!
//! A [`Rule`] is one check imferno performs, identified by a stable,
//! edition-free [`RuleId`] (`<SPEC>/<Name>`, e.g. `ST2067-3/SegmentDuration`).
//! Its [`EditionEntry`] table holds, for every edition of the spec imferno
//! knows, the clause the rule comes from (or `None` when the edition does
//! not contain the rule), its normative strength and its default severity.
//!
//! A finding is created with [`crate::ValidationIssue::from_rule`], passing
//! the edition actually applied; the finding's display code is derived from
//! the table row: `SPEC:EDITION:CLAUSE/Name`. Fixing a wrong clause or
//! edition is a table edit — the rule id does not change.
//!
//! Every spec has one table module under [`crate::catalogue`].

use std::fmt;

use serde::{Deserialize, Serialize};

use super::{Category, IssueSource, Severity};

// ─────────────────────────────────────────────────────────────────────────────
// Spec
// ─────────────────────────────────────────────────────────────────────────────

/// A document whose rules imferno checks.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Spec {
    /// SMPTE ST 377-1 — MXF File Format.
    #[serde(rename = "ST377-1")]
    St377_1,
    /// SMPTE ST 377-4 — MXF Multichannel Audio Labeling.
    #[serde(rename = "ST377-4")]
    St377_4,
    /// SMPTE ST 429-8 — Packing List.
    #[serde(rename = "ST429-8")]
    St429_8,
    /// SMPTE ST 429-9 — Asset Mapping and File Segmentation.
    #[serde(rename = "ST429-9")]
    St429_9,
    /// SMPTE ST 2067-2 — IMF Core Constraints.
    #[serde(rename = "ST2067-2")]
    St2067_2,
    /// SMPTE ST 2067-3 — IMF Composition Playlist.
    #[serde(rename = "ST2067-3")]
    St2067_3,
    /// SMPTE ST 2067-9 — IMF Sidecar Composition Map.
    #[serde(rename = "ST2067-9")]
    St2067_9,
    /// SMPTE ST 2067-21 — IMF Application #2E.
    #[serde(rename = "ST2067-21")]
    St2067_21,
    /// SMPTE ST 2067-201 — IMF IAB Level 0 Plug-in.
    #[serde(rename = "ST2067-201")]
    St2067_201,
    /// SMPTE ST 2067-202 — IMF ISXD Plug-in.
    #[serde(rename = "ST2067-202")]
    St2067_202,
    /// The XML Schema layer: findings of the runtime XSD validator against
    /// the vendored schema of a document's namespace.
    #[serde(rename = "XSD")]
    Xsd,
    /// imferno's own checks (parse failures, package scanning) that no
    /// single SMPTE clause defines.
    #[serde(rename = "IMFERNO")]
    Imferno,
}

impl Spec {
    /// Every spec, in catalogue order.
    pub const ALL: &'static [Spec] = &[
        Spec::St377_1,
        Spec::St377_4,
        Spec::St429_8,
        Spec::St429_9,
        Spec::St2067_2,
        Spec::St2067_3,
        Spec::St2067_9,
        Spec::St2067_21,
        Spec::St2067_201,
        Spec::St2067_202,
        Spec::Xsd,
        Spec::Imferno,
    ];

    /// The spec's identifier as used in rule ids and codes (`ST2067-3`).
    pub const fn id(self) -> &'static str {
        match self {
            Spec::St377_1 => "ST377-1",
            Spec::St377_4 => "ST377-4",
            Spec::St429_8 => "ST429-8",
            Spec::St429_9 => "ST429-9",
            Spec::St2067_2 => "ST2067-2",
            Spec::St2067_3 => "ST2067-3",
            Spec::St2067_9 => "ST2067-9",
            Spec::St2067_21 => "ST2067-21",
            Spec::St2067_201 => "ST2067-201",
            Spec::St2067_202 => "ST2067-202",
            Spec::Xsd => "XSD",
            Spec::Imferno => "IMFERNO",
        }
    }

    /// Human-readable title.
    pub const fn title(self) -> &'static str {
        match self {
            Spec::St377_1 => "SMPTE ST 377-1 — MXF File Format",
            Spec::St377_4 => "SMPTE ST 377-4 — MXF Multichannel Audio Labeling",
            Spec::St429_8 => "SMPTE ST 429-8 — Packing List",
            Spec::St429_9 => "SMPTE ST 429-9 — Asset Mapping and File Segmentation",
            Spec::St2067_2 => "SMPTE ST 2067-2 — Core Constraints",
            Spec::St2067_3 => "SMPTE ST 2067-3 — Composition Playlist",
            Spec::St2067_9 => "SMPTE ST 2067-9 — Sidecar Composition Map",
            Spec::St2067_21 => "SMPTE ST 2067-21 — Application #2E",
            Spec::St2067_201 => "SMPTE ST 2067-201 — IAB Level 0 Plug-in",
            Spec::St2067_202 => "SMPTE ST 2067-202 — ISXD Plug-in",
            Spec::Xsd => "XML Schema validation",
            Spec::Imferno => "imferno engine",
        }
    }

    /// Parse a spec id (`ST2067-3`).
    pub fn from_id(id: &str) -> Option<Spec> {
        Spec::ALL.iter().copied().find(|s| s.id() == id)
    }

    /// The authority that produces findings of this spec.
    pub const fn source(self) -> IssueSource {
        match self {
            Spec::Xsd => IssueSource::XsdLayer,
            Spec::Imferno => IssueSource::EngineInternal,
            _ => IssueSource::ProseRule,
        }
    }
}

impl fmt::Display for Spec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Strength
// ─────────────────────────────────────────────────────────────────────────────

/// Normative strength of a rule in an edition's text.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Strength {
    /// A requirement ("shall").
    Shall,
    /// A recommendation ("should").
    Should,
    /// A permission or an observation ("may").
    May,
    /// Deprecated in this edition.
    Deprecated,
}

impl Strength {
    /// The default severity a rule of this strength gets:
    /// shall → Error, should → Warning, may → Info, deprecated → Warning.
    pub const fn default_severity(self) -> Severity {
        match self {
            Strength::Shall => Severity::Error,
            Strength::Should => Severity::Warning,
            Strength::May => Severity::Info,
            Strength::Deprecated => Severity::Warning,
        }
    }

    /// The strength implied by a severity (inverse of
    /// [`Strength::default_severity`]; Critical → shall). Used to derive the
    /// initial strength of migrated rules until it is read from the spec text.
    pub const fn from_severity(severity: Severity) -> Strength {
        match severity {
            Severity::Critical | Severity::Error => Strength::Shall,
            Severity::Warning => Strength::Should,
            Severity::Info => Strength::May,
        }
    }
}

impl fmt::Display for Strength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Strength::Shall => "shall",
            Strength::Should => "should",
            Strength::May => "may",
            Strength::Deprecated => "deprecated",
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Editions
// ─────────────────────────────────────────────────────────────────────────────

/// A published edition of one spec. Each spec has its own edition type, so
/// a rule can only be reported against an edition of its own spec.
pub trait Edition: Copy + Eq + fmt::Debug + Send + Sync + 'static {
    /// The spec this is an edition of.
    const SPEC: Spec;
    /// Every edition imferno knows, oldest first.
    fn all() -> &'static [Self];
    /// The edition's label in codes (`"2016"`); empty for single-edition
    /// pseudo-specs such as `IMFERNO`.
    fn label(self) -> &'static str;
}

/// Declare a per-spec edition enum.
macro_rules! editions {
    (
        $(#[$meta:meta])*
        pub enum $name:ident for $spec:path {
            $( $(#[$vmeta:meta])* $variant:ident => $label:literal, )+
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum $name {
            $( $(#[$vmeta])* $variant, )+
        }

        impl $name {
            /// Every edition, oldest first.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)+];
        }

        impl $crate::diagnostics::rule::Edition for $name {
            const SPEC: $crate::diagnostics::rule::Spec = $spec;
            fn all() -> &'static [Self] {
                Self::ALL
            }
            fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label,)+
                }
            }
        }
    };
}
pub(crate) use editions;

// ─────────────────────────────────────────────────────────────────────────────
// Rule tables
// ─────────────────────────────────────────────────────────────────────────────

/// A documented interpretation of a rule (working-group issue, plugfest
/// finding) that the check follows.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Interpretation {
    /// What imferno does and why.
    pub text: &'static str,
    /// Link to the issue or finding.
    pub url: Option<&'static str>,
}

/// One row of a rule's edition table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditionEntry<E: Edition> {
    /// The edition this row describes.
    pub edition: E,
    /// Clause of this edition the rule comes from; `None` when the edition
    /// does not contain the rule. Empty for pseudo-specs without clauses.
    pub clause: Option<&'static str>,
    /// Normative strength in this edition.
    pub strength: Strength,
    /// Severity a finding gets unless a rules config overrides it.
    pub default_severity: Severity,
    /// Why `default_severity` differs from `strength.default_severity()`.
    /// Required when they differ.
    pub severity_reason: Option<&'static str>,
    /// Working-group / plugfest interpretation the check follows.
    pub interpretation: Option<Interpretation>,
    /// `true` only when this row (clause, or absence) was checked against
    /// this edition's text. Defaults to `false`.
    pub verified: bool,
    /// The clause is known or suspected to be wrong (the C1 list); to be
    /// fixed in the table, without renaming.
    pub suspect: bool,
    /// `false` while `strength` is derived from the default severity rather
    /// than read from the edition's text.
    pub strength_verified: bool,
}

/// A row for an edition that contains the rule. Strength is derived from
/// `severity`; neither strength nor clause is marked verified.
pub const fn row<E: Edition>(
    edition: E,
    clause: &'static str,
    severity: Severity,
) -> EditionEntry<E> {
    EditionEntry {
        edition,
        clause: Some(clause),
        strength: Strength::from_severity(severity),
        default_severity: severity,
        severity_reason: None,
        interpretation: None,
        verified: false,
        suspect: false,
        strength_verified: false,
    }
}

/// A row for an edition that does not contain the rule.
pub const fn absent<E: Edition>(edition: E) -> EditionEntry<E> {
    EditionEntry {
        edition,
        clause: None,
        strength: Strength::May,
        default_severity: Severity::Info,
        severity_reason: None,
        interpretation: None,
        verified: false,
        suspect: false,
        strength_verified: false,
    }
}

impl<E: Edition> EditionEntry<E> {
    /// Mark the row as checked against this edition's text.
    pub const fn verified(mut self) -> Self {
        self.verified = true;
        self
    }

    /// Mark the clause as known or suspected to be wrong.
    pub const fn suspect(mut self) -> Self {
        self.suspect = true;
        self.verified = false;
        self
    }

    /// Give the reason the default severity differs from the strength.
    pub const fn reason(mut self, reason: &'static str) -> Self {
        self.severity_reason = Some(reason);
        self
    }

    /// Set the normative strength (read from the text).
    pub const fn strength(mut self, strength: Strength) -> Self {
        self.strength = strength;
        self.strength_verified = true;
        self
    }

    /// Record the interpretation the check follows.
    pub const fn interpret(mut self, text: &'static str, url: Option<&'static str>) -> Self {
        self.interpretation = Some(Interpretation { text, url });
        self
    }
}

/// A rule: one check, with one row per edition of its spec.
#[derive(Debug)]
pub struct Rule<E: Edition> {
    /// Name, unique within the spec (`SegmentDuration`).
    pub name: &'static str,
    /// Category bucket.
    pub category: Category,
    /// What the rule checks.
    pub description: &'static str,
    /// A minimal illustration of a violation.
    pub example: Option<&'static str>,
    /// One row per edition in `E::all()`, in that order.
    pub editions: &'static [EditionEntry<E>],
}

impl<E: Edition> Rule<E> {
    /// The spec this rule belongs to.
    pub fn spec(&self) -> Spec {
        E::SPEC
    }

    /// Stable, edition-free id: `<SPEC>/<Name>`.
    pub fn id(&self) -> RuleId {
        RuleId(format!("{}/{}", E::SPEC.id(), self.name))
    }

    /// The table row for `edition`.
    pub fn entry(&self, edition: E) -> Option<&EditionEntry<E>> {
        self.editions.iter().find(|e| e.edition == edition)
    }

    /// The display code for `edition` (`ST2067-3:2016:7.2.2/SegmentDuration`),
    /// or `None` when the edition does not contain the rule.
    pub fn code(&self, edition: E) -> Option<String> {
        let clause = self.entry(edition)?.clause?;
        Some(display_code(E::SPEC, edition.label(), clause, self.name))
    }

    /// Whether `edition` contains the rule.
    pub fn in_edition(&self, edition: E) -> bool {
        self.entry(edition).is_some_and(|e| e.clause.is_some())
    }
}

/// Rules are equal when they are the same rule (same spec and name).
impl<E: Edition> PartialEq for Rule<E> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<E: Edition> Eq for Rule<E> {}

/// `SPEC:EDITION:CLAUSE/Name`, dropping empty parts.
pub fn display_code(spec: Spec, edition: &str, clause: &str, name: &str) -> String {
    let mut s = String::from(spec.id());
    for part in [edition, clause] {
        if !part.is_empty() {
            s.push(':');
            s.push_str(part);
        }
    }
    s.push('/');
    s.push_str(name);
    s
}

/// Stable, edition-free rule identifier: `<SPEC>/<Name>`. Also the SARIF
/// `ruleId`.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RuleId(pub String);

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Type-erased catalogue view
// ─────────────────────────────────────────────────────────────────────────────

/// Object-safe view of a [`Rule`] of any spec, for catalogue iteration.
pub trait AnyRule: Sync {
    /// Spec of the rule.
    fn spec(&self) -> Spec;
    /// Name, unique within the spec.
    fn name(&self) -> &'static str;
    /// Category.
    fn category(&self) -> &Category;
    /// Description.
    fn description(&self) -> &'static str;
    /// Example.
    fn example(&self) -> Option<&'static str>;
    /// Edition rows, with editions as labels.
    fn edition_rows(&self) -> Vec<EditionInfo>;
    /// Labels of every edition of the spec, oldest first.
    fn edition_labels(&self) -> Vec<&'static str>;

    /// `<SPEC>/<Name>`.
    fn id(&self) -> RuleId {
        RuleId(format!("{}/{}", self.spec().id(), self.name()))
    }

    /// Serializable description of the rule.
    fn info(&self) -> RuleInfo {
        RuleInfo {
            id: self.id(),
            name: self.name().to_string(),
            spec: self.spec(),
            category: self.category().clone(),
            description: self.description().to_string(),
            example: self.example().map(str::to_string),
            editions: self.edition_rows(),
        }
    }
}

impl<E: Edition> AnyRule for Rule<E> {
    fn spec(&self) -> Spec {
        E::SPEC
    }
    fn name(&self) -> &'static str {
        self.name
    }
    fn category(&self) -> &Category {
        &self.category
    }
    fn description(&self) -> &'static str {
        self.description
    }
    fn example(&self) -> Option<&'static str> {
        self.example
    }
    fn edition_rows(&self) -> Vec<EditionInfo> {
        self.editions
            .iter()
            .map(|e| EditionInfo {
                edition: e.edition.label().to_string(),
                clause: e.clause.map(str::to_string),
                code: e
                    .clause
                    .map(|c| display_code(E::SPEC, e.edition.label(), c, self.name)),
                strength: e.strength,
                default_severity: e.default_severity,
                severity_reason: e.severity_reason.map(str::to_string),
                interpretation: e.interpretation.map(|i| InterpretationInfo {
                    text: i.text.to_string(),
                    url: i.url.map(str::to_string),
                }),
                verified: e.verified,
                suspect: e.suspect,
                strength_verified: e.strength_verified,
            })
            .collect()
    }
    fn edition_labels(&self) -> Vec<&'static str> {
        E::all().iter().map(|e| e.label()).collect()
    }
}

/// Serializable rule description (`listRules()` entry).
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleInfo {
    /// `<SPEC>/<Name>` — usable as a rules-config key (all editions).
    pub id: RuleId,
    /// Name, unique within the spec; usable as a rules-config key (every
    /// rule with that name).
    pub name: String,
    /// Spec the rule belongs to.
    pub spec: Spec,
    /// Category.
    pub category: Category,
    /// What the rule checks.
    pub description: String,
    /// A minimal illustration of a violation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    /// One row per edition of the spec imferno knows, oldest first.
    pub editions: Vec<EditionInfo>,
}

/// Serializable edition row of a [`RuleInfo`].
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditionInfo {
    /// Edition label (`"2016"`; empty for single-edition pseudo-specs).
    pub edition: String,
    /// Clause in this edition; absent when the edition does not contain the rule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clause: Option<String>,
    /// Display code findings carry for this edition
    /// (`SPEC:EDITION:CLAUSE/Name`); absent when the edition does not
    /// contain the rule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Normative strength.
    pub strength: Strength,
    /// Default severity.
    pub default_severity: Severity,
    /// Why the default severity differs from the strength's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity_reason: Option<String>,
    /// Interpretation the check follows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<InterpretationInfo>,
    /// `true` only when the row was checked against this edition's text.
    pub verified: bool,
    /// The clause is known or suspected to be wrong.
    pub suspect: bool,
    /// `false` while the strength is derived from the default severity.
    pub strength_verified: bool,
}

/// Serializable [`Interpretation`].
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterpretationInfo {
    /// What imferno does and why.
    pub text: String,
    /// Link to the issue or finding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}
