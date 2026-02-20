# imf-wasm

WebAssembly bindings for the `imf-rs` IMF parser. Exposes parsing, source-asset extraction, delivery comparison, and CPL validation to JavaScript and TypeScript.

## Build

```bash
cd imf-wasm

# Browser (ES module)
wasm-pack build --target web --out-dir pkg .

# Node.js (CommonJS)
wasm-pack build --target nodejs --out-dir pkg .
```

## Usage

```javascript
import init, {
    parseVolindexTyped,         // VOLINDEX.xml → typed VolumeIndex
    parseAssetmapTyped,         // ASSETMAP.xml → typed AssetMap
    parseCplTyped,              // CPL XML      → typed CompositionPlaylist
    extractSourceAsset,         // CPL XML      → SourceAsset
    compareDelivery,            // SourceAsset + DeliveryRequest → DeliveryComparison
    validateCpl,                // CPL XML      → ValidationResultV1 (auto spec)
    validateCplWithSpecSelection, // CPL XML    → ValidationResultV1 (pinned spec)
    getVersion,
} from './pkg/imf_wasm.js';

await init();

// Parse individual files
const assetMap = parseAssetmapTyped(assetmapXml);
const cpl = parseCplTyped(cplXml);

// Extract source asset (the main integration point)
const sourceAsset = extractSourceAsset(cplXml);
console.log(sourceAsset.videoQuality);      // "UHD" | "HD" | "SD"
console.log(sourceAsset.videoDynamicRange); // "SDR" | "HDR_10" | "HDR_DOLBY_VISION"
console.log(sourceAsset.audioType);         // "STEREO" | "DOLBY_DIGITAL_PLUS" | "DOLBY_ATMOS"

// Compare against a delivery spec
const comparison = compareDelivery(sourceAsset, {
    audioLanguages: ["en", "nl"],
    subtitleLanguages: ["nl"],
    captionLanguages: [],
    forcedNarrativeLanguages: [],
    audioType: "DOLBY_DIGITAL_PLUS",
    videoQuality: "UHD",
    videoDynamicRange: "HDR_DOLBY_VISION",
});
console.log(comparison.matches);
console.log(comparison.missingAudioLanguages);

// Validate a CPL (auto-detects spec from CPL xmlns + ApplicationIdentification)
const result = validateCpl(cplXml);
console.log(result.status);  // "Valid" | "ValidWithWarnings" | "Invalid" | "Error"
console.log(result.issues);

// Validate with pinned spec versions (mirrors CLI --core-spec / --app2e-spec)
const result2 = validateCplWithSpecSelection(cplXml, "v2020", "iab2021");
```

## API Reference

All functions are synchronous after `init()` resolves.

| Function | Input | Output |
|---|---|---|
| `parseVolindexTyped(xml)` | VOLINDEX XML string | `VolumeIndex` |
| `parseAssetmapTyped(xml)` | ASSETMAP XML string | `AssetMap` |
| `parseCplTyped(xml)` | CPL XML string | `CompositionPlaylist` |
| `extractSourceAsset(cplXml)` | CPL XML string | `SourceAsset` |
| `compareDelivery(asset, spec)` | `SourceAsset`, `DeliveryRequest` | `DeliveryComparison` |
| `validateCpl(cplXml)` | CPL XML string | `ValidationResultV1` |
| `validateCplWithSpecSelection(cplXml, coreSpec?, app2eSpec?)` | CPL XML, optional spec pins | `ValidationResultV1` |
| `getVersion()` | — | `string` |

### `validateCplWithSpecSelection` spec values

- `coreSpec`: `"auto"` (default) \| `"v2013"` \| `"v2016"` \| `"v2020"`
- `app2eSpec`: `"auto"` (default) \| `"none"` \| `"v2020"` \| `"v2021"` \| `"v2023"` \| `"iab2019"` \| `"iab2021"`

## TypeScript Types

Types are generated from Rust structs via `ts-rs`. Run from the workspace root:

```bash
cargo run -p st2067-3 --features typescript --bin generate_types
# Outputs .ts files to bindings/
```

## License

MIT
