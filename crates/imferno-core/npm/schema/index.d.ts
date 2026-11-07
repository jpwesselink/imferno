/** A JSON Schema object (Draft 7). */
type JsonSchema = Record<string, unknown>;

/** `{ package, validation }` as returned by `validate()` in @imferno/node. */
export declare const validationResult: JsonSchema;
/** The full parsed package (`Imferno`) as returned by `parsePackage()` in @imferno/node. */
export declare const imfernoPackage: JsonSchema;
export declare const imfReport: JsonSchema;
export declare const validationReport: JsonSchema;
export declare const compositionPlaylist: JsonSchema;
export declare const assetMap: JsonSchema;
export declare const packingList: JsonSchema;
export declare const volumeIndex: JsonSchema;
export declare const rulesConfig: JsonSchema;

export declare const schemas: {
  validationResult: JsonSchema;
  imfernoPackage: JsonSchema;
  imfReport: JsonSchema;
  validationReport: JsonSchema;
  compositionPlaylist: JsonSchema;
  assetMap: JsonSchema;
  packingList: JsonSchema;
  volumeIndex: JsonSchema;
  rulesConfig: JsonSchema;
};
