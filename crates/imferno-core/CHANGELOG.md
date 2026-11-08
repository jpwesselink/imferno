# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [4.0.0](https://github.com/jpwesselink/imferno/compare/imferno-core-v3.0.1...imferno-core-v4.0.0) - 2026-11-08

### Added

- *(diagnostics)* IMFERNO:Package/EditionConflict for namespace-vs-evidence contradictions
- *(validation)* [**breaking**] bounce CPLs that are not IMF Composition Playlists
- *(mxf)* [**breaking**] ST 377-1:2019 and ST 377-4:2021 editions, chosen from the dated references
- *(validation)* [**breaking**] ST 2067-3:2020 as its own edition, chosen and announced per CPL
- SARIF 2.1.0 output (core, CLI --format sarif, NAPI/WASM toSarif)
- *(core)* [**breaking**] report every finding from the rule tables, per edition
- *(core)* per-edition rule model and rule tables
- *(validation)* give SegmentDuration mismatches in edit units
- *(validation)* name the Segment by Id in SegmentDuration findings
- *(package)* require the Asset Map file to be named ASSETMAP.xml
- *(package)* report a CPL whose Id differs from its AssetMap asset Id
- *(bindings)* formatReport accepts the ValidationResult from validate()
- *(core)* JSON Schema for the full parsed package and ValidationResult
- *(validation)* report CPLs whose application profile was not validated
- *(assetmap)* check ST 429-9:2014 Annex A.2 path rules on disk
- *(mxf)* require Duration on Timeline Track structural components
- *(validation)* check APP2.HT.IRV parameter B limits (ST 2067-21:2026)
- *(validation)* validate each ST 2067-21 edition against its own text
- *(validation)* audit P3 batch — IAB §5.10.2/Annex C.2/§6.2 and ISXD §6/§9.3 gap rules, font/otf (AUDIT-18, AUDIT-21, AUDIT-13 partial)
- *(xsd)* emit Info notice when the schema pre-pass is skipped + e2e PatternInvalid pin

### Fixed

