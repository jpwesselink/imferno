---
title: Standards Coverage
description: Which SMPTE ST-2067 standards imferno implements and to what degree.
---

| Standard | Title | Editions | Coverage |
|---|---|---|---|
| ST 429-9 | Volume Index / Asset Map | 2014 | Complete |
| ST-2067-2 | Core Constraints and Packing List | 2013 / 2016 / 2020 | Complete |
| ST-2067-3 | Composition Playlist | 2013 / 2016 / 2020 | Complete |
| ST-2067-21 | Application #2E (UHD/HDR) | 2020 / 2023 / 2025 | Complete |
| ST-2067-201 | IAB Level 0 Plug-in | 2019 / 2021 | Complete |
| ST 377-1 | MXF File Format | 2011 | Partial — header partition only; no KLV traversal or essence decoding |
| ST-2067-9 | Sidecar Composition Map | 2018 | Complete |
| ST 429-8 | D-Cinema Packing List | 2007 | Not implemented |
| ST-2067-100 | Output Profile List | 2014 | Not implemented |
| ST-2067-202 | ISXD Plug-in | 2022 | Complete |
| ST-2067-203 | S-ADM Audio Plug-in | 2023 | Not implemented |
| ST 377-41 | MXF MGA / S-ADM Virtual Tracks | — | Not implemented |
| ST 379-2 | MXF Generic Container | 2010 | Not implemented |
| ST 422 | JPEG 2000 in MXF | 2014 | Not implemented |

Standards not listed are not implemented and not on the roadmap. The "Not implemented" entries above are those present in the [Netflix Photon](https://github.com/Netflix/photon) reference implementation.

## Known gaps

- MXF is inspected at the header partition level only — no frame-level decode or KLV traversal
- Multi-volume packages (volumeIndex > 1) parse but are not fully validated
- Output Profile List (OPL) transformation is not implemented

## Test corpus

Tests run against the [Netflix Photon](https://github.com/Netflix/photon) reference corpus. Covered profiles: App2, App2E 2020/2021, App5 (IMAX), IAB, ISXD, HT (JPEG 2000 High Throughput), MERIDIAN.
