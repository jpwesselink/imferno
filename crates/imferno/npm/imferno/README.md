# imferno

SMPTE ST 2067 IMF validator. Prebuilt native binaries for all major platforms.

## Install

```bash
npm install -g imferno
```

Or use directly with npx:

```bash
npx imferno@latest validate ./my-imf-package
```

## Usage

```bash
# Validate an IMF package (hashes verified by default)
imferno validate ./path/to/imf/package

# Show detailed CPL information
imferno cpl ./path/to/imf/package

# JSON output
imferno validate ./path/to/imf/package --format json

# Skip hash verification
imferno validate ./path/to/imf/package --skip-hashes

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
