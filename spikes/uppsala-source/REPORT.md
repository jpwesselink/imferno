# Spike: uppsala as imferno's lossless source layer

Branch `spike/uppsala-source`, standalone crate `spikes/uppsala-source/` (own
`[workspace]`; path-depends on `imferno-core`; uppsala patched to the fork's
`attr-ranges` branch, commit `695962e`). No production code changed.

Re-run from `spikes/uppsala-source/`:

```sh
cargo test                                   # unit tests + property tests (256 cases each)
cargo run --release --bin corpus             # criteria 1, 2
cargo run --release --bin edit               # criterion 3
cargo run --release --bin assetmap_compare   # criterion 4
cargo run --release --bin single_parse       # criterion 5 (needs ../smpte-specs/txt, or $SMPTE_SPECS_TXT)
cargo run --release --bin perf               # criterion 6, native (also writes target/synthetic-5mb-cpl.xml)
wasm-pack build --target nodejs --release --no-opt --out-dir target/wasm-pkg -- --features wasm
node wasm-bench.mjs                          # criterion 6, wasm32
cargo run --release --bin malformed          # criterion 7
CARGO_PROFILE_TEST_OPT_LEVEL=3 PROPTEST_CASES=2000000 cargo test --test proptest_ranges   # criterion 8
```

Machine: Apple Silicon (aarch64-apple-darwin, 10 cores), rustc 1.99.1, Node 22.18.0.

## Verdict: GO

uppsala works as the lossless source layer. Over the whole corpus its node
ranges index the source exactly (0 violations, every byte accounted for),
edits made by splicing at those ranges reparse cleanly and keep finding
positions correct, one parse feeds both the XSD validator and a typed model,
and parsing is about 2x faster than imferno's current quick-xml/serde parse.
It took one small, self-contained fork patch (attribute ranges). The real
costs are memory (about 15x the input natively, 9x on wasm32), fork
maintenance, and a few error-reporting gaps that are cheap to fix in the
fork. None of these justifies writing and maintaining an own lossless parser
on `rowan`. Keep `rowan` as the fallback if upstream rejects the patches and
the fork becomes a burden, or if error recovery (several well-formedness
errors per pass) becomes a hard requirement.

## Summary

| # | Criterion | Result | Numbers |
|---|---|---|---|
| 1 | Round-trip / range fidelity | PASS | 294 files (267 IMF + 27 other XML, 9.3 MB), all parse, 485,048 nodes, **0 violations**; leaf + tag + prolog bytes = file bytes exactly. uppsala accepts every file quick-xml and imferno's parsers accept. |
| 2 | Attribute value ranges | PASS (fork patch) | not in uppsala 0.12; added `attribute_spans/attribute_span/attribute_range/namespace_declaration_span` on fork branch `attr-ranges`; 16 new tests; uppsala suite 779 passed / 0 failed / 5 ignored + 24 doctests; 2,375 corpus attributes + 3,042 xmlns spans verified |
| 3 | Edit by range | PASS | 3 edit kinds x 144 CPLs = 432 edits: 432/432 reparse, invariants hold, byte diff == edit window; 44/44 pre-existing findings found at the mapped position; 1,987/1,987 new findings on a node |
| 4 | Typed AssetMap from the DOM | PASS | 40 ASSETMAPs: 39 both parsers, 1 only the typed model; 0 differing values in shared fields; every difference explained (defaults, dropped fields, imferno ignoring namespaces/unknown elements); 16 synthetic variants |
| 5 | Single parse | PASS | 144/144 CPLs: XSD findings from the shared Document identical to imferno's XSD pass; 18/18 XSD findings (CPL + AM) map to a node; AM findings land on typed-model fields / findings |
| 6 | Performance | PASS (time), CONCERN (memory) | 5 MB CPL: uppsala 16.1 ms vs `parse_cpl` 31.8 ms native, 33.3 ms vs 61.3 ms wasm32; peak heap 78 MiB vs 15 MiB native, 46 MiB vs 22 MiB wasm. XSD position lookup is O(input) per finding: 1,146 findings on 5 MB = 573 ms vs 19 ms clean |
| 7 | Malformed XML | PASS with caveats | 15 well-formedness faults: 12 reported on the fault's line (5 within 2 bytes); unclosed element reported at the parent's end tag (+333 lines); EOF errors have no position; encoding is the caller's job |
| 8 | Robustness | PASS | proptest, about 8 minutes in total: 2.66M generated documents, 2M mutated corpus files, 4M arbitrary markup strings; 0 panics, 0 invariant violations |

