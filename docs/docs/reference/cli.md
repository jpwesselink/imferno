---
title: CLI Reference
description: "imferno CLI — validate, export, and report on IMF packages."
---

## Install

imferno is a native Rust binary — fast, offline, no runtime. Install via Cargo or npm:

```bash
# Via Cargo (native Rust binary)
cargo install imferno

# Via npm (prebuilt binaries for all platforms)
npm install -g imferno

# Or run without installing
npx imferno@latest validate ./my-package
```

---

## `imferno validate`

Validate an IMF package against SMPTE ST-2067.

```bash
imferno validate <PATH> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--verify-hashes` | Verify SHA-1 hashes of all assets against PKL (slow) |
| `--format <FORMAT>` | Output format: `summary` (default), `json` |
| `--core-spec <SPEC>` | Core spec version: `auto` (default), `v2013`, `v2016`, `v2020` |
| `--app2e-spec <SPEC>` | App profile: `auto` (default), `none`, `v2020`, `v2021`, `v2023` |
| `--xml-only` | Skip file manifest and MXF header checks (validates XML structure only) |
| `--exit-zero` | Always exit 0, even on validation errors (useful for CI) |
| `--rules-config <PATH>` | Path to a JSON rules config file |

### Examples

```bash
# Basic validation
imferno validate ./my-imp

# JSON output for CI pipelines
imferno validate ./my-imp --format json --exit-zero

# Full validation with hash verification
imferno validate ./my-imp --verify-hashes

# Force specific spec versions
imferno validate ./my-imp --core-spec v2020 --app2e-spec v2023

# XML-only mode (skip disk I/O — useful for remote filesystems)
imferno validate ./my-imp --xml-only

# Custom rules config
imferno validate ./my-imp --rules-config rules.json
```

---

## `imferno cpl`

Show detailed CPL information.

```bash
imferno cpl <PATH> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--uuid <UUID>` | CPL UUID (shows first CPL if not specified) |

### Examples

```bash
imferno cpl ./my-imp
imferno cpl ./my-imp --uuid urn:uuid:abcd1234-...
```

---

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Validation passed (or `--exit-zero` used) |
| `1` | Validation errors found |

---

## Rules config

Create a `rules.json` to override severity levels:

```json
{
    "ST2067-2:2020:8.3/FileNotFound": "critical",
    "ST2067-21:2023:7.1/AppIdMismatch": "off"
}
```

See [Configuration](/guide/config/) for details on available severities and typed code constants.
