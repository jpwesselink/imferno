# imferno

SMPTE ST 2067 IMF parser and validator for Rust, Node.js, and the browser.

## Packages

### Rust crates (crates.io)

| Crate | Description |
|---|---|
| [`imferno-core`](https://crates.io/crates/imferno-core) | All parsing and validation logic |
| [`imferno`](https://crates.io/crates/imferno) | Command-line tool |

### npm packages

| Package | Description |
|---|---|
| [`imferno`](https://www.npmjs.com/package/imferno) | CLI — prebuilt native binaries for all platforms |
| [`@imferno/wasm`](https://www.npmjs.com/package/@imferno/wasm) | WebAssembly bindings for JS/TS |
| [`@imferno/schema`](https://www.npmjs.com/package/@imferno/schema) | JSON Schema definitions for all IMF types |

Platform binaries (installed automatically via `imferno`):

| Package | Platform |
|---|---|
| `@imferno/linux-x64-gnu` | Linux x64 |
| `@imferno/linux-arm64-gnu` | Linux ARM64 |
| `@imferno/darwin-x64` | macOS x64 |
| `@imferno/darwin-arm64` | macOS ARM64 |
| `@imferno/win32-x64-msvc` | Windows x64 |
| `@imferno/win32-arm64-msvc` | Windows ARM64 |

## Install

```bash
# CLI via npm
npm install -g imferno

# WASM bindings
npm install @imferno/wasm

# JSON schemas for validating imferno output
npm install @imferno/schema

# Rust crate
cargo add imferno-core
```

## Usage

### CLI

```bash
# Validate an IMF package
imferno validate ./my-imp

# Export a full report (JSON)
imferno export ./my-imp

# Inspect package structure
imferno inspect ./my-imp
```

### JavaScript / TypeScript

```javascript
import { validatePackage, extractSourceAsset } from '@imferno/wasm';

const report = await validatePackage({
  'ASSETMAP.xml': assetmapXml,
  'PKL_abc.xml': pklXml,
  'CPL_def.xml': cplXml,
});
```

### JSON Schema validation

```javascript
import Ajv from 'ajv';
import { imfReport } from '@imferno/schema';

const ajv = new Ajv();
const validate = ajv.compile(imfReport);

const data = JSON.parse(imfernoExportOutput);
if (!validate(data)) console.error(validate.errors);
```

### Rust

```rust
use imferno_core::package::{read_dir, Imferno, ValidationOptions};

let files = read_dir("./my-imp")?;
let report = Imferno::parse_and_validate(files, &ValidationOptions::default());

for issue in &report.errors {
    eprintln!("[{}] {}", issue.code, issue.message);
}
```

## Standards coverage

| Standard | Title | Status |
|---|---|---|
| ST 429-9 | Volume Index / Asset Map | Complete |
| ST 2067-2 | Core Constraints & Packing List | Complete |
| ST 2067-3 | Composition Playlist | Complete |
| ST 2067-9 | Sidecar Composition Map | Complete |
| ST 2067-21 | Application #2E (UHD/HDR) | Complete |
| ST 2067-201 | IAB Level 0 Plug-in | Complete |
| ST 2067-202 | ISXD Plug-in | Complete |
| ST 377-1 | MXF File Format | Partial — header partition only |

## Docs

https://jpwesselink.github.io/imferno

## License

MIT
