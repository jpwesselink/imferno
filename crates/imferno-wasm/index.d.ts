/**
 * Auto-initializing imferno WASM bindings.
 *
 * Every function initialises the WASM module on first use, so all of them
 * return Promises. Value types (`ValidationResult`, `Imferno`, `ImfReport`,
 * `ValidateOptions`, ...) live in ./types.d.ts, generated from the Rust types
 * by `cargo xtask generate-types-ts` and shared verbatim with @imferno/node.
 */

import type {
    ImfReport,
    Imferno,
    RuleInfo,
    ValidateOptions,
    ValidationReport,
    ValidationResult,
} from './types.js';

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

/** A SARIF 2.1.0 log (https://docs.oasis-open.org/sarif/sarif/v2.1.0/). */
export interface SarifLog {
    $schema?: string;
    version: "2.1.0";
    runs: Array<Record<string, unknown>>;
}

/**
 * Convert a validation result to a SARIF 2.1.0 log: rules by rule id
 * (`ST2067-3/SegmentDuration`), each finding pointing at the applied
 * edition's clause in that edition's taxonomy (`ST2067-3:2016`).
 * @param report The object `validate()` returned, or its `validation`.
 */
export function toSarif(report: ValidationResult | ValidationReport): Promise<SarifLog>;

/** The rule catalogue: one entry per rule, with its per-edition table. */
export function listRules(): Promise<RuleInfo[]>;

/** Get the library version */
export function getVersion(): Promise<string>;

/**
 * Initialise the WASM module explicitly. Optional: every function initialises
 * on first use. Pass the `.wasm` bytes to avoid fetching them.
 */
export function init(wasmBytes?: ArrayBuffer | ArrayBufferView): Promise<void>;

/** Rule ids (`SPEC/Name`) per spec, for use as rules-config keys */
export { codes } from './codes.js';
