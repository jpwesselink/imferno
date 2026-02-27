---
title: Quick Start
description: Parse and validate your first IMF package.
---

## CLI

```bash
npm install -g imferno
```

```sh
# Validate an IMF package
imferno validate /path/to/your.imp

# Export a full report (JSON)
imferno export /path/to/your.imp

# Inspect package structure
imferno inspect /path/to/your.imp
```

## JavaScript / TypeScript (WASM)

```bash
npm install @imferno/wasm
```

```javascript
import { validatePackage, extractSourceAsset } from '@imferno/wasm';

const files = {
    'ASSETMAP.xml': assetmapXmlString,
    'PKL_xxx.xml':  pklXmlString,
    'CPL_xxx.xml':  cplXmlString,
};

const report = await validatePackage(files);
console.log(report.isCompliant, report.errors);

// Extract source asset metadata from a CPL
const sourceAsset = await extractSourceAsset(cplXmlString);
```

## JSON Schema validation

Validate the structure of imferno's JSON output before processing it:

```bash
npm install @imferno/schema
```

```javascript
import Ajv from 'ajv';
import { imfReport } from '@imferno/schema';

const ajv = new Ajv();
const validate = ajv.compile(imfReport);

const data = JSON.parse(imfernoExportOutput);
if (!validate(data)) {
    console.error(validate.errors);
}
```

## Rust

Add `imferno-core` to your `Cargo.toml`:

```toml
[dependencies]
imferno-core = "0.1"
```

### Validate a package

```rust
use imferno_core::package::{Imferno, ValidationOptions, read_dir};

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
use imferno_core::package::{Imferno, read_dir};

let files = read_dir("/path/to/your.imp")?;
let pkg = Imferno::parse(files)?;
let inspection = pkg.inspect();

println!("{} CPLs, {} assets", inspection.cpl_count, inspection.asset_count);

for cpl in pkg.list_cpls() {
    println!("  {} — {}", cpl.id, cpl.title);
}
```
