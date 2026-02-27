import { describe, it, expect, beforeAll } from 'vitest';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import path from 'path';

import wasmInit, * as wasm from '../imferno_wasm.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const testData = path.resolve(__dirname, '../../../test-data');

function fixture(rel) {
    return readFileSync(path.join(testData, rel), 'utf-8');
}

beforeAll(async () => {
    const wasmPath = path.join(__dirname, '..', 'imferno_wasm_bg.wasm');
    const wasmBuffer = readFileSync(wasmPath);
    await wasmInit(wasmBuffer);
});

const app5 = 'Application5/PhotonApp5Test';
const volindexXml = fixture(`${app5}/VOLINDEX.xml`);
const assetmapXml = fixture(`${app5}/ASSETMAP.xml`);
const pklXml = fixture(`${app5}/PKL_f45a2034-317d-40fa-b07f-b9c5f3f15cfa.xml`);
const cplXml = fixture(`${app5}/CPL_cfad00b4-77b5-4d06-bd9d-48bc21c8fc0e.xml`);

const packageFiles = {
    'ASSETMAP.xml': assetmapXml,
    'PKL_f45a2034-317d-40fa-b07f-b9c5f3f15cfa.xml': pklXml,
    'CPL_cfad00b4-77b5-4d06-bd9d-48bc21c8fc0e.xml': cplXml,
    'VOLINDEX.xml': volindexXml,
};

// ─── getVersion ──────────────────────────────────────────────────────────────
describe('getVersion', () => {
    it('returns a semver string', () => {
        const version = wasm.getVersion();
        expect(version).toMatch(/^\d+\.\d+\.\d+/);
    });
});

// ─── parseVolindexTyped ──────────────────────────────────────────────────────
describe('parseVolindexTyped', () => {
    it('parses a valid VOLINDEX', () => {
        const result = wasm.parseVolindexTyped(volindexXml);
        expect(result).toBeDefined();
    });

    it('throws on invalid XML', () => {
        expect(() => wasm.parseVolindexTyped('<bad>')).toThrow();
    });
});

// ─── parseAssetmapTyped ──────────────────────────────────────────────────────
describe('parseAssetmapTyped', () => {
    it('parses a valid ASSETMAP and returns the id', () => {
        const result = wasm.parseAssetmapTyped(assetmapXml);
        expect(result).toBeDefined();
        expect(result.id).toBe('aa8669d7-6ebc-4839-9855-b5d7f9aa7f21');
    });

    it('returns asset_list with correct count', () => {
        const result = wasm.parseAssetmapTyped(assetmapXml);
        expect(result.asset_list.assets.length).toBe(5);
    });

    it('assets have id and chunk_list', () => {
        const result = wasm.parseAssetmapTyped(assetmapXml);
        const first = result.asset_list.assets[0];
        expect(first.id).toBeDefined();
        expect(first.chunk_list.chunks.length).toBeGreaterThan(0);
        expect(first.chunk_list.chunks[0].path).toBeDefined();
    });

    it('throws on invalid XML', () => {
        expect(() => wasm.parseAssetmapTyped('not xml')).toThrow();
    });
});

// ─── parsePklTyped ───────────────────────────────────────────────────────────
describe('parsePklTyped', () => {
    it('parses a valid PKL and returns the id', () => {
        const result = wasm.parsePklTyped(pklXml);
        expect(result).toBeDefined();
        expect(result.id).toBe('f45a2034-317d-40fa-b07f-b9c5f3f15cfa');
    });

    it('returns assets with hashes and sizes', () => {
        const result = wasm.parsePklTyped(pklXml);
        const assets = result.asset_list.assets;
        expect(assets.length).toBe(4);
        for (const asset of assets) {
            expect(asset.hash).toBeDefined();
            expect(asset.size).toBeDefined();
        }
    });

    it('throws on invalid XML', () => {
        expect(() => wasm.parsePklTyped('<oops/>')).toThrow();
    });
});

// ─── parseCplTyped ───────────────────────────────────────────────────────────
describe('parseCplTyped', () => {
    it('parses a valid CPL', () => {
        const result = wasm.parseCplTyped(cplXml);
        expect(result).toBeDefined();
    });

    it('throws on invalid XML', () => {
        expect(() => wasm.parseCplTyped('<nope/>')).toThrow();
    });
});

