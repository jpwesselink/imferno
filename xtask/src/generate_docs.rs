//! Generates `docs/docs/guide/codes.md` — a single page with every
//! validation code across all supported SMPTE specs.
//!
//! Each enum exposes a `ALL: &'static [Self]` slice and implements the
//! `ValidationCode` trait (`code`, `description`, `default_severity`, `category`).
//! This module iterates those slices to produce one consolidated Markdown
//! reference page — no hand-editing required.

use imferno_core::assetmap::codes::{
    St2067_2_2013_Core, St2067_2_2016_Core, St2067_2_2020, St2067_2_2020_Core, St429_9_2014,
};
use imferno_core::cpl::codes::{St2067_3_2013, St2067_3_2016};
use imferno_core::diagnostics::codes::ValidationCode;
use imferno_core::mxf::codes::{ImfernoMxf, St2067_2_2016, St377_1_2011, St377_4_2012};
use imferno_core::package::codes::ImfernoCode;
use imferno_core::scm::codes::St2067_9_2018;
use imferno_core::validation::codes::{
    St2067_21_2014, St2067_21_2016, St2067_21_2020, St2067_21_2023, St2067_21_2026,
};
use imferno_core::validation::iab_codes::{St2067_201_2019, St2067_201_2021, St2067_201_2026Delta};
use imferno_core::validation::isxd_codes::St2067_202_2023;
use imferno_core::xsd::codes::XsdConstraintCode;

use std::fs;
use std::path::Path;

// ─────────────────────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Rendering helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Render a ValidationCode slice as a Markdown table.
fn code_table<C: ValidationCode>(codes: &[C]) -> String {
    let mut out = String::from(
        "| Code | Description | Default Severity | Category |\n\
         |------|-------------|-----------------|----------|\n",
    );
    for c in codes {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            c.code(),
            c.description(),
            c.default_severity(),
            c.category(),
        ));
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Single page
// ─────────────────────────────────────────────────────────────────────────────

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

fn codes_page() -> String {
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

    // ── ST 429-9 ──
    s.push_str("## ST 429-9 — Volume Index\n\n");
    s.push_str(&code_table(St429_9_2014::ALL));
    s.push('\n');

    // ── ST 377-1 ──
    s.push_str("## ST 377-1 — MXF File Format\n\n");
    s.push_str(&code_table(St377_1_2011::ALL));
    s.push('\n');

    // ── ST 377-4 ──
    s.push_str("## ST 377-4 — MCA Sub-Descriptor Linkage\n\n");
    s.push_str(&code_table(St377_4_2012::ALL));
    s.push('\n');

    // ── ST 2067-2 ──
    s.push_str("## ST 2067-2 — Core Constraints & Packing List\n\n");
    s.push_str("### Package-level (2020)\n\n");
    s.push_str(&code_table(St2067_2_2020::ALL));
    s.push('\n');
    s.push_str("### Core Constraints — 2013\n\n");
    s.push_str(&code_table(St2067_2_2013_Core::ALL));
    s.push('\n');
    s.push_str("### Core Constraints — 2016\n\n");
    s.push_str(&code_table(St2067_2_2016_Core::ALL));
    s.push('\n');
    s.push_str("### Core Constraints — 2020\n\n");
    s.push_str(&code_table(St2067_2_2020_Core::ALL));
    s.push('\n');
    s.push_str("### Essence Layer — 2016 (inherited by later editions)\n\n");
    s.push_str(&code_table(St2067_2_2016::ALL));
    s.push('\n');

    // ── ST 2067-3 ──
    s.push_str("## ST 2067-3 — Composition Playlist\n\n");
    s.push_str("### 2013\n\n");
    s.push_str(&code_table(St2067_3_2013::ALL));
    s.push('\n');
    // 2016 covers 2020 too — the 2020 publication's CPL XSD is byte-identical
    // to 2016 (modulo header) and the catalogue is bit-for-bit identical.
    s.push_str("### 2016 / 2020\n\n");
    s.push_str(&code_table(St2067_3_2016::ALL));
    s.push('\n');

    // ── ST 2067-9 ──
    s.push_str("## ST 2067-9 — Sidecar Composition Map\n\n");
    s.push_str(&code_table(St2067_9_2018::ALL));
    s.push('\n');

    // ── ST 2067-21 ──
    s.push_str("## ST 2067-21 — Application #2E\n\n");
    s.push_str("### 2014\n\n");
    s.push_str(&code_table(St2067_21_2014::ALL));
    s.push('\n');
    s.push_str("### 2016\n\n");
    s.push_str(&code_table(St2067_21_2016::ALL));
    s.push('\n');
    s.push_str("### 2020 (incl. Amendment 1:2020)\n\n");
    s.push_str(&code_table(St2067_21_2020::ALL));
    s.push('\n');
    s.push_str("### 2023\n\n");
    s.push_str(&code_table(St2067_21_2023::ALL));
    s.push('\n');
    s.push_str("### 2026\n\n");
    s.push_str(&code_table(St2067_21_2026::ALL));
    s.push('\n');

    // ── ST 2067-201 ──
    s.push_str("## ST 2067-201 — IAB Plug-in\n\n");
    s.push_str("### 2019\n\n");
    s.push_str(&code_table(St2067_201_2019::ALL));
    s.push('\n');
    s.push_str("### 2021\n\n");
    s.push_str(&code_table(St2067_201_2021::ALL));
    s.push('\n');
    s.push_str("### 2026\n\n");
    s.push_str("The 2026 edition inherits every 2021 code above and adds:\n\n");
    s.push_str(&code_table(St2067_201_2026Delta::ALL));
    s.push('\n');

    // ── ST 2067-202 ──
    s.push_str("## ST 2067-202 — ISXD Plug-in\n\n");
    s.push_str(&code_table(St2067_202_2023::ALL));
    s.push('\n');

    // ── XSD ──
    s.push_str("## XSD Schema Validation\n\n");
    s.push_str(
        "Codes emitted when a document fails validation against its \
         SMPTE XML schema.\n\n",
    );
    s.push_str(&code_table(XsdConstraintCode::ALL));
    s.push('\n');

    // ── imferno ──
    s.push_str("## imferno\n\n");
    s.push_str(
        "Codes emitted by imferno's package-level logic for conditions \
         that don't map to a specific SMPTE spec clause.\n\n",
    );
    s.push_str(&code_table(ImfernoCode::ALL));
    s.push('\n');
    s.push_str(&code_table(ImfernoMxf::ALL));
    s.push('\n');

    s
}
