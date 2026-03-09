/*!
    End-to-end testing for the compiler
*/
#![allow(clippy::unwrap_used)]

use super::*;
use crate::hc_core::compile_core_module;
use crate::ir::Path;
use crate::types::SymbolTable;

fn assert_logger_is_ok(
    logger: &Logger,
    message: &str,
) {
    let is_ok = logger.is_ok();
    if !is_ok {
        logger.print_logs();
    }
    assert!(is_ok, "{message}");
}

fn file_logger_has_error_message(
    logger: &FileLogger,
    message: &str,
) -> bool {
    logger
        .iter()
        .any(|diagnostic| diagnostic.message.contains(message))
}

#[test]
fn demo_reports_missing_trait_instance() {
    let source = "module demo =\n\ttrait Id : self =\n\t\tlet id : self -> self\n\tend\n\n\tlet value = id 1\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("missing_trait_instance.hc", source);
    let modules = parse::parse(source, &mut file_logger)
        .map(|source_file| source_file.modules())
        .unwrap_or_default()
        .into_iter()
        .flat_map(|module| ir::module(module, &mut file_logger))
        .collect::<Vec<_>>();
    for module in modules {
        let _ = types::resolve_module_with_symbols(&mut symbols, module, &mut file_logger);
    }
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn associated_constant_trait_item_compiles() {
    let source = "module demo =\n\ttrait default : a =\n\t\tlet default : a\n\tend\n\timpl default : core::integer =\n\t\tlet default = 42\n\tend\n\tlet value : core::integer = default\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn top_level_grouped_polymorphic_destructuring_compiles_and_validates() {
    let source = "module demo =\n\tlet default_pair = (core::default, core::default)\n\tlet (a, b) = default_pair\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn local_grouped_refutable_polymorphic_destructuring_compiles_and_validates() {
    let source = "module demo =\n\tlet result : core::integer =\n\t\tmatch (core::default, core::default, 1) with\n\t\t| (left_local, right_local, 0) => left_local\n\t\t| _ => core::default\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn higher_rank_parameter_annotation_compiles_and_validates() {
    let source = "module demo =\n\tlet id : for a in a -> a = fn x => x\n\tlet apply = fn (f: for a in a -> a) => (f 1, f true)\n\tlet result = apply id\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn higher_rank_parameter_rejects_unannotated_lambda_argument() {
    let source = "module demo =\n\tlet apply = fn (f: for a in a -> a) => (f 1, f true)\n\tlet result = apply (fn x => x)\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let modules = parse::parse(source, &mut file_logger)
        .map(|source_file| source_file.modules())
        .unwrap_or_default()
        .into_iter()
        .flat_map(|module| ir::module(module, &mut file_logger))
        .collect::<Vec<_>>();
    for module in modules {
        let _ = types::resolve_module_with_symbols(&mut symbols, module, &mut file_logger);
    }
    assert!(
        file_logger_has_error_message(&file_logger, "Higher-rank annotation required"),
        "expected higher-rank annotation diagnostic"
    );
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn unconstrained_polymorphic_annotation_is_rejected() {
    let source = "module demo =\n\tlet (f: for a in a -> a -> a) = fn a b => a + b\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let modules = parse::parse(source, &mut file_logger)
        .map(|source_file| source_file.modules())
        .unwrap_or_default()
        .into_iter()
        .flat_map(|module| ir::module(module, &mut file_logger))
        .collect::<Vec<_>>();
    for module in modules {
        let _ = types::resolve_module_with_symbols(&mut symbols, module, &mut file_logger);
    }
    assert!(
        file_logger_has_error_message(
            &file_logger,
            "Polymorphic annotation is missing trait constraints"
        ),
        "expected constrained-polymorphism annotation error"
    );
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn conditional_trait_impl_compiles_and_validates() {
    let source = "module demo =\n\ttrait doubler : a =\n\t\tlet double : a -> a\n\tend\n\timpl doubler : for a in a where core::add a =\n\t\tlet double = fn x => x + x\n\tend\n\tlet value : core::integer = double 21\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn orphan_rule_rejects_foreign_trait_for_foreign_type() {
    let source =
        "module demo =\n\timpl core::default : core::glyph =\n\t\tlet default = 'a'\n\tend\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(
        file_logger_has_error_message(&file_logger, "Invalid trait instance"),
        "expected orphan-rule trait instance error"
    );
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn orphan_rule_allows_foreign_trait_for_local_type() {
    let source = "module demo =\n\ttype Token = { value: core::integer }\n\timpl core::default : Token =\n\t\tlet default = { value = 0 }\n\tend\n\tlet token : Token = core::default\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn core_artifact_validates() {
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let core = compile_core_module(&mut symbols, &mut logger);
    let _ = validate_artifact(core, &mut logger);
    assert_logger_is_ok(&logger, "Core artifact should validate");
}

#[test]
fn wasm_type_alias_requires_symbol_name() {
    let source = "module demo =\n\twasm => (\n\t\t(type integer (struct i64))\n\t\t(global $asdf integer)\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn wasm_function_name_requires_symbol_name() {
    let source = "module demo =\n\tlet foo = fn x => x\n\twasm => (\n\t\t(func foo)\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn wasm_function_name_accepts_symbol_name() {
    let source = "module demo =\n\twasm => (\n\t\t(func $foo)\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn wasm_reports_undefined_register_usage() {
    let source = "module demo =\n\twasm => (\n\t\t(func $foo\n\t\t\tget $b\n\t\t)\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn wasm_memory_maximum_cannot_be_less_than_initial() {
    let source = "module demo =\n\twasm => (\n\t\t(memory $mem 2 1)\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn reports_duplicate_global_term_definition() {
    let source = "module demo =\n\tlet a = 1\n\tlet a = 2\nend\n";
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = parse::parse(source, &mut file_logger)
        .into_iter()
        .flat_map(|source_file| source_file.modules())
        .flat_map(|module| ir::module(module, &mut file_logger))
        .collect::<Vec<_>>();
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "IR construction should fail");
}

#[test]
fn reports_duplicate_constructor_definition_during_ir_construction() {
    let source = "module demo =\n\ttype a = | Dup\n\ttype b = | Dup\nend\n";
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = parse::parse(source, &mut file_logger)
        .into_iter()
        .flat_map(|source_file| source_file.modules())
        .flat_map(|module| ir::module(module, &mut file_logger))
        .collect::<Vec<_>>();
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "IR construction should fail");
}

#[test]
fn duplicate_trait_definition_is_not_re_emitted_in_typechecking() {
    let source = "module demo =\n\ttrait Eq : a =\n\t\tlet eq : a -> a -> core::boolean\n\tend\n\ttrait Eq : a =\n\t\tlet eq : a -> a -> core::boolean\n\tend\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    let messages = file_logger
        .iter()
        .map(|diagnostic| diagnostic.message.clone())
        .collect::<Vec<_>>();
    logger.consume_file(file_logger);
    assert!(
        messages
            .iter()
            .any(|message| message == "Duplicate type definition"),
        "Expected duplicate type diagnostic from IR construction"
    );
    assert!(
        !messages
            .iter()
            .any(|message| message == "Duplicate trait definition"),
        "Typechecking should not re-emit trait duplicate diagnostics"
    );
}

#[test]
fn inline_wasm_expression_compiles() {
    let source = "module demo =\n\tlet i = 1\n\tlet j = (wasm : core::integer) => (\n\t\t(local $tmp (struct i64))\n\t\tget i\n\t\tset $tmp\n\t\tget $tmp\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn inline_wasm_expression_can_use_toplevel_wasm_type_alias() {
    let source = "module demo =\n\twasm => (\n\t\t(type $integer (struct i64))\n\t)\n\tlet i = 1\n\tlet j = (wasm : core::integer) => (\n\t\t(local $tmp $integer)\n\t\tget i\n\t\tset $tmp\n\t\tget $tmp\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn bracketed_operator_name_is_canonicalized_in_ir() {
    let source = "module demo =\n\tlet [ + ] = fn a b => a + b\nend\n";
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let module = parse::parse(source, &mut file_logger)
        .and_then(|source_file| source_file.modules().into_iter().next())
        .and_then(|module| ir::module(module, &mut file_logger))
        .expect("expected module");
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "IR construction should succeed");

    let Some(ir::Statement::Term(term)) = module.statements.first() else {
        panic!("expected first statement to be a term");
    };
    let ir::TermKind::Let {
        assignee,
        scope: ir::ScopeKind::Global,
        ..
    } = &term.kind
    else {
        panic!("expected global let statement");
    };
    let ir::PatternKind::Identifier(path) = &assignee.kind else {
        panic!("expected identifier assignee");
    };
    assert_eq!(path.major, "demo");
    assert_eq!(path.minor, "[+]");
}

