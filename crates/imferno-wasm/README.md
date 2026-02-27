# imferno-wasm

SMPTE ST 2067 IMF parser and validator for JavaScript and TypeScript, powered by WebAssembly.

## Install

```bash
npm install imferno-wasm
```

The package ships a prebuilt `.wasm` binary — no build step required.

## Usage

```javascript
import {
    parseAssetmapTyped,
    parseCplTyped,
    parsePklTyped,
    parseVolindexTyped,
    validatePackage,
    validateCplWithSpecSelection,
    inspectPackage,
    extractSourceAsset,
    compareDelivery,
    getVersion,
} from 'imferno-wasm';

// Parse individual XML files
const assetMap = await parseAssetmapTyped(assetmapXml);
const cpl = await parseCplTyped(cplXml);
const pkl = await parsePklTyped(pklXml);
const volindex = await parseVolindexTyped(volindexXml);

// Validate a full IMF package (pass all XML files as a map)
const report = await validatePackage({
    'ASSETMAP.xml': assetmapXml,
    'PKL_abc.xml': pklXml,
    'CPL_def.xml': cplXml,
});
console.log(report.errors);
console.log(report.warnings);

// Validate a single CPL with spec selection
const cplReport = await validateCplWithSpecSelection(cplXml, 'v2020', 'v2023');

// Inspect package structure
const info = await inspectPackage({
    'ASSETMAP.xml': assetmapXml,
    'PKL_abc.xml': pklXml,
    'CPL_def.xml': cplXml,
});
console.log(info.cplCount);
console.log(info.unreferencedAssets);

// Extract source asset from a CPL
const sourceAsset = await extractSourceAsset(cplXml);

// Compare against a delivery spec
const comparison = await compareDelivery(sourceAsset, deliverySpec);
```

WASM initialization is handled automatically on first call.

## API

| Function | Input | Output |
|---|---|---|
| `parseAssetmapTyped(xml)` | ASSETMAP XML | `AssetMap` |
| `parseCplTyped(xml)` | CPL XML | `CompositionPlaylist` |
| `parsePklTyped(xml)` | PKL XML | `PackingList` |
| `parseVolindexTyped(xml)` | VOLINDEX XML | `VolumeIndex` |
| `validatePackage(files, rules?)` | `{ filename: xml }` map | `ValidationReport` |
| `validateCplWithSpecSelection(xml, core?, app?)` | CPL XML + spec pins | `ValidationReport` |
| `inspectPackage(files)` | `{ filename: xml }` map | package metadata |
| `extractSourceAsset(xml)` | CPL XML | `SourceAsset` |
| `compareDelivery(asset, spec)` | `SourceAsset` + `DeliveryRequest` | `DeliveryComparison` |
| `getVersion()` | — | version string |

### Spec selection values

- `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"`
- `app2eSpec`: `"auto"` | `"none"` | `"v2020"` | `"v2021"` | `"v2023"`

## License

MIT
