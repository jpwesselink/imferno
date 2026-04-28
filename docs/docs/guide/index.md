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
| [`@imferno/node`](https://www.npmjs.com/package/@imferno/node) | Native NAPI bindings — call `validatePath` / `validateUri` / `buildReport` from JS/TS in-process. Filesystem access, hash verification, S3 input (with the `aws-s3` build). The fastest way to validate from a Node server. |

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

## Why Rust?

The choice was deliberate, not aesthetic. IMF tooling has traditionally lived
in C/C++ (asdcplib, Photon's JNI bridges, Qt-based IMFTool) where memory
errors and concurrency bugs are still routine. Rust gives the same native
performance with guarantees those stacks can't make.

### What Rust brings to a validator

- **Memory safety without GC** — no segfaults, no buffer overflows, no
  surprise allocator pauses while you're reading a 50 GB MXF. Predictable
  latency for ingest pipelines.
- **Exhaustive pattern matching on enums** — IMF is a long tail of *"shall"*
  constraints across multiple spec years. Each `match` over a CPL/AssetMap
  variant is checked at compile time; spec drift becomes a compile error,
  not a silent runtime bug.
- **`Result` and `Option` instead of exceptions** — every error path is part
  of the function signature. Validators can't accidentally throw out of a
  parser five frames deep; a missing element either becomes a structured
  validation issue or a typed `Err`, and the compiler enforces handling.
- **Zero-cost abstractions** — the high-level parser code (iterators,
  trait dispatch, generics) compiles down to the same loops and pointer
  arithmetic you'd write by hand in C, without the readability cost.
- **Fearless concurrency** — parallel hash verification across hundreds of
  MXFs uses `Send`/`Sync` and the type system to rule out data races at
  compile time. No "let's hope nobody mutates this" disclaimers.
- **Single static binary** — `cargo install imferno` (or one `npm install`
  for the prebuilt) gives you a zero-dependency executable. No JVM, no
  Python wheels, no shared libraries to ship.
- **First-class cross-compilation** — one `cargo build --target …` produces
  binaries for Linux/macOS/Windows on x64 + arm64 from any host.

### Three runtimes, one codebase

This is the part you can't easily replicate in any other language:

- **WASM** — `rustc` has a first-class `wasm32-unknown-unknown` target.
  The parser/validator compiles unchanged for browsers, Cloudflare Workers,
  Vercel Edge, Deno Deploy, and any sandboxed JS runtime. Distributed as
  [`@imferno/wasm`](https://www.npmjs.com/package/@imferno/wasm).
- **NAPI-RS** — the same Rust code, wrapped as a native Node.js addon via
  [napi-rs](https://napi.rs). In-process speed, no subprocess overhead, no
  child-process JSON marshalling. Distributed as
  [`@imferno/node`](https://www.npmjs.com/package/@imferno/node).
- **Native CLI** — a static binary, prebuilt for six platforms (Linux,
  macOS, Windows × x64, arm64). Distributed via Cargo *and* npm so shell
  scripts and CI jobs don't need a Rust toolchain.

The same validator runs in your browser dev tools, on a Lambda, in your
ingest pipeline, and on your laptop — without porting, without
reimplementation, and without three different sets of bugs.

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
