//! One-shot driver: run the MXF essence-layer + audio-MCA rules against
//! every MXF in a directory and print a per-file summary table.
//!
//! Used to grade imferno's ST 2067-203 / -204 coverage against the
//! Fraunhofer SMPTE working-group test corpus, since those fixtures are
//! standalone MXFs without an IMP wrapper (so `imferno validate` can't
//! drive them directly).
//!
//! Usage: `cargo run --example fraunhofer_corpus_check -- /path/to/dir`

use std::path::Path;

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("usage: fraunhofer_corpus_check <dir-of-mxfs>");
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read_dir {dir}: {e}"))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("mxf"))
        .collect();
    entries.sort();

    println!(
        "{:<60} {:>10}  {:<40}  {:>6}",
        "file", "size", "descriptor", "issues"
    );
    println!("{:-<120}", "");

    for path in &entries {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        // Same call as the production package-validation site:
        // footer-first with header fallback.
        match imferno_core::mxf::metadata::parse_mxf_to_regxml_with_partition_fallback(
            path,
            regxml::RootMode::Preface,
        ) {
            Ok(xml) => {
                let descriptor = detect_descriptor(&xml);
                let audio_issues = imferno_core::mxf::audio_mca::check_audio_mca(&xml, path);
                println!(
                    "{:<60} {:>10}  {:<40}  {:>6}",
                    name,
                    format_size(size),
                    descriptor,
                    audio_issues.len()
                );
                for issue in &audio_issues {
                    println!(
                        "    {:?}  {}  {}",
                        issue.severity, issue.code, issue.message
                    );
                }
            }
            Err(e) => {
                println!(
                    "{:<60} {:>10}  PARSE FAILED {:?}",
                    name,
                    format_size(size),
                    e
                );
            }
        }
    }
}

fn detect_descriptor(regxml: &str) -> &'static str {
    if regxml.contains("MGASoundEssenceDescriptor") {
        "MGASoundEssenceDescriptor (SADM)"
    } else if regxml.contains("WAVEPCMDescriptor") {
        "WAVEPCMDescriptor"
    } else if regxml.contains("IABEssenceDescriptor") {
        "IABEssenceDescriptor"
    } else if regxml.contains("CDCIDescriptor") {
        "CDCIDescriptor (video)"
    } else if regxml.contains("RGBADescriptor") {
        "RGBADescriptor (video)"
    } else if regxml.contains("JPEG2000SubDescriptor") {
        "JPEG2000 video"
    } else {
        "unknown"
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.0}KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes}B")
    }
}

#[allow(dead_code)]
fn _ignore_unused(_: &Path) {}
