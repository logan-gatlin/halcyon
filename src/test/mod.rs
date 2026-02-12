/*!
    End-to-end testing for the compiler
*/
#![allow(clippy::unwrap_used)]

use super::*;
use crate::hc_core::compile_core_module;

fn compile_file(name: &str) -> Vec<Artifact> {
    let path = format!("src/test/{name}.hc");
    let input = std::fs::read_to_string(&path).expect("Failed to read test file");
    let mut logger = Logger::new();
    let mut symbols = SymbolTable::new();
    let mut arts = vec![];
    if let Some(core) = compile_core_module(&mut logger, &mut symbols) {
        arts.push(core);
    }
    arts.extend(compile_source(&path, &input, &mut logger, &mut symbols));
    logger.print_logs();
    assert!(logger.is_ok());
    arts
}

fn exec_file(artifacts: &[Artifact]) {
    use wasmtime::*;
    use wasmtime_wasi::p2::WasiCtxBuilder;
    use wasmtime_wasi::preview1::{
        self,
        WasiP1Ctx,
    };

    let mut config = Config::default();
    config.wasm_gc(true);
    config.wasm_function_references(true);
    config.debug_info(true);
    config.wasm_backtrace_details(WasmBacktraceDetails::Enable);
    let engine = Engine::new(&config).unwrap();
    let mut linker = Linker::<WasiP1Ctx>::new(&engine);

    preview1::add_to_linker_sync(&mut linker, |ctx| ctx).unwrap();

    let wasi_ctx = WasiCtxBuilder::new().inherit_stdout().build_p1();
    let mut store = Store::new(&engine, wasi_ctx);
    for artifact in artifacts {
        let module = Module::new(&engine, artifact.binary()).unwrap();
        let instance = linker.instantiate(&mut store, &module).unwrap();
        linker
            .instance(&mut store, artifact.module_name(), instance)
            .unwrap();
    }
}

#[test]
fn demo() {
    let artifacts = compile_file("demo");
    exec_file(&artifacts);
}
