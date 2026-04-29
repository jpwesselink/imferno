import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

const PLATFORMS = {
  "darwin-arm64": "@imferno/node-darwin-arm64",
  "darwin-x64": "@imferno/node-darwin-x64",
  "linux-x64": "@imferno/node-linux-x64-gnu",
  "linux-arm64": "@imferno/node-linux-arm64-gnu",
  "win32-x64": "@imferno/node-win32-x64-msvc",
  "win32-arm64": "@imferno/node-win32-arm64-msvc",
};

const key = `${process.platform}-${process.arch}`;
const pkg = PLATFORMS[key];

if (!pkg) {
  throw new Error(
    `@imferno/node: unsupported platform ${process.platform}-${process.arch}\n` +
    `Supported: ${Object.keys(PLATFORMS).join(", ")}`
  );
}

let nativeModule;
try {
  nativeModule = require(pkg);
} catch {
  try {
    const pkgDir = dirname(require.resolve(`${pkg}/package.json`));
    nativeModule = require(join(pkgDir, "imferno-node.node"));
  } catch {
    try {
      nativeModule = require(join(__dirname, "imferno-node.node"));
    } catch {
      throw new Error(
        `@imferno/node: could not load native module "${pkg}"\n\n` +
        `This usually means the optional dependency was not installed.\n` +
        `Try reinstalling with: npm install @imferno/node`
      );
    }
  }
}

export const {
  getVersion,
  buildReport,
  buildReportFromPath,
  buildReportFromUri,
  formatReport,
  parsePackage,
  parsePackageFromPath,
  validate,
  validatePath,
  validateUri,
} = nativeModule;

export { codes } from "./codes.mjs";
