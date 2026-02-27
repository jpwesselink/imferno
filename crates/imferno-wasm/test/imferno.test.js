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

// ─── validate ────────────────────────────────────────────────────────────────
describe('validate', () => {
    it('validates a package and returns report + parsed data', () => {
        const result = wasm.validate(packageFiles);
        expect(result).toBeDefined();
        expect(result).toHaveProperty('report');
        expect(result).toHaveProperty('cpls');
        expect(result).toHaveProperty('assetMap');
        expect(result).toHaveProperty('packingLists');
        expect(result).toHaveProperty('volumeIndex');
        expect(result).toHaveProperty('unreferencedAssets');
        expect(result).toHaveProperty('declaredSidecars');
    });

    it('report has expected shape', () => {
        const result = wasm.validate(packageFiles);
        expect(result.report).toHaveProperty('errors');
        expect(result.report).toHaveProperty('warnings');
        expect(result.report).toHaveProperty('info');
        expect(Array.isArray(result.report.errors)).toBe(true);
    });

    it('returns parsed CPLs', () => {
        const result = wasm.validate(packageFiles);
        expect(Array.isArray(result.cpls)).toBe(true);
        expect(result.cpls.length).toBeGreaterThan(0);
    });

    it('returns parsed asset map', () => {
        const result = wasm.validate(packageFiles);
        expect(result.assetMap).toBeDefined();
        expect(result.assetMap.id).toBe('aa8669d7-6ebc-4839-9855-b5d7f9aa7f21');
    });

    it('returns errors for missing ASSETMAP', () => {
        const files = { 'random.xml': '<foo/>' };
        const result = wasm.validate(files);
        expect(result).toBeDefined();
        expect(result.report.critical.length).toBeGreaterThan(0);
        expect(result.assetMap).toBeFalsy();
    });

    it('accepts spec selection options', () => {
        const result = wasm.validate(packageFiles, {
            coreSpec: 'v2020',
            app2eSpec: 'v2023',
        });
        expect(result).toBeDefined();
        expect(result.report).toHaveProperty('errors');
    });

    it('accepts rules option', () => {
        const result = wasm.validate(packageFiles, { rules: {} });
        expect(result).toBeDefined();
        expect(result.report).toHaveProperty('errors');
    });

    it('rejects invalid coreSpec', () => {
        expect(() => wasm.validate(packageFiles, { coreSpec: 'v9999' })).toThrow();
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

    it('validate works through wrapper', async () => {
        const result = await wrapper.validate(packageFiles);
        expect(result).toHaveProperty('report');
        expect(result).toHaveProperty('cpls');
        expect(result.report).toHaveProperty('errors');
    });

    it('validate with options works through wrapper', async () => {
        const result = await wrapper.validate(packageFiles, { coreSpec: 'auto' });
        expect(result).toHaveProperty('report');
    });
});
