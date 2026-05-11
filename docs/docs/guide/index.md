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

The parsing engine is written once, in Rust (`imferno-core`), and exposed
through every runtime that actually ships in production media stacks. Pick
the package that matches *where your code runs*, not how it's distributed.

### Rust

The source of truth. Use these directly when your application is itself in
Rust, or when you want to embed validation in another native tool.

| Crate | When to reach for it |
|---|---|
| [`imferno-core`](https://crates.io/crates/imferno-core) | Library — call from any Rust app: parse, validate, inspect, generate JSON reports. The `Storage` trait lets you point it at local FS or S3 (with the `aws-s3` feature). |
| [`imferno`](https://crates.io/crates/imferno) | Command-line tool installable via `cargo install imferno`. Same code as the npm CLI, just built from source on your machine. |

### Node.js

For server-side validation pipelines, ingest gates, CI hooks, and CLI use.
NAPI gives you native speed without a child-process boundary.

| Package | When to reach for it |
|---|---|
| [`imferno`](https://www.npmjs.com/package/imferno) (npm) | Drop-in CLI: `npm install -g imferno` ships prebuilt binaries for Linux/macOS/Windows on x64 + arm64. Perfect for CI and shell scripts that don't want a Rust toolchain. |
| [`@imferno/node`](https://www.npmjs.com/package/@imferno/node) | Native NAPI bindings - call `validatePath` / `validateUri` / `validate` from JS/TS in-process. Filesystem access, hash verification, S3 input (with the `aws-s3` build). The fastest way to validate from a Node server. |

### Browser / WebAssembly

Anywhere JS runs and you can't load a native binary — browsers, edge runtimes,
bundlers, sandboxed serverless platforms.

| Package | When to reach for it |
|---|---|
| [`@imferno/wasm`](https://www.npmjs.com/package/@imferno/wasm) | Pure-WASM build of the same engine. Pass XML strings in, get a validation report out. No filesystem access (it's a sandboxed runtime), so callers fetch the manifest XMLs themselves and hand them to `validate({})`. |

### Schemas (runtime-agnostic)

| Package | When to reach for it |
|---|---|
| [`@imferno/schema`](https://www.npmjs.com/package/@imferno/schema) | JSON Schema definitions for every type the engine emits — `ValidationReport`, `Imferno`, CPL, AssetMap, PKL, etc. Use to type-check imferno's output in any language with a JSON Schema validator. |

### How they relate

```
                     ┌──────────────────┐
                     │   imferno-core   │   Rust crate — engine
                     │   (parser +      │   (parses XML, validates,
                     │    validators)   │    storage trait)
                     └────────┬─────────┘
                              │ same code, three doors
            ┌─────────────────┼─────────────────┐
            ▼                 ▼                 ▼
       imferno CLI       @imferno/node      @imferno/wasm
       (cargo install     (NAPI bindings,    (WASM bindings,
        or npm install     in-process JS)     browser/edge)
        for binaries)
```

`@imferno/schema` describes the JSON these all emit, so consumers can
validate `imferno`'s output without depending on any of them.

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
| ST 377-1:2011 | MXF file structure | Partial — header partition only |

## Why Rust?

Four reasons Rust fits this particular problem.

**Type safety.** IMF entities — CPL, AssetMap, PKL, sidecar maps — translate
cleanly into Rust structs and enums. The compiler catches mismatches at
build time: when a new spec year adds a field, every consumer site that
needs to handle it stops compiling until you do. Renaming, refactoring, and
extending the parser are operations the type system makes safe instead of
operations a code review has to catch.

**Easy bindings to other languages.** Rust's FFI story is unusually good.
[napi-rs](https://napi.rs) wraps Rust functions as native Node.js modules
with a single `#[napi]` attribute — that's how
[`@imferno/node`](https://www.npmjs.com/package/@imferno/node) exists. The
same pattern applies elsewhere: PyO3 for Python, jni-rs for Java/Kotlin,
csbindgen for C#, UniFFI for Swift/Kotlin. When a user asks for "imferno in
language X", the answer is a binding crate, not a rewrite.

**Easy WASM.** `wasm32-unknown-unknown` is a first-class `rustc` target.
[`wasm-bindgen`](https://rustwasm.github.io/wasm-bindgen/) handles the JS
interop. The same engine ships to browsers, Cloudflare Workers, Vercel
Edge, Deno Deploy — anywhere a sandboxed JS runtime exists — distributed
as [`@imferno/wasm`](https://www.npmjs.com/package/@imferno/wasm).

**Cross-compiled binaries for every platform people actually use.**
`cargo build --target …` (with [`cross`](https://github.com/cross-rs/cross)
for the awkward triples) produces native binaries from a single CI matrix:
Linux x64 / arm64, macOS x64 / arm64 (Intel + Apple Silicon), Windows x64
/ arm64. No per-platform build farms, no Docker-in-Docker tricks, no
cross-toolchain hand-rolling. The CLI ships as a single static binary on
every one of them.
