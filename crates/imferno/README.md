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
# Validate an IMF package — accepts a local path, a file:// URI, or an
# s3://bucket/prefix/ URI
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

The npm-distributed CLI ships with S3 support enabled — no extra build flags
needed. `cargo install imferno` defaults to FS-only; pass
`--features aws-s3` to include the S3 backend.

The S3 backend uses the default AWS credential chain (env vars, profile, or
EC2 / ECS / EKS IMDS). Only XML manifest files are fetched over the
network — MXF essence files are not downloaded.

## License

MIT
