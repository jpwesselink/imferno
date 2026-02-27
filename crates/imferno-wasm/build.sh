#!/bin/bash
set -e

echo "Building WASM module..."
RUSTFLAGS="--cfg getrandom_js" wasm-pack build --target web

echo "Copying WASM artifacts to crate root..."
cp pkg/imferno_wasm.js .
cp pkg/imferno_wasm.d.ts .
cp pkg/imferno_wasm_bg.wasm .
cp pkg/imferno_wasm_bg.wasm.d.ts .

echo "Build complete!"
