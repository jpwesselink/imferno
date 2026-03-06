# imferno-wasm Examples

Examples showing how to use `imferno-wasm` for IMF package validation.

## Quick Start

```javascript
import { buildReport, formatReport } from 'imferno-wasm';

const report = buildReport({
    'ASSETMAP.xml': assetmapXml,
    'PKL_abc.xml': pklXml,
    'CPL_def.xml': cplXml,
});

console.log(formatReport(report));
console.log('Compliant:', report.validation.is_compliant);
```

## Validate with Options

```javascript
import { buildReport, codes } from 'imferno-wasm';

const report = buildReport(
    {
        'ASSETMAP.xml': assetmapXml,
        'PKL_abc.xml': pklXml,
        'CPL_def.xml': cplXml,
    },
    {
        coreSpec: 'v2020',
        app2eSpec: 'v2023',
        rules: {
            [codes.ST2067_21_2023.AppIdMismatch]: 'error',
        },
    },
);
```

## API

| Function | Description |
|----------|-------------|
| `buildReport(files, options?)` | Validate a full IMF package, returns structured report |
| `formatReport(report)` | Pretty-print an ImfReport as a human-readable string |
| `codes` | Typed validation code constants for use in `rules` config |
| `getVersion()` | Get library version |

## Node.js with Filesystem Access

For path-based validation and MXF header checks, use [`@imferno/node`](https://www.npmjs.com/package/@imferno/node) instead.
