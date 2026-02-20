//! Builds `crates/imferno-wasm` with wasm-pack and syncs the output into
//! `docs/public/wasm/` so the Astro site always uses the freshest build.

use std::path::Path;
use std::process::Command;
use std::{env, fs};

pub fn run() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask has a parent directory");

    let wasm_crate = workspace_root.join("crates/imferno-wasm");
    let pkg_dir = wasm_crate.join("pkg");
    let docs_wasm = workspace_root.join("docs/public/wasm");

    // ── 1. Run wasm-pack ────────────────────────────────────────────────────
    println!("building imf-wasm with wasm-pack…");

    let status = Command::new("wasm-pack")
        .args(["build", "--target", "web"])
        .current_dir(&wasm_crate)
        .env("RUSTFLAGS", "--cfg getrandom_js")
        .status()
        .unwrap_or_else(|e| {
            eprintln!("failed to run wasm-pack: {e}");
            eprintln!("install it with: cargo install wasm-pack");
            std::process::exit(1);
        });

    if !status.success() {
        eprintln!("wasm-pack failed with exit code {:?}", status.code());
        std::process::exit(status.code().unwrap_or(1));
    }

    // ── 2. Sync pkg/ → docs/public/wasm/ ───────────────────────────────────
    println!("syncing pkg/ → docs/public/wasm/…");
    fs::create_dir_all(&docs_wasm).expect("create docs/public/wasm");

    let mut copied = 0usize;
    for entry in fs::read_dir(&pkg_dir).expect("read pkg dir") {
        let entry = entry.expect("read dir entry");
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        // Skip package metadata files that the browser doesn't need.
        if matches!(name.as_ref(), "package.json" | "README.md" | "LICENSE" | ".gitignore") {
            continue;
        }

        let src = entry.path();
        let dst = docs_wasm.join(&file_name);
        fs::copy(&src, &dst)
            .unwrap_or_else(|e| panic!("copy {name}: {e}"));
        copied += 1;
    }

    println!("  copied {copied} files to {}", docs_wasm.display());
}
