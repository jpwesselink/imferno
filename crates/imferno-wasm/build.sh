#!/bin/bash
set -e

echo "🧹 Cleaning previous builds..."
rm -rf pkg target ../target

echo "🚀 Building WASM module..."
RUSTFLAGS="--cfg getrandom_js" wasm-pack build --target web

echo "📝 Generating TypeScript definitions..."
cd ..
cargo run -p st2067-3 --bin generate_types --features typescript
cd imf-wasm

echo "📋 Copying TypeScript definitions to pkg..."
cp ../bindings/*.ts pkg/

echo "✅ Build complete!"
echo ""
echo "📦 Generated files:"
echo "   - pkg/imf_wasm.js (JavaScript bindings)"
echo "   - pkg/imf_wasm_bg.wasm (WebAssembly module)"
echo "   - pkg/imf_wasm.d.ts (Basic TypeScript definitions)"
echo "   - pkg/*.ts (27 SMPTE interface definitions)"
echo ""
echo "🧪 To test:"
echo "   node example.mjs         # JavaScript example"
echo "   node check-language.js   # TypeScript language checker"