#[test]
fn toplevel_wasm_function_declaration_is_lowered() {
    let source = "module demo =\n\twasm => (\n\t\t(type $integer (struct i64))\n\t\t(func $id\n\t\t\t(param $x $integer)\n\t\t\t(result $integer)\n\t\t\tget $x\n\t\t)\n\t)\n\tlet i = 1\n\tlet j = (wasm : core::integer) => (\n\t\tget i\n\t\tcall $id\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn core_print_string_compiles() {
    let source = "module demo =\n\tlet _ = core::print_string \"hello\"\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn recursive_sum_type_definition_compiles() {
    let source = "module demo =\n\ttype List: a = | Cons (a, List a) | Nil\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn mutually_recursive_sum_type_definitions_are_rejected_without_forward_decls() {
    let source =
        "module demo =\n\ttype Even = | EvenZ | EvenS Odd\n\ttype Odd = | OddZ | OddS Even\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_has_error_message(
        &file_logger,
        "Undefined type"
    ));
}

#[test]
fn recursive_type_alias_is_rejected() {
    let source = "module demo =\n\ttype ~Loop = Loop\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn recursive_non_sum_named_type_is_rejected() {
    let source = "module demo =\n\ttype Node = { next: Node }\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn recursive_named_type_expression_is_rejected() {
    let source = "module demo =\n\ttype Loop = Loop\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn mutually_recursive_type_aliases_are_rejected() {
    let source = "module demo =\n\ttype ~A = B\n\ttype ~B = A\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn recursive_mixed_cycle_with_struct_is_rejected() {
    let source = "module demo =\n\ttype A = | MkA B\n\ttype B = { inner: A }\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn recursive_sum_type_works_with_trait_dispatch() {
    let source = "module demo =\n\ttype List = | Nil | Cons List\n\ttrait Empty : self =\n\t\tlet empty : self -> core::boolean\n\tend\n\timpl Empty : List =\n\t\tlet empty = fn _ => true\n\tend\n\tlet use-empty = fn (xs: List) => empty xs\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn forward_type_reference_is_rejected() {
    let source = "module demo =\n\ttype A = B\n\ttype B = core::integer\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_has_error_message(
        &file_logger,
        "Undefined type"
    ));
}

