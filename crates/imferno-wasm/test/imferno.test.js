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

    // MERIDIAN: 2013 namespaces, App #2E 2016 ApplicationIdentification.
    const meridian = 'MERIDIAN_Netflix_Photon_161006';
    const meridianFiles = Object.fromEntries(
        [
            'ASSETMAP.xml',
            'VOLINDEX.xml',
            'PKL_f5e93462-aed2-44ad-a4ba-2adb65823e7c.xml',
            'CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml',
            'OPL_8cf83c32-4949-4f00-b081-01e12b18932f.xml',
        ].map((f) => [f, fixture(`${meridian}/${f}`)]),
    );
    const findings = (r) =>
        ['critical', 'errors', 'warnings', 'info'].flatMap((k) => r.validation[k] ?? []);

    it('coreSpec pins the ST 2067-3 edition it cites unless cplSpec pins it', () => {
        const pinned = findings(wasm.buildReport(meridianFiles, { coreSpec: 'v2016' }));
        expect(pinned.some((i) => i.rule === 'IMFERNO/EditionConflict')).toBe(false);
        const cpl = pinned.filter((i) => i.spec === 'ST2067-3');
        expect(cpl.length).toBeGreaterThan(0);
        for (const i of cpl) expect(i.edition, i.code).toBe('2016');
        const both = findings(
            wasm.buildReport(meridianFiles, { coreSpec: 'v2016', cplSpec: 'v2013' }),
        );
        for (const i of both.filter((i) => i.spec === 'ST2067-3')) {
            expect(i.edition, i.code).toBe('2013');
        }
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

// ─── SARIF + rule catalogue ──────────────────────────────────────────────────
describe('toSarif', () => {
    it('returns a SARIF 2.1.0 log with one result per finding', () => {
        const result = wasm.validate(packageFiles);
        const log = wasm.toSarif(result);
        expect(log.version).toBe('2.1.0');
        const v = result.validation;
        const findings = ['critical', 'errors', 'warnings', 'info', 'suppressed']
            .reduce((n, k) => n + (v[k]?.length ?? 0), 0);
        expect(log.runs[0].results.length).toBe(findings);
        for (const r of log.runs[0].results) {
            expect(r.ruleId).toMatch(/^[A-Z0-9-]+\/[A-Za-z0-9]+$/);
        }
    });

    it('reports unknown rules keys as rule_warnings', () => {
        const result = wasm.validate(packageFiles, { rules: { BogusKey: 'off' } });
        expect(result.rule_warnings).toEqual([
            { key: 'BogusKey', reason: { kind: 'matches_nothing' } },
        ]);
    });

    it('rejects a non-report', () => {
        expect(() => wasm.toSarif({ nope: true })).toThrow();
    });
});

describe('listRules', () => {
    it('returns every rule with its edition table', () => {
        const rules = wasm.listRules();
        expect(rules.length).toBeGreaterThan(100);
        const seg = rules.find((r) => r.id === 'ST2067-3/SegmentDuration');
        expect(seg.editions.map((e) => e.edition)).toEqual(['2013', '2016', '2020']);
        expect(seg.editions[1].code).toBe('ST2067-3:2016:7.2/SegmentDuration');
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
