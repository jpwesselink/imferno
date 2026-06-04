# Catalogue / error-code TODOs

External review of the published rule catalogue surfaced six concrete
actionable items. Recorded here so they don't get lost.

Source: review thread, 2026-05-27.

---

## 1. Cross-edition duplication — decide & annotate

**Observed**: ST 2067-2:2013 / 2016 / 2020 blocks are nearly identical
(only meaningful difference: 2013 has fewer `XSD-*` codes; 2016 and
2020 are essentially copy-paste).

**Diagnose**:
- Are the constraints genuinely stable across editions, or have we
  over-generated codes per edition where the rules are version-stable?

**Then act**:
- If stable across editions, either:
  - Document the rule once with an applicability range
    (`applies: [2013, 2016, 2020]`) and emit the edition as a separate
    field on the issue, OR
  - Keep per-edition codes but add a `same-as-previous-edition: true`
    annotation so consumers know the constraint didn't actually
    change.
- Either way, surface a one-line "delta from previous edition: …"
  per edition block in the published catalogue. Preempts the obvious
  SMPTE-crowd question.

**Acceptance**: the 2013/2016/2020 blocks are visibly distinct in
either content or annotation; a reader can answer "what changed
between editions?" without diffing.

---

## 2. Audit specific cross-edition gaps

**Specifics flagged**:
- `ST2067-2:2013:10/TimedText-MalformedLanguageTag` exists in 2013
  but `TimedText-SampleRate` and `TimedText-EmptyLanguageTag` are
  missing from 2013 (present in 2016/2020). Intentional or gap?
- `ST 2067-21:2020` has one code (`AppIdMismatch`); 2023 has dozens.
  Is 2020 actually that thin, or are most 2023 codes also applicable
  to 2020 and just not duplicated?
- `ST 2067-201:2019` and `:2021` blocks are bit-for-bit identical.
  Is there an actual delta between those editions?

### Audit results (2026-06-04)

Computed by diffing the per-edition prefixes inside
`crates/imferno-core/tests/snapshots/validation-codes.txt`:

| Spec family             | Pair                | Code-set diff                          |
|-------------------------|---------------------|----------------------------------------|
| ST 2067-2 (Core)        | 2013 vs 2016        | **0** — identical (38 codes each)      |
| ST 2067-2 (Core)        | 2016 vs 2020        | **+11 in 2020** (AssetMap/PKL/checksum rules promoted into Core) |
| ST 2067-3 (CPL)         | 2013 vs 2016        | **0** — identical (14 codes each)      |
| ST 2067-201 (IAB)       | 2019 vs 2021        | **0** — identical (21 codes each)      |
| ST 2067-21 (App2E)      | 2020 vs 2023        | 2020 = 1 code, 2023 = 70 codes         |

**Per-finding verdict**:

- `TimedText-MalformedLanguageTag` / `-SampleRate` / `-EmptyLanguageTag` in
  ST 2067-2:2013 — **all three exist** in 2013. The original review
  observation was stale; current snapshot has them.
- ST 2067-21:2020 vs :2023 — **real coverage gap**, not duplication.
  2020 has only `AppIdMismatch` implemented; 2023 has 70 codes covering
  picture descriptor, audio descriptor, MCA rules. Conceptually most of
  those should apply to 2020 documents too. Tracked as catalogue gap
  rather than a cross-edition annotation problem.
- ST 2067-201:2019 vs :2021 — **bit-for-bit identical confirmed**. The
  2021 edition appears to have changed no normative content the catalogue
  models. Candidate for the "same-as-previous-edition" annotation under
  Item 1.

**Action items derived from the audit**:

- ST 2067-2:2013 ↔ 2016, ST 2067-3:2013 ↔ 2016, ST 2067-201:2019 ↔ 2021:
  three pairs of identical code sets — perfect targets for the Item 1
  "same-as-previous-edition" annotation. Tracked as catalogue-cleanup
  follow-up.
- ST 2067-2:2016 → 2020: real spec delta, no action; document the +11
  added codes in the public catalogue (PKL/AssetMap promotion).
- ST 2067-21:2020 sparse: file as a coverage ticket (out of scope for
  the cross-edition cleanup pass; rules need writing, not annotating).

---

## 3. Disambiguate field-presence vs value-validity rules

**Observed**: in ST 2067-21 picture descriptor coverage, the
`Required-*` family (`StoredWidth`, `StoredHeight`, `SampleRate`,
`FrameLayout`, `ColorPrimaries`, `TransferCharacteristic`,
`PictureCompression`, `ComponentDepth`) is one rule per field —
defensible, easier to suppress per-field. Fine.

But the rule set mixes presence vs validity inconsistently:
e.g. `6.2.1/CodingEquations` (absence) and `6.2.3/CodingEquations`
(invalid value) share a name across different sections — reads as a
collision.

**Status: ✅ already addressed (verified 2026-06-04)**.
Current codes use unambiguous suffixes:
- `ST2067-21:2023:6.2.1/CodingEquationsMissing` (presence)
- `ST2067-21:2023:6.2.3/CodingEquationsUnknown` (validity)
The same `-Missing` / `-Unknown` discipline is applied across the
picture descriptor coverage. Collision sweep across the whole
`validation/codes.rs` shows zero remaining presence-vs-validity name
clashes (the only suffix duplication is intentional cross-edition
identity, e.g. `AppIdMismatch` in 2020 + 2023).

