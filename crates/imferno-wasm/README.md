# @imferno/wasm

SMPTE ST 2067 IMF parser and validator for JavaScript and TypeScript, powered by WebAssembly.

Part of the [`imferno`](https://github.com/jpwesselink/imferno) ecosystem. See also [`@imferno/schema`](https://www.npmjs.com/package/@imferno/schema) for JSON Schema validation of imferno output.

## Install

```bash
npm install @imferno/wasm
```

> **Note:** For Node.js with filesystem access (path-based validation, hash verification), use [`@imferno/node`](https://www.npmjs.com/package/@imferno/node) instead.

The package ships a prebuilt `.wasm` binary -- no build step required.

## Usage

```javascript
import { buildReport, formatReport, codes, getVersion } from '@imferno/wasm';

// Validate a full IMF package (pass all XML files as a map)
const report = await buildReport({
    'ASSETMAP.xml': assetmapXml,
    'PKL_abc.xml': pklXml,
    'CPL_def.xml': cplXml,
});

// Pretty-print the report
console.log(await formatReport(report));

// Check compliance programmatically
if (!report.validation.is_compliant) {
    for (const err of report.validation.errors) {
        console.error(err.code, err.message);
    }
}

// Inspect package metadata
console.log('CPL count:', report.package.cplCount);
console.log('CPLs:', report.cpls);

// Get library version
console.log(await getVersion());
```

### Validate with options

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
            [codes.ST2067_2_2020.FileNotFound]: 'critical',
            [codes.ST2067_2_2020.ChecksumMismatch]: 'off',
        },
    },
);
```

WASM initialization is handled automatically on first call.

## API

| Export | Description |
|--------|-------------|
| `buildReport(files, options?)` | Parse and validate an IMF package, returns an `ImfReport` |
| `formatReport(report)` | Render an `ImfReport` as a human-readable string |
| `codes` | Typed validation code constants for use in `rules` config |
| `getVersion()` | Get library version |

All exports are **async** (WASM must be initialized before use).

### ImfReport shape

```json
{
    "package": {
        "assetMapId": "...",
        "cplCount": 1,
        "pklCount": 1,
        "assetCount": 5,
        "unreferencedAssets": []
    },
    "cpls": [
        {
            "id": "...",
            "title": "...",
            "applicationProfile": "App2E_2023",
            "editRate": "24000/1001",
            "segmentCount": 1,
            "isSupplemental": false,
            "sequences": [],
            "markers": []
        }
    ],
    "validation": {
        "is_compliant": true,
        "is_playable": true,
        "critical": [],
        "errors": [],
        "warnings": [],
        "info": []
    }
}
```

### Options

| Field | Values | Default |
|-------|--------|---------|
| `coreSpec` | `"auto"`, `"v2013"`, `"v2016"`, `"v2020"` | auto-detect |
| `app2eSpec` | `"auto"`, `"none"`, `"v2020"`, `"v2021"`, `"v2023"` | auto-detect |
| `rules` | ESLint-style rules object (`{ [code]: severity }`) | all enabled |

Rule severities: `"critical"`, `"error"`, `"warning"`, `"info"`, `"off"`.

## License

MIT
