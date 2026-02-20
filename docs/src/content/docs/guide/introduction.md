---
title: Introduction
description: What imferno is, what IMF is, and why this exists.
---

**imferno** is a Rust library for working with IMF (Interoperable Master Format) packages. It parses, validates, and inspects IMPs — the delivery format used by Netflix, Amazon, and major broadcasters for archival-quality content exchange.

## What is IMF?

IMF (Interoperable Master Format) is a family of SMPTE standards — ST 2067 — defining how finished audiovisual content is packaged for delivery and archive. An IMP (IMF Package) is a directory containing:

- **ASSETMAP.xml** — inventory of all files and their UUIDs
- **PKL** (Packing List) — SHA-1 hashes and sizes for integrity verification
- **CPL** (Composition Playlist) — timeline describing how track files compose a finished piece
- **MXF track files** — the actual essence: video, audio, subtitles

IMF is the successor to DCP for long-form content. It supports multiple versions of a title (different languages, territories, ratings) in a single package via supplemental CPLs — without duplicating essence.

## What imferno does

- Parses ASSETMAP, PKL, and CPL XML per the SMPTE ST 2067 spec
- Validates structural integrity and file hashes
- Inspects track composition, language tags, and application profiles
- Exposes a high-level `ImfPackage` API for common queries
- Compiles to WebAssembly for browser use

## Standards coverage

| Standard | Description | Status |
|---|---|---|
| ST 429-9:2007 / ST 2067-9:2016, :2020 | Volume Index & Asset Map | Parsing |
| ST 2067-2:2013, :2016, :2020 | Core Constraints & Packing List | Validation |
| ST 2067-3:2013, :2016, :2020 | Composition Playlist | Parsing |
| ST 2067-21:2014 – :2023 | Application Profile #2E (UHD/HDR) | Validation |
| ST 2067-201:2019, :2021 | IAB (Immersive Audio Bitstream) | Validation |
| ST 2067-202:2022 | ISXD (Immersive Sound XML Data) Plug-in | Validation |
| ST 422 | JPEG 2000 in MXF | Profile |
| ST 377-4 | MCA Audio Labels | Parsing |
| ST 377-1 | MXF file structure | Partial |

## Why Rust?

imferno brings SMPTE ST 2067 correctness to Rust — with zero GC pauses, a native CLI, and WASM compilation for browser tooling.
