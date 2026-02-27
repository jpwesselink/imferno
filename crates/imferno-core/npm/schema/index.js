import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const schemasDir = join(__dirname, "schemas");

function loadSchema(name) {
  return JSON.parse(readFileSync(join(schemasDir, `${name}.json`), "utf-8"));
}

export const imfReport = loadSchema("imf-report");
export const validationReport = loadSchema("validation-report");
export const compositionPlaylist = loadSchema("composition-playlist");
export const assetMap = loadSchema("asset-map");
export const packingList = loadSchema("packing-list");
export const volumeIndex = loadSchema("volume-index");
export const sourceAsset = loadSchema("source-asset");
export const deliveryRequest = loadSchema("delivery-request");
export const deliveryComparison = loadSchema("delivery-comparison");
export const rulesConfig = loadSchema("rules-config");

export const schemas = {
  imfReport,
  validationReport,
  compositionPlaylist,
  assetMap,
  packingList,
  volumeIndex,
  sourceAsset,
  deliveryRequest,
  deliveryComparison,
  rulesConfig,
};
