//! Generates `docs/src/content/docs/reference/codes/*.md` from the typed
//! validation-code enums defined in the spec crates.
//!
//! Each enum exposes a `ALL: &'static [Self]` slice and implements the
//! `ValidationCode` trait (`code`, `description`, `default_severity`, `category`).
//! This module iterates those slices to produce consistent Markdown reference
//! pages — no hand-editing required.

use imferno_core::assetmap::codes::{
    St2067_2_2013_Core, St2067_2_2016_Core, St2067_2_2020, St2067_2_2020_Core, St429_9_2014,
};
use imferno_core::cpl::codes::{St2067_3_2013, St2067_3_2016, St2067_3_2020};
use imferno_core::diagnostics::codes::ValidationCode;
use imferno_core::mxf::codes::St377_1_2011;
use imferno_core::scm::codes::St2067_9_2018;
use imferno_core::validation::codes::{St2067_21_2020, St2067_21_2023, St2067_21_2025};
use imferno_core::validation::iab_codes::{St2067_201_2019, St2067_201_2021};
use imferno_core::validation::isxd_codes::St2067_202_2022;
use imferno_core::package::codes::ImfernoCode;

use std::fs;
use std::path::Path;

// ─────────────────────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────────────────────

pub fn run() {
    // Resolve output directory relative to the workspace root.
    // __file__ is inside `xtask/src/`, so go up two levels to reach root.
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask has a parent directory");

    let out = workspace_root.join("docs/src/content/docs/reference/codes");
    fs::create_dir_all(&out).expect("create output directory");

    write(&out, "st429-9.md", st429_9_page());
    write(&out, "st377-1.md", st377_1_page());
    write(&out, "st2067-2.md", st2067_2_page());
    write(&out, "st2067-3.md", st2067_3_page());
    write(&out, "st2067-21.md", st2067_21_page());
    write(&out, "st2067-201.md", st2067_201_page());
    write(&out, "st2067-202.md", st2067_202_page());
    write(&out, "st2067-9.md", st2067_9_page());
    write(&out, "imferno.md", imferno_page());

    println!("docs written to {}", out.display());
}

