use super::BindingSpec;

pub(crate) fn emit(
    spec: &BindingSpec,
    json: &str,
) -> String {
    let mut out = String::new();
    out.push_str("const BINDING_SPEC = ");
    out.push_str(json);
    out.push_str(";\n\n");
    out.push_str("export const wasmFileName = ");
    out.push_str(&quote_string(&spec.wasm_file_name));
    out.push_str(";\n\n");
    out.push_str("function describeValue(value) {\n");
    out.push_str("  if (value === null) {\n");
    out.push_str("    return \"null\";\n");
    out.push_str("  }\n");
    out.push_str("  if (Array.isArray(value)) {\n");
    out.push_str("    return \"array\";\n");
    out.push_str("  }\n");
    out.push_str("  return typeof value;\n");
    out.push_str("}\n\n");
    out.push_str("function validatePrimitive(kind, value, context) {\n");
    out.push_str("  if (kind === \"externref\") {\n");
    out.push_str("    return;\n");
    out.push_str("  }\n");
    out.push_str("  if (kind === \"number\") {\n");
    out.push_str("    if (typeof value !== \"number\") {\n");
    out.push_str("      throw new TypeError(`${context}: expected number, received ${describeValue(value)}`);\n");
    out.push_str("    }\n");
    out.push_str("    return;\n");
    out.push_str("  }\n");
    out.push_str("  if (kind === \"bigint\" && typeof value !== \"bigint\") {\n");
    out.push_str("    throw new TypeError(`${context}: expected bigint, received ${describeValue(value)}`);\n");
    out.push_str("  }\n");
    out.push_str("}\n\n");
    out.push_str("function validateFunctionImport(moduleName, importName, rawFunction, parameters, results) {\n");
    out.push_str("  if (typeof rawFunction !== \"function\") {\n");
    out.push_str(
        "    throw new TypeError(`Missing function import ${moduleName}.${importName}`);\n",
    );
    out.push_str("  }\n\n");
    out.push_str("  return (...args) => {\n");
    out.push_str("    if (args.length < parameters.length) {\n");
    out.push_str("      throw new TypeError(`Import ${moduleName}.${importName} expects ${parameters.length} argument(s), received ${args.length}`);\n");
    out.push_str("    }\n");
    out.push_str("    for (let index = 0; index < parameters.length; index += 1) {\n");
    out.push_str("      validatePrimitive(parameters[index], args[index], `Invalid import ${moduleName}.${importName} argument #${index}`);\n");
    out.push_str("    }\n\n");
    out.push_str("    const result = rawFunction(...args);\n\n");
    out.push_str("    if (results.length === 0) {\n");
    out.push_str("      return undefined;\n");
    out.push_str("    }\n");
    out.push_str("    if (results.length === 1) {\n");
    out.push_str("      validatePrimitive(results[0], result, `Invalid import ${moduleName}.${importName} return value`);\n");
    out.push_str("      return result;\n");
    out.push_str("    }\n\n");
    out.push_str("    if (!Array.isArray(result) || result.length !== results.length) {\n");
    out.push_str("      throw new TypeError(`Import ${moduleName}.${importName} must return an array with ${results.length} item(s)`);\n");
    out.push_str("    }\n");
    out.push_str("    for (let index = 0; index < results.length; index += 1) {\n");
    out.push_str("      validatePrimitive(results[index], result[index], `Invalid import ${moduleName}.${importName} return value #${index}`);\n");
    out.push_str("    }\n");
    out.push_str("    return result;\n");
    out.push_str("  };\n");
    out.push_str("}\n\n");
    out.push_str("function validateGlobalImport(moduleName, importName, rawGlobal, mutable) {\n");
    out.push_str("  if (rawGlobal === undefined) {\n");
    out.push_str("    throw new TypeError(`Missing global import ${moduleName}.${importName}`);\n");
    out.push_str("  }\n");
    out.push_str("  if (!mutable) {\n");
    out.push_str("    return rawGlobal;\n");
    out.push_str("  }\n");
    out.push_str("  if (!(rawGlobal instanceof WebAssembly.Global)) {\n");
    out.push_str("    throw new TypeError(`Global import ${moduleName}.${importName} must be a WebAssembly.Global`);\n");
    out.push_str("  }\n");
    out.push_str("  return rawGlobal;\n");
    out.push_str("}\n\n");
    out.push_str("export function validateImports(imports = {}) {\n");
    out.push_str("  if (imports === null || typeof imports !== \"object\") {\n");
    out.push_str("    throw new TypeError(\"Imports must be an object\");\n");
    out.push_str("  }\n\n");
    out.push_str("  const validated = { ...imports };\n");
    out.push_str("  for (const moduleSpec of BINDING_SPEC.imports) {\n");
    out.push_str("    const moduleValue = imports[moduleSpec.module];\n");
    out.push_str("    if (moduleValue === null || typeof moduleValue !== \"object\") {\n");
    out.push_str("      throw new TypeError(`Missing import module ${moduleSpec.module}`);\n");
    out.push_str("    }\n\n");
    out.push_str("    const validatedModule = { ...moduleValue };\n");
    out.push_str("    for (const functionSpec of moduleSpec.functions) {\n");
    out.push_str("      const rawFunction = moduleValue[functionSpec.import_name];\n");
    out.push_str("      validatedModule[functionSpec.import_name] = validateFunctionImport(\n");
    out.push_str("        moduleSpec.module,\n");
    out.push_str("        functionSpec.import_name,\n");
    out.push_str("        rawFunction,\n");
    out.push_str("        functionSpec.parameters,\n");
    out.push_str("        functionSpec.results,\n");
    out.push_str("      );\n");
    out.push_str("    }\n\n");
    out.push_str("    for (const globalSpec of moduleSpec.globals) {\n");
    out.push_str("      const rawGlobal = moduleValue[globalSpec.import_name];\n");
    out.push_str("      validatedModule[globalSpec.import_name] = validateGlobalImport(\n");
    out.push_str("        moduleSpec.module,\n");
    out.push_str("        globalSpec.import_name,\n");
    out.push_str("        rawGlobal,\n");
    out.push_str("        globalSpec.mutable,\n");
    out.push_str("      );\n");
    out.push_str("    }\n\n");
    out.push_str("    validated[moduleSpec.module] = validatedModule;\n");
    out.push_str("  }\n\n");
    out.push_str("  return validated;\n");
    out.push_str("}\n\n");
    out.push_str("function toInstantiatedModule(module, instance) {\n");
    out.push_str("  return {\n");
    out.push_str("    module,\n");
    out.push_str("    instance,\n");
    out.push_str("    exports: instance.exports,\n");
    out.push_str("  };\n");
    out.push_str("}\n\n");
    out.push_str("export async function instantiate(source, imports = {}) {\n");
    out.push_str("  const validatedImports = validateImports(imports);\n");
    out.push_str("  if (source instanceof WebAssembly.Module) {\n");
    out.push_str("    const instance = await WebAssembly.instantiate(source, validatedImports);\n");
    out.push_str("    return toInstantiatedModule(source, instance);\n");
    out.push_str("  }\n\n");
    out.push_str(
        "  const instantiated = await WebAssembly.instantiate(source, validatedImports);\n",
    );
    out.push_str("  return toInstantiatedModule(instantiated.module, instantiated.instance);\n");
    out.push_str("}\n\n");
    out.push_str("export async function instantiateStreaming(source, imports = {}) {\n");
    out.push_str("  const validatedImports = validateImports(imports);\n");
    out.push_str("  const response = await source;\n");
    out.push_str("  const instantiated = await WebAssembly.instantiateStreaming(response, validatedImports);\n");
    out.push_str("  return toInstantiatedModule(instantiated.module, instantiated.instance);\n");
    out.push_str("}\n\n");
    out.push_str("export async function instantiateFromUrl(url = wasmFileName, imports = {}) {\n");
    out.push_str("  const response = await fetch(url);\n");
    out.push_str("  if (typeof WebAssembly.instantiateStreaming === \"function\") {\n");
    out.push_str("    try {\n");
    out.push_str("      return await instantiateStreaming(response.clone(), imports);\n");
    out.push_str("    } catch (_error) {}\n");
    out.push_str("  }\n");
    out.push_str("  const bytes = await response.arrayBuffer();\n");
    out.push_str("  return instantiate(bytes, imports);\n");
    out.push_str("}\n\n");
    out.push_str("export async function start(source, imports = {}) {\n");
    out.push_str("  const instantiated = await instantiate(source, imports);\n");
    out.push_str("  const startFn = instantiated.exports._start;\n");
    out.push_str("  if (typeof startFn !== \"function\") {\n");
    out.push_str("    throw new TypeError(\"Export _start is missing or not a function\");\n");
    out.push_str("  }\n");
    out.push_str("  startFn();\n");
    out.push_str("  return instantiated;\n");
    out.push_str("}\n");
    out
}

fn quote_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}