// ─── validateCplWithSpecSelection ────────────────────────────────────────────
describe('validateCplWithSpecSelection', () => {
    it('validates a CPL with auto spec detection', () => {
        const report = wasm.validateCplWithSpecSelection(cplXml);
        expect(report).toBeDefined();
        expect(report).toHaveProperty('errors');
        expect(report).toHaveProperty('warnings');
        expect(Array.isArray(report.errors)).toBe(true);
        expect(Array.isArray(report.warnings)).toBe(true);
    });

    it('validates with explicit spec versions', () => {
        const report = wasm.validateCplWithSpecSelection(cplXml, 'v2020', 'v2023');
        expect(report).toBeDefined();
        expect(report).toHaveProperty('errors');
    });

    it('returns error report for invalid XML', () => {
        const report = wasm.validateCplWithSpecSelection('<bad/>');
        expect(report).toBeDefined();
        expect(report.critical.length).toBeGreaterThan(0);
    });

    it('returns error report for invalid coreSpec value', () => {
        const report = wasm.validateCplWithSpecSelection(cplXml, 'v9999');
        expect(report).toBeDefined();
        expect(report.critical.length).toBeGreaterThan(0);
    });
});

// ─── validatePackage ─────────────────────────────────────────────────────────
describe('validatePackage', () => {
    it('validates a package from a file map', () => {
        const report = wasm.validatePackage(packageFiles);
        expect(report).toBeDefined();
        expect(report).toHaveProperty('errors');
        expect(report).toHaveProperty('warnings');
        expect(report).toHaveProperty('info');
    });

    it('returns errors for missing ASSETMAP', () => {
        const files = { 'random.xml': '<foo/>' };
        const report = wasm.validatePackage(files);
        expect(report).toBeDefined();
        expect(report.critical.length).toBeGreaterThan(0);
    });

    it('accepts optional rules config', () => {
        const report = wasm.validatePackage(packageFiles, {});
        expect(report).toBeDefined();
        expect(report).toHaveProperty('errors');
    });
});

// ─── inspectPackage ──────────────────────────────────────────────────────────
describe('inspectPackage', () => {
    it('returns an object without throwing', () => {
        const info = wasm.inspectPackage(packageFiles);
        expect(info).toBeDefined();
    });
});

// ─── extractSourceAsset ──────────────────────────────────────────────────────
describe('extractSourceAsset', () => {
    it('extracts a source asset from a CPL', () => {
        const result = wasm.extractSourceAsset(cplXml);
        expect(result).toBeDefined();
    });

    it('throws on invalid XML', () => {
        expect(() => wasm.extractSourceAsset('<bad/>')).toThrow();
    });
});

// ─── Auto-init wrapper ──────────────────────────────────────────────────────
describe('auto-init wrapper (index.js)', () => {
    let wrapper;

    beforeAll(async () => {
        wrapper = await import('../index.js');
    });

    it('parseAssetmapTyped works through wrapper', async () => {
        const result = await wrapper.parseAssetmapTyped(assetmapXml);
        expect(result.id).toBe('aa8669d7-6ebc-4839-9855-b5d7f9aa7f21');
    });

    it('parseCplTyped works through wrapper', async () => {
        const result = await wrapper.parseCplTyped(cplXml);
        expect(result).toBeDefined();
    });

    it('parsePklTyped works through wrapper', async () => {
        const result = await wrapper.parsePklTyped(pklXml);
        expect(result.id).toBe('f45a2034-317d-40fa-b07f-b9c5f3f15cfa');
    });

    it('parseVolindexTyped works through wrapper', async () => {
        const result = await wrapper.parseVolindexTyped(volindexXml);
        expect(result).toBeDefined();
    });

    it('getVersion works through wrapper', async () => {
        const version = await wrapper.getVersion();
        expect(version).toMatch(/^\d+\.\d+\.\d+/);
    });

    it('validatePackage works through wrapper', async () => {
        const report = await wrapper.validatePackage(packageFiles);
        expect(report).toHaveProperty('errors');
    });

    it('validateCplWithSpecSelection works through wrapper', async () => {
        const report = await wrapper.validateCplWithSpecSelection(cplXml);
        expect(report).toHaveProperty('errors');
    });

    it('extractSourceAsset works through wrapper', async () => {
        const result = await wrapper.extractSourceAsset(cplXml);
        expect(result).toBeDefined();
    });
});
