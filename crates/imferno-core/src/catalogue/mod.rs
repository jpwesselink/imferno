//! The rule catalogue: one table per spec.
//!
//! Each module declares its spec's edition type and one `static` [`Rule`]
//! per check, with a row per edition (see [`crate::diagnostics::rule`]).
//! Findings are created with [`crate::ValidationIssue::from_rule`].
//!
//! [`Rule`]: crate::diagnostics::rule::Rule

pub mod imferno;
pub mod st2067_2;
pub mod st2067_201;
pub mod st2067_202;
pub mod st2067_21;
pub mod st2067_3;
pub mod st2067_5;
pub mod st2067_9;
pub mod st377_1;
pub mod st377_4;
pub mod st429_8;
pub mod st429_9;
pub mod xsd;

use crate::diagnostics::rule::{AnyRule, Edition, RuleInfo, Spec};

impl Edition for crate::validation::App2EEdition {
    const SPEC: Spec = Spec::St2067_21;
    fn all() -> &'static [Self] {
        Self::ALL
    }
    fn label(self) -> &'static str {
        match self {
            Self::V2014 => "2014",
            Self::V2016 => "2016",
            Self::V2020 => "2020",
            Self::V2023 => "2023",
            Self::V2026 => "2026",
        }
    }
}

/// The rule tables of every spec, in catalogue order.
pub fn tables() -> [(Spec, &'static [&'static dyn AnyRule]); 13] {
    [
        (Spec::St377_1, st377_1::ALL),
        (Spec::St377_4, st377_4::ALL),
        (Spec::St429_8, st429_8::ALL),
        (Spec::St429_9, st429_9::ALL),
        (Spec::St2067_2, st2067_2::ALL),
        (Spec::St2067_3, st2067_3::ALL),
        (Spec::St2067_5, st2067_5::ALL),
        (Spec::St2067_9, st2067_9::ALL),
        (Spec::St2067_21, st2067_21::ALL),
        (Spec::St2067_201, st2067_201::ALL),
        (Spec::St2067_202, st2067_202::ALL),
        (Spec::Xsd, xsd::ALL),
        (Spec::Imferno, imferno::ALL),
    ]
}

/// Every rule of every spec.
pub fn all() -> impl Iterator<Item = &'static dyn AnyRule> {
    tables()
        .into_iter()
        .flat_map(|(_, rules)| rules.iter().copied())
}

/// Serializable description of every rule (`listRules()`).
pub fn rule_infos() -> Vec<RuleInfo> {
    all().map(|r| r.info()).collect()
}

/// Look up a rule by its id (`ST2067-3/SegmentDuration`).
pub fn find(id: &str) -> Option<&'static dyn AnyRule> {
    all().find(|r| r.id().0 == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::IssueSource;
    use std::collections::HashSet;

    #[test]
    fn every_table_lists_only_its_own_spec() {
        for (spec, rules) in tables() {
            for r in rules {
                assert_eq!(r.spec(), spec, "{} listed under {spec}", r.id());
            }
        }
    }

    #[test]
    fn rule_names_are_unique_within_a_spec_and_ids_globally() {
        let mut ids = HashSet::new();
        for r in all() {
            assert!(ids.insert(r.id()), "duplicate rule id {}", r.id());
            assert!(
                r.name().chars().all(|c| c.is_ascii_alphanumeric()),
                "rule name {} must be a plain identifier",
                r.name()
            );
        }
    }

    #[test]
    fn every_rule_has_one_row_per_known_edition_in_order() {
        for r in all() {
            let rows: Vec<String> = r.edition_rows().into_iter().map(|e| e.edition).collect();
            let labels: Vec<String> = r.edition_labels().into_iter().map(String::from).collect();
            assert_eq!(
                rows,
                labels,
                "{}: edition rows must match the spec's editions",
                r.id()
            );
        }
    }

    #[test]
    fn every_rule_exists_in_at_least_one_edition() {
        for r in all() {
            assert!(
                r.edition_rows().iter().any(|e| e.clause.is_some()),
                "{} has no edition that contains it",
                r.id()
            );
        }
    }

    #[test]
    fn severity_follows_strength_or_has_a_reason() {
        for r in all() {
            for e in r.edition_rows() {
                if e.clause.is_none() {
                    continue;
                }
                if e.default_severity != e.strength.default_severity() {
                    assert!(
                        e.severity_reason.as_deref().is_some_and(|s| !s.is_empty()),
                        "{} ({}): default severity {:?} differs from strength {} without a reason",
                        r.id(),
                        e.edition,
                        e.default_severity,
                        e.strength
                    );
                }
            }
        }
    }

    /// ST 2067-2:2016 §8.3 has Notes 1-5; Note 6 (an asset in the Packing
    /// List but not the Asset Map is not ingested) exists only in 2020.
    #[test]
    fn unresolved_uuid_2016_cites_only_2016_text() {
        use st2067_2::St2067_2Edition;
        let row = st2067_2::UnresolvedUuid
            .entry(St2067_2Edition::V2016)
            .unwrap();
        let text = row.interpretation.unwrap().text;
        assert!(!text.contains("Note 6"), "{text}");
        let row = st2067_2::UnresolvedUuid
            .entry(St2067_2Edition::V2020)
            .unwrap();
        assert!(row.interpretation.unwrap().text.contains("Note 6"));
    }

    /// A clause a description cites (`§6.8`) is a clause of one of the
    /// rule's own edition rows, unless the citation names another document
    /// (`ST 382 §10`).
    #[test]
    fn description_clauses_are_clauses_of_the_rule() {
        for r in all().filter(|r| r.spec().source() == IssueSource::ProseRule) {
            let desc = r.description();
            let own = r.spec().id().replace("ST", "ST ");
            for (i, _) in desc.match_indices('§') {
                let before = desc[..i].trim_end();
                if before.ends_with(char::is_numeric) && !before.ends_with(&own) {
                    continue; // cites another document
                }
                let clause: String = desc[i + '§'.len_utf8()..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '.')
                    .collect();
                let clause = clause.trim_end_matches('.');
                assert!(
                    r.edition_rows()
                        .iter()
                        .filter_map(|e| e.clause.as_deref())
                        .any(|c| c == clause
                            || c.starts_with(&format!("{clause}."))
                            || clause.starts_with(&format!("{c}."))),
                    "{}: description cites §{clause}, which no edition row has",
                    r.id()
                );
            }
        }
    }

    #[test]
    fn display_codes_carry_their_own_edition() {
        for r in all() {
            for e in r.edition_rows() {
                if let Some(code) = &e.code {
                    let prefix = if e.edition.is_empty() {
                        format!("{}:", r.spec().id())
                    } else {
                        format!("{}:{}", r.spec().id(), e.edition)
                    };
                    assert!(code.starts_with(&prefix), "{code} vs edition {}", e.edition);
                    assert!(code.ends_with(&format!("/{}", r.name())), "{code}");
                }
            }
        }
    }
}
