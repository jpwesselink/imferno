---
title: Node.js API
description: "@imferno/node — native Node.js bindings with filesystem access."
---

## Install

```bash
npm install @imferno/node
```

Native bindings via NAPI. Provides filesystem access for path-based validation. All functions are **synchronous**.

---

## `validate` / `validatePath`

Parse and validate an IMF package in one call. This is the **recommended entry point**.

`validatePath` reads XML files from disk. `validate` takes a filename-to-string map.

```javascript
import { validatePath, formatReport } from '@imferno/node';

const result = validatePath('./my-imp');

// Full parsed package
console.log(result.package.compositionPlaylists);

// Validation results
if (!result.validation.is_compliant) {
  for (const err of result.validation.errors) {
    console.error(err.code, err.message);
  }
}

// Pretty-print the validation report
console.log(formatReport(result));
```

From string content (no filesystem access):

```javascript
import { validate } from '@imferno/node';

const result = validate({
  'VOLINDEX.xml': volindexXml,
  'ASSETMAP.xml': assetmapXml,
  'PKL.xml': pklXml,
  'CPL.xml': cplXml,
});
```

### Options

```javascript
const result = validatePath('./my-imp', {
  coreSpec: 'v2020',
  app2eSpec: 'v2023',
  skipDiskChecks: false,
  rules: {
    'ST2067-21:2023:7.1/AppIdMismatch': 'error',
    'IMFERNO:Package/UnreferencedAsset': 'off',
  },
});
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `coreSpec` | `"auto" \| "v2013" \| "v2016" \| "v2020"` | `"auto"` | Core constraints spec version |
| `app2eSpec` | `"auto" \| "none" \| "v2020" \| "v2021" \| "v2023"` | `"auto"` | Application profile version |
| `skipDiskChecks` | `boolean` | `false` | Skip file existence/size and MXF header checks |
| `rules` | `Record<string, string>` | `{}` | ESLint-style severity overrides |

---

## `parsePackage` / `parsePackageFromPath`

Parse an IMF package **without running validation**. Returns the full `Imferno` struct.

```javascript
import { parsePackageFromPath } from '@imferno/node';

const pkg = parsePackageFromPath('./my-imp');
console.log(pkg.compositionPlaylists);
console.log(pkg.packingLists);
console.log(pkg.assetMap);
```

From string content:

```javascript
import { parsePackage } from '@imferno/node';

const pkg = parsePackage({
  'VOLINDEX.xml': volindexXml,
  'ASSETMAP.xml': assetmapXml,
  'PKL.xml': pklXml,
  'CPL.xml': cplXml,
});
```

---

## `buildReport` / `buildReportFromPath` (legacy)

Returns an `ImfReport` summary. Kept for backwards compatibility — prefer `validate()` / `validatePath()` instead.

```javascript
import { buildReportFromPath, formatReport } from '@imferno/node';

const report = buildReportFromPath('./my-imp');
console.log(formatReport(report));
```

From string content:

```javascript
import { buildReport } from '@imferno/node';

const report = buildReport({
  'ASSETMAP.xml': assetmapXml,
  'PKL.xml': pklXml,
  'CPL.xml': cplXml,
});
```

Accepts the same options as `validatePath` / `validate`.

---

## `formatReport`

Pretty-print a `ValidationResult` or `ImfReport` as a human-readable string. Same output as `imferno validate` on the CLI.

```javascript
import { validatePath, formatReport } from '@imferno/node';

const result = validatePath('./my-imp');
console.log(formatReport(result));
```

---

## `getVersion`

```javascript
import { getVersion } from '@imferno/node';
console.log(getVersion()); // "2.0.0"
```

---

## `codes`

Typed validation code constants for use in `rules` config. Provides autocomplete and typo protection.

```javascript
import { codes } from '@imferno/node';

codes.ST2067_2_2020.FileNotFound    // "ST2067-2:2020:8.3/FileNotFound"
codes.ST2067_21_2023.FrameRate      // "ST2067-21:2023:5.2/FrameRate"
codes.ST2067_201_2021.MCATagSymbolInvalid  // ...
```

See [Configuration](/guide/config/) for how to use `codes` with rules.

---

## `ValidationResult`

`validate()` and `validatePath()` return this structure:

```typescript
interface ValidationResult {
  package: Imferno;      // full parsed package
  validation: {          // ValidationReport
    critical: ValidationIssue[];
    errors: ValidationIssue[];
    warnings: ValidationIssue[];
    info: ValidationIssue[];
    is_playable: boolean;
    is_compliant: boolean;
    profile: string;
    timestamp: string;
  };
}
```

The `Imferno` type contains `compositionPlaylists`, `packingLists`, `assetMap`, and other parsed IMF package data.

---

## `ImfReport` (legacy)

`buildReport()` and `buildReportFromPath()` return this structure. For new code, prefer `ValidationResult` from `validate()`.

```typescript
interface ImfReport {
  package: {
    assetMapId: string;
    volumeIndex: number;
    assetCount: number;
    cplCount: number;
    issueDate: string;
    issuer: string | null;
    creator: string | null;
    pklCount: number;
    scmCount: number;
    sidecarCount: number;
    unreferencedAssets: { id: string; path: string }[];
  };
  cpls: {
    id: string;
    title: string;
    editRate: string;
    sequences: string[];
    applicationProfile: string | null;
    segmentCount: number;
    timecodeStart: string | null;
    isSupplemental: boolean;
    unresolvedAncestorAssetIds: string[];
    markers: { label: string; offset: number; annotation: string | null }[];
  }[];
  validation: {
    critical: ValidationIssue[];
    errors: ValidationIssue[];
    warnings: ValidationIssue[];
    info: ValidationIssue[];
    is_playable: boolean;
    is_compliant: boolean;
    profile: string;
    timestamp: string;
  };
}
```
