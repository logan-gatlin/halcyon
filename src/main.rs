use halcyon_lib::hc_core::CoreImpl;
use halcyon_lib::{
    Logger,
    compile_source,
};

extern crate halcyon_lib;

fn main() {
    enum_iterator::all::<CoreImpl>().for_each(|i| println!("{i:?}"));
    let source = include_str!("test/demo.hc");
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let modules = compile_source(source, &mut file_logger);
    for module in modules {
        println!("{module:#?}");
    }
    logger.consume_file(file_logger);
    logger.print_logs();
}
