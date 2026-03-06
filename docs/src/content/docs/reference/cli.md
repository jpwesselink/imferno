---
title: CLI Reference
description: "imferno CLI — validate, export, and report on IMF packages."
tableOfContents: true
---

## Quick run

No global install needed. Run the latest version directly via npx:

```bash
npx imferno@latest validate ./my-package
```

## Install

```bash
# Via npm (prebuilt binaries for all platforms)
npm install -g imferno

# Via Cargo
cargo install imferno
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

## `imferno export`

Export a full report (package metadata, validation results, and CPL analysis) as JSON.

```bash
imferno export <PATH> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--ancestor <PATH>` | Path to ancestor IMP directory (for supplemental packages) |
| `--core-spec <SPEC>` | Core spec version: `auto` (default), `v2013`, `v2016`, `v2020` |
| `--app2e-spec <SPEC>` | App profile: `auto` (default), `none`, `v2020`, `v2021`, `v2023` |
| `--xml-only` | Skip file manifest and MXF header checks |
| `--rules-config <PATH>` | Path to a JSON rules config file |

### Examples

```bash
# Export full report as JSON
imferno export ./my-imp > report.json

# Export supplemental package with ancestor
imferno export ./supplemental-imp --ancestor ./original-imp

# Pipe into report for pretty-printing
imferno export ./my-imp | imferno report -
```

---

## `imferno report`

Pretty-print a previously exported JSON report.

```bash
imferno report <PATH>
```

`PATH` is a JSON file exported by `imferno export`, or `-` to read from stdin.

### Examples

```bash
# Print from file
imferno report report.json

# Pipe from export
imferno export ./my-imp | imferno report -

# Pipe from curl (remote report)
cat report.json | imferno report -
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
