//! Static linker for Halcyon-compiled binaries.
//!
//! This module links two or more binaries that were produced by this compiler,
//! and emits a single final binary.
//!
//! # Input requirements
//! - Each input binary must include the lowered-module metadata custom section
//!   (`halcyon.asm.v1`).
//! - Each input binary must include the type-signature custom section
//!   (`type_signature`).
//!
//! These sections are emitted by the compiler backend and are used to recover
//! enough semantic information to perform static linking.
//!
//! # Execution order
//! Start-function execution order is **explicit and positional**:
//! inputs are initialized in the exact order provided to [`link_binaries`] or
//! [`link_artifacts`].
//!
//! The linker does not reorder inputs based on dependency analysis.
//!
//! # What is merged
//! - global imports/definitions
//! - function imports/definitions
//! - memory flag (at most one input may define memory)
//! - type-signature metadata
//!
//! # Relinkable outputs
//! Linked outputs are re-linkable by default. The linker re-emits both custom
//! metadata sections and uses qualified export names unless configured
//! otherwise.

use indexmap::{
    IndexMap,
    IndexSet,
};

use crate::asm::custom_section::TypeSignatureSection;
use crate::asm::module_section::LoweredModuleSection;
use crate::asm::{
    self,
    ExportPolicy,
    Function,
    FunctionImport,
    Instruction,
    Module,
    SourceOrigin,
    Type,
};
use crate::ir::Path;
use crate::types::TypeDefinition;
use crate::{
    Artifact,
    FileLogger,
    WithContext,
};

#[derive(Debug, Clone)]
/// Controls how input modules are linked into one final module.
pub struct LinkOptions {
    /// Name of the final linked module.
    pub module_name: String,

    /// If true, unresolved global imports fail linking.
    ///
    /// If false, unresolved global imports are preserved as imports in the
    /// linked output.
    pub strict: bool,

    /// Global export naming policy for the linked output.
    pub export_policy: ExportPolicy,
}

impl Default for LinkOptions {
    fn default() -> Self {
        Self {
            module_name: "linked".to_string(),
            strict: true,
            export_policy: ExportPolicy::Qualified,
        }
    }
}

#[derive(Debug, Clone)]
/// Errors produced by static linking.
pub enum LinkError {
    /// No inputs were provided.
    EmptyInput,

    /// An input could not be parsed as valid wasm.
    WasmParse { index: usize, message: String },

    /// Input is missing lowered-module metadata.
    MissingLoweredModuleSection { index: usize },

    /// Input has malformed lowered-module metadata.
    InvalidLoweredModuleSection { index: usize },

    /// Input is missing type-signature metadata.
    MissingTypeSignatureSection { index: usize },

    /// Input has malformed type-signature metadata.
    InvalidTypeSignatureSection { index: usize },

    /// Multiple inputs declare the same module name.
    DuplicateModuleName { name: String },

    /// Multiple inputs define the same global.
    DuplicateGlobal { path: Path },

    /// Multiple inputs define the same function.
    DuplicateFunction { path: Path },

    /// Multiple conflicting imports for the same function path.
    DuplicateFunctionImport { path: Path },

    /// A function path is both imported and defined.
    FunctionImportConflict { path: Path },

    /// A resolved global import type does not match the provider type.
    GlobalTypeMismatch {
        path: Path,
        expected: Type,
        found: Type,
    },

    /// A global import was not resolved (in strict mode).
    UnresolvedGlobalImport { path: Path },

    /// More than one input defines linear memory.
    ConflictingMemoryDefinitions,

    /// Type signatures disagree for the same named type.
    SignatureTypeConflict { path: Path },

    /// Type schemes disagree for the same named term.
    SignatureTermConflict { path: Path },

}

