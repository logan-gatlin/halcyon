use halcyon_lib::types::SymbolTable;
use halcyon_lib::{
    Artifact,
    Logger,
    compile_core_module,
    compile_source,
    validate_artifact,
};
use wasmtime::{
    Config,
    Engine,
    Linker,
    Module,
    Store,
};
use wasmtime_wasi::p2::WasiCtxBuilder;
use wasmtime_wasi::preview1;

extern crate halcyon_lib;

fn link_and_run_artifacts(artifacts: &[Artifact]) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();
    config.wasm_gc(true);
    config.wasm_reference_types(true);
    config.wasm_function_references(true);

    let engine = Engine::new(&config)?;
    let mut linker: Linker<preview1::WasiP1Ctx> = Linker::new(&engine);
    preview1::add_to_linker_sync(&mut linker, |ctx| ctx)?;

    let mut builder = WasiCtxBuilder::new();
    builder.inherit_stdio();
    builder.inherit_args();
    let wasi = builder.build_p1();
    let mut store = Store::new(&engine, wasi);

    for artifact in artifacts {
        let module = Module::new(&engine, &artifact.binary)?;
        linker.module(&mut store, &artifact.module_name, &module)?;
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let source = "module demo =\n\tlet _ = core::print_string \"hello from halcyon\\n\"\nend\n";
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let mut symbol_table = SymbolTable::new();

    let core = validate_artifact(compile_core_module(&mut symbol_table), &mut logger);
    let modules = compile_source(source, &mut file_logger, &mut symbol_table)
        .into_iter()
        .map(|artifact| validate_artifact(artifact, &mut logger))
        .collect::<Vec<_>>();

    logger.consume_file(file_logger);
    logger.print_logs();
    if !logger.is_ok() {
        return Err("Compilation failed".into());
    }

    let mut artifacts = Vec::with_capacity(modules.len() + 1);
    artifacts.push(core);
    artifacts.extend(modules);
    link_and_run_artifacts(&artifacts)
}

#[allow(clippy::print_stdout)]
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