---

## 1. Round-trip / range fidelity: PASS

Corpus: every `.xml` under `test-data/` (267 files, 3.04 MB: 144 CPL, 40
ASSETMAP, 40 PKL, 34 VOLINDEX, 8 OPL, 1 other). `docs/docs/public/samples/` and
`fuzz/seeds/parse_xml/` do not exist on `main`. Extra corpus: the 27 other
tracked XML files (vendored XSDs, SMPTE registers, RegXML dictionaries;
6.30 MB, up to 2.3 MB per file).

For every node the checker (`src/invariants.rs`, written independently of
uppsala's scanner) verifies:

* the range lies inside the parent's range, on char boundaries;
* elements: `&src[range]` starts with `<` plus the written QName and ends with
  the matching end tag (whitespace allowed before `>`) or `/>`;
* text: decoding `&src[range]` (predefined entities, char refs) equals the DOM
  text (after XML line-end normalization, which uppsala applies to text);
  CDATA, comments and PIs are byte-exact;
* children tile the parent: the first child starts at the end of the start
  tag, siblings are contiguous, the last child ends at the start of the end
  tag. So ranges are nested, ordered and non-overlapping, and the only bytes of
  an element not covered by a child are its own two tags;
* outside the root element there is only a BOM, the XML declaration, the
  DOCTYPE and whitespace;
* `node_line`/`node_column` agree with an independent line index;
* attributes: see criterion 2.

| corpus | files | parsed | violations | nodes | leaf + tag + prolog bytes = file bytes |
|---|---|---|---|---|---|
| IMF (`test-data`) | 267 | 267 | **0** | 128,255 | 1,467,942 + 1,560,063 + 12,918 = 3,040,923 (exact) |
| extra (XSD, registers) | 27 | 27 | **0** | 356,793 | 3,753,720 + 2,548,275 + 906 = 6,302,901 (exact) |

Unit tests show the checker is sensitive: pointing a parsed DOM at text that
differs by one byte in a text node, a tag name, an attribute value or a tag
delimiter yields violations.

**Acceptance compared with quick-xml.** All 294 files are accepted by both
uppsala and quick-xml's `Reader` (which imferno's serde parsers sit on). No
file accepted by imferno's `parse_cpl`, `parse_assetmap`, `parse_pkl`,
`parse_opl`, `parse_volindex` or `parse_scm` is rejected by uppsala. uppsala is
stricter on XML that is not well-formed: quick-xml accepts 8 of the 15 injected
faults in criterion 7 that uppsala rejects (unquoted or duplicate attributes,
an undefined entity, a control character, content after the root, a file
truncated in half, among others). That changes behaviour only for files that
are not XML; there are none in the corpus.

**uppsala behaviour worth knowing (not failures):**

* Text content gets XML 1.0 §2.11 line-end normalization (CR LF to LF) in the
  DOM; 36 corpus files contain CR. Attribute values get no normalization at
  all (neither line ends nor the §3.3.3 whitespace rule), only entity
  expansion. Neither matters for a source layer, because the raw bytes are
  always available by range.
* The XML declaration and DOCTYPE are not nodes and have no range. They are the
  only non-whitespace bytes not covered by a node.

## 2. Attribute value ranges: PASS (after a fork patch)

