# @imferno/wasm Examples

Examples showing how to use `@imferno/wasm` for IMF package validation and parsing.

## Quick Start

```javascript
import { validate } from '@imferno/wasm';

const result = await validate({
    'ASSETMAP.xml': assetmapXml,
    'PKL_abc.xml': pklXml,
    'CPL_def.xml': cplXml,
});

console.log('Compliant:', result.report.is_compliant);
console.log('Errors:', result.report.errors);
console.log('Warnings:', result.report.warnings);
console.log('CPLs:', result.cpls);
```

## Validate with Options

```javascript
import { validate } from '@imferno/wasm';

const result = await validate(
    {
        'ASSETMAP.xml': assetmapXml,
        'PKL_abc.xml': pklXml,
        'CPL_def.xml': cplXml,
    },
    {
        coreSpec: 'v2020',
        app2eSpec: 'v2023',
        rules: {
            'ST2067-21:2023:7.1/AppIdMismatch': 'error',
        },
    },
);
```

## Parse Individual Files

```javascript
import {
    parseCplTyped,
    parseAssetmapTyped,
    parsePklTyped,
    parseVolindexTyped,
} from '@imferno/wasm';

const cpl = await parseCplTyped(cplXml);
const assetMap = await parseAssetmapTyped(assetmapXml);
const pkl = await parsePklTyped(pklXml);
const volindex = await parseVolindexTyped(volindexXml);
```

## API

| Function | Description |
|----------|-------------|
| `validate(files, options?)` | Validate a full IMF package, returns report + parsed data |
| `parseCplTyped(xml)` | Parse CPL XML |
| `parseAssetmapTyped(xml)` | Parse ASSETMAP.xml |
| `parsePklTyped(xml)` | Parse PKL XML |
| `parseVolindexTyped(xml)` | Parse VOLINDEX.xml |
| `getVersion()` | Get library version |

WASM initialization is handled automatically on first call.

## Node.js with Filesystem Access

For path-based validation, hash verification, and MXF header checks, use [`@imferno/node`](https://www.npmjs.com/package/@imferno/node) instead.
