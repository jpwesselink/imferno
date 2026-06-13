//! Generates `docs/src/content/docs/guide/codes.md` — a single page with every
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
use imferno_core::mxf::codes::St377_1_2011;
use imferno_core::package::codes::ImfernoCode;
use imferno_core::scm::codes::St2067_9_2018;
use imferno_core::validation::codes::{St2067_21_2020, St2067_21_2023, St2067_21_2025};
use imferno_core::validation::iab_codes::{St2067_201_2019, St2067_201_2021};
use imferno_core::validation::isxd_codes::St2067_202_2022;

use std::fs;
use std::path::Path;

// ─────────────────────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────────────────────

pub fn run() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask has a parent directory");

    let out_dir = workspace_root.join("docs/src/content/docs/guide");
    fs::create_dir_all(&out_dir).expect("create output directory");

    // Remove old per-spec pages if they exist
    let old_codes_dir = workspace_root.join("docs/src/content/docs/reference/codes");
    if old_codes_dir.exists() {
        fs::remove_dir_all(&old_codes_dir).ok();
        println!("  removed old reference/codes/ directory");
    }

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

    // ── ST 429-9 ──
    s.push_str("## ST 429-9 — Volume Index\n\n");
    s.push_str(&code_table(St429_9_2014::ALL));
    s.push('\n');

    // ── ST 377-1 ──
    s.push_str("## ST 377-1 — MXF File Format\n\n");
    s.push_str(&code_table(St377_1_2011::ALL));
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
    s.push_str("### 2020\n\n");
    s.push_str(&code_table(St2067_21_2020::ALL));
    s.push('\n');
    s.push_str("### 2023\n\n");
    s.push_str(&code_table(St2067_21_2023::ALL));
    s.push('\n');
    s.push_str("### 2025\n\n");
    s.push_str(&code_table(St2067_21_2025::ALL));
    s.push('\n');

    // ── ST 2067-201 ──
    s.push_str("## ST 2067-201 — IAB Plug-in\n\n");
    s.push_str("### 2019\n\n");
    s.push_str(&code_table(St2067_201_2019::ALL));
    s.push('\n');
    s.push_str("### 2021\n\n");
    s.push_str(&code_table(St2067_201_2021::ALL));
    s.push('\n');

    // ── ST 2067-202 ──
    s.push_str("## ST 2067-202 — ISXD Plug-in\n\n");
    s.push_str(&code_table(St2067_202_2022::ALL));
    s.push('\n');

    // ── imferno ──
    s.push_str("## imferno\n\n");
    s.push_str(
        "Codes emitted by imferno's package-level logic for conditions \
         that don't map to a specific SMPTE spec clause.\n\n",
    );
    s.push_str(&code_table(ImfernoCode::ALL));
    s.push('\n');

    s
}
