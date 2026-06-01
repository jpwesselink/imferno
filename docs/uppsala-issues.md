# Issues with uppsala for the imferno runtime-XSD use case

This document captures the two uppsala v0.4.0 limitations imferno's
runtime-XSD architecture hit during the
`spike/xsd-runtime` work. They are empirical findings from
integration tests against real SMPTE CPL fixtures and synthetic
broken CPLs — not bugs uppsala has reported elsewhere (verified by
reading <https://github.com/kushaldas/uppsala/issues> at time of
writing; nothing about either of these in the 12 existing issues).

Both limitations are real but bounded: uppsala remains the best
pure-Rust XSD validator we evaluated. These notes exist so we (or
upstream) can address them with the empirical context already worked
out.

Crate version pinned: **uppsala 0.4.0** (Apr 2026 release).
Reproducer code: `crates/imferno-core/examples/xsd_validate_spike_uppsala.rs`
and `crates/imferno-core/tests/xsd_runtime.rs`.

---

## Issue 1 — `ValidationError` has no element-path field

### Summary

Each diagnostic uppsala returns from `XsdValidator::validate` carries
only `{ message: String, line: Option<usize>, column: Option<usize> }`.
There is no field telling the consumer which XML element the
violation pertains to. The element name is embedded in the message
text (e.g. `"Expected at least 1 occurrence(s) of element 'IssueDate',
found 0"`) but extracting it requires substring/regex parsing of the
human-readable message, which is brittle across uppsala versions.

### Where in the source

`src/error.rs:67`:

```rust
pub struct ValidationError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}
```

No XPath, no node reference, no element-name field.

### Reproducer

```rust
use uppsala::{parse, XsdValidator};

let schema = parse(r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="thing">
    <xs:complexType><xs:sequence>
      <xs:element name="name" type="xs:string"/>
    </xs:sequence></xs:complexType>
  </xs:element>
</xs:schema>"#).unwrap();
let validator = XsdValidator::from_schema(&schema).unwrap();

let doc = parse("<thing></thing>").unwrap();
let errors = validator.validate(&doc);

// errors[0] = ValidationError {
//     message: "Expected at least 1 occurrence(s) of element 'name', found 0",
//     line: Some(1),
//     column: Some(1),
// }
// — but no `element_path: "/thing/name"` or `element_name: "name"` field.
```

### Impact for imferno

The translator (`imferno_core::xsd::translate`) classifies each
uppsala diagnostic into one of five generic catalogue codes:
`XSD/ElementMissing`, `XSD/UnexpectedElement`, `XSD/PatternInvalid`,
`XSD/TypeInvalid`, `XSD/SchemaConstraintFailed`. With element-path
info, the translator could refine to specific catalogue codes
(e.g. `CoreConstraintsCode::IssueDate` instead of generic
`XSD/ElementMissing`) by looking up `(element_path, kind)` in a small
table. This would let us un-ignore the 12 `core_flags_*` tests
currently marked `#[ignore]` in `crates/imferno-core/src/validation/mod.rs`
that assert specific catalogue codes fire.

The current workaround (parse the element name out of the message
text via regex) is brittle: uppsala message text is not part of any
stability contract, so a future uppsala release that rephrases
messages would silently break the translator. We deliberately chose
not to take this workaround during the spike for that reason.

### Suggested fix sketch (high-level)

Add a field to `ValidationError`:

```rust
pub struct ValidationError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub element_path: Option<String>,  // NEW: XPath-style "/foo/bar/baz" or just element name
}
```

The validator already walks the instance document tree to find
violations — passing the current element/node path through to the
emission site should be a localized change in `src/xsd/validation.rs`.

---

## Issue 2 — Pattern/restriction facets on imported-namespace types are not applied

### Summary

When an XSD imports another namespace via `<xs:import>` and an element
in the primary schema is typed against an imported type (e.g.
`<xs:element name="Id" type="dcml:UUIDType"/>`), uppsala loads the
imported schema (when `schemaLocation` is provided) but does NOT
apply that imported type's facets (patterns, enumerations, length
restrictions, etc.) during instance validation.

Built-in types (`xs:dateTime`, `xs:positiveInteger`, etc.) and types
defined in the primary schema's own targetNamespace work normally.
Only imported-namespace types are skipped.

### Reproducer

`specs/imf-cpl.xsd` (SMPTE ST 2067-3:2013 CPL schema) declares:

```xml
<xs:import namespace="http://www.smpte-ra.org/schemas/433/2008/dcmlTypes/"/>
...
<xs:element name="Id" type="dcml:UUIDType"/>
```

The dcml stub at `specs/dcml-types-stub.xsd` (vendored in this
repo) declares:

```xml
<xs:simpleType name="UUIDType">
  <xs:restriction base="xs:anyURI">
    <xs:pattern value="urn:uuid:[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-..."/>
  </xs:restriction>
</xs:simpleType>
```

