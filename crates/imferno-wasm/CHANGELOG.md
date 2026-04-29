# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.1.0](https://github.com/jpwesselink/imferno/compare/imferno-wasm-v2.0.0...imferno-wasm-v2.1.0) - 2026-04-29

### Added

- add validate() as primary API — returns { package, validation }
- expose full Imferno struct through WASM and NAPI
- [**breaking**] document v2 API entry points ([#13](https://github.com/jpwesselink/imferno/pull/13))
- unified buildReport API, timeline sequences, and docs playground overhaul
- typed validation code constants for Rust and TypeScript
- [**breaking**] v1.0.0 — unified validate() API, remove SourceAsset/delivery
- migrate npm packages to @imferno scope
- initial release of imferno-core, imferno, and imferno-wasm

### Fixed

- remaining review findings — type safety, dedup, and encapsulation
- address all code review findings across 5 agents
- update WASM tests and wrapper for buildReport API ([#9](https://github.com/jpwesselink/imferno/pull/9))
- update docs to use imferno_wasm module and fix property name casing
- adjust ASSETMAP null check in test to handle undefined
- serialize validate() result as plain JS objects, not Maps
- resolve all clippy warnings across workspace
- update imferno-wasm README with correct API and usage
- ship prebuilt wasm binary in npm package
- update wasm package name and imports to imferno
- remove stale st2067-3 reference from wasm build script
- add version to imferno-core path dependencies

### Other

- rebuild WASM with CplSequence.language field
- update all documentation for v2 API
- release
- bump version references to 1.1.0
- bump all crates to v1.1.0
- release
- update documentation and fix consistency issues
- release
- *(imferno-wasm)* release v0.1.3
- update README and docs for @imferno npm ecosystem
- bump imferno-wasm to 0.1.3 for @imferno/wasm publish
- release
- release ([#2](https://github.com/jpwesselink/imferno/pull/2))
- add vitest suite for imferno-wasm and wire into CI
- bump imferno-wasm to 0.1.2
- bump imferno-wasm to 0.1.1
- release v0.1.0

## [2.0.0](https://github.com/jpwesselink/imferno/compare/imferno-wasm-v1.1.0...imferno-wasm-v2.0.0) - 2026-03-04

### Added

- [**breaking**] document v2 API entry points ([#13](https://github.com/jpwesselink/imferno/pull/13))
- unified buildReport API, timeline sequences, and docs playground overhaul
- typed validation code constants for Rust and TypeScript
- [**breaking**] v1.0.0 — unified validate() API, remove SourceAsset/delivery
- migrate npm packages to @imferno scope
- initial release of imferno-core, imferno, and imferno-wasm

### Fixed

- update WASM tests and wrapper for buildReport API ([#9](https://github.com/jpwesselink/imferno/pull/9))
- update docs to use imferno_wasm module and fix property name casing
- adjust ASSETMAP null check in test to handle undefined
- serialize validate() result as plain JS objects, not Maps
- resolve all clippy warnings across workspace
- update imferno-wasm README with correct API and usage
- ship prebuilt wasm binary in npm package
- update wasm package name and imports to imferno
- remove stale st2067-3 reference from wasm build script
- add version to imferno-core path dependencies

### Other

- bump version references to 1.1.0
- bump all crates to v1.1.0
- release
- update documentation and fix consistency issues
- release
- *(imferno-wasm)* release v0.1.3
- update README and docs for @imferno npm ecosystem
- bump imferno-wasm to 0.1.3 for @imferno/wasm publish
- release
- release ([#2](https://github.com/jpwesselink/imferno/pull/2))
- add vitest suite for imferno-wasm and wire into CI
- bump imferno-wasm to 0.1.2
- bump imferno-wasm to 0.1.1
- release v0.1.0

## [1.0.0] - 2026-02-28

### Added

- unified `validate(files, options?)` API replacing individual validator functions
- spec version selection via `coreSpec` and `app2eSpec` options
- ESLint-style rules config for severity overrides

### Removed

- `extractSourceAsset`, `compareDelivery` — removed in favor of unified validate API
- `validateCpl`, `validateCplWithSpecSelection` — replaced by `validate()`

## [0.1.3](https://github.com/jpwesselink/imferno/releases/tag/imferno-wasm-v0.1.3) - 2026-02-27

### Added

- migrate npm packages to @imferno scope
- initial release of imferno-core, imferno, and imferno-wasm

### Fixed

- update imferno-wasm README with correct API and usage
- ship prebuilt wasm binary in npm package
- update wasm package name and imports to imferno
- remove stale st2067-3 reference from wasm build script
- add version to imferno-core path dependencies

### Other

- update README and docs for @imferno npm ecosystem
- bump imferno-wasm to 0.1.3 for @imferno/wasm publish
- release
- release ([#2](https://github.com/jpwesselink/imferno/pull/2))
- add vitest suite for imferno-wasm and wire into CI
- bump imferno-wasm to 0.1.2
- bump imferno-wasm to 0.1.1
- release v0.1.0

## [0.1.2](https://github.com/jpwesselink/imferno/releases/tag/imferno-wasm-v0.1.2) - 2026-02-27

### Added

- initial release of imferno-core, imferno, and imferno-wasm

### Fixed

- update imferno-wasm README with correct API and usage
- ship prebuilt wasm binary in npm package
- update wasm package name and imports to imferno
- remove stale st2067-3 reference from wasm build script
- add version to imferno-core path dependencies

### Other

- add vitest suite for imferno-wasm and wire into CI
- bump imferno-wasm to 0.1.2
- bump imferno-wasm to 0.1.1
- release v0.1.0

## [0.1.0](https://github.com/jpwesselink/imferno/releases/tag/imferno-wasm-v0.1.0) - 2026-02-27

### Added

- initial release of imferno-core, imferno, and imferno-wasm
