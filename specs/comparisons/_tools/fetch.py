#!/usr/bin/env python3
"""Generate the skeleton comparison .md for a vendored XSD.

Usage:
    python3 specs/comparisons/_tools/fetch.py <xsd-basename>
    python3 specs/comparisons/_tools/fetch.py --all

For a given XSD basename, looks up the matching SMPTE prose PDF URL
from manifest.json, fetches and hashes it, extracts the page count,
hashes the XSD, and writes specs/comparisons/<basename-without-.xsd>.md
with frontmatter + empty sections. The body (Sections A/B/D rows)
stays human-filled.

For paywalled specs (prose_url=null in manifest) writes a stub doc.

PDF cache lives in /tmp/spec-staging/ to avoid re-downloads.
Does NOT vendor the PDFs (they're frequently re-published; the
citation is URL + sha256, not a local copy).
"""

from __future__ import annotations

import hashlib
import json
import sys
import urllib.request
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
TOOLS_DIR = Path(__file__).resolve().parent
COMPARISONS_DIR = REPO_ROOT / "specs" / "comparisons"
SPECS_DIR = REPO_ROOT / "specs"
PDF_CACHE = Path("/tmp/spec-staging")
MANIFEST = TOOLS_DIR / "manifest.json"


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def fetch_pdf(url: str, dest: Path) -> None:
    PDF_CACHE.mkdir(exist_ok=True)
    if dest.exists() and dest.stat().st_size > 0:
        return
    req = urllib.request.Request(url, headers={"User-Agent": "imferno-spec-fetch/1"})
    with urllib.request.urlopen(req) as resp, dest.open("wb") as f:
        f.write(resp.read())


def pdf_page_count(path: Path) -> int:
    import pypdf

    return len(pypdf.PdfReader(str(path)).pages)


def xsd_line_count(path: Path) -> int:
    return sum(1 for _ in path.open("rb"))


def skeleton(xsd_name: str, info: dict, paywalled: bool) -> str:
    """Build the .md skeleton with frontmatter + empty section headers."""
    xsd_path = SPECS_DIR / xsd_name
    xsd_sha = sha256_file(xsd_path)
    xsd_lines = xsd_line_count(xsd_path)

    if paywalled:
        prose_block = (
            "prose_url: null  # PDF not available on pub.smpte.org public preview\n"
            "prose_sha256: null\n"
            "prose_pages: null"
        )
        body = (
            "## Status: deferred\n\n"
            "Prose is not available on pub.smpte.org public preview "
            "(paywalled or not yet released). Comparison cannot be "
            "authored without the source document.\n\n"
            "Re-run `python3 specs/comparisons/_tools/fetch.py "
            f"{xsd_name}` if the prose becomes available later.\n"
        )
    else:
        pdf_url = info["prose_url"]
        pdf_dest = PDF_CACHE / Path(pdf_url).name
        fetch_pdf(pdf_url, pdf_dest)
        prose_sha = sha256_file(pdf_dest)
        pages = pdf_page_count(pdf_dest)
        prose_block = (
            f"prose_url: {pdf_url}\n"
            f"prose_sha256: {prose_sha}\n"
            f"prose_pages: {pages}"
        )
        body = SECTION_BODY

    catalogue = info.get("catalogue_files", [])
    catalogue_yaml = (
        "catalogue_files:\n"
        + "\n".join(f"  - {p}" for p in catalogue)
        if catalogue
        else "catalogue_files: []"
    )

    return f"""---
spec: {info['spec']}
edition: {info['edition']}
title: {info['title']}
xsd: specs/{xsd_name}
xsd_sha256: {xsd_sha}
xsd_lines: {xsd_lines}
namespace: {info.get('namespace') or 'null'}
{prose_block}
{catalogue_yaml}
---

{body}"""


SECTION_BODY = """## Summary
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
"""


def write_doc(xsd_name: str, info: dict) -> Path:
    paywalled = info.get("prose_url") is None
    content = skeleton(xsd_name, info, paywalled)
    out = COMPARISONS_DIR / (Path(xsd_name).stem + ".md")
    out.write_text(content)
    return out


def main(argv: list[str]) -> int:
    manifest = json.loads(MANIFEST.read_text())["specs"]

    if len(argv) >= 2 and argv[1] == "--all":
        for xsd_name, info in manifest.items():
            out = write_doc(xsd_name, info)
            print(f"  wrote {out.relative_to(REPO_ROOT)}")
        return 0

    if len(argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2

    xsd_name = argv[1]
    if xsd_name not in manifest:
        print(f"unknown xsd: {xsd_name}", file=sys.stderr)
        print(f"known: {', '.join(sorted(manifest))}", file=sys.stderr)
        return 1

    out = write_doc(xsd_name, manifest[xsd_name])
    print(f"wrote {out.relative_to(REPO_ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