impl LinkError {
    pub fn report(
        self,
        logger: &mut FileLogger,
    ) {
        match self {
            Self::EmptyInput => {
                logger.error("Cannot link an empty set of binaries").done();
            }
            Self::WasmParse { index, message } => {
                logger
                    .error(format!("Failed to parse binary #{index}"))
                    .note(message)
                    .done();
            }
            Self::MissingLoweredModuleSection { index } => {
                logger
                    .error(format!(
                        "Binary #{index} is missing `{}` metadata",
                        LoweredModuleSection::NAME
                    ))
                    .done();
            }
            Self::InvalidLoweredModuleSection { index } => {
                logger
                    .error(format!(
                        "Binary #{index} contains invalid `{}` metadata",
                        LoweredModuleSection::NAME
                    ))
                    .done();
            }
            Self::MissingTypeSignatureSection { index } => {
                logger
                    .error(format!(
                        "Binary #{index} is missing `{}` metadata",
                        TypeSignatureSection::NAME
                    ))
                    .done();
            }
            Self::InvalidTypeSignatureSection { index } => {
                logger
                    .error(format!(
                        "Binary #{index} contains invalid `{}` metadata",
                        TypeSignatureSection::NAME
                    ))
                    .done();
            }
            Self::DuplicateModuleName { name } => {
                logger
                    .error(format!("Multiple binaries define module `{name}`"))
                    .done();
            }
            Self::DuplicateGlobal { path } => {
                logger
                    .error(format!("Multiple binaries define global `{path}`"))
                    .done();
            }
            Self::DuplicateFunction { path } => {
                logger
                    .error(format!("Multiple binaries define function `{path}`"))
                    .done();
            }
            Self::DuplicateFunctionImport { path } => {
                logger
                    .error(format!("Conflicting function imports for `{path}`"))
                    .done();
            }
            Self::FunctionImportConflict { path } => {
                logger
                    .error(format!("`{path}` is both an imported and defined function"))
                    .done();
            }
            Self::GlobalTypeMismatch {
                path,
                expected,
                found,
            } => {
                logger
                    .error(format!("Global import `{path}` has conflicting types"))
                    .note(format!("Imported as `{found}`"))
                    .note(format!("Provided as `{expected}`"))
                    .done();
            }
            Self::UnresolvedGlobalImport { path } => {
                logger
                    .error(format!("Unresolved global import `{path}`"))
                    .done();
            }
            Self::ConflictingMemoryDefinitions => {
                logger
                    .error("Linked binaries define more than one memory")
                    .done();
            }
            Self::SignatureTypeConflict { path } => {
                logger
                    .error(format!(
                        "Conflicting type signature definitions for `{path}`"
                    ))
                    .done();
            }
            Self::SignatureTermConflict { path } => {
                logger
                    .error(format!(
                        "Conflicting term signature definitions for `{path}`"
                    ))
                    .done();
            }
        }
    }
}

#[derive(Debug, Clone)]
struct LinkInput {
    module: Module,
    signature: TypeSignatureSection,
}

/// Link in-memory artifacts into one new linked artifact.
///
/// Input order is preserved and determines initialization order.
///
/// Returns `None` after reporting an error diagnostic to `logger` when linking
/// fails.
pub fn link_artifacts(
    artifacts: &[Artifact],
    options: LinkOptions,
    logger: &mut FileLogger,
) -> Option<Artifact> {
    let binaries = artifacts
        .iter()
        .map(|artifact| artifact.binary.as_slice())
        .collect::<Vec<_>>();
    match link_binaries(&binaries, options) {
        Ok(artifact) => Some(artifact),
        Err(error) => {
            error.report(logger);
            None
        }
    }
}

/// Link compiled wasm binaries into one new linked artifact.
///
/// Input order is preserved and determines initialization order.
///
/// Each input must contain linker metadata custom sections emitted by this
/// compiler.
pub fn link_binaries<B: AsRef<[u8]>>(
    binaries: &[B],
    options: LinkOptions,
) -> Result<Artifact, LinkError> {
    if binaries.is_empty() {
        return Err(LinkError::EmptyInput);
    }

    let inputs = binaries
        .iter()
        .enumerate()
        .map(|(index, binary)| decode_link_input(binary.as_ref(), index))
        .collect::<Result<Vec<_>, _>>()?;

    let mut linked = merge_inputs(&inputs, &options)?;
    linked.sig = merge_type_signatures(&inputs)?;
    linked.export_policy = options.export_policy;
    let encoded = asm::encode(linked);

    Ok(Artifact {
        module_name: options.module_name,
        ir_module: None,
        binary: encoded.binary,
        source_map: encoded.source_map,
    })
}

fn decode_link_input(
    binary: &[u8],
    index: usize,
) -> Result<LinkInput, LinkError> {
    let mut module = None;
    let mut signature = None;
    let mut saw_module_section = false;
    let mut saw_signature_section = false;

    for payload in wasmparser::Parser::new(0).parse_all(binary) {
        let payload = payload.map_err(|error| {
            LinkError::WasmParse {
                index,
                message: error.message().to_string(),
            }
        })?;
        if let wasmparser::Payload::CustomSection(reader) = payload {
            if reader.name() == LoweredModuleSection::NAME {
                saw_module_section = true;
                module = LoweredModuleSection::decode_data_slice(reader.data());
            } else if reader.name() == TypeSignatureSection::NAME {
                saw_signature_section = true;
                signature = TypeSignatureSection::decode_data_slice(reader.data());
            }
        }
    }

    let mut module = if saw_module_section {
        module.ok_or(LinkError::InvalidLoweredModuleSection { index })?
    } else {
        return Err(LinkError::MissingLoweredModuleSection { index });
    };
    let signature = if saw_signature_section {
        signature.ok_or(LinkError::InvalidTypeSignatureSection { index })?
    } else {
        return Err(LinkError::MissingTypeSignatureSection { index });
    };

    if module.name.is_empty() {
        return Err(LinkError::InvalidLoweredModuleSection { index });
    }
    module.sig = signature.clone();

    Ok(LinkInput { module, signature })
}

