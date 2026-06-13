---
title: Getting Started
description: Parse and validate your first IMF package.
---

### CLI

```bash
# Install via npm (prebuilt binaries for all platforms)
npm install -g imferno

# Or via Cargo
cargo install imferno
```

```sh
# Validate an IMF package
imferno validate /path/to/your.imp

# JSON output
imferno validate /path/to/your.imp --format json

# Skip hash verification (hashes are verified by default)
imferno validate /path/to/your.imp --skip-hashes

# Override rule severity inline
imferno validate /path/to/your.imp --rule SegmentDuration=off

# Custom rules config
imferno validate /path/to/your.imp --rules-config rules.json
```

See the [CLI Reference](/reference/cli/) for all commands and options.

### Rust

```toml
[dependencies]
imferno-core = "3"
```

```rust
use imferno_core::package::{validate, read_dir, ValidationOptions};

let files = read_dir("/path/to/your.imp")?;
let result = validate(files, &ValidationOptions::default());

// result.package — full parsed Imferno struct
// result.validation — ValidationReport

if result.validation.is_compliant {
    println!("OK");
} else {
    for issue in &result.validation.critical {
        eprintln!("[critical] {} — {}", issue.code, issue.message);
    }
    for issue in &result.validation.errors {
        eprintln!("[error] {} — {}", issue.code, issue.message);
    }
}
```

For cloud-stored packages, use the URI-aware entry point — accepts `file://`,
bare paths, and (with `--features aws-s3`) `s3://bucket/prefix/`:

```rust
use imferno_core::package::{read, Imferno};
use imferno_core::storage::{fs::FsStorage, StorageUri};

let uri = StorageUri::parse("s3://my-bucket/path/to/imp/")?;
// let storage = FsStorage::new();
// or with the aws-s3 feature:
let storage = imferno_core::storage::s3::S3Storage::from_default()?;
let files = read(&uri, &storage)?;
let package = Imferno::parse(files)?;
```

See the [Rust API Reference](/reference/rust/) for the full API surface.

### WASM

```bash
npm install @imferno/wasm
```

ESM module powered by WebAssembly. Use it in any browser or bundler. All WASM functions are **async**.

```javascript
import { validate, formatReport } from '@imferno/wasm';

const result = await validate({
    'VOLINDEX.xml': volindexXml,
    'ASSETMAP.xml': assetmapXml,
    'PKL_abc.xml':  pklXml,
    'CPL_def.xml':  cplXml,
});

// Full parsed package
console.log(result.package.compositionPlaylists);

// Pretty-print the validation report
console.log(formatReport(result));

// Check programmatically
if (!result.validation.is_compliant) {
    for (const err of result.validation.errors) {
        console.error(err.code, err.message);
    }
}
```

See the [WASM API Reference](/reference/wasm/) for the full API surface.

### Node.js

```bash
npm install @imferno/node
```

Native bindings via NAPI — filesystem access, hash verification, and native speed. All Node.js functions are **synchronous**.

```javascript
import { validatePath, validateUri, formatReport } from '@imferno/node';

// Local FS path
const result = validatePath('./my-imp');

// Or via URI (file://, bare path, or — with aws-s3 build — s3:// URIs)
const result2 = validateUri('s3://my-bucket/path/to/imp/');

// Full parsed package
console.log(result.package.compositionPlaylists);

// Pretty-print the validation report
console.log(formatReport(result));

// Check programmatically
if (!result.validation.is_compliant) {
    for (const err of result.validation.errors) {
        console.error(err.code, err.message);
    }
}
```

#### Validate from strings

Same API as `@imferno/wasm` — no filesystem access:

```javascript
import { validate, formatReport } from '@imferno/node';

const result = validate({
    'ASSETMAP.xml': assetmapXml,
    'PKL_abc.xml': pklXml,
    'CPL_def.xml': cplXml,
});

console.log(formatReport(result));
```

See the [Node.js API Reference](/reference/node/) for the full API surface.
