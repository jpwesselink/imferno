# @imferno/node

Native Node.js bindings for the [imferno](https://github.com/jpwesselink/imferno) SMPTE ST-2067 IMF validator, powered by [napi-rs](https://napi.rs).

Unlike [`imferno-wasm`](https://www.npmjs.com/package/imferno-wasm), this package has full filesystem access — validate packages by path and check MXF headers.

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
import { buildReportFromPath, formatReport } from '@imferno/node';

const report = buildReportFromPath('./my-imp');

console.log('Compliant:', report.validation.is_compliant);
console.log(formatReport(report));
```

### Validate with options

```javascript
import { buildReportFromPath, codes } from '@imferno/node';

const report = buildReportFromPath('./my-imp', {
  coreSpec: 'v2020',
  app2eSpec: 'v2023',
  rules: {
    [codes.ST2067_21_2023.AppIdMismatch]: 'error',
    [codes.Imferno.UnreferencedAsset]: 'off',
  },
});
```

### Validate from strings

```javascript
import { buildReport, formatReport } from '@imferno/node';

const report = buildReport({
  'ASSETMAP.xml': assetmapXml,
  'PKL_abc.xml': pklXml,
  'CPL_def.xml': cplXml,
});

console.log(formatReport(report));
```

## API

| Function | Description |
|----------|-------------|
| `buildReportFromPath(path, options?)` | Validate an IMF package directory on disk |
| `buildReport(files, options?)` | Validate from in-memory XML strings |
| `formatReport(report)` | Pretty-print an ImfReport as a human-readable string |
| `codes` | Typed validation code constants for use in `rules` config |
| `getVersion()` | Get library version |

### Options

- `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"`
- `app2eSpec`: `"auto"` | `"none"` | `"v2020"` | `"v2021"` | `"v2023"`
- `skipDiskChecks`: `boolean` — skip file manifest and MXF checks (buildReportFromPath only)
- `rules`: `Record<string, "error" | "warn" | "info" | "off">` — ESLint-style severity overrides

## License

MIT