uppsala 0.12 has no attribute positions. Added on the fork, branch
[`attr-ranges`](https://github.com/jpwesselink/uppsala/tree/attr-ranges)
(commit `695962e`, one commit on top of `imferno-patches-0.12`): a new file
`src/source_span.rs` (about 200 lines including unit tests), two lines in
`lib.rs`, and `tests/attribute_ranges.rs`.

```rust
pub struct AttributeSpan { pub name: Range<usize>, pub value: Range<usize>, pub quote: u8 }
impl AttributeSpan {
    pub fn full(&self) -> Range<usize>;                       // name through closing quote
    pub fn is_namespace_declaration(&self, input: &str) -> bool;
}
impl Document<'_> {
    pub fn attribute_spans(&self, id: NodeId) -> Option<Vec<AttributeSpan>>;          // all, incl. xmlns, source order
    pub fn attribute_span(&self, id: NodeId, name: &QName) -> Option<AttributeSpan>;  // matched on namespace URI + local name
    pub fn attribute_range(&self, id: NodeId, name: &QName) -> Option<Range<usize>>;  // raw value, without quotes
    pub fn namespace_declaration_span(&self, id: NodeId, prefix: &str) -> Option<AttributeSpan>;
}
```

Design: parsing pays nothing. Spans are recovered lazily by re-scanning the
element's start tag; the input is already known to be well-formed, so the scan
only looks for `=`, the quotes and `>`. The n-th DOM attribute is paired with
the n-th non-xmlns span and the written name is checked, so an element mutated
after parsing returns `None` rather than a wrong span. Value ranges are raw
(entity references unexpanded): the range an editor replaces.

Tests (`tests/attribute_ranges.rs`: 11 integration tests, plus 3 unit tests and
2 doctests): both quote styles and the other quote inside a value; predefined,
character and internal-DTD entity references; multi-line values and whitespace
around `=`; UTF-8 names and values; `>`, `/` and `/>` inside values; namespaced
attributes matched on URI rather than prefix; namespace declarations; nested
elements; decoding every raw value back to the DOM value; mutated and built
elements. Full uppsala suite: **779 passed, 0 failed, 5 ignored**, plus 24
doctests; clippy clean.

Corpus: 2,375 attributes and 3,042 namespace declarations. Each one has a span
inside its start tag whose decoded value equals the DOM value, and the start
tags contain nothing but the name, the spans, whitespace and `>` or `/>`.

## 3. Edit by range: PASS

`src/bin/edit.rs`, on **all 144 CPLs** (2013 and 2016 namespaces). Each edit is
applied separately, by splicing text at an uppsala range:

| edit | applied | reparse ok | invariants hold after | byte diff == edit window | old findings at mapped position | new findings on a node |
|---|---|---|---|---|---|---|
| (a) `<ContentTitle>` text set to `"Edited &amp;\ntitle"` (adds a line) | 144 | 144 | 144 | 144 | 15/15 | none new |
| (b) first attribute value set to `"not a language!"` | 144 | 144 | 144 | 144 | 15/15 | 101/101, all on the edited element |
| (c) `<IssueDate>` element deleted | 144 | 144 | 144 | 144 | 14/14 (1 was inside the deleted span) | 1,886/1,886 |

"Old findings at mapped position": each XSD finding of the original text that
lies outside the edit was mapped through the edit (offset plus the byte delta,
then line and column from a fresh line index) and was found, with the same
message, at exactly that line and column among the edited text's findings.

After (b) every new finding lands on the element whose attribute was edited
(`Annotation@language` 30, `Issuer@language` 10, `Audience@scope` 52,
`ContentKind@scope` 1, `Resource@xsi:type` 8). After (c), 143 "expected
IssueDate" and 143 "sequence must occur" findings land on
`<CompositionPlaylist>`; the rest are uppsala's XSD cascade ("Unexpected
element 'X' in sequence" for every following sibling), each on the sibling it
names. The cascade is noise from uppsala's content-model matcher, not a
position problem.

To insert text into an empty element the spike computes the end of the start
tag itself: uppsala has no start-tag, content or end-tag range API. That is
easy to add next to `attribute_spans` (same lazy re-scan).

## 4. Typed model from the DOM (ST 429-9:2014 AssetMap): PASS

`src/assetmap.rs`. The content models are transcribed by hand from the 2014
Annex A schema (element names, cardinality, attributes; no spec text is
copied). imferno vendors **no** ST 429-9 XSD in `crates/imferno-core/specs/`;
for criterion 5 the schema is read at runtime from the local spec text.

* `Value` is `Absent`, `Empty { at, raw }` or `Present { text, at, raw }`, for
  simple content and for attributes. `text` is decoded but never trimmed or
  case-folded; `raw` is the source range (the insertion point when empty);
  `at` holds range, line and column.
* Structure the schema does not know becomes a finding with its position:
  `AM/UnknownElement` (AM namespace, not in the content model),
  `AM/ForeignElement` (another namespace where the model has no `xs:any`),
  `AM/DuplicateElement`, `AM/UnexpectedText`, `AM/ElementInSimpleContent`,
  `AM/UnknownAttribute`, `AM/NotAnAssetMap`. `xsi:*` attributes are allowed.
* A foreign element where the model has `xs:any ##other` is kept as an
  `Extension` (namespace, name, position; the source text is the payload). The
  2014 AM schema has no `xs:any`, so a unit test with a modified model
  exercises this path.
* Cardinality and order are left to the XSD layer (criterion 5).

Comparison with `imferno_core::assetmap::parse_assetmap` on the 40 ASSETMAPs:

| result | count | explanation |
|---|---|---|
| both produce a model | 39 | |
| imferno errors, typed model built | 1 | `BadXML/ASSETMAP.xml` has a `<Chunk>` directly in `<Asset>` (no `<ChunkList>`). serde fails with "missing field ChunkList"; the typed model reports `AM/UnknownElement` at 23:9 and keeps the rest; the XSD reports the missing ChunkList on the `<Asset>` at 21:5. |
| differing values in shared fields | 0 | Id, AnnotationText, Creator, VolumeCount, IssueDate, Issuer, Asset Id and PackingList, Chunk Path are identical as text |
| `Chunk/VolumeIndex` absent | 100 chunks | imferno fills in the default `1`; the typed model keeps `Absent` (the default is a semantic fact, not a source fact) |
| kept by the typed model, dropped by imferno | 63 `@language`, 32 `Offset`, 32 `Length` | imferno's AssetMap type has no fields for them |

The corpus has no padded, empty or oddly cased values, so 16 synthetic variants
of a real ASSETMAP (`Application5/MultiAppIMP/ASSETMAP.xml`) cover the rest:

| variant | typed model | imferno today |
|---|---|---|
| padded Creator `"  IMF-Tool  "` | `Present("  IMF-Tool  ")` | `Some("  IMF-Tool  ")` (quick-xml 0.42 does not trim) |
| `<Creator/>` | `Empty` | `Some("")` |
| whitespace-only Creator | `Present("   ")` | `Some("   ")` |
| Creator absent (required by the schema) | `None`; the XSD reports it | `None`, no diagnostic from the parser |
| `language=""` | `language: Empty` | attribute dropped |
| upper-case UUID | raw `5671D298-…` kept | lower-cased |
| bare UUID (no `urn:uuid:`) | raw kept; the XSD pattern fails | accepted silently (`ImfUuid::parse` is lenient) |
| `&amp;` plus CDATA in Creator | `"A & B <c> "` | same |
| unknown `<Bogus>` in the AM namespace | `AM/UnknownElement` at 4:52 | silently ignored |
| foreign-namespace `<v:X>` | `AM/ForeignElement` at 4:52 | silently ignored |
| duplicate `<Issuer>` | first kept, `AM/DuplicateElement` at 7:22 | **parse error**, no model |
| every element `am:`-prefixed | identical model | identical values, but `namespace = Unknown("")` (the regex-based root namespace detection only sees `xmlns="…"`) |
| prefixed root, unprefixed children (children in no namespace) | 6 `AM/ForeignElement`, fields `Absent` | parsed as if namespaced (serde ignores namespaces) |
| `<PackingList>1</PackingList>` | `Present("1")` | `Some(true)` |
| `<VolumeCount> 1 </VolumeCount>` | `Present(" 1 ")` | `1` |
| wrong root namespace | no model, `AM/NotAnAssetMap` at 2:1 | full model, `namespace = Unknown(…)` |

Every difference is one of: raw versus normalized value (by design), default
filled in versus absent (by design), a field imferno does not model, or
imferno ignoring namespaces and unknown structure.

## 5. Single parse: PASS

`src/bin/single_parse.rs`. One `uppsala::parse` per file; the same `&Document`
goes to `XsdValidator::validate` and to the typed model.

* **CPL**: for 144/144 CPLs, validating the shared Document gives findings
  identical (message, line, column) to imferno's current XSD pass
  (`xsd::validate_parsed_cpl`, which parses `source_xml` again with uppsala
  after quick-xml/serde has parsed it). imferno never reaches its XSD pass for
  the 2 CPLs `parse_cpl` rejects; compared with `xsd::validate_against_schema`
  instead, those are identical too. 15/15 findings map to a node by line and
  column.
* **ASSETMAP**: 40/40 validated against the ST 429-9:2014 schema (read at
  runtime from `smpte-specs/txt/st429-9-2014.txt`, page headers and footers
  stripped). 3 findings, all in `BadXML/ASSETMAP.xml`; 3/3 map to a node. 2
  land on the `<Asset>` the typed model holds as a field; 1 lands on the
  `<Chunk>` the typed model already flagged as `AM/UnknownElement` (same node,
  same position).

uppsala's `ValidationError` carries only `line` and `column` (plus the fork's
`element_path`), not a `NodeId` or byte offset. Mapping back goes through a
line index (byte offset to node start). That worked for 100% of findings
because uppsala derives each position from a node, but a `node` or `offset`
field on `ValidationError` would remove the indirection (a small fork patch,
like `element_path`).

