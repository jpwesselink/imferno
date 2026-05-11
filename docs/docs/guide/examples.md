---
title: Examples
description: Working examples showing how to use imferno in your projects.
---

## Node.js — Validate an IMF package

This example validates a local IMF package using `@imferno/node`, configures custom rule overrides with typed code constants, and prints the results.

### Setup

```sh
npm install @imferno/node
```

### Code

```typescript
import { validatePath, codes } from "@imferno/node";
import { resolve } from "node:path";

const impPath = resolve("./my-package");

const result = validatePath(impPath, {
  rules: {
    // Promote checksum mismatches to critical
    [codes.ST2067_2_2020.ChecksumMismatch]: "critical",
    // Suppress unreferenced asset warnings
    [codes.Imferno.UnreferencedAsset]: "off",
  },
});

console.log("Compliant:", result.validation.is_compliant);
console.log("Errors:", result.validation.errors.length);
console.log("Warnings:", result.validation.warnings.length);

for (const issue of result.validation.errors) {
  console.log(`  ${issue.code}: ${issue.message}`);
}

// Access the full parsed package
console.log("CPLs:", Object.keys(result.package.compositionPlaylists).length);

process.exit(result.validation.is_compliant ? 0 : 1);
```

### What's happening

1. **`validatePath(path, options?)`** reads an IMF package directory from disk, parses all XML files, and runs validation against every applicable SMPTE spec.

2. **`codes`** gives you typed constants for all 250+ validation rules - autocomplete in your editor, no typos.

3. **`result.validation`** contains `is_compliant`, `errors`, `warnings`, `info`, and `critical` arrays. Each issue has a `code`, `message`, `severity`, and `location`.

4. **`result.package`** is the full parsed `Imferno` struct with composition playlists, packing lists, asset map, and all essence descriptors.

5. The process exits with code 1 if the package is non-compliant, making it easy to use in CI pipelines.

:::tip[Run the example locally]
The full working example is in the repository at [`examples/node/`](https://github.com/jpwesselink/imferno/tree/main/examples/node):

```sh
git clone https://github.com/jpwesselink/imferno.git
cd imferno/examples/node
npm install
npm run validate
```
:::

See the [Configuration](/guide/config/) page for all available options and the [Validation Codes](/guide/codes/) reference for the full rule catalogue.