- *(validation)* canonical raw Resource form resolves namespaces, CDATA and references
- *(catalogue)* NotImfCompositionPlaylist rests on the shall sentences
- *(validation)* the IAB validator from get_validator chooses its edition per CPL
- *(mxf)* assumed ST 2067-2 editions are not evidence for the MXF editions
- *(validation)* an edition fixed by the namespace stays determined when evidence contradicts it
- *(validation)* the IAB edition follows the dated ST 2067-2 references
- *(mxf)* MXF editions from the referencing CPLs; MXF checks take their edition
- *(validation)* judge 2020 'identical' Resources on their raw XML
- *(package)* only Packing List namespaces name the package edition
- *(catalogue)* [**breaking**] MarkerLabelUnknown is shall/Error in every ST 2067-3 edition
- *(mxf)* read the "ST 377-4:2011" citation of ST 2067-2:2013/2016 as ST 377-4:2012
- *(validation)* the ST 2067-3 edition follows the dated ST 2067-2 and ST 2067-201 references
- *(validation)* [**breaking**] resolve the IAB plug-in edition like every other edition
- *(validation)* announce evidence that contradicts the namespace's edition
- *(validation)* [**breaking**] one edition resolution for the package and every CPL
- *(validation)* resolve the ApplicationIdentification namespace with a namespace-aware parser
- *(catalogue)* EssenceDescriptorList description cites §6.8, not §6.4.2
- *(catalogue)* ST 2067-2:2016 UnresolvedUuid cites only 2016 §8.3
- *(validation)* use the same core-edition choice in the configurable registry
- *(catalogue)* ground TimedTextMappingKindNot0x13 in ST 429-5 Annex A
- *(validation)* require the EssenceDescriptorList for ST 2067-2:2013 too
- *(validation)* allow EntryPoint == IntrinsicDuration
- *(catalogue)* remove IMFERNO/IntrinsicDurationZero and CompositionTimecodeRateMismatch
- *(catalogue)* ContentKindUnknown is an Error in every edition
- *(timed-text)* accept superset profile designators under ST 2067-2:2020
- *(catalogue)* describe the Table H.3 should-limit on HtIrvParameterBRecommended
- *(catalogue)* correct three strengths found in review
- *(validation)* re-emit ResourceList, ContentTitle and required-track rules
- *(mxf)* check the OP1a qualifier bits per ST 2067-5 §5.2.1
- *(catalogue)* verify ST 2067-21 clauses and strength per edition
- *(catalogue)* verify ST 2067-2 clauses and strength per edition
- *(catalogue)* verify ST 2067-201 clauses and strength per edition
- *(catalogue)* verify ST 2067-202 clauses and strength
- *(catalogue)* verify ST 2067-9 clauses and strength
- *(catalogue)* verify ST 377-4 clauses and strength
- *(catalogue)* verify ST 377-1 clauses and strength
- *(catalogue)* verify ST 429-9 clauses and strength
- *(catalogue)* verify ST 429-8 clauses and strength
- *(catalogue)* verify ST 2067-3 clauses and strength per edition
- 2013 Packing List Size/Hash under ST 429-8; announce the ST 429-9 edition
- package edition, 2013 delivery rules, verified rows, rule warnings, SARIF paths
- *(validation)* edition-exact CplIdAssetMapMismatch codes
- *(validation)* one DigitalSignature notice per package, signed CPLs only
- *(package)* report unlisted files also with --skip-disk-checks, as Info
- *(validation)* allow SourceDuration 0 on a Resource
- *(core)* drop the Tsify derive merged in from main
- *(package)* allow ".." in AssetMap paths that stay inside the package
- *(validation)* warn when ApplicationIdentification lists only plug-ins
- *(package)* no TrackFileNotInPackage for track files listed in a PKL
- *(validation)* require canonical URN form for ISXD DataEssenceCoding
- *(assetmap)* run ST 429-9 Annex A.2 path rules for every input
- *(assetmap)* name every offending path character once per path
- *(package)* report CPL track files outside the package as info
- *(validation)* compute App #2E segment duration per ST 2067-3
- *(validation)* ignore the UL version byte for ISXD DataEssenceCoding
- *(validation)* pin HT MAGB_P mapping to ITU-T T.814 Table 4
- *(validation)* describe J2KExtendedCapabilities as required for HTJ2K
- *(validation)* drop non-ST 2067-21 rules from the App #2E 2023 code set
- *(validation)* accept the 8K JPEG 2000 IMF profiles under ST 2067-21:2026
- *(validation)* stop the IAB and ISXD plug-ins from running App #2E 2023 rules
- *(cpl)* compare ContentKind text exactly, without trimming whitespace
- *(mxf)* reject partition pack lengths that run past end of file before reading
- *(mxf)* use the ST 377-1 Table 29 RIP key and parse the RIP locally
- *(mxf)* check the RIP key and bound its length before reading it
- *(mxf)* apply the partition pack size cap only to untrusted offsets
- *(validation)* reject COLOR.8 under the App2E 2020 validator
- *(mxf)* check the partition pack key before reading a value at a footer offset
- *(cpl)* do not flag an absent ContentKind
- *(validation)* cap Y'C'BC'R image frames at 3840x2160 in App2E
- *(validation)* flag missing ApplicationIdentification when ExtensionProperties is absent
- *(mxf)* locate the footer reliably before flagging an open header partition
- *(cpl)* match ContentKind against Table 3 / Table 4 case-sensitively
- *(validation)* align App2E frame rates with Table 3
- *(validation)* check App2E image frame size against Table 3 ranges
- *(validation)* enforce the Table 3 interlaced column for SeparateFields in App2E
- *(validation)* derive App2E image frame height from StoredHeight per Table 9
- *(validation)* accept ApplicationIdentification lists that include the App2E URI (ST 2067-21 §7.1)
- *(scm)* lower CplNotFound to a warning (ST 2067-9:2018 §8)
- *(iab)* drop ChannelCountNotZero from the ST 2067-201:2021 catalogue
- *(mxf)* only flag an open header partition when the MXF file is open (ST 377-1 §6.2.4)
- *(validation)* lower NoCpls default severity to info (ST 2067-2:2020 §7.1)
- *(validation)* skip timed text tracks in VirtualTrackEditRate (ST 2067-2:2020 §6.3.3)
- *(validation)* allow partial soundfield groups in audio track files (ST 2067-2 §5.3.6.6)
- *(cpl)* bound marker Offset by IntrinsicDuration (ST 2067-3 §6.13)
- *(cpl)* check ContentKind against ST 2067-3 §6.1.8 Table 3 / Table 4
- *(cpl)* check marker labels against ST 2067-3 Table 19 / Table 21
- *(validation)* require 24-bit audio quantization in App2E (ST 2067-2:2020 §5.3.2.3)
- *(validation)* accept 96 kHz audio in App2E (ST 2067-2:2020 §5.3.2.2)
- *(validation)* allow interlaced FrameLayout in App2E (ST 2067-21:2023 §6.2.1.3)
- *(clippy)* apply question-mark suggestion in video_info_from_descriptor
- *(mxf)* cfg-gate iab_labeling for wasm32
- *(validation)* audit P2 batch — 9 spec-conformance findings across audio MCA, IAB, ISXD
- *(validation)* audit P1 batch — invented IAB rule, timed-text UL byte, ADM-gated Mode A
- *(audio)* skip §5.3 WAVE-PCM rules on ST 2067-201 IAB essence (AUDIT-1)
- *(validation)* register IAB/ISXD plug-ins in validator dispatch
- *(audio)* Mode A channel labeling + footer→header partition fallback

### Other

- *(catalogue)* fix the SoundfieldChannelCount example
- regenerate catalogue artefacts from the rule tables
- Merge remote-tracking branch 'origin/main' into fix/real-world-findings
- *(validation)* pin DanglingEssenceDescriptor for zero-length resources
- Merge remote-tracking branch 'origin/main' into fix/napi-typescript-types
- *(core)* cargo fmt
- *(wasm)* drop the raw wasm export and the Tsify declarations
- *(assetmap)* pin that empty path segments are not flagged
- *(assetmap)* cite the ST 2067-2 HTTP Map Profile per edition
- *(validation)* record HTREV parameter B as not checked
- *(validation)* hand-check App #2E segment duration cases
- *(validation)* note open SMPTE issues on affected rules
- *(validation)* drop redundant rebinding in interlaced FrameLayout test
- *(validation)* pin J2K-HT-Not-Allowed to the App2E 2020 validator
- Merge pull request #94 from jpwesselink/chore/uppsala-0.12
- *(deps)* uppsala 0.12, inline dcml types into primary schemas
- *(deps)* bump thiserror 2, strum 0.28, quick-xml 0.42, sha1/sha2 0.11, criterion 0.8
- remove Astro leftovers and internal notes, fix codes page generator
- Merge branch 'main' into feat/fraunhofer-corpus-end-to-end
- Merge branch 'main' into feat/fraunhofer-corpus-end-to-end
- Merge branch 'main' into fix/register-iab-isxd-plugins
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
