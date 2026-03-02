# imferno-core

Rust library for parsing and validating SMPTE ST 2067 (IMF) packages. All parsing, validation, and inspection logic lives here.

Part of the [`imferno`](https://github.com/jpwesselink/imferno) ecosystem.

## Install

```toml
[dependencies]
imferno-core = "1.1"
```

## Usage

### Validate a package

```rust
use imferno_core::package::{Imferno, ValidationOptions, read_dir};

let files = read_dir("./my-imp")?;
let report = Imferno::parse_and_validate(files, &ValidationOptions::default());

for issue in &report.errors {
    eprintln!("[{}] {}", issue.code, issue.message);
}
```

### Parse and inspect

```rust
use imferno_core::package::{Imferno, read_dir};

let files = read_dir("./my-imp")?;
let pkg = Imferno::parse(files)?;

for (id, cpl) in &pkg.composition_playlists {
    println!("{} — {}", id, cpl.content_title);
}
```

### Parse individual files

```rust
use imferno_core::cpl::parse_cpl;
use imferno_core::assetmap::{parse_assetmap, parse_pkl, parse_volindex};

let cpl = parse_cpl(cpl_xml)?;
let asset_map = parse_assetmap(assetmap_xml)?;
let pkl = parse_pkl(pkl_xml)?;
let volindex = parse_volindex(volindex_xml)?;
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

MXF test fixtures are stored as GitHub Release assets. Fetch them before running tests:

```bash
./scripts/fetch-test-data.sh
cargo test -p imferno-core
```

## License

MIT
