# imferno-wasm

SMPTE ST-2067 IMF parser and validator for JavaScript and TypeScript, powered by WebAssembly.

Part of the [`imferno`](https://github.com/jpwesselink/imferno) ecosystem.

## Install

```bash
npm install imferno-wasm
```

> **Note:** For Node.js with filesystem access (path-based validation, hash verification), use [`@imferno/node`](https://www.npmjs.com/package/@imferno/node) instead.

The package ships a prebuilt `.wasm` binary — no build step required.

## Usage

```javascript
import { buildReport, formatReport, codes, getVersion } from 'imferno-wasm';

// Validate a full IMF package (pass all XML files as a map)
const report = buildReport({
    'ASSETMAP.xml': assetmapXml,
    'PKL_abc.xml': pklXml,
    'CPL_def.xml': cplXml,
});

// Pretty-print
console.log(formatReport(report));

// Check programmatically
if (!report.validation.is_compliant) {
    for (const err of report.validation.errors) {
        console.error(err.code, err.message);
    }
}

// Validate with custom rules (typed codes give autocomplete + typo protection)
const report2 = buildReport(
    {
        'ASSETMAP.xml': assetmapXml,
        'PKL_abc.xml': pklXml,
        'CPL_def.xml': cplXml,
    },
    {
        coreSpec: 'v2020',
        app2eSpec: 'v2023',
        rules: {
            [codes.ST2067_2_2020.FileNotFound]: 'critical',
            [codes.ST2067_2_2020.ChecksumMismatch]: 'off',
        },
    },
);
```

## API

| Export | Description |
|--------|-------------|
| `buildReport(files, options?)` | Validate a full IMF package, returns structured report |
| `formatReport(report)` | Pretty-print an ImfReport as a human-readable string |
| `codes` | Typed validation code constants for use in `rules` config |
| `getVersion()` | Get library version |

### Spec selection values

- `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"`
- `app2eSpec`: `"auto"` | `"none"` | `"v2020"` | `"v2021"` | `"v2023"`

## License

MIT
