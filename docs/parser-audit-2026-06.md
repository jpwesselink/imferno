# Spec parser correctness audit — June 2026

A systematic correctness review of every SMPTE-spec parser and the
severity-override engine in `crates/imferno-core/`. The audit
identifies findings; fixes ship as separate commits (one ticket per
finding, listed at the bottom).

## Findings legend

| Tag | Meaning |
|-----|---------|
| `OK` | Reviewed, no concern. |
| `smell` | Pattern that could mask a bug but doesn't today. Annotate or refactor. |
| `gap` | Spec rule not enforced; feature genuinely missing. Open ticket. |
| `bug` | Wrong observable behaviour. Open ticket + regression test. |

## Spec citation confidence

When an audit row cites a SMPTE prose section, the source is tagged:

| Tag | Source | Confidence |
|-----|--------|------------|
| `firsthand-pdf` | Direct from a SMPTE PDF (paywalled but accessible) | high |
| `firsthand-xsd` | Direct from the vendored XSD text under `specs/` | high |
| `secondhand-comparison-doc` | From `specs/comparisons/*.md` (AI-extracted) | medium |
| `cross-reference-tool` | From reference-implementation source code | low-medium |
| `smpte-ra-namespace` | From smpte-ra.org/ns index | high |

## Audit axes

Eight subsystems. Each section reports findings against the six
audit questions (silent failures / edition coverage / required-vs-
optional discipline / test coverage / edge cases / dep boundary).
Axis 8 uses a different set of questions specific to the override
engine.

---

## 1. CPL parser — `cpl/mod.rs`

**Entry point**: `parse_cpl(xml: &str) -> Result<CompositionPlaylist, CplParseError>` at
`cpl/mod.rs:2884`. Delegates to `parse_cpl_with_options` which detects
namespace, runs signature checks, strips XML namespaces, then
`quick_xml::de::from_str`. Optional strict modes for unknown XML
tokens and basic schema constraints. Source XML retained on
`source_xml` field for downstream runtime-XSD pre-pass.

**Test coverage**: 75 tests across `cpl/{mod,types,validate}.rs`. No
`#[ignore]`, no `TODO`/`FIXME`. Coverage is broad but skewed toward
positive parse cases; negative-parse / malformed-input tests are
fewer.

| ID | Severity | Citation | Finding |
|----|----------|----------|---------|
| A1 | `smell` | firsthand-xsd | `parse_cpl_with_options` line 2897 silently falls back to `CplNamespace::default()` = `Smpte2067_3_2013` when `detect_root_namespace` returns None. A CPL with malformed/missing xmlns gets validated against the 2013 ruleset — wrong if the document was authored for a newer edition. **Fix:** `CplNamespace::from_uri_opt` returning `None` should map to `CplNamespace::Unknown(String::new())` so downstream validators see "namespace unknown" instead of "namespace 2013". |
| A2 | `gap` | smpte-ra-namespace | `CplNamespace::Smpte2067_3_2020` maps `http://www.smpte-ra.org/ns/2067-3/2020`, but the actually-published ST 2067-3:2020 XSD still targets `http://www.smpte-ra.org/schemas/2067-3/2016` (SMPTE retained the 2016 namespace for the 2020 edition). Real 2020 CPLs therefore land as `Smpte2067_3_2016`; the `Smpte2067_3_2020` variant is reachable only via a synthetic URI that doesn't appear in the wild. **Action:** verify against smpte-ra.org/ns whether `/ns/2067-3/2020` is a real registered namespace at all; if not, drop the variant or mark it deprecated with a comment. |
| A3 | `OK` | n/a | Line 3304 `extract_cpl_track_codecs_from_xml().unwrap_or_default()` is intentional — there's a sibling `try_extract_cpl_track_codecs_from_xml` that returns `Result` for callers who care. Dual-API pattern, deliberate. Doc-comment at 3299-3302 calls this out explicitly. |
| A4 | `OK` | firsthand-xsd | `LanguageString { text: text.unwrap_or_default() }` at line 1141 — empty `text` matches XSD `dcml:UserTextType` which is `xs:string` with no `minLength` facet. Semantic non-empty check would be downstream; XSD layer allows. |
| A5 | `OK` | n/a | BOM stripping at line 520 (`strip_prefix("\u{FEFF}").unwrap_or(xml)`) — standard pattern. |
| A6 | `smell` | n/a | `ContentKind::scope` / `MarkerLabel::scope` use `as_deref().unwrap_or(DEFAULT_SCOPE)` for missing optional attributes. Spec says `scope` is optional with documented default; fine, but the fallback should be a typed const not a string literal at the use site. Already factored into `CONTENT_KIND_DEFAULT_SCOPE` / `MARKER_LABEL_DEFAULT_SCOPE` constants — actually fine. |
| A7 | `gap` | n/a | No test for: (a) parse_cpl with empty string input, (b) parse_cpl with XML that has multiple competing default xmlns declarations, (c) parse_cpl that's well-formed XML but not a CompositionPlaylist root element. Add 3 negative-parse regression tests. |
| A8 | `gap` | secondhand-comparison-doc | Required-vs-optional discipline not fully audited — `CompositionPlaylist` struct fields not yet cross-referenced against ST 2067-3 SHALL/SHOULD list. The `specs/comparisons/imf-cpl.md` has the prose list; need a row-by-row check. **Action:** open follow-up ticket, deferred from this audit. |

**Dep boundary**: CPL parser delegates XML deserialisation to `quick_xml::de`. `quick_xml` is a well-tested crate; we don't audit its internals. Our boundary: we hand it a UTF-8 `&str` after namespace-prefix stripping (`strip_xml_namespaces`); any quick_xml errors come back as `CplParseError::Xml(quick_xml::DeError)`.

## 2. AssetMap / PKL / OPL / VolumeIndex parsers — `assetmap/mod.rs`, `assetmap/volindex.rs`

**Entry points**: `parse_assetmap` (`assetmap/mod.rs:1050`),
`parse_pkl` (`:1062`), `parse_opl` (`:1075`), `parse_volindex`
(`assetmap/volindex.rs:57`). All four detect namespace, deserialise
via `quick_xml::de`, then map raw struct to domain struct.

**Test coverage**: ~39 tests in `assetmap/mod.rs`, ~2 in
`volindex.rs`. No `#[ignore]`.