---

## 4. Fix category mis-tagging; add Container/Wrapping

**Observed**:
- `ST2067-202:*` (ISXD) codes are tagged `Audio`. ISXD is **data**
  essence (subtitles, ancillary data, dynamic metadata), not audio.
- MXF-level issues currently fold into `Encoding` alongside
  codec-level issues. They're going to need their own bucket as MXF
  coverage deepens.

**Status: ✅ already addressed (verified 2026-06-04)**.
- ISXD codes now use `Category::Data` (see
  `crates/imferno-core/src/validation/isxd_codes.rs:80`).
- MXF codes mostly use `Category::Container` (partition packs,
  timed-text mapping kind). Only `NoEssenceContainers` and `Op1a`
  remain under `Category::Encoding` and that's intentional —
  they describe operational-pattern UL declarations, not codec
  encoding.

**Action**:
- Re-tag ISXD codes. Candidate categories: `Data`, `Metadata` (if
  that's the closest fit in the existing taxonomy), or a new `Subtitle`
  / `AncillaryData` if appropriate.
- Add `Container` (or `Wrapping`) as a category for MXF wrapping
  constraints. Reserve `Encoding` for codec-level (J2K, PCM, IAB)
  issues.

**Acceptance**: every catalogue row's category accurately reflects
where in the stack the issue lives.

---

## 5. Stabilize XSD-pinned codes

**Observed**: codes like `XSD-121-127`, `XSD-66`, `XSD-88` pin to
line numbers in the canonical XSD. If SMPTE re-releases the schema
with different formatting (whitespace, reordering), every one of
these codes becomes ambiguous.

**Action**: pick one (or both):
- **Structural ids**: `XSD/CompositionTimecode/TimecodeDropFrame-required`
  — verbose but stable across formatting changes.
- **Hash-pin**: each code refers to lines N–M of `imf-cpl-2016.xsd`
  `sha256:abc…`. Eternal, unambiguous. SMPTE folks will respect the
  rigor.

Recommended: emit BOTH — structural id as the canonical code, line
range as a `displayHint` field for human convenience.

**Document the convention** in a new `docs/error-code-format.md` so
others (downstream validators, Studio's `report.d.ts`) can rely on
it.

**Acceptance**: no rule code embeds a line number as its sole
identifier; the format spec exists and is referenced from the
catalogue header.

**Status: ✅ already addressed (verified 2026-06-04).**
- Zero line-number-pinned codes remain anywhere in
  `crates/imferno-core/src/`. The reviewer's concern about
  `XSD-121-127`, `XSD-66`, `XSD-88` reflects an earlier code shape
  that no longer exists.
- Current canonical XSD codes are structural: `XSD/ElementMissing`,
  `XSD/UnexpectedElement`, `XSD/PatternInvalid`, `XSD/TypeInvalid`,
  `XSD/SchemaConstraintFailed`
  (`crates/imferno-core/src/xsd/codes.rs`).
- Per-element refinement is done by the translator
  (`crates/imferno-core/src/xsd/mod.rs::translate`) which appends the
  uppsala `element_path` to produce `XSD/<Class>/<ElementName>`
  (e.g. `XSD/ElementMissing/EditRate`,
  `XSD/PatternInvalid/IssueDate`). This is the recommended structural
  form.
- The format convention is documented in
  [`docs/error-code-format.md`](./error-code-format.md) §
  "Stable structural identifiers for XSD-pinned codes" with a
  migration-status table.
- No hash-pinning has been added — moot since no codes carry line
  ranges. The recommendation in the format doc stays as fallback
  guidance if line ranges ever get reintroduced.

---

## 6. Add `Example violation` and `Remediation hint` columns

**Observed**: each catalogue row currently has `Code`, `Description`,
`Default Severity`, `Category`. That's a reference. Two more columns
turn it into a tool.

**Action**: add to every row:
- **`Example violation`** — a one-line snippet showing what triggers
  the rule.

**Out of scope here**: per-code remediation guidance ("how to fix
it"). Authoring accurate, spec-grounded fix instructions across the
full catalogue is its own project — and a candidate commercial
product distinct from the open-source engine — so the engine
catalogue intentionally stops at description + example.

**Acceptance**: every catalogue row has a populated `example` column;
the published table renders it inline.

---

## Studio impact (heads-up, not blocking)

When items 4, 5, 6 land, the studio side benefits from:

- **#4** — `apps/web/app/routes/settings.validation-rules.tsx` could
  color/group by the richer category taxonomy. The custom-rules
  `report.d.ts` could expose `category` as a typed enum rather than
  `string`.
- **#5** — the rule-code parser at `validation-rules.tsx` (regex
  `/^([^:]+:[^:]+):/`) still works with structural ids. No change
  needed unless we want to surface the structural id as a
  click-through.
- **#6** — Studio's report-detail page renders issue `message` only;
  once `example` is populated, the report could surface the example
  fragment alongside to help operators recognize the violation.

None of these block the engine work — they're cleanup the studio
should do in lockstep with the engine releases.

---

## Strategic context (out of scope here, but worth keeping in mind)

The catalogue itself is the load-bearing asset — more than the engine.
Rule-coverage count is a more credible benchmark than wall-clock
speed. A paid version with delivery-profile overlays (Netflix,
Disney, BBC, Amazon) is a defensible commercial product. See the
review thread for the longer version.