fn merge_inputs(
    inputs: &[LinkInput],
    options: &LinkOptions,
) -> Result<Module, LinkError> {
    let mut merged = Module::new(options.module_name.clone());

    let normalized_modules = inputs
        .iter()
        .map(|input| namespace_temporary_paths(&input.module))
        .collect::<Vec<_>>();

    let mut module_names = IndexSet::new();
    let mut provided_globals = IndexMap::new();
    let mut memory_count = 0usize;

    for module in normalized_modules.iter() {
        if !module_names.insert(module.name.clone()) {
            return Err(LinkError::DuplicateModuleName {
                name: module.name.clone(),
            });
        }

        if module.has_memory {
            memory_count += 1;
        }

        for (path, type_) in module.globals.iter() {
            if provided_globals
                .insert(path.clone(), type_.clone())
                .is_some()
            {
                return Err(LinkError::DuplicateGlobal { path: path.clone() });
            }
        }
    }

    if memory_count > 1 {
        return Err(LinkError::ConflictingMemoryDefinitions);
    }
    merged.has_memory = memory_count == 1;

    for module in normalized_modules.iter() {
        for (path, import_type) in module.imports.iter() {
            if let Some(provided_type) = provided_globals.get(path) {
                if !provided_type.structural_eq(import_type) {
                    return Err(LinkError::GlobalTypeMismatch {
                        path: path.clone(),
                        expected: provided_type.clone(),
                        found: import_type.clone(),
                    });
                }
                continue;
            }

            if options.strict {
                return Err(LinkError::UnresolvedGlobalImport { path: path.clone() });
            }

            if let Some(existing) = merged.imports.get(path) {
                if !existing.structural_eq(import_type) {
                    return Err(LinkError::GlobalTypeMismatch {
                        path: path.clone(),
                        expected: existing.clone(),
                        found: import_type.clone(),
                    });
                }
            } else {
                merged.imports.insert(path.clone(), import_type.clone());
            }
        }
    }

    for module in normalized_modules.iter() {
        for (path, type_) in module.globals.iter() {
            if merged.globals.insert(path.clone(), type_.clone()).is_some() {
                return Err(LinkError::DuplicateGlobal { path: path.clone() });
            }
        }
    }

    for module in normalized_modules.iter() {
        for (path, function) in module.functions.iter() {
            if merged.function_imports.contains_key(path) {
                return Err(LinkError::FunctionImportConflict { path: path.clone() });
            }
            if merged
                .functions
                .insert(path.clone(), function.clone())
                .is_some()
            {
                return Err(LinkError::DuplicateFunction { path: path.clone() });
            }
        }
    }

    for module in normalized_modules.iter() {
        for (path, function_import) in module.function_imports.iter() {
            if merged.functions.contains_key(path) {
                return Err(LinkError::FunctionImportConflict { path: path.clone() });
            }
            if let Some(existing) = merged.function_imports.get(path) {
                if !function_import_eq(existing, function_import) {
                    return Err(LinkError::DuplicateFunctionImport { path: path.clone() });
                }
            } else {
                merged
                    .function_imports
                    .insert(path.clone(), function_import.clone());
            }
        }
    }

    for module in normalized_modules.iter() {
        for (file_name, record) in module.source_files.iter() {
            merged
                .source_files
                .entry(file_name.clone())
                .or_insert_with(|| record.clone());
        }
    }

    let start_path = unique_start_path(&merged, &options.module_name);
    let start_ops = normalized_modules
        .iter()
        .map(|module| Instruction::Call(module.start.clone()))
        .collect::<Vec<_>>();
    let start_op_origins = normalized_modules
        .iter()
        .map(start_origin)
        .collect::<Vec<_>>();

    merged.functions.insert(
        start_path.clone(),
        Function {
            parameters: IndexMap::new(),
            returns: Vec::new(),
            variables: IndexMap::new(),
            ops: start_ops,
            op_origins: start_op_origins,
        },
    );
    merged.start = start_path;

    Ok(merged)
}

fn start_origin(module: &Module) -> Option<SourceOrigin> {
    module
        .functions
        .get(&module.start)
        .and_then(|function| function.op_origins.iter().flatten().next())
        .cloned()
}

fn function_import_eq(
    left: &FunctionImport,
    right: &FunctionImport,
) -> bool {
    left.module == right.module
        && left.name == right.name
        && left.params.as_ref() == right.params.as_ref()
        && left.results.as_ref() == right.results.as_ref()
}

