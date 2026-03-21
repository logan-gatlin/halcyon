#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;

use halcyon_lib::parse::ast::HasName;
use halcyon_lib::{
    Logger,
    ir,
    parse,
    types,
};

use common::{
    bounded_source,
    core_symbols,
};

fuzz_target!(|data: &[u8]| {
    let mut source = String::from("bundle fuzz\n");
    source.push_str(&bounded_source(data, 12_288));

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

    let mut symbols = core_symbols();
    let mut type_logger = logger.new_file("fuzz.hc", source);
    let _ = types::resolve_module_with_symbols_and_schemes(&mut symbols, module, &mut type_logger);
    logger.consume_file(type_logger);
});
