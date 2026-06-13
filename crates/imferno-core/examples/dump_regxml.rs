//! Quick scratch helper: convert an MXF file to RegXML and print to stdout.
//!
//! Used during essence-rule development to inspect what the
//! regxmllib-rs pipeline emits for our vendored test fixtures, so the
//! audio MCA / IMSC / IAB rule assertions know what to walk.
//!
//! Usage: `cargo run --example dump_regxml -- path/to/file.mxf`

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: dump_regxml <file.mxf>");
    let opts = regxml::MxfFragmentOptions {
        partition: regxml::PartitionTarget::Header,
        ..Default::default()
    };
    match imferno_core::mxf::metadata::parse_mxf_to_regxml(std::path::Path::new(&path), opts) {
        Ok(xml) => {
            eprintln!("output is {} bytes", xml.len());
            println!("{xml}");
        }
        Err(e) => {
            eprintln!("error: {e:?}");
            std::process::exit(1);
        }
    }
}
