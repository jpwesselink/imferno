//! Generates `docs/docs/guide/codes.md` — every rule of every spec, with
//! its per-edition table (clause, default severity), from the rule
//! catalogue. No hand-editing.

use imferno_core::catalogue;
use imferno_core::diagnostics::rule::AnyRule;

use std::fs;
use std::path::Path;

pub fn run() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask has a parent directory");

    let out_dir = workspace_root.join("docs/docs/guide");
    fs::create_dir_all(&out_dir).expect("create output directory");

    let path = out_dir.join("codes.md");
    fs::write(&path, codes_page()).unwrap_or_else(|e| panic!("write codes.md: {e}"));
    println!("  wrote {}", path.display());
}

const INTRO: &str = r#"---
title: Validation Codes
description: Every validation rule imferno checks, per spec and per edition.
---

Every finding names the rule it reports and the edition of the spec that was
applied, as a code like `ST2067-3:2016:7.2.2/SegmentDuration`. Use the rule
ids and codes to [configure rule severity](/guide/config/).

## Code format

```
<SPEC>:<EDITION>:<CLAUSE>/<Name>
```

| Part | Meaning |
|------|---------|
| `SPEC` | SMPTE document, e.g. `ST2067-2`, `ST429-9`. `IMFERNO` for engine checks with no SMPTE source; `XSD` for schema validation. |
| `EDITION` | The edition of the spec that was **applied** (`2013`, `2016`, `2020`, …). For `XSD` the schema used (`CPL-2016`); omitted for `IMFERNO`. |
| `CLAUSE` | Clause of that edition the rule comes from (`7.2.2`, `A.2`). For `IMFERNO` a group (`Package`, `Mxf`). |
| `Name` | The rule's name, unique within the spec. |

Each rule has a stable **rule id** `SPEC/Name` (e.g. `ST2067-3/SegmentDuration`)
that does not change when a clause is corrected; the clause and severity of
each edition live in the rule's edition table below.

In the tables, each edition column shows the clause and default severity.
`—` means the edition does not contain the rule. `✓` marks a row checked
against that edition's text; `†` a clause known or suspected to be wrong.
Unmarked rows are carried over from earlier catalogues and not yet checked.

"#;

fn severity_short(s: imferno_core::Severity) -> &'static str {
    match s {
        imferno_core::Severity::Critical => "critical",
        imferno_core::Severity::Error => "error",
        imferno_core::Severity::Warning => "warning",
        imferno_core::Severity::Info => "info",
    }
}

fn escape(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

fn spec_table(rules: &[&'static dyn AnyRule]) -> String {
    let Some(first) = rules.first() else {
        return String::new();
    };
    let editions = first.edition_labels();
    let single = editions.len() == 1 && editions[0].is_empty();
    let mut out = String::from("| Rule | Description | Category |");
    let mut sep = String::from("|------|-------------|----------|");
    for e in &editions {
        out.push_str(&format!(" {} |", if single { "Clause" } else { e }));
        sep.push_str("------|");
    }
    out.push('\n');
    out.push_str(&sep);
    out.push('\n');
    for r in rules {
        out.push_str(&format!(
            "| `{}` | {} | {} |",
            r.id(),
            escape(r.description()),
            r.category()
        ));
        for row in r.edition_rows() {
            let cell = match &row.clause {
                None if row.verified => "—✓".to_string(),
                None => "—".to_string(),
                Some(c) => {
                    let clause = if c.is_empty() { "·" } else { c.as_str() };
                    format!(
                        "{}{} {}",
                        clause,
                        if row.suspect {
                            "†"
                        } else if row.verified {
                            "✓"
                        } else {
                            ""
                        },
                        severity_short(row.default_severity)
                    )
                }
            };
            out.push_str(&format!(" {cell} |"));
        }
        out.push('\n');
    }
    out
}

fn codes_page() -> String {
    let mut s = String::from(INTRO);
    for (spec, rules) in catalogue::tables() {
        s.push_str(&format!("## {}\n\n", spec.title()));
        s.push_str(&spec_table(rules));
        s.push('\n');
    }
    s
}
