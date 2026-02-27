use schemars::schema_for;
use std::fs;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../crates/imferno-core/npm/schema/schemas")
}

pub fn run() {
    let dir = out_dir();
    fs::create_dir_all(&dir).expect("create schemas dir");

    let schemas: Vec<(&str, schemars::schema::RootSchema)> = vec![
        (
            "imf-report",
            schema_for!(imferno_core::package::report::ImfReport),
        ),
        (
            "validation-report",
            schema_for!(imferno_core::ValidationReport),
        ),
        (
            "composition-playlist",
            schema_for!(imferno_core::cpl::CompositionPlaylist),
        ),
        ("asset-map", schema_for!(imferno_core::assetmap::AssetMap)),
        (
            "packing-list",
            schema_for!(imferno_core::assetmap::PackingList),
        ),
        (
            "volume-index",
            schema_for!(imferno_core::assetmap::VolumeIndex),
        ),
        (
            "rules-config",
            schema_for!(imferno_core::diagnostics::rules::RulesConfig),
        ),
    ];

    for (name, schema) in &schemas {
        let json = serde_json::to_string_pretty(schema).expect("serialize schema");
        let path = dir.join(format!("{name}.json"));
        fs::write(&path, &json).expect("write schema");
        eprintln!("  wrote {}", path.display());
    }

    eprintln!("generated {} schemas in {}", schemas.len(), dir.display());
}
