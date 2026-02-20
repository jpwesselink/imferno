# IMF WASM Parser Examples

This directory contains examples showing how to use the IMF (Interoperable Master Format) WASM parser in both JavaScript and TypeScript.

## Quick Start

```bash
# Run the JavaScript example
node example.mjs

# For TypeScript (requires compilation)
npx tsc example.ts && node example.js
```

## Examples Overview

### `example.mjs` - JavaScript Example
A comprehensive JavaScript demonstration showing:
- ✅ **Working implementation** with current runtime behavior
- 🔧 Complete IMF package parsing (VOLINDEX, ASSETMAP)
- 📊 Asset analysis and SMPTE compliance validation
- 🏆 Production-ready code examples

### `example.ts` - TypeScript Example
Advanced TypeScript demonstration featuring:
- 🛡️ **Full type safety** with SMPTE interfaces
- 📝 IntelliSense support for all 27 generated types
- 🔧 Compile-time error prevention
- 📚 Complete API documentation

### `test-types.mjs` - Basic Test
Simple test file for quick validation of core functionality.

## Key Features Demonstrated

### SMPTE Specification Support
- **VolumeIndex**: Entry point XML parsing
- **AssetMap**: UUID-to-file mapping with metadata
- **CompositionPlaylist**: Complete CPL structure (planned)
- **Asset Analysis**: Packing lists, chunks, and volume indexing

### TypeScript Benefits
- 🎯 **27 Generated Interfaces**: Complete SMPTE type coverage
- 🔧 **IDE Support**: Full autocomplete and error detection
- 🛡️ **Type Safety**: Catch errors at compile time
- 📝 **camelCase Fields**: TypeScript-friendly naming conventions

### Runtime Behavior
- ⚡ **Fast**: Optimized WebAssembly parsing
- 🔒 **Safe**: Memory-safe Rust implementation
- 📋 **Complete**: Full SMPTE specification support
- 🌐 **Compatible**: Works in Node.js and browsers

## Usage Patterns

### Basic Parsing
```javascript
import init, { parseVolindexTyped } from './pkg/imf_wasm.js';

await init();
const volindex = parseVolindexTyped(xmlContent);
console.log('Index:', volindex.Index);
```

### TypeScript with Full Types
```typescript
import { parseAssetmapTyped } from './pkg/imf-types';
import type { AssetMap } from './pkg/imf-types';

const assetmap: AssetMap = parseAssetmapTyped(xmlContent);
// Full IntelliSense and type checking available
```

### Asset Analysis
```javascript
const assetmap = parseAssetmapTyped(assetmapXml);
assetmap.AssetList.Asset.forEach(asset => {
    console.log('Asset ID:', asset.Id);
    console.log('File path:', asset.ChunkList.Chunk[0].Path);
    console.log('Is packing list:', asset.PackingList || false);
});
```

## Notes

- **Runtime Data**: Currently uses PascalCase field names (matching SMPTE XML)
- **TypeScript Definitions**: Provide camelCase interfaces for better DX
- **SMPTE Compliance**: All parsing follows official SMPTE standards
- **Performance**: WebAssembly ensures fast, efficient parsing

## Next Steps

1. **Try the examples**: Run `node example.mjs` to see it in action
2. **Integrate TypeScript**: Use `./pkg/imf-types.d.ts` for type safety
3. **Explore APIs**: Check the 27 generated SMPTE interfaces
4. **Build applications**: Use the typed parsing functions in your projects