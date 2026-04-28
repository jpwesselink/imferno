const path = require("path");

const PLATFORMS = {
  "darwin-arm64": "@imferno/darwin-arm64",
  "darwin-x64": "@imferno/darwin-x64",
  "linux-x64": "@imferno/linux-x64-gnu",
  "linux-arm64": "@imferno/linux-arm64-gnu",
  "win32-x64": "@imferno/win32-x64-msvc",
  "win32-arm64": "@imferno/win32-arm64-msvc",
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
    // Fallback: try loading the .node file from the platform package directory
    const pkgDir = path.dirname(require.resolve(`${pkg}/package.json`));
    nativeModule = require(path.join(pkgDir, "imferno-node.node"));
  } catch {
    try {
      // Local development: try loading the .node file from the same directory
      nativeModule = require(path.join(__dirname, "imferno-node.node"));
    } catch {
      throw new Error(
        `@imferno/node: could not load native module "${pkg}"\n\n` +
        `This usually means the optional dependency was not installed.\n` +
        `Try reinstalling with: npm install @imferno/node`
      );
    }
  }
}

const { codes } = require("./codes.js");

module.exports = { ...nativeModule, codes };
