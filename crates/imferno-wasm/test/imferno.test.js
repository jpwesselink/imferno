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

// ─── buildReport ─────────────────────────────────────────────────────────────
describe('buildReport', () => {
    it('builds a report from a valid package', () => {
        const result = wasm.buildReport(packageFiles);
        expect(result).toBeDefined();
        expect(result).toHaveProperty('package');
        expect(result).toHaveProperty('cpls');
        expect(result).toHaveProperty('validation');
    });

    it('package summary has expected fields', () => {
        const result = wasm.buildReport(packageFiles);
        const pkg = result.package;
        expect(pkg.assetMapId).toBe('aa8669d7-6ebc-4839-9855-b5d7f9aa7f21');
        expect(pkg.assetCount).toBe(5);
        expect(pkg.cplCount).toBeGreaterThan(0);
        expect(pkg.pklCount).toBeGreaterThan(0);
    });

    it('returns CPL reports with sequences', () => {
        const result = wasm.buildReport(packageFiles);
        expect(Array.isArray(result.cpls)).toBe(true);
        expect(result.cpls.length).toBeGreaterThan(0);
        const cpl = result.cpls[0];
        expect(cpl).toHaveProperty('id');
        expect(cpl).toHaveProperty('title');
        expect(cpl).toHaveProperty('segmentCount');
        expect(cpl).toHaveProperty('sequences');
        expect(Array.isArray(cpl.sequences)).toBe(true);
    });

    it('sequences have resources with durations', () => {
        const result = wasm.buildReport(packageFiles);
        const cpl = result.cpls[0];
        if (cpl.sequences.length > 0) {
            const seq = cpl.sequences[0];
            expect(seq).toHaveProperty('type');
            expect(seq).toHaveProperty('id');
            expect(seq).toHaveProperty('trackId');
            expect(Array.isArray(seq.resources)).toBe(true);
            if (seq.resources.length > 0) {
                const res = seq.resources[0];
                expect(res).toHaveProperty('id');
                expect(res).toHaveProperty('intrinsicDuration');
            }
        }
    });

    it('validation report has expected shape', () => {
        const result = wasm.buildReport(packageFiles);
        const v = result.validation;
        expect(v).toHaveProperty('errors');
        expect(v).toHaveProperty('warnings');
        expect(v).toHaveProperty('info');
        expect(Array.isArray(v.errors)).toBe(true);
    });

    it('throws on missing ASSETMAP', () => {
        const files = { 'random.xml': '<foo/>' };
        expect(() => wasm.buildReport(files)).toThrow();
    });

    it('accepts coreSpec option', () => {
        const result = wasm.buildReport(packageFiles, {
            coreSpec: 'v2020',
        });
        expect(result).toBeDefined();
        expect(result.validation).toHaveProperty('errors');
    });

    it('accepts app2eSpec option', () => {
        const result = wasm.buildReport(packageFiles, {
            app2eSpec: 'v2023',
        });
        expect(result).toBeDefined();
    });

    it('accepts rules option', () => {
        const result = wasm.buildReport(packageFiles, { rules: {} });
        expect(result).toBeDefined();
    });

    it('rejects invalid coreSpec', () => {
        expect(() => wasm.buildReport(packageFiles, { coreSpec: 'v9999' })).toThrow();
    });
});

// ─── formatReport ────────────────────────────────────────────────────────────
describe('formatReport', () => {
    it('formats a report as a string', () => {
        const report = wasm.buildReport(packageFiles);
        const formatted = wasm.formatReport(report);
        expect(typeof formatted).toBe('string');
        expect(formatted.length).toBeGreaterThan(0);
    });

    it('output contains validation info', () => {
        const report = wasm.buildReport(packageFiles);
        const formatted = wasm.formatReport(report);
        expect(formatted).toMatch(/ok|error|warning|valid/i);
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
            'imf-report', 'validation-report',
            'rules-config',
        ];
        for (const name of names) {
            const schema = loadSchema(name);
            expect(() => ajv.compile(schema)).not.toThrow();
        }
    });

    it('buildReport output matches imf-report schema', () => {
        const schema = loadSchema('imf-report');
        const validate = ajv.compile(schema);
        const result = wasm.buildReport(packageFiles);
        const valid = validate(result);
        if (!valid) {
            console.error('Schema validation errors:', JSON.stringify(validate.errors, null, 2));
        }
        expect(valid).toBe(true);
    });

    it('rejects invalid data against imf-report schema', () => {
        const schema = loadSchema('imf-report');
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

    it('buildReport works through wrapper', async () => {
        const result = await wrapper.buildReport(packageFiles);
        expect(result).toHaveProperty('package');
        expect(result).toHaveProperty('cpls');
        expect(result).toHaveProperty('validation');
        expect(result.package.assetMapId).toBe('aa8669d7-6ebc-4839-9855-b5d7f9aa7f21');
    });

    it('formatReport works through wrapper', async () => {
        const report = await wrapper.buildReport(packageFiles);
        const formatted = await wrapper.formatReport(report);
        expect(typeof formatted).toBe('string');
        expect(formatted.length).toBeGreaterThan(0);
    });

    it('getVersion works through wrapper', async () => {
        const version = await wrapper.getVersion();
        expect(version).toMatch(/^\d+\.\d+\.\d+/);
    });

    it('buildReport with options works through wrapper', async () => {
        const result = await wrapper.buildReport(packageFiles, { coreSpec: 'auto' });
        expect(result).toHaveProperty('package');
    });
});