The integration test pins the behavior (currently `#[ignore]`d
because uppsala doesn't apply the facet):

```rust
// crates/imferno-core/tests/xsd_runtime.rs
#[test]
#[ignore = "uppsala v0.4.0 doesn't apply imported-namespace type facets"]
fn composite_schema_catches_dcml_typed_violations() {
    let bad_uuid = r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
        <Id>not-a-uuid</Id>
        ...
    </CompositionPlaylist>"#;
    let issues = validate_against_composite_schema(bad_uuid, &primary, &specs, None);
    // EXPECTED: issues contains XSD/PatternInvalid for "not-a-uuid"
    //   (the dcml:UUIDType pattern facet should fire)
    // OBSERVED: issues is empty — uppsala silently accepts the value
    assert!(issues.iter().any(|i| i.code == "XSD/PatternInvalid"));
}
```

The schema IS being loaded — `composite_schema_validates_real_fixture_with_dcml_types_bound`
passes, confirming the import resolution itself works. The facet
just isn't applied during validation.

### Impact for imferno

Every CPL/PKL/SCM Id, EditRate, and TrackFileId is typed against
`dcml:UUIDType` or `dcml:RationalType`. uppsala's silent acceptance
of any string for these fields means our runtime-XSD pass cannot
catch malformed UUIDs or non-rational EditRates without
post-processing the parsed AST ourselves.

For most real-world IMF documents this is a theoretical gap —
mainstream IMF tooling (IMFTool, Photon, EZ-Validator) always emits
the URN form of UUIDs and well-formed RationalTypes. But for
strict XSD-conformance validation (e.g. a hostile or
malformed-but-parseable input), uppsala lets it through.

### Workarounds attempted

1. **Tightening our own parser** — we tightened `ImfUuid::parse` to
   add a `parse_urn` variant requiring the `urn:uuid:` prefix and
   wired it into SCM parsing. This works for SCM (which uses a raw
   intermediate type) but couldn't be applied to CPL XML without
   breaking the JSON API (same serde `Deserialize` impl handles
   both). Partial win, documented in commit `2baa301`.

2. **Shim layer** — considered building a post-uppsala dcml-type
   facet checker that walks the parsed CPL and applies pattern
   facets ourselves. Rejected because once the CPL is parsed into
   the typed Rust struct, the original string forms are gone
   (UUIDs become `Uuid` values, Rationals become
   `{numerator, denominator}`), so the URN prefix presence/absence
   is unrecoverable.

### Suggested fix sketch (high-level)

The fix lives in `src/xsd/validation.rs` where simple-type
validation happens. When the validator looks up a type by QName
across an import boundary, it currently retrieves the type
definition but appears not to dispatch the facet-check pass over
the value. The fix is to apply the same facet-validation logic to
imported types that is applied to types in the primary
targetNamespace.

---

## Mitigations in imferno today

The branch `spike/xsd-runtime` ships the runtime-XSD architecture
with both limitations documented in code:

- `crates/imferno-core/src/xsd/mod.rs` — docstring on
  `validate_against_composite_schema` documents Issue 2 inline.
- `crates/imferno-core/tests/xsd_runtime.rs` — Issue 2 is pinned via
  the `#[ignore]`d integration test
  `composite_schema_catches_dcml_typed_violations`. When uppsala
  lands a fix, un-ignoring this test auto-detects it.
- `crates/imferno-core/src/validation/mod.rs` — 12 `core_flags_*`
  tests marked `#[ignore]` with a uniform reason
  `"XSD-overlap check gutted; runtime-XSD validator will re-emit"`.
  These are the tests Issue 1's fix would unblock by enabling
  per-element catalogue-code refinement.

## How to verify a future uppsala release fixes either issue

```bash
cd imferno
git checkout spike/xsd-runtime

# Bump uppsala version in crates/imferno-core/Cargo.toml, then:
cargo test -p imferno-core --features xsd-runtime --test xsd_runtime -- --include-ignored

# Issue 2 fixed → composite_schema_catches_dcml_typed_violations passes
# Issue 1 fixed → start refining the translator and un-ignore the
#                 12 core_flags_* tests in src/validation/mod.rs
```

## Alternatives evaluated

For context on why uppsala was picked despite these gaps:

| Crate | XSD validation | Verdict |
|---|---|---|
| **uppsala** | XSD 1.1, two gaps documented above | Picked: best pure-Rust option |
| **xmloxide** | XSD 1.0 subset; silently lax-validates on unresolved imports (worse than uppsala on the same axis) | Rejected |
| **exml** | libxml2 port, "implementation is still insufficient" per author | Not production-ready |
| **anyxml** | Same author as exml, newer; explicitly chose RELAX NG over XSD | Wrong tool for our use case |
| **libxml2** (C, via FFI) | Full XSD, full element paths, applies imported facets | Rejected for WASM portability; also officially unmaintained since Dec 2025 |
| **libxml2-wasm** (JS) | Same as libxml2 via JS | Open option for the WASM path specifically; trades pure-Rust for the gaps closed |

See `crates/imferno-core/examples/xsd_validate_spike_uppsala.rs` and
`crates/imferno-core/examples/xsd_validate_spike.rs` for the
side-by-side evaluation that drove the choice.
