# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
