---
title: API Reference
description: imf-parser public API — Imferno, ValidationReport, low-level spec parsers.
---

## `read_dir`

Reads the XML documents from an IMF package directory into a filename → content map. MXF essence files are not loaded.

```rust
use imf_parser::read_dir;

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

    // Native only — set to stream MXF from this path for hash verification.
    // Always None in WASM.
    #[cfg(not(target_arch = "wasm32"))]
    pub verify_hashes: Option<PathBuf>,
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

## Low-level spec parsers

Each spec crate is independently usable.

### `st2067-3` — CPL

```rust
use st2067_3::{parse_cpl, CompositionPlaylist};

let cpl: CompositionPlaylist = parse_cpl(&xml_str)?;
```

### `st2067-2` — ASSETMAP / PKL

```rust
use st2067_2::{parse_assetmap, parse_pkl, AssetMap, PackingList};

let asset_map: AssetMap    = parse_assetmap(&xml_str)?;
let pkl:       PackingList = parse_pkl(&xml_str)?;
```

### `st429-9` — VOLINDEX

```rust
use st429_9::{parse_volindex, VolumeIndex};

let volindex: VolumeIndex = parse_volindex(&xml_str)?;
```

### `st377-1` — MXF

```rust
use st377_1::codes::St377_1_2011;
// Header-partition inspection only; no essence decoding.
```
