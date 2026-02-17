extern crate halcyon_lib;

fn main() {
}
/*
use halcyon_lib::asm::validate_wasm;
use halcyon_lib::hc_core::compile_core_module;
use halcyon_lib::*;
use std::path::PathBuf;

fn compile_file_arg() {
    let mut args = std::env::args().skip(1);
    if args.len() < 1 || args.len() > 2 {
        eprintln!("Usage: halcyon <file-path> <optional-output-path>");
        std::process::exit(1);
    }
    let path = args.next().unwrap();
    let str = std::fs::read_to_string(&path).expect("Could not open file");
    let mut logger = Logger::new();
    let mut symbols = SymbolTable::new();
    let mut bins = vec![];

    //let p = parse_lossless::parse(&str, &mut logger.new_file(&path, &str));

    let core_module = compile_core_module(&mut logger, &mut symbols).unwrap();
    bins.push(core_module.into_binary());

    for artifact in compile_source(&path, &str, &mut logger, &mut symbols) {
        let binary = artifact.into_binary();
        if !binary.is_empty() {
            bins.push(binary);
        }
    }

    let output_path = args.next();
    for (i, bin) in bins.iter().enumerate() {
        if bin.is_empty() {
            continue;
        }
        let wat = wasmprinter::print_bytes(bin)
            .unwrap_or_else(|_| "Failed to parse generated WASM".into());
        if let Some(path) = &output_path {
            let path = PathBuf::from(path);
            let out = if bins.len() > 1 {
                let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                let ext = path
                    .extension()
                    .map(|e| e.to_string_lossy())
                    .unwrap_or_default();
                let name = if ext.is_empty() {
                    format!("{stem}_{i}")
                } else {
                    format!("{stem}_{i}.{ext}")
                };
                path.with_file_name(name)
            } else {
                path
            };
            std::fs::write(out, bin).unwrap();
        } else {
            println!("{wat}");
        }
        let wat_logger = validate_wasm(&path, bin, &mut logger);
        logger.consume_file(wat_logger);
        logger.print_logs();
        if !logger.is_ok() {
            std::process::exit(1);
        }
    }
    // Emit source-level diagnostics
    logger.print_logs();
    if !logger.is_ok() {
        std::process::exit(1);
    }
}

fn main() {
    compile_file_arg();
}
*/
