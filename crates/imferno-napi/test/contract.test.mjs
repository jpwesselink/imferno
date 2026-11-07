// Contract test: every value the native module returns must conform to the
// TypeScript type its published declaration (index.d.ts) promises — for valid
// and invalid IMF alike.
//
// The JSON Schema used here is derived from the published .d.ts files
// themselves (ts-json-schema-generator), not from the Rust side, so a drift
// between what we declare and what we serialise fails here. Object types are
// closed (`additionalProperties: false`): an undeclared field is a failure too.
// The schemas are generated for the named types (ValidationResult, Imferno,
// ...); types.test-d.ts — type-checked below — pins each exported function's
// signature to exactly those names, so together they cover the signatures.
//
// Requires a built module: `npm run build` (napi build) first.
import { describe, it, expect, beforeAll } from "vitest";
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createGenerator } from "ts-json-schema-generator";
import Ajv from "ajv";
import ts from "typescript";
import { inspectDeclarations } from "../../../scripts/dts-contract.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const pkgDir = resolve(here, "..");
const repoRoot = resolve(pkgDir, "../..");
const testData = join(repoRoot, "test-data");
const require = createRequire(import.meta.url);

// Load through the package entry point (falls back to ./imferno-node.node).
const api = require(pkgDir);

/** Every directory under test-data/ that holds an ASSETMAP (valid and invalid IMPs). */
function findPackages(dir) {
  const out = [];
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name);
    if (e.isDirectory()) out.push(...findPackages(p));
    else if (/^assetmap/i.test(e.name) && !out.includes(dir)) out.push(dir);
  }
  return out;
}

function readXmlFiles(dir) {
  const files = {};
  for (const f of readdirSync(dir)) {
    if (f.toLowerCase().endsWith(".xml")) files[f] = readFileSync(join(dir, f), "utf-8");
  }
  return files;
}

const packages = findPackages(testData).sort();

let ajv;
const validators = {};
function check(typeName, value, label) {
  const v = validators[typeName];
  if (!v(value)) {
    const errs = v.errors
      .slice(0, 10)
      .map((e) => `  ${e.instancePath || "/"} ${e.message} ${JSON.stringify(e.params)}`)
      .join("\n");
    throw new Error(`${label}: output violates declared type ${typeName}\n${errs}`);
  }
}

beforeAll(() => {
  const generator = createGenerator({
    path: join(pkgDir, "{index,types}.d.ts"),
    tsconfig: join(here, "tsconfig.json"),
    type: "*",
    expose: "export",
    additionalProperties: false,
    skipTypeCheck: true,
    topRef: true,
  });
  const schema = generator.createSchema("*");
  ajv = new Ajv({ strict: false, allErrors: true });
  ajv.addSchema(schema, "api");
  for (const t of ["ValidationResult", "Imferno", "ImfReport", "RuleInfo", "ValidateOptions", "UriValidateOptions"]) {
    validators[t] = ajv.getSchema(`api#/definitions/${t}`);
    if (!validators[t]) throw new Error(`type ${t} not found in index.d.ts`);
  }
  validators["RuleInfo[]"] = ajv.compile({ type: "array", items: { $ref: "api#/definitions/RuleInfo" } });
}, 120_000);

describe("declarations", () => {
  it("type-check (tsc, strict) including the type-level contract", () => {
    const tsc = require.resolve("typescript/bin/tsc");
    execFileSync(process.execPath, [tsc, "-p", join(here, "tsconfig.json")], { stdio: "pipe" });
  }, 120_000);

  it("reach no `any` from any export (functions, properties, nested types)", () => {
    const { anyPaths } = inspectDeclarations(ts, join(pkgDir, "index.d.ts"));
    expect(anyPaths).toEqual([]);
  });

  it("declare exactly the values the module exports", () => {
    const { valueExports } = inspectDeclarations(ts, join(pkgDir, "index.d.ts"));
    expect(valueExports).toEqual(Object.keys(api).sort());
  });

});

describe("hand-written option types match what parse_options reads", () => {
  const declaredKeys = (t) => Object.keys(ajv.getSchema(`api#/definitions/${t}`).schema.properties).sort();
  const readKeys = (rsPath) =>
    [...new Set([...readFileSync(rsPath, "utf-8").matchAll(/opts\s*\.get\("([A-Za-z0-9]+)"\)/g)].map((m) => m[1]))].sort();

  it("NAPI: UriValidateOptions declares exactly the keys src/lib.rs reads", () => {
    expect(declaredKeys("UriValidateOptions")).toEqual(readKeys(join(pkgDir, "src/lib.rs")));
  });

  it("WASM: ValidateOptions declares exactly the keys imferno-wasm/src/lib.rs reads", () => {
    expect(declaredKeys("ValidateOptions")).toEqual(readKeys(join(repoRoot, "crates/imferno-wasm/src/lib.rs")));
  });

  const literals = (t, prop) => {
    const s = ajv.getSchema(`api#/definitions/${t}`).schema.properties[prop];
    const target = s.$ref ? ajv.getSchema(`api${s.$ref}`).schema : s;
    return target.enum;
  };
  const meridian = join(testData, "MERIDIAN_Netflix_Photon_161006");

  it("every declared coreSpec / app2eSpec literal is accepted by the engine", () => {
    const core = literals("ValidateOptions", "coreSpec");
    const app = literals("ValidateOptions", "app2eSpec");
    expect(core).toContain("auto");
    expect(app).toContain("none");
    for (const coreSpec of core) expect(() => api.validatePath(meridian, { coreSpec }), coreSpec).not.toThrow();
    for (const app2eSpec of app) expect(() => api.validatePath(meridian, { app2eSpec }), app2eSpec).not.toThrow();
    expect(() => api.validatePath(meridian, { coreSpec: "v2019" })).toThrow(/coreSpec/);
    expect(() => api.validatePath(meridian, { app2eSpec: "v2021" })).toThrow(/app2eSpec/);
  });

  it("every declared rule severity is accepted by the engine", () => {
    const rulesSchema = ajv.getSchema("api#/definitions/RulesConfig").schema;
    const ap = rulesSchema.additionalProperties;
    const ref = ap.$ref ?? ap.anyOf.find((s) => s.$ref).$ref;
    const sev = ajv.getSchema(`api${ref}`).schema.enum;
    expect(sev.length).toBeGreaterThan(0);
    for (const s of sev) expect(() => api.validatePath(meridian, { rules: { SegmentDuration: s } }), s).not.toThrow();
    expect(() => api.validatePath(meridian, { rules: { SegmentDuration: "fatal" } })).toThrow(/rules/);
  });
});

