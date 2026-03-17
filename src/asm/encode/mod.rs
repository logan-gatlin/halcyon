use std::collections::{
    BTreeSet,
    HashMap,
};

mod instruction;
mod source_map;
mod type_section;
mod dwarf;

use super::module_section::LoweredModuleSection;
use super::resolve::ResolvedModule;
use super::*;
use instruction::encode_instruction;
use source_map::build_source_map_json;
use dwarf::build_dwarf_sections;
use type_section::{
    TypeSection,
    default_value,
};
use wasm_encoder::{
    EntityType,
    ExportKind,
    GlobalType,
    ImportSection,
    NameMap,
    NameSection,
    ProducersField,
    RefType,
};

/// Handles encode.
pub fn encode(asm_module: Module) -> EncodedModule {
    let resolved = resolve_module(asm_module)
        .and_then(|resolved| {
            verify_module(&resolved)?;
            Ok(resolved)
        })
        .unwrap_or_else(|error| panic!("{error}"));
    encode_resolved_module(resolved)
}

/// Handles encode resolved module.
fn encode_resolved_module(resolved: ResolvedModule) -> EncodedModule {
    let asm_module = &resolved.lowered;

    let mut name_section = NameSection::new();
    let mut global_names = NameMap::new();
    let mut type_section = TypeSection::new();
    let mut import_section = ImportSection::new();
    let mut function_section = wasm_encoder::FunctionSection::new();
    let mut table_section = wasm_encoder::TableSection::new();
    let mut memory_section = wasm_encoder::MemorySection::new();
    let mut global_section = wasm_encoder::GlobalSection::new();
    let mut export_section = wasm_encoder::ExportSection::new();
    let mut element_section = wasm_encoder::ElementSection::new();
    let mut code_section = wasm_encoder::CodeSection::new();
    let mut producer_section = wasm_encoder::ProducersSection::new();

    producer_section.field(
        "language",
        ProducersField::new().value("Halcyon", crate::COMPILER_VERSION_STRING),
    );

    let mut global_namespace = HashMap::new();
    let func_namespace = resolved
        .function_indices
        .iter()
        .map(|(path, index)| (path.clone(), *index))
        .collect::<HashMap<_, _>>();
    let mut referenced_funcs: BTreeSet<u32> = BTreeSet::new();

    name_section.module(&asm_module.name);

    for (idx, (path, fi)) in asm_module.function_imports.iter().enumerate() {
        let type_idx = type_section.new_function(&fi.params, &fi.results);
        import_section.import(&fi.module, &fi.name, EntityType::Function(type_idx));
        debug_assert_eq!(func_namespace.get(path), Some(&(idx as u32)));
    }

    let mut global_id = 0;
    for (name, type_) in asm_module.imports.iter() {
        import_section.import(
            &name.major,
            &name.minor,
            EntityType::Global(GlobalType {
                val_type: type_section.valtype_of(type_),
                mutable: true,
                shared: false,
            }),
        );
        global_namespace.insert(name, global_id);
        global_names.append(global_id, &format!("{name}"));
        global_id += 1;
    }

    for (name, type_) in asm_module.globals.iter() {
        let val_type = type_section.valtype_of(type_);
        global_section.global(
            GlobalType {
                val_type,
                mutable: true,
                shared: false,
            },
            &default_value(&val_type),
        );
        if let Some(export_name) = asm_module.export_policy.global_export_name(name) {
            export_section.export(&export_name, ExportKind::Global, global_id);
        }
        global_namespace.insert(name, global_id);
        global_names.append(global_id, &name.minor);
        global_id += 1;
    }

    if asm_module.has_memory {
        memory_section.memory(wasm_encoder::MemoryType {
            minimum: 1,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });
        export_section.export("memory", ExportKind::Memory, 0);
    }

    export_section.export("_start", ExportKind::Func, resolved.start_function_index);

    let mut function_operator_origins = Vec::new();
    for function in &resolved.functions {
        let parameter_types = function.parameters.values().cloned().collect::<Vec<_>>();
        function_section.function(type_section.new_function(&parameter_types, &function.returns));

        let mut function_body = wasm_encoder::Function::new_with_locals_types(
            function
                .variables
                .iter()
                .map(|(_, type_)| type_section.valtype_of(type_)),
        );
        let mut expanded_origins = Vec::new();

        for (op_index, op) in function.ops.iter().enumerate() {
            let emitted = encode_instruction(
                op,
                &mut type_section,
                &mut function_body,
                &mut referenced_funcs,
            );
            let origin = function.op_origins.get(op_index).cloned().unwrap_or(None);
            expanded_origins.extend(std::iter::repeat_n(origin, emitted));
        }

        function_operator_origins.push(expanded_origins);
        function_body.instruction(&wasm_encoder::Instruction::End);
        code_section.function(&function_body);
    }

    let referenced_funcs = referenced_funcs.into_iter().collect::<Vec<_>>();
    table_section.table(wasm_encoder::TableType {
        element_type: RefType::FUNCREF,
        table64: false,
        minimum: referenced_funcs.len() as u64,
        maximum: Some(referenced_funcs.len() as u64),
        shared: false,
    });
    element_section.declared(wasm_encoder::Elements::Functions(referenced_funcs.into()));

    let mut func_names = NameMap::new();
    for (path, &index) in &func_namespace {
        func_names.append(index, &format!("{path}"));
    }
    name_section.functions(&func_names);
    name_section.globals(&global_names);

    let source_map_url = format!("{}.wasm.map", asm_module.name);
    let source_map_url_section = wasm_encoder::CustomSection {
        name: "sourceMappingURL".into(),
        data: source_map_url.as_bytes().to_vec().into(),
    };

    let mut preliminary_module = wasm_encoder::Module::new();
    preliminary_module
        .section(&name_section)
        .section(&type_section)
        .section(&import_section)
        .section(&function_section)
        .section(&table_section);
    if asm_module.has_memory {
        preliminary_module.section(&memory_section);
    }
    preliminary_module
        .section(&global_section)
        .section(&export_section)
        .section(&element_section)
        .section(&code_section)
        .section(&asm_module.sig)
        .section(&LoweredModuleSection::new(asm_module));

    let preliminary_binary = preliminary_module.finish();
    let dwarf_sections = build_dwarf_sections(asm_module, &preliminary_binary, &function_operator_origins);

    let mut module = wasm_encoder::Module::new();
    module
        .section(&name_section)
        .section(&type_section)
        .section(&import_section)
        .section(&function_section)
        .section(&table_section);
    if asm_module.has_memory {
        module.section(&memory_section);
    }
    module
        .section(&global_section)
        .section(&export_section)
        .section(&element_section)
        .section(&code_section)
        .section(&asm_module.sig)
        .section(&LoweredModuleSection::new(asm_module));
    for (name, data) in dwarf_sections {
        module.section(&wasm_encoder::CustomSection {
            name: name.into(),
            data: data.into(),
        });
    }
    module.section(&source_map_url_section);
    let binary = module.finish();
    let source_map = build_source_map_json(asm_module, &binary, &function_operator_origins);
    EncodedModule { binary, source_map }
}
