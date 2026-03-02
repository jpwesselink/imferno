---
title: API Reference
description: imferno-core public API — Imferno, ValidationReport, parsers.
tableOfContents: true
---

## `read_dir`

Reads the XML documents from an IMF package directory into a filename → content map. MXF essence files are not loaded.

```rust
use imferno_core::package::read_dir;

let files: HashMap<String, String> = read_dir("/path/to/your.imp")?;
```

---

## `Imferno`

The central type. Holds the fully parsed in-memory representation of an IMF package.

### Parse

```rust
// Parse from a file map (works on disk, in WASM, and in tests)
Imferno::parse(files: HashMap<String, String>) -> Result<Self>
```

### Parse and validate

```rust
// Parse + validate in one call — the most common entry point
Imferno::parse_and_validate(files: HashMap<String, String>, options: &ValidationOptions) -> ValidationReport
```

### Validate

```rust
// Structural check — no MXF reads
fn validate(&self, options: &ValidationOptions) -> ValidationReport

// Structural + stream every MXF for hash verification. Not available in WASM.
fn validate_hashes(&self, options: &ValidationOptions) -> ValidationReport
```

### Inspect

```rust
fn inspect(&self) -> PackageInspection
fn list_cpls(&self) -> Vec<CplSummary>
fn get_cpl(&self, uuid: ImfUuid) -> Option<&CompositionPlaylist>
fn get_main_cpl(&self) -> Option<&CompositionPlaylist>
fn get_asset_path(&self, uuid: ImfUuid) -> Option<&PathBuf>
fn analyze_tracks(&self) -> Vec<TrackAnalysis>
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

## `PackageInspection`

```rust
pub struct PackageInspection {
    pub path:                 PathBuf,
    pub volume_index:         u32,
    pub asset_map_id:         String,
    pub asset_count:          usize,
    pub cpl_count:            usize,
    pub cpl_uuids:            Vec<String>,
    pub main_cpl:             Option<CplSummary>,
    pub asset_map_issuer:     Option<String>,
    pub asset_map_creator:    Option<String>,
    pub asset_map_issue_date: String,
}
```

---

## `CplSummary`

```rust
pub struct CplSummary {
    pub id:         String,
    pub title:      String,
    pub kind:       String,
    pub issue_date: String,
    pub segments:   usize,
    pub issuer:     Option<String>,
    pub creator:    Option<String>,
    pub annotation: Option<String>,
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

For the full list of codes see [Validation Codes](/reference/codes/st2067-2/).

---

## Low-level parsers

All parsers live in `imferno_core` submodules:

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
