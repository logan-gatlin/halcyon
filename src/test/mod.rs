/*!
    End-to-end testing for the compiler
*/
#![allow(clippy::unwrap_used)]

use super::*;
use crate::hc_core::{
    CoreType,
    compile_core_module,
};
use crate::ir::Path;
use crate::types::symbol_table::Symbol;
use crate::types::{
    SymbolTable,
    Type,
};

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("HALCYON_LOG"))
        .with_test_writer()
        .try_init();
}

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

#[test]
fn compile_source_linked_with_core_returns_binary_on_success() {
    let source = "module demo =\n\tlet value : core::Integer = core::default\nend\n";
    let binary = compile_source_linked_with_core(source)
        .expect("expected successful compilation and linking with core");
    assert!(binary.starts_with(&WASM_MAGIC_NUMBER));
}

#[test]
fn compile_source_linked_with_core_returns_error_messages() {
    let source = "module demo =\n\tlet broken = missing_symbol\nend\n";
    let errors = compile_source_linked_with_core(source)
        .expect_err("expected compile errors for unknown symbol");
    assert!(!errors.is_empty(), "expected at least one error message");
}

#[test]
fn zero_argument_function_rejects_non_unit_argument() {
    let source = "module demo =\n\tlet thunk = fn => ()\n\tlet value = thunk 1\nend\n";
    let errors = compile_source_linked_with_core(source)
        .expect_err("calling a zero-argument function with a non-unit argument should fail");
    assert!(!errors.is_empty(), "expected at least one type error");
}

fn file_logger_has_error_message(
    logger: &FileLogger,
    message: &str,
) -> bool {
    logger
        .iter()
        .any(|diagnostic| diagnostic.message.contains(message))
}

fn logger_has_error_message(
    logger: &Logger,
    message: &str,
) -> bool {
    logger
        .iter()
        .any(|diagnostic| diagnostic.message.contains(message))
}

fn logger_has_error_label_in_file(
    logger: &Logger,
    message: &str,
    file_name: &str,
) -> bool {
    logger.serialize().iter().any(|diagnostic| {
        diagnostic.message.contains(message)
            && diagnostic
                .labels
                .iter()
                .any(|label| label.file_name == file_name)
    })
}

fn file_logger_count_message(
    logger: &FileLogger,
    message: &str,
) -> usize {
    logger
        .iter()
        .filter(|diagnostic| diagnostic.message.contains(message))
        .count()
}

#[test]
fn compile_source_with_options_resolves_imports_during_ir_generation() {
    let source = "bundle demo\nimport \"dep\"\nlet value : core::Integer = core::default\n";
    let dep_source = "let imported_value : core::Integer = core::default\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();

    let artifacts = compile_source_with_options(
        "demo.hc",
        source,
        &mut logger,
        &mut symbols,
        CompileOptions {
            demo_mode: false,
            use_core: true,
            emit_source_map: true,
            emit_dwarf: true,
            resolve_import: |path| path.ends_with("dep").then(|| dep_source.to_string()),
        },
    );

    assert_logger_is_ok(&logger, "compilation with imported source should succeed");
    assert!(
        artifacts.len() >= 2,
        "use_core=true should include core and bundle artifacts"
    );
}

#[test]
fn compile_source_with_options_reports_duplicate_imports() {
    let source =
        "bundle demo\nimport \"dep\", \"dep\"\nlet value : core::Integer = core::default\n";
    let dep_source = "let imported_value : core::Integer = core::default\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();

    let _ = compile_source_with_options(
        "demo.hc",
        source,
        &mut logger,
        &mut symbols,
        CompileOptions {
            demo_mode: false,
            use_core: true,
            emit_source_map: true,
            emit_dwarf: true,
            resolve_import: |path| path.ends_with("dep").then(|| dep_source.to_string()),
        },
    );

    assert!(
        logger_has_error_message(&logger, "Duplicate import"),
        "expected duplicate import diagnostic"
    );
}

#[test]
fn compile_source_with_options_resolves_imports_inside_modules() {
    let source = "bundle demo
module foo =
    import \"dep\"
end
module app =
    let value : core::Integer = bundle::foo::imported_value
end
";
    let dep_source = "let imported_value : core::Integer = core::default\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();

    let artifacts = compile_source_with_options(
        "demo.hc",
        source,
        &mut logger,
        &mut symbols,
        CompileOptions {
            demo_mode: false,
            use_core: true,
            emit_source_map: true,
            emit_dwarf: true,
            resolve_import: |path| path.ends_with("dep").then(|| dep_source.to_string()),
        },
    );

    assert_logger_is_ok(
        &logger,
        "module-scoped import should resolve into module scope",
    );
    assert!(
        artifacts.len() >= 2,
        "use_core=true should include core and bundle artifacts"
    );
}

#[test]
fn compile_source_with_options_reports_bundle_not_first_in_root() {
    let source = "let value = 1\nbundle demo\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();

    let _ = compile_source_with_options(
        "demo.hc",
        source,
        &mut logger,
        &mut symbols,
        CompileOptions {
            demo_mode: false,
            use_core: false,
            emit_source_map: true,
            emit_dwarf: true,
            resolve_import: |_| None,
        },
    );

    assert!(
        logger_has_error_message(&logger, "Missing bundle declaration"),
        "expected root bundle ordering diagnostic"
    );
}

