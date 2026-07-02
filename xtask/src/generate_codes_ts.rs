//! Generates `crates/imferno-wasm/codes.js` and `crates/imferno-wasm/codes.d.ts`
//! from the typed validation-code enums defined in imferno-core.
//!
//! Each enum exposes `ALL: &'static [Self]` and implements `ValidationCode`.
//! This module iterates those slices to produce JavaScript constants with
//! full TypeScript literal types — giving autocomplete and typo protection.

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
use imferno_core::validation::isxd_codes::St2067_202_2023;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Convert a hyphenated name to camelCase: "ResourceList-Empty" → "ResourceListEmpty"
fn to_camel(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut cap_next = false;
    for ch in name.chars() {
        if ch == '-' {
            cap_next = true;
        } else if cap_next {
            out.extend(ch.to_uppercase());
            cap_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// Extract (js_key, full_code_string) pairs from a ValidationCode slice.
/// Handles duplicate suffixes by appending the section number, and converts
/// hyphens to camelCase for valid JS identifiers.
fn entries<C: ValidationCode>(codes: &[C]) -> Vec<(String, &'static str)> {
    // First pass: collect raw suffix → code pairs and detect duplicates
    let raw: Vec<_> = codes
        .iter()
        .map(|c| {
            let full = c.code();
            let suffix = full.rsplit('/').next().unwrap_or(full);
            (suffix, full)
        })
        .collect();

    // Count how many times each suffix appears
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for (suffix, _) in &raw {
        *counts.entry(suffix).or_default() += 1;
    }

    // Second pass: disambiguate duplicates by prepending the section part
    raw.into_iter()
        .map(|(suffix, full)| {
            let key = if counts[suffix] > 1 {
                // Extract section from "SPEC:YEAR:SECTION/Name" → "SECTION"
                let before_slash = full.rsplit_once('/').map(|(l, _)| l).unwrap_or(full);
                let section = before_slash.rsplit_once(':').map(|(_, s)| s).unwrap_or("");
                // Map well-known sections to descriptive prefixes
                let prefix = match section {
                    "7" | "XSD" => "AssetMap",
                    "9" => "Pkl",
                    other => &to_camel(other),
                };
                format!("{prefix}{}", to_camel(suffix))
            } else {
                to_camel(suffix)
            };
            (key, full)
        })
        .collect()
}

/// Render a single enum group as JS object properties.
fn js_object<C: ValidationCode>(name: &str, codes: &[C]) -> String {
    let mut out = format!("  {name}: {{\n");
    for (key, code) in entries(codes) {
        if key.starts_with(|c: char| c.is_ascii_digit()) || key.contains('.') {
            out.push_str(&format!("    \"{key}\": \"{code}\",\n"));
        } else {
            out.push_str(&format!("    {key}: \"{code}\",\n"));
        }
    }
    out.push_str("  }");
    out
}

/// Render a single enum group as TS declaration properties with literal types.
fn dts_object<C: ValidationCode>(name: &str, codes: &[C]) -> String {
    let mut out = format!("  readonly {name}: {{\n");
    for (key, code) in entries(codes) {
        if key.starts_with(|c: char| c.is_ascii_digit()) || key.contains('.') {
            out.push_str(&format!("    readonly \"{key}\": \"{code}\";\n"));
        } else {
            out.push_str(&format!("    readonly {key}: \"{code}\";\n"));
        }
    }
    out.push_str("  }");
    out
}

pub fn run() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask has a parent directory");

    let out_dir = workspace_root.join("crates/imferno-wasm");

    // Collect all enum groups
    let js_groups = [
        js_object("ST429_9_2014", St429_9_2014::ALL),
        js_object("ST377_1_2011", St377_1_2011::ALL),
        js_object("ST2067_2_2020", St2067_2_2020::ALL),
        js_object("ST2067_2_2013_Core", St2067_2_2013_Core::ALL),
        js_object("ST2067_2_2016_Core", St2067_2_2016_Core::ALL),
        js_object("ST2067_2_2020_Core", St2067_2_2020_Core::ALL),
        js_object("ST2067_3_2013", St2067_3_2013::ALL),
        js_object("ST2067_3_2016", St2067_3_2016::ALL),
        js_object("ST2067_9_2018", St2067_9_2018::ALL),
        js_object("ST2067_21_2020", St2067_21_2020::ALL),
        js_object("ST2067_21_2023", St2067_21_2023::ALL),
        js_object("ST2067_21_2025", St2067_21_2025::ALL),
        js_object("ST2067_201_2019", St2067_201_2019::ALL),
        js_object("ST2067_201_2021", St2067_201_2021::ALL),
        js_object("ST2067_202_2023", St2067_202_2023::ALL),
        js_object("Imferno", ImfernoCode::ALL),
    ];

    let dts_groups = [
        dts_object("ST429_9_2014", St429_9_2014::ALL),
        dts_object("ST377_1_2011", St377_1_2011::ALL),
        dts_object("ST2067_2_2020", St2067_2_2020::ALL),
        dts_object("ST2067_2_2013_Core", St2067_2_2013_Core::ALL),
        dts_object("ST2067_2_2016_Core", St2067_2_2016_Core::ALL),
        dts_object("ST2067_2_2020_Core", St2067_2_2020_Core::ALL),
        dts_object("ST2067_3_2013", St2067_3_2013::ALL),
        dts_object("ST2067_3_2016", St2067_3_2016::ALL),
        dts_object("ST2067_9_2018", St2067_9_2018::ALL),
        dts_object("ST2067_21_2020", St2067_21_2020::ALL),
        dts_object("ST2067_21_2023", St2067_21_2023::ALL),
        dts_object("ST2067_21_2025", St2067_21_2025::ALL),
        dts_object("ST2067_201_2019", St2067_201_2019::ALL),
        dts_object("ST2067_201_2021", St2067_201_2021::ALL),
        dts_object("ST2067_202_2023", St2067_202_2023::ALL),
        dts_object("Imferno", ImfernoCode::ALL),
    ];

    // codes.js
    let js = format!(
        "// Auto-generated by `cargo xtask generate-codes-ts` \u{2014} do not edit.\n\
         export const codes = {{\n\
         {}\n\
         }};\n",
        js_groups.join(",\n")
    );

    // codes.d.ts
    let dts = format!(
        "// Auto-generated by `cargo xtask generate-codes-ts` \u{2014} do not edit.\n\
         export declare const codes: {{\n\
         {}\n\
         }};\n",
        dts_groups.join(";\n")
    );

    // ── WASM package (ESM) ──
    let wasm_js = out_dir.join("codes.js");
    let wasm_dts = out_dir.join("codes.d.ts");
    fs::write(&wasm_js, &js).unwrap_or_else(|e| panic!("write codes.js: {e}"));
    fs::write(&wasm_dts, &dts).unwrap_or_else(|e| panic!("write codes.d.ts: {e}"));
    println!("  wrote {}", wasm_js.display());
    println!("  wrote {}", wasm_dts.display());

    // ── NAPI package (CJS) ──
    let napi_dir = workspace_root.join("crates/imferno-napi");
    let cjs = format!(
        "// Auto-generated by `cargo xtask generate-codes-ts` \u{2014} do not edit.\n\
         const codes = {{\n\
         {}\n\
         }};\n\
         module.exports = {{ codes }};\n",
        js_groups.join(",\n")
    );
    let napi_js = napi_dir.join("codes.js");
    let napi_dts = napi_dir.join("codes.d.ts");
    fs::write(&napi_js, cjs).unwrap_or_else(|e| panic!("write napi codes.js: {e}"));
    fs::write(&napi_dts, &dts).unwrap_or_else(|e| panic!("write napi codes.d.ts: {e}"));
    println!("  wrote {}", napi_js.display());
    println!("  wrote {}", napi_dts.display());
}
