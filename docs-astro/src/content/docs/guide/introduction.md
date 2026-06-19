---
title: Introduction
description: What imferno is, what IMF is, and why this exists.
---

**imferno** is a Rust library for working with IMF (Interoperable Master Format) packages. It parses, validates, and inspects IMPs — the delivery format used by Netflix, Amazon, and major broadcasters for archival-quality content exchange.

## What is IMF?

IMF (Interoperable Master Format) is a family of SMPTE standards — ST-2067 — defining how finished audiovisual content is packaged for delivery and archive. An IMP (IMF Package) is a directory containing:

- **ASSETMAP.xml** — inventory of all files and their UUIDs
- **PKL** (Packing List) — SHA-1 hashes and sizes for integrity verification
- **CPL** (Composition Playlist) — timeline describing how track files compose a finished piece
- **MXF track files** — the actual essence: video, audio, subtitles

IMF is the successor to DCP for long-form content. It supports multiple versions of a title (different languages, territories, ratings) in a single package via supplemental CPLs — without duplicating essence.

## What imferno does

- Parses ASSETMAP, PKL, CPL, and VOLINDEX XML per the SMPTE ST-2067 spec
- Validates structural integrity against Core Constraints and Application profiles
- Inspects track composition, language tags, and application profiles
- Exports structured JSON reports with full type definitions
- Available as a native CLI, Rust crate, and WebAssembly module

## Ecosystem

| Package | Description |
|---|---|
| [`imferno`](https://www.npmjs.com/package/imferno) | CLI with prebuilt native binaries |
| [`@imferno/wasm`](https://www.npmjs.com/package/@imferno/wasm) | WebAssembly bindings for JS/TS |
| [`@imferno/node`](https://www.npmjs.com/package/@imferno/node) | Native Node.js bindings (filesystem + hash verification) |
| [`@imferno/schema`](https://www.npmjs.com/package/@imferno/schema) | JSON Schema definitions for structural validation |
| [`imferno-core`](https://crates.io/crates/imferno-core) | Rust crate with all parsing and validation logic |

## Standards coverage

| Standard | Description | Status |
|---|---|---|
| ST 429-9:2014 | Volume Index & Asset Map | Complete |
| ST 2067-2:2013, :2016, :2020 | Core Constraints & Packing List | Complete |
| ST 2067-3:2013, :2016, :2020 | Composition Playlist | Complete |
| ST 2067-9:2018 | Sidecar Composition Map | Complete |
| ST 2067-21:2020, :2023, :2025 | Application Profile #2E (UHD/HDR) | Complete |
| ST 2067-201:2019, :2021 | IAB (Immersive Audio Bitstream) | Complete |
| ST 2067-202:2022 | ISXD (Immersive Sound XML Data) Plug-in | Complete |
| ST 377-1:2011 | MXF file structure | Partial — full header metadata via RegXML (Preface, MaterialPackage, descriptors, sub-descriptors); no essence sample decoding |

## Why Rust?

imferno brings SMPTE ST-2067 correctness to Rust — with zero GC pauses, a native CLI, and WASM compilation for browser tooling.