## 6. Performance: PASS on time, CONCERN on memory

Release build of the spike (`opt-level = 3`, fat LTO, one codegen unit). Median
of repeated runs; peak heap from a counting global allocator, measured above
the live baseline (excluding the input string).

#### test-data/Netflix_Sony_Plugfest_2015/CPL_BLACKL_202_HD_REC709_178_LAS_8fad47bb-ab01-4f0d-a08c-d1e6c6cb62b4_audio_homogeneity_fail.xml (45412 bytes, 2146 nodes)

| operation | median time | peak heap |
|---|---|---|
| uppsala::parse | 147.0 µs | 483 KiB |
| uppsala::parse + LineIndex | 171.6 µs | 491 KiB |
| LineIndex alone | 24.4 µs | 9 KiB |
| imferno parse_cpl (quick-xml serde) | 406.6 µs | 1309 KiB |
| quick-xml Reader, events only | 65.0 µs | 0 KiB |

#### test-data/Netflix_Sony_Plugfest_2015/ASSETMAP.xml (4726 bytes, 338 nodes)

| operation | median time | peak heap |
|---|---|---|
| uppsala::parse | 18.8 µs | 101 KiB |
| uppsala::parse + LineIndex | 21.2 µs | 102 KiB |
| LineIndex alone | 2.4 µs | 2 KiB |
| uppsala::parse + LineIndex + typed AssetMap | 38.3 µs | 113 KiB |
| imferno parse_assetmap (quick-xml serde) | 25.8 µs | 4 KiB |
| quick-xml Reader, events only | 7.7 µs | 0 KiB |

