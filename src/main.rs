extern crate halcyon_lib;
use codespan_reporting::diagnostic::{
    Diagnostic,
    Label,
};
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
        //let asm_module = asm::lower_module(ir_module, symbols);
        //eprintln!("{}", asm_module.pretty());
        //asm::encode(asm_module)
        vec![]
    } else {
        eprintln!("Output is empty");
        std::process::exit(1);
    }
}

/// Find the WAT line number (1-indexed) for a given binary offset.
/// Returns the line whose binary offset is the largest that doesn't exceed `target_offset`.
fn find_wat_line_for_offset(
    offset_map: &[(usize, Option<usize>)],
    target_offset: usize,
) -> usize {
    let mut best_line = 1;
    for &(line, offset) in offset_map {
        if let Some(off) = offset
            && off <= target_offset
        {
            best_line = line;
        }
    }
    best_line
}

/// Compute the byte offset in the WAT string for the start of a given line (1-indexed).
fn wat_line_byte_offset(
    wat: &str,
    line: usize,
) -> usize {
    wat.lines()
        .take(line.saturating_sub(1))
        .map(|l| l.len() + 1) // +1 for newline
        .sum()
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
    let wat_config = codespan_reporting::term::Config {
        display_style: term::DisplayStyle::Short,
        ..Default::default()
    };

    // Emit source-level diagnostics
    for d in &logger {
        term::emit_to_write_style(&mut writer, &config, &files, d).unwrap();
    }
    if !logger.is_ok() {
        std::process::exit(1);
    }

    // Generate WAT with offset mapping for validation error reporting
    let mut wat_storage = String::new();
    let offset_map: Vec<(usize, Option<usize>)> = wasmprinter::Config::new()
        .offsets_and_lines(&bin, &mut wat_storage)
        .map(|iter| {
            iter.enumerate()
                .map(|(idx, (offset, _text))| (idx + 1, offset)) // 1-indexed lines
                .collect()
        })
        .unwrap_or_default();

    if let Some(path) = args.next() {
        std::fs::write(path, &bin).unwrap();
    } else {
        println!("{wat_storage}");
    }

    // Validate the binary and report errors with WAT line numbers
    if let Err(e) = wasmparser::validate(&bin) {
        let error_offset = e.offset();
        let wat_line = find_wat_line_for_offset(&offset_map, error_offset);
        let byte_start = wat_line_byte_offset(&wat_storage, wat_line);
        let byte_end = wat_line_byte_offset(&wat_storage, wat_line + 1).min(wat_storage.len());

        let mut wat_files: SimpleFiles<&str, &str> = SimpleFiles::new();
        let wat_file_id = wat_files.add("<generated wat>", &wat_storage);

        let diagnostic: Diagnostic<usize> = Diagnostic::error()
            .with_message(e.message())
            .with_labels(vec![Label::primary(wat_file_id, byte_start..byte_end)]);

        term::emit_to_write_style(&mut writer, &wat_config, &wat_files, &diagnostic).unwrap();
        std::process::exit(1);
    }
}

fn main() {
    compile_file_arg();
}
