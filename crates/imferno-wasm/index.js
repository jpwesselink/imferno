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

// Validation
export async function validateCplWithSpecSelection(cplXml, coreSpec, app2eSpec) {
    await ensureInit();
    return wasm.validateCplWithSpecSelection(cplXml, coreSpec, app2eSpec);
}

export async function validatePackage(files, rules) {
    await ensureInit();
    return wasm.validatePackage(files, rules);
}

// Inspection
export async function inspectPackage(files) {
    await ensureInit();
    return wasm.inspectPackage(files);
}

// Source asset / delivery
export async function extractSourceAsset(cplXml) {
    await ensureInit();
    return wasm.extractSourceAsset(cplXml);
}

export async function compareDelivery(sourceAssetJson, deliverySpecJson) {
    await ensureInit();
    return wasm.compareDelivery(sourceAssetJson, deliverySpecJson);
}

// Utility
export async function getVersion() {
    await ensureInit();
    return wasm.getVersion();
}

// For users who want manual control
export { wasmInit as init };
export { wasm };
