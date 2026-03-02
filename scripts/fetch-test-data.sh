#!/usr/bin/env bash
set -euo pipefail

# Fetch MXF test fixtures from GitHub Release assets.
# These are not stored in git — run this before `cargo test`.

REPO="jpwesselink/imferno"
TAG="test-data-v1"
ARCHIVE="imferno-test-data-mxf.tar.gz"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Skip if MXFs already present
if compgen -G "$ROOT/test-data/**/*.mxf" > /dev/null 2>&1; then
    echo "MXF files already present, skipping download."
    echo "To force re-download, run: find test-data -name '*.mxf' -delete"
    exit 0
fi

echo "Downloading test data from github.com/$REPO/releases/tag/$TAG ..."

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

gh release download "$TAG" -p "$ARCHIVE" -R "$REPO" -D "$TMPDIR"
tar xzf "$TMPDIR/$ARCHIVE" -C "$ROOT"

COUNT=$(find "$ROOT/test-data" -name '*.mxf' | wc -l | tr -d ' ')
echo "Extracted $COUNT MXF files into test-data/"
