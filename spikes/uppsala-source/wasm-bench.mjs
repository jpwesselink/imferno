// Criterion 6 (wasm32): time uppsala vs imferno's quick-xml parse under Node.
//
// Build:  wasm-pack build --target nodejs --release --no-opt \
//           --out-dir target/wasm-pkg -- --features wasm
// Input:  run `cargo run --release --bin perf` first (writes the 5 MB CPL).
// Run:    node wasm-bench.mjs            (timings, one process)
//         node wasm-bench.mjs mem <fn>   (memory high-water of one fn, fresh process)

import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const pkg = require(path.join(here, "target/wasm-pkg/uppsala_source_spike.js"));
const repo = path.join(here, "../..");

const inputs = {
  "largest CPL (45 KB)": path.join(repo, "test-data/Netflix_Sony_Plugfest_2015/CPL_BLACKL_202_HD_REC709_178_LAS_8fad47bb-ab01-4f0d-a08c-d1e6c6cb62b4_audio_homogeneity_fail.xml"),
  "synthetic 5 MB CPL": path.join(here, "target/synthetic-5mb-cpl.xml"),
};
const fns = ["bench_uppsala", "bench_uppsala_indexed", "bench_imferno_cpl", "bench_quick_xml_events"];

if (process.argv[2] === "mem") {
  const xml = readFileSync(inputs["synthetic 5 MB CPL"], "utf8");
  const before = pkg.wasm_memory_bytes();
  pkg[process.argv[3]](xml, 1);
  const after = pkg.wasm_memory_bytes();
  console.log(JSON.stringify({ before, after }));
  process.exit(0);
}

for (const [label, file] of Object.entries(inputs)) {
  const xml = readFileSync(file, "utf8");
  console.log(`\n### ${label} (${xml.length} chars)\n`);
  console.log("| operation (wasm32, Node " + process.version + ") | median per parse |");
  console.log("|---|---|");
  for (const fn of fns) {
    pkg[fn](xml, 1); // warm-up
    // Batch so one sample takes >= ~20 ms, then take the median of samples.
    let iters = 1;
    for (;;) {
      const t = performance.now();
      pkg[fn](xml, iters);
      if (performance.now() - t > 20 || iters > 1e5) break;
      iters *= 2;
    }
    const samples = [];
    const start = performance.now();
    while (samples.length < 5 || (performance.now() - start < 1000 && samples.length < 50)) {
      const t = performance.now();
      pkg[fn](xml, iters);
      samples.push((performance.now() - t) / iters);
    }
    samples.sort((a, b) => a - b);
    const ms = samples[samples.length >> 1];
    const shown = ms >= 1 ? `${ms.toFixed(2)} ms` : `${(ms * 1000).toFixed(1)} µs`;
    console.log(`| ${fn} | ${shown} |`);
  }
}

console.log("\n### wasm linear memory high-water, synthetic 5 MB CPL (fresh instance per fn)\n");
console.log("| operation | memory before | memory after |");
console.log("|---|---|---|");
for (const fn of fns) {
  const out = execFileSync(process.execPath, [fileURLToPath(import.meta.url), "mem", fn]).toString();
  const { before, after } = JSON.parse(out);
  console.log(`| ${fn} | ${(before / 1048576).toFixed(1)} MiB | ${(after / 1048576).toFixed(1)} MiB |`);
}
