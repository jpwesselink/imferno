#!/bin/bash
set -e

echo "Building WASM module..."
RUSTFLAGS="--cfg getrandom_js" wasm-pack build --target web

echo "Build complete!"
