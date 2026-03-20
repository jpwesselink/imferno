# @imferno/wasm Examples

Examples showing how to use `@imferno/wasm` for IMF package validation.

## Quick Start

```javascript
import { buildReport, formatReport } from '@imferno/wasm';

const report = await buildReport({
    'ASSETMAP.xml': assetmapXml,
    'PKL_abc.xml': pklXml,
    'CPL_def.xml': cplXml,
});

console.log('Compliant:', report.validation.is_compliant);
console.log('Errors:', report.validation.errors);
console.log('Warnings:', report.validation.warnings);
console.log('CPLs:', report.cpls);

// Pretty-print
console.log(await formatReport(report));
```

## Validate with Options

```javascript
import { buildReport, codes } from '@imferno/wasm';

const report = await buildReport(
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

## Custom Rules with Typed Codes

```javascript
import { buildReport, codes } from '@imferno/wasm';

const report = await buildReport(files, {
    rules: {
        [codes.ST2067_2_2020.FileNotFound]: 'critical',
        [codes.ST2067_2_2020.ChecksumMismatch]: 'off',
        [codes.ST2067_21_2023.FrameRate]: 'warning',
    },
});
```

## API

| Function | Description |
|----------|-------------|
| `buildReport(files, options?)` | Parse and validate an IMF package, returns an `ImfReport` |
| `formatReport(report)` | Render an `ImfReport` as a human-readable string |
| `codes` | Typed validation code constants for use in `rules` config |
| `getVersion()` | Get library version |

All exports are **async** (WASM must be initialized before use).

## Node.js with Filesystem Access

For path-based validation, hash verification, and MXF header checks, use [`@imferno/node`](https://www.npmjs.com/package/@imferno/node) instead.
