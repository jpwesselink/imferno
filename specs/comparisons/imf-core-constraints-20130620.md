---
spec: ST 2067-2
edition: 2013
title: IMF Core Constraints
xsd: specs/imf-core-constraints-20130620.xsd
xsd_sha256: 5e87522b8ac7de5bb35917843ad0c90cb20e4b8ae611020f8f5c494dbccf1764
xsd_lines: 35
namespace: http://www.smpte-ra.org/schemas/2067-2/2013
prose_url: https://pub.smpte.org/doc/st2067-2/20130829-pub/st2067-2-2013.pdf
prose_sha256: 222a2a80afbc711faf76c1250b8b4ce5d71cf953dde8ed69fe5f282608224937
prose_pages: 31
catalogue_files:
  - crates/imferno-core/src/assetmap/codes.rs
---

## Summary
- _N_ XSD constructs inventoried
- _M_ prose constraints classified
- _K_ prose-only residue (semantic / cross-field / cross-doc)
- _0_ conflicts

## A. XSD construct inventory

Each XSD construct paired with the prose section that defines it.
Status values: `matches` (XSD and prose say the same thing),
`under-constrained` (XSD looser than prose — prose adds value-set
or other tightening), `schema-only` (prose silent, schema is the
normative source per the spec's "cardinality and defaults are
specified in the schema only" delegation).

| XSD line(s) | Construct | Prose § | Status | Notes |
|---|---|---|---|---|
| | | | | |

## B. Prose constraints not in XSD

Constraints carried by prose that XSD's grammar cannot express:
value-set membership, cross-field consistency, conditional
cardinality, cross-document references, semantic invariants.

| Prose § | Constraint | Category | Notes |
|---|---|---|---|
| | | | |

## C. Conflicts (prose vs XSD)

Per the spec's "prose shall take precedence" clause, conflicts
should not exist — if found here, this is a defect to report.

_None found._

## D. Gaps

Rows that could not be confidently cited on both sides. Each gap
either resolves with deeper reading (and moves to A/B) or surfaces
a structural ambiguity in the spec.

| Item | Reason |
|---|---|
| | |

## Catalogue coverage cross-check

For each row in Section B (prose-only constraints), the catalogue
should either already implement it OR this is a coverage TODO.
Listed here for review:

| Section B row | Implemented? | Catalogue code (if yes) |
|---|---|---|
| | | |
