// Type-level contract for the published declarations (index.d.ts + types.d.ts).
// Type-checked (never executed) by `tsc -p test/tsconfig.json`, which
// types-contract.test.js runs. A failure here means the public signatures changed.
import * as api from "../index.js";
import type {
  ImfReport,
  Imferno,
  RuleInfo,
  SarifLog,
  ValidateOptions,
  ValidationReport,
  ValidationResult,
} from "../index.js";

type IsAny<T> = 0 extends 1 & T ? true : false;
type Equals<A, B> =
  (<T>() => T extends A ? 1 : 2) extends <T>() => T extends B ? 1 : 2 ? true : false;
function assert<T extends true>(): void {}

// No exported function may take or return `any`.
type Fns = { [K in keyof typeof api]: (typeof api)[K] extends (...a: any[]) => any ? K : never }[keyof typeof api];
type AnyInSignature<F> = F extends (...args: infer A) => infer R
  ? IsAny<R> extends true
    ? true
    : true extends { [I in keyof A]: IsAny<A[I]> }[number]
      ? true
      : false
  : false;
type FnsWithAny = { [K in Fns]: AnyInSignature<(typeof api)[K]> extends true ? K : never }[Fns];
assert<Equals<FnsWithAny, never>>();

// Every function initialises WASM first, so every one returns a Promise.
assert<Equals<ReturnType<typeof api.validate>, Promise<ValidationResult>>>();
assert<Equals<ReturnType<typeof api.parsePackage>, Promise<Imferno>>>();
assert<Equals<ReturnType<typeof api.buildReport>, Promise<ImfReport>>>();
assert<Equals<ReturnType<typeof api.formatReport>, Promise<string>>>();
assert<Equals<Parameters<typeof api.formatReport>, [report: ValidationResult | ImfReport]>>();
assert<Equals<ReturnType<typeof api.toSarif>, Promise<SarifLog>>>();
assert<Equals<Parameters<typeof api.toSarif>, [report: ValidationResult | ValidationReport]>>();
assert<Equals<ReturnType<typeof api.listRules>, Promise<RuleInfo[]>>>();
assert<Equals<ReturnType<typeof api.getVersion>, Promise<string>>>();
assert<Equals<ReturnType<typeof api.init>, Promise<void>>>();

assert<Equals<Parameters<typeof api.validate>, [files: Record<string, string>, options?: ValidateOptions | null]>>();
assert<Equals<Parameters<typeof api.parsePackage>, [files: Record<string, string>]>>();

// Spot checks of the serialised shape: the WASM build uses camelCase CPL fields.
declare const r: ValidationResult;
const compliant: boolean = r.validation.is_compliant;
for (const cpl of Object.values(r.package.compositionPlaylists)) {
  if (cpl) {
    const title = cpl.contentTitle;
    void title;
  }
}
void compliant;

// @ts-expect-error — not a core spec edition
api.validate({}, { coreSpec: "v2019" });
// @ts-expect-error — not a rule severity
api.validate({}, { rules: { SegmentDuration: "fatal" } });

const c: "ST2067-2/FileNotFound" = api.codes.ST2067_2.FileNotFound;
void c;