describe("runtime outputs conform to their declared types", () => {
  it("finds the test-data packages", () => {
    expect(packages.length).toBeGreaterThan(20);
  });

  describe.each(packages.map((p) => [p.slice(testData.length + 1), p]))("%s", (_name, dir) => {
    it("validatePath / validateUri / validate → ValidationResult", () => {
      check("ValidationResult", api.validatePath(dir), "validatePath");
      check("ValidationResult", api.validatePath(dir, { aggregateRepeats: true, skipDiskChecks: true }), "validatePath(opts)");
      check("ValidationResult", api.validateUri(pathToFileURL(dir).href), "validateUri");
      const inMemory = api.validate(readXmlFiles(dir));
      check("ValidationResult", inMemory, "validate");
      // formatReport accepts what validate* returns.
      expect(api.formatReport(inMemory)).toMatch(/CPL|ASSETMAP|error|ok/);
    });

    it("parsePackage* → Imferno, buildReport* → ImfReport (when they don't throw)", () => {
      let parsed;
      try {
        parsed = api.parsePackageFromPath(dir);
      } catch (e) {
        expect(e).toBeInstanceOf(Error);
      }
      if (parsed !== undefined) {
        check("Imferno", parsed, "parsePackageFromPath");
        check("Imferno", api.parsePackage(readXmlFiles(dir)), "parsePackage");
      }
      let report;
      try {
        report = api.buildReportFromPath(dir);
      } catch (e) {
        expect(e).toBeInstanceOf(Error);
      }
      if (report !== undefined) {
        check("ImfReport", report, "buildReportFromPath");
        check("ImfReport", api.buildReport(readXmlFiles(dir)), "buildReport");
        check("ImfReport", api.buildReportFromUri(dir), "buildReportFromUri");
        expect(typeof api.formatReport(report)).toBe("string");
      }
    });
  });

  it("degenerate inputs (no files, garbage XML) still return a ValidationResult", () => {
    check("ValidationResult", api.validate({}), "validate({})");
    check("ValidationResult", api.validate({ "ASSETMAP.xml": "<not-xml" }), "validate(garbage)");
    check("ValidationResult", api.validate({ "ASSETMAP.xml": "<AssetMap/>", "CPL.xml": "<x/>" }), "validate(empty AssetMap)");
  });

  it("suppressed findings (rules: off) keep the declared shape", () => {
    const dir = join(testData, "MERIDIAN_Netflix_Photon_161006");
    const all = api.validatePath(dir).validation;
    const first = [...all.critical, ...all.errors, ...all.warnings, ...all.info][0];
    expect(first, "fixture should produce at least one finding").toBeDefined();
    const r = api.validatePath(dir, { rules: { [first.code]: "off" } });
    check("ValidationResult", r, "validatePath(rules off)");
    expect(r.validation.suppressed?.length ?? 0).toBeGreaterThan(0);
  });

  it("the SCM fixture exercises the SidecarCompositionMap shape", () => {
    const r = api.validatePath(join(testData, "SCM/SidecarIMP"));
    check("ValidationResult", r, "validatePath(SCM)");
    const scms = Object.values(r.package.sidecarCompositionMaps);
    expect(scms).toHaveLength(1);
    expect(scms[0].sidecarAssets[0].cplIds).toHaveLength(1);
  });

  it("listRules → RuleInfo[]", () => {
    const rules = api.listRules();
    expect(rules.length).toBeGreaterThan(100);
    check("RuleInfo[]", rules, "listRules");
  });

  it("getVersion → string", () => {
    expect(typeof api.getVersion()).toBe("string");
  });

  it("codes is exported from the main entry", () => {
    expect(api.codes.ST2067_2_2020.FileNotFound).toBe("ST2067-2:2020:8.3/FileNotFound");
  });
});

describe("the guard itself", () => {
  it("rejects a value with an undeclared field or a wrong-typed field", () => {
    const r = api.validate({});
    expect(validators.ValidationResult({ ...r, extra: 1 })).toBe(false);
    expect(validators.ValidationResult({ ...r, validation: { ...r.validation, is_compliant: "yes" } })).toBe(false);
  });

  it("test data is present", () => {
    expect(existsSync(testData)).toBe(true);
  });
});
