mod build_wasm;
mod generate_docs;
mod generate_schema;

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("build-wasm") => build_wasm::run(),
        Some("generate-docs") => generate_docs::run(),
        Some("generate-schema") => generate_schema::run(),
        Some("build-docs") => {
            build_wasm::run();
            generate_docs::run();
        }
        Some(cmd) => {
            eprintln!("unknown command: {cmd}");
            usage();
            std::process::exit(1);
        }
        None => {
            usage();
            std::process::exit(1);
        }
    }
}

fn usage() {
    eprintln!("usage: cargo xtask <command>");
    eprintln!();
    eprintln!("commands:");
    eprintln!("  build-wasm       build imf-wasm with wasm-pack and sync to docs/public/wasm/");
    eprintln!("  generate-docs    generate reference/codes/*.md from spec crate enums");
    eprintln!("  generate-schema  generate JSON Schema files into crates/imferno-core/npm/schema/schemas/");
    eprintln!("  build-docs       run build-wasm then generate-docs");
}