#### test-data/Netflix_Sony_Plugfest_2015/PKL_befcd2d4-f35c-45d7-99bb-7f64b51b103c_corrected.xml (5565 bytes, 296 nodes)

| operation | median time | peak heap |
|---|---|---|
| uppsala::parse | 19.4 µs | 120 KiB |
| uppsala::parse + LineIndex | 22.4 µs | 121 KiB |
| LineIndex alone | 2.9 µs | 1 KiB |
| imferno parse_pkl (quick-xml serde) | 28.9 µs | 5 KiB |
| quick-xml Reader, events only | 7.1 µs | 0 KiB |

#### test-data/OPL/OPL_8cf83c32-4949-4f00-b081-01e12b18932f.xml (7325 bytes, 257 nodes)

| operation | median time | peak heap |
|---|---|---|
| uppsala::parse | 21.9 µs | 83 KiB |
| uppsala::parse + LineIndex | 26.0 µs | 83 KiB |
| LineIndex alone | 4.0 µs | 1 KiB |
| imferno parse_opl (quick-xml serde) | 34.5 µs | 6 KiB |
| quick-xml Reader, events only | 7.1 µs | 0 KiB |

#### synthetic 5 MB CPL (5252362 bytes, 235372 nodes)

| operation | median time | peak heap |
|---|---|---|
| uppsala::parse | 16.03 ms | 79815 KiB |
| uppsala::parse + LineIndex | 18.92 ms | 80839 KiB |
| LineIndex alone | 2.80 ms | 1026 KiB |
| imferno parse_cpl (quick-xml serde) | 32.11 ms | 15192 KiB |
| quick-xml Reader, events only | 6.26 ms | 0 KiB |

#### crates/imferno-core/resources/registers/Elements.xml (2297175 bytes, 133098 nodes)

| operation | median time | peak heap |
|---|---|---|
| uppsala::parse | 8.04 ms | 34615 KiB |
| uppsala::parse + LineIndex | 9.18 ms | 35063 KiB |
| LineIndex alone | 1.22 ms | 449 KiB |
| quick-xml Reader, events only | 3.43 ms | 0 KiB |

