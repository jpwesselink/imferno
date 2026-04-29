# @imferno/node

Native Node.js bindings for the [imferno](https://github.com/jpwesselink/imferno) SMPTE ST 2067 IMF validator, powered by [napi-rs](https://napi.rs).

Unlike [`@imferno/wasm`](https://www.npmjs.com/package/@imferno/wasm), this package has full filesystem access — validate packages by path and check MXF headers directly from disk.

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

```js
import { buildReportFromPath } from "@imferno/node";

const report = buildReportFromPath("./my-imp");

console.log("Compliant:", report.validation.isCompliant);
console.log("Errors:", report.validation.errors.length);
console.log("Warnings:", report.validation.warnings.length);
console.log("CPLs:", report.package.cplCount);
```

### Validate from a URI (filesystem or S3)

```js
import { validateUri, buildReportFromUri } from "@imferno/node";

// Local FS path or file:// URI
const result = validateUri("./my-imp");
const result2 = validateUri("file:///abs/path/to/my-imp");

// S3 — prebuilt npm binaries include S3 support; uses the default AWS
// credential chain (env vars, profile, or EC2/ECS/EKS IMDS)
const result3 = validateUri("s3://my-bucket/path/to/imp/");

// Same options object as validatePath
const report = buildReportFromUri("s3://my-bucket/path/to/imp/", {
  coreSpec: "v2020",
});
```

### Validate with options

```js
import { buildReportFromPath } from "@imferno/node";

const report = buildReportFromPath("./my-imp", {
  coreSpec: "v2020",
  app2eSpec: "v2023",
  skipDiskChecks: true,
  rules: {
    "ST2067-21:2023:7.1/AppIdMismatch": "error",
    "IMFERNO:Package/UnreferencedAsset": "off",
  },
});
```

### Validate from in-memory strings

```js
import { buildReport } from "@imferno/node";

const report = buildReport({
  "ASSETMAP.xml": assetmapXml,
  "PKL_abc.xml": pklXml,
  "CPL_def.xml": cplXml,
});
```

### Pretty-print a report

```js
import { buildReportFromPath, formatReport } from "@imferno/node";

const report = buildReportFromPath("./my-imp");
console.log(formatReport(report));
```

### Get the library version

```js
import { getVersion } from "@imferno/node";

console.log(getVersion()); // e.g. "2.0.0"
```

## API

| Function | Description |
|----------|-------------|
| `buildReport(files, options?)` | Validate from in-memory XML strings |
| `buildReportFromPath(path, options?)` | Validate an IMF package directory on disk |
| `formatReport(report)` | Pretty-print an `ImfReport` as a human-readable string |
| `getVersion()` | Get the library version |

### Options

| Option | Type | Description |
|--------|------|-------------|
| `coreSpec` | `"auto" \| "v2013" \| "v2016" \| "v2020"` | Core constraints spec version |
| `app2eSpec` | `"auto" \| "none" \| "v2020" \| "v2021" \| "v2023"` | Application spec version |
| `skipDiskChecks` | `boolean` | Skip file manifest and MXF checks (`buildReportFromPath` only) |
| `rules` | `Record<string, "error" \| "warn" \| "info" \| "off">` | ESLint-style per-rule severity overrides |

> **Note:** Hash verification (`verifyHashes`) is not currently exposed via NAPI.

### ImfReport shape

The returned object has this top-level structure (all keys are camelCase):

```json
{
  "package": {
    "assetMapId": "...",
    "volumeIndex": 1,
    "assetCount": 12,
    "cplCount": 1,
    "pklCount": 1,
    "issueDate": "2024-01-15T12:00:00Z",
    "issuer": "...",
    "creator": "...",
    "scmCount": 0,
    "sidecarCount": 0,
    "unreferencedAssets": []
  },
  "cpls": [
    {
      "id": "urn:uuid:...",
      "title": "My Composition",
      "applicationProfile": "App2E_2021",
      "editRate": "24000/1001",
      "segmentCount": 1,
      "isSupplemental": false,
      "sequences": [],
      "markers": []
    }
  ],
  "validation": {
    "isCompliant": true,
    "isPlayable": true,
    "profile": "SMPTE",
    "critical": [],
    "errors": [],
    "warnings": [],
    "info": [],
    "timestamp": "2024-01-15T12:00:00Z"
  }
}
```

## License

MIT
