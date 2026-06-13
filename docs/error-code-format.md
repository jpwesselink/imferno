# Validation code format

This document defines the structure of validation issue codes emitted
by Imferno. It is intended to be stable enough that downstream
tools (Studio, third-party validators, SMPTE community references) can
parse, route, suppress, and cross-reference codes mechanically.

## Code shape

```
<SPEC>:<EDITION>:<SECTION>/<IDENTIFIER>
```

Examples:
- `ST2067-2:2020:7.2/CompositionTimecode-Required`
- `ST2067-21:2023:6.2.1/CodingEquationsMissing`
- `ST2067-21:2023:6.2.3/CodingEquationsUnknown`
- `IMFERNO:Package/ParseError` (engine-internal, no spec)

Components:

| Part         | Format                              | Notes |
|--------------|-------------------------------------|-------|
| `SPEC`       | `ST<doc-number>` for Standards (e.g. `ST2067-2`, `ST429-9`). `RP<doc>` for Recommended Practices. `EG<doc>` for Engineering Guidelines. `IMFERNO` for engine-internal issues with no SMPTE source. | Future: surface `ST`/`RP`/`EG` distinction explicitly so receivers can apply different default severities to recommendations vs standards. |
| `EDITION`    | Four-digit publication year (`2013`, `2016`, `2020`, …). Omitted for `IMFERNO` codes. | |
| `SECTION`    | Spec section number (`6.2.1`) or table reference (`Table-3`) or schema element path (see XSD pinning below). | |
| `IDENTIFIER` | Camel-case symbolic name of the issue. See suffix convention below. | |

## Suffix convention for presence vs validity

For each constrained field, two kinds of issue can fire:

- **Presence**: the field is required but absent. Suffix the
  identifier with `Missing`.
  Example: `…6.2.1/CodingEquationsMissing`.
- **Validity**: the field is present but the value is wrong (out of
  range, unrecognized enum value, malformed). Suffix `Unknown` for
  unrecognized enums/ULs, otherwise use a verb that describes the
  failure (e.g. `…/SampledHeightMismatch`).
  Example: `…6.2.3/CodingEquationsUnknown`.

A "plain" identifier (no `Missing`/`Unknown` suffix) is reserved for
codes where the value-vs-presence distinction doesn't apply — e.g.
boolean-like constraints (`AlphaTransparency`, `FrameLayoutInterlaced`)
or cross-field consistency checks. Do not use a plain identifier when
both presence and validity codes exist for the same field.

## Stable structural identifiers for XSD-pinned codes

Codes that reference XSD constraints by line number — `XSD-121-127`,
`XSD-66`, `XSD-88` — are **brittle**. If SMPTE re-releases the
canonical XSD with different formatting (whitespace, declaration
reorder), the line range no longer points at the same constraint.

Going forward, XSD-derived codes SHOULD use stable structural
identifiers of the form:

```
XSD/<ParentElement>/<Child>-<constraint>
```

Examples:
- `XSD/CompositionPlaylist/ContentTitle-required`
- `XSD/CompositionTimecode/TimecodeDropFrame-required`
- `XSD/Locale/RegionList-maxOccurs`

The line range MAY be kept as a `displayHint` on the issue (for
human-readable debugging), but the canonical machine identifier is
the structural form.

### Migration status

| Spec / family       | Status |
|---------------------|--------|
| ST 2067-2 Core (CPL XSD constraints) | **Migrated** — `XSD-66/IssueDate`, `XSD-88/EditRate`, `XSD-121-127/CompositionTimecode-*` are now `XSD/CompositionPlaylist/*` and `XSD/CompositionTimecode/*` respectively. |
| ST 2067-3 (PKL XSD constraints)      | Not yet — no line-range codes currently in use. |
| Other specs                          | None of the remaining catalogues currently use line-range XSD codes. |

If new XSD line-range codes get introduced before the structural-id
migration is fully formalized, the canonical XSD file SHOULD be
hash-pinned in this table so the line range is unambiguous against a
specific schema release.