#### XSD validation of the synthetic 5 MB CPL (http://www.smpte-ra.org/schemas/2067-3/2013)

| operation | median time | peak heap |
|---|---|---|
| validate(&doc), clean: 0 findings | 19.26 ms | 17 KiB |
| validate(&doc), every <Id> invalid: 1146 findings | 572.70 ms | 483 KiB |

**Reading the numbers**

* uppsala parses about **2x faster** than imferno's `parse_cpl` (45 KB CPL:
  148 µs vs 409 µs; 5 MB: 16.1 ms vs 31.8 ms). With imferno's shipping
  profile (`opt-level = "z"`) everything is about 2.5x slower but the ratios
  hold (5 MB: 43.9 ms vs 73.4 ms). For the small ASSETMAP / PKL / OPL files,
  uppsala alone is faster than the serde parse; the typed AssetMap on top
  (38 µs total) is in the same range as `parse_assetmap` (26 µs). All far
  below anything a user notices.
* Today imferno already parses each CPL twice (quick-xml/serde for the model,
  uppsala again inside the XSD pass). With uppsala as the source layer that
  becomes one parse.
* **Memory**: about 340 bytes per node natively (78 MiB for a 5 MB CPL with
  235k nodes, against 15 MiB for `parse_cpl`). Most of it is the arena: each
  node slot is large (`NodeKind` with an inline `Element`: QName of three
  `Cow`s, two `Vec`s, plus five links and two offsets), and for inputs of
  256 KB or more uppsala reserves `input.len() / 14` slots up front (375k
  slots for 235k nodes). Small files are fine (45 KB CPL: 483 KiB, against
  1.3 MiB for `parse_cpl`). On wasm32 (4-byte pointers) the 5 MB CPL takes
  46 MiB of linear memory against 22 MiB for `parse_cpl`; wasm memory never
  shrinks. Fine for real IMF CPLs (tens to hundreds of KB); worth a fork
  patch (smaller node layout, a lazier reservation) before multi-MB documents
  are a browser use case.
* **Position lookups in the XSD error path are O(input) per finding**:
  uppsala's `node_line`/`node_column` rescan the input from byte 0 on each
  call. On the 5 MB CPL with every `<Id>` made invalid (1,146 findings),
  validation takes 573 ms against 19 ms when clean. Harmless for normal
  documents; fix in the fork with a line index (the spike's `LineIndex`
  builds in 2.6 ms for 5 MB and answers in O(log n)).

wasm32: the spike crate (uppsala + imferno-core + quick-xml, `--features wasm`)
built with `wasm-pack --target nodejs --no-opt` (1.97 MB `.wasm`) and timed
under Node 22.18 (`wasm-bench.mjs`; the loop runs inside wasm, so string
marshalling is paid once per batch):

#### largest CPL (45 KB) (45412 chars)

| operation (wasm32, Node v22.18.0) | median per parse |
|---|---|
| bench_uppsala | 210.8 µs |
| bench_uppsala_indexed | 254.5 µs |
| bench_imferno_cpl | 746.4 µs |
| bench_quick_xml_events | 107.9 µs |

#### synthetic 5 MB CPL (5252362 chars)

| operation (wasm32, Node v22.18.0) | median per parse |
|---|---|
| bench_uppsala | 33.28 ms |
| bench_uppsala_indexed | 38.33 ms |
| bench_imferno_cpl | 61.34 ms |
| bench_quick_xml_events | 15.04 ms |

#### wasm linear memory high-water, synthetic 5 MB CPL (fresh instance per fn)

| operation | memory before | memory after |
|---|---|---|
| bench_uppsala | 1.4 MiB | 47.6 MiB |
| bench_uppsala_indexed | 1.4 MiB | 48.1 MiB |
| bench_imferno_cpl | 1.4 MiB | 23.5 MiB |
| bench_quick_xml_events | 1.4 MiB | 6.4 MiB |

## 7. Malformed XML: PASS with caveats

uppsala stops at the first error. Faults injected into
`MERIDIAN_Netflix_Photon_161006/CPL_0eb3d1b9-….xml` (`src/bin/malformed.rs`);
"fault at" is where the injected change is:

