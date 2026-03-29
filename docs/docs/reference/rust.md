---
title: Rust API
description: imferno-core public API — validate, Imferno, ValidationReport, parsers.
---

## `validate`

Parse and validate an IMF package in one call. This is the **recommended entry point**.

```rust
use imferno_core::package::{validate, read_dir, ValidationOptions};

let files = read_dir("/path/to/your.imp")?;
let result = validate(files, &ValidationOptions::default());

// Full parsed package
let cpls = &result.package.composition_playlists;

// Validation results
if result.validation.is_compliant {
    println!("OK");
} else {
    for issue in &result.validation.errors {
        eprintln!("[error] {} — {}", issue.code, issue.message);
    }
}
```

---

## `ValidationResult`

Returned by `validate()`. Contains the full parsed package and the validation report.

```rust
pub struct ValidationResult {
    pub package: Imferno,
    pub validation: ValidationReport,
}
```

---

## `read_dir`

Reads the XML documents from an IMF package directory into a filename-to-content map. MXF essence files are not loaded.

```rust
use imferno_core::package::read_dir;

let files: HashMap<String, String> = read_dir("/path/to/your.imp")?;
```

---

## `Imferno`

The central type. Holds the fully parsed in-memory representation of an IMF package.

### Parse (without validation)

```rust
use imferno_core::package::{Imferno, read_dir};

let files = read_dir("/path/to/your.imp")?;
let package = Imferno::parse(files)?;
```

Use `Imferno::parse()` when you need the parsed package without running validation. For most use cases, prefer `validate()` instead.

### Validate an existing package

```rust
// Structural check — no MXF reads
fn validate(&self, options: &ValidationOptions) -> ValidationReport

// Sequential hash verification (reads every file)
fn validate_file_hashes(&self) -> Vec<FileValidationError>

// Parallel hash verification with tokio
// Requires feature = "tokio"
fn hash_verification_size(&self) -> u64  // total bytes to hash
async fn validate_file_hashes_parallel(
    &self,
    concurrency: usize,
    progress: Arc<HashProgressTracker>,  // per-file progress tracking
) -> Vec<FileValidationError>
```

### Query

```rust
fn get_cpl(&self, uuid: ImfUuid) -> Option<&CompositionPlaylist>
fn get_main_cpl(&self) -> Option<&CompositionPlaylist>
fn get_cpl_details(&self, uuid: &str) -> Option<CplDetails>
fn get_asset_path(&self, uuid: ImfUuid) -> Option<&PathBuf>
fn analyze_tracks(&self) -> Vec<TrackAnalysis>
```

### Formatting

```rust
use imferno_core::package::{format_validation_result, FormatOptions, ReportFormat};

let result = validate(files, &opts);

// Plain text
let text = format_validation_result(&result, &FormatOptions::default());

// Markdown
let md = format_validation_result(&result, &FormatOptions {
    format: ReportFormat::Markdown,
    color: false,
});

// CSV (one row per issue)
let csv = format_validation_result(&result, &FormatOptions {
    format: ReportFormat::Csv,
    color: false,
});
```

---

## `ValidationOptions`

```rust
#[derive(Debug, Default, Clone)]
pub struct ValidationOptions {
    pub rules: RulesConfig,

    /// Core constraints spec version. None = auto-detect from CPL namespace.
    pub core_spec: Option<CoreSpecTarget>,

    /// Application profile spec versions. None = auto-detect from CPL.
    pub app_specs: Option<Vec<AppSpecTarget>>,

    /// Path for hash verification (native only). None = skip.
    #[cfg(not(target_arch = "wasm32"))]
    pub verify_hashes: Option<PathBuf>,

    /// Skip file manifest and MXF header checks (native only).
    #[cfg(not(target_arch = "wasm32"))]
    pub skip_disk_checks: bool,
}
```

---

## `ValidationReport`

```rust
pub struct ValidationReport {
    pub critical:     Vec<ValidationIssue>,
    pub errors:       Vec<ValidationIssue>,
    pub warnings:     Vec<ValidationIssue>,
    pub info:         Vec<ValidationIssue>,
    pub is_playable:  bool,
    pub is_compliant: bool,
    pub profile:      ValidationProfile,
    pub timestamp:    String,
}

impl ValidationReport {
    fn total_issues(&self) -> usize
    fn has_critical(&self) -> bool
    fn has_errors(&self) -> bool
    fn summary(&self) -> String
    fn merge(&mut self, other: ValidationReport)
}
```

---

## `ValidationIssue`

```rust
pub struct ValidationIssue {
    pub severity:   Severity,                 // Critical | Error | Warning | Info
    pub category:   Category,                 // Structure | Asset | Reference | Timing | ...
    pub location:   Location,
    pub code:       String,                   // e.g. "ST2067-2:2020:8.3/FileNotFound"
    pub message:    String,
    pub suggestion: Option<String>,
    pub context:    HashMap<String, String>,  // optional key/value annotations
}
```

For the full list of codes see the [Validation Codes](/guide/codes) reference.

---

## Low-level parsers

All parsers live in `imferno_core` submodules. Use these when you need to parse individual XML documents outside of a full package context.

### CPL

```rust
use imferno_core::cpl::parse_cpl;

let cpl = parse_cpl(&xml_str)?;
```

### ASSETMAP / PKL

```rust
use imferno_core::assetmap::{parse_assetmap, parse_pkl};

let asset_map = parse_assetmap(&xml_str)?;
let pkl = parse_pkl(&xml_str)?;
```

### VOLINDEX

```rust
use imferno_core::assetmap::parse_volindex;

let volindex = parse_volindex(&xml_str)?;
```
