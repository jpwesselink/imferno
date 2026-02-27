# imferno

SMPTE ST 2067 IMF validator and inspector. Prebuilt native binaries for all major platforms.

## Install

```bash
npm install -g imferno
```

Or use directly with npx:

```bash
npx imferno validate ./my-imf-package
```

## Usage

```bash
# Validate an IMF package
imferno validate ./path/to/imf/package

# Inspect package structure
imferno inspect ./path/to/imf/package

# Show detailed CPL information
imferno cpl <uuid> ./path/to/imf/package

# Generate a full report
imferno report ./path/to/imf/package

# Show version
imferno --version
```

## Supported Platforms

| Platform       | Architecture |
|----------------|--------------|
| Linux          | x64, arm64   |
| macOS          | x64, arm64   |
| Windows        | x64, arm64   |

## License

MIT
