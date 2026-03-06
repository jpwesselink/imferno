---
title: Node.js API
description: "@imferno/node — native Node.js bindings with filesystem access."
tableOfContents: true
---

## Install

```bash
npm install @imferno/node
```

Native bindings via NAPI. Provides filesystem access for path-based validation.

---

## `buildReportFromPath`

Build a structured report from an IMF package on disk.

```javascript
import { buildReportFromPath, formatReport } from '@imferno/node';

const report = buildReportFromPath('./my-imp');

// Pretty-print
console.log(formatReport(report));

// Check programmatically
if (!report.validation.is_compliant) {
  for (const err of report.validation.errors) {
    console.error(err.code, err.message);
  }
}

// Store in a database
await db.insert('validations', report);
```

### Options

```javascript
const report = buildReportFromPath('./my-imp', {
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

## `buildReport`

Same as `buildReportFromPath`, but takes XML file contents as a string map instead of a path. No filesystem access.

```javascript
import { buildReport, formatReport } from '@imferno/node';

const report = buildReport({
  'VOLINDEX.xml': volindexXml,
  'ASSETMAP.xml': assetmapXml,
  'PKL.xml': pklXml,
  'CPL.xml': cplXml,
});

console.log(formatReport(report));
```

Accepts the same options as `buildReportFromPath` (except `skipDiskChecks`).

---

## `formatReport`

Pretty-print an `ImfReport` as a human-readable string. Same output as `imferno report` on the CLI.

```javascript
import { buildReportFromPath, formatReport } from '@imferno/node';

const report = buildReportFromPath('./my-imp');
console.log(formatReport(report));
```

---

## `getVersion`

```javascript
import { getVersion } from '@imferno/node';
console.log(getVersion()); // "1.1.0"
```

---

## `ImfReport`

Both `buildReport` and `buildReportFromPath` return the same structure:

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
    is_playable: boolean;
    is_compliant: boolean;
    profile: string;
    timestamp: string;
  };
}
```
