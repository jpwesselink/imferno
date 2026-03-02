---
title: Validation
description: How imferno validates IMF packages and what the results mean.
---

Validation returns a `ValidationReport` — a structured result with issues sorted into four severity buckets.

## ValidationReport

```rust
pub struct ValidationReport {
    pub critical: Vec<ValidationIssue>, // package cannot be used
    pub errors:   Vec<ValidationIssue>, // spec non-conformance
    pub warnings: Vec<ValidationIssue>, // spec deviation, may still work
    pub info:     Vec<ValidationIssue>, // informational

    pub is_playable:  bool, // false when critical is non-empty
    pub is_compliant: bool, // false when critical or errors is non-empty
}
```

Each `ValidationIssue` carries a typed code string (e.g. `ST2067-2:2020:8.3/FileNotFound`), a human-readable message, severity, category, and optional location. See the [Validation Codes](/reference/codes/st2067-2/) reference for the full catalogue.

## Structural validation

Checks referential integrity and file presence — does not read MXF content.

- All assets in the PKL exist on disk
- Declared file sizes match
- CPL UUIDs resolve to known assets
- No duplicate UUIDs
- CPL structure conforms to ST 2067-2 and ST 2067-3 Core Constraints

```rust
use imferno_core::package::{Imferno, ValidationOptions, read_dir};

let files = read_dir("/path/to/your.imp")?;
let report = Imferno::parse_and_validate(files, &ValidationOptions::default());

println!("{}", report.summary());
// → "Validation Report: 0 critical, 1 errors, 2 warnings, 0 info"
```

## Hash validation

Streams every MXF and compares SHA-1/SHA-256 hashes against PKL declarations. Slow on large packages; not available in WASM.

```rust
use imferno_core::package::{Imferno, ValidationOptions, read_dir};

let files = read_dir("/path/to/your.imp")?;
let pkg = Imferno::parse(files)?;
let report = pkg.validate_hashes(&ValidationOptions::default());
```

## Severity overrides

Every validation rule can be set to `off`, `info`, `warn`, `error`, or `critical`. See [Configuration](/guide/config/) for details.

## CLI

```sh
# Structural validation, human-readable
imferno validate /path/to/your.imp

# With hash verification
imferno validate /path/to/your.imp --verify-hashes

# JSON output (full ValidationReport)
imferno validate /path/to/your.imp --format json

# Export a full report (package metadata + validation + CPL analysis)
imferno export /path/to/your.imp
```
