# Spec ↔ XSD comparison docs

One `.md` per vendored SMPTE XSD, pairing each schema construct
against its prose clause and inventorying the prose-only residue
the schema cannot express.

## Why these exist

SMPTE XSDs are explicit normative elements of their parent standard
(e.g. ST 2067-3:2013 §5.1: "Each Composition Playlist instance
shall conform to the XML schema definitions"). They carry the
structural and simple-type slice; the prose carries everything XSD's
grammar can't — value-set membership, cross-field consistency,
conditional cardinality, cross-document references, semantic
invariants. The "prose shall take precedence" clause in §5.1 is a
tell that XSD's expressive limits are known by the spec author.

The comparison docs are the citation backbone. They support:

- **Runtime-XSD architecture decision** — knowing exactly which
  hand-rolled imferno checks are redundant with an XSD pass and
  which cover prose-only constraints that must stay hand-rolled.
- **Catalogue coverage audit** — for each spec, which prose
  constraints are implemented in our catalogue and which are gaps.
- **SMPTE re-publication detection** — the frontmatter records
  XSD and PDF sha256 hashes. Re-fetching shows what shifted.

## Per-doc shape

Frontmatter records: spec doc, edition, XSD path + sha256 + line
count, prose URL + sha256 + page count, catalogue files.

Body has four sections plus a coverage cross-check:

- **A. XSD construct inventory** — every XSD construct paired with
  its prose section. Status: `matches`, `under-constrained`, or
  `schema-only`.
- **B. Prose constraints not in XSD** — prose-only residue
  classified by category (value-set, cross-field, conditional,
  cross-doc, semantic, uniqueness).
- **C. Conflicts** — should be empty per §5.1; flag if found.
- **D. Gaps** — uncited rows. Either resolve into A/B with deeper
  reading, or surface as a spec ambiguity.
- **Catalogue coverage cross-check** — for each Section B row,
  whether our catalogue already implements it.

## Authoring discipline

**Citation-required.** Every row in A/B/C cites both the XSD line
and the prose §. Anything uncited goes in D explicitly marked
"needs verification" — never silently confabulated.

**Short quotes only.** Prose constraints are referenced by their §
number and a short identifying phrase (e.g. `§6.1.3 — "shall
indicate the time and date"`). Don't reproduce paragraphs.

**Bottom-up.** Walk the XSD first (Section A from schema → prose),
then walk the prose (Section B from prose → schema). Two passes
catch things one-pass missed.

**Spot-check before declaring done.** Pick 5 random rows in A and
5 in B. Verify each citation. If any are wrong, audit the rest.

## Running the fetch tool

```bash
# Generate a single skeleton for one XSD
python3 specs/comparisons/_tools/fetch.py imf-cpl.xsd

# Generate skeletons for all vendored XSDs
python3 specs/comparisons/_tools/fetch.py --all
```

The tool fetches the matching prose PDF (cached in `/tmp/spec-staging/`),
hashes both XSD and PDF, records page count, and writes a
pre-filled frontmatter + empty section headers into
`specs/comparisons/<basename>.md`. The body (A/B/D rows) is
human-filled — the tool intentionally doesn't auto-generate
comparison content because that's the interpretive work this
document is for.

Requires `pypdf` (install with `pip install --user pypdf`).

## Manifest

`_tools/manifest.json` maps each vendored XSD basename to its
matching SMPTE prose PDF on pub.smpte.org. Editions are aligned —
the XSD's publication year matches the PDF's. Three specs have no
free PDF on the public preview and get stub docs:
- `ST429-14-2014.xsd` (Auxiliary Data)
- `st430-1-2017-kdm.xsd` (KDM)
- `st430-3-2012.xsd` (ETM) — actually free, this list is wrong if it includes 430-3

(Check `manifest.json` for the current paywall list — entries with
`prose_url: null`.)

## Index

| Comparison | Spec | Edition | Status |
|---|---|---|---|
| [imf-core-constraints-20130620.md](imf-core-constraints-20130620.md) | ST 2067-2 | 2013 | — |
| [st2067-2a-2016.md](st2067-2a-2016.md) | ST 2067-2 | 2016 | — |
| [st2067-2a-2020.md](st2067-2a-2020.md) | ST 2067-2 | 2020 | — |
| [imf-cpl.md](imf-cpl.md) | ST 2067-3 | 2013 | pilot |
| [st2067-3a-2016.md](st2067-3a-2016.md) | ST 2067-3 | 2016 | — |
| [st2067-3a-2020.md](st2067-3a-2020.md) | ST 2067-3 | 2020 | — |
| [st2067-21a-2016.md](st2067-21a-2016.md) | ST 2067-21 | 2016 | — |
| [st2067-100a-2014.md](st2067-100a-2014.md) | ST 2067-100 | 2014 | — |
| [st2067-102a-2014.md](st2067-102a-2014.md) | ST 2067-102 | 2014 | — |
| [st2067-103b-2014.md](st2067-103b-2014.md) | ST 2067-103 | 2014 | — |
| [SMPTE-429-8-PKL-2007.md](SMPTE-429-8-PKL-2007.md) | ST 429-8 | 2007 | — |
| [ST429-14-2014.md](ST429-14-2014.md) | ST 429-14 | 2014 | — |
| [st430-1-2017-kdm.md](st430-1-2017-kdm.md) | ST 430-1 | 2017 | — |
| [st430-3-2012.md](st430-3-2012.md) | ST 430-3 | 2012 | — |
