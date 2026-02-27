/*!
    End-to-end testing for the compiler
*/
#![allow(clippy::unwrap_used)]

use super::*;
use crate::hc_core::compile_core_module;
use crate::types::SymbolTable;

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
        let module = Module::new(&engine, &artifact.binary).unwrap();
        let instance = linker.instantiate(&mut store, &module).unwrap();
        linker
            .instance(&mut store, &artifact.module_name, instance)
            .unwrap();
    }
}

#[test]
fn demo() {
    let source = include_str!("demo.hc");
    let mut symbols = SymbolTable::new();
    compile_core_module(&mut symbols);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let artifacts = compile_source(source, &mut file_logger, &mut symbols);
    logger.consume_file(file_logger);
    assert!(logger.is_ok(), "Compilation failed");
    exec_file(&artifacts);
}
