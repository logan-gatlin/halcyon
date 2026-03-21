#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;

use halcyon_lib::parse::lexer;
use halcyon_lib::{
    Logger,
    Span,
};

use common::bounded_source;

fuzz_target!(|data: &[u8]| {
    let source = bounded_source(data, 65_536);
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("fuzz.hc", source.clone());
    let tokens = lexer::tokenize(source.chars(), &mut file_logger);

    for token in tokens {
        if let Span::Source { start, width, .. } = token.span {
            assert!(start <= source.len());
            assert!(start.saturating_add(width) <= source.len());
        }
    }

    logger.consume_file(file_logger);
});
