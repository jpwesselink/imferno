// Shared by the @imferno/node and @imferno/wasm contract tests: inspects a
// package's public declaration entry point with the TypeScript compiler API.
// Takes the `typescript` module as a parameter so each package uses its own
// devDependency.

/**
 * @param {typeof import("typescript")} ts
 * @param {string} entryFile absolute path to the package's index.d.ts
 * @param {import("typescript").CompilerOptions} [options]
 * @returns {{ valueExports: string[], anyPaths: string[] }}
 *   `valueExports`: names of runtime values the declarations export (to
 *   compare with the real module's keys); `anyPaths`: every place an `any`
 *   is reachable from an export, through functions, properties, index
 *   signatures, unions and type arguments (standard-library members are not
 *   walked, only their type arguments).
 */
export function inspectDeclarations(ts, entryFile, options = {}) {
  const program = ts.createProgram([entryFile], {
    strict: true,
    noEmit: true,
    types: [],
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
    ...options,
  });
  const diagnostics = ts.getPreEmitDiagnostics(program);
  if (diagnostics.length) {
    const msg = ts.formatDiagnostics(diagnostics, {
      getCanonicalFileName: (f) => f,
      getCurrentDirectory: () => process.cwd(),
      getNewLine: () => "\n",
    });
    throw new Error(`declarations do not type-check:\n${msg}`);
  }
  const checker = program.getTypeChecker();
  const source = program.getSourceFile(entryFile);
  const moduleSymbol = checker.getSymbolAtLocation(source);

  const isLibSymbol = (sym) =>
    !!sym?.declarations?.length &&
    sym.declarations.every((d) => program.isSourceFileDefaultLibrary(d.getSourceFile()));

  const anyPaths = [];
  const seen = new Set();
  function visit(type, path) {
    if (type.flags & ts.TypeFlags.Any) {
      anyPaths.push(path);
      return;
    }
    if (seen.has(type)) return;
    seen.add(type);
    if (type.isUnionOrIntersection()) {
      for (const t of type.types) visit(t, path);
      return;
    }
    if (!(type.flags & ts.TypeFlags.Object)) return;
    if (type.objectFlags & ts.ObjectFlags.Reference) {
      checker.getTypeArguments(type).forEach((t, i) => visit(t, `${path}<${i}>`));
    }
    if (type.aliasTypeArguments) type.aliasTypeArguments.forEach((t, i) => visit(t, `${path}<${i}>`));
    if (isLibSymbol(type.getSymbol())) return; // Promise, Array, Record internals
    for (const sig of [...type.getCallSignatures(), ...type.getConstructSignatures()]) {
      for (const p of sig.getParameters()) visit(checker.getTypeOfSymbol(p), `${path}(${p.name})`);
      visit(checker.getReturnTypeOfSignature(sig), `${path}()`);
    }
    for (const prop of checker.getPropertiesOfType(type)) {
      visit(checker.getTypeOfSymbol(prop), `${path}.${prop.name}`);
    }
    for (const info of checker.getIndexInfosOfType(type)) visit(info.type, `${path}[]`);
  }

  const valueExports = [];
  for (let sym of checker.getExportsOfModule(moduleSymbol)) {
    const name = sym.name;
    if (sym.flags & ts.SymbolFlags.Alias) sym = checker.getAliasedSymbol(sym);
    if (sym.flags & ts.SymbolFlags.Value) {
      valueExports.push(name);
      visit(checker.getTypeOfSymbol(sym), name);
    }
    if (sym.flags & ts.SymbolFlags.Type) visit(checker.getDeclaredTypeOfSymbol(sym), name);
  }
  return { valueExports: valueExports.sort(), anyPaths };
}
