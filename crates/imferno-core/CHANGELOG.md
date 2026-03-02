# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
