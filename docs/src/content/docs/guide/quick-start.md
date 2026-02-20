---
title: Quick Start
description: Parse and validate your first IMF package.
---

## Rust

Add `imf-parser` to your `Cargo.toml`:

```toml
[dependencies]
imf-parser = { git = "https://github.com/jpwesselink/imferno" }
```

### Validate a package

```rust
use imf_parser::{Imferno, ValidationOptions, read_dir};

let files = read_dir("/path/to/your.imp")?;
let report = Imferno::parse_and_validate(files, &ValidationOptions::default());

if report.is_compliant {
    println!("OK");
} else {
    for issue in &report.critical {
        eprintln!("[critical] {} — {}", issue.code, issue.message);
    }
    for issue in &report.errors {
        eprintln!("[error] {} — {}", issue.code, issue.message);
    }
}
```

### Parse and inspect

```rust
use imf_parser::{Imferno, read_dir};

let files = read_dir("/path/to/your.imp")?;
let pkg = Imferno::parse(files)?;
let inspection = pkg.inspect();

println!("{} CPLs, {} assets", inspection.cpl_count, inspection.asset_count);

for cpl in pkg.list_cpls() {
    println!("  {} — {}", cpl.id, cpl.title);
}
```

### Hash verification

Streams MXF essence files and compares SHA-1/SHA-256 hashes against PKL declarations. Slow on large packages; not available in WASM.

```rust
use imf_parser::{Imferno, ValidationOptions, read_dir};

let files = read_dir("/path/to/your.imp")?;
let pkg = Imferno::parse(files)?;
let report = pkg.validate_hashes(&ValidationOptions::default());
```

## CLI

```sh
# Inspect
imf inspect /path/to/your.imp

# Validate (structural)
imf validate /path/to/your.imp

# Validate with hash verification
imf validate /path/to/your.imp --verify-hashes

# JSON output
imf validate /path/to/your.imp --format json
```

## WASM

```js
import init, { validatePackage } from '/wasm/imf_wasm.js';
await init();

const files = {
    'ASSETMAP.xml': assetmapXmlString,
    'PKL_xxx.xml':  pklXmlString,
    'CPL_xxx.xml':  cplXmlString,
};

const report = validatePackage(files, null);
console.log(report.is_compliant, report.critical, report.errors);
```