| case | fault at (line:col) | uppsala error | uppsala at | points at fault? | quick-xml |
|---|---|---|---|---|---|
| `unclosed element (</ContentTitle> removed)` | 9:23 | `Mismatched end tag: expected </ContentTitle>, found </CompositionPlaylist>` | 342:23 | no: +333 lines | byte 14418 (`ill-formed document: expected </ContentTitle>, but </CompositionPlaylist> was found`) |
| `start tag missing '>'` | 10:14 | Expected '=' | 10:18 | same line, +4 bytes | byte 14433 (`ill-formed document: expected </ContentKind>, but </CompositionPlaylist> was found`) |
| unquoted attribute value | 9:24 | Expected quote character | 9:24 | yes | accepted |
| attribute without value | 9:23 | Expected '=' | 9:23 | yes | accepted |
| duplicate attribute | 9:21 | Duplicate attribute: a | 9:26 | same line, +5 bytes | accepted |
| mismatched end tag | 9:23 | `Mismatched end tag: expected </ContentTitle>, found </ContentTitel>` | 9:38 | same line, +15 bytes | byte 466 (`ill-formed document: expected </ContentTitle>, but </ContentTitel> was found`) |
| `raw '<' in text` | 9:20 | Expected XML name | 9:21 | yes | byte 14436 (`ill-formed document: expected </>, but </CompositionPlaylist> was found`) |
| raw '&' in text | 9:20 | Expected XML name | 9:21 | yes | byte 463 (ill-formed document: entity or character reference not closed: `;` not found before end of input) |
| undefined entity | 9:20 | Unknown entity reference: &foo; | 9:25 | same line, +5 bytes | accepted |
| undeclared prefix | 10:1 | Undeclared namespace prefix: x | 10:15 | same line, +14 bytes | byte 501 (`ill-formed document: expected </x:ContentKind>, but </ContentKind> was found`) |
| control character U+0001 | 9:19 | Invalid XML character U+0001 | 9:15 | same line, -4 bytes | accepted |
| text after root | 344:1 | Content found outside of root element | 344:1 | yes | accepted |
| comment containing '--' | 10:8 | Comments must not contain '--' | 10:16 | same line, +8 bytes | accepted |
| truncated at 50% | 190:2 | Unexpected end of input | (none) | no position (EOF) | accepted |
| truncated inside a start tag | 10:6 | Unexpected end of input | (none) | no position (EOF) | byte 482 (`syntax error: tag not closed: > not found before end of input`) |
| declared UTF-16, content UTF-8 | 1:31 | **accepted** | - | - | accepted |
| Latin-1 byte (0xE9) in UTF-8 text | 9:19 | (not reached) `String::from_utf8`: invalid UTF-8 | 9:19 (byte 462) | yes (exact byte) | n/a |
| Latin-1 document declaring ISO-8859-1 | 9:19 | (not reached) `String::from_utf8`: invalid UTF-8 | 9:19 (byte 467) | yes (exact byte) | n/a |
| UTF-16LE with BOM | 1:1 | (not reached) `String::from_utf8`: invalid UTF-8 | 1:1 (byte 0) | yes (exact byte) | n/a |

* Of the 15 well-formedness faults, 12 are reported on the fault's line: 5
  within 2 bytes of it, 7 a few bytes further along (at the second value of a
  duplicate attribute, at the `>` of a mismatched end tag, at the end of a
  start tag with an undeclared prefix, at the start of the text run holding a
  control character, and so on). Good enough to put a marker on the right
  line.
* **Unclosed element**: reported where any streaming parser notices it, at the
  parent's end tag 333 lines later. The message names `</ContentTitle>` but not
  where it was opened. **Truncation**: `UnexpectedEof` has no position (it is
  the end of the input) and does not name the open elements. Both are fixable
  in the error type (carry the start offsets of the open-element stack).
* **Encoding**: uppsala takes `&str`, so decoding is the caller's job. Invalid
  UTF-8 (Latin-1 bytes, a UTF-16 file) is caught by `String::from_utf8` at the
  exact byte. A declared `encoding="UTF-16"` or `"ISO-8859-1"` is never checked
  against the bytes; imferno has to decide what to do with the declaration
  (decode according to it, or report a mismatch).
* uppsala's errors carry line and column but no byte offset; the spike maps
  them back with its line index.

