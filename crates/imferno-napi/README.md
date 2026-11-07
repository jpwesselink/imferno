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

```ts
import { validatePath } from "@imferno/node";

const { package: pkg, validation } = validatePath("./my-imp");

console.log("Compliant:", validation.is_compliant);
console.log("Errors:", validation.errors.length);
for (const issue of validation.errors) {
  console.log(issue.severity, issue.code, issue.message, issue.location.file);
}
console.log("CPLs:", Object.keys(pkg.compositionPlaylists).length);
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

```ts
import { validatePath, type PathValidateOptions } from "@imferno/node";

const options: PathValidateOptions = {
  coreSpec: "v2020",
  app2eSpec: "v2023",
  skipDiskChecks: true,
  rules: {
    "ST2067-21:2023/AppIdMismatch": "error",
    "IMFERNO:Package/UnreferencedAsset": "off",
  },
};
const result = validatePath("./my-imp", options);
```

### Validate from in-memory strings

```js
import { validate } from "@imferno/node";

const result = validate({
  "ASSETMAP.xml": assetmapXml,
  "PKL_abc.xml": pklXml,
  "CPL_def.xml": cplXml,
});
```

### Pretty-print a report

```js
import { validatePath, formatReport } from "@imferno/node";

const result = validatePath("./my-imp");
console.log(formatReport(result)); // same text as `imferno validate`
```

### Get the library version

```js
import { getVersion } from "@imferno/node";

console.log(getVersion()); // e.g. "2.0.0"
```

## API

| Function | Description |
|----------|-------------|
| `validate(files, options?)` | Validate from in-memory XML strings (returns `ValidationResult`) |
| `validatePath(path, options?)` | Validate a package directory on disk (returns `ValidationResult`) |
| `validateUri(uri, options?)` | Validate via URI: `file://`, bare path, or `s3://` (returns `ValidationResult`) |
| `parsePackage(files)` | Parse without validation (returns full `Imferno` struct) |
| `parsePackageFromPath(path)` | Parse from disk without validation |
| `buildReport(files, options?)` | Legacy: validate from strings (returns `ImfReport`) |
| `buildReportFromPath(path, options?)` | Legacy: validate from disk (returns `ImfReport`) |
| `buildReportFromUri(uri, options?)` | Legacy: validate via URI (returns `ImfReport`) |
| `formatReport(result)` | Pretty-print a `ValidationResult` (or legacy `ImfReport`) as text |
| `listRules()` | Get the rule catalogue (one entry per rule, with its per-edition table) |
| `toSarif(result)` | Convert a validation result to a SARIF 2.1.0 log |
| `getVersion()` | Get the library version |

### Options

| Option | Type | Description |
|--------|------|-------------|
| `coreSpec` | `"auto" \| "v2013" \| "v2016" \| "v2020"` | Core constraints spec version |
| `app2eSpec` | `"auto" \| "none" \| "v2014" \| "v2016" \| "v2020" \| "v2023" \| "v2026"` | Application spec version |
| `rules` | `Record<string, "off" \| "info" \| "warn" \| "error" \| "critical">` | ESLint-style per-rule severity overrides |
| `aggregateRepeats` | `boolean` | Collapse repeated identical findings into one issue with `additional_instances` |
| `skipDiskChecks` | `boolean` | Skip file manifest and MXF checks (path / URI functions only; in-memory input never touches disk) |
| `credentials` | `S3Credentials \| null` | Explicit S3 credentials (URI functions only) |

### TypeScript

The package ships precise declarations, generated from the Rust types (no
`any`). Every function is synchronous. All value types are exported from the
main entry:

```ts
import type {
  ValidationResult,   // validate*, { package, validation }
  Imferno,            // parsePackage*, the full parsed package
  ImfReport,          // buildReport* (legacy)
  ValidationIssue,    // severity, category, code, message, location, ...
  ValidateOptions,    // coreSpec, app2eSpec, rules, aggregateRepeats
  PathValidateOptions,
  UriValidateOptions,
  RuleInfo,           // listRules() entries
} from "@imferno/node";
```

Field casing follows the serialised output: the validation report uses
snake_case (`is_compliant`, `cpl_id`), the parsed package uses camelCase at the
top level (`compositionPlaylists`) and the SMPTE element names inside CPLs
(`ContentTitle`, `SegmentList`).

### Return types

`validate()` / `validatePath()` / `validateUri()` return a `ValidationResult`:

```json
{
  "package": { /* full parsed Imferno struct */ },
  "validation": {
    "is_compliant": true,
    "is_playable": true,
    "profile": "SMPTE",
    "timestamp": "2026-01-01T12:00:00.000000+00:00",
    "critical": [],
    "errors": [],
    "warnings": [],
    "info": []
  }
}
```

`buildReport()` / `buildReportFromPath()` / `buildReportFromUri()` return a legacy `ImfReport` summary. For new code, prefer the `validate*` functions.

## License

MIT
