use halcyon_lib::types::SymbolTable;
use halcyon_lib::{
    compile_core_module,
    compile_source,
    validate_artifact,
    Logger,
};

extern crate halcyon_lib;

#[allow(clippy::print_stdout)]
fn main() {
    let source = include_str!("test/demo.hc");
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let mut symbol_table = SymbolTable::new();
    compile_core_module(&mut symbol_table);
    let modules = compile_source(source, &mut file_logger, &mut symbol_table);
    modules
        .into_iter()
        .map(|a| validate_artifact(a, &mut logger))
        .flat_map(|a| a.ir_module)
        .for_each(|ir| println!("{ir}"));
    logger.consume_file(file_logger);
    logger.print_logs();
}
