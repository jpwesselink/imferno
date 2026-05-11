# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.4.0](https://github.com/jpwesselink/imferno/compare/imferno-v2.3.0...imferno-v2.4.0) - 2026-05-11

### Added

- hash verification on by default, per-file output, CPL location enrichment

### Other

- Merge pull request #46 from jpwesselink/feat/cli-hash-default-and-cpl-location
- rustfmt

## [2.1.0](https://github.com/jpwesselink/imferno/compare/imferno-v2.0.0...imferno-v2.1.0) - 2026-04-29

### Added

- *(storage)* introduce Storage trait and unified read() entry point ([#35](https://github.com/jpwesselink/imferno/pull/35))
- add --rule CLI flag for inline severity overrides
- glow animates right-to-left (toward the leading edge)
- switch to subtle glow_effect for progress bars
- cycle_effect for smooth sliding gradient, 3x speed, 80ms ticker
- chromakopia glow animation on progress bars, plain label
- neon glow animation on [hashing] label via chromakopia glow_effect
- animated fire gradient glow on hashing progress bars
- sort smallest files first, add --hash-concurrency flag
- fixed-width filename column with per-file progress bar alongside
- multi-line per-file progress display for parallel hash verification
- live fire gradient progress bar during parallel hash verification
- parallel file hashing with tokio (8 concurrent files)
- streaming hash with per-MB progress — no more frozen progress bar
- fire gradient animation on hash verification progress bar 🔥
- progress indicator with chromakopia gradient for hash verification
- format_validation_result with text, markdown, and CSV output
- rename --xml-only to --skip-disk-checks
- remove export and report commands from CLI
- CLI validate uses validate() API, deprecate export/report

### Fixed

- address all code review findings on PR #32
- same text color for queued and hashing files
- use fixed-width status labels [matched] [hashing] [queued] [mismatch]
- remove redundant size after progress bar
- middle-truncate filenames, bar on right, no bar when done
- use ANSI clear-line escape to properly wipe progress bar
- plain 'hashing' label, no flame emoji or gradient
- widen progress line clear to 120 chars
- smooth fire gradient on progress bar — no more color blocks
- remove bin field from platform packages to fix npx resolution

### Other

- split top nav into Guide + Reference ([#37](https://github.com/jpwesselink/imferno/pull/37))
- rustfmt

## [2.0.0](https://github.com/jpwesselink/imferno/compare/imferno-v1.1.0...imferno-v2.0.0) - 2026-03-04

### Added

- [**breaking**] update CLI doc comments for v2 API ([#14](https://github.com/jpwesselink/imferno/pull/14))
- unified buildReport API, timeline sequences, and docs playground overhaul

### Fixed

- update WASM tests and wrapper for buildReport API ([#9](https://github.com/jpwesselink/imferno/pull/9))

## [1.0.1](https://github.com/jpwesselink/imferno/compare/imferno-v1.0.0...imferno-v1.0.1) - 2026-03-02

### Fixed

- update CLI help test to match renamed about text and export subcommand

### Other

- update documentation and fix consistency issues

## [1.0.0] - 2026-02-28

### Added

- `--rules-config` flag for ESLint-style severity overrides
- `--xml-only` flag for skipping disk checks
- `--exit-zero` flag for CI pipelines

### Changed

- stable release matching imferno-core v1.0.0

## [0.1.5](https://github.com/jpwesselink/imferno/compare/imferno-v0.1.4...imferno-v0.1.5) - 2026-02-27

### Fixed

- resolve all clippy warnings across workspace

## [0.1.4](https://github.com/jpwesselink/imferno/compare/imferno-v0.1.3...imferno-v0.1.4) - 2026-02-27

### Other

- update README and docs for @imferno npm ecosystem

## [0.1.3](https://github.com/jpwesselink/imferno/compare/imferno-v0.1.2...imferno-v0.1.3) - 2026-02-27

### Added

- migrate npm packages to @imferno scope
- rename `report` subcommand to `export`

### Other

- add READMEs to platform binary packages pointing to imferno

## [0.1.2](https://github.com/jpwesselink/imferno/compare/imferno-v0.1.1...imferno-v0.1.2) - 2026-02-27

### Fixed

- move npm binary scaffolds into crates/imferno/

## [0.1.1](https://github.com/jpwesselink/imferno/compare/imferno-v0.1.0...imferno-v0.1.1) - 2026-02-27

### Fixed

- add readme to crates, fix npm publish trigger

## [0.1.0](https://github.com/jpwesselink/imferno/releases/tag/imferno-v0.1.0) - 2026-02-27

### Added

- initial release of imferno-core, imferno, and imferno-wasm
