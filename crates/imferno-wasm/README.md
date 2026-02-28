# @imferno/wasm

SMPTE ST 2067 IMF parser and validator for JavaScript and TypeScript, powered by WebAssembly.

Part of the [`imferno`](https://github.com/jpwesselink/imferno) ecosystem. See also [`@imferno/schema`](https://www.npmjs.com/package/@imferno/schema) for JSON Schema validation of imferno output.

## Install

```bash
npm install @imferno/wasm
```

> **Note:** For Node.js with filesystem access (path-based validation, hash verification), use [`@imferno/node`](https://www.npmjs.com/package/@imferno/node) instead.

The package ships a prebuilt `.wasm` binary — no build step required.

## Usage

```javascript
import {
    parseAssetmapTyped,
    parseCplTyped,
    parsePklTyped,
    parseVolindexTyped,
    validate,
    getVersion,
} from '@imferno/wasm';

// Parse individual XML files
const assetMap = await parseAssetmapTyped(assetmapXml);
const cpl = await parseCplTyped(cplXml);
const pkl = await parsePklTyped(pklXml);
const volindex = await parseVolindexTyped(volindexXml);

// Validate a full IMF package (pass all XML files as a map)
const result = await validate({
    'ASSETMAP.xml': assetmapXml,
    'PKL_abc.xml': pklXml,
    'CPL_def.xml': cplXml,
});
console.log(result.report.errors);
console.log(result.report.warnings);
console.log(result.cpls);
console.log(result.unreferencedAssets);

// Validate with spec selection and custom rules
const result2 = await validate(
    {
        'ASSETMAP.xml': assetmapXml,
        'PKL_abc.xml': pklXml,
        'CPL_def.xml': cplXml,
    },
    { coreSpec: 'v2020', app2eSpec: 'v2023' },
);
```

WASM initialization is handled automatically on first call.

## API

| Function | Description |
|----------|-------------|
| `validate(files, options?)` | Validate a full IMF package, returns report + parsed data |
| `parseCplTyped(xml)` | Parse CPL XML |
| `parseAssetmapTyped(xml)` | Parse ASSETMAP.xml |
| `parsePklTyped(xml)` | Parse PKL XML |
| `parseVolindexTyped(xml)` | Parse VOLINDEX.xml |
| `getVersion()` | Get library version |

### Spec selection values

- `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"`
- `app2eSpec`: `"auto"` | `"none"` | `"v2020"` | `"v2021"` | `"v2023"`

## License

MIT
