//! IMF CLI - Command-line tool for validating IMF packages

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use imferno_core::package::{build_report, format_report, ImfReport, Imferno, ValidationOptions};
use imferno_core::validation::{AppSpecTarget, CoreSpecTarget};
use imferno_core::{Category, Severity, ValidationIssue, ValidationProfile, ValidationReport};
use std::io::{IsTerminal, Read as _};
use std::path::PathBuf;

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

        /// Verify SHA-1 hashes of all assets against PKL (slow)
        #[arg(long)]
        verify_hashes: bool,

        /// Output format (summary = human-readable, json = full report)
        #[arg(short, long, value_enum, default_value = "summary")]
        format: OutputFormat,

        /// Core constraints spec version selection
        #[arg(long, value_enum, default_value = "auto")]
        core_spec: CoreSpecVersion,

        /// Application profile selection (App2E + ST 2067-201 IAB plug-ins)
        #[arg(long, value_enum, default_value = "auto")]
        app2e_spec: App2eSpecVersion,

        /// Skip file manifest (existence/size) and MXF header checks.
        /// Validates XML structure only.
        #[arg(long)]
        xml_only: bool,

        /// Always exit with code 0, even when validation errors are found
        #[arg(long)]
        exit_zero: bool,

        /// Path to a JSON rules config file with ESLint-style severity overrides.
        #[arg(long, value_name = "PATH")]
        rules_config: Option<PathBuf>,
    },

    /// Export a full report as JSON (package metadata + validation + CPL analysis)
    Export {
        /// Path to the IMF package directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Path to ancestor IMP directory (for supplemental packages)
        #[arg(long)]
        ancestor: Option<PathBuf>,

        /// Core constraints spec version selection
        #[arg(long, value_enum, default_value = "auto")]
        core_spec: CoreSpecVersion,

        /// Application profile selection
        #[arg(long, value_enum, default_value = "auto")]
        app2e_spec: App2eSpecVersion,

        /// Skip file manifest and MXF header checks
        #[arg(long)]
        xml_only: bool,

        /// Path to a JSON rules config file
        #[arg(long, value_name = "PATH")]
        rules_config: Option<PathBuf>,
    },

    /// Pretty-print a previously exported JSON report
    Report {
        /// Path to a JSON report file, or "-" for stdin
        #[arg(value_name = "PATH")]
        path: String,
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
    Summary,
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate {
            path,
            verify_hashes,
            format,
            core_spec,
            app2e_spec,
            xml_only,
            exit_zero,
            rules_config,
        } => {
            cmd_validate(
                &path,
                verify_hashes,
                format,
                core_spec,
                app2e_spec,
                xml_only,
                exit_zero,
                rules_config.as_deref(),
            )?;
        }
        Commands::Export {
            path,
            ancestor,
            core_spec,
            app2e_spec,
            xml_only,
            rules_config,
        } => {
            cmd_export(
                &path,
                ancestor.as_deref(),
                core_spec,
                app2e_spec,
                xml_only,
                rules_config.as_deref(),
            )?;
        }
        Commands::Report { path } => {
            cmd_report(&path)?;
        }
        Commands::Cpl { path, uuid } => {
            cmd_cpl(&path, uuid)?;
        }
    }

    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn parse_rules(path: Option<&std::path::Path>) -> Result<imferno_core::package::RulesConfig> {
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
    xml_only: bool,
    rules: imferno_core::package::RulesConfig,
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
        skip_disk_checks: xml_only,
        ..Default::default()
    }
}

// ── Commands ─────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn cmd_validate(
    path: &PathBuf,
    verify_hashes: bool,
    format: OutputFormat,
    core_spec: CoreSpecVersion,
    app2e_spec: App2eSpecVersion,
    xml_only: bool,
    exit_zero: bool,
    rules_config_path: Option<&std::path::Path>,
) -> Result<()> {
    let rules = parse_rules(rules_config_path)?;
    let options = make_options(core_spec, app2e_spec, xml_only, rules);
    let color = use_color() && !matches!(format, OutputFormat::Json);

    // Parse
    let parse_result = imferno_core::package::read_dir(path).and_then(Imferno::parse);
    let package = match parse_result {
        Ok(p) => p,
        Err(e) => {
            if matches!(format, OutputFormat::Json) {
                let mut report = ValidationReport::new(ValidationProfile::SMPTE);
                report.add(
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
                println!("{}", serde_json::to_string_pretty(&report)?);
                return Ok(());
            }
            return Err(e.into());
        }
    };

    // Build report
    let mut report = build_report(&package, &options, None).map_err(|e| anyhow::anyhow!(e))?;

    // Hash verification (appends to report.validation)
    if verify_hashes {
        if !matches!(format, OutputFormat::Json) {
            println!("Verifying file hashes (this may take a moment)...");
        }
        let hash_errs: Vec<_> = package
            .validate_file_hashes()
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
            report.validation.add(ValidationIssue::new(
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
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        OutputFormat::Summary => {
            print!("{}", format_report(&report, color));
            let has_errors =
                !report.validation.critical.is_empty() || !report.validation.errors.is_empty();
            if has_errors && !exit_zero {
                return Err(anyhow::anyhow!("Validation failed"));
            }
        }
    }

    Ok(())
}

fn cmd_export(
    path: &PathBuf,
    ancestor_path: Option<&std::path::Path>,
    core_spec: CoreSpecVersion,
    app2e_spec: App2eSpecVersion,
    xml_only: bool,
    rules_config_path: Option<&std::path::Path>,
) -> Result<()> {
    let rules = parse_rules(rules_config_path)?;
    let options = make_options(core_spec, app2e_spec, xml_only, rules);

    let package = Imferno::parse(imferno_core::package::read_dir(path)?)?;

    let ancestor = if let Some(anc_path) = ancestor_path {
        Some(Imferno::parse(imferno_core::package::read_dir(anc_path)?)?)
    } else {
        None
    };

    let report =
        build_report(&package, &options, ancestor.as_ref()).map_err(|e| anyhow::anyhow!(e))?;

    println!("{}", serde_json::to_string_pretty(&report)?);

    Ok(())
}

fn cmd_report(path: &str) -> Result<()> {
    let json = if path == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .context("Failed to read from stdin")?;
        buf
    } else {
        std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read report file: {}", path))?
    };

    let report: ImfReport = serde_json::from_str(&json).context("Invalid ImfReport JSON")?;

    let color = use_color();
    print!("{}", format_report(&report, color));

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