#[test]
fn forward_module_path_term_reference_is_rejected() {
    let source = "module demo =\n\tlet a = demo::b\n\tlet b = 1\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_has_error_message(
        &file_logger,
        "Undefined term"
    ));
}

#[test]
fn nested_module_terms_are_inlined_into_toplevel_module() {
    let source = "module demo =\n\tmodule math =\n\t\tlet add-one = fn x => x + 1\n\tend\n\tlet value : core::integer = math::add-one 41\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn nested_module_paths_fallback_to_absolute_modules() {
    let source = "module demo =\n\tmodule inner =\n\t\tlet value : core::integer = core::default\n\tend\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn nested_module_paths_can_use_root_for_external_modules() {
    let source = "module demo =\n\tmodule inner =\n\t\tlet value : root::core::integer = root::core::default\n\tend\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn nested_module_types_are_reachable_via_relative_paths() {
    let source = "module demo =\n\tmodule model =\n\t\ttype ~Token = root::core::integer\n\tend\n\tlet value : model::Token = 1\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn use_imports_module_symbols_for_following_statements() {
    let source = "module demo =\n\tuse core\n\tlet value : integer = default\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn use_only_applies_to_following_statements() {
    let source =
        "module demo =\n\tlet before = default\n\tuse core\n\tlet after : integer = default\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_has_error_message(
        &file_logger,
        "Undefined term"
    ));
}

