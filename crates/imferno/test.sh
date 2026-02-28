#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/target/release/imferno"
TEST_DATA="$ROOT/test-data/MERIDIAN_Netflix_Photon_161006"

echo "Building imferno CLI..."
cargo build --release -p imferno

echo ""
echo "imferno $($BIN validate --help 2>&1 | head -1 || true)"
echo ""

# validate (summary)
echo "=== validate (summary) ==="
$BIN validate "$TEST_DATA"

echo ""

# validate (json)
echo "=== validate (json) ==="
$BIN validate "$TEST_DATA" --format json | node -e "
const data = JSON.parse(require('fs').readFileSync('/dev/stdin', 'utf-8'));
console.log('Compliant:', data.is_compliant);
console.log('Errors:', data.errors.length);
console.log('Warnings:', data.warnings.length);
"

echo ""

# validate (detailed)
echo "=== validate (detailed) ==="
$BIN validate "$TEST_DATA" --format detailed

echo ""

# validate with rules override
echo "=== validate with rules override ==="
RULES_FILE=$(mktemp)
echo '{ "ST2067-21:2023:7.1/AppIdMismatch": "error" }' > "$RULES_FILE"
$BIN validate "$TEST_DATA" --rules-config "$RULES_FILE" --exit-zero
rm -f "$RULES_FILE"

echo ""

# inspect
echo "=== inspect ==="
$BIN inspect "$TEST_DATA"

echo ""

# export
echo "=== export (first 20 lines) ==="
$BIN export "$TEST_DATA" | head -20

echo ""
echo "All tests passed."
