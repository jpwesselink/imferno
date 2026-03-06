# imferno

SMPTE ST-2067 IMF parser and validator for Rust, Node.js, and the browser.

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
| [`imferno-wasm`](https://www.npmjs.com/package/imferno-wasm) | WebAssembly bindings for JS/TS |
| [`@imferno/node`](https://www.npmjs.com/package/@imferno/node) | Native Node.js bindings (filesystem + hash verification) |

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
npm install imferno-wasm

# Native Node.js bindings (filesystem access, hash verification)
npm install @imferno/node

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

# Show CPL information
imferno cpl ./my-imp
```

### Node.js (native bindings)

```javascript
import { buildReportFromPath, formatReport } from '@imferno/node';

const report = buildReportFromPath('./my-imp');
console.log(formatReport(report));
```

### Browser / WASM

```javascript
import { buildReport, formatReport } from 'imferno-wasm';

const report = buildReport({
  'ASSETMAP.xml': assetmapXml,
  'PKL_abc.xml': pklXml,
  'CPL_def.xml': cplXml,
});

console.log(formatReport(report));
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
| ST-2067-2 | Core Constraints & Packing List | Complete |
| ST-2067-3 | Composition Playlist | Complete |
| ST-2067-9 | Sidecar Composition Map | Complete |
| ST-2067-21 | Application #2E (UHD/HDR) | Complete |
| ST-2067-201 | IAB Level 0 Plug-in | Complete |
| ST-2067-202 | ISXD Plug-in | Complete |
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
