use super::{
    AbiType,
    BindingSpec,
    ExportKind,
};

pub(crate) fn emit(spec: &BindingSpec) -> String {
    let mut out = String::new();
    out.push_str("export type ExternRef = unknown;\n\n");
    out.push_str("export interface HalcyonImports {\n");
    out.push_str("  [module_name: string]: Record<string, unknown>;\n");
    for module in spec.imports.iter() {
        out.push_str("  ");
        out.push_str(&quote_string(&module.module));
        out.push_str(": {\n");
        if module.functions.is_empty() && module.globals.is_empty() {
            out.push_str("    [name: string]: unknown;\n");
        }
        for function in module.functions.iter() {
            out.push_str("    ");
            out.push_str(&quote_string(&function.import_name));
            out.push_str(": ");
            out.push_str(&function_type(&function.parameters, &function.results));
            out.push_str(";\n");
        }
        for global in module.globals.iter() {
            out.push_str("    ");
            out.push_str(&quote_string(&global.import_name));
            if global.mutable {
                out.push_str(": WebAssembly.Global;\n");
            } else {
                out.push_str(": ");
                out.push_str(global.type_.ts_name());
                out.push_str(";\n");
            }
        }
        out.push_str("  };\n");
    }
    out.push_str("}\n\n");

    out.push_str("export interface HalcyonExports extends WebAssembly.Exports {\n");
    for export in spec.exports.iter() {
        out.push_str("  ");
        out.push_str(&quote_string(&export.name));
        out.push_str(": ");
        match export.kind {
            ExportKind::Function => {
                out.push_str(&function_type(&export.parameters, &export.results));
            }
            ExportKind::Global => {
                if export.mutable {
                    out.push_str("WebAssembly.Global");
                } else {
                    out.push_str(export.value_type.unwrap_or(AbiType::ExternRef).ts_name());
                }
            }
            ExportKind::Memory => out.push_str("WebAssembly.Memory"),
            ExportKind::Table => out.push_str("WebAssembly.Table"),
            ExportKind::Tag => out.push_str("unknown"),
        }
        out.push_str(";\n");
    }
    out.push_str("}\n\n");

    out.push_str("export interface InstantiatedModule {\n");
    out.push_str("  module: WebAssembly.Module;\n");
    out.push_str("  instance: WebAssembly.Instance;\n");
    out.push_str("  exports: HalcyonExports;\n");
    out.push_str("}\n\n");

    out.push_str("export const wasmFileName: string;\n");
    out.push_str(
        "export function validateImports(imports?: Record<string, unknown>): HalcyonImports;\n",
    );
    out.push_str(
        "export function instantiate(source: BufferSource | WebAssembly.Module, imports?: Record<string, unknown>): Promise<InstantiatedModule>;\n",
    );
    out.push_str(
        "export function instantiateStreaming(source: Response | Promise<Response>, imports?: Record<string, unknown>): Promise<InstantiatedModule>;\n",
    );
    out.push_str(
        "export function instantiateFromUrl(url?: string | URL, imports?: Record<string, unknown>): Promise<InstantiatedModule>;\n",
    );
    out.push_str(
        "export function start(source: BufferSource | WebAssembly.Module, imports?: Record<string, unknown>): Promise<InstantiatedModule>;\n",
    );

    out
}

fn function_type(
    parameters: &[AbiType],
    results: &[AbiType],
) -> String {
    let parameters = parameters
        .iter()
        .enumerate()
        .map(|(index, type_)| format!("arg{index}: {}", type_.ts_name()))
        .collect::<Vec<_>>()
        .join(", ");
    let result_type = match results {
        [] => "void".to_string(),
        [result] => result.ts_name().to_string(),
        many => {
            let tuple = many
                .iter()
                .map(|type_| type_.ts_name())
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{tuple}]")
        }
    };
    format!("({parameters}) => {result_type}")
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
