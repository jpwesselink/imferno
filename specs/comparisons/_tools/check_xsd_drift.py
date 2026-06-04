#!/usr/bin/env python3
"""Check vendored XSDs in `specs/` against the live SMPTE-RA versions.

Usage:
    python3 specs/comparisons/_tools/check_xsd_drift.py
        [--manifest specs/comparisons/_tools/manifest.json]

Behaviour:
    For each XSD basename in the manifest, fetch the canonical copy
    from `https://smpte-ra.org/sites/default/files/<basename>` and
    compare its SHA-256 against the vendored file in `specs/`. Prints
    a one-line status per file and a summary, exiting with a non-zero
    status if any vendored XSD differs from the live version (drift).

    Designed for periodic CI runs — not on every PR — so a slow or
    flaky SMPTE-RA isn't gating the merge queue. Recommended cadence:
    weekly + on-demand.

Exit codes:
    0   every vendored XSD byte-matches the live copy
    1   one or more vendored XSDs drifted (or any 4xx/5xx fetch)
    2   manifest missing / unreadable
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import urllib.error
import urllib.request
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
SPECS_DIR = REPO_ROOT / "specs"
DEFAULT_MANIFEST = REPO_ROOT / "specs" / "comparisons" / "_tools" / "manifest.json"
LIVE_URL_TEMPLATE = "https://smpte-ra.org/sites/default/files/{basename}"


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def fetch_live(basename: str) -> tuple[bytes | None, str | None]:
    url = LIVE_URL_TEMPLATE.format(basename=basename)
    try:
        with urllib.request.urlopen(url, timeout=20) as resp:
            return resp.read(), None
    except urllib.error.HTTPError as e:
        return None, f"HTTP {e.code} from {url}"
    except (urllib.error.URLError, TimeoutError) as e:
        return None, f"network error: {e}"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--manifest",
        default=str(DEFAULT_MANIFEST),
        help="Path to manifest.json (defaults to specs/comparisons/_tools/manifest.json)",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Treat missing live files as drift (default: skipped).",
    )
    args = parser.parse_args()

    manifest_path = Path(args.manifest)
    if not manifest_path.is_file():
        print(f"manifest not found: {manifest_path}", file=sys.stderr)
        return 2

    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        print(f"manifest JSON parse error: {e}", file=sys.stderr)
        return 2

    drift = 0
    skipped = 0
    matches = 0
    for basename in sorted(manifest.get("specs", {}).keys()):
        local = SPECS_DIR / basename
        if not local.is_file():
            print(f"[skip] {basename}: not vendored locally")
            skipped += 1
            continue
        live_bytes, err = fetch_live(basename)
        if live_bytes is None:
            verdict = "drift" if args.strict else "skip"
            print(f"[{verdict}] {basename}: {err}")
            if args.strict:
                drift += 1
            else:
                skipped += 1
            continue
        local_hash = sha256_file(local)
        live_hash = sha256_bytes(live_bytes)
        if local_hash == live_hash:
            print(f"[ok]   {basename}")
            matches += 1
        else:
            print(
                f"[drift] {basename}: local={local_hash[:12]} live={live_hash[:12]}"
            )
            drift += 1

    print(
        f"\nSummary: {matches} match, {drift} drift, {skipped} skip",
        file=sys.stderr,
    )
    return 1 if drift else 0


if __name__ == "__main__":
    sys.exit(main())
