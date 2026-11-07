/**
 * Auto-initializing imferno WASM bindings.
 *
 * Every function initialises the WASM module on first use, so all of them
 * return Promises. Value types (`ValidationResult`, `Imferno`, `ImfReport`,
 * `ValidateOptions`, ...) live in ./types.d.ts, generated from the Rust types
 * by `cargo xtask generate-types-ts` and shared verbatim with @imferno/node.
 */

import type { ImfReport, Imferno, ValidateOptions, ValidationResult } from './types.js';

export * from './types.js';

/**
 * Parse and validate an IMF package. This is the recommended entry point.
 *
 * Parse failures are reported as findings in `validation`, not thrown.
 * @param files Object mapping filenames to XML string content
 * @param options Optional validation options (spec selection, rules)
 * @returns `{ package, validation }` — the full parsed package and all findings.
 */
export function validate(
    files: Record<string, string>,
    options?: ValidateOptions | null,
): Promise<ValidationResult>;

/**
 * Parse an IMF package without validating it.
 * @param files Object mapping filenames to XML string content
 * @returns The full parsed package. Rejects if the package cannot be parsed.
 */
export function parsePackage(files: Record<string, string>): Promise<Imferno>;

/**
 * Legacy: validate and return the lossy `ImfReport` summary. Prefer `validate`.
 * @param files Object mapping filenames to XML string content
 * @param options Optional validation options (spec selection, rules)
 * @returns Rejects if the package cannot be parsed.
 */
export function buildReport(
    files: Record<string, string>,
    options?: ValidateOptions | null,
): Promise<ImfReport>;

/**
 * Pretty-print a validation result as a human-readable string.
 * @param report The object `validate()` returned, or (legacy) `buildReport()`'s.
 */
export function formatReport(report: ValidationResult | ImfReport): Promise<string>;

/** Get the library version */
export function getVersion(): Promise<string>;

/**
 * Initialise the WASM module explicitly. Optional: every function initialises
 * on first use. Pass the `.wasm` bytes to avoid fetching them.
 */
export function init(wasmBytes?: ArrayBuffer | ArrayBufferView): Promise<void>;

/** Typed validation code constants for use in rules config */
export { codes } from './codes.js';