| ID | Severity | Citation | Finding |
|----|----------|----------|---------|
| B1 | `smell` | n/a | Same shape as A1 — `parse_assetmap`/`parse_pkl` lines 1053/1065 silently fall back to `Default::default()` (Dci429_9 / Dci429_8 respectively) when namespace detection fails. Older-era default leaks into newer-era documents with malformed xmlns. Should be `Unknown(String::new())`. |
| B2 | `gap` | smpte-ra-namespace | Same shape as A2 — `Smpte2067_9_2020` and `Smpte2067_2_2020` map URIs in the `/ns/` form (`http://www.smpte-ra.org/ns/2067-9/2020`, `http://www.smpte-ra.org/ns/2067-2/2020`). Need to verify these match what SMPTE-RA actually publishes; the smpte-ra.org/ns scrape showed `/ns/2067-50` but the 2067-2/9 namespaces aren't listed under `/ns/` there. Possibly fictional URIs same as the CPL case. |
| B3 | `OK` | firsthand-xsd | `HashAlgorithm::from_uri` returns `Option`, caller at `:1010-1021` correctly emits an `AssetMapParseError::Field` with "unsupported hash algorithm URI" message on `None`. The `None`/SHA-1 default at line 1020 is *only* applied when the entire `<HashAlgorithm>` element is absent — matches ST 2067-2 §9. **The Phase-1 inventory mis-flagged this as a silent default; correcting here.** |
| B4 | `OK` | firsthand-xsd | `ImfUuid::parse` enforces URN form including hyphens; `SmpteUl::parse` strips the URN prefix and validates hex+length. Both reject malformed input loudly via `ImfTypeError`. |
| B5 | `gap` | secondhand-comparison-doc | `parse_opl` (line 1075) drops the entire `MacroList` because xsi:type polymorphism with vendor extension namespaces doesn't deserialise via serde. The doc-comment at 1071-1074 calls this out. **Action:** open ticket for OPL macro parsing — needs a custom deserializer or manual XML walk. Today this is a silent feature gap: OPLs with macros parse "successfully" but the macros are gone. |
| B6 | `gap` | n/a | `VolumeIndex` carries only an `Index: u32` per ST 2067-9 §5. No edge-case tests for `Index = 0` (spec says positive integer) or `Index` overflowing u32. Add 2 regression tests. |
| B7 | `gap` | n/a | None of `parse_assetmap` / `parse_pkl` / `parse_opl` / `parse_volindex` have negative-input tests for: (a) empty string, (b) well-formed XML with wrong root element, (c) two competing `xmlns` default namespaces. Add 12 regression tests (3 per parser). |
| B8 | `OK` | n/a | Annotation/Issuer/Creator are `Option<String>` matching XSD `minOccurs="0"` for those elements. Aligns with spec. |

**Dep boundary**: same as axis 1 — `quick_xml::de` for XML
deserialisation. `chrono` is **not** used for IssueDate validation
(stored as `String`, no datetime parsing) — that's a separate
`smell` (B9) but probably intentional since the XSD layer's
`xs:dateTime` check handles it.

| B9 | `smell` | firsthand-xsd | `IssueDate` is stored as raw `String` on every doc type. No parsing means no compile-time guarantee the value is a valid xs:dateTime. XSD pre-pass catches this at the schema layer, so the parser-side smell is benign — but a future refactor where the XSD pre-pass becomes optional would silently lose this check. **Action:** none today, but document the dependency in the struct's doc-comment. |

## 3. SCM parser — `scm/mod.rs`

**Entry point**: `parse_scm(xml: &str) -> Result<SidecarCompositionMap, ScmParseError>`
at `scm/mod.rs:119`. Deserialises via `quick_xml::de`, then strictly
validates every UUID via `ImfUuid::parse_urn` (XSD-strict — `urn:uuid:`
prefix required).

**Test coverage**: 6 tests in `scm/mod.rs`. Per Phase-1 inventory:
none exercise the parser specifically — they test the surrounding
struct shape. **Confirmed.**

| ID | Severity | Citation | Finding |
|----|----------|----------|---------|
| C1 | `OK` | firsthand-xsd | UUID strictness is correct: `ImfUuid::parse_urn` rejects bare UUIDs (no `urn:uuid:` prefix), matching dcml:UUIDType. SCM does this consistently for top-level Id and every nested SidecarAsset / CPL reference. |
| C2 | `OK` | firsthand-xsd | Lines 129/136 `unwrap_or_default` are on `Option<List>` fields where the list element is itself optional in the XSD (SidecarAssetList, AssociatedCPLList). Empty list = absent list, fine. |
| C3 | `gap` | n/a | No parser-direct tests: empty SCM, malformed SCM, SCM with bare-UUID Ids (should error per C1 logic but no regression test pins it), SCM with multiple SidecarAssets referring to the same CPL. Add 4 regression tests. |
| C4 | `gap` | secondhand-comparison-doc | Signer/Signature presence flags only — no signature verification. Matches stated scope but should be documented at the type level so callers know the SCM struct doesn't represent a *verified* SCM. |
| C5 | `gap` | n/a | No SCM fixture under `test-data/`. Every SCM test uses hand-rolled XML strings. Vendor a known-good IMF package with an SCM document if possible (e.g. from a Netflix sample or AMWA plugfest). |
| C6 | `smell` | n/a | `IssueDate` stored as `String`, no datetime validation. Same as B9 — relies on XSD pre-pass for `xs:dateTime` check. |
| C7 | `OK` | smpte-ra-namespace | Single namespace handled (`http://www.smpte-ra.org/ns/2067-9/2018`); no DCI legacy variant since SCM is IMF-only. |

## 4. MXF partition-pack parser — `mxf/mod.rs`

**Entry point**: `parse_mxf_header_info(path)` / `parse_mxf_header_info_from_reader(reader)`
at `mxf/mod.rs:127,136`. Hand-rolled KLV parser — reads the 16-byte
partition pack key, BER-decodes the length, parses the fixed
partition-pack fields per ST 377-1 §7.1 Table 13. Returns
`MxfHeaderInfo { version, operational_pattern: String, essence_containers: Vec<String> }`.

**Coexistence**: also called from `validate_mxf_headers` alongside
the new `mxf::essence::validate_mxf_essence` (smpte-mxf-backed,
shipped this session). Both fire on every MXF; the new validator
emits richer diagnostics. The two parsers should be reconciled in a
follow-up — keeping both is duplication but neither is dead today.

**Test coverage**: 5 tests in `mxf/mod.rs` —
`valid_header_partition_pack_parsed`, `non_mxf_data_rejected`,
`body_partition_pack_rejected`, `essence_containers_parsed`, plus 1
`#[ignore = "requires test-data MXF files (large)"]`. All synthetic.

