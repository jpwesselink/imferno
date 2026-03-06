import { validatePath, codes } from "@imferno/node";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const impPath = resolve(__dirname, "../../test-data/HT/IMP");

const result = validatePath(impPath, {
  rules: {
    // Promote checksum mismatches to critical
    [codes.ST2067_2_2020.ChecksumMismatch]: "critical",
    // Suppress unreferenced asset warnings
    [codes.Imferno.UnreferencedAsset]: "off",
  },
});

console.log("Compliant:", result.report.is_compliant);
console.log("Errors:", result.report.errors.length);
console.log("Warnings:", result.report.warnings.length);
console.log("Info:", result.report.info.length);

if (result.report.errors.length > 0) {
  console.log("\nErrors:");
  for (const issue of result.report.errors) {
    console.log(`  ${issue.code}: ${issue.message}`);
  }
}

if (result.report.warnings.length > 0) {
  console.log("\nWarnings:");
  for (const issue of result.report.warnings) {
    console.log(`  ${issue.code}: ${issue.message}`);
  }
}

if (result.report.info.length > 0) {
  console.log("\nInfo:");
  for (const issue of result.report.info) {
    console.log(`  ${issue.code}: ${issue.message}`);
  }
}

console.log("\nFull report:");
console.log(JSON.stringify(result, null, 2));

process.exit(result.report.is_compliant ? 0 : 1);
