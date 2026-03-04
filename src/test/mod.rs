/*!
    End-to-end testing for the compiler
*/
#![allow(clippy::unwrap_used)]

use super::*;
use crate::hc_core::compile_core_module;
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
    let source = include_str!("demo.hc");
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
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
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn core_artifact_validates() {
    let mut symbols = SymbolTable::new();
    let core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let _ = validate_artifact(core, &mut logger);
    assert_logger_is_ok(&logger, "Core artifact should validate");
}

#[test]
fn wasm_type_alias_requires_symbol_name() {
    let source = "module demo =\n\twasm => (\n\t\t(type integer (struct i64))\n\t\t(global $asdf integer)\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn wasm_function_name_requires_symbol_name() {
    let source = "module demo =\n\tlet foo = fn x => x\n\twasm => (\n\t\t(func foo)\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn wasm_function_name_accepts_symbol_name() {
    let source = "module demo =\n\twasm => (\n\t\t(func $foo)\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn wasm_reports_undefined_register_usage() {
    let source = "module demo =\n\twasm => (\n\t\t(func $foo\n\t\t\tget $b\n\t\t)\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn wasm_memory_maximum_cannot_be_less_than_initial() {
    let source = "module demo =\n\twasm => (\n\t\t(memory $mem 2 1)\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
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
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
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
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn inline_wasm_expression_can_use_toplevel_wasm_type_alias() {
    let source = "module demo =\n\twasm => (\n\t\t(type $integer (struct i64))\n\t)\n\tlet i = 1\n\tlet j = (wasm : core::integer) => (\n\t\t(local $tmp $integer)\n\t\tget i\n\t\tset $tmp\n\t\tget $tmp\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
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
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn core_print_string_compiles() {
    let source = "module demo =\n\tlet _ = core::print_string \"hello\"\nend\n";
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn recursive_sum_type_definition_compiles() {
    let source = "module demo =\n\ttype List: a = | Cons (a, List a) | Nil\nend\n";
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
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
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
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
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn recursive_non_sum_named_type_is_rejected() {
    let source = "module demo =\n\ttype Node = { next: Node }\nend\n";
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn recursive_named_type_expression_is_rejected() {
    let source = "module demo =\n\ttype Loop = Loop\nend\n";
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn mutually_recursive_type_aliases_are_rejected() {
    let source = "module demo =\n\ttype ~A = B\n\ttype ~B = A\nend\n";
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn recursive_mixed_cycle_with_struct_is_rejected() {
    let source = "module demo =\n\ttype A = | MkA B\n\ttype B = { inner: A }\nend\n";
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn recursive_sum_type_works_with_trait_dispatch() {
    let source = "module demo =\n\ttype List = | Nil | Cons List\n\ttrait Empty : self =\n\t\tlet empty : self -> core::boolean\n\tend\n\timpl Empty : List =\n\t\tlet empty = fn _ => true\n\tend\n\tlet use-empty = fn (xs: List) => empty xs\nend\n";
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn forward_type_reference_is_rejected() {
    let source = "module demo =\n\ttype A = B\n\ttype B = core::integer\nend\n";
    let mut symbols = SymbolTable::new();
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
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
    let _core = compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_has_error_message(
        &file_logger,
        "Undefined term"
    ));
}
