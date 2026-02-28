#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

echo "Building @imferno/node..."
napi build --release

echo ""
echo "Testing @imferno/node..."
node -e "
const m = require('./imferno-node.node');

console.log('imferno-node v' + m.getVersion());
console.log('Exports:', Object.keys(m).join(', '));
console.log('');

// validatePath
const result = m.validatePath('../../test-data/MERIDIAN_Netflix_Photon_161006');
console.log('=== validatePath ===');
console.log('Compliant:', result.report.is_compliant);
console.log('Errors:', result.report.errors.length);
console.log('Warnings:', result.report.warnings.length);
console.log('CPLs:', result.cpls.length);
console.log('PKLs:', result.packingLists.length);
console.log('VolumeIndex:', !!result.volumeIndex);

// validate (string-based)
const fs = require('fs');
const path = require('path');
const dir = '../../test-data/MERIDIAN_Netflix_Photon_161006';
const files = {};
for (const f of fs.readdirSync(dir)) {
  if (f.endsWith('.xml')) files[f] = fs.readFileSync(path.join(dir, f), 'utf-8');
}
const r2 = m.validate(files);
console.log('');
console.log('=== validate (string-based) ===');
console.log('Compliant:', r2.report.is_compliant);
console.log('Errors:', r2.report.errors.length);

// rules override
const r3 = m.validate(files, { rules: { 'ST2067-21:2023:7.1/AppIdMismatch': 'error' } });
console.log('');
console.log('=== validate with rules override ===');
console.log('Compliant:', r3.report.is_compliant);
console.log('Errors:', r3.report.errors.length);

console.log('');
console.log('All tests passed.');
"
