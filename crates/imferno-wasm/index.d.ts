/**
 * Auto-initializing IMF Parser TypeScript Definitions
 *
 * All functions automatically handle WASM initialization.
 */

// Re-export the generated types
export * from './pkg/imf_wasm.js';

// Import the main types we use
import type { AssetMap, CompositionPlaylist, VolumeIndex, OutputProfileList, PackingList, Kdm } from './pkg/imf_wasm.js';

/**
 * Parse ASSETMAP.xml content
 * @param xmlContent The XML content as a string
 * @returns Promise resolving to typed AssetMap object
 */
export function parseAssetmapTyped(xmlContent: string): Promise<AssetMap>;

/**
 * Parse CPL.xml content
 * @param xmlContent The XML content as a string
 * @returns Promise resolving to typed CompositionPlaylist object
 */
export function parseCplTyped(xmlContent: string): Promise<CompositionPlaylist>;

/**
 * Parse VOLINDEX.xml content
 * @param xmlContent The XML content as a string
 * @returns Promise resolving to typed VolumeIndex object
 */
export function parseVolindexTyped(xmlContent: string): Promise<VolumeIndex>;


/**
 * Get the library version
 * @returns Promise resolving to version string
 */
export function getVersion(): Promise<string>;