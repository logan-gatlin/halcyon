mod emit_dts;
mod emit_js;
mod emit_json;
mod types;

pub use types::AbiType;

use std::fmt;

use indexmap::IndexMap;
use wasmparser::{
    ExternalKind,
    Parser,
    Payload,
    TypeRef,
};

use crate::asm::custom_section::TypeSignatureSection;
use crate::asm::module_section::LoweredModuleSection;
use crate::asm::{
    ExportPolicy,
    Function,
    FunctionImport,
    Module,
    Type,
};
use crate::ir::Path;

#[derive(Debug, Clone, serde::Serialize)]
pub struct BindingSpec {
    pub module_name: String,
    pub wasm_file_name: String,
    pub imports: Box<[ImportModule]>,
    pub exports: Box<[ExportedItem]>,
    pub signature: SignatureSummary,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportModule {
    pub module: String,
    pub functions: Box<[ImportedFunction]>,
    pub globals: Box<[ImportedGlobal]>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportedFunction {
    pub local_name: String,
    pub import_name: String,
    pub parameters: Box<[AbiType]>,
    pub results: Box<[AbiType]>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportedGlobal {
    pub local_name: String,
    pub import_name: String,
    #[serde(rename = "type")]
    pub type_: AbiType,
    pub mutable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportKind {
    Function,
    Global,
    Memory,
    Table,
    Tag,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportedItem {
    pub name: String,
    pub kind: ExportKind,
    pub index: u32,
    pub local_name: Option<String>,
    pub parameters: Box<[AbiType]>,
    pub results: Box<[AbiType]>,
    #[serde(rename = "type")]
    pub value_type: Option<AbiType>,
    pub mutable: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SignatureSummary {
    pub imported_types: Box<[String]>,
    pub defined_type_count: usize,
    pub defined_term_count: usize,
}

#[derive(Debug, Clone)]
pub struct GeneratedBindings {
    pub spec: BindingSpec,
    pub javascript: String,
    pub typescript: String,
    pub json: String,
}

#[derive(Debug, Clone, Default)]
pub struct GenerateBindingsOptions {
    pub module_name: Option<String>,
    pub wasm_file_name: Option<String>,
}

#[derive(Debug)]
pub enum BindingGenerationError {
    MissingMetadata(&'static str),
    InvalidMetadata(&'static str),
    WasmParse(String),
    MetadataMismatch(String),
    Json(serde_json::Error),
}

impl fmt::Display for BindingGenerationError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::MissingMetadata(name) => {
                write!(f, "missing metadata custom section `{name}`")
            }
            Self::InvalidMetadata(name) => {
                write!(f, "invalid metadata custom section `{name}`")
            }
            Self::WasmParse(message) => write!(f, "failed to parse wasm: {message}"),
            Self::MetadataMismatch(message) => write!(f, "metadata mismatch: {message}"),
            Self::Json(error) => write!(f, "failed to serialize bindings JSON: {error}"),
        }
    }
}

impl std::error::Error for BindingGenerationError {
}

impl From<serde_json::Error> for BindingGenerationError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub fn generate_js_bindings(
    binary: &[u8],
    options: GenerateBindingsOptions,
) -> Result<GeneratedBindings, BindingGenerationError> {
    let mut spec = BindingSpec::from_binary(binary)?;
    let module_name_overridden = options.module_name.is_some();
    if let Some(module_name) = options.module_name {
        spec.module_name = module_name;
    }
    if let Some(wasm_file_name) = options.wasm_file_name {
        spec.wasm_file_name = wasm_file_name;
    } else if module_name_overridden {
        spec.wasm_file_name = format!("{}.wasm", spec.module_name);
    }

    let json = emit_json::emit(&spec)?;
    let javascript = emit_js::emit(&spec, &json);
    let typescript = emit_dts::emit(&spec);

    Ok(GeneratedBindings {
        spec,
        javascript,
        typescript,
        json,
    })
}

impl BindingSpec {
    pub fn from_binary(binary: &[u8]) -> Result<Self, BindingGenerationError> {
        let ParsedWasm {
            module,
            signature,
            function_imports,
            global_import_count,
            exports,
        } = parse_binary(binary)?;

        validate_imports(&module, &function_imports, global_import_count)?;

        let FunctionIndexData {
            by_index: functions_by_index,
            by_path: function_index_by_path,
        } = function_index_data(&module);
        validate_exports(&module, &exports, &function_index_by_path)?;

        let globals_by_index = global_index_data(&module);

        Ok(Self {
            module_name: module.name.clone(),
            wasm_file_name: format!("{}.wasm", module.name),
            imports: import_modules(&module),
            exports: exports_with_types(&exports, &functions_by_index, &globals_by_index)?,
            signature: SignatureSummary {
                imported_types: signature
                    .imported_types
                    .iter()
                    .map(|path| format!("{path}"))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                defined_type_count: signature.defined_types.len(),
                defined_term_count: signature.defined_terms.len(),
            },
        })
    }
}

#[derive(Debug, Clone)]
struct ParsedWasm {
    module: Module,
    signature: TypeSignatureSection,
    function_imports: Vec<WasmFunctionImport>,
    global_import_count: usize,
    exports: Vec<WasmExport>,
}

fn parse_binary(binary: &[u8]) -> Result<ParsedWasm, BindingGenerationError> {
    let mut saw_lowered_module_section = false;
    let mut saw_signature_section = false;
    let mut lowered_module = None;
    let mut signature = None;
    let mut function_imports = Vec::new();
    let mut global_import_count = 0usize;
    let mut exports = Vec::new();

    for payload in Parser::new(0).parse_all(binary) {
        let payload = payload
            .map_err(|error| BindingGenerationError::WasmParse(error.message().to_string()))?;
        match payload {
            Payload::CustomSection(reader) => {
                if reader.name() == LoweredModuleSection::NAME {
                    saw_lowered_module_section = true;
                    if lowered_module.is_none() {
                        lowered_module = LoweredModuleSection::decode_data_slice(reader.data());
                    }
                }
                if reader.name() == TypeSignatureSection::NAME {
                    saw_signature_section = true;
                    if signature.is_none() {
                        signature = TypeSignatureSection::decode_data_slice(reader.data());
                    }
                }
            }
            Payload::ImportSection(reader) => {
                for import in reader.into_imports() {
                    let import = import.map_err(|error| {
                        BindingGenerationError::WasmParse(error.message().to_string())
                    })?;
                    match import.ty {
                        TypeRef::Func(_) | TypeRef::FuncExact(_) => {
                            function_imports.push(WasmFunctionImport {
                                module: import.module.to_string(),
                                name: import.name.to_string(),
                            });
                        }
                        TypeRef::Global(_) => {
                            global_import_count += 1;
                        }
                        _ => {}
                    }
                }
            }
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export.map_err(|error| {
                        BindingGenerationError::WasmParse(error.message().to_string())
                    })?;
                    exports.push(WasmExport {
                        name: export.name.to_string(),
                        kind: export.kind,
                        index: export.index,
                    });
                }
            }
            _ => {}
        }
    }

    let module = if saw_lowered_module_section {
        lowered_module.ok_or(BindingGenerationError::InvalidMetadata(
            LoweredModuleSection::NAME,
        ))?
    } else {
        return Err(BindingGenerationError::MissingMetadata(
            LoweredModuleSection::NAME,
        ));
    };

    let signature = if saw_signature_section {
        signature.ok_or(BindingGenerationError::InvalidMetadata(
            TypeSignatureSection::NAME,
        ))?
    } else {
        return Err(BindingGenerationError::MissingMetadata(
            TypeSignatureSection::NAME,
        ));
    };

    Ok(ParsedWasm {
        module,
        signature,
        function_imports,
        global_import_count,
        exports,
    })
}

#[derive(Debug, Clone)]
struct WasmFunctionImport {
    module: String,
    name: String,
}

#[derive(Debug, Clone)]
struct WasmExport {
    name: String,
    kind: ExternalKind,
    index: u32,
}

fn validate_imports(
    module: &Module,
    function_imports: &[WasmFunctionImport],
    global_import_count: usize,
) -> Result<(), BindingGenerationError> {
    if function_imports.len() != module.function_imports.len() {
        return Err(BindingGenerationError::MetadataMismatch(format!(
            "function import count mismatch: metadata has {}, wasm has {}",
            module.function_imports.len(),
            function_imports.len()
        )));
    }

    for ((path, expected), actual) in module.function_imports.iter().zip(function_imports.iter()) {
        if expected.module != actual.module || expected.name != actual.name {
            return Err(BindingGenerationError::MetadataMismatch(format!(
                "function import `{path}` mismatch: metadata expects {}.{}, wasm has {}.{}",
                expected.module, expected.name, actual.module, actual.name
            )));
        }
    }

    if global_import_count != module.imports.len() {
        return Err(BindingGenerationError::MetadataMismatch(format!(
            "global import count mismatch: metadata has {}, wasm has {}",
            module.imports.len(),
            global_import_count
        )));
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct FunctionIndexData {
    by_index: IndexMap<u32, FunctionIndexEntry>,
    by_path: IndexMap<Path, u32>,
}

#[derive(Debug, Clone)]
struct FunctionIndexEntry {
    local_name: String,
    parameters: Box<[AbiType]>,
    results: Box<[AbiType]>,
}

fn function_index_data(module: &Module) -> FunctionIndexData {
    let mut by_index = IndexMap::new();
    let mut by_path = IndexMap::new();
    let mut index = 0u32;

    for (path, function_import) in module.function_imports.iter() {
        by_path.insert(path.clone(), index);
        by_index.insert(
            index,
            function_index_entry_for_import(path, function_import),
        );
        index += 1;
    }

    for (path, function) in module.functions.iter() {
        by_path.insert(path.clone(), index);
        by_index.insert(index, function_index_entry_for_function(path, function));
        index += 1;
    }

    FunctionIndexData { by_index, by_path }
}

fn function_index_entry_for_import(
    path: &Path,
    function_import: &FunctionImport,
) -> FunctionIndexEntry {
    FunctionIndexEntry {
        local_name: format!("{path}"),
        parameters: lowered_types(&function_import.params),
        results: lowered_types(&function_import.results),
    }
}

fn function_index_entry_for_function(
    path: &Path,
    function: &Function,
) -> FunctionIndexEntry {
    FunctionIndexEntry {
        local_name: format!("{path}"),
        parameters: function
            .parameters
            .values()
            .map(AbiType::from_lowered)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        results: function
            .returns
            .iter()
            .map(AbiType::from_lowered)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    }
}

fn lowered_types(types: &[Type]) -> Box<[AbiType]> {
    types
        .iter()
        .map(AbiType::from_lowered)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

#[derive(Debug, Clone)]
struct GlobalIndexEntry {
    local_name: String,
    type_: AbiType,
    mutable: bool,
}

fn global_index_data(module: &Module) -> IndexMap<u32, GlobalIndexEntry> {
    let mut globals = IndexMap::new();
    let mut index = 0u32;

    for (path, type_) in module.imports.iter() {
        globals.insert(
            index,
            GlobalIndexEntry {
                local_name: format!("{path}"),
                type_: AbiType::from_lowered(type_),
                mutable: true,
            },
        );
        index += 1;
    }

    for (path, type_) in module.globals.iter() {
        globals.insert(
            index,
            GlobalIndexEntry {
                local_name: format!("{path}"),
                type_: AbiType::from_lowered(type_),
                mutable: true,
            },
        );
        index += 1;
    }

    globals
}

#[derive(Debug, Clone)]
struct ExpectedExport {
    name: String,
    kind: ExternalKind,
    index: u32,
}

fn expected_exports(
    module: &Module,
    function_index_by_path: &IndexMap<Path, u32>,
) -> Result<Vec<ExpectedExport>, BindingGenerationError> {
    let start_index = function_index_by_path
        .get(&module.start)
        .copied()
        .ok_or_else(|| {
            BindingGenerationError::MetadataMismatch(format!(
                "missing start function `{}` in metadata",
                module.start
            ))
        })?;

    let mut exports = vec![ExpectedExport {
        name: "_start".to_string(),
        kind: ExternalKind::Func,
        index: start_index,
    }];

    if module.has_memory {
        exports.push(ExpectedExport {
            name: "memory".to_string(),
            kind: ExternalKind::Memory,
            index: 0,
        });
    }

    let global_import_count = module.imports.len() as u32;
    for (offset, (path, _)) in module.globals.iter().enumerate() {
        let Some(export_name) = global_export_name(module.export_policy, path) else {
            continue;
        };
        exports.push(ExpectedExport {
            name: export_name,
            kind: ExternalKind::Global,
            index: global_import_count + offset as u32,
        });
    }

    Ok(exports)
}

fn validate_exports(
    module: &Module,
    exports: &[WasmExport],
    function_index_by_path: &IndexMap<Path, u32>,
) -> Result<(), BindingGenerationError> {
    for expected in expected_exports(module, function_index_by_path)? {
        let Some(actual) = exports.iter().find(|item| item.name == expected.name) else {
            return Err(BindingGenerationError::MetadataMismatch(format!(
                "missing export `{}` expected by metadata",
                expected.name
            )));
        };
        if actual.kind != expected.kind || actual.index != expected.index {
            return Err(BindingGenerationError::MetadataMismatch(format!(
                "export `{}` mismatch: metadata expects {:?} index {}, wasm has {:?} index {}",
                expected.name, expected.kind, expected.index, actual.kind, actual.index
            )));
        }
    }

    Ok(())
}

fn global_export_name(
    export_policy: ExportPolicy,
    path: &Path,
) -> Option<String> {
    match export_policy {
        ExportPolicy::MinorOnly => Some(path.minor.clone()),
        ExportPolicy::Qualified => Some(format!("{path}")),
        ExportPolicy::None => None,
    }
}

fn import_modules(module: &Module) -> Box<[ImportModule]> {
    #[derive(Default)]
    struct Builder {
        functions: Vec<ImportedFunction>,
        globals: Vec<ImportedGlobal>,
    }

    let mut modules = IndexMap::<String, Builder>::new();

    for (path, function_import) in module.function_imports.iter() {
        let builder = modules.entry(function_import.module.clone()).or_default();
        builder.functions.push(ImportedFunction {
            local_name: format!("{path}"),
            import_name: function_import.name.clone(),
            parameters: lowered_types(&function_import.params),
            results: lowered_types(&function_import.results),
        });
    }

    for (path, type_) in module.imports.iter() {
        let builder = modules.entry(path.major.clone()).or_default();
        builder.globals.push(ImportedGlobal {
            local_name: format!("{path}"),
            import_name: path.minor.clone(),
            type_: AbiType::from_lowered(type_),
            mutable: true,
        });
    }

    modules
        .into_iter()
        .map(|(module, builder)| {
            ImportModule {
                module,
                functions: builder.functions.into_boxed_slice(),
                globals: builder.globals.into_boxed_slice(),
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn exports_with_types(
    exports: &[WasmExport],
    functions_by_index: &IndexMap<u32, FunctionIndexEntry>,
    globals_by_index: &IndexMap<u32, GlobalIndexEntry>,
) -> Result<Box<[ExportedItem]>, BindingGenerationError> {
    exports
        .iter()
        .map(|export| {
            Ok(match export.kind {
                ExternalKind::Func => {
                    let Some(function) = functions_by_index.get(&export.index) else {
                        return Err(BindingGenerationError::MetadataMismatch(format!(
                            "exported function index {} does not exist in metadata",
                            export.index
                        )));
                    };
                    ExportedItem {
                        name: export.name.clone(),
                        kind: ExportKind::Function,
                        index: export.index,
                        local_name: Some(function.local_name.clone()),
                        parameters: function.parameters.clone(),
                        results: function.results.clone(),
                        value_type: None,
                        mutable: false,
                    }
                }
                ExternalKind::Global => {
                    let Some(global) = globals_by_index.get(&export.index) else {
                        return Err(BindingGenerationError::MetadataMismatch(format!(
                            "exported global index {} does not exist in metadata",
                            export.index
                        )));
                    };
                    ExportedItem {
                        name: export.name.clone(),
                        kind: ExportKind::Global,
                        index: export.index,
                        local_name: Some(global.local_name.clone()),
                        parameters: Box::new([]),
                        results: Box::new([]),
                        value_type: Some(global.type_),
                        mutable: global.mutable,
                    }
                }
                ExternalKind::Memory => {
                    ExportedItem {
                        name: export.name.clone(),
                        kind: ExportKind::Memory,
                        index: export.index,
                        local_name: None,
                        parameters: Box::new([]),
                        results: Box::new([]),
                        value_type: None,
                        mutable: false,
                    }
                }
                ExternalKind::Table => {
                    ExportedItem {
                        name: export.name.clone(),
                        kind: ExportKind::Table,
                        index: export.index,
                        local_name: None,
                        parameters: Box::new([]),
                        results: Box::new([]),
                        value_type: None,
                        mutable: false,
                    }
                }
                ExternalKind::Tag => {
                    ExportedItem {
                        name: export.name.clone(),
                        kind: ExportKind::Tag,
                        index: export.index,
                        local_name: None,
                        parameters: Box::new([]),
                        results: Box::new([]),
                        value_type: None,
                        mutable: false,
                    }
                }
                ExternalKind::FuncExact => {
                    return Err(BindingGenerationError::MetadataMismatch(
                        "exact function exports are not supported".to_string(),
                    ));
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

#[cfg(test)]
mod tests;
