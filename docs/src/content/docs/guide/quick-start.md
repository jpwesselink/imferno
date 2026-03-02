---
title: Quick Start
description: Parse and validate your first IMF package.
tableOfContents: true
---

## CLI

```bash
# Install via npm (prebuilt binaries for all platforms)
npm install -g imferno

# Or via Cargo
cargo install imferno
```

```sh
# Validate an IMF package
imferno validate /path/to/your.imp

# Validate with JSON output
imferno validate /path/to/your.imp --format json

# Validate with detailed output
imferno validate /path/to/your.imp --format detailed

# Verify SHA-1 hashes against PKL
imferno validate /path/to/your.imp --verify-hashes

# Use a custom rules config
imferno validate /path/to/your.imp --rules-config rules.json

# Export a full report (JSON)
imferno export /path/to/your.imp

# Inspect package structure
imferno inspect /path/to/your.imp
```

## Node.js (native bindings)

Native bindings with filesystem access, hash verification, and native speed.

```bash
npm install @imferno/node
```

### Validate a package on disk

```javascript
const { validatePath } = require('@imferno/node');

const { report, cpls, assetMap, packingLists, volumeIndex } = validatePath('./my-imp');

console.log('Compliant:', report.is_compliant);
console.log('Errors:', report.errors.length);
console.log('Warnings:', report.warnings.length);
console.log('CPLs:', cpls.length);
```

### Validate with options

```javascript
const result = validatePath('./my-imp', {
  coreSpec: 'v2020',
  app2eSpec: 'v2023',
  verifyHashes: true,
  rules: {
    'ST2067-21:2023:7.1/AppIdMismatch': 'error',
    'IMFERNO:Package/UnreferencedAsset': 'off',
  },
});
```

### Validate from strings (same as WASM)

```javascript
const { validate } = require('@imferno/node');

const result = validate({
  'ASSETMAP.xml': assetmapXml,
  'PKL_abc.xml': pklXml,
  'CPL_def.xml': cplXml,
});
```

### Parse individual files

```javascript
const { parseCpl, parseAssetmap, parsePkl, parseVolindex } = require('@imferno/node');

const cpl = parseCpl(cplXmlString);
const assetMap = parseAssetmap(assetmapXmlString);
const pkl = parsePkl(pklXmlString);
const volindex = parseVolindex(volindexXmlString);
```

## Browser / WASM

WebAssembly bindings for browser and Node.js. No filesystem access — pass XML strings directly.

```bash
npm install @imferno/wasm
```

```javascript
import { validate } from '@imferno/wasm';

const { report, cpls, assetMap } = await validate({
    'ASSETMAP.xml': assetmapXmlString,
    'PKL_xxx.xml':  pklXmlString,
    'CPL_xxx.xml':  cplXmlString,
});

console.log(report.is_compliant, report.errors);
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
imferno-core = "1.1"
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
