#![allow(clippy::unwrap_used)]
/*!
    End-to-end testing for the compiler
*/
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::StandardStream;

use crate::hc_core::core_symbol_table;

use super::*;

fn compile_file(name: &str) -> Vec<Vec<u8>> {
    let path = format!("src/test/{name}.hc");
    let input = std::fs::read_to_string(&path).expect("Failed to read test file");

    let mut symbols = core_symbol_table();
    let mut files = SimpleFiles::new();
    let file_id = files.add(path, input.clone());
    let mut logger = Logger::new(file_id);

    let mut bins = vec![];

    let tokens = tokenize(input.chars(), &mut logger);
    let parse_trees = parse(&mut logger, tokens);
    for tree in parse_trees {
        let mut ir_module = build_ir(&mut logger, &mut symbols, tree);
        semantic::analyze(&mut ir_module, &mut symbols, &mut logger);
        let asm_module = asm::lower_module(&ir_module, &symbols);
        bins.push(asm::encode(asm_module));
    }
    if !logger.is_ok() {
        let mut writer =
            StandardStream::stderr(codespan_reporting::term::termcolor::ColorChoice::Always);
        let config = codespan_reporting::term::Config {
            display_style: term::DisplayStyle::Rich,
            ..Default::default()
        };
        for d in &logger {
            term::emit_to_write_style(&mut writer, &config, &files, d).unwrap();
        }
    }
    bins
}

fn exec_file(wasms: &[Vec<u8>]) {
    use wasmtime::*;
    let mut config = Config::default();
    config.wasm_gc(true);
    config.wasm_function_references(true);
    config.debug_info(true);
    config.wasm_backtrace_details(WasmBacktraceDetails::Enable);
    let engine = Engine::new(&config).unwrap();
    let mut linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    for wasm in wasms.iter() {
        let module = Module::new(&engine, wasm).unwrap();
        let instance = linker.instantiate(&mut store, &module).unwrap();
    }
}

#[test]
fn demo() {
    let bins = compile_file("demo");
    exec_file(&bins);
}
