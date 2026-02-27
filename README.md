# imferno

SMPTE ST 2067 IMF parser and validator for Rust and WebAssembly.

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
| ST 429-8 | D-Cinema Packing List | Not implemented |
| ST 2067-100 | Output Profile List | Not implemented |
| ST 2067-203 | S-ADM Audio Plug-in | Not implemented |
| ST 377-41 | MXF MGA / S-ADM Virtual Tracks | Not implemented |
| ST 379-2 | MXF Generic Container | Not implemented |
| ST 422 | JPEG 2000 in MXF | Not implemented |

## Crates

| Crate | Description |
|---|---|
| [`imferno-core`](crates/imferno-core) | All parsing and validation logic |
| [`imferno`](crates/imferno) | Command-line tool |
| [`imferno-wasm`](crates/imferno-wasm) | WebAssembly bindings (published to npm) |

## Usage

```bash
# Validate an IMF package
imferno validate ./my-imp

# Build WASM and generate docs
cargo xtask build-docs
```

## Docs

https://jpwesselink.github.io/imferno

## License

MIT
