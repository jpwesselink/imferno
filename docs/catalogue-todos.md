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

**Action**: for each, decide intentional-vs-gap; fix gaps; for
intentional sameness, annotate per Item 1.

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

**Action**: pick a naming convention and apply it consistently.
Suggestions:
- `…/CodingEquations-Missing` vs `…/CodingEquations-Invalid`
- Or category prefix: `Presence/CodingEquations` vs
  `Validity/CodingEquations`

**Acceptance**: no two codes share a name across sections; the kind of
violation is unambiguous from the code alone.

---

## 4. Fix category mis-tagging; add Container/Wrapping

**Observed**:
- `ST2067-202:*` (ISXD) codes are tagged `Audio`. ISXD is **data**
  essence (subtitles, ancillary data, dynamic metadata), not audio.
- MXF-level issues currently fold into `Encoding` alongside
  codec-level issues. They're going to need their own bucket as MXF
  coverage deepens.

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

---

## 6. Add `Example violation` and `Remediation hint` columns

**Observed**: each catalogue row currently has `Code`, `Description`,
`Default Severity`, `Category`. That's a reference. Two more columns
turn it into a tool.

**Action**: add to every row:
- **`Example violation`** — a one-line snippet showing what triggers
  the rule.
- **`Remediation hint`** — concrete fix instruction, e.g. "Add a
  `CompositionTimecode/TimecodeStartAddress` element matching format
  `HH:MM:SS:FF`" or "Re-encode the audio at 48 kHz."

**Strategic note**: this is the highest-leverage documentation work
available. It converts the catalogue from "documentation of what
Imferno checks" into "the public reference for what to do when X
breaks." That's the asset that matters more than the engine.

**Acceptance**: every catalogue row has populated example + remediation
columns; the published table renders them inline.

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
  it could surface the `remediation hint` alongside if the engine
  starts emitting it on each issue. Trivial UI addition once the
  field exists.

None of these block the engine work — they're cleanup the studio
should do in lockstep with the engine releases.

---

## Strategic context (out of scope here, but worth keeping in mind)

The catalogue itself is the load-bearing asset — more than the engine.
Rule-coverage count is a more credible benchmark than wall-clock
speed. A paid version with delivery-profile overlays (Netflix,
Disney, BBC, Amazon) is a defensible commercial product. See the
review thread for the longer version.
