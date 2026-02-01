extern crate halcyon_lib;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::StandardStream;
use halcyon_lib::hc_core::core_symbol_table;
use halcyon_lib::*;

pub fn compile(
    input: &str,
    logger: &mut Logger,
    symbols: &mut SymbolTable,
) -> Vec<u8> {
    let tokens = tokenize(input.chars(), logger);
    let parse_trees = parse(logger, tokens);

    if let Some(p) = parse_trees.into_iter().next() {
        let mut ir_module = build_ir(logger, symbols, p);
        semantic::analyze(&mut ir_module, symbols, logger);
        eprintln!("{}\n", ir_module.pretty());
        let asm_module = asm::lower_module(ir_module, symbols);
        //eprintln!("{}", asm_module.pretty());
        let bin = asm::encode(asm_module);
        if let Err(e) = wasmparser::validate(&bin) {
            eprintln!("FAILED TO VALIDATE:");
            eprintln!("{e}");
        };
        bin
    } else {
        unreachable!()
    }
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
    let bin = compile(&str, &mut logger, &mut symbols);
    let mut writer =
        StandardStream::stderr(codespan_reporting::term::termcolor::ColorChoice::Always);
    let config = codespan_reporting::term::Config {
        display_style: term::DisplayStyle::Rich,
        ..Default::default()
    };
    for d in &logger {
        term::emit_to_write_style(&mut writer, &config, &files, d).unwrap();
    }
    if !logger.is_ok() {
        std::process::exit(1);
    }
    if let Some(path) = args.next() {
        std::fs::write(path, bin).unwrap();
    } else {
        let wat = wasmprinter::print_bytes(&bin).unwrap();
        println!("{wat}");
    }
}

fn main() {
    compile_file_arg();
}
