//! Generates `types.d.ts` for the Node (`crates/imferno-napi`) and WASM
//! (`crates/imferno-wasm`) packages from the Rust types' JSON Schema.
//!
//! Single source of truth: the `schemars::JsonSchema` derives on the
//! serialised Rust types (imferno-core's `jsonschema` feature). For each
//! package this runs imferno-core's `bindings_schema` example with that
//! package's feature set — the `wasm` feature renames some serialised fields,
//! so the two packages can differ — writes the combined schema to
//! `target/`, and hands it to `crates/imferno-napi/scripts/schema-to-dts.mjs`
//! (json-schema-to-typescript), which emits the package's `types.d.ts`.
//!
//! Requires `npm install` in `crates/imferno-napi` first (for the converter).

use std::fs;
use std::path::Path;
use std::process::Command;

pub fn run() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask has a parent directory");
    let napi_dir = workspace_root.join("crates/imferno-napi");
    if !napi_dir
        .join("node_modules/json-schema-to-typescript")
        .exists()
    {
        eprintln!(
            "error: json-schema-to-typescript not installed — run `npm install` in {}",
            napi_dir.display()
        );
        std::process::exit(1);
    }
    let script = napi_dir.join("scripts/schema-to-dts.mjs");
    let target = workspace_root.join("target");
    fs::create_dir_all(&target).expect("create target dir");

    let packages = [
        ("napi", "jsonschema", napi_dir.join("types.d.ts")),
        (
            "wasm",
            "jsonschema,wasm",
            workspace_root.join("crates/imferno-wasm/types.d.ts"),
        ),
    ];

    for (name, features, out) in packages {
        let output = Command::new(env!("CARGO"))
            .current_dir(workspace_root)
            .args([
                "run",
                "--quiet",
                "-p",
                "imferno-core",
                "--example",
                "bindings_schema",
                "--features",
                features,
            ])
            .output()
            .expect("failed to run cargo");
        if !output.status.success() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            eprintln!("error: bindings_schema example failed for {name}");
            std::process::exit(1);
        }
        let schema_path = target.join(format!("imferno-types-{name}.schema.json"));
        fs::write(&schema_path, &output.stdout).expect("write combined schema");
        eprintln!("  wrote {}", schema_path.display());

        let status = Command::new("node")
            .arg(&script)
            .arg(&schema_path)
            .arg(&out)
            .status()
            .expect("failed to run node (is Node.js installed?)");
        if !status.success() {
            eprintln!("error: {} failed for {name}", script.display());
            std::process::exit(1);
        }
    }
}
