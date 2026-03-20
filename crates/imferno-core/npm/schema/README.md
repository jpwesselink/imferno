# @imferno/schema

JSON Schema definitions for SMPTE ST 2067 (IMF) types. Use these schemas to validate the JSON output from [`imferno`](https://www.npmjs.com/package/imferno), [`@imferno/wasm`](https://www.npmjs.com/package/@imferno/wasm), or [`@imferno/node`](https://www.npmjs.com/package/@imferno/node).

## Install

```bash
npm install @imferno/schema
```

## Usage

```js
import Ajv from "ajv";
import { imfReport } from "@imferno/schema";

const ajv = new Ajv();
const validate = ajv.compile(imfReport);

const data = JSON.parse(imfernoOutput);
if (validate(data)) {
  console.log("Valid IMF report");
} else {
  console.error(validate.errors);
}
```

## Available Schemas

| Export | Description |
|--------|-------------|
| `imfReport` | Full IMF package report (metadata + validation + CPL analysis) |
| `validationReport` | Validation issues and diagnostics |
| `compositionPlaylist` | SMPTE ST 2067-3 Composition Playlist |
| `assetMap` | SMPTE ST 2067-2 Asset Map |
| `packingList` | SMPTE ST 2067-2 Packing List |
| `volumeIndex` | SMPTE ST 429-9 Volume Index |
| `rulesConfig` | ESLint-style rules configuration for validation |

All schemas are also available via the `schemas` named export:

```js
import { schemas } from "@imferno/schema";
// schemas.imfReport, schemas.assetMap, etc.
```

## License

MIT
