#!/usr/bin/env bash
# Build @imferno/node and run its tests (incl. the type contract test).
set -euo pipefail

cd "$(dirname "$0")"

[ -d node_modules ] || npm install

echo "Building @imferno/node..."
npx napi build --release

echo ""
echo "Smoke test..."
node -e "
const m = require('.');
const result = m.validatePath('../../test-data/MERIDIAN_Netflix_Photon_161006');
console.log('imferno-node v' + m.getVersion());
console.log('Compliant:', result.validation.is_compliant);
console.log('Errors:', result.validation.errors.length);
console.log('Warnings:', result.validation.warnings.length);
console.log('CPLs:', Object.keys(result.package.compositionPlaylists).length);
console.log('PKLs:', Object.keys(result.package.packingLists).length);
"

echo ""
echo "Testing @imferno/node..."
npx vitest run
