use clap::{Parser, Subcommand};
use halcyon_lib::*;

#[derive(Debug, Subcommand)]
enum Commands {
    /// Validate the program without producing output
    #[command(flatten_help = true)]
    Check { input_path: std::path::PathBuf },
    /// Compile and link the program
    Build {
        input_path: std::path::PathBuf,
        /// Output file to write to (defaults to 'out.wasm')
        #[arg(short('o'), long, default_value("out.wasm"))]
        output: std::path::PathBuf,
    },
    /// Compile, link, and execute the program
    Run { input_path: std::path::PathBuf },
}

#[derive(Parser, Debug)]
#[command(name = "hcc", about = "Halcyon compiler")]
struct CmdArgs {
    #[command(subcommand)]
    command: Commands,
}

pub fn execute(wasm: Vec<u8>) {
    use wasmtime::*;
    let mut config = Config::default();
    config.wasm_gc(true);
    config.wasm_function_references(true);
    let engine = Engine::new(&config).unwrap();
    let module = Module::new(&engine, &wasm).unwrap();
    let mut linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    let memory = Memory::new(&mut store, MemoryType::new(1, None)).unwrap();
    linker
        .func_wrap(
            "sys",
            "print_string",
            move |_callee: Caller<'_, ()>, ptr: i32, len: i32| {
                let mut buffer = vec![0; len as usize];
                memory.read(_callee, ptr as usize, &mut buffer).unwrap();
                let s = String::from_utf8(buffer).unwrap();
                println!("{s}");
            },
        )
        .unwrap()
        .define(&mut store, "sys", "memory", Extern::Memory(memory))
        .unwrap();
    let _instance = linker.instantiate(&mut store, &module).unwrap();
}

fn hcc_main() -> std::result::Result<(), String> {
    let args = CmdArgs::parse();
    match args.command {
        Commands::Check { .. } => todo!(),
        Commands::Build { input_path, output } => {
            let file = std::fs::read(input_path).map_err(|e| e.to_string())?;
            let binary = compile(&String::from_utf8_lossy(&file))?;
            std::fs::write(output, binary).map_err(|e| e.to_string())?;
        }
        Commands::Run { input_path } => {
            let file = std::fs::read(input_path).map_err(|e| e.to_string())?;
            let binary = compile(&String::from_utf8_lossy(&file))?;
            execute(binary)
        }
    }
    Ok(())
}

fn main() {
    match hcc_main() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
