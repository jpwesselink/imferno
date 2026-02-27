//! IMF CLI - Command-line tool for inspecting IMF packages

use clap::{Parser, Subcommand};
use imferno_core::package::{Imferno, ValidationOptions};
use std::path::PathBuf;
use anyhow::{Context, Result};
use imferno_core::validation::{
    validate_cpl_with_registry, AppSpecTarget, ConfigurableValidatorRegistry, CoreSpecTarget,
    ValidatorSelection,
};
use std::io::IsTerminal;
use imferno_core::{Severity, ValidationIssue, Category, ValidationProfile, ValidationReport};

// ── ANSI colour helpers ───────────────────────────────────────────────────────

fn use_color() -> bool {
    std::io::stdout().is_terminal()
        && std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM").map_or(true, |t| t != "dumb")
}

fn c_red(s: &str, on: bool)    -> String { if on { format!("\x1b[31m{}\x1b[0m", s) } else { s.to_string() } }
fn c_yellow(s: &str, on: bool) -> String { if on { format!("\x1b[33m{}\x1b[0m", s) } else { s.to_string() } }
fn c_cyan(s: &str, on: bool)   -> String { if on { format!("\x1b[36m{}\x1b[0m", s) } else { s.to_string() } }
fn c_green(s: &str, on: bool)  -> String { if on { format!("\x1b[32m{}\x1b[0m", s) } else { s.to_string() } }
fn c_bold(s: &str, on: bool)   -> String { if on { format!("\x1b[1m{}\x1b[0m", s) } else { s.to_string() } }
fn c_dim(s: &str, on: bool)    -> String { if on { format!("\x1b[2m{}\x1b[0m", s) } else { s.to_string() } }

#[derive(Parser)]
#[command(name = "imf")]
#[command(about = "IMF package inspection tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect an IMF package directory
    Inspect {
        /// Path to the IMF package directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Output format
        #[arg(short, long, value_enum, default_value = "summary")]
        format: OutputFormat,
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

    /// Validate an IMF package structure
    Validate {
        /// Path to the IMF package directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Verify SHA-1 hashes of all assets against PKL (slow — reads every file)
        #[arg(long)]
        verify_hashes: bool,

        /// Output format (summary = human-readable, json = ValidationReport)
        #[arg(short, long, value_enum, default_value = "summary")]
        format: OutputFormat,

        /// Core constraints spec version selection
        #[arg(long, value_enum, default_value = "auto")]
        core_spec: CoreSpecVersion,

        /// Application profile selection (App2E + ST 2067-201 IAB plug-ins)
        #[arg(long, value_enum, default_value = "auto")]
        app2e_spec: App2eSpecVersion,

        /// Skip file manifest (existence/size) and MXF header checks.
        /// Validates XML structure only — useful for packages on slow or remote filesystems
        /// (e.g. S3 via MacFUSE) where disk I/O would be prohibitively slow.
        #[arg(long)]
        xml_only: bool,

        /// Always exit with code 0, even when validation errors are found (useful for CI
        /// pipelines that collect results without failing the build)
        #[arg(long)]
        exit_zero: bool,

        /// Path to a JSON rules config file with ESLint-style severity overrides.
        /// Example: { "SegmentDuration": "warn", "DanglingEssenceDescriptor": "off" }
        #[arg(long, value_name = "PATH")]
        rules_config: Option<PathBuf>,
    },

    /// Export a full report (source asset + validation + optional delivery comparison)
    Export {
        /// Path to the IMF package directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Path to ancestor IMP directory (for supplemental packages)
        #[arg(long)]
        ancestor: Option<PathBuf>,

        /// Path to delivery spec JSON file
        #[arg(long)]
        delivery_spec: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum OutputFormat {
    Summary,
    Json,
    Detailed,
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
        Commands::Inspect { path, format } => {
            inspect_package(&path, format)?;
        }
        Commands::Cpl { path, uuid } => {
            show_cpl(&path, uuid)?;
        }
        Commands::Validate {
            path,
            verify_hashes,
            xml_only,
            format,
            core_spec,
            app2e_spec,
            exit_zero,
            rules_config,
        } => {
            validate_package(&path, verify_hashes, xml_only, format, core_spec, app2e_spec, exit_zero, rules_config.as_deref())?;
        }
        Commands::Export { path, ancestor, delivery_spec } => {
            generate_report(&path, ancestor.as_deref(), delivery_spec.as_deref())?;
        }
    }

    Ok(())
}

fn inspect_package(path: &PathBuf, format: OutputFormat) -> Result<()> {
    let package = Imferno::parse(imferno_core::package::read_dir(path)?)?;
    let inspection = package.inspect();

    match format {
        OutputFormat::Summary => {
            println!("IMF Package: {}", inspection.path.display());
            println!("============");
            println!("Volume Index: {}", inspection.volume_index);
            println!("Asset Map ID: {}", inspection.asset_map_id);
            println!("Total Assets: {}", inspection.asset_count);
            println!("CPL Count: {}", inspection.cpl_count);

            if let Some(ref main_cpl) = inspection.main_cpl {
                println!("\nMain CPL:");
                println!("  Title: {}", main_cpl.title);
                println!("  Kind: {}", main_cpl.kind);
                println!("  ID: {}", main_cpl.id);
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&inspection)?);
        }
        OutputFormat::Detailed => {
            println!("IMF Package Details");
            println!("===================");
            println!("Path: {}", inspection.path.display());
            println!("Volume Index: {}", inspection.volume_index);
            println!("\nAsset Map:");
            println!("  ID: {}", inspection.asset_map_id);
            println!("  Issue Date: {}", inspection.asset_map_issue_date);
            if let Some(ref issuer) = inspection.asset_map_issuer {
                println!("  Issuer: {}", issuer);
            }
            if let Some(ref creator) = inspection.asset_map_creator {
                println!("  Creator: {}", creator);
            }

            println!("\nAssets ({}):", inspection.asset_count);
            for asset in &package.asset_map.asset_list.assets {
                println!("  - {}", asset.id);
                for chunk in &asset.chunk_list.chunks {
                    println!("    └─ {}", chunk.path);
                }
            }

            println!("\nComposition Playlists ({}):", inspection.cpl_count);
            let cpls = package.list_cpls();
            for cpl in &cpls {
                println!("  CPL: {}", cpl.id);
                println!("    Title: {}", cpl.title);
                println!("    Kind: {}", cpl.kind);
                println!("    Issue Date: {}", cpl.issue_date);
                if let Some(ref issuer) = cpl.issuer {
                    println!("    Issuer: {}", issuer);
                }
                println!("    Segments: {}", cpl.segments);
            }
        }
    }

    Ok(())
}