## Cross-edition applicability

When a constraint is identical across multiple editions of a spec,
prefer ONE of these two patterns — don't silently duplicate:

1. **Explicit applicability range** — emit the same code but include
   `appliesTo: [2013, 2016, 2020]` on the issue. The catalogue
   documents the rule once.
2. **Per-edition codes with delta annotation** — keep distinct codes
   per edition (`ST2067-2:2013:7.2/Foo` vs `ST2067-2:2020:7.2/Foo`)
   but annotate `sameAsPreviousEdition: <prev-prefix>` in the catalogue
   when the constraint hasn't changed. Consumers can collapse the
   presentation.

**Imferno uses pattern 2.** The `ValidationCode::previous_identical_edition()`
trait method (added 2026-06-04) returns the predecessor edition's
prefix when the catalogue is bit-for-bit identical. `listRules`
surfaces this as a `sameAsPreviousEdition` field on each affected
rule. Currently set on three editions whose catalogues are verified
identical to their predecessor:

| This edition         | sameAsPreviousEdition |
|----------------------|-----------------------|
| `ST2067-2:2016`      | `ST2067-2:2013`       |
| `ST2067-3:2016`      | `ST2067-3:2013`       |
| `ST2067-201:2021`    | `ST2067-201:2019`     |

The cross-edition snapshot diff used to derive these is in
[`catalogue-todos.md`](./catalogue-todos.md) Item 2.

## Application context

IMF has app-specific constraint sets (App #2, App #2E, App #5, …).
The current convention encodes the app inside the spec
(`ST2067-21` is App #2E). Future codes that apply only when a
specific app is in use SHOULD additionally tag the issue with an
`app: "2E"` field rather than baking the app into the identifier.

This keeps `IDENTIFIER` describing the constraint and lets receivers
filter by app independently of spec.

## Severity defaulting

Default severity per code is set in code via `default_severity()` and
falls into one of:

- `Critical` — the package is unparseable; downstream checks can't run.
- `Error` — clear conformance failure; package is not deliverable.
- `Warning` — likely problem, may be acceptable depending on
  workflow / receiver profile.
- `Info` — informational; surfaces context, not a failure.

Severities are advisory; receivers (e.g. Studio operators with
per-rule overrides) can promote or demote any code per their delivery
profile.

For `RP*` (Recommended Practice) and `EG*` (Engineering Guideline)
codes, the default should bias toward `Warning` rather than `Error` —
a recommendation violation is materially different from a standard
violation.

## Catalogue row schema

In addition to the code itself, every catalogue row exposes:

| Field             | Required | Notes |
|-------------------|----------|-------|
| `code`            | yes      | The full identifier above. |
| `description`     | yes      | One-line human-readable summary. |
| `default_severity`| yes      | See above. |
| `category`        | yes      | `Structure | Schema | Reference | Asset | Timing | Encoding | Container | Audio | Video | Subtitle | Data | Metadata | Security | StudioSpecific(name)`. |
| `example`         | should   | A one-line snippet showing what triggers the rule. |
| `appliesTo`       | optional | Editions this code is valid for (when using applicability ranges per above). |
| `displayHint`     | optional | Human-readable hint, e.g. XSD line range. |

Per-code remediation guidance is intentionally **not** carried in the
engine catalogue. Authoring accurate, spec-grounded fix instructions
is a separate body of work; see `catalogue-todos.md` for context.

## Versioning policy

Renaming a code is a **breaking change** for any consumer storing
severity overrides keyed by code string. When renaming:

1. Bump the engine version (semver minor at minimum; major if the
   consumer-visible API changes too).
2. Document the rename in the changelog with old → new mapping.
3. Where possible, ship a one-version deprecation window where both
   old and new strings emit (with the old marked deprecated in the
   catalogue) before fully removing the old.

A deprecated code MUST still appear in the catalogue with a clear
`deprecated: "use ST2067-21:2023:6.2.1/CodingEquationsMissing"`
annotation until the next major release.
