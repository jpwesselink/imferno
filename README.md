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
| [`@imferno/node`](https://www.npmjs.com/package/@imferno/node) | Native Node.js bindings (filesystem + hash verification) |
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

# WASM bindings (browser + Node.js)
npm install @imferno/wasm

# Native Node.js bindings (filesystem access, hash verification)
npm install @imferno/node

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

### Node.js (native bindings)

```javascript
const { validatePath } = require('@imferno/node');

const { report, cpls, assetMap } = validatePath('./my-imp');
```

### Browser / WASM

```javascript
import { validate } from '@imferno/wasm';

const { report, cpls, assetMap } = await validate({
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

### Rust — Storage trait (URI-aware)

The `imferno_core::storage` module abstracts where IMF packages live —
local filesystem (always on) and S3 (behind the `aws-s3` feature flag).
Use `package::read` for cloud-aware reads:

```rust
use imferno_core::package::{read, Imferno};
use imferno_core::storage::{fs::FsStorage, StorageUri};

// Local filesystem (bare paths and file:// URIs both work)
let uri = StorageUri::parse("/path/to/imp")?;
let storage = FsStorage::new();
let files = read(&uri, &storage)?;
let package = Imferno::parse(files)?;
```

S3 (requires `--features aws-s3`):

```rust
# #[cfg(feature = "aws-s3")] {
use imferno_core::storage::s3::S3Storage;
let uri = imferno_core::storage::StorageUri::parse("s3://my-bucket/path/to/imp/")?;
let storage = S3Storage::from_default()?;  // uses default AWS credential chain
let files = imferno_core::package::read(&uri, &storage)?;
# }
```

The CLI accepts the same URI forms:

```bash
imferno validate /path/to/imp
imferno validate file:///path/to/imp
imferno --features aws-s3 validate s3://my-bucket/path/to/imp/
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

## Development

MXF test fixtures are stored as GitHub Release assets (not in git). Fetch them before running tests:

```bash
./scripts/fetch-test-data.sh
cargo test --workspace
```

## Docs

https://jpwesselink.github.io/imferno

## Sponsor

If imferno is useful to your workflow, consider [sponsoring the project](https://github.com/sponsors/jpwesselink).

## License

MIT
