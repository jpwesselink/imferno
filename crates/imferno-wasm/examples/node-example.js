// Validate an IMF package directory with @imferno/wasm from Node.js.
//
// Build first (./build.sh), then:
//   node examples/node-example.js ../../test-data/MERIDIAN_Netflix_Photon_161006

import { readdirSync, readFileSync } from 'fs';
import { join } from 'path';
import { validate, formatReport, getVersion } from '../index.js';

const dir = process.argv[2];
if (!dir) {
    console.error('usage: node examples/node-example.js <imp-directory>');
    process.exit(1);
}

// The WASM build has no filesystem access: pass the XML files as a map.
const files = {};
for (const name of readdirSync(dir)) {
    if (name.toLowerCase().endsWith('.xml')) {
        files[name] = readFileSync(join(dir, name), 'utf-8');
    }
}

console.log(`imferno ${await getVersion()}\n`);

const result = await validate(files, { app2eSpec: 'auto' });
console.log(await formatReport(result));

for (const cpl of Object.values(result.package.compositionPlaylists)) {
    const title = typeof cpl.contentTitle === 'string' ? cpl.contentTitle : cpl.contentTitle.$text;
    console.log(`CPL ${cpl.id}: ${title}`);
}

process.exitCode = result.validation.is_compliant ? 0 : 1;
