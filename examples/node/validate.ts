import { buildReportFromPath, formatReport, codes } from "@imferno/node";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const impPath = resolve(__dirname, "../../test-data/HT/IMP");

const report = buildReportFromPath(impPath, {
  rules: {
    // Promote checksum mismatches to critical
    [codes.ST2067_2_2020.ChecksumMismatch]: "critical",
    // Suppress unreferenced asset warnings
    [codes.Imferno.UnreferencedAsset]: "off",
  },
});

console.log("Compliant:", report.validation.is_compliant);
console.log("Errors:", report.validation.errors.length);
console.log("Warnings:", report.validation.warnings.length);
console.log("Info:", report.validation.info.length);

if (report.validation.errors.length > 0) {
  console.log("\nErrors:");
  for (const issue of report.validation.errors) {
    console.log(`  ${issue.code}: ${issue.message}`);
  }
}

if (report.validation.warnings.length > 0) {
  console.log("\nWarnings:");
  for (const issue of report.validation.warnings) {
    console.log(`  ${issue.code}: ${issue.message}`);
  }
}

if (report.validation.info.length > 0) {
  console.log("\nInfo:");
  for (const issue of report.validation.info) {
    console.log(`  ${issue.code}: ${issue.message}`);
  }
}

console.log("\nFormatted report:");
console.log(formatReport(report));

process.exit(report.validation.is_compliant ? 0 : 1);
