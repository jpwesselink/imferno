/* tslint:disable */
/* eslint-disable */

/**
 * Build a structured report from an IMF package.
 *
 * Pass all XML files as a plain JS object where each key is the filename
 * and each value is the file's text content.
 *
 * Returns an `ImfReport` containing package metadata, CPL analysis, and
 * validation results. This is the same JSON that the CLI `export` command produces.
 *
 * Options (all optional):
 * - `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"` (ST 2067-2; a
 *   pin also pins the editions it cites: ST 2067-3, ST 2067-201, ST 377-1/-4)
 * - `cplSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"` (ST 2067-3; wins
 *   over the edition `coreSpec` implies)
 * - `iabSpec`: `"auto"` | `"v2019"` | `"v2021"` | `"v2026"` (ST 2067-201; wins
 *   over the edition `coreSpec` implies)
 * - `app2eSpec`: `"auto"` | `"none"` | `"v2014"` | `"v2016"` | `"v2020"` | `"v2023"` | `"v2026"`
 * - `rules`: ESLint-style rules configuration object
 */
export function buildReport(files: any, options: any): any;

/**
 * Format a validation result as a human-readable string.
 *
 * Pass the object returned by `validate()`, or (legacy) by `buildReport()`.
 */
export function formatReport(report: any): string;

/**
 * Get library version
 */
export function getVersion(): string;

/**
 * Initialize the WASM module
 */
export function init(): void;

/**
 * The rule catalogue: one entry per rule with its per-edition table.
 */
export function listRules(): any;

/**
 * Parse an IMF package from in-memory files, returning the full parsed package.
 *
 * Unlike `buildReport` which returns a summary, this returns the complete
 * `Imferno` struct with all essence descriptors, locales, content versions, etc.
 */
export function parsePackage(files: any): any;

/**
 * Convert a validation result (or its `validation` report) to a SARIF 2.1.0 log.
 */
export function toSarif(report: any): any;

/**
 * Parse and validate an IMF package in one call.
 *
 * Returns `{ package, validation }` where `package` is the full `Imferno`
 * struct and `validation` is the `ValidationReport` with all findings.
 *
 * This is the recommended entry point.
 */
export function validate(files: any, options: any): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly buildReport: (a: any, b: any) => [number, number, number];
    readonly formatReport: (a: any) => [number, number, number, number];
    readonly getVersion: () => [number, number];
    readonly init: () => void;
    readonly listRules: () => [number, number, number];
    readonly parsePackage: (a: any) => [number, number, number];
    readonly toSarif: (a: any) => [number, number, number];
    readonly validate: (a: any, b: any) => [number, number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
