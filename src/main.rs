extern crate halcyon_lib;
use std::path::PathBuf;

use codespan_reporting::files::SimpleFiles;
use halcyon_lib::asm::validate_wasm;
use halcyon_lib::hc_core::core_symbol_table;
use halcyon_lib::*;

fn compile_file_arg() {
    let mut args = std::env::args().skip(1);
    if args.len() < 1 || args.len() > 2 {
        eprintln!("Usage: halcyon <file-path> <optional-output-path>");
        std::process::exit(1);
    }
    let path = args.next().unwrap();
    let str = std::fs::read_to_string(&path).expect("Could not open file");
    let mut files = SimpleFiles::new();
    let file_id = files.add(path, str.clone());
    let mut logger = Logger::new(file_id);
    let mut symbols = core_symbol_table();
    let mut bins = vec![];

    #[allow(unused_variables)]
    for Artifact {
        module_name,
        parse_tree,
        ir_module,
        asm_module,
        binary,
    } in compile(&str, &mut logger, &mut symbols)
    {
        /*
        eprintln!("{}", ir_module.pretty());
        if let Some(asm_module) = asm_module {
            eprintln!("{}", asm_module.pretty());
        }
        */
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
        validate_wasm(bin, &mut logger)
    }
    // Emit source-level diagnostics
    logger.print(&files);
    if !logger.is_ok() {
        std::process::exit(1);
    }
}

fn main() {
    compile_file_arg();
}
