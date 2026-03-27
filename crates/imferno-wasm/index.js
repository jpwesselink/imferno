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

// Build a structured report from an IMF package
export async function buildReport(files, options) {
    await ensureInit();
    return wasm.buildReport(files, options);
}

// Format an ImfReport as a human-readable string
export async function formatReport(report) {
    await ensureInit();
    return wasm.formatReport(report);
}

// Parse an IMF package, returning the full Imferno struct
export async function parsePackage(files) {
    await ensureInit();
    return wasm.parsePackage(files);
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
