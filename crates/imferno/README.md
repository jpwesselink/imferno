# imferno

SMPTE ST 2067 IMF validator. Command-line tool for parsing and validating IMF packages.

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
# Validate an IMF package — accepts a local path, a file:// URI, or
# (with --features aws-s3) an s3://bucket/prefix/ URI
imferno validate ./my-imp
imferno validate file:///abs/path/to/my-imp
imferno validate s3://my-bucket/path/to/imp/

# Validate with JSON output
imferno validate ./my-imp --format json

# Verify SHA-1 hashes against PKL
imferno validate ./my-imp --verify-hashes

# Use a custom rules config
imferno validate ./my-imp --rules-config rules.json
```

S3 input requires building with the `aws-s3` feature:

```bash
cargo install imferno --features aws-s3
```

The S3 backend uses the default AWS credential chain (env vars, profile, or
EC2 IMDS). Only XML manifest files are fetched over the network — MXF
binaries are not downloaded.

## License

MIT
