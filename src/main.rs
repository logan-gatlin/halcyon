extern crate halcyon_lib;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::StandardStream;
use halcyon_lib::*;

pub fn compile(
    input: &str,
    logger: &mut Logger,
    symbols: &mut SymbolTable,
) -> Vec<u8> {
    let tokens = tokenize(input.chars(), logger);
    let parse_trees = parse(logger, tokens);

    for p in parse_trees {
        let ir_module = build_ir(logger, symbols, p);
        println!("{}\n", ir_module.pretty());
    }
    vec![]
}

fn compile_file_arg() {
    let mut args = std::env::args().skip(1);
    assert_eq!(args.len(), 1, "Expected 1 path argument");
    let path = args.next().unwrap();
    let str = std::fs::read_to_string(&path).expect("Could not open file");
    let mut files = SimpleFiles::new();
    let file_id = files.add(path, str.clone());
    let mut logger = Logger::new(file_id);
    let mut symbols = SymbolTable::new();
    let _bytes = compile(&str, &mut logger, &mut symbols);
    let mut writer =
        StandardStream::stdout(codespan_reporting::term::termcolor::ColorChoice::Always);
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
}

fn main() {
    compile_file_arg();
}