fn write(dir: &Path, name: &str, content: String) {
    let path = dir.join(name);
    fs::write(&path, content).unwrap_or_else(|e| panic!("write {name}: {e}"));
    println!("  wrote {name}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Rendering helpers
// ─────────────────────────────────────────────────────────────────────────────

fn frontmatter(title: &str, description: &str) -> String {
    format!("---\ntitle: {title}\ndescription: {description}\n---\n\n")
}

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

/// Render a section heading + table.
fn section<C: ValidationCode>(heading: &str, codes: &[C]) -> String {
    format!("## {heading}\n\n{}\n", code_table(codes))
}

/// Render a sub-section (###) heading + table.
fn subsection<C: ValidationCode>(heading: &str, codes: &[C]) -> String {
    format!("### {heading}\n\n{}\n", code_table(codes))
}

// ─────────────────────────────────────────────────────────────────────────────
// Pages
// ─────────────────────────────────────────────────────────────────────────────

fn st429_9_page() -> String {
    let mut s = frontmatter(
        "ST 429-9 Validation Codes",
        "Validation codes from SMPTE ST 429-9:2014 (Volume Index & AssetMap).",
    );
    s.push_str(
        "Codes emitted when validating the `VOLINDEX.xml` document per SMPTE ST 429-9:2014.\n\n",
    );
    s.push_str(&section("St429\\_9\\_2014", St429_9_2014::ALL));
    s
}

fn st377_1_page() -> String {
    let mut s = frontmatter(
        "ST 377-1 Validation Codes",
        "Validation codes from SMPTE ST 377-1:2011 (MXF File Format).",
    );
    s.push_str("Codes emitted when inspecting MXF essence files per SMPTE ST 377-1:2011.\n\n");
    s.push_str(&section("St377\\_1\\_2011", St377_1_2011::ALL));
    s
}

fn st2067_2_page() -> String {
    let mut s = frontmatter(
        "ST 2067-2 Validation Codes",
        "Validation codes from SMPTE ST 2067-2 — package-level checks \
         and Core Constraints CPL structure rules across all three spec editions.",
    );

    s.push_str(
        "ST 2067-2 contributes two groups of codes:\n\n\
         1. **Package-level codes** (`St2067_2_2020`) — AssetMap, PKL, and file-integrity \
         checks emitted by `imferno-core`.\n\
         2. **Core Constraints CPL codes** — CPL structure rules shared across all three spec \
         editions (2013 / 2016 / 2020), emitted by the CoreConstraints validators. \
         The only difference between editions is the year in the code prefix.\n\n\
         ---\n\n",
    );

    s.push_str(&section(
        "ST 2067-2:2020 — Package-level",
        St2067_2_2020::ALL,
    ));
    s.push_str("---\n\n");

    // Core constraints: shared body, three editions.
    // We show one representative table per edition using ALL on each typed enum.
    s.push_str(
        "## Core Constraints CPL codes\n\n\
         The same ~38 rules apply to all three CPL editions. \
         The code prefix encodes the edition year.\n\n",
    );
    s.push_str(&subsection("ST 2067-2:2013", St2067_2_2013_Core::ALL));
    s.push_str(&subsection("ST 2067-2:2016", St2067_2_2016_Core::ALL));
    s.push_str(&subsection("ST 2067-2:2020", St2067_2_2020_Core::ALL));
    s
}

fn st2067_3_page() -> String {
    let mut s = frontmatter(
        "ST 2067-3 Validation Codes",
        "Validation codes from SMPTE ST 2067-3 (Composition Playlist) — \
         editions 2013, 2016, and 2020.",
    );
    s.push_str(
        "These codes are emitted when validating a Composition Playlist (CPL) per \
         SMPTE ST 2067-3. Three editions are supported. \
         The codes are identical across editions — only the year prefix differs.\n\n",
    );
    s.push_str(&subsection("ST 2067-3:2013", St2067_3_2013::ALL));
    s.push_str(&subsection("ST 2067-3:2016", St2067_3_2016::ALL));
    s.push_str(&subsection("ST 2067-3:2020", St2067_3_2020::ALL));
    s
}

fn st2067_21_page() -> String {
    let mut s = frontmatter(
        "ST 2067-21 Validation Codes",
        "Validation codes from SMPTE ST 2067-21 (Application Profile #2E) — \
         editions 2020, 2023, and 2025.",
    );
    s.push_str(
        "SMPTE ST 2067-21 defines Application Profile #2E (App2E), which adds constraints \
         on top of the Core Constraints for UHD, HDR, and advanced audio. \
         Three editions are implemented.\n\n---\n\n",
    );

    s.push_str(&section("ST 2067-21:2020", St2067_21_2020::ALL));
    s.push_str("---\n\n");

    // 2023 is large — split into spec-section subsections.
    s.push_str("## ST 2067-21:2023\n\n");
    s.push_str(&subsection(
        "§5.2 — Frame rates and resolutions",
        &filter(St2067_21_2023::ALL, |c| c.code().contains(":5.2/")),
    ));
    s.push_str(&subsection(
        "§5.3 — Language tags",
        &filter(St2067_21_2023::ALL, |c| c.code().contains(":5.3/")),
    ));
    s.push_str(&subsection(
        "§6.2 — Color system and required fields",
        &filter(St2067_21_2023::ALL, |c| {
            let code = c.code();
            code.contains(":6.2/")
                || code.contains(":6.2.1/")
                || code.contains(":6.2.2/")
                || code.contains(":6.2.3/")
                || code.contains(":6.2.4/")
                || code.contains(":6.2.5/")
        }),
    ));
    s.push_str(&subsection(
        "§6.3 — RGBA descriptor",
        &filter(St2067_21_2023::ALL, |c| {
            c.code().contains(":6.3/") || c.code().contains(":6.3.2/")
        }),
    ));
    s.push_str(&subsection(
        "§6.4 — Bit depth and chroma",
        &filter(St2067_21_2023::ALL, |c| {
            c.code().contains(":6.4/") || c.code().contains(":6.4.3/")
        }),
    ));
    s.push_str(&subsection(
        "§6.5 — Audio and JPEG 2000 sub-descriptor",
        &filter(St2067_21_2023::ALL, |c| {
            c.code().contains(":6.5/") || c.code().contains(":6.5.2/")
        }),
    ));
    s.push_str(&subsection(
        "§7 — Application constraints",
        &filter(St2067_21_2023::ALL, |c| {
            c.code().contains(":7.1/")
                || c.code().contains(":7.2/")
                || c.code().contains(":7.4/")
                || c.code().contains(":7.5/")
        }),
    ));

    s.push_str("---\n\n");
    s.push_str(&section("ST 2067-21:2025", St2067_21_2025::ALL));
    s
}

fn st2067_201_page() -> String {
    let mut s = frontmatter(
        "ST 2067-201 Validation Codes",
        "Validation codes from SMPTE ST 2067-201 (IAB Level 0 Plug-in) — \
         editions 2019 and 2021.",
    );
    s.push_str(
        "These codes are emitted when validating the IAB (Immersive Audio Bitstream) \
         Level 0 Plug-in per SMPTE ST 2067-201. \
         Both editions share the same 21 rules; only the year prefix differs.\n\n",
    );
    s.push_str(&subsection("ST 2067-201:2019", St2067_201_2019::ALL));
    s.push_str(&subsection("ST 2067-201:2021", St2067_201_2021::ALL));
    s
}

fn st2067_9_page() -> String {
    let mut s = frontmatter(
        "ST 2067-9 Validation Codes",
        "Validation codes from SMPTE ST 2067-9:2018 (Sidecar Composition Map).",
    );
    s.push_str(
        "Codes emitted when validating a Sidecar Composition Map (SCM) per \
         SMPTE ST 2067-9:2018. The SCM associates sidecar assets (e.g. IAB audio) \
         with a CPL by UUID without modifying the CPL itself.\n\n",
    );
    s.push_str(&section("St2067\\_9\\_2018", St2067_9_2018::ALL));
    s
}

fn st2067_202_page() -> String {
    let mut s = frontmatter(
        "ST 2067-202 Validation Codes",
        "Validation codes from SMPTE ST 2067-202:2022 (ISXD Plug-in).",
    );
    s.push_str(
        "These codes are emitted when validating the ISXD (Immersive Sound eXtensible \
         Description) Plug-in per SMPTE ST 2067-202:2022.\n\n",
    );
    s.push_str(&section("St2067\\_202\\_2022", St2067_202_2022::ALL));
    s
}

fn imferno_page() -> String {
    let mut s = frontmatter(
        "imferno Validation Codes",
        "Validation codes emitted by imferno itself (not tied to a specific SMPTE spec).",
    );
    s.push_str(
        "These codes are emitted by imferno's package-level logic for conditions \
         that don't map to a specific SMPTE spec clause.\n\n",
    );
    s.push_str(&section("Imferno", ImfernoCode::ALL));
    s
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Return a Vec of copies of items in `codes` matching `pred`.
fn filter<C: ValidationCode + Copy>(codes: &[C], pred: impl Fn(&C) -> bool) -> Vec<C> {
    codes.iter().copied().filter(|c| pred(c)).collect()
}
