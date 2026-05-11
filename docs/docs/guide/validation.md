---
title: Validation
description: How imferno validates IMF packages and what the results mean.
---

Validation returns a `ValidationReport` — a structured result with issues sorted into four severity buckets: `critical`, `error`, `warning`, and `info`.

## What gets checked

**Structural validation** — referential integrity and file presence (no MXF content reads):
- All assets in the PKL exist on disk
- Declared file sizes match
- CPL UUIDs resolve to known assets
- No duplicate UUIDs
- CPL structure conforms to ST 2067-2 and ST 2067-3 Core Constraints

**Application profile validation** — ST 2067-21 App2E, ST 2067-201 IAB, ST 2067-202 ISXD

**Hash validation** — SHA-1/SHA-256 against PKL declarations (on by default; use `--skip-hashes` to opt out; not available in WASM)

## Compliance flags

- `is_compliant` — `false` when `critical` or `errors` is non-empty
- `is_playable` — `false` when `critical` is non-empty

## Usage

### CLI

```sh
# Validate (hashes verified by default)
imferno validate /path/to/your.imp

# Skip hash verification for faster validation
imferno validate /path/to/your.imp --skip-hashes

# JSON output (full ValidationReport)
imferno validate /path/to/your.imp --format json

# CI mode — always exit 0
imferno validate /path/to/your.imp --format json --exit-zero
```

The CLI also accepts `file://` URIs and (with `--features aws-s3`)
`s3://bucket/prefix/` URIs:

```bash
imferno validate file:///abs/path/to/your.imp
imferno validate s3://my-bucket/path/to/your.imp/
```

See the [CLI Reference](/reference/cli/) for all options.

### Rust

#### Structural validation

```rust
use imferno_core::package::{Imferno, ValidationOptions, read_dir};

let files = read_dir("/path/to/your.imp")?;
let report = Imferno::parse_and_validate(files, &ValidationOptions::default());

println!("{}", report.summary());
// "Validation Report: 0 critical, 1 errors, 2 warnings, 0 info"
```

#### Hash validation

```rust
let files = read_dir("/path/to/your.imp")?;
let pkg = Imferno::parse(files)?;
let report = pkg.validate_hashes(&ValidationOptions::default());
```

#### Cloud storage (S3)

```rust
use imferno_core::package::{read, Imferno, ValidationOptions};
use imferno_core::storage::{s3::S3Storage, StorageUri};

let uri = StorageUri::parse("s3://my-bucket/path/to/imp/")?;
let storage = S3Storage::from_default()?;     // default AWS credential chain
let files = read(&uri, &storage)?;
let report = Imferno::parse_and_validate(files, &ValidationOptions::default());
```

Requires the `aws-s3` Cargo feature. Only XML manifest files are fetched
over the network; MXF essence files are not downloaded.

See the [Rust API Reference](/reference/rust/) for the full API surface.

### WASM

```typescript
import { buildReport } from '@imferno/wasm';

const report = await buildReport({
    'ASSETMAP.xml': assetmapXml,
    'PKL_abc.xml': pklXml,
    'CPL_def.xml': cplXml,
});

console.log(report.validation.is_compliant);
console.log(report.validation.errors);
```

See the [WASM API Reference](/reference/wasm/) for the full API surface.

### Node.js

#### Validate on disk

```javascript
import { buildReportFromPath } from '@imferno/node';

const report = buildReportFromPath('./my-imp');
console.log(report.validation.is_compliant);
```

#### Validate from strings

```javascript
import { buildReport } from '@imferno/node';

const report = buildReport({
    'ASSETMAP.xml': assetmapXml,
    'PKL_abc.xml': pklXml,
    'CPL_def.xml': cplXml,
});
```

See the [Node.js API Reference](/reference/node/) for the full API surface.

## Validation codes

Each issue carries a typed code string (e.g. `ST2067-2:2020:8.3/FileNotFound`), a human-readable message, severity, category, and optional location.

See the [Validation Codes](/guide/codes/) reference for the full catalogue.
