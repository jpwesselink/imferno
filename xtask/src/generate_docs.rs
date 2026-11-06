//! Generates the validation-code catalogue for the docs site from the spec
//! crate enums:
//!
//! - `docs/docs/guide/codes.md`: the reference page with every code.
//! - `docs/docs/public/rules.json`: the same catalogue, grouped per spec,
//!   for the playground's advanced rule settings.
//!
//! Each enum exposes a `ALL: &'static [Self]` slice and implements the
//! `ValidationCode` trait (`code`, `description`, `default_severity`, `category`).
//! Both outputs are rendered from one section list (`catalogue()`), so they
//! can't drift apart — no hand-editing required.

use imferno_core::assetmap::codes::{
    St2067_2_2013_Core, St2067_2_2016_Core, St2067_2_2020, St2067_2_2020_Core, St429_9_2014,
};
use imferno_core::cpl::codes::{St2067_3_2013, St2067_3_2016};
use imferno_core::diagnostics::codes::ValidationCode;
use imferno_core::mxf::codes::{ImfernoMxf, St2067_2_2016, St377_1_2011, St377_4_2012};
use imferno_core::package::codes::ImfernoCode;
use imferno_core::scm::codes::St2067_9_2018;
use imferno_core::validation::codes::{St2067_21_2020, St2067_21_2023, St2067_21_2025};
use imferno_core::validation::iab_codes::{St2067_201_2019, St2067_201_2021, St2067_201_2026Delta};
use imferno_core::validation::isxd_codes::St2067_202_2023;
use imferno_core::xsd::codes::XsdConstraintCode;

use serde_json::json;
use std::fs;
use std::path::Path;

// ─────────────────────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────────────────────

pub fn run() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask has a parent directory");

    let sections = catalogue();

    let out_dir = workspace_root.join("docs/docs/guide");
    fs::create_dir_all(&out_dir).expect("create output directory");
    let path = out_dir.join("codes.md");
    fs::write(&path, codes_page(&sections)).unwrap_or_else(|e| panic!("write codes.md: {e}"));
    println!("  wrote {}", path.display());

    let public_dir = workspace_root.join("docs/docs/public");
    fs::create_dir_all(&public_dir).expect("create public directory");
    let path = public_dir.join("rules.json");
    fs::write(&path, rules_json(&sections)).unwrap_or_else(|e| panic!("write rules.json: {e}"));
    println!("  wrote {}", path.display());
}

// ─────────────────────────────────────────────────────────────────────────────
// Catalogue model
// ─────────────────────────────────────────────────────────────────────────────

struct Row {
    code: &'static str,
    description: &'static str,
    severity: String,
    category: String,
}

/// A block of codes; `title` renders as an `###` heading.
struct Group {
    title: Option<&'static str>,
    intro: Option<&'static str>,
    rows: Vec<Row>,
}

/// One spec (or engine area); `title` renders as an `##` heading.
struct Section {
    title: &'static str,
    intro: Option<&'static str>,
    groups: Vec<Group>,
}

fn rows<C: ValidationCode>(codes: &[C]) -> Vec<Row> {
    codes
        .iter()
        .map(|c| Row {
            code: c.code(),
            description: c.description(),
            severity: c.default_severity().to_string(),
            category: c.category().to_string(),
        })
        .collect()
}

fn group(title: &'static str, rows: Vec<Row>) -> Group {
    Group {
        title: Some(title),
        intro: None,
        rows,
    }
}

fn untitled(rows: Vec<Row>) -> Group {
    Group {
        title: None,
        intro: None,
        rows,
    }
}

fn section(title: &'static str, groups: Vec<Group>) -> Section {
    Section {
        title,
        intro: None,
        groups,
    }
}

