//! IMF CLI — Command-line tool for validating IMF packages.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use imferno_core::package::{
    format_validation_result, validate, FormatOptions, Imferno, ReportFormat, RulesConfig,
    ValidationOptions, ValidationResult,
};
use imferno_core::validation::{AppSpecTarget, CoreSpecTarget};
use imferno_core::{Category, Severity, ValidationIssue, ValidationProfile, ValidationReport};
use std::io::IsTerminal;
use std::path::PathBuf;

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.0}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.0}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

fn use_color() -> bool {
    std::io::stdout().is_terminal()
        && std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM").map_or(true, |t| t != "dumb")
}

#[derive(Parser)]
#[command(name = "imferno")]
#[command(about = "SMPTE ST 2067 IMF validator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate an IMF package structure
    Validate {
        /// Path to the IMF package directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Verify SHA-1/SHA-256 hashes of all assets against PKL
        #[arg(long)]
        verify_hashes: bool,

        /// Number of files to hash in parallel (default: 8)
        #[arg(long, default_value = "8")]
        hash_concurrency: usize,

        /// Output format (summary = human-readable, json = full report)
        #[arg(short, long, value_enum, default_value = "summary")]
        format: OutputFormat,

        /// Core constraints spec version selection
        #[arg(long, value_enum, default_value = "auto")]
        core_spec: CoreSpecVersion,

        /// Application profile selection (App2E + ST 2067-201 IAB plug-ins)
        #[arg(long, value_enum, default_value = "auto")]
        app2e_spec: App2eSpecVersion,

        /// Skip file existence/size checks and MXF header validation.
        /// Only validates XML documents (ASSETMAP, PKL, CPL, VOLINDEX).
        #[arg(long)]
        skip_disk_checks: bool,

        /// Always exit with code 0, even when validation errors are found
        #[arg(long)]
        exit_zero: bool,

        /// Path to a JSON rules config file with ESLint-style severity overrides.
        #[arg(long, value_name = "PATH")]
        rules_config: Option<PathBuf>,
    },

    /// Show detailed CPL information
    Cpl {
        /// Path to the IMF package directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// CPL UUID (optional, shows first CPL if not specified)
        #[arg(short, long)]
        uuid: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum OutputFormat {
    /// Human-readable summary with optional color
    Summary,
    /// Markdown — tables and headers, embeddable in PRs
    Markdown,
    /// CSV — one row per issue, importable into Excel
    Csv,
    /// Full JSON (ValidationResult with package + validation)
    Json,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum CoreSpecVersion {
    Auto,
    V2013,
    V2016,
    V2020,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum App2eSpecVersion {
    Auto,
    None,
    V2020,
    V2021,
    V2023,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate {
            path,
            verify_hashes,
            hash_concurrency,
            format,
            core_spec,
            app2e_spec,
            skip_disk_checks,
            exit_zero,
            rules_config,
        } => {
            cmd_validate(
                &path,
                verify_hashes,
                hash_concurrency,
                format,
                core_spec,
                app2e_spec,
                skip_disk_checks,
                exit_zero,
                rules_config.as_deref(),
            )
            .await
        }
        Commands::Cpl { path, uuid } => cmd_cpl(&path, uuid),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn parse_rules(path: Option<&std::path::Path>) -> Result<RulesConfig> {
    match path {
        Some(p) => {
            let json = std::fs::read_to_string(p)
                .with_context(|| format!("Cannot read rules config: {}", p.display()))?;
            serde_json::from_str(&json)
                .with_context(|| format!("Invalid rules config JSON: {}", p.display()))
        }
        None => Ok(Default::default()),
    }
}

fn make_options(
    core_spec: CoreSpecVersion,
    app2e_spec: App2eSpecVersion,
    skip_disk_checks: bool,
    rules: RulesConfig,
) -> ValidationOptions {
    let core_spec_target = match core_spec {
        CoreSpecVersion::Auto => None,
        CoreSpecVersion::V2013 => Some(CoreSpecTarget::St2067_2_2013),
        CoreSpecVersion::V2016 => Some(CoreSpecTarget::St2067_2_2016),
        CoreSpecVersion::V2020 => Some(CoreSpecTarget::St2067_2_2020),
    };

    let app_spec_targets = match app2e_spec {
        App2eSpecVersion::Auto => None,
        App2eSpecVersion::None => Some(vec![]),
        App2eSpecVersion::V2020 => Some(vec![AppSpecTarget::St2067_21_2020]),
        App2eSpecVersion::V2021 => Some(vec![AppSpecTarget::St2067_21_2021]),
        App2eSpecVersion::V2023 => Some(vec![AppSpecTarget::St2067_21_2023]),
    };

    ValidationOptions {
        rules,
        core_spec: core_spec_target,
        app_specs: app_spec_targets,
        skip_disk_checks,
        ..Default::default()
    }
}

// ── Commands ─────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
async fn cmd_validate(
    path: &PathBuf,
    verify_hashes: bool,
    hash_concurrency: usize,
    format: OutputFormat,
    core_spec: CoreSpecVersion,
    app2e_spec: App2eSpecVersion,
    skip_disk_checks: bool,
    exit_zero: bool,
    rules_config_path: Option<&std::path::Path>,
) -> Result<()> {
    let rules = parse_rules(rules_config_path)?;
    let options = make_options(core_spec, app2e_spec, skip_disk_checks, rules);
    let color = use_color() && !matches!(format, OutputFormat::Json);

    // Read files
    let files = match imferno_core::package::read_dir(path) {
        Ok(f) => f,
        Err(e) => {
            if matches!(format, OutputFormat::Json) {
                let mut validation = ValidationReport::new(ValidationProfile::SMPTE);
                validation.add(
                    ValidationIssue::new(
                        Severity::Critical,
                        Category::Structure,
                        "PARSE-PACKAGE-FAILED",
                        format!("Failed to load IMF package: {}", e),
                    )
                    .with_suggestion(
                        "Ensure the directory contains VOLINDEX.xml and ASSETMAP.xml.",
                    ),
                );
                println!("{}", serde_json::to_string_pretty(&validation)?);
                return Ok(());
            }
            return Err(e.into());
        }
    };

    // Validate (parse + check in one call)
    let mut result: ValidationResult = validate(files, &options);

    // Hash verification — parallel with tokio
    if verify_hashes {
        use imferno_core::package::{HashFileStatus, HashProgressTracker};
        use std::sync::atomic::AtomicBool;
        use std::sync::Arc;

        let show_progress = !matches!(format, OutputFormat::Json) && color;
        let tracker = Arc::new(HashProgressTracker::new());

        // Spawn progress display ticker
        let stop = Arc::new(AtomicBool::new(false));
        let ticker = if show_progress {
            let t = tracker.clone();
            let s = stop.clone();
            Some(tokio::spawn(async move {
                use chromakopia::{Color, Gradient};
                let fire = Gradient::new(vec![
                    Color::new(220, 38, 38),
                    Color::new(249, 115, 22),
                    Color::new(250, 204, 21),
                ]);
                let palette = fire.palette(100);
                let bar_width = 20;

                let mut last_lines = 0;
                let mut frame: usize = 0;
                loop {
                    if s.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }
                    frame += 1;

                    let snap = t.snapshot();
                    let total_done: u64 = snap.iter().map(|(_, d, _, _)| *d).sum();
                    let total_size: u64 = snap.iter().map(|(_, _, s, _)| *s).sum();
                    let overall = if total_size > 0 {
                        total_done as f64 / total_size as f64
                    } else {
                        0.0
                    };
                    let pct = (overall * 100.0).min(100.0) as usize;

                    // Move cursor up to overwrite previous output
                    if last_lines > 0 {
                        eprint!("\x1b[{}A", last_lines);
                    }

                    // Overall progress line
                    let done_mb = total_done as f64 / 1_048_576.0;
                    let total_mb = total_size as f64 / 1_048_576.0;
                    eprintln!(
                        "\x1b[2K  hashing  {}% {:.0}/{:.0}MB",
                        pct, done_mb, total_mb,
                    );

                    // Per-file lines: fixed-width name | bar | size
                    let mut lines = 1;
                    let name_width = 30;
                    for (name, bytes_done, size, status) in &snap {
                        let short_name = if name.len() > name_width {
                            let half = (name_width - 1) / 2;
                            let tail = name_width - 1 - half;
                            format!("{}…{}", &name[..half], &name[name.len() - tail..])
                        } else {
                            format!("{:<width$}", name, width = name_width)
                        };
                        let size_str = format_size(*size);
                        let make_bar = |pct: f64, full_green: bool| -> String {
                            let filled = (pct * bar_width as f64) as usize;
                            let phase = frame as f64 * 0.15; // animation speed
                            (0..bar_width)
                                .map(|i| {
                                    if i < filled {
                                        if full_green {
                                            "\x1b[32m█\x1b[0m".to_string()
                                        } else {
                                            // Shift gradient position by phase for animation
                                            let t = ((i as f64 / bar_width as f64) + phase) % 1.0;
                                            let idx = (t * (palette.len() - 1) as f64) as usize;
                                            let c = &palette[idx.min(palette.len() - 1)];
                                            format!("\x1b[38;2;{};{};{}m█\x1b[0m", c.r, c.g, c.b)
                                        }
                                    } else {
                                        "\x1b[38;5;238m░\x1b[0m".to_string()
                                    }
                                })
                                .collect()
                        };
                        match status {
                            HashFileStatus::Done => {
                                eprintln!(
                                    "\x1b[2K  \x1b[32m[ matched ]\x1b[0m {} {:>8}",
                                    short_name, size_str,
                                );
                            }
                            HashFileStatus::Failed => {
                                eprintln!(
                                    "\x1b[2K  \x1b[31m[mismatch]\x1b[0m {} {:>8}",
                                    short_name, size_str,
                                );
                            }
                            HashFileStatus::Hashing => {
                                let file_pct = if *size > 0 {
                                    *bytes_done as f64 / *size as f64
                                } else {
                                    0.0
                                };
                                let bar = make_bar(file_pct, false);
                                let done_str = format_size(*bytes_done);
                                eprintln!(
                                    "\x1b[2K  [ hashing ] {} {:>8} {} {}",
                                    short_name, size_str, bar, done_str,
                                );
                            }
                            HashFileStatus::Waiting => {
                                eprintln!("\x1b[2K  [  queued ] {} {:>8}", short_name, size_str,);
                            }
                        }
                        lines += 1;
                    }
                    last_lines = lines;
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                }
            }))
        } else {
            None
        };

        // Run parallel hash verification
        let errs = result
            .package
            .validate_file_hashes_parallel(hash_concurrency, tracker.clone())
            .await;

        // Stop progress ticker
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(t) = ticker {
            let _ = t.await;
        }
        if show_progress {
            // Clear the progress display
            let snap = tracker.snapshot();
            for _ in 0..=snap.len() {
                eprint!("\x1b[2K\x1b[1A");
            }
            eprint!("\x1b[2K");
        }

        let hash_errs: Vec<_> = errs
            .into_iter()
            .filter(|e| {
                !matches!(
                    e,
                    imferno_core::package::FileValidationError::Missing { .. }
                )
            })
            .collect();
        if hash_errs.is_empty() && !matches!(format, OutputFormat::Json) {
            println!("  ok  All PKL file hashes verified");
        }
        for err in &hash_errs {
            result.validation.add(ValidationIssue::new(
                Severity::Error,
                Category::Asset,
                "FILE-HASH-ERROR",
                err.to_string(),
            ));
        }
    }

    // Output
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Summary | OutputFormat::Markdown | OutputFormat::Csv => {
            let report_format = match format {
                OutputFormat::Markdown => ReportFormat::Markdown,
                OutputFormat::Csv => ReportFormat::Csv,
                _ => ReportFormat::Text,
            };
            let opts = FormatOptions {
                format: report_format,
                color,
            };
            print!("{}", format_validation_result(&result, &opts));
            let has_errors =
                !result.validation.critical.is_empty() || !result.validation.errors.is_empty();
            if has_errors && !exit_zero {
                return Err(anyhow::anyhow!("Validation failed"));
            }
        }
    }

    Ok(())
}

fn cmd_cpl(path: &PathBuf, uuid: Option<String>) -> Result<()> {
    let package = Imferno::parse(imferno_core::package::read_dir(path)?)?;

    let cpl_uuid = if let Some(uuid) = uuid {
        uuid
    } else if let Some(cpl) = package.get_main_cpl() {
        cpl.id.to_string()
    } else {
        return Err(anyhow::anyhow!("No CPLs found in package"));
    };

    let details = package
        .get_cpl_details(&cpl_uuid)
        .ok_or_else(|| anyhow::anyhow!("CPL with UUID {} not found", cpl_uuid))?;

    println!("CPL Details");
    println!("===========");
    println!("ID: {}", details.id);
    println!("Title: {}", details.title);
    println!("Kind: {}", details.kind);
    println!("Issue Date: {}", details.issue_date);

    if let Some(ref annotation) = details.annotation {
        println!("Annotation: {}", annotation);
    }
    if let Some(ref issuer) = details.issuer {
        println!("Issuer: {}", issuer);
    }
    if let Some(ref creator) = details.creator {
        println!("Creator: {}", creator);
    }
    if let Some(ref originator) = details.content_originator {
        println!("Content Originator: {}", originator);
    }

    if !details.content_versions.is_empty() {
        println!("\nContent Versions:");
        for version in &details.content_versions {
            println!("  - {}", version);
        }
    }

    println!("\nSegments: {}", details.segments.len());
    for (i, segment) in details.segments.iter().enumerate() {
        println!(
            "  Segment {}: {} ({} sequences)",
            i + 1,
            segment.id,
            segment.sequence_count
        );
    }

    Ok(())
}
