// Contract test: what the WASM bindings return must conform to the types
// declared in types.d.ts (generated from the Rust types and shared with
// @imferno/node). The JSON Schema is derived from the .d.ts itself, with
// closed objects, so a drift between declaration and serialisation fails here.
import { describe, it, expect, beforeAll } from 'vitest';
import { readFileSync, readdirSync } from 'fs';
import { fileURLToPath } from 'url';
import path from 'path';
import Ajv from 'ajv';
import { createGenerator } from 'ts-json-schema-generator';
import { execFileSync } from 'child_process';
import { createRequire } from 'module';
import ts from 'typescript';
import { inspectDeclarations } from '../../../scripts/dts-contract.mjs';
import * as api from '../index.js';

import wasmInit, * as wasm from '../imferno_wasm.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const testData = path.resolve(__dirname, '../../../test-data');

function findPackages(dir) {
    const out = [];
    for (const e of readdirSync(dir, { withFileTypes: true })) {
        const p = path.join(dir, e.name);
        if (e.isDirectory()) out.push(...findPackages(p));
        else if (/^assetmap/i.test(e.name) && !out.includes(dir)) out.push(dir);
    }
    return out;
}

function readXmlFiles(dir) {
    const files = {};
    for (const f of readdirSync(dir)) {
        if (f.toLowerCase().endsWith('.xml')) files[f] = readFileSync(path.join(dir, f), 'utf-8');
    }
    return files;
}

const packages = findPackages(testData).sort();
const validators = {};

function check(typeName, value, label) {
    const v = validators[typeName];
    if (!v(value)) {
        const errs = v.errors
            .slice(0, 10)
            .map((e) => `  ${e.instancePath || '/'} ${e.message} ${JSON.stringify(e.params)}`)
            .join('\n');
        throw new Error(`${label}: output violates declared type ${typeName}\n${errs}`);
    }
}

beforeAll(async () => {
    await wasmInit(readFileSync(path.join(__dirname, '..', 'imferno_wasm_bg.wasm')));
    const schema = createGenerator({
        path: path.join(__dirname, '..', 'types.d.ts'),
        type: '*',
        expose: 'export',
        additionalProperties: false,
        skipTypeCheck: true,
        topRef: true,
    }).createSchema('*');
    const ajv = new Ajv({ strict: false, allErrors: true });
    ajv.addSchema(schema, 'types');
    for (const t of ['ValidationResult', 'Imferno', 'ImfReport']) {
        validators[t] = ajv.getSchema(`types#/definitions/${t}`);
        if (!validators[t]) throw new Error(`type ${t} not found in types.d.ts`);
    }
}, 120_000);

describe('declarations', () => {
    const entry = path.join(__dirname, '..', 'index.d.ts');

    it('type-check (tsc, strict) including the type-level contract', () => {
        const tsc = createRequire(import.meta.url).resolve('typescript/bin/tsc');
        execFileSync(process.execPath, [tsc, '-p', path.join(__dirname, 'tsconfig.json')], { stdio: 'pipe' });
    }, 120_000);

    it('reach no `any` from any export (functions, properties, nested types)', () => {
        expect(inspectDeclarations(ts, entry).anyPaths).toEqual([]);
    });

    it('declare exactly the values index.js exports', () => {
        expect(inspectDeclarations(ts, entry).valueExports).toEqual(Object.keys(api).sort());
    });
});

describe('WASM outputs conform to types.d.ts', () => {
    it('finds the test-data packages', () => {
        expect(packages.length).toBeGreaterThan(20);
    });

    it.each(packages.map((p) => [p.slice(testData.length + 1), p]))('%s', (_name, dir) => {
        const files = readXmlFiles(dir);
        check('ValidationResult', wasm.validate(files, undefined), 'validate');
        check('ValidationResult', wasm.validate(files, { aggregateRepeats: true }), 'validate(aggregateRepeats)');
        let parsed;
        try {
            parsed = wasm.parsePackage(files);
        } catch {
            // Unparseable packages reject; validate() above still covered them.
        }
        if (parsed !== undefined) {
            check('Imferno', parsed, 'parsePackage');
            check('ImfReport', wasm.buildReport(files, undefined), 'buildReport');
        }
    });

    it('degenerate inputs still return a ValidationResult', () => {
        check('ValidationResult', wasm.validate({}, undefined), 'validate({})');
        check('ValidationResult', wasm.validate({ 'ASSETMAP.xml': '<not-xml' }, undefined), 'validate(garbage)');
    });
});
