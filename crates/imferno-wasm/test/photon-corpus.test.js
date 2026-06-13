/**
 * Photon corpus smoke validation under wasm.
 *
 * Walks `test-data/` (the vendored Photon-derived IMF corpus), gathers
 * every XML asset inside each package directory (ASSETMAP / PKL / CPL /
 * OPL / SCM / VOLINDEX), and feeds the file map straight into the wasm
 * `validate()` entry point. MXFs are deliberately skipped — those are
 * heavy binaries we don't need to fetch into the browser, and the
 * structural validators (parsers, XSD pre-pass, CPL/PKL/SCM checks)
 * fire on XML alone.
 *
 * Why this test exists: our 504-test `cargo test --workspace` runs on
 * native only. Before this file, the only wasm coverage was the 22
 * hand-rolled tests in `imferno.test.js` exercising one App #5 package.
 * That gap let a `std::env::temp_dir()` panic ship to `feat/full-smpte-
 * compliance` because no test exercised the XSD pre-pass on wasm.
 *
 * Test contract per package:
 *   - `validate(files, {})` must not throw `RuntimeError: unreachable`
 *     (i.e. no rust panic / no `unreachable!()` / no `temp_dir`-shaped
 *     crash). A graceful `Error: …` is OK — it just means the package
 *     legitimately failed to parse (e.g. BadXML directory).
 *   - When the call returns, the result must carry a `validation` key
 *     with a `summary` substructure.
 *
 * We don't assert on the FINDINGS — the corpus contains both
 * intentionally-broken samples and known-clean references; that's
 * separate per-fixture coverage.
 */

import { describe, it, expect, beforeAll } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'fs';
import { fileURLToPath } from 'url';
import path from 'path';

import wasmInit, * as wasm from '../imferno_wasm.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const TEST_DATA_ROOT = path.resolve(__dirname, '../../../test-data');

beforeAll(async () => {
    const wasmPath = path.join(__dirname, '..', 'imferno_wasm_bg.wasm');
    const wasmBuffer = readFileSync(wasmPath);
    await wasmInit(wasmBuffer);
});

/**
 * Recursively find every directory beneath `root` that contains at
 * least one `*.xml` file at its top level. Each such directory is one
 * "package" we feed to `validate()`.
 */
function findPackageDirs(root) {
    const out = [];
    const stack = [root];
    while (stack.length) {
        const dir = stack.pop();
        let entries;
        try {
            entries = readdirSync(dir, { withFileTypes: true });
        } catch {
            continue;
        }
        const hasXml = entries.some(
            e => e.isFile() && e.name.toLowerCase().endsWith('.xml'),
        );
        if (hasXml) {
            out.push(dir);
        }
        for (const e of entries) {
            if (e.isDirectory()) {
                stack.push(path.join(dir, e.name));
            }
        }
    }
    out.sort();
    return out;
}

/**
 * Read every `*.xml` at the top level of `dir` into a `{filename:
 * content}` map suitable for handing to the wasm `validate()` API.
 * MXFs and other binaries are deliberately skipped.
 */
function readPackageXmls(dir) {
    const files = {};
    for (const entry of readdirSync(dir)) {
        const full = path.join(dir, entry);
        try {
            if (!statSync(full).isFile()) continue;
        } catch {
            continue;
        }
        if (!entry.toLowerCase().endsWith('.xml')) continue;
        files[entry] = readFileSync(full, 'utf-8');
    }
    return files;
}

describe('Photon corpus — wasm validation smoke tests', () => {
    const packageDirs = findPackageDirs(TEST_DATA_ROOT);

    // Sanity: we expect to find at least a dozen packages once the
    // corpus is in place. If this drops to 0 the test file is doing
    // nothing useful and the contract should fail loudly.
    it('discovers at least 10 Photon packages with XML assets', () => {
        expect(packageDirs.length).toBeGreaterThanOrEqual(10);
    });

    for (const pkgDir of packageDirs) {
        const rel = path.relative(TEST_DATA_ROOT, pkgDir);
        it(`validates ${rel} without a wasm panic`, () => {
            const files = readPackageXmls(pkgDir);
            // No XMLs (deeply-nested-only) — nothing to do.
            if (Object.keys(files).length === 0) {
                return;
            }
            let result;
            try {
                result = wasm.validate(files, {});
            } catch (e) {
                // A graceful `Error: ...` from the wasm boundary is
                // acceptable (e.g. BadXML / MissingFilesAndAssetMapEntries
                // packages can't parse). A `RuntimeError: unreachable`
                // would be a Rust panic — that's the failure mode we
                // care about catching.
                const msg = String(e);
                expect(
                    msg.includes('unreachable'),
                    `wasm panicked on ${rel}: ${msg}`,
                ).toBe(false);
                return;
            }
            expect(result).toBeDefined();
            expect(result).toHaveProperty('validation');
            // Validation report shape: critical / errors / warnings /
            // info / suppressed arrays at the top level (see
            // ValidationReport in imferno-core).
            expect(Array.isArray(result.validation.critical)).toBe(true);
            expect(Array.isArray(result.validation.errors)).toBe(true);
            expect(Array.isArray(result.validation.warnings)).toBe(true);
        });
    }
});
