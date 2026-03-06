/**
 * Auto-initializing IMF Parser TypeScript Definitions
 *
 * All functions automatically handle WASM initialization.
 */

// Re-export all generated types from wasm-pack
export * from './imferno_wasm.d.ts';

/**
 * Build a structured report from an IMF package.
 * @param files Object mapping filenames to XML string content
 * @param options Optional validation options (spec selection, rules)
 * @returns Full IMF report with package info, CPLs, and validation results
 */
export function buildReport(
    files: Record<string, string>,
    options?: any,
): Promise<any>;

/**
 * Format an ImfReport as a human-readable string.
 * @param report The report returned by buildReport
 * @returns Formatted string representation
 */
export function formatReport(report: any): Promise<string>;

/** Get the library version */
export function getVersion(): Promise<string>;

/** Manual WASM initialization (for advanced use) */
export function init(wasmBuffer?: ArrayBuffer): Promise<void>;

/** Typed validation code constants for use in rules config */
export { codes } from './codes.js';

/** Raw WASM bindings (for advanced use) */
export declare const wasm: typeof import('./imferno_wasm.js');
