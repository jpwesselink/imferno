/**
 * Auto-initializing IMF Parser
 *
 * Automatically handles WASM initialization so developers
 * don't have to deal with init() functions or WASM buffers.
 */
import wasmInit, * as wasm from './imferno_wasm.js';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import path from 'path';

let initPromise = null;

function ensureInit() {
    if (!initPromise) {
        if (typeof process !== 'undefined' && process.versions && process.versions.node) {
            try {
                const __dirname = path.dirname(fileURLToPath(import.meta.url));
                const wasmPath = path.join(__dirname, 'imferno_wasm_bg.wasm');
                const wasmBuffer = readFileSync(wasmPath);
                initPromise = wasmInit(wasmBuffer);
            } catch (error) {
                console.error('Failed to load WASM in Node.js:', error);
                throw error;
            }
        } else {
            initPromise = wasmInit();
        }
    }
    return initPromise;
}

// Parsing
export async function parseAssetmapTyped(xmlContent) {
    await ensureInit();
    return wasm.parseAssetmapTyped(xmlContent);
}

export async function parseCplTyped(xmlContent) {
    await ensureInit();
    return wasm.parseCplTyped(xmlContent);
}

export async function parsePklTyped(xmlContent) {
    await ensureInit();
    return wasm.parsePklTyped(xmlContent);
}

export async function parseVolindexTyped(xmlContent) {
    await ensureInit();
    return wasm.parseVolindexTyped(xmlContent);
}

// Validation — the unified function
export async function validate(files, options) {
    await ensureInit();
    return wasm.validate(files, options);
}

// Utility
export async function getVersion() {
    await ensureInit();
    return wasm.getVersion();
}

// Typed validation codes
export { codes } from './codes.js';

// For users who want manual control
export { wasmInit as init };
export { wasm };
