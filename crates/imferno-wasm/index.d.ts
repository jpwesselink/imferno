/**
 * Auto-initializing IMF Parser TypeScript Definitions
 *
 * All functions automatically handle WASM initialization.
 */

/** Parse ASSETMAP.xml content */
export function parseAssetmapTyped(xmlContent: string): Promise<any>;

/** Parse CPL XML content */
export function parseCplTyped(xmlContent: string): Promise<any>;

/** Parse PKL XML content */
export function parsePklTyped(xmlContent: string): Promise<any>;

/** Parse VOLINDEX.xml content */
export function parseVolindexTyped(xmlContent: string): Promise<any>;

/** Options for the validate function */
export interface ValidateOptions {
    /** Core constraints spec version: "auto" | "v2013" | "v2016" | "v2020" */
    coreSpec?: "auto" | "v2013" | "v2016" | "v2020";
    /** Application profile version: "auto" | "none" | "v2020" | "v2021" | "v2023" */
    app2eSpec?: "auto" | "none" | "v2020" | "v2021" | "v2023";
    /** ESLint-style rules configuration */
    rules?: Record<string, string>;
}

/** Result returned by the validate function */
export interface ValidateResult {
    /** Validation report with issues, compliance status, and profile */
    report: any;
    /** Parsed Composition Playlists */
    cpls: any[];
    /** Parsed AssetMap (null if parsing failed) */
    assetMap: any | null;
    /** Parsed Packing Lists */
    packingLists: any[];
    /** Parsed Volume Index (null if parsing failed) */
    volumeIndex: any | null;
    /** Assets in the AssetMap with no CPL or SCM reference */
    unreferencedAssets: { id: string; path: string }[];
    /** Sidecar assets declared in SCMs */
    declaredSidecars: { id: string; cplIds: string[] }[];
}

/**
 * Validate a full IMF package and return both validation report and parsed data.
 * @param files Object mapping filenames to XML string content
 * @param options Optional validation options (spec selection, rules)
 * @returns Validation report + parsed package data
 */
export function validate(
    files: Record<string, string>,
    options?: ValidateOptions,
): Promise<ValidateResult>;

/** Get the library version */
export function getVersion(): Promise<string>;

/** Manual WASM initialization (for advanced use) */
export function init(wasmBuffer?: ArrayBuffer): Promise<void>;

/** Typed validation code constants for use in rules config */
export { codes } from './codes.js';

/** Raw WASM bindings (for advanced use) */
export declare const wasm: typeof import('./imferno_wasm.js');