#[test]
fn use_imports_nested_modules_for_path_resolution() {
    let source = "module demo =\n\tmodule M =\n\t\tmodule N =\n\t\t\tlet x = 1\n\t\tend\n\tend\n\tuse M\n\tlet value : core::integer = N::x\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn use_reports_ambiguity_when_multiple_modules_provide_symbol() {
    let source = "module demo =\n\tmodule A =\n\t\tlet x = 1\n\tend\n\tmodule B =\n\t\tlet x = 2\n\tend\n\tuse A\n\tuse B\n\tlet y = x\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_has_error_message(
        &file_logger,
        "Ambiguous term"
    ));
}

#[test]
fn use_scope_does_not_leak_into_nested_module() {
    let source = "module demo =\n\tuse core\n\tmodule inner =\n\t\tlet value : integer = default\n\tend\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_has_error_message(
        &file_logger,
        "Undefined type"
    ));
}

#[test]
fn use_alias_binds_module_name_without_opening_contents() {
    let source = "module demo =\n\tuse core as c\n\tlet value : c::integer = c::default\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn use_alias_does_not_import_unqualified_symbols() {
    let source = "module demo =\n\tuse core as c\n\tlet value : integer = default\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_has_error_message(
        &file_logger,
        "Undefined type"
    ));
}

#[test]
fn use_alias_name_collisions_are_reported() {
    let source = "module demo =\n\tmodule A =\n\t\tlet x = 1\n\tend\n\tmodule B =\n\t\tlet x = 2\n\tend\n\tuse A as m\n\tuse B as m\n\tlet value = 0\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_has_error_message(
        &file_logger,
        "Duplicate module alias"
    ));
}

#[test]
fn use_expression_imports_only_inside_in_body() {
    let source = "module demo =\n\tlet inside : core::integer = use core in default\n\tlet outside = default\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_has_error_message(
        &file_logger,
        "Undefined term"
    ));
}

#[test]
fn use_expression_alias_works() {
    let source = "module demo =\n\tlet value : core::integer = use core as c in c::default\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn use_expression_alias_collisions_are_reported() {
    let source = "module demo =\n\tmodule A =\n\t\tlet x = 1\n\tend\n\tmodule B =\n\t\tlet x = 2\n\tend\n\tlet value = use A as m in use B as m in m::x\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_has_error_message(
        &file_logger,
        "Duplicate module alias"
    ));
}

#[test]
fn implicit_core_prelude_is_used_when_available() {
    let source = "module core =\n\tmodule prelude =\n\t\tlet answer = 1\n\tend\nend\n\nmodule demo =\n\tlet value : root::core::integer = answer\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn missing_core_prelude_is_ignored_for_implicit_use() {
    let source = "module demo =\n\tlet value = default\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_has_error_message(
        &file_logger,
        "Undefined term"
    ));
}

#[test]
fn forall_annotation_type_checks() {
    let source = "module demo =
  let id: for a in a -> a = fn x => x
end
";
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let modules = parse::parse(source, &mut file_logger)
        .map(|source_file| source_file.modules())
        .unwrap_or_default()
        .into_iter()
        .flat_map(|module| ir::module(module, &mut file_logger))
        .collect::<Vec<_>>();
    assert!(!modules.is_empty(), "should produce at least one module");
    let is_ok = file_logger.is_ok();
    logger.consume_file(file_logger);
    if !is_ok {
        logger.print_logs();
    }
    assert_logger_is_ok(
        &logger,
        "forall annotation should parse and lower to IR without errors",
    );
}

#[test]
fn forall_impl_head_type_checks() {
    let source = "module demo =
  trait Id : t =
    let id : t -> t
  end

  impl Id : for a in a =
    let id = fn x => x
  end

  let value = id 1
end
";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    let trait_path = Path::new("demo", "Id");
    let impls = symbols
        .trait_impls()
        .get(&trait_path)
        .cloned()
        .unwrap_or_default();
    assert_eq!(impls.len(), 1, "expected one impl for demo::Id");
    assert_eq!(impls[0].parameters, 1, "expected one impl parameter");
    assert_eq!(
        impls[0].head.arguments.len(),
        1,
        "expected one trait argument"
    );
    assert_eq!(
        impls[0].head.arguments[0].pretty(),
        "'a",
        "expected generic impl head argument"
    );
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "forall impl head should type-check");
}