#[test]
fn compile_source_with_options_reports_duplicate_bundle_declarations_globally() {
    let source = "bundle demo\nimport \"dep\"\nlet value : core::Integer = core::default\n";
    let dep_source = "bundle dep\nlet imported_value : core::Integer = core::default\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();

    let _ = compile_source_with_options(
        "demo.hc",
        source,
        &mut logger,
        &mut symbols,
        CompileOptions {
            demo_mode: false,
            use_core: true,
            emit_source_map: true,
            emit_dwarf: true,
            resolve_import: |path| path.ends_with("dep").then(|| dep_source.to_string()),
        },
    );

    assert!(
        logger_has_error_message(&logger, "Duplicate bundle declaration"),
        "expected duplicate bundle diagnostic across imported files"
    );
}

#[test]
fn compile_source_with_options_reports_type_errors_in_import_file() {
    let source = "bundle demo\nimport \"dep\"\nlet value = 1\n";
    let dep_source = "let broken = if true then 1 else \"oops\"\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();

    let _ = compile_source_with_options(
        "demo.hc",
        source,
        &mut logger,
        &mut symbols,
        CompileOptions {
            demo_mode: false,
            use_core: false,
            emit_source_map: true,
            emit_dwarf: true,
            resolve_import: |path| path.ends_with("dep").then(|| dep_source.to_string()),
        },
    );

    assert!(
        logger_has_error_label_in_file(&logger, "Type mismatch", "dep"),
        "expected imported type error to point at import file"
    );
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
    let source = "module demo =\n\ttrait DefaultValue : a =\n\t\tlet default : a\n\tend\n\timpl DefaultValue core::Integer =\n\t\tlet default = 42\n\tend\n\tlet value : core::Integer = default\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn associated_type_trait_item_compiles() {
    let source = "module demo =\n\ttrait Iterator : iter =\n\t\ttype Item\n\t\tlet next : iter -> Iterator::Item iter\n\tend\n\ttype IntIter = { value: core::Integer }\n\timpl Iterator IntIter =\n\t\ttype Item = core::Integer\n\t\tlet next = fn iter => iter.value\n\tend\n\tlet value : Iterator::Item IntIter = next { value = 1 }\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn trait_alias_impl_and_dispatch_compile() {
    let source = "module demo =\n\ttrait Eq : a =\n\t\tlet eq : a -> a -> a\n\tend\n\ttrait ~Equal = Eq\n\timpl Equal Integer =\n\t\tlet eq = fn x _ => x\n\tend\n\tlet value = eq 1 1\nend\n";
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
fn trait_alias_in_where_constraint_resolves() {
    let source = "module demo =\n\ttrait Eq : a =\n\t\tlet eq : a -> a -> a\n\tend\n\ttrait ~Equal = Eq\n\timpl Eq Integer =\n\t\tlet eq = fn x _ => x\n\tend\n\tlet check : for a in a -> a -> a where Equal a = eq\n\tlet value = check 1 1\nend\n";
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
fn trait_alias_target_must_be_trait() {
    let source = "module demo =\n\ttype Token = core::Integer\n\ttrait ~Equal = Token\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(
        file_logger_has_error_message(&file_logger, "Invalid trait alias"),
        "expected invalid trait alias diagnostic"
    );
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn trait_alias_is_available_to_later_modules_through_use_imports() {
    let source = "bundle demo\nmodule first =\n\ttrait Eq : a =\n\t\tlet eq : a -> a -> a\n\tend\n\ttrait ~Equal = Eq\nend\n\nmodule second =\n\tuse root::demo::first\n\ttype Token = { value: Integer }\n\timpl Equal Token =\n\t\tlet eq = fn x _ => x\n\tend\n\tlet value = eq { value = 1 } { value = 2 }\nend\n";
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
    let source = "module demo =\n\tlet result : core::Integer =\n\t\tmatch (core::default, core::default, 1) with\n\t\t| (left_local, right_local, 0) => left_local\n\t\t| _ => core::default\nend\n";
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
fn higher_rank_parameter_accepts_unannotated_lambda_argument() {
    let source = "module demo =\n\tlet apply = fn (f: for a in a -> a) => (f 1, f true)\n\tlet result = apply (fn x => x)\nend\n";
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
fn unconstrained_polymorphic_annotation_is_rejected() {
    let source = "module demo =\n\tlet (f: for a in a -> a -> a) = fn a b => a + b\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
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
fn strict_forall_value_annotation_is_enforced() {
    let source = "module demo =\n\tlet a: for a in a = 1\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(
        file_logger_has_error_message(&file_logger, "Type mismatch"),
        "expected strict annotation type mismatch"
    );
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn conditional_trait_impl_compiles_and_validates() {
    let source = "module demo =\n\ttrait Doubler : a =\n\t\tlet double : a -> a\n\tend\n\timpl Doubler for a in a where core::ops::Add a =\n\t\tlet double = fn x => x + x\n\tend\n\tlet value : core::Integer = double 21\nend\n";
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
fn higher_kinded_trait_impl_compiles_and_validates() {
    let source = "module demo =\n\ttrait Monad : m =\n\t\tlet map : for a b in (a -> b) -> m a -> m b\n\tend\n\timpl Monad core::opt::Option =\n\t\tlet map = core::opt::map\n\tend\n\tlet result : core::opt::Option core::Integer = map (fn x => x + 1) (core::opt::Some 1)\nend\n";
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
fn higher_kinded_trait_map_and_flatten_impl_compiles_and_validates() {
    init_tracing();
    let source = "module demo =\n\ttrait Monad : m =\n\t\tlet map : for a b in (a -> b) -> m a -> m b\n\t\tlet flatten : for a in m (m a) -> m a\n\tend\n\timpl Monad core::opt::Option =\n\t\tlet map = core::opt::map\n\t\tlet flatten = fn opt => match opt with\n\t\t\t| core::opt::Some value => value\n\t\t\t| core::opt::None => core::opt::None\n\tend\n\tlet result : core::opt::Option core::Integer = flatten (map (fn x => core::opt::Some (x + 1)) (core::opt::Some 1))\nend\n";
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
fn higher_kinded_trait_impl_rejects_wrong_kind_argument() {
    let source = "module demo =\n\ttrait Monad : m =\n\t\tlet map : for a b in (a -> b) -> m a -> m b\n\tend\n\timpl Monad core::Integer =\n\t\tlet map = fn f value => f value\n\tend\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(
        file_logger_has_error_message(&file_logger, "Invalid trait argument kind"),
        "expected wrong-kind trait argument error"
    );
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn impl_direct_self_reference_reports_circular_definition() {
    let source = "module demo =\n\ttrait F : t =\n\t\tlet f : t -> t\n\tend\n\timpl F core::Integer =\n\t\tlet f = f\n\tend\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);

    assert!(file_logger_has_error_message(
        &file_logger,
        "Invalid circular impl definition"
    ));
    assert!(file_logger.iter().any(|diagnostic| {
        diagnostic.message == "Invalid circular impl definition"
            && diagnostic
                .notes
                .iter()
                .any(|note| note.contains("function-guarded"))
    }));
    assert!(!file_logger_has_error_message(
        &file_logger,
        "Unknown identifier"
    ));

    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn impl_mutual_non_function_cycle_reports_circular_definition() {
    let source = "module demo =\n\ttrait Pair : t =\n\t\tlet f : t -> t\n\t\tlet g : t -> t\n\tend\n\timpl Pair core::Integer =\n\t\tlet f = g\n\t\tlet g = f\n\tend\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);

    assert_eq!(
        file_logger_count_message(&file_logger, "Invalid circular impl definition"),
        1
    );
    assert!(!file_logger_has_error_message(
        &file_logger,
        "Unknown identifier"
    ));

    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn impl_mutually_recursive_functions_are_allowed() {
    let source = "module demo =\n\ttrait Pair : t =\n\t\tlet f : t -> t\n\t\tlet g : t -> t\n\tend\n\timpl Pair core::Integer =\n\t\tlet f = fn x => g x\n\t\tlet g = fn x => f x\n\tend\n\tlet value : core::Integer = f 1\nend\n";
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
fn higher_kinded_annotation_rejects_constructor_kind_where_type_is_required() {
    init_tracing();
    let source = "module demo =\n\tlet value : core::opt::Option = core::opt::Some 1\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(
        file_logger_has_error_message(&file_logger, "Kind mismatch"),
        "expected annotation kind mismatch"
    );
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn core_monad_methods_dispatch_for_option_array_and_result() {
    init_tracing();
    let source = "module demo =\n\tlet opt_map : core::opt::Option core::Integer = core::hkt::map (fn x => x + 1) (core::opt::Some 1)\n\tlet opt_flatten : core::opt::Option core::Integer = core::hkt::flatten (core::opt::Some (core::opt::Some 1))\n\tlet opt_bind : core::opt::Option core::Integer = core::hkt::flatten (core::hkt::map (fn x => core::opt::Some (x + 1)) (core::opt::Some 1))\n\tlet arr_map : [] core::Integer = core::hkt::map (fn x => x + 1) [1, 2, 3]\n\tlet res_bind : core::result::Result core::String core::Integer = core::hkt::flatten (core::hkt::map (fn x => core::result::Ok (x + 1)) (core::result::Ok 1))\nend\n";
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
        "module demo =\n\timpl core::Default core::Glyph =\n\t\tlet default = 'a'\n\tend\nend\n";
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
    let source = "module demo =\n\ttype Token = { value: core::Integer }\n\timpl core::Default Token =\n\t\tlet default = { value = 0 }\n\tend\n\tlet token : Token = core::default\nend\n";
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
fn naming_style_lints_warn_for_non_conforming_names() {
    let source = "module Demo =\n\ttype my_type = { BadField: core::Integer }\n\ttype BadSum = | lower\n\ttrait my_trait : a =\n\t\tlet BadItem : a -> a\n\tend\n\timpl my_trait core::Integer =\n\t\tlet BadItem = fn x => x\n\tend\n\tlet BadLet = 1\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);

    assert!(
        file_logger_count_message(&file_logger, "Naming style") >= 8,
        "expected naming-style warnings for module/type/field/constructor/trait/trait-items/let"
    );
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
    let source = "module demo =\n\ttrait Eq : a =\n\t\tlet eq : a -> a -> core::Boolean\n\tend\n\ttrait Eq : a =\n\t\tlet eq : a -> a -> core::Boolean\n\tend\nend\n";
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
    let duplicate_trait_count = messages
        .iter()
        .filter(|message| *message == "Duplicate trait definition")
        .count();
    assert!(
        duplicate_trait_count == 1,
        "Expected exactly one duplicate trait diagnostic from IR construction"
    );
    assert!(
        !messages
            .iter()
            .any(|message| message == "Duplicate type definition"),
        "Traits should not report duplicate type diagnostics"
    );
}

#[test]
fn type_alias_cannot_reference_trait_name() {
    let source = "module demo =\n\ttrait Null = end\n\ttype ~N = Null\nend\n";
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
fn inline_wasm_expression_compiles() {
    let source = "module demo =\n\tlet i = 1\n\tlet j = (wasm : core::Integer) => (\n\t\t(local $tmp (struct i64))\n\t\tget i\n\t\tset $tmp\n\t\tget $tmp\n\t)\nend\n";
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
    let source = "module demo =\n\twasm => (\n\t\t(type $integer (struct i64))\n\t)\n\tlet i = 1\n\tlet j = (wasm : core::Integer) => (\n\t\t(local $tmp $integer)\n\t\tget i\n\t\tset $tmp\n\t\tget $tmp\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn inline_wasm_truncate_instructions_compile_and_validate() {
    let source = "module demo =\n\twasm => (\n\t\t(type $integer (struct i64))\n\t)\n\tlet trunc_demo = (wasm : core::Integer) => (\n\t\tf64.const 9.75\n\t\ti32.trunc_f64_s\n\t\tdrop\n\t\tf64.const 9.75\n\t\ti32.trunc_f64_u\n\t\tdrop\n\t\tf32.const 9.75\n\t\ti32.trunc_f32_s\n\t\tdrop\n\t\tf32.const 9.75\n\t\ti32.trunc_f32_u\n\t\tdrop\n\t\tf64.const 9.75\n\t\ti64.trunc_f64_s\n\t\tdrop\n\t\tf64.const 9.75\n\t\ti64.trunc_f64_u\n\t\tdrop\n\t\tf32.const 9.75\n\t\ti64.trunc_f32_s\n\t\tdrop\n\t\tf32.const 9.75\n\t\ti64.trunc_f32_u\n\t\tdrop\n\t\ti64.const 42\n\t\ti32.wrap_i64\n\t\tdrop\n\t\tf64.const 9.75\n\t\tf32.demote_f64\n\t\tdrop\n\t\ti64.const 0\n\t\tstruct.new $integer\n\t)\nend\n";
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
fn infix_operator_uses_standard_name_resolution() {
    let source = "module demo =\n\tlet [+] = fn left right => left\n\tlet value = 1 + 2\nend\n";
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let module = parse::parse(source, &mut file_logger)
        .and_then(|source_file| source_file.modules().into_iter().next())
        .and_then(|module| ir::module(module, &mut file_logger))
        .expect("expected module");
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "IR construction should succeed");

    let Some(ir::Statement::Term(term)) = module.statements.get(1) else {
        panic!("expected second statement to be a term");
    };
    let ir::TermKind::Let {
        value,
        scope: ir::ScopeKind::Global,
        ..
    } = &term.kind
    else {
        panic!("expected global let statement");
    };
    let ir::TermKind::Call { callee, .. } = &value.kind else {
        panic!("expected operator application");
    };
    let ir::TermKind::Call { callee, .. } = &callee.kind else {
        panic!("expected curried operator application");
    };
    let ir::TermKind::Identifier(path) = &callee.kind else {
        panic!("expected operator identifier");
    };
    assert_eq!(path.major, "demo");
    assert_eq!(path.minor, "[+]");
}

#[test]
fn nested_function_captures_propagate_to_intermediate_lambdas() {
    let source =
        "module demo =\n\tlet compose = fn first second value => first (second value)\nend\n";
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
        value,
        scope: ir::ScopeKind::Global,
        ..
    } = &term.kind
    else {
        panic!("expected global let statement");
    };
    let ir::TermKind::Function {
        parameter_name: first,
        body: second_fn,
        ..
    } = &value.kind
    else {
        panic!("expected outer function");
    };
    let ir::TermKind::Function {
        parameter_name: second,
        captures: second_captures,
        body: value_fn,
        ..
    } = &second_fn.kind
    else {
        panic!("expected middle function");
    };
    assert!(
        second_captures
            .iter()
            .any(|(capture, _)| capture == &first.inner),
        "middle function should capture outer parameter"
    );

    let ir::TermKind::Function {
        captures: value_captures,
        ..
    } = &value_fn.kind
    else {
        panic!("expected inner function");
    };
    assert!(
        value_captures
            .iter()
            .any(|(capture, _)| capture == &first.inner),
        "inner function should capture first parameter"
    );
    assert!(
        value_captures
            .iter()
            .any(|(capture, _)| capture == &second.inner),
        "inner function should capture second parameter"
    );
}

#[test]
fn zero_argument_function_lowers_with_implicit_unit_parameter_type() {
    let source = "module demo =\n\tlet thunk = fn => ()\nend\n";
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
        value,
        scope: ir::ScopeKind::Global,
        ..
    } = &term.kind
    else {
        panic!("expected global let statement");
    };
    let ir::TermKind::Function {
        parameter_type: Some(parameter_type),
        ..
    } = &value.kind
    else {
        panic!("expected lowered function to have an implicit parameter type");
    };
    let ir::TypeExprKind::Instantiation(path, args) = &parameter_type.kind else {
        panic!("expected implicit parameter type to be a concrete unit instantiation");
    };
    assert_eq!(path, &CoreType::Unit.path());
    assert!(args.is_empty(), "unit type should not have type arguments");
}

#[test]
fn let_expression_binding_is_recursive_during_ir_lowering() {
    let source = "module demo =\n\tlet value = let loop = fn x => loop x in loop\nend\n";
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
        value: outer_value,
        scope: ir::ScopeKind::Global,
        ..
    } = &term.kind
    else {
        panic!("expected global let statement");
    };
    let ir::TermKind::Let {
        assignee,
        value: recursive_value,
        scope: ir::ScopeKind::Local,
        ..
    } = &outer_value.kind
    else {
        panic!("expected nested local let expression");
    };
    let ir::PatternKind::Identifier(loop_path) = &assignee.kind else {
        panic!("expected recursive let assignee to be an identifier");
    };
    let ir::TermKind::Function { body, .. } = &recursive_value.kind else {
        panic!("expected recursive let value to be a function");
    };
    let ir::TermKind::Call { callee, .. } = &body.kind else {
        panic!("expected recursive function body to call itself");
    };
    let ir::TermKind::Identifier(callee_path) = &callee.kind else {
        panic!("expected recursive call callee to be an identifier");
    };
    assert_eq!(callee_path, loop_path);
}

#[test]
fn plus_operator_resolves_through_prelude_aliases() {
    let source = "module demo =\n\tmodule ops =\n\t\tlet [+] = fn left right => left\n\tend\n\tmodule prelude =\n\t\tlet [+] = ops::[+]\n\tend\n\tuse prelude\n\tlet value = 1 + 2\nend\n";
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let module = parse::parse(source, &mut file_logger)
        .and_then(|source_file| source_file.modules().into_iter().next())
        .and_then(|module| ir::module(module, &mut file_logger))
        .expect("expected module");
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "IR construction should succeed");

    let Some(ir::Statement::Term(term)) = module.statements.last() else {
        panic!("expected final statement to be a term");
    };
    let ir::TermKind::Let {
        value,
        scope: ir::ScopeKind::Global,
        ..
    } = &term.kind
    else {
        panic!("expected global let statement");
    };
    let ir::TermKind::Call { callee, .. } = &value.kind else {
        panic!("expected operator application");
    };
    let ir::TermKind::Call { callee, .. } = &callee.kind else {
        panic!("expected curried operator application");
    };
    let ir::TermKind::Identifier(path) = &callee.kind else {
        panic!("expected operator identifier");
    };
    assert_eq!(path.major, "demo");
    assert_eq!(path.minor, "prelude::[+]");
}

#[test]
fn semicolon_syntax_remains_non_overridable() {
    let source = "module demo =\n\tlet [;] = fn _ kept => kept\n\tlet value = 1; 2\nend\n";
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let module = parse::parse(source, &mut file_logger)
        .and_then(|source_file| source_file.modules().into_iter().next())
        .and_then(|module| ir::module(module, &mut file_logger))
        .expect("expected module");
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "IR construction should succeed");

    let Some(ir::Statement::Term(term)) = module.statements.get(1) else {
        panic!("expected second statement to be a term");
    };
    let ir::TermKind::Let {
        value,
        scope: ir::ScopeKind::Global,
        ..
    } = &term.kind
    else {
        panic!("expected global let statement");
    };
    assert!(matches!(value.kind, ir::TermKind::Semicolon(_, _)));
}

#[test]
fn toplevel_wasm_function_declaration_is_lowered() {
    let source = "module demo =\n\twasm => (\n\t\t(type $integer (struct i64))\n\t\t(func $id\n\t\t\t(param $x $integer)\n\t\t\t(result $integer)\n\t\t\tget $x\n\t\t)\n\t)\n\tlet i = 1\n\tlet j = (wasm : core::Integer) => (\n\t\tget i\n\t\tcall $id\n\t)\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn prelude_print_compiles() {
    let source = "module demo =\n\tlet _ = print \"hello\"\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "Compilation failed");
}

#[test]
fn do_statement_lowers_like_let_underscore() {
    let source = "module demo =\n\tdo 1\nend\n";
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
    assert!(matches!(assignee.kind, ir::PatternKind::Hole));
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
fn struct_type_spread_definitions_compile() {
    let source = "module demo =\n\ttype Vector2 = { x: core::Integer, y: core::Integer }\n\ttype Circle = { radius: core::Integer, ..Vector2 }\n\ttype Box: a = { inner: a }\n\ttype Box2 = { ..Box core::Integer }\n\tlet circle: Circle = { radius = 1, x = 0, y = 0 }\n\tlet box2: Box2 = { inner = 1 }\nend\n";
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
fn struct_type_spread_reports_duplicate_fields() {
    let source = "module demo =\n\ttype Vector2 = { x: core::Integer, y: core::Integer }\n\ttype Bad = { x: core::Integer, ..Vector2 }\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(
        file_logger_has_error_message(&file_logger, "Duplicate struct field"),
        "expected duplicate struct field diagnostic"
    );
}

#[test]
fn recursive_sum_type_works_with_trait_dispatch() {
    let source = "module demo =\n\ttype List = | Nil | Cons List\n\ttrait Empty : self =\n\t\tlet empty : self -> core::Boolean\n\tend\n\timpl Empty List =\n\t\tlet empty = fn _ => true\n\tend\n\tlet use-empty = fn (xs: List) => empty xs\nend\n";
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
    let source = "module demo =\n\ttype A = B\n\ttype B = core::Integer\nend\n";
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
    let source = "bundle demo\nlet a = demo::b\nlet b = 1\n";
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
fn unresolved_placeholder_type_constructor_reports_specific_error() {
    let source = "module demo =\n\ttype ~Const: a = a\n\tlet _: Const _ = \"\"\n\tlet _: _ String = \"\"\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_has_error_message(
        &file_logger,
        "Could not infer placeholder type constructor"
    ));
}

#[test]
fn nested_module_terms_are_inlined_into_toplevel_module() {
    let source = "module demo =\n\tmodule math =\n\t\tlet add-one = fn x => x + 1\n\tend\n\tlet value : core::Integer = math::add-one 41\nend\n";
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
    let source = "module demo =\n\tmodule inner =\n\t\tlet value : core::Integer = core::default\n\tend\nend\n";
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
    let source = "module demo =\n\tmodule inner =\n\t\tlet value : root::core::Integer = root::core::default\n\tend\nend\n";
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
fn nested_module_paths_can_use_bundle_for_current_bundle_roots() {
    let source = "bundle demo\nmodule first =\n\tlet value : core::Integer = 41\nend\nmodule outer =\n\tmodule first =\n\t\tlet value : core::Integer = 0\n\tend\n\tlet local : core::Integer = first::value\n\tlet absolute : core::Integer = bundle::first::value\nend\n";
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
fn implicit_bundle_relative_path_fallback_emits_warning() {
    let source = "bundle demo\nmodule values =\n\tlet answer : core::Integer = 41\nend\nmodule consumer =\n\tlet value : core::Integer = demo::values::answer\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);

    assert_eq!(
        file_logger_count_message(&file_logger, "Implicit bundle-relative path"),
        1
    );
    assert!(
        file_logger.is_ok(),
        "Compilation should succeed with a warning"
    );
}

#[test]
fn implicit_bundle_relative_path_warning_is_not_emitted_for_local_resolution() {
    let source = "bundle demo\nmodule outer =\n\tmodule demo =\n\t\tmodule values =\n\t\t\tlet answer : core::Integer = 1\n\t\tend\n\tend\n\tlet local : core::Integer = demo::values::answer\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);

    assert_eq!(
        file_logger_count_message(&file_logger, "Implicit bundle-relative path"),
        0
    );
    assert!(file_logger.is_ok(), "Compilation failed");
}

#[test]
fn implicit_bundle_relative_path_warning_is_not_emitted_for_undefined_path() {
    let source = "bundle demo\nlet value : core::Integer = demo::missing\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);

    assert!(file_logger_has_error_message(
        &file_logger,
        "Undefined term"
    ));
    assert_eq!(
        file_logger_count_message(&file_logger, "Implicit bundle-relative path"),
        0
    );
}

#[test]
fn implicit_bundle_relative_use_target_fallback_emits_warning() {
    let source = "bundle demo\nmodule helper =\n\tlet token : core::Integer = 1\nend\nmodule consumer =\n\tuse demo::helper\n\tlet value : core::Integer = token\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);

    assert_eq!(
        file_logger_count_message(&file_logger, "Implicit bundle-relative path"),
        1
    );
    assert!(
        file_logger.is_ok(),
        "Compilation should succeed with a warning"
    );
}

#[test]
fn implicit_bundle_relative_use_target_without_suffix_emits_bundle_warning() {
    let source = "bundle core\nmodule consumer =\n\tuse core\n\tlet value = 1\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("core.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);

    assert_eq!(
        file_logger_count_message(&file_logger, "Implicit bundle-relative path"),
        1
    );
    assert!(
        file_logger.iter().any(|diagnostic| {
            diagnostic.message.contains("Implicit bundle-relative path")
                && diagnostic.labels.iter().any(|label| {
                    label.message.contains("`core` resolves to `root::core::`")
                        && label.message.contains(
                            "use `bundle` to refer to the current bundle in a less ambiguous way",
                        )
                })
        }),
        "warning should include root-qualified resolution and bundle suggestion"
    );
    assert!(
        file_logger.is_ok(),
        "Compilation should succeed with a warning"
    );
}

#[test]
fn implicit_bundle_relative_use_target_without_suffix_no_warning_for_local_module() {
    let source = "bundle demo\nmodule outer =\n\tmodule demo =\n\t\tlet value = 1\n\tend\n\tuse demo\n\tlet result = value\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);

    assert_eq!(
        file_logger_count_message(&file_logger, "Implicit bundle-relative path"),
        0
    );
    assert!(file_logger.is_ok(), "Compilation failed");
}

#[test]
fn nested_module_types_are_reachable_via_relative_paths() {
    let source = "module demo =\n\tmodule model =\n\t\ttype ~Token = root::core::Integer\n\tend\n\tlet value : model::Token = 1\nend\n";
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
    let source = "module demo =\n\tuse core\n\tlet value = array::empty\nend\n";
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
fn use_bundle_imports_symbols_from_bundle_root_modules() {
    let source = "bundle demo\nmodule first =\n\tlet value : core::Integer = 1\nend\nmodule second =\n\tuse bundle::first\n\tlet result : core::Integer = value\nend\n";
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
fn use_bundle_without_suffix_opens_current_bundle_root() {
    let source = "bundle demo\nmodule helper =\n\tlet token : core::Integer = 1\nend\nmodule consumer =\n\tuse bundle\n\tlet value : core::Integer = helper::token\nend\n";
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
fn use_bundle_expression_without_suffix_opens_current_bundle_root() {
    let source = "bundle demo\nmodule helper =\n\tlet token : core::Integer = 1\nend\nmodule consumer =\n\tlet value : core::Integer = use bundle in helper::token\nend\n";
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
    let source = "module demo =\n\tmodule helper =\n\t\tlet token = 1\n\tend\n\tlet before = token\n\tuse helper\n\tlet after = token\nend\n";
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
    let source = "module demo =\n\tmodule M =\n\t\tmodule N =\n\t\t\tlet x = 1\n\t\tend\n\tend\n\tuse M\n\tlet value : core::Integer = N::x\nend\n";
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
    let source = "module demo =\n\tmodule helper =\n\t\tlet token = 1\n\tend\n\tuse helper\n\tmodule inner =\n\t\tlet value = token\n\tend\nend\n";
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
fn use_alias_binds_module_name_without_opening_contents() {
    let source = "module demo =\n\tuse core as c\n\tlet value = c::array::empty\nend\n";
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
    let source = "module demo =\n\tmodule helper =\n\t\tlet token = 1\n\tend\n\tuse helper as h\n\tlet value = token\nend\n";
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
    let source = "module demo =\n\tmodule helper =\n\t\tlet token = 1\n\tend\n\tlet inside = use helper in token\n\tlet outside = token\nend\n";
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
    let source = "module demo =\n\tlet value = use core as c in c::array::empty\nend\n";
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
    let source = "bundle core\nmodule prelude =\n\tlet answer = 1\nend\n\nmodule demo =\n\tlet value = answer\nend\n";
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
    let source =
        "module core =\n\tlet local = 1\nend\n\nmodule demo =\n\tlet value = answer\nend\n";
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
    let source = "bundle demo
trait Id : t =
  let id : t -> t
end

impl Id for a in a =
  let id = fn x => x
end

let value = id 1
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
        "a",
        "expected generic impl head argument"
    );
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "forall impl head should type-check");
}

#[test]
fn constructor_alias_creates_term_and_pattern_name() {
    let source = "bundle demo\ntype Option : a = | None | Some a\nlet | Just = Some\nlet make = Just 1\nlet unwrap_or = fn fallback value =>\n\tmatch value with\n\t| Just inner => inner\n\t| None => fallback\nlet result : core::Integer = unwrap_or 0 (Just 7)\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "constructor alias should compile and validate");
    assert!(
        symbols.terms().contains_key(&Path::new("demo", "Just")),
        "expected constructor alias to be a term"
    );
    assert!(
        symbols.constructors().contains(&Path::new("demo", "Just")),
        "expected constructor alias to be a constructor"
    );
}

#[test]
fn constructor_alias_is_available_via_use_imports() {
    let source = "bundle demo\nmodule first =\n\ttype Option : a = | None | Some a\n\tlet | Just = Some\n\tlet | Nothing = None\nend\n\nmodule second =\n\tuse root::demo::first\n\tlet value = Just 3\n\tlet result : core::Integer =\n\t\tmatch value with\n\t\t| Just n => n\n\t\t| Nothing => 0\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "constructor aliases should import through use");
}

#[test]
fn constructor_alias_rhs_must_resolve_in_constructor_namespace() {
    let source = "module demo =\n\tlet value = 1\n\tlet | Alias = value\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_has_error_message(
        &file_logger,
        "Undefined constructor"
    ));
    logger.consume_file(file_logger);
    assert!(!logger.is_ok(), "Compilation should fail");
}

#[test]
fn constructor_alias_name_lints_when_not_pascal_case() {
    let source = "module demo =\n\ttype Option : a = | None | Some a\n\tlet | some = Some\nend\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _ = compile_source(source, &mut file_logger, &mut symbols);
    assert!(file_logger_count_message(&file_logger, "Naming style") >= 1);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "constructor alias naming lint should be a warning");
}

#[test]
fn named_type_expression_def_produces_wrapper_constructor() {
    let source = "bundle demo\ntype Int = core::Integer\nlet wrapped : Int = Int 1\nlet unwrap = fn (value: Int) =>\n\tmatch value with\n\t| Int inner => inner\nlet result : core::Integer = unwrap wrapped\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "named type constructors should compile");
    assert!(
        symbols.terms().contains_key(&Path::new("demo", "Int")),
        "expected named type constructor to be available as term"
    );
    assert!(
        symbols.constructors().contains(&Path::new("demo", "Int")),
        "expected named type constructor to be available as constructor"
    );
}

#[test]
fn sum_type_does_not_publish_typename_constructor() {
    let source = "bundle demo\ntype Option : a = | None | Some a\nlet value : Option core::Integer = Some 1\n";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(&logger, "sum constructors should still compile");
    assert!(
        !symbols
            .constructors()
            .contains(&Path::new("demo", "Option"))
    );
    assert!(symbols.constructors().contains(&Path::new("demo", "Some")));
    assert!(symbols.constructors().contains(&Path::new("demo", "None")));
}

#[test]
fn spread_tail_array_pattern_does_not_redefine_local_symbol() {
    let source = "module demo =
\tlet take_tail = fn arr => match arr with
\t\t| [] => []
\t\t| [_, ..tail] => tail
end
";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(
        &logger,
        "spread-tail array patterns should not redefine locals during lowering",
    );
}

#[test]
fn nested_array_pattern_lowers_without_wasm_type_mismatch() {
    let source = "module demo =
\tlet flatten = fn arr => match arr with
\t\t| [] => []
\t\t| [head, ..tail] => head + (flatten tail)
\tlet value = flatten [[1], [2], [3]]
end
";
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);

    for artifact in artifacts.into_vec() {
        let _ = validate_artifact(artifact, &mut logger);
    }

    assert_logger_is_ok(
        &logger,
        "nested array patterns should encode with consistent wasm array types",
    );
}

#[test]
fn core_resolves_array_equal_specialization_for_integer_arrays() {
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    assert_logger_is_ok(&logger, "core should compile");

    let specialization = symbols
        .resolve_method_specialization(
            &Path::new("core", "ops::[==]"),
            &[Type::Array(Box::new(Type::Integer))],
        )
        .expect("method specialization lookup should not fail");

    let specialization =
        specialization.expect("expected array equality specialization for integer arrays");
    assert_ne!(
        specialization.impl_method_path,
        Path::new("core", "ops::[==]"),
        "array equality should dispatch to an impl method, not the trait item"
    );

    let method_scheme = symbols
        .terms()
        .get(&specialization.impl_method_path)
        .unwrap_or_else(|| {
            panic!(
                "missing impl method scheme {}",
                specialization.impl_method_path
            )
        });
    assert!(
        !method_scheme.predicates.is_empty(),
        "array equality impl method should keep required predicates; got type {} with {} predicates",
        method_scheme.type_.pretty(),
        method_scheme.predicates.len()
    );
}

#[test]
fn core_resolves_array_equal_specialization_for_string_arrays() {
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    assert_logger_is_ok(&logger, "core should compile");

    let specialization = symbols
        .resolve_method_specialization(
            &Path::new("core", "ops::[==]"),
            &[Type::Array(Box::new(Type::String))],
        )
        .expect("method specialization lookup should not fail");

    let specialization =
        specialization.expect("expected array equality specialization for string arrays");
    assert_ne!(
        specialization.impl_method_path,
        Path::new("core", "ops::[==]"),
        "array equality should dispatch to an impl method, not the trait item"
    );

    let method_scheme = symbols
        .terms()
        .get(&specialization.impl_method_path)
        .unwrap_or_else(|| {
            panic!(
                "missing impl method scheme {}",
                specialization.impl_method_path
            )
        });
    assert!(
        !method_scheme.predicates.is_empty(),
        "array equality impl method should keep required predicates; got type {} with {} predicates",
        method_scheme.type_.pretty(),
        method_scheme.predicates.len()
    );
}

#[test]
fn core_compare_trait_item_retains_compare_predicate() {
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    assert_logger_is_ok(&logger, "core should compile");

    let compare_method = symbols
        .terms()
        .get(&Path::new("core", "ops::[>]"))
        .expect("missing core compare trait item");

    assert!(
        compare_method
            .predicates
            .iter()
            .any(|predicate| predicate.trait_name == Path::new("core", "ops::Compare")),
        "core::ops::[>] should require the Compare predicate"
    );
}

#[test]
fn core_ge_operator_retains_compare_and_equal_predicates() {
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    assert_logger_is_ok(&logger, "core should compile");

    let ge = symbols
        .terms()
        .get(&Path::new("core", "ops::[>=]"))
        .expect("missing core >= operator");

    let has_compare = ge
        .predicates
        .iter()
        .any(|predicate| predicate.trait_name == Path::new("core", "ops::Compare"));
    let has_equal = ge
        .predicates
        .iter()
        .any(|predicate| predicate.trait_name == Path::new("core", "ops::Equal"));

    let predicate_names = ge
        .predicates
        .iter()
        .map(|predicate| predicate.trait_name.to_string())
        .collect::<Vec<_>>();

    assert!(
        has_compare,
        "core::ops::[>=] should require Compare; got predicates {:?} and type {}",
        predicate_names,
        ge.type_.pretty()
    );
    assert!(
        has_equal,
        "core::ops::[>=] should require Equal; got predicates {:?} and type {}",
        predicate_names,
        ge.type_.pretty()
    );
}

#[test]
fn prelude_gt_alias_retains_compare_predicate() {
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    assert_logger_is_ok(&logger, "core should compile");

    let prelude_gt = symbols
        .terms()
        .get(&Path::new("core", "prelude::[>]"))
        .expect("missing prelude > alias");

    assert!(
        prelude_gt
            .predicates
            .iter()
            .any(|predicate| predicate.trait_name == Path::new("core", "ops::Compare")),
        "prelude::[>] should require Compare"
    );
}

#[test]
fn user_defined_ge_function_infers_compare_and_equal_predicates() {
    let source = "module demo =\n\tlet ge = fn left right => if left > right then true else left == right\nend\n";

    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let _core = compile_core_module(&mut symbols, &mut logger);
    let mut file_logger = logger.new_file("demo.hc", source);
    let _artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert_logger_is_ok(&logger, "demo source should compile");

    let ge = symbols
        .terms()
        .iter()
        .find_map(|(path, scheme)| path.minor.ends_with("ge").then_some(scheme))
        .expect("missing ge symbol");

    let has_compare = ge
        .predicates
        .iter()
        .any(|predicate| predicate.trait_name == Path::new("core", "ops::Compare"));
    let has_equal = ge
        .predicates
        .iter()
        .any(|predicate| predicate.trait_name == Path::new("core", "ops::Equal"));

    let predicate_names = ge
        .predicates
        .iter()
        .map(|predicate| predicate.trait_name.to_string())
        .collect::<Vec<_>>();

    assert!(
        has_compare,
        "demo::ge should require Compare; got predicates {:?} and type {}",
        predicate_names,
        ge.type_.pretty()
    );
    assert!(
        has_equal,
        "demo::ge should require Equal; got predicates {:?} and type {}",
        predicate_names,
        ge.type_.pretty()
    );
}

#[test]
fn debug_dump_core_wat_for_big_num_validation() {
    let mut symbols = SymbolTable::new();
    let mut logger = Logger::new();
    let core = compile_core_module(&mut symbols, &mut logger);
    assert!(logger.is_ok(), "core compile should succeed for debug dump");
    let wat = core
        .decompile_to_wat()
        .expect("core binary should decompile for debug dump");
    std::fs::write("/tmp/core_debug.wat", wat).expect("write debug wat");
}
