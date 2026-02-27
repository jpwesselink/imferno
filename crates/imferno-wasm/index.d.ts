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

/**
 * Validate a CPL with configurable spec selection.
 * @param cplXml CPL XML content
 * @param coreSpec "auto" | "v2013" | "v2016" | "v2020"
 * @param app2eSpec "auto" | "none" | "v2020" | "v2021" | "v2023"
 * @returns ValidationReport
 */
export function validateCplWithSpecSelection(
    cplXml: string,
    coreSpec?: string,
    app2eSpec?: string,
): Promise<any>;

/**
 * Validate a full IMF package from an in-memory map of filename to XML content.
 * @param files Object mapping filenames to XML string content
 * @param rules Optional ESLint-style rules configuration
 * @returns ValidationReport
 */
export function validatePackage(
    files: Record<string, string>,
    rules?: any,
): Promise<any>;

/**
 * Inspect an IMF package and return structural metadata.
 * @param files Object mapping filenames to XML string content
 * @returns { cplCount, scmCount, declaredSidecars, unreferencedAssets }
 */
export function inspectPackage(
    files: Record<string, string>,
): Promise<any>;

/** Extract a SourceAsset from CPL XML */
export function extractSourceAsset(cplXml: string): Promise<any>;

/** Compare a SourceAsset against a delivery spec */
export function compareDelivery(sourceAssetJson: any, deliverySpecJson: any): Promise<any>;

/** Get the library version */
export function getVersion(): Promise<string>;

/** Manual WASM initialization (for advanced use) */
export function init(wasmBuffer?: ArrayBuffer): Promise<void>;

/** Raw WASM bindings (for advanced use) */
export declare const wasm: typeof import('./imferno_wasm.js');
