#!/usr/bin/env node

const { execFileSync } = require("child_process");
const path = require("path");

const PLATFORMS = {
  "linux-x64": "imferno-linux-x64-gnu",
  "linux-arm64": "imferno-linux-arm64-gnu",
  "darwin-x64": "imferno-darwin-x64",
  "darwin-arm64": "imferno-darwin-arm64",
  "win32-x64": "imferno-win32-x64-msvc",
  "win32-arm64": "imferno-win32-arm64-msvc",
};

const key = `${process.platform}-${process.arch}`;
const pkg = PLATFORMS[key];

if (!pkg) {
  console.error(
    `imferno: unsupported platform ${process.platform} ${process.arch}\n` +
    `Supported: ${Object.keys(PLATFORMS).join(", ")}`
  );
  process.exit(1);
}

let binPath;
try {
  const pkgDir = path.dirname(require.resolve(`${pkg}/package.json`));
  const ext = process.platform === "win32" ? ".exe" : "";
  binPath = path.join(pkgDir, `imferno${ext}`);
} catch {
  console.error(
    `imferno: could not find package "${pkg}"\n\n` +
    `This usually means the optional dependency was not installed.\n` +
    `Try reinstalling with: npm install imferno`
  );
  process.exit(1);
}

try {
  execFileSync(binPath, process.argv.slice(2), { stdio: "inherit" });
} catch (e) {
  if (e.status !== null) process.exit(e.status);
  throw e;
}
