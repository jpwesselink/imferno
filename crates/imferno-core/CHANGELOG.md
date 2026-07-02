# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [3.1.0](https://github.com/jpwesselink/imferno/compare/imferno-core-v3.0.1...imferno-core-v3.1.0) - 2026-07-02

### Added

- *(xsd)* emit Info notice when the schema pre-pass is skipped + e2e PatternInvalid pin

### Fixed

- *(validation)* audit P1 batch — invented IAB rule, timed-text UL byte, ADM-gated Mode A
- *(audio)* skip §5.3 WAVE-PCM rules on ST 2067-201 IAB essence (AUDIT-1)

### Other

- Merge branch 'main' into feat/fraunhofer-corpus-end-to-end
- Merge branch 'main' into feat/fraunhofer-corpus-end-to-end
- *(deps)* quick-xml 0.31 → 0.41 (RUSTSEC-2026-0194/-0195)
- refresh stale "MXF header partition only" claim

## [3.0.1](https://github.com/jpwesselink/imferno/compare/imferno-core-v3.0.0...imferno-core-v3.0.1) - 2026-06-19

### Fixed

- *(audio)* skip ST 2067-2 §5.3 WAVE-PCM rules on ST 2067-203 SADM/MGA tracks

## [2.4.1](https://github.com/jpwesselink/imferno/compare/imferno-core-v2.4.0...imferno-core-v2.4.1) - 2026-05-18

### Fixed

- *(core)* add migration example to ImfReport deprecation docs

## [2.4.0](https://github.com/jpwesselink/imferno/compare/imferno-core-v2.3.0...imferno-core-v2.4.0) - 2026-05-11

### Added

- hash verification on by default, per-file output, CPL location enrichment

### Fixed

- allow(deprecated) on test using build_report

### Other

- deprecate ImfReport, build_report(), and format_report()
- Merge pull request #46 from jpwesselink/feat/cli-hash-default-and-cpl-location
- rustfmt

## [2.1.0](https://github.com/jpwesselink/imferno/compare/imferno-core-v2.0.0...imferno-core-v2.1.0) - 2026-04-29

### Added

- *(storage)* introduce Storage trait and unified read() entry point ([#35](https://github.com/jpwesselink/imferno/pull/35))
- inline media info on track lines, show resource IDs
- show video/audio media info in CLI summary output
- add --rule CLI flag for inline severity overrides
- add read_s3() behind aws-s3 feature flag
- consistent camelCase serde naming for all public structs
- sort smallest files first, add --hash-concurrency flag
- multi-line per-file progress display for parallel hash verification
- live fire gradient progress bar during parallel hash verification
- parallel file hashing with tokio (8 concurrent files)
- streaming hash with per-MB progress — no more frozen progress bar
- progress indicator with chromakopia gradient for hash verification
- format_validation_result with text, markdown, and CSV output
- add validate() as primary API — returns { package, validation }
- expose full Imferno struct through WASM and NAPI
- add channel count and soundfield info to sequences
- add language tags to sequence report output
- barrel re-export validation codes and add usage docs ([#16](https://github.com/jpwesselink/imferno/pull/16))

### Fixed

- UnlistedEssence now detects all unlisted files, not just MXF
- replace unreachable!() in Imferno::empty() with direct struct construction
- address all code review findings on PR #32
- remove debug dump test that triggered clippy in CI
- plural serde renames + Tailwind CSS + contentTitle parsing
- use plural names for Vec fields in WASM serde output
- clippy — use &Path instead of &PathBuf in serialize_path
- pass language field through playground mapping + add test
- remaining review findings — type safety, dedup, and encapsulation
- address all code review findings across 5 agents

### Other

- *(clippy)* silence rust-1.95 lints surfaced after CI toolchain bump ([#36](https://github.com/jpwesselink/imferno/pull/36))
- rustfmt
- cargo fmt
- apply cargo fmt
- update all documentation for v2 API
- apply cargo fmt
- add tests for security fixes, new APIs, and serde round-trips

### Security

- fix path traversal and integer overflow vulnerabilities

### Added

- `storage` module exposing a `Storage` trait that abstracts package I/O.
- `FsStorage` (always available) and `S3Storage` (behind `aws-s3` feature) implementations.
- `StorageUri` parser supporting `file://`, `s3://`, and bare-path inputs.
- `package::read(uri, &dyn Storage)` as the unified trait-based entry point.
- `package::read_xml_files` (canonical name; `read` is the public alias).
- New CLI input forms: `imferno validate file://path`, `imferno validate /path`, and (with `--features aws-s3`) `imferno validate s3://bucket/prefix/`.
- New NAPI exports: `validateUri` and `buildReportFromUri`.

### Changed

- `package::read_dir` now delegates to `FsStorage` internally. Public signature and behavior unchanged.
- `package::read_s3` now delegates to `S3Storage` internally. Public signature and behavior unchanged.

## [2.0.0](https://github.com/jpwesselink/imferno/compare/imferno-core-v1.1.0...imferno-core-v2.0.0) - 2026-03-04

### Added

- [**breaking**] document v2 API entry points ([#13](https://github.com/jpwesselink/imferno/pull/13))
- unified buildReport API, timeline sequences, and docs playground overhaul
- typed validation code constants for Rust and TypeScript

### Fixed

- update WASM tests and wrapper for buildReport API ([#9](https://github.com/jpwesselink/imferno/pull/9))

### Other

- bump version references to 1.1.0

## [1.0.1](https://github.com/jpwesselink/imferno/compare/imferno-core-v1.0.0...imferno-core-v1.0.1) - 2026-03-02

### Other

- fix audit issues — severity values, export subcommand, test data instructions
- update documentation and fix consistency issues

## [1.0.0] - 2026-02-28

### Added

- unified `validate()` API for full IMF package validation
- `ValidationOptions.core_spec` and `app_specs` for spec version selection
- `ValidationOptions.skip_disk_checks` for XML-only validation
- `@imferno/node` native Node.js bindings via napi-rs
- `@imferno/schema` JSON Schema package for all IMF types
- ST 2067-202:2022 ISXD Plug-in support

### Changed

- stable release of all parsing and validation APIs

## [0.1.4](https://github.com/jpwesselink/imferno/compare/imferno-core-v0.1.3...imferno-core-v0.1.4) - 2026-02-27

### Fixed

- resolve all clippy warnings across workspace

### Other

- add validation code snapshot and stability tests

## [0.1.3](https://github.com/jpwesselink/imferno/compare/imferno-core-v0.1.2...imferno-core-v0.1.3) - 2026-02-27

### Other

- update README and docs for @imferno npm ecosystem

## [0.1.2](https://github.com/jpwesselink/imferno/compare/imferno-core-v0.1.1...imferno-core-v0.1.2) - 2026-02-27

### Added

- add @imferno/schema npm package with JSON Schema generation
- add JSON Schema support via schemars for all IMF types

## [0.1.1](https://github.com/jpwesselink/imferno/compare/imferno-core-v0.1.0...imferno-core-v0.1.1) - 2026-02-27

### Fixed

- add readme to crates, fix npm publish trigger

## [0.1.0](https://github.com/jpwesselink/imferno/releases/tag/imferno-core-v0.1.0) - 2026-02-27

### Added

- initial release of imferno-core, imferno, and imferno-wasm