## 8. Robustness: PASS

`tests/proptest_ranges.rs`, run with `CARGO_PROFILE_TEST_OPT_LEVEL=3` (optimized,
debug assertions and overflow checks still on). Every case checks that uppsala
does not panic, and whenever it accepts the input, that every criterion 1/2
invariant holds (including line and column against the line index).

| property | cases | wall time | accepted by uppsala | result |
|---|---|---|---|---|
| `generated_documents`: random well-formed documents (prefixes and namespace declarations, both quote styles, entity and char references in text and attributes, CR LF, BOM, XML declaration, comments, CDATA, PIs, self-closing tags, odd whitespace inside tags) | 1,500,000 (plus 1,164,514 in a first run that stopped at proptest's filter-reject cap, since fixed) | 351 s | all (required) | 0 failures |
| `mutated_corpus`: corpus files under 12 KB with 1 to 3 random edits (insert, delete or replace with `<`, `>`, `&`, quotes, `<!--`, `]]>`, `</x>`, CR, ...) | 2,000,000 | about 150 s (3 tests in parallel) | about 18% (366 of 2,000 sampled) | 0 failures |
| `arbitrary_markup`: strings of up to 40 markup tokens, each checked bare and wrapped in `<r>…</r>` | 2,000,000 x 2 | same run | about 21% of the wrapped ones (412 of 2,000 sampled) | 0 failures |

No panic, no invariant violation.

---

## What had to be added to uppsala

1. **Attribute spans** (done; branch `attr-ranges`): `attribute_spans`,
   `attribute_span`, `attribute_range`, `namespace_declaration_span`.

Recommended next. Each is small and in the same style; none was needed to
prove the point:

2. `start_tag_range`, `content_range` and `end_tag_range` per element (edit
   inside an empty element, rename a tag).
3. A `node: Option<NodeId>` (or byte offset) on `ValidationError`.
4. A byte offset on the `XmlError` variants, a position for `UnexpectedEof`,
   and the open element's position in "mismatched end tag" and EOF errors.
5. O(log n) line and column lookup (fixes the quadratic XSD error path).
6. A smaller arena node and a lazier up-front reservation (memory).

## Risks

* **Fork maintenance.** Two patches now (`element_path`, attribute spans), with
  more recommended. `source_span.rs` is self-contained (one new file and two
  lines in `lib.rs`), cheap to rebase, and plausible upstream material.
  Whether upstream accepts it is unknown.
* **Memory** on large documents: about 15x the input natively, 9x on wasm32.
  Fine for IMF CPLs as they are; relevant for multi-MB CPLs in the browser.
* **O(input) position lookups** in uppsala's XSD error path, on large files
  with many findings.
* **No error recovery**: one well-formedness error per pass, so the editor
  shows only the first. Accepted for v1 by design.
* **Stricter than quick-xml** on XML that is not well-formed. Correct, but a
  behaviour change for anyone feeding imferno broken XML today.
* Attribute values are not normalized by uppsala (XML 1.0 §3.3.3). Typed values
  read from the DOM therefore keep tabs and newlines in attribute values. That
  happens to match the "raw values" goal, but it is non-conformant and could
  change if upstream fixes it; reading values from the raw range plus an own
  decoder avoids depending on it.
* ST 429-9 uses one namespace for its 2007 and 2014 editions, so the edition
  cannot be read from the document. Not an uppsala issue, but the "schema per
  edition" design needs another signal for the Asset Map.

## Branches

* imferno: `spike/uppsala-source` (this crate and this report). No PR.
* uppsala fork: `attr-ranges` at `695962e`. No PR.
* **Incident to fix by hand:** the first push of `attr-ranges` also updated
  the remote `imferno-patches-0.12` branch (the local branch tracked it, and
  `push.default` is `upstream`). `origin/imferno-patches-0.12` now points at
  `695962e`, one commit ahead of `0ac4150`. imferno pins uppsala by rev
  `0ac4150`, so its builds are unaffected. Restoring the branch needs a force
  push, which the agent was not allowed to run:
  `git push --force-with-lease=refs/heads/imferno-patches-0.12:695962e103b5663fab2ef35e86ea77527ae0a475 origin 0ac4150b028c97fb6d97996c69390ddc3d9a249e:refs/heads/imferno-patches-0.12`
