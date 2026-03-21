#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;

use halcyon_lib::parse::ast::HasName;
use halcyon_lib::{
    Logger,
    asm,
    ir,
    parse,
    types,
};

use common::{
    ByteCursor,
    bounded_source,
    core_symbols,
};

fn mutate_ir_module(
    module: ir::Module<()>,
    data: &[u8],
) -> ir::Module<()> {
    let mut cursor = ByteCursor::new(data);
    let ir::Module { name, statements } = module;
    let mut statements = statements.into_vec();

    if statements.is_empty() {
        return ir::Module {
            name,
            statements: statements.into_boxed_slice(),
        };
    }

    if cursor.next_bool() {
        let keep = cursor.next_usize(statements.len().saturating_add(1));
        statements.truncate(keep);
        if statements.is_empty() {
            return ir::Module {
                name,
                statements: statements.into_boxed_slice(),
            };
        }
    }

    if cursor.next_bool() {
        let rotate = cursor.next_usize(statements.len());
        statements.rotate_left(rotate);
    }

    if cursor.next_bool() {
        let duplicate_index = cursor.next_usize(statements.len());
        let duplicate = statements[duplicate_index].clone();
        let insert_at = cursor.next_usize(statements.len().saturating_add(1));
        statements.insert(insert_at, duplicate);
    }

    if cursor.next_bool() && statements.len() > 1 {
        let remove_index = cursor.next_usize(statements.len());
        let _ = statements.remove(remove_index);
    }

    ir::Module {
        name,
        statements: statements.into_boxed_slice(),
    }
}

fuzz_target!(|data: &[u8]| {
    let mut source = String::from("bundle fuzz\n");
    source.push_str(&bounded_source(data, 16_384));

    let mut logger = Logger::new();
    let mut parse_logger = logger.new_file("fuzz.hc", source.clone());
    let Some(source_file) = parse::parse(&source, &mut parse_logger) else {
        logger.consume_file(parse_logger);
        return;
    };

    let bundle_name = source_file
        .bundle_declaration()
        .and_then(|declaration| declaration.name_text())
        .unwrap_or_else(|| "fuzz".to_string());
    let Some(module) =
        ir::bundle_statements(bundle_name, &source_file.statements(), &mut parse_logger)
    else {
        logger.consume_file(parse_logger);
        return;
    };
    logger.consume_file(parse_logger);

    let module = mutate_ir_module(module, data);
    let mut symbols = core_symbols();

    let mut type_logger = logger.new_file("fuzz.hc", source);
    let resolved =
        types::resolve_module_with_symbols_and_schemes(&mut symbols, module, &mut type_logger);
    logger.consume_file(type_logger);
    if !logger.is_ok() {
        return;
    }

    let elaborated = ir::elaborate_module(resolved, &symbols);
    let _ = asm::compile_module(
        elaborated,
        &symbols,
        &Vec::new(),
        asm::DebugInfoOptions::none(),
    );
});
