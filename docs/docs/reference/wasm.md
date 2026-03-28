---
title: WASM API
description: "@imferno/wasm — WebAssembly bindings for JavaScript and TypeScript."
---

## Install

```bash
npm install @imferno/wasm
```

ESM module powered by WebAssembly. Use it in any browser or bundler. All functions are **async**.

---

## `validate`

Parse and validate an IMF package in one call. This is the **recommended entry point**.

Pass all XML files as a filename-to-string map.

```typescript
import { validate, formatReport } from '@imferno/wasm';

const result = await validate({
  'VOLINDEX.xml': volindexXml,
  'ASSETMAP.xml': assetmapXml,
  'PKL.xml': pklXml,
  'CPL.xml': cplXml,
});

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

### Options

```typescript
const result = await validate(files, {
  coreSpec: 'v2020',
  app2eSpec: 'v2023',
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
| `rules` | `Record<string, string>` | `{}` | ESLint-style severity overrides |

---

## `parsePackage`

Parse an IMF package **without running validation**. Returns the full `Imferno` struct.

```typescript
import { parsePackage } from '@imferno/wasm';

const pkg = await parsePackage({
  'VOLINDEX.xml': volindexXml,
  'ASSETMAP.xml': assetmapXml,
  'PKL.xml': pklXml,
  'CPL.xml': cplXml,
});

console.log(pkg.compositionPlaylists);
console.log(pkg.packingLists);
console.log(pkg.assetMap);
```

---

## `buildReport` (legacy)

Returns an `ImfReport` summary. Kept for backwards compatibility — prefer `validate()` instead.

```typescript
import { buildReport, formatReport } from '@imferno/wasm';

const report = await buildReport({
  'VOLINDEX.xml': volindexXml,
  'ASSETMAP.xml': assetmapXml,
  'PKL.xml': pklXml,
  'CPL.xml': cplXml,
});

console.log(formatReport(report));
```

Accepts the same options as `validate`.

---

## `formatReport`

Pretty-print a `ValidationResult` or `ImfReport` as a human-readable string. Same output as `imferno validate` on the CLI.

```typescript
import { validate, formatReport } from '@imferno/wasm';

const result = await validate(files);
console.log(formatReport(result));
```

---

## `codes`

Typed validation code constants for use in `rules` config. Provides autocomplete and typo protection.

```typescript
import { codes } from '@imferno/wasm';

codes.ST2067_2_2020.FileNotFound    // "ST2067-2:2020:8.3/FileNotFound"
codes.ST2067_21_2023.FrameRate      // "ST2067-21:2023:5.2/FrameRate"
codes.ST2067_201_2021.MCATagSymbolInvalid  // ...
```

See [Configuration](/guide/config/) for how to use `codes` with rules.

---

## `getVersion`

```typescript
import { getVersion } from '@imferno/wasm';
console.log(await getVersion()); // "2.0.0"
```

---

## `ValidationResult`

`validate()` returns this structure:

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

`buildReport()` returns this structure. For new code, prefer `ValidationResult` from `validate()`.

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