fn show_cpl(path: &PathBuf, uuid: Option<String>) -> Result<()> {
    let package = Imferno::parse(imferno_core::package::read_dir(path)?)?;

    let cpl_uuid = if let Some(uuid) = uuid {
        uuid
    } else {
        let inspection = package.inspect();
        if let Some(ref main_cpl) = inspection.main_cpl {
            main_cpl.id.clone()
        } else {
            return Err(anyhow::anyhow!("No CPLs found in package"));
        }
    };

    let details = package.get_cpl_details(&cpl_uuid)
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
        println!("  Segment {}: {} ({} sequences)", i + 1, segment.id, segment.sequence_count);
    }

    Ok(())
}

fn validate_package(
    path: &PathBuf,
    verify_hashes: bool,
    xml_only: bool,
    format: OutputFormat,
    core_spec: CoreSpecVersion,
    app2e_spec: App2eSpecVersion,
    exit_zero: bool,
    rules_config_path: Option<&std::path::Path>,
) -> Result<()> {
    let rules: imferno_core::package::RulesConfig = match rules_config_path {
        Some(p) => {
            let json = std::fs::read_to_string(p)
                .with_context(|| format!("Cannot read rules config: {}", p.display()))?;
            serde_json::from_str(&json)
                .with_context(|| format!("Invalid rules config JSON: {}", p.display()))?
        }
        None => Default::default(),
    };

    let color = use_color() && !matches!(format, OutputFormat::Json);

    if !matches!(format, OutputFormat::Json) {
        println!("Validating IMF package: {}", c_bold(&path.display().to_string(), color));
    }

    // Parse
    let parse_result = imferno_core::package::read_dir(path)
        .and_then(|files| Imferno::parse(files));
    let package = match parse_result {
        Ok(p) => {
            if !matches!(format, OutputFormat::Json) {
                println!("  {}  VOLINDEX.xml found", c_green("ok", color));
                println!("  {}  ASSETMAP.xml found", c_green("ok", color));
            }
            p
        }
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
                    .with_suggestion("Ensure the directory contains VOLINDEX.xml and ASSETMAP.xml.")
                );
                println!("{}", serde_json::to_string_pretty(&report)?);
                return Ok(());
            }
            return Err(e.into());
        }
    };

    let inspection = package.inspect();

    if !matches!(format, OutputFormat::Json) {
        println!("  {}  {} assets mapped", c_green("ok", color), inspection.asset_count);
        println!("  {}  {} CPL(s) parsed", c_green("ok", color), inspection.cpl_count);
        let scm_count = package.sidecar_composition_maps.len();
        if scm_count > 0 {
            let total_sidecars: usize = package.sidecar_composition_maps.values()
                .map(|s| s.sidecar_assets.len())
                .sum();
            println!("  {}  {} SCM(s) parsed, {} sidecar asset(s) declared", c_green("ok", color), scm_count, total_sidecars);
            for scm in package.sidecar_composition_maps.values() {
                for sa in &scm.sidecar_assets {
                    let cpl_labels: Vec<String> = sa.cpl_ids.iter()
                        .map(|id| format!("{:.8}", id.to_string()))
                        .collect();
                    let filename = package.asset_paths.get(&sa.id)
                        .and_then(|p| p.file_name())
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| sa.id.to_string());
                    println!("        {} → CPL(s): [{}]",
                        c_dim(&filename, color),
                        c_dim(&cpl_labels.join(", "), color));
                }
            }
        }
        let unref = package.unreferenced_assets();
        if !unref.is_empty() {
            println!("  {}  {} unreferenced asset(s) — no CPL track reference, no SCM declaration",
                c_yellow("info", color), unref.len());
            for asset in &unref {
                let filename = asset.chunk_list.chunks.first()
                    .map(|c| c.path.as_str())
                    .unwrap_or("(no path)");
                println!("        {}", c_dim(filename, color));
            }
        }
    }

    let options = ValidationOptions { rules, ..Default::default() };

    // Structural validation with configurable built-in spec/profile selection.
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

    let registry = ConfigurableValidatorRegistry::new(ValidatorSelection {
        core_spec: core_spec_target,
        app_specs: app_spec_targets,
        ..Default::default()
    });

    let mut report = package.validate_package_structure_with_cpl_validator(|cpl| {
        validate_cpl_with_registry(cpl, &registry)
    }, xml_only);
    report = report.apply_rules(&options.rules);

    if !matches!(format, OutputFormat::Json) {
        let all_issues: Vec<_> = report.critical.iter()
            .chain(report.errors.iter())
            .chain(report.warnings.iter())
            .chain(report.info.iter())
            .collect();

        if !all_issues.is_empty() {
            println!("{}", c_bold("Validation findings:", color));

            for issue in &all_issues {
                let (label, colorize): (&str, fn(&str, bool) -> String) = match issue.severity {
                    Severity::Critical => ("error",   c_red),
                    Severity::Error    => ("error",   c_red),
                    Severity::Warning  => ("warning", c_yellow),
                    Severity::Info     => ("info",    c_cyan),
                };
                let location = if let Some(ref c) = issue.location.cpl_id {
                    c_dim(&format!(" [CPL:{}]", &c[..c.len().min(8)]), color)
                } else if let Some(ref f) = issue.location.file {
                    let fname = f.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                    c_dim(&format!(" [{}]", fname), color)
                } else {
                    String::new()
                };
                println!(
                    "  {} {}{} {}",
                    colorize(&format!("{:<7}", label), color),
                    c_bold(&issue.code, color),
                    location,
                    issue.message,
                );
                if let Some(ref s) = issue.suggestion {
                    println!("          {} {}", c_dim("→", color), c_dim(s, color));
                }
            }
        }
    }

    // File existence is already checked inside validate_package_structure_with_cpl_validator.
    // Only run the separate hash-verification pass when explicitly requested.
    if verify_hashes {
        if !matches!(format, OutputFormat::Json) {
            println!("Verifying file hashes (this may take a moment)...");
        }
        let hash_errs: Vec<_> = package.validate_file_hashes()
            .into_iter()
            .filter(|e| !matches!(e, imferno_core::package::FileValidationError::Missing { .. }))
            .collect();
        if hash_errs.is_empty() && !matches!(format, OutputFormat::Json) {
            println!("  {}  All PKL file hashes verified", c_green("ok", color));
        }
        for err in &hash_errs {
            report.add(ValidationIssue::new(
                Severity::Error,
                Category::Asset,
                "FILE-HASH-ERROR",
                err.to_string(),
            ));
            if !matches!(format, OutputFormat::Json) {
                println!("✗ {}", err);
            }
        }
    }

    if matches!(format, OutputFormat::Json) {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        let total_errors = report.critical.len() + report.errors.len();
        let total_warnings = report.warnings.len();
        if total_errors > 0 {
            let mut reasons = Vec::new();
            if total_errors > 0 { reasons.push(format!("{} error(s)", total_errors)); }
            if total_warnings > 0 { reasons.push(format!("{} warning(s)", total_warnings)); }
            println!("{} {}", c_red("failed", color), c_bold(&reasons.join(", "), color));
            if !exit_zero {
                return Err(anyhow::anyhow!("Validation failed"));
            }
        } else if total_warnings > 0 {
            println!("{}", c_yellow(&format!("valid  {} warning(s)", total_warnings), color));
        } else {
            println!("{}", c_green("valid", color));
        }
    }

    Ok(())
}

fn generate_report(path: &PathBuf, ancestor_path: Option<&std::path::Path>, delivery_spec_path: Option<&std::path::Path>) -> Result<()> {
    let package = Imferno::parse(imferno_core::package::read_dir(path)?)?;

    let ancestor = if let Some(anc_path) = ancestor_path {
        Some(Imferno::parse(imferno_core::package::read_dir(anc_path)?)?)
    } else {
        None
    };

    let delivery_spec = if let Some(spec_path) = delivery_spec_path {
        let json_str = std::fs::read_to_string(spec_path)?;
        let spec: imferno_core::package::DeliveryRequest = serde_json::from_str(&json_str)?;
        Some(spec)
    } else {
        None
    };

    let report = imferno_core::package::build_report(&package, ancestor.as_ref(), delivery_spec.as_ref())
        .map_err(|e| anyhow::anyhow!(e))?;

    println!("{}", serde_json::to_string_pretty(&report)?);

    Ok(())
}
