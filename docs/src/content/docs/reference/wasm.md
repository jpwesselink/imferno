---
title: WASM API
description: "@imferno/wasm — WebAssembly bindings for JavaScript and TypeScript."
tableOfContents: true
---

## Install

```bash
npm install @imferno/wasm
```

ESM module powered by WebAssembly. Use it in any browser or bundler.

---

## `buildReport`

Build a structured report from an IMF package. Pass all XML files as a filename to string map.

```typescript
import { buildReport, formatReport } from '@imferno/wasm';

const report = buildReport({
  'VOLINDEX.xml': volindexXml,
  'ASSETMAP.xml': assetmapXml,
  'PKL.xml': pklXml,
  'CPL.xml': cplXml,
});

// Pretty-print
console.log(formatReport(report));

// Check programmatically
if (!report.validation.isCompliant) {
  for (const err of report.validation.errors) {
    console.error(err.code, err.message);
  }
}
```

### Options

```typescript
const report = buildReport(files, {
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

## `formatReport`

Pretty-print an `ImfReport` as a human-readable string. Same output as `imferno report` on the CLI.

```typescript
import { buildReport, formatReport } from '@imferno/wasm';

const report = buildReport(files);
console.log(formatReport(report));
```

---

## `codes`

Typed validation code constants for use in `rules` config. Provides autocomplete and typo protection.

```typescript
import { codes } from '@imferno/wasm';

codes.ST2067_2_2020.FileNotFound    // "ST2067-2:2020:8.3/FileNotFound"
codes.ST2067_21_2023.FrameRate      // "ST2067-21:2023:5.2/FrameRate"
codes.ST2067_201_2021.SoundfieldGroupLinkId  // ...
```

See [Configuration](/guide/config/) for how to use `codes` with rules.

---

## `getVersion`

```typescript
import { getVersion } from '@imferno/wasm';
console.log(getVersion()); // "1.1.0"
```

---

## `ImfReport`

`buildReport` returns the same structure as the Node.js API and CLI `export` command:

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
    isPlayable: boolean;
    isCompliant: boolean;
    profile: string;
    timestamp: string;
  };
}
```
