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

# Verify SHA-1 hashes against PKL
imferno validate /path/to/your.imp --verify-hashes

# Custom rules config
imferno validate /path/to/your.imp --rules-config rules.json

# Export a full report (JSON)
imferno export /path/to/your.imp
```

See the [CLI Reference](/reference/cli/) for all commands and options.

### Rust

```toml
[dependencies]
imferno-core = "2.0"
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
import { validatePath, formatReport } from '@imferno/node';

const result = validatePath('./my-imp');

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
