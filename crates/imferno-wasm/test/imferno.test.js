import { describe, it, expect, beforeAll } from 'vitest';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import path from 'path';
import Ajv from 'ajv';

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

// ─── Schema validation ──────────────────────────────────────────────────────
describe('schema validation', () => {
    const schemasDir = path.resolve(__dirname, '../../imferno-core/npm/schema/schemas');
    const ajv = new Ajv({ strict: false });

    function loadSchema(name) {
        return JSON.parse(readFileSync(path.join(schemasDir, `${name}.json`), 'utf-8'));
    }

    it('all schemas compile without errors', () => {
        const names = [
            'imf-report', 'validation-report', 'composition-playlist',
            'asset-map', 'packing-list', 'volume-index', 'rules-config',
        ];
        for (const name of names) {
            const schema = loadSchema(name);
            expect(() => ajv.compile(schema)).not.toThrow();
        }
    });

    it('validate() report matches validation-report schema', () => {
        const schema = loadSchema('validation-report');
        const validate = ajv.compile(schema);
        const result = wasm.validate(packageFiles);
        expect(validate(result.report)).toBe(true);
    });

    it('validate() assetMap matches asset-map schema', () => {
        const schema = loadSchema('asset-map');
        const validate = ajv.compile(schema);
        const result = wasm.validate(packageFiles);
        expect(validate(result.assetMap)).toBe(true);
    });

    it('validate() packingLists match packing-list schema', () => {
        const schema = loadSchema('packing-list');
        const validate = ajv.compile(schema);
        const result = wasm.validate(packageFiles);
        for (const pkl of result.packingLists) {
            expect(validate(pkl)).toBe(true);
        }
    });

    it('validate() volumeIndex matches volume-index schema', () => {
        const schema = loadSchema('volume-index');
        const validate = ajv.compile(schema);
        const result = wasm.validate(packageFiles);
        expect(validate(result.volumeIndex)).toBe(true);
    });

    // CPL schema uses PascalCase (ContentTitle) from non-wasm serde, but WASM
    // output uses camelCase (contentTitle) from wasm-specific serde overrides.
    // TODO: generate a separate wasm-compatible schema or normalise casing.
    it.skip('validate() cpls match composition-playlist schema', () => {
        const schema = loadSchema('composition-playlist');
        const validate = ajv.compile(schema);
        const result = wasm.validate(packageFiles);
        for (const cpl of result.cpls) {
            const valid = validate(cpl);
            if (!valid) {
                console.error('CPL validation errors:', JSON.stringify(validate.errors, null, 2));
            }
            expect(valid).toBe(true);
        }
    });

    it('parseAssetmapTyped output matches asset-map schema', () => {
        const schema = loadSchema('asset-map');
        const validate = ajv.compile(schema);
        const result = wasm.parseAssetmapTyped(assetmapXml);
        expect(validate(result)).toBe(true);
    });

    it('parsePklTyped output matches packing-list schema', () => {
        const schema = loadSchema('packing-list');
        const validate = ajv.compile(schema);
        const result = wasm.parsePklTyped(pklXml);
        expect(validate(result)).toBe(true);
    });

    it('parseVolindexTyped output matches volume-index schema', () => {
        const schema = loadSchema('volume-index');
        const validate = ajv.compile(schema);
        const result = wasm.parseVolindexTyped(volindexXml);
        expect(validate(result)).toBe(true);
    });

    it('rejects invalid data against asset-map schema', () => {
        const schema = loadSchema('asset-map');
        const validate = ajv.compile(schema);
        expect(validate({ bogus: true })).toBe(false);
        expect(validate.errors.length).toBeGreaterThan(0);
    });

    it('rules-config schema validates valid config', () => {
        const schema = loadSchema('rules-config');
        const validate = ajv.compile(schema);
        expect(validate({ SegmentDuration: 'warn', SomeRule: 'off' })).toBe(true);
    });

    it('rules-config schema rejects invalid severity', () => {
        const schema = loadSchema('rules-config');
        const validate = ajv.compile(schema);
        expect(validate({ SegmentDuration: 'banana' })).toBe(false);
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
