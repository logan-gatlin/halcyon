use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::StandardStream;

use crate::hc_core::core_symbol_table;

use super::*;

/// Tests that a file passes the typechecker
macro_rules! typecheck {
    ($($name:ident)*) => {
        $(
            #[test]
            fn $name() {
                let name = stringify! {$name};
                let path = format!("src/test/{name}.hc");
                let input = std::fs::read_to_string(&path).expect("Failed to read test file");

                let mut symbols = core_symbol_table();
                let mut files = SimpleFiles::new();
                let file_id = files.add(path, input.clone());
                let mut logger = Logger::new(file_id);

                let tokens = tokenize(input.chars(), &mut logger);
                let parse_trees = parse(&mut logger, tokens);
                for tree in parse_trees {
                    let mut ir_module = build_ir(&mut logger, &mut symbols, tree);
                    semantic::analyze(&mut ir_module, &mut symbols, &mut logger);
                }
                if !logger.is_ok() {
                    let mut writer =
                        StandardStream::stderr(codespan_reporting::term::termcolor::ColorChoice::Always);
                    let config = codespan_reporting::term::Config {
                        display_style: term::DisplayStyle::Rich,
                        ..Default::default()
                    };
                    for d in &logger {
                        term::emit_to_write_style(&mut writer, &config, &files, d).unwrap();
                    }
                }
            }
        )*
    }
}

typecheck! {
    demo
    inference
}
