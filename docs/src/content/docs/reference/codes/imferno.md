---
title: imferno Tool Codes
description: Tool-level observation codes emitted by imferno (not derived from any SMPTE spec).
---

Codes in the `IMFERNO:` namespace are tool-level observations emitted by imferno itself. They are not normative violations against any SMPTE standard — they surface structural findings that may indicate a misconfigured or incomplete package.

## ImfernoCode

| Code | Description | Default Severity | Category |
|------|-------------|-----------------|----------|
| `IMFERNO:Package/UnreferencedAsset` | Asset is present in the AssetMap but not referenced by any CPL Virtual Track and has no SCM declaration. Likely a sidecar essence (e.g. Dolby Atmos MXF) delivered without an accompanying SCM document. | INFO | Structure |
| `IMFERNO:Package/UnlistedEssence` | MXF file is present in the package directory but not listed in the AssetMap. The file is invisible to any conforming IMF reader. Not emitted during `--xml-only` validation. | WARNING | Structure |
