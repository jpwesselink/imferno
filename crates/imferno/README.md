# imferno

SMPTE ST 2067 IMF validator and inspector. Command-line tool for parsing, validating, and inspecting IMF packages.

Part of the [`imferno`](https://github.com/jpwesselink/imferno) ecosystem.

## Install

```bash
# Via Cargo
cargo install imferno

# Via npm (prebuilt binaries)
npm install -g imferno
```

## Usage

```bash
# Validate an IMF package
imferno validate ./my-imp

# Validate with JSON output
imferno validate ./my-imp --format json

# Verify SHA-1 hashes against PKL
imferno validate ./my-imp --verify-hashes

# Use a custom rules config
imferno validate ./my-imp --rules-config rules.json

# Export a full report (JSON)
imferno export ./my-imp

# Inspect package structure
imferno inspect ./my-imp
```

## License

MIT
