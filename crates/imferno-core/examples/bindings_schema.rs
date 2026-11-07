//! Prints one combined JSON Schema (draft-07) covering every type that crosses
//! the JS boundary in the NAPI / WASM bindings. Used by
//! `cargo xtask generate-types-ts` to generate each package's `types.d.ts`.
//!
//! Run once per binding feature set, because the `wasm` feature changes the
//! serialised field names of some types (e.g. CPL fields are camelCase under
//! `wasm`, PascalCase otherwise):
//!
//! ```sh
//! cargo run -p imferno-core --example bindings_schema --features jsonschema        # NAPI
//! cargo run -p imferno-core --example bindings_schema --features jsonschema,wasm   # WASM
//! ```
#![allow(deprecated)] // ImfReport is deprecated but still returned by buildReport*.

use schemars::gen::SchemaSettings;
use serde_json::{json, Map, Value};

fn main() {
    let mut gen = SchemaSettings::draft07().into_generator();

    // Root types, exported under their Rust names. Every definition reachable
    // from them is emitted too.
    let roots = [
        (
            "ValidationResult",
            gen.subschema_for::<imferno_core::package::ValidationResult>(),
        ),
        (
            "Imferno",
            gen.subschema_for::<imferno_core::package::Imferno>(),
        ),
        (
            "ImfReport",
            gen.subschema_for::<imferno_core::package::ImfReport>(),
        ),
        (
            "RulesConfig",
            gen.subschema_for::<imferno_core::diagnostics::rules::RulesConfig>(),
        ),
    ];

    let definitions: Map<String, Value> = gen
        .definitions()
        .iter()
        .map(|(name, schema)| {
            (
                name.clone(),
                serde_json::to_value(schema).expect("serialize definition"),
            )
        })
        .collect();
    let properties: Map<String, Value> = roots
        .iter()
        .map(|(name, schema)| {
            (
                (*name).to_string(),
                serde_json::to_value(schema).expect("serialize root"),
            )
        })
        .collect();
    let required: Vec<&str> = roots.iter().map(|(n, _)| *n).collect();

    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "ImfernoTypes",
        "description": "Container for every type that crosses the imferno JS boundary. Not a runtime value.",
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
        "definitions": definitions,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&schema).expect("serialize schema")
    );
}
