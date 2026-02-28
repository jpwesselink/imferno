# @imferno/node

Native Node.js bindings for the [imferno](https://github.com/jpwesselink/imferno) SMPTE ST 2067 IMF validator, powered by [napi-rs](https://napi.rs).

Unlike [`@imferno/wasm`](https://www.npmjs.com/package/@imferno/wasm), this package has full filesystem access — validate packages by path, verify SHA hashes against PKL, and check MXF headers.

## Install

```bash
npm install @imferno/node
```

Prebuilt binaries are provided for:

| Platform | Architecture |
|----------|-------------|
| Linux    | x64, arm64  |
| macOS    | x64, arm64  |
| Windows  | x64, arm64  |

## Usage

### Validate a package on disk

```javascript
const { validatePath } = require('@imferno/node');

const { report, cpls, assetMap, packingLists, volumeIndex } = validatePath('./my-imp');

console.log('Compliant:', report.is_compliant);
console.log('Errors:', report.errors.length);
console.log('Warnings:', report.warnings.length);
```

### Validate with options

```javascript
const result = validatePath('./my-imp', {
  coreSpec: 'v2020',
  app2eSpec: 'v2023',
  verifyHashes: true,
  rules: {
    'ST2067-21:2023:7.1/AppIdMismatch': 'error',
    'IMFERNO:Package/UnreferencedAsset': 'off',
  },
});
```

### Validate from strings

```javascript
const { validate } = require('@imferno/node');

const result = validate({
  'ASSETMAP.xml': assetmapXml,
  'PKL_abc.xml': pklXml,
  'CPL_def.xml': cplXml,
});
```

### Parse individual files

```javascript
const { parseCpl, parseAssetmap, parsePkl, parseVolindex } = require('@imferno/node');

const cpl = parseCpl(cplXmlString);
const assetMap = parseAssetmap(assetmapXmlString);
const pkl = parsePkl(pklXmlString);
const volindex = parseVolindex(volindexXmlString);
```

## API

| Function | Description |
|----------|-------------|
| `validatePath(path, options?)` | Validate an IMF package directory on disk |
| `validate(files, options?)` | Validate from in-memory XML strings |
| `parseCpl(xml)` | Parse CPL XML |
| `parseAssetmap(xml)` | Parse ASSETMAP.xml |
| `parsePkl(xml)` | Parse PKL XML |
| `parseVolindex(xml)` | Parse VOLINDEX.xml |
| `getVersion()` | Get library version |

### Options

- `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"`
- `app2eSpec`: `"auto"` | `"none"` | `"v2020"` | `"v2021"` | `"v2023"`
- `verifyHashes`: `boolean` — verify SHA hashes against PKL (validatePath only)
- `skipDiskChecks`: `boolean` — skip file manifest and MXF checks (validatePath only)
- `rules`: `Record<string, "error" | "warn" | "info" | "off">` — ESLint-style severity overrides

## License

MIT
