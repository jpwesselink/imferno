---
title: imferno Validation Codes
description: Validation codes emitted by imferno itself (not tied to a specific SMPTE spec).
---

These codes are emitted by imferno's package-level logic for conditions that don't map to a specific SMPTE spec clause.

## Imferno

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `IMFERNO:Package/UnreferencedAsset` | Asset is present in the AssetMap but not referenced by any CPL Virtual Track and has no SCM declaration. Likely a sidecar essence without an SCM. | INFO | Structure |
| `IMFERNO:Package/UnlistedEssence` | MXF file is present in the package directory but not listed in the AssetMap. The file is invisible to any conforming IMF reader. | WARNING | Structure |

