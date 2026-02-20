/**
 * Auto-initializing IMF Parser
 *
 * This module automatically handles WASM initialization so developers
 * don't have to deal with init() functions or WASM buffers.
 */
import init, * as wasm from './pkg/imf_wasm.js';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import path from 'path';

// Auto-initialize WASM on first import
let initPromise = null;

function ensureInit() {
    if (!initPromise) {
        // Check if we're in Node.js environment
        if (typeof process !== 'undefined' && process.versions && process.versions.node) {
            // Node.js environment - load WASM file directly
            try {
                const __dirname = path.dirname(fileURLToPath(import.meta.url));
                const wasmPath = path.join(__dirname, 'pkg', 'imf_wasm_bg.wasm');
                const wasmBuffer = readFileSync(wasmPath);
                initPromise = init(wasmBuffer);
            } catch (error) {
                console.error('Failed to load WASM in Node.js:', error);
                throw error;
            }
        } else {
            // Browser environment - use default init
            initPromise = init();
        }
    }
    return initPromise;
}

// Wrap all parsing functions with auto-init
export async function parseAssetmapTyped(xmlContent) {
    await ensureInit();
    return wasm.parseAssetmapTyped(xmlContent);
}

export async function parseCplTyped(xmlContent) {
    await ensureInit();
    return wasm.parseCplTyped(xmlContent);
}

export async function parseVolindexTyped(xmlContent) {
    await ensureInit();
    return wasm.parseVolindexTyped(xmlContent);
}


export async function getVersion() {
    await ensureInit();
    return wasm.getVersion();
}

// For users who want manual control
export { init } from './pkg/imf_wasm.js';
export * as wasm from './pkg/imf_wasm.js';