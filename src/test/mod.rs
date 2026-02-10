/*!
    End-to-end testing for the compiler
*/
#![allow(clippy::unwrap_used)]

use super::*;

fn compile_file(name: &str) -> Vec<Artifact> {
    let path = format!("src/test/{name}.hc");
    let input = std::fs::read_to_string(&path).expect("Failed to read test file");
    let mut logger = Logger::new();
    let mut symbols = SymbolTable::new();
    let arts = compile_source(&path, &input, &mut logger, &mut symbols);
    logger.print_logs();
    assert!(logger.is_ok());
    arts
}

fn exec_file(artifacts: &[Artifact]) {
    use wasmtime::*;
    let mut config = Config::default();
    config.wasm_gc(true);
    config.wasm_function_references(true);
    config.debug_info(true);
    config.wasm_backtrace_details(WasmBacktraceDetails::Enable);
    let engine = Engine::new(&config).unwrap();
    let mut linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    for artifact in artifacts {
        let module = Module::new(&engine, &artifact.binary).unwrap();
        let instance = linker.instantiate(&mut store, &module).unwrap();
        linker
            .instance(&mut store, &artifact.module_name, instance)
            .unwrap();
    }
}

#[test]
fn demo() {
    let artifacts = compile_file("demo");
    exec_file(&artifacts);
}
