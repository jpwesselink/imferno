# Handover: Imferno Non-SMPTE Rules System Design

## Context

imferno is a Rust-based IMF (Interoperable Master Format) package parser and validator. It currently validates against SMPTE standards (ST 2067-2, ST 2067-3, ST 2067-21, etc.). We are designing a system for imferno's own validation rules that go beyond the SMPTE specs.

## Project structure

Workspace root: `/Users/jpwesselink/projects/pathe-thuis/imf-rs`

Key crates:
- `crates/imferno-core` - all parsing + validation logic
- `crates/imferno-cli` - CLI binary (`imferno`)
- `crates/imferno-wasm` - WASM bindings
- `crates/imferno-napi` - Node.js native bindings

Key files for this work:
- `crates/imferno-core/src/validation/mod.rs` - `ConstraintsValidator` trait, `ValidatorRegistry` trait, `ConfigurableValidatorRegistry`
- `crates/imferno-core/src/package/mod.rs` - `Imferno` struct, `validate()`, `ValidationOptions`
- `crates/imferno-core/src/package/codes.rs` - existing `ImfernoCode` enum (the `IMFERNO:` namespace)
- `crates/imferno-core/src/diagnostics/rules.rs` - `RulesConfig`, ESLint-style severity overrides
- `crates/imferno-core/src/diagnostics/codes.rs` - `ValidationCode` trait

## Design decisions made

### 1. Three rule classes

| Class | What it is | Behavior | Examples |
|-------|-----------|----------|----------|
| **SMPTE** | Derived from a published SMPTE spec | Always on (default) | `St2067_2_2020::FileNotFound`, `St2067_21_2023::*` |
| **Imferno** | Best-practice checks the spec misses | Opt-in | `UnreferencedAsset`, `UnlistedEssence`, `PathTraversal` |
| **Operational** | Tool cannot do its job | Always fires, not suppressible, not a "rule" | `XmlReadError`, `ReadDirError`, `DirEntryError` |

### 2. Operational errors move to Result::Err

Currently `XmlReadError`, `ReadDirError`, `DirEntryError` are `ImfernoCode` variants emitted as `ValidationIssue`. They should become `Result::Err` instead, since they are not findings about the package but failures of the tool itself.

### 3. Imferno rule subclasses

| Subclass | What it catches | Current/planned rules |
|----------|----------------|----------------------|
| **Security** | Things that could be exploited | `PathTraversal` |
| **Completeness** | Package has loose ends the spec does not care about | `UnreferencedAsset`, `UnlistedEssence` |
| **Consistency** | Metadata claims do not match content reality | Subtitle language verification (planned) |

### 4. Imferno rules are opt-in

SMPTE compliance is the authority and the default. Imferno rules are the "we think you should also check this" layer, activated explicitly. The rationale: "0 errors = SMPTE compliant" should remain unambiguous.

### 5. Open validator registry with builder pattern

Currently `validate()` internally constructs a `ConfigurableValidatorRegistry` from enum-based `ValidationOptions` fields (`core_spec`, `app_specs`). Users cannot add their own validators.

The design decision is to use a builder pattern where the user constructs and owns the registry:

```rust
let registry = ValidatorRegistryBuilder::new()
    .with_smpte_auto_detect()           // or .with_core(App2E2023)
    .with(MySubtitleLanguageChecker)     // custom ConstraintsValidator
    .build();

let report = package.validate_with_registry(&registry, &options);
```

This replaces the current flow where `validate()` builds the registry internally from `ValidationOptions` fields.

### 6. Current architecture (what exists today)

```
ConstraintsValidator trait
    fn spec_id(&self) -> &str
    fn validate_cpl(&self, cpl: &CompositionPlaylist) -> Vec<ValidationIssue>

ValidatorRegistry trait
    fn resolve_namespace(&self, uri: &str) -> Option<Box<dyn ConstraintsValidator>>
    fn resolve_for_cpl(&self, cpl: &CompositionPlaylist) -> Vec<Box<dyn ConstraintsValidator>>

ConfigurableValidatorRegistry (closed, enum-driven)
    delegates to BuiltinValidatorRegistry

Package validation flow:
    validate() -> builds registry internally -> calls validate_package_structure_with_cpl_validator(closure)
```

The CPL validator injection seam (`Fn(&CompositionPlaylist) -> Vec<ValidationIssue>`) already exists. Package-level checks (unreferenced assets, unlisted essences, path traversal, MXF headers, segment durations) are hardcoded in `validate_package_structure_with_cpl_validator`.

## Open design questions

### CPL-level vs package-level validators

`ConstraintsValidator` only validates CPLs. But imferno rules like `UnreferencedAsset`, `UnlistedEssence`, `PathTraversal` operate on the full `Imferno` struct. Should the builder accept both?

```rust
let registry = ValidatorRegistryBuilder::new()
    .with_smpte_auto_detect()
    .with_cpl_validator(MySubtitleChecker)       // ConstraintsValidator
    .with_package_validator(MyCompletenessCheck)  // PackageValidator trait (new)
    .build();
```

This was the last question asked before the conversation paused. No decision was made.

### Essence parsers

For content-aware rules (e.g. subtitle language verification), imferno needs to parse essence content (TTML/IMSC). Currently timed text is opaque. Two-phase pipeline was discussed:

```
raw package -> [parsers] -> enriched package -> [validators] -> issues
```

An `EssenceParser` trait was sketched but not designed in detail. This depends on the package-level validator question above.

### Subtitle language verification (first consistency rule)

The concrete use case driving this: "subtitles claim language is X, verify that at least N% of the text content is actually language X." Requires:
1. TTML/IMSC parser to extract text
2. Language detection (e.g. `lingua-rs` or `whatlang`)
3. A configurable threshold parameter

Decision: build as a built-in validator first, but design the interface so it could become a user-authored rule later.

### User-authored rules (future)

External process approach was favored as the simplest starting point: pipe serialized `Imferno` JSON to stdin of a user command, get back JSON issues on stdout. Any language, zero SDK needed:

```
imferno validate ./pkg --plugin "python my_rule.py"
```

This is future work, not part of the current design scope.

## What to do next

1. Decide: should the builder support both CPL-level and package-level validators, or just CPL-level for now?
2. Write a formal design spec based on these decisions
3. Create an implementation plan
4. Implement the open registry + builder
5. Move operational errors (`XmlReadError`, `ReadDirError`, `DirEntryError`) to `Result::Err`
6. Move imferno best-practice rules (`UnreferencedAsset`, `UnlistedEssence`, `PathTraversal`) behind opt-in
7. Add subtitle language verification as the first consistency rule

## Style notes

- Never use em-dash characters. Use period + new sentence, colon, parentheses, or " - " instead.
- Never add `Co-Authored-By: Claude` or any AI attribution to git commits.