| ID | Severity | Citation | Finding |
|----|----------|----------|---------|
| D1 | `smell` | n/a | Body length capped at 4096 bytes (line 172) to avoid allocation attacks on hostile BER lengths. Partition packs with very many essence containers (>~248) would silently truncate. Real-world header partition packs are <1 KB so this is defensive; document at the type level so callers know the essence container list may be truncated. |
| D2 | `smell` | n/a | Line 211-213 silent `break` if the body is truncated mid essence-container UL. So a malformed partition pack returns a *partial* essence container list with no error. Either return an error here (preferred — a truncated body is malformed) or emit a `Warning`-class indicator. |
| D3 | `OK` | firsthand-spec | Strict KLV key prefix check (line 155-157) — rejects non-header partition packs and non-MXF data with `MxfParseError::NotMxf`. Same for BER length undershoot (line 164-169 returns `PartitionPackTooShort`). |
| D4 | `gap` | n/a | Coexistence with `mxf::essence::validate_mxf_essence` (smpte-mxf-backed) means two parsers run on every MXF, producing overlapping but differently-coded diagnostics for the same concerns (e.g. OP1a is checked in both). Plan a consolidation: pick one as the source of truth, deprecate the other. Today's behaviour isn't *wrong*, just duplicative — the aggregation logic groups by code so user impact is limited. |
| D5 | `gap` | n/a | No test for: (a) BER length encoded in long form spanning >1 byte (current code path uses `read_ber_length(reader, 16)`), (b) truncated essence container batch mid-element, (c) partition pack with `count > 0, elem_size != 16` (silently skipped by line 205), (d) operational pattern UL with non-OP1a byte 13. Add 4 regression tests. |
| D6 | `OK` | n/a | The acknowledged-placeholder comment about descriptors (`MxfHeaderInfo` doesn't carry typed essence descriptors) is the documented intentional design — CPL EssenceDescriptors are the authoritative source for format info per the module-doc. The new `mxf::metadata` pipeline covers the *actual* descriptor parsing for the few ST 2067-2 §5.x rules that need it. |

## 5. MXF → RegXML pipeline — `mxf/metadata.rs`

**Entry point**: `parse_mxf_to_regxml(path, options) -> Result<String, MxfFragmentError>`
at `mxf/metadata.rs:61`. Thin wrapper around `regxml::MxfFragmentBuilder::from_reader`
that supplies the embedded SMPTE metadictionary (Elements + Groups +
Types registers, vendored at `crates/imferno-core/resources/registers/`).

**Test coverage**: 2 tests — `embedded_dictionaries_load_successfully`
and `parse_mxf_to_regxml_surfaces_io_error_for_missing_file`. The
real round-trip is exercised indirectly via `mxf::audio_mca`'s
fixture tests (audio1.mxf, audio2.mxf, video1.mxf).

| ID | Severity | Citation | Finding |
|----|----------|----------|---------|
| E1 | `smell` | n/a | `dictionaries()` (line 43-49) silently returns `None` if `import_registers` fails. The doc-comment calls this out ("engine misconfigured"), but the silent path means a corrupted vendored register XML would manifest only as a runtime "missing dictionaries" error from `parse_mxf_to_regxml`, not at startup. Consider a `dictionaries_or_panic()` debug-build assertion or an explicit startup probe. |
| E2 | `OK` | n/a | `MxfFragmentError::Xml("...UTF-8: …")` wrap for non-UTF-8 RegXML output (line 74-76) is correct — regxml emits UTF-8 per spec, any non-UTF-8 indicates a corrupted MXF or a regxml-side bug. |
| E3 | `gap` | n/a | `MxfFragmentOptions` is taken by value and the only documented option in our use is `PartitionTarget::Header`. Tests don't cover `PartitionTarget::Footer`, `RootMode::EssenceDescriptor`, or `event_handler` / `auid_namer` injection. Add 2-3 regression tests across option combinations. |
| E4 | `OK` | n/a | The embedded register XMLs at `resources/registers/` are vendored from regxmllib-rs (commit pinned). Tracking new SMPTE register revisions is a separate concern documented earlier this session (suggested `git log` on sandflow/regxmllib's resources every few months). |
| E5 | `gap` | n/a | No test for: (a) MXF file that opens but is not actually an MXF (e.g. a `.txt` renamed), (b) MXF with valid partition pack but no header metadata at all (`HeaderByteCount = 0` — regxml should return empty Preface, our code returns Ok with empty-ish RegXML), (c) MXF whose Preface UL is from a namespace not in the loaded metadictionary. Add 3 regression tests. |

## 6. MXF audio MCA + timed-text rules — `mxf/{audio_mca,timed_text,essence}.rs`

**Modules**:
- `mxf::essence` (4 tests) — partition-pack-layer ST 377-1 + ST 2067-2 §5.2 rules
- `mxf::audio_mca` (19 tests) — ST 2067-2 §5.3 audio descriptor + MCA sub-descriptor rules
- `mxf::timed_text` (6 tests) — ST 2067-2 §5.4 timed-text rules

Recent code review (commit `4405b49`) caught a real bug in
`extract_field` (boundary mismatch could prefix-match `:Channel`
against `:ChannelCount`) and three clippy lints. Re-audit beyond
those fixes:

| ID | Severity | Citation | Finding |
|----|----------|----------|---------|
| F1 | `smell` | n/a | `extract_field`/`extract_all_fields`/`count_elements` assume regxml's output is well-formed XML *without* CDATA sections, XML comments inside elements, or `>` characters inside attribute values. regxml today emits none of those for ST 2067-2 §5.3/§5.4 essence fields, but the assumption is brittle. **Action:** add a doc-comment to each helper documenting the assumption; if regxml ever emits CDATA the walkers break silently. |
| F2 | `OK` | firsthand-xsd | Channel count / MCA channel ID parsing silently skips non-numeric values (lines 100, 195 — `.parse::<u32>().ok()`). The XSD pre-pass catches the type error at the structural layer; here, silently skipping a non-parseable value is correct — emitting our own error would double-report. |
| F3 | `OK` | firsthand-xsd | `parse_ul_bytes` returns `Option`; callers that get `None` (malformed UL string) skip the rule rather than emit a diagnostic. Same rationale as F2 — XSD layer catches it. |
| F4 | `gap` | secondhand-comparison-doc | Spec coverage for ST 2067-2 §5.3 audio: we implement WAVEPCMDescriptor presence, AudioSampleRate ∈ {48k, 96k}, QuantizationBits = 24, ChannelLabel count = ChannelCount, MCAChannelID coverage 1..N, exactly 1 SoundfieldGroupLabel, MCALinkID presence, SoundfieldGroupLinkID match, ChannelAssignment UL prefix, RFC-5646 spoken language, MCATitle/Version/ContentKind/ElementKind. **Not checked yet** (gaps): MaxCodecBitRate (§5.3.2.5), AAR-vs-PCM separation, MCAEpisode, MCALanguage on each channel label. Verify each against `specs/comparisons/imf-core-constraints-20130620.md` and add the missing ones. |
| F5 | `gap` | secondhand-comparison-doc | Spec coverage for ST 2067-2 §5.4 timed text: we implement UCSEncoding=UTF-8, NamespaceURI ∈ IMSC1, MIMEType whitelist, Mapping Kind = 0x13. **Not checked yet** (gaps): SamplingFrameRate, ZipResourceSubDescriptor handling for IMSC1 image profile resource bundles, multiple TimeTextResourceSubDescriptor enumeration. Audit against `specs/comparisons/st2067-2a-2016.md`. |
| F6 | `OK` | n/a | Each rule's severity / category lives in the typed code enum (`mxf::codes::St2067_2_2016`, `St377_4_2012`, `St377_1_2011`, `ImfernoMxf`), not at the emission site (refactored last session). Single source of truth maintained. |
| F7 | `gap` | n/a | None of the essence modules test against a *truly broken* RegXML — e.g. RegXML with a WAVEPCMDescriptor that's missing every MCA sub-descriptor entirely (not just one), or RegXML with deeply nested attribute namespacing. Synthetic snippets in tests are all "mostly clean, one rule broken". Add 2-3 hostile-input regression tests. |
| F8 | `smell` | n/a | The `extract_field`/`extract_all_fields`/`count_elements` walkers materialise `String` on every match (e.g. `out.push(body.to_string())`). For very large MXFs (e.g. an entire IAB programme) this is a noticeable allocation hot path. Consider `&str` slices into the original buffer if profiling shows it matters. **Action:** profile, then act — premature without numbers. |

## 7. XSD pre-pass — `xsd/mod.rs`

**Entry points**: `validate_parsed_cpl(cpl)` (line 86),
`validate_cpl_xml(raw_xml)` (line 271), plus composite-schema
variants. Delegates structural validation to the patched
`uppsala` 0.4 fork (commits `8a1a9c73` adding `element_path` to
ValidationError + xmlns-aware QName prefix resolution). 5
diagnostic kinds: `XSD/PatternInvalid`, `XSD/ElementMissing`,
`XSD/TypeInvalid`, `XSD/UnexpectedElement`, `XSD/SchemaConstraintFailed`,
each appended with `/<element_path>` for grep-ability.

**Test coverage**: integration suite `tests/xsd_runtime.rs` (9
tests), plus the 12 re-enabled `core_flags_*` tests in
`validation/mod.rs`. The `composite_schema_catches_dcml_typed_violations`
test that was previously `#[ignore]` is now passing (uppsala patch
2 + base_path fix in commit `39d9c94`).

| ID | Severity | Citation | Finding |
|----|----------|----------|---------|
| G1 | `OK` | firsthand-xsd | Composite-schema base_path fix (commit `39d9c94`) confirmed working — `validate_against_composite_schema_str` synthesises a virtual `__primary.xsd` filename inside `specs_dir` so uppsala's `.parent()` recovers the actual directory. Documented in code. |
| G2 | `OK` | n/a | uppsala fork pinned via `[patch.crates-io]` to `imferno-patches` branch. Patches 1 (`element_path`) and 2 (xmlns-aware QName) both applied and tested. Re-base needed if uppsala upstream lands new revisions. |
| G3 | `gap` | firsthand-xsd | `dcml-types-stub.xsd` carries only `UUIDType`, `RationalType`, `UserTextType`, `LanguageType`. ST 433 dcml namespace actually defines more types (`PhysicalDimensions`, `BoundingBox` etc.) — most are unused by IMF docs, but unverified. Action: enumerate dcml types referenced by every vendored XSD and confirm each is in the stub or knowingly absent. |
| G4 | `gap` | n/a | XSD pre-pass fires on CPL only (`validate_parsed_cpl`). PKL, AssetMap, OPL, VolumeIndex, SCM **don't** get an XSD pre-pass even though we vendor their schemas. Consider extending the pipeline so structural diagnostics fire on every IMF doc, not just CPL. |
| G5 | `smell` | n/a | `validate_against_composite_schema_str` line 108-156 catches uppsala errors and wraps them as `XSD/SchemaConstraintFailed` if unrecognised. The `translate()` function classifies by substring match on the message text — if uppsala changes a message string in a new release, we'd silently downgrade classifications to the generic fallback. Pin a regression test against each of the 5 expected message shapes. |
| G6 | `OK` | n/a | XSD pre-pass is *always on* (commit `01ed78d` dropped the feature gate). Confirmed: no `#[cfg(feature = "xsd-runtime")]` gates remain in any module. |
| G7 | `gap` | n/a | `validate_cpl_xml` (`raw_xml: &str`) and `validate_parsed_cpl(&cpl)` are two entry points covering different call paths. The behaviour should match (same diagnostics for the same XML); no test currently pins this equivalence. Add a regression test that feeds the same CPL XML through both and asserts diagnostic-set equality. |

## 8. Severity overrides + suppressed bucket — `diagnostics/{rules,mod}.rs`

**Components**: `RulesConfig` (ESLint-style severity overrides),
`apply_rules` (`rules.rs:239`), `match_specificity` (`:118`),
`glob_match`/`segment_matches`/`parse_source` for the matcher
internals, plus `ValidationReport::suppressed` bucket and
`IssueSource::from_code` (`mod.rs:141`).

**Test coverage**: 12 tests in `rules.rs`, 30 in `mod.rs`. Includes
a 100-iteration determinism regression test
(`apply_rules_remains_deterministic_across_runs`).

| ID | Severity | Citation | Finding |
|----|----------|----------|---------|
| H1 | `OK` | n/a | **Glob matcher edge cases**: `segment_matches` handles 0/1/N `*` per segment; tests cover empty pattern, leading/trailing `*`, mid-segment `*` (`ST2067-*:2020`), multi-segment glob (`XSD/*/UUID`), and the multi-`*`-per-segment case (`ST2067-*:2020:*/EditRate`). Unicode segment names not tested but the matcher is byte-aware via `find()` — should work transparently. |
| H2 | `OK` | n/a | **Precedence determinism**: regression test `apply_rules_remains_deterministic_across_runs` runs the same overlapping-pattern setup 100 times and asserts identical results. Pinned. |
| H3 | `gap` | n/a | **Source-prefix correctness**: `IssueSource::from_code` (mod.rs:141) classifies via `starts_with("XSD/")` / `starts_with("IMFERNO:" | "IMFERNO/")` / fallthrough to `ProseRule`. Verified for the four hard-coded prefixes (XSD/IMFERNO/ST2067-/dcml-) via `issue_source_from_code_*` tests. **Gap**: codes from `mxf::codes::St377_1_2011` start with `"ST377-1:"` and `ImfernoMxf` starts with `"IMFERNO:Mxf/"` — both classify correctly today, but a new code family added in a future session won't have a regression test pinning its `IssueSource` mapping. **Action:** add a test that enumerates `ValidationCode::ALL` for every typed enum and asserts the inferred `IssueSource` matches expectation. |
| H4 | `OK` | n/a | **Suppressed bucket semantics**: `apply_rules_suppressed_bucket_does_not_affect_compliance` test pins is_playable / is_compliant ignore suppressed. `apply_rules_source_prefix_off_moves_to_suppressed_bucket` confirms `Off` → demoted to Info + `context["suppressed_by"]` annotation + lands in `suppressed`. |
| H5 | `gap` | n/a | **Round-trip serialisation with populated `suppressed` + `additional_instances`**: no regression test currently. `ValidationReport` should survive serde JSON round-trip carrying both buckets populated. Add 1 test. |
| H6 | `OK` | n/a | **Glob-vs-fixed-code overlap**: `apply_rules_full_code_beats_glob` test pins this. `apply_rules_specific_glob_beats_general_glob` pins longer-prefix-glob beats shorter. |
| H7 | `smell` | n/a | `parse_source` (`:219`) accepts only the exact strings `"XsdLayer"`, `"ProseRule"`, `"EngineInternal"`. A typo like `"source:xsdlayer"` (lowercase) silently does nothing (returns `None`, so no rule matches). Either accept case-insensitive or emit a config-loading diagnostic. **Action:** decide — strict matching is defensible for ESLint-style config; if so, add a doc-comment example. |
| H8 | `gap` | n/a | **Config validation at load time**: today a `RulesConfig` with an unrecognised `source:Foo` or a glob like `XSD/**/UUID` (double-star is unsupported) is silently ignored at `apply_rules` time. Operators get no feedback that their config has a typo. Add a `RulesConfig::validate() -> Vec<String>` returning unmatchable-pattern warnings. |

## 9. SMPTE-RA namespace cross-reference

**Namespaces our enums currently match** (14 URIs across CPL, PKL,
AssetMap, VolumeIndex, SCM):

```
http://www.smpte-ra.org/ns/2067-2/2020          (PKL Smpte2067_2_2020)
http://www.smpte-ra.org/ns/2067-3/2020          (CPL Smpte2067_3_2020)
http://www.smpte-ra.org/ns/2067-9/2018          (SCM)
http://www.smpte-ra.org/ns/2067-9/2020          (AssetMap Smpte2067_9_2020)
http://www.smpte-ra.org/schemas/2067-100/2014   (OPL)
http://www.smpte-ra.org/schemas/2067-2/2013     (PKL Smpte2067_2_2013)
http://www.smpte-ra.org/schemas/2067-2/2016     (PKL Smpte2067_2_2016)
http://www.smpte-ra.org/schemas/2067-2/2016/PKL (PKL Smpte2067_2_2016Pkl)
http://www.smpte-ra.org/schemas/2067-3/2013     (CPL Smpte2067_3_2013)
http://www.smpte-ra.org/schemas/2067-3/2016     (CPL Smpte2067_3_2016)
http://www.smpte-ra.org/schemas/2067-9/2016     (AssetMap Smpte2067_9_2016)
http://www.smpte-ra.org/schemas/429-7/2006/CPL  (DCI CPL)
http://www.smpte-ra.org/schemas/429-8/2007/PKL  (DCI PKL)
http://www.smpte-ra.org/schemas/429-9/2007/AM   (DCI AssetMap)
```

**What smpte-ra.org/ns publishes** (from the scrape — IMF-relevant entries):

- `/schemas/2067-2`, `/schemas/2067-3`, `/schemas/2067-9` (under `/schemas/` path)
- `/schemas/2067-100`, `/2067-101`, `/2067-102`, `/2067-103`
- `/schemas/2067-21`
- `/ns/2067-50` (App #5 ACES) — the only IMF-family namespace on the `/ns/` path
- `/schemas/429-7`, `/schemas/429-8`, `/schemas/429-9`

| ID | Severity | Citation | Finding |
|----|----------|----------|---------|
| I1 | `bug` | firsthand-pdf | **Revised after fetching canonical SMPTE-published zips.** Earlier I called SMPTE-RA's `/ns/...` 404 "fictional namespace" — that was wrong. SMPTE-RA only hosts landing pages for some registered namespaces; the canonical truth lives in the doc-zip at pub.smpte.org. Updated verdicts:<br>· `http://www.smpte-ra.org/ns/2067-2/2020` (PKL 2020) → **REAL**, SMPTE-RA has a landing page.<br>· `http://www.smpte-ra.org/ns/2067-3/2020` (CPL 2020) → **UNREACHABLE**, confirmed from `st2067-3-20200407-pub.zip` whose `st2067-3a-2020.xsd` declares `targetNamespace="http://www.smpte-ra.org/schemas/2067-3/2016"`. ST 2067-3:2020 reuses the 2016 namespace.<br>· `http://www.smpte-ra.org/ns/2067-9/2020` (AssetMap 2020) → **likely UNREACHABLE**, ST 2067-9 has only 2018 edition published per SMPTE standards listing.<br>· `http://www.smpte-ra.org/ns/2067-9/2018` (SCM) → **CORRECT**, confirmed from `st2067-9-20180522-pub.zip` whose `st2067-9a-2018.xsd` declares `targetNamespace="http://www.smpte-ra.org/ns/2067-9/2018"` exactly.<br>**Action:** (a) keep `Smpte2067_2_2020` as-is; (b) remove `CplNamespace::Smpte2067_3_2020` (genuinely unreachable); (c) remove `AssetMapNamespace::Smpte2067_9_2020` (no 2020 edition exists); (d) **SCM mapping is correct, no change needed**. The earlier "fix SCM" sub-action is withdrawn. |

**Lesson logged**: smpte-ra.org 404 ≠ "namespace doesn't exist". The
authoritative source for any namespace verification is the canonical
zip at `pub.smpte.org/doc/<spec>/<date>-pub/<spec>-<date>-pub.zip`,
which contains the published XSD body. Future audit work should
fetch that zip before declaring a URI fictional.
| I2 | `OK` | smpte-ra-namespace | The 10 `/schemas/` URIs we map all appear on smpte-ra.org/ns's index. Coverage of those is correct. |
| I3 | `gap` | smpte-ra-namespace | SMPTE-RA lists `/schemas/2067-21` but we have no `App2ENamespace::from_uri()` enum — App 2E namespace handling lives elsewhere (probably in `validation/mod.rs::ApplicationIdentification`). Confirm there's no parsing entry-point for an App2E namespace match; if so, fine. |
| I4 | `gap` | smpte-ra-namespace | SMPTE-RA lists `/schemas/2067-101`, `/2067-102`, `/2067-103` (OPL extensions) but our `parse_opl` only handles the base namespace. Sub-namespace dispatch for the macro-types in those extensions is the missing OPL feature (finding B5). |
| I5 | `gap` | smpte-ra-namespace | SMPTE-RA lists `/ns/2067-50` (ACES App #5). We have no parser for it. Out of scope today (no IMF App #5 documents in our test corpus), but document as a known gap. |

## 10. Vendored XSD freshness

**Method**: SMPTE-RA serves XSDs at the predictable URL
`https://smpte-ra.org/sites/default/files/<basename>.xsd`.
For each vendored file in `specs/`, fetch the canonical version and
byte-diff. Drift = "vendored XSD stale" finding.

**Result — 14 of 17 vendored XSDs are byte-identical to SMPTE-RA**:

```
match    imf-cpl.xsd
match    imf-core-constraints-20130620.xsd
match    st2067-3a-2016.xsd
match    st2067-3a-2020.xsd
match    st2067-2a-2016.xsd
match    st2067-2a-2020.xsd
match    st2067-21a-2016.xsd
match    st2067-100a-2014.xsd
match    st2067-102a-2014.xsd
match    st2067-103b-2014.xsd
match    SMPTE-429-8-PKL-2007.xsd
match    ST429-14-2014.xsd
match    st430-1-2017-kdm.xsd
match    st430-3-2012.xsd
```

**3 not found at the predictable URL**:

```
404      st2067-201-2019.xsd   (IAB)
404      st2067-202a-2023.xsd  (ISXD)
404      st2067-203-2023.xsd   (MGA-SADM)
```

These were sourced from Netflix Photon's resource bundle in
commit `c29d76e` rather than directly from SMPTE-RA.

| ID | Severity | Citation | Finding |
|----|----------|----------|---------|
| J1 | `OK` | smpte-ra-namespace | 14 of 17 vendored XSDs verified byte-identical to canonical SMPTE-RA versions. No drift. |
| J2 | `smell` | n/a | 3 plugin XSDs (IAB / ISXD / MGA-SADM) not at the predictable SMPTE-RA path — sourced from Photon. They could still match SMPTE-RA's published versions but at a different URL. **Action:** find the actual SMPTE-RA URLs for these (probably linked from `/schemas/2067-201`, `/schemas/2067-202`, `/schemas/2067-203` landing pages) and diff. |
| J3 | `gap` | n/a | No automated drift check in CI. Pin a `scripts/diff-vendored-xsds.sh` or a `cargo xtask` so a future SMPTE-RA revision doesn't silently land while imferno keeps using stale schemas. |

---

## Fix tickets

Compiled from all axes. Ranked by severity then by impact. Each
finding ID in parentheses links back to the section above.

### Bugs (real wrong behaviour — fix soon)

> **[FIX-1 ✅ done]** Dropped `CplNamespace::Smpte2067_3_2020` and the
> `St2067_3_2020` rule enum. Verified `st2067-3a-2020.xsd` is
> byte-identical to `st2067-3a-2016.xsd` apart from the header text,
> so the 2020 publication adds no new constraints — collapse to a
> single 2016 rule set covers both editions. Side-effects:
> `IMF_CPL_2020_XSD` include dropped; `package::codes::St2067_3_2020`
> re-export rewired to 2016; `validate_cpl_dispatches_*` tests
> updated to assert "ST 2067-2:2016" core dispatch. Snapshot
> `tests/snapshots/validation-codes.txt` regenerated (-14 codes).
> NAPI `listRules` enumerator no longer iterates `St2067_3_2020`.

> **[FIX-2 ✅ done]** Dropped `AssetMapNamespace::Smpte2067_9_2020`.
> No `st2067-9*-2020.xsd` vendored, no 2020 edition published.
> Regression test renamed to
> `assetmap_with_fake_2020_namespace_lands_in_unknown`.

> **Retraction — SCM namespace mismatch (no ticket, no fix needed).**
> An earlier draft of this section called the SCM normative namespace
> wrong because `smpte-ra.org/ns/2067-9/2018` returns 404. Subsequent
> fetch of `st2067-9-20180522-pub.zip` confirmed the canonical
> `st2067-9a-2018.xsd` declares
> `targetNamespace="http://www.smpte-ra.org/ns/2067-9/2018"` — exactly
> what imferno maps. **Lesson**: SMPTE-RA 404 ≠ namespace doesn't
> exist; always verify against the doc-zip at pub.smpte.org.

### Smells (could mask a bug — annotate or refactor)

> **[FIX-3 ✅ done]** `parse_cpl`, `parse_assetmap`, `parse_pkl` now
> fall back to `Unknown(String::new())` on missing root xmlns instead
> of `Default::default()`. Three regression tests added
> (`cpl_without_root_xmlns_lands_in_unknown_not_2013`,
> `assetmap_without_root_xmlns_lands_in_unknown_not_dci`,
> `pkl_without_root_xmlns_lands_in_unknown_not_dci`). Verified all
> three tests fail against the pre-fix code path.

> **[FIX-4 ✅ done]** Added `MxfParseError::PartitionPackTooLarge { got, cap }`.
> The 4 KiB body cap now errors loudly when the partition pack
> declares a length above the cap, rather than silently truncating
> and possibly losing essence-container UL entries from the batch.
> Regression test `oversized_partition_pack_returns_too_large` added
> with a synthetic 5000-byte body.

> **[FIX-5 ✅ done]** `parse_source` is now case-insensitive via
> `eq_ignore_ascii_case`. Operator-friendly keys like `source:xsdlayer`,
> `source:XSDLAYER`, and `source:XsDlAyEr` all resolve. Regression
> test `rule_matches_source_prefix_case_insensitively` added.
> Doc-comment updated to reflect the new contract.

> **[FIX-6 ✅ done]** Six unit tests pin `classify()` against the
> uppsala 0.4 message shapes for `ElementMissing`,
> `UnexpectedElement`, `PatternInvalid` (×2 phrasings), `TypeInvalid`,
> and the `SchemaConstraintFailed` fallback. An uppsala upgrade that
> re-words any of these now trips a test rather than silently
> downgrading the diagnostic.

### Gaps (feature genuinely missing — open ticket)

> **[FIX-7 ✅ done]** Added a struct-level doc-block to
> `CompositionPlaylist` mapping every field to its ST 2067-3 §6/§7
> spec status (required / optional / parser-lenient with validator
> follow-up). Captures the policy: parser tolerates missing
> required fields and stores them as `Option<T>`; `validate_cpl`
> reports them as Error-severity findings. Five parser-lenient
> fields enumerated explicitly: `content_kind`,
> `content_version_list`, `essence_descriptor_list`, `edit_rate`,
> `locale_list`.

> **[FIX-8 ✅ done]** OPL `<MacroList>` is now extracted via a small
> event-driven `quick_xml::reader::Reader` walker — serde can't handle
> the abstract `MacroType` + `xsi:type` polymorphism with vendor-
> specific extension namespaces, so the walker runs *alongside* the
> existing serde deserialisation of the top-level fields. Each
> `<Macro xsi:type="...">` becomes an `OplMacro { xsi_type, name,
> annotation, extra_fields }` where `extra_fields` is a
> `Vec<(local_name, text)>` bag for the type-specific payload. The
> parser knows nothing about `PresetMacroType` /
> `AudioRoutingMixingMacroType` / etc — callers can map `xsi_type` to
> their own enums or treat it as freeform metadata. Three regression
> tests added (`opl_with_empty_macro_list_yields_no_macros`,
> `opl_with_preset_macro_captures_all_fields`,
> `opl_with_multiple_macros_captures_all_in_order`); the two
> pre-existing fixture-backed OPL tests still pass.

> **[FIX-9 ✅ done as far as vendorable]** Vendored the SCM XSD
> (`specs/st2067-9a-2018.xsd`) + the modern PKL XSD
> (`specs/st2067-2b-2016.xsd`) and added four new entry points:
> `validate_opl_xml`, `validate_scm_xml`, `validate_pkl_xml`,
> `validate_assetmap_xml` (skipped — see below).
>
> **PKL coverage** dispatches by namespace:
> - `Dci429_8` → `SMPTE-429-8-PKL-2007.xsd`
> - `Smpte2067_2_2016Pkl` → `st2067-2b-2016.xsd`
> - `Smpte2067_2_2020` → `st2067-2b-2016.xsd` (canonical 2020 PKL XSD
>   reuses the 2016 namespace + body)
> - bare `Smpte2067_2_2013` / `Smpte2067_2_2016` → skip (no companion
>   XSD in the wild)
>
> **Gaps confirmed unresolvable from vendoring (2026-06-13 probe of
> pub.smpte.org + Photon's resource bundle):**
> - **IAB:2021** — no separate XSD published. Photon doesn't have it,
>   SMPTE's `st2067-201-20201109-pub.zip` ships only the PDF, the catalogue
>   is bit-for-bit identical to 2019. Workaround: keep `St2067_201_2021`
>   with the `previous_identical_edition = "ST2067-201:2019"` annotation;
>   XSD pre-pass for 2021-namespace documents skips and structural checks
>   still fire.
> - **IAB:2026** — newly published (discovered 2026-06-13); also PDF-only
>   in `st2067-201-20260325-pub.zip`. No catalogue entries yet — separate
>   follow-up to add `St2067_201_2026`.
> - **AssetMap (any edition)** — `st2067-9-20180522-pub.zip` and
>   `st429-9-20141124-pub.zip` both ship only the PDF. The AssetMap
>   schema is defined inline in spec prose with no standalone XSD.
> - **VolumeIndex** — same situation. ST 429-14:2014 covers AuxData, not
>   VolumeIndex; the VolumeIndex schema is also prose-only.
>
> Six regression tests added (OPL clean-pass, OPL missing-field, SCM
> clean-pass, PKL skip-on-bare-namespace, PKL modern-2016, PKL 2020).
> Drift CI (`xsd-drift.yml`) covers the new `st2067-2b-2016.xsd`
> entry via the updated `manifest.json`.

> **[FIX-10 ✅ done]** New integration test
> `tests/parser_negative_inputs.rs` covers 6 parsers × 3 probes
> (empty, malformed XML, wrong root) = 18 tests. All assert that
> the parser returns `Err(...)` rather than panicking. Confirms the
> CPL/AssetMap/PKL/OPL/SCM/VolumeIndex contract is uniform.

> **[FIX-11 ✅ done]** New integration test
> `tests/issue_source_inference.rs` enumerates every typed code
> enum's `ALL` const (17 enums covering CPL/AssetMap/MXF/SCM/App2E/IAB/
> ISXD/Imferno/XSD/Volindex/Core) and pins each code's inferred
> `IssueSource`. Any future code-prefix change that breaks the
> contract trips one of three tests.

> **[FIX-12 ✅ done]** Regression test
> `validation_report_serde_round_trip_with_suppressed_and_aggregate`
> covers both populated buckets — an aggregated error with two
> additional instances + a suppressed info issue with the
> `suppressed_by` context annotation. Both survive the round-trip.

> **[FIX-13 ✅ done]** Added `RulesConfig::validate(known_codes)`
> returning `Vec<RuleValidationWarning>`. Three reason kinds:
> `UnknownSource`, `MatchesNothing`, `UnsupportedPattern` (the latter
> with an actionable hint, e.g. for `**` it suggests `*/*` or
> source-prefix). Four regression tests added covering clean config,
> unknown-source, match-nothing, and double-star.

> **[FIX-14 ✅ catalogued]** §5.3 / §5.4 coverage-gap audit. Each
> sub-item below is a follow-up rule-implementation ticket; the
> audit's role is to enumerate them so the gap surface is visible.
> Implementation lives in `crates/imferno-core/src/mxf/audio_mca.rs`
> and `crates/imferno-core/src/mxf/timed_text.rs`.
>
> **§5.3 audio essence (covered today)**: WAVEPCMDescriptor present,
> AudioSampleRate ∈ {48000, 96000}, QuantizationBits = 24,
> ChannelCount > 0, MCAChannelID coverage, SoundfieldGroupLinkID
> match, MCALanguage / MCATitle / MCATitleVersion / MCAAudioContentKind /
> MCAAudioElementKind, ChannelAssignment UL whitelist, RFC-5646
> spoken language, Wave Clip-Wrapped via ContainerFormat UL.
>
> **§5.3 audio essence (gap tickets)**:
> - FIX-14a: AAR vs PCM separation — flag a SoundDescriptor that
>   mixes AES3 audio (compressed) with WAVEPCMDescriptor (uncompressed)
>   in the same essence.
> - FIX-14b: MCAEpisode label validation — accept only published
>   episode-label ULs from SMPTE-RA registers/335 (when present).
> - FIX-14c: SamplingFrameRate (Edit Unit Rate) cross-check against
>   the CPL's referenced `EditRate` — should match.
>
> **§5.4 timed-text (covered today)**: TimedTextDescriptor /
> IMFTimedTextDescriptor presence, UCSEncoding = UTF-8, IMSC1 namespace
> whitelist, TimeTextResourceSubDescriptor MIMEType ∈ {image/png,
> application/x-font-opentype}, Mapping Kind UL = 0x13.
>
> **§5.4 timed-text (gap tickets)**:
> - FIX-14d: ZipResourceSubDescriptor — present when timed text is
>   packaged as ZIP; flag absence on TTML2 content claiming ZIP form.
> - FIX-14e: ResourceID consistency — every
>   TimeTextResourceSubDescriptor's `EssenceResourceID` must match a
>   ResourceID in the embedded TTML.
>
> Each gap ticket gets its own commit + regression fixture when
> picked up. None are currently scheduled.

> **[FIX-15 ✅ done]** Enumerated every `dcml:<Type>` reference
> across the 14 vendored XSDs (`grep -oE 'dcml:[A-Za-z][A-Za-z0-9]+'
> specs/*.xsd`). Result: exactly three types referenced —
> `UUIDType`, `UserTextType`, `RationalType` — and all three are
> stubbed. The audit's earlier worry about `PhysicalDimensions` /
> `BoundingBox` is unfounded: those ST 433 types exist but no IMF
> XSD imports them. **No code change needed.**

> **[FIX-16 ✅ audited]** Plugin XSD vendoring status:
>
> | Spec    | Vendored XSD                | Edition (target namespace)    | Code-side rule sets         | Gap |
> |---------|-----------------------------|-------------------------------|-----------------------------|-----|
> | ST 2067-201 (IAB)      | `st2067-201-2019.xsd`  | `/ns/2067-201/2019` | `St2067_201_2019`, `St2067_201_2021` | **2021 XSD not vendored** — 2021-namespace IAB docs skip the XSD pre-pass even though the catalogue knows them |
> | ST 2067-202 (ISXD)     | `st2067-202a-2023.xsd` | `/ns/2067-202/2022` | `St2067_202_2022`           | none |
> | ST 2067-203 (MGA-SADM) | `st2067-203-2023.xsd`  | `/ns/2067-203/2022` | (no rule set today)         | catalogue gap — no MGA-SADM-specific rules implemented yet |
>
> **Follow-up tickets:**
> - FIX-16a: vendor `st2067-201-2021.xsd` (or equivalent) and add a
>   namespace-dispatch arm to `validate_iab_xml` once that wrapper exists.
> - FIX-16b: scope MGA-SADM rules — start from prose §5.5 (if present in
>   ST 2067-203) and add a `mga_codes.rs` catalogue.
>
> Neither blocks current-corpus validation since the test fixtures
> declare the 2019/2022 namespaces respectively.

> **[FIX-17 ✅ done]** Added
> `specs/comparisons/_tools/check_xsd_drift.py` — SHA-256 compares
> each vendored XSD against the canonical
> `https://smpte-ra.org/sites/default/files/<basename>` URL. Exits
> 0 on match, 1 on drift, 2 on manifest read failure. Wired up via
> `.github/workflows/xsd-drift.yml` (weekly Mondays 06:00 UTC +
> workflow_dispatch). Deliberately not in the per-PR pipeline so
> SMPTE-RA flakes don't block merges.

> **[FIX-18 deferred]** App #5 (ACES) parser, namespace
> `http://www.smpte-ra.org/ns/2067-50/...`. Genuinely new parser
> surface — would need ACES-specific schema vendoring, CPL
> application-identification dispatch arm, and a corpus fixture.
> No customer demand visible today and no IMF App #5 packages in
> the test corpus, so left as a known gap. Reopen when an
> ACES-bearing IMP enters scope.

> **[FIX-19 ✅ done]** Vendored the canonical
> `st2067-9b-2018.xml` from the SMPTE-published
> `st2067-9-20180522-pub.zip` (BSD-3-Clause per the zip's
> `readme.txt`) into
> `crates/imferno-core/tests/fixtures/scm/`. Integration test
> `tests/scm_fixture.rs` parses it and asserts the Id + asset shape.
>
> **The fixture immediately exposed a real bug**: imferno's SCM
> parser expected `<IssueDate>`, `<Issuer>`, `<Annotation>` directly
> under `<SidecarCompositionMap>`, but the canonical XSD wraps them
> in a `<Properties>` element. Imferno would have rejected every
> conformant SCM in the wild. **Parser refactored**: introduced a
> raw `Properties` deserialiser between root and the wrapped fields;
> dropped the spurious `Creator` field (not in the XSD); all six
> hand-rolled SCM unit tests updated to wrap fields in
> `<Properties>` per spec.

> **[FIX-20 ✅ done]** Documented the four assumptions
> `extract_all_fields` (and its callers) make about RegXML output:
> no CDATA, no XML comments inside elements, no raw `>` in attribute
> values, single-prefix namespace style. Added five edge-case tests
> covering: open tags with attributes, self-closing forms, whitespace
> trimming, sibling non-concatenation, and prefix collision
> (`:Channel` vs `:ChannelCount`).

### Summary

| Tag | Count |
|-----|-------|
| `bug` | 2 |
| `smell` | 4 |
| `gap` | 14 |
| `OK` | 21 |

**Total findings**: 41. Of those, **6 carry "open ticket" actions
that need code changes** (FIX-1 through FIX-6); the other 14 fix
tickets are larger feature work or test-coverage expansion.

### Progress (2026-06-04)

The 6 small-scope tickets are resolved (FIX-1 through FIX-6). Each
landed with a regression test where applicable; full results:

| Ticket | Status | Net effect |
|--------|--------|------------|
| FIX-1 | ✅ done | Dropped unreachable `Smpte2067_3_2020` CPL variant + `St2067_3_2020` rule enum |
| FIX-2 | ✅ done | Dropped unreachable `Smpte2067_9_2020` AssetMap variant |
| FIX-3 | ✅ done | CPL/AssetMap/PKL fall back to `Unknown("")` instead of default variant on missing xmlns |
| FIX-4 | ✅ done | `MxfParseError::PartitionPackTooLarge` replaces silent 4 KiB truncation |
| FIX-5 | ✅ done | `parse_source` matches `source:XsdLayer` case-insensitively |
| FIX-6 | ✅ done | XSD classifier pinned with 6 message-shape unit tests |
| FIX-7 | ✅ done | Struct-level doc-block on `CompositionPlaylist` mapping every field to ST 2067-3 spec status |
| FIX-8 | ✅ done | OPL `MacroList` extracted via `quick_xml` walker into flexible `OplMacro { xsi_type, name, annotation, extra_fields }` records |
| FIX-9 | ✅ partial | Wired OPL / SCM / DCI-PKL into the XSD pre-pass; modern-PKL / AssetMap / VolumeIndex need XSDs vendored first |
| FIX-10 | ✅ done | `tests/parser_negative_inputs.rs` — 18 negative tests across 6 parsers |
| FIX-11 | ✅ done | `tests/issue_source_inference.rs` pins all 17 code-enum `ALL` consts |
| FIX-12 | ✅ done | Serde round-trip test for populated `suppressed` + `additional_instances` |
| FIX-13 | ✅ done | `RulesConfig::validate()` + `RuleValidationWarning` |
| FIX-14 | ✅ catalogued | §5.3/§5.4 coverage gaps itemised as FIX-14a..e sub-tickets |
| FIX-15 | ✅ done | dcml-types stub audit — complete coverage confirmed (no action) |
| FIX-16 | ✅ audited | Plugin XSD vendoring status mapped — IAB 2021 + MGA-SADM rules are the two real gaps (FIX-16a, FIX-16b) |
| FIX-17 | ✅ done | `check_xsd_drift.py` + `.github/workflows/xsd-drift.yml` weekly drift check |
| FIX-18 | deferred | App #5 (ACES) parser — no corpus, no demand today |
| FIX-19 | ✅ done | Vendored SMPTE-published SCM example fixture; uncovered + fixed parser bug (missing `<Properties>` wrapper, spurious `Creator` field) |
| FIX-20 | ✅ done | MXF essence-walker assumptions documented + 5 edge-case tests |

Test count baseline: 461 → 489 passing lib tests + 3 new
integration test files (`issue_source_inference.rs`,
`parser_negative_inputs.rs`, plus FIX-9's in-lib XSD tests).
**+47 regression tests** total across the FIX-1..20 sweep. The 7
pre-existing missing-MXF-fixture failures remain out-of-scope per
the audit charter.
