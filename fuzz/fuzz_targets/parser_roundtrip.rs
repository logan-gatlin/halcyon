#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;

use halcyon_lib::parse::ast::{
    self,
    AstNode,
    HasName,
};
use halcyon_lib::{
    Logger,
    parse,
};

use common::bounded_source;

fn exercise_statement(statement: ast::Statement) {
    match statement {
        ast::Statement::Bundle(bundle) => {
            let _ = bundle.name_text();
            let _ = bundle.name_text_spanned();
        }
        ast::Statement::Import(import_statement) => {
            let _ = import_statement.path_literals();
        }
        ast::Statement::Use(use_statement) => {
            let _ = use_statement.target();
            let _ = use_statement.alias_name_spanned();
        }
        ast::Statement::Let(let_statement) => {
            let _ = let_statement.is_pattern_alias();
            let _ = let_statement.alias_name_spanned();
            let _ = let_statement.alias_target();
            let _ = let_statement.pattern();
            let _ = let_statement.value();
        }
        ast::Statement::Do(do_statement) => {
            let _ = do_statement.value();
        }
        ast::Statement::Type(type_statement) => {
            let _ = type_statement.name_text();
            let _ = type_statement.is_alias();
            let _ = type_statement.type_params();
            let _ = type_statement.type_def();
        }
        ast::Statement::Trait(trait_statement) => {
            let _ = trait_statement.name_text();
            let _ = trait_statement.is_alias();
            let _ = trait_statement.alias_target();
            let _ = trait_statement.trait_params();
            for method in trait_statement.methods() {
                let _ = method.name_text();
                let _ = method.ty();
            }
        }
        ast::Statement::Impl(impl_statement) => {
            let _ = impl_statement.trait_name();
            let _ = impl_statement.type_args();
            for method in impl_statement.methods() {
                let _ = method.name_text();
                let _ = method.value();
            }
        }
        ast::Statement::Module(module) => {
            let _ = module.name_text();
            for statement in module.statements() {
                exercise_statement(statement);
            }
        }
        ast::Statement::Wasm(wasm_statement) => {
            if let Some(sexpr) = wasm_statement.sexpr() {
                let _ = sexpr.items();
            }
        }
    }
}

fn exercise_casts(source_file: &ast::SourceFile) {
    let file_id = source_file.file_id();
    let mut stack = vec![source_file.syntax().clone()];
    while let Some(node) = stack.pop() {
        let _ = ast::Statement::cast(node.clone()).map(|statement| statement.with_file_id(file_id));
        let _ = ast::TypeDef::cast(node.clone()).map(|type_def| type_def.with_file_id(file_id));
        let _ = ast::TypeExpr::cast(node.clone()).map(|type_expr| type_expr.with_file_id(file_id));
        let _ = ast::Expr::cast(node.clone()).map(|expr| expr.with_file_id(file_id));
        let _ = ast::Pattern::cast(node.clone()).map(|pattern| pattern.with_file_id(file_id));
        let _ = ast::Sexpr::cast(node.clone()).map(|sexpr| sexpr.with_file_id(file_id));
        stack.extend(node.children());
    }
}

fuzz_target!(|data: &[u8]| {
    let source = bounded_source(data, 65_536);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("fuzz.hc", source.clone());
    let Some(source_file) = parse::parse(&source, &mut file_logger) else {
        logger.consume_file(file_logger);
        return;
    };

    assert_eq!(source_file.syntax().text().to_string(), source);

    for statement in source_file.items() {
        exercise_statement(statement);
    }
    let _ = source_file.bundle_declaration();
    let _ = source_file.modules();
    let _ = source_file.imports();
    let _ = source_file.statements();

    exercise_casts(&source_file);
    logger.consume_file(file_logger);
});
