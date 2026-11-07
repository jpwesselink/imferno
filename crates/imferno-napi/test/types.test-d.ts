// Type-level contract for the published declarations (index.d.ts + types.d.ts).
// Type-checked (never executed) by `tsc -p test/tsconfig.json`, which
// contract.test.mjs runs. A failure here means the public signatures changed.
import * as api from "../index";
import type {
  ImfReport,
  Imferno,
  PathValidateOptions,
  RuleInfo,
  SarifLog,
  UriValidateOptions,
  ValidateOptions,
  ValidationIssue,
  ValidationReport,
  ValidationResult,
} from "../index";

type IsAny<T> = 0 extends 1 & T ? true : false;
type Equals<A, B> =
  (<T>() => T extends A ? 1 : 2) extends <T>() => T extends B ? 1 : 2 ? true : false;
function assert<T extends true>(): void {}

// No export may be `any`, nor take or return `any`.
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

// Return types — exactly the generated types, synchronous (no Promise).
assert<Equals<ReturnType<typeof api.validate>, ValidationResult>>();
assert<Equals<ReturnType<typeof api.validatePath>, ValidationResult>>();
assert<Equals<ReturnType<typeof api.validateUri>, ValidationResult>>();
assert<Equals<ReturnType<typeof api.parsePackage>, Imferno>>();
assert<Equals<ReturnType<typeof api.parsePackageFromPath>, Imferno>>();
assert<Equals<ReturnType<typeof api.buildReport>, ImfReport>>();
assert<Equals<ReturnType<typeof api.buildReportFromPath>, ImfReport>>();
assert<Equals<ReturnType<typeof api.buildReportFromUri>, ImfReport>>();
assert<Equals<ReturnType<typeof api.formatReport>, string>>();
assert<Equals<Parameters<typeof api.formatReport>, [report: ValidationResult | ImfReport]>>();
assert<Equals<ReturnType<typeof api.listRules>, RuleInfo[]>>();
assert<Equals<ReturnType<typeof api.toSarif>, SarifLog>>();
assert<Equals<Parameters<typeof api.toSarif>, [report: ValidationResult | ValidationReport]>>();
assert<Equals<ReturnType<typeof api.getVersion>, string>>();

// Option parameters.
assert<Equals<Parameters<typeof api.validate>[1], ValidateOptions | undefined | null>>();
assert<Equals<Parameters<typeof api.validatePath>[1], PathValidateOptions | undefined | null>>();
assert<Equals<Parameters<typeof api.validateUri>[1], UriValidateOptions | undefined | null>>();

// Field-level spot checks of the serialised shape (snake_case report fields,
// camelCase package fields).
declare const r: ValidationResult;
const compliant: boolean = r.validation.is_compliant;
const issue: ValidationIssue | undefined = r.validation.errors[0];
if (issue) {
  const sev: "Info" | "Warning" | "Error" | "Critical" = issue.severity;
  const code: string = issue.code;
  const cplId: string | null | undefined = issue.location.cpl_id;
  void sev, code, cplId;
}
const cplCount: number = Object.keys(r.package.compositionPlaylists).length;
void compliant, cplCount;

// Options reject unknown spec editions and severities.
// @ts-expect-error — not a core spec edition
api.validate({}, { coreSpec: "v2019" });
// @ts-expect-error — not an App #2E edition
api.validate({}, { app2eSpec: "v2021" });
// @ts-expect-error — not a rule severity
api.validate({}, { rules: { SegmentDuration: "fatal" } });
// @ts-expect-error — skipDiskChecks only applies to on-disk packages
api.validate({}, { skipDiskChecks: true });
api.validatePath("x", { coreSpec: "v2020", app2eSpec: "none", skipDiskChecks: true, rules: { "XSD/*": "off" } });
api.validateUri("s3://b/p/", { credentials: { accessKeyId: "a", secretAccessKey: "s" } });

// `codes` is re-exported from the main entry.
const c: "ST2067-2/FileNotFound" = api.codes.ST2067_2.FileNotFound;
void c;
