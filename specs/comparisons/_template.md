---
spec: ST XXXX-Y
edition: YYYY
title: Short title
xsd: specs/<basename>.xsd
xsd_sha256: <hash>
xsd_lines: <N>
namespace: <target namespace>
prose_url: <pub.smpte.org URL>
prose_sha256: <hash>
prose_pages: <N>
catalogue_files:
  - crates/imferno-core/src/<module>/codes.rs
---

## Summary
- _N_ XSD constructs inventoried
- _M_ prose constraints classified
- _K_ prose-only residue (semantic / cross-field / cross-doc)
- _0_ conflicts

## A. XSD construct inventory

Each XSD construct paired with the prose section that defines it.
Status values:
- `matches` — XSD and prose say the same thing
- `under-constrained` — XSD looser than prose; prose adds value-set or other tightening
- `schema-only` — prose silent, schema is the normative source (per the standard's "cardinality and defaults are specified in the schema only" delegation, where present)

| XSD line(s) | Construct | Prose § | Status | Notes |
|---|---|---|---|---|
| | | | | |

## B. Prose constraints not in XSD

Constraints carried by prose that XSD's grammar cannot express:
value-set membership, cross-field consistency, conditional cardinality,
cross-document references, semantic invariants.

| Prose § | Constraint | Category | Notes |
|---|---|---|---|
| | | | |

## C. Conflicts (prose vs XSD)

Per the spec's "prose shall take precedence" clause, conflicts
should not exist — if found, report to SMPTE registrar.

_None found._

## D. Gaps

Rows that could not be confidently cited on both sides.

| Item | Reason |
|---|---|
| | |

## Catalogue coverage cross-check

| Section B row | Implemented? | Catalogue code (if yes) |
|---|---|---|
| | | |