fn catalogue() -> Vec<Section> {
    vec![
        section(
            "ST 429-9 — Volume Index",
            vec![untitled(rows(St429_9_2014::ALL))],
        ),
        section(
            "ST 377-1 — MXF File Format",
            vec![untitled(rows(St377_1_2011::ALL))],
        ),
        section(
            "ST 377-4 — MCA Sub-Descriptor Linkage",
            vec![untitled(rows(St377_4_2012::ALL))],
        ),
        section(
            "ST 2067-2 — Core Constraints & Packing List",
            vec![
                group("Package-level (2020)", rows(St2067_2_2020::ALL)),
                group("Core Constraints — 2013", rows(St2067_2_2013_Core::ALL)),
                group("Core Constraints — 2016", rows(St2067_2_2016_Core::ALL)),
                group("Core Constraints — 2020", rows(St2067_2_2020_Core::ALL)),
                group(
                    "Essence Layer — 2016 (inherited by later editions)",
                    rows(St2067_2_2016::ALL),
                ),
            ],
        ),
        section(
            "ST 2067-3 — Composition Playlist",
            vec![
                group("2013", rows(St2067_3_2013::ALL)),
                // 2016 covers 2020 too — the 2020 publication's CPL XSD is
                // byte-identical to 2016 (modulo header) and the catalogue is
                // bit-for-bit identical.
                group("2016 / 2020", rows(St2067_3_2016::ALL)),
            ],
        ),
        section(
            "ST 2067-9 — Sidecar Composition Map",
            vec![untitled(rows(St2067_9_2018::ALL))],
        ),
        section(
            "ST 2067-21 — Application #2E",
            vec![
                group("2020", rows(St2067_21_2020::ALL)),
                group("2023", rows(St2067_21_2023::ALL)),
                group("2025", rows(St2067_21_2025::ALL)),
            ],
        ),
        section(
            "ST 2067-201 — IAB Plug-in",
            vec![
                group("2019", rows(St2067_201_2019::ALL)),
                group("2021", rows(St2067_201_2021::ALL)),
                Group {
                    title: Some("2026"),
                    intro: Some("The 2026 edition inherits every 2021 code above and adds:"),
                    rows: rows(St2067_201_2026Delta::ALL),
                },
            ],
        ),
        section(
            "ST 2067-202 — ISXD Plug-in",
            vec![untitled(rows(St2067_202_2023::ALL))],
        ),
        Section {
            title: "XSD Schema Validation",
            intro: Some(
                "Codes emitted when a document fails validation against its SMPTE XML schema.",
            ),
            groups: vec![untitled(rows(XsdConstraintCode::ALL))],
        },
        Section {
            title: "imferno",
            intro: Some(
                "Codes emitted by imferno's package-level logic for conditions \
                 that don't map to a specific SMPTE spec clause.",
            ),
            groups: vec![
                untitled(rows(ImfernoCode::ALL)),
                untitled(rows(ImfernoMxf::ALL)),
            ],
        },
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Markdown page
// ─────────────────────────────────────────────────────────────────────────────

/// Render rows as a Markdown table.
fn code_table(rows: &[Row]) -> String {
    let mut out = String::from(
        "| Code | Description | Default Severity | Category |\n\
         |------|-------------|-----------------|----------|\n",
    );
    for r in rows {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            r.code, r.description, r.severity, r.category,
        ));
    }
    out
}

const CODE_FORMAT: &str = r#"## Code format

```
<SPEC>:<EDITION>:<SECTION>/<IDENTIFIER>
```

| Part | Meaning |
|------|---------|
| `SPEC` | SMPTE document, e.g. `ST2067-2`, `ST429-9`. `IMFERNO` for engine-level issues with no SMPTE source; `XSD` for schema violations. |
| `EDITION` | Publication year of the spec edition (`2013`, `2016`, `2020`, ...). Omitted for `IMFERNO` and `XSD` codes. |
| `SECTION` | Spec section or table the rule comes from (`6.2.1`, `Table-3`). |
| `IDENTIFIER` | CamelCase name of the issue. |

Identifier conventions:

- `...Missing`: a required field is absent (e.g. `ST2067-21:2023:6.2.1/CodingEquationsMissing`).
- `...Unknown`: a field is present but holds an unrecognised value or UL (e.g. `ST2067-21:2023:6.2.3/CodingEquationsUnknown`).
- Other names describe a specific failure (`...Mismatch`) or a constraint where presence vs. value doesn't apply.

Each spec edition has its own codes, even when a rule is unchanged between editions, so a code always tells you exactly which edition was applied. Code strings are stable: a rename is treated as a breaking change and listed in the changelog, so severity overrides keyed by code keep working across releases.

"#;

fn codes_page(sections: &[Section]) -> String {
    let mut s = String::from(
        "---\n\
         title: Validation Codes\n\
         description: Complete reference of all validation codes across every supported SMPTE spec.\n\
         ---\n\n\
         Every validation issue emitted by imferno carries a code like \
         `ST2067-2:2020:8.3/FileNotFound`. Use these codes to \
         [configure rule severity](/guide/config/).\n\n",
    );
    s.push_str(CODE_FORMAT);

    for sec in sections {
        s.push_str(&format!("## {}\n\n", sec.title));
        if let Some(intro) = sec.intro {
            s.push_str(&format!("{intro}\n\n"));
        }
        for g in &sec.groups {
            if let Some(title) = g.title {
                s.push_str(&format!("### {title}\n\n"));
            }
            if let Some(intro) = g.intro {
                s.push_str(&format!("{intro}\n\n"));
            }
            s.push_str(&code_table(&g.rows));
            s.push('\n');
        }
    }
    s
}

// ─────────────────────────────────────────────────────────────────────────────
// rules.json (playground)
// ─────────────────────────────────────────────────────────────────────────────

/// Map a catalogue severity (`Severity`'s Display) onto the rule-config
/// vocabulary the engine accepts (`off | info | warn | error | critical`).
fn config_severity(display: &str) -> &'static str {
    match display {
        "CRITICAL" => "critical",
        "ERROR" => "error",
        "WARNING" => "warn",
        "INFO" => "info",
        other => panic!("unmapped severity {other:?}; extend config_severity"),
    }
}

fn rules_json(sections: &[Section]) -> String {
    let sections: Vec<_> = sections
        .iter()
        .map(|sec| {
            json!({
                "title": sec.title,
                "groups": sec.groups.iter().map(|g| json!({
                    "title": g.title,
                    "rules": g.rows.iter().map(|r| json!({
                        "code": r.code,
                        "description": r.description,
                        "defaultSeverity": config_severity(&r.severity),
                        "category": r.category,
                    })).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    let mut out = serde_json::to_string_pretty(&json!({ "sections": sections }))
        .expect("serialise rules.json");
    out.push('\n');
    out
}
