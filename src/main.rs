extern crate halcyon_lib;
use std::path::PathBuf;

use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::StandardStream;
use halcyon_lib::asm::validate_wasm;
use halcyon_lib::hc_core::core_symbol_table;
use halcyon_lib::*;

pub fn compile(
    input: &str,
    logger: &mut Logger,
    symbols: &mut SymbolTable,
) -> Vec<Vec<u8>> {
    let tokens = tokenize(input.chars(), logger);
    let parse_trees = parse(logger, tokens);
    let mut bins = vec![];

    for p in parse_trees {
        let mut ir_module = build_ir(logger, symbols, p);
        semantic::analyze(&mut ir_module, symbols, logger);
        //eprintln!("{}\n", ir_module.pretty());
        let asm_module = asm::lower_module(&ir_module, symbols);
        //eprintln!("{}", asm_module.pretty());
        bins.push(asm::encode(asm_module));
    }
    bins
}

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
    let bins = compile(&str, &mut logger, &mut symbols);

    let mut writer =
        StandardStream::stderr(codespan_reporting::term::termcolor::ColorChoice::Always);
    let config = codespan_reporting::term::Config {
        display_style: term::DisplayStyle::Rich,
        ..Default::default()
    };
    // Emit source-level diagnostics
    for d in &logger {
        term::emit_to_write_style(&mut writer, &config, &files, d).unwrap();
    }
    if !logger.is_ok() {
        std::process::exit(1);
    }
    let output_path = args.next();
    for (i, bin) in bins.iter().enumerate() {
        // Generate WAT with offset mapping for validation error reporting
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
        if let Err(e) = validate_wasm(bin) {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

fn main() {
    compile_file_arg();
}