fn unique_start_path(
    module: &Module,
    module_name: &str,
) -> Path {
    let mut index = 0usize;
    loop {
        let suffix = if index == 0 {
            "[linked_init]".to_string()
        } else {
            format!("[linked_init#{index}]")
        };
        let candidate = Path::new(module_name, suffix);
        if !module.functions.contains_key(&candidate)
            && !module.function_imports.contains_key(&candidate)
        {
            return candidate;
        }
        index += 1;
    }
}

fn merge_type_signatures(inputs: &[LinkInput]) -> Result<TypeSignatureSection, LinkError> {
    let mut merged = TypeSignatureSection::default();
    let mut imported_types = IndexSet::new();

    for input in inputs {
        let signature = &input.signature;

        for (path, definition) in signature.defined_types.iter() {
            if let Some(existing) = merged.defined_types.get(path) {
                if !type_definition_eq(existing, definition) {
                    return Err(LinkError::SignatureTypeConflict { path: path.clone() });
                }
            } else {
                merged
                    .defined_types
                    .insert(path.clone(), definition.clone());
            }
        }

        for (path, scheme) in signature.defined_terms.iter() {
            if let Some(existing) = merged.defined_terms.get(path) {
                if existing != scheme {
                    return Err(LinkError::SignatureTermConflict { path: path.clone() });
                }
            } else {
                merged.defined_terms.insert(path.clone(), scheme.clone());
            }
        }

        for path in signature.imported_types.iter() {
            if !merged.defined_types.contains_key(path) {
                imported_types.insert(path.clone());
            }
        }
    }

    merged.imported_types = imported_types
        .into_iter()
        .filter(|path| !merged.defined_types.contains_key(path))
        .collect();
    merged.rebuild_index_map_for_encoding();
    Ok(merged)
}

fn type_definition_eq(
    left: &TypeDefinition,
    right: &TypeDefinition,
) -> bool {
    left.parameters == right.parameters
        && left.parameter_kinds == right.parameter_kinds
        && left.body == right.body
        && left.kind == right.kind
}

fn namespace_temporary_paths(module: &Module) -> Module {
    let temp_major = format!("[temp:{}]", module.name);
    let remap = |path: &Path| remap_temporary_path(path, &temp_major);

    let imports = module
        .imports
        .iter()
        .map(|(path, type_)| (remap(path), type_.clone()))
        .collect();
    let globals = module
        .globals
        .iter()
        .map(|(path, type_)| (remap(path), type_.clone()))
        .collect();
    let functions = module
        .functions
        .iter()
        .map(|(path, function)| (remap(path), remap_function_paths(function, &temp_major)))
        .collect();
    let function_imports = module
        .function_imports
        .iter()
        .map(|(path, function_import)| (remap(path), function_import.clone()))
        .collect();

    Module {
        name: module.name.clone(),
        imports,
        globals,
        functions,
        function_imports,
        has_memory: module.has_memory,
        sig: module.sig.clone(),
        export_policy: module.export_policy,
        start: remap(&module.start),
        source_files: module.source_files.clone(),
        closure_counter: module.closure_counter,
        source_file_lookup: module.source_file_lookup.clone(),
    }
}

fn remap_function_paths(
    function: &Function,
    temp_major: &str,
) -> Function {
    let remap = |path: &Path| remap_temporary_path(path, temp_major);

    let parameters = function
        .parameters
        .iter()
        .map(|(path, type_)| (remap(path), type_.clone()))
        .collect();
    let variables = function
        .variables
        .iter()
        .map(|(path, type_)| (remap(path), type_.clone()))
        .collect();
    let ops = function
        .ops
        .iter()
        .map(|instruction| remap_instruction_paths(instruction, temp_major))
        .collect();

    Function {
        parameters,
        returns: function.returns.clone(),
        variables,
        ops,
        op_origins: function.op_origins.clone(),
    }
}

fn remap_instruction_paths(
    instruction: &Instruction,
    temp_major: &str,
) -> Instruction {
    let remap = |path: &Path| remap_temporary_path(path, temp_major);
    match instruction {
        Instruction::Set(path) => Instruction::Set(remap(path)),
        Instruction::Get(path) => Instruction::Get(remap(path)),
        Instruction::Func(path) => Instruction::Func(remap(path)),
        Instruction::Call(path) => Instruction::Call(remap(path)),
        _ => instruction.clone(),
    }
}

fn remap_temporary_path(
    path: &Path,
    temp_major: &str,
) -> Path {
    if path.major == "[temp]" {
        Path::new(temp_major, path.minor.clone())
    } else {
        path.clone()
    }
}

#[cfg(test)]
mod tests;
