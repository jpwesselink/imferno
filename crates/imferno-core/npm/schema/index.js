import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const schemasDir = join(__dirname, "schemas");

function loadSchema(name) {
  return JSON.parse(readFileSync(join(schemasDir, `${name}.json`), "utf-8"));
}

export const validationResult = loadSchema("validation-result");
export const imfernoPackage = loadSchema("imferno-package");
export const imfReport = loadSchema("imf-report");
export const validationReport = loadSchema("validation-report");
export const compositionPlaylist = loadSchema("composition-playlist");
export const assetMap = loadSchema("asset-map");
export const packingList = loadSchema("packing-list");
export const volumeIndex = loadSchema("volume-index");
export const rulesConfig = loadSchema("rules-config");

export const schemas = {
  validationResult,
  imfernoPackage,
  imfReport,
  validationReport,
  compositionPlaylist,
  assetMap,
  packingList,
  volumeIndex,
  rulesConfig,
};
