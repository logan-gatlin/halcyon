use halcyon_lib::{
    compile_source,
    Logger,
};

extern crate halcyon_lib;

fn main() {
    let source = include_str!("test/demo.hc");
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file("demo.hc", source);
    let modules = compile_source(source, &mut file_logger);
    for module in modules {
        println!("{}", module.pretty());
    }
    logger.consume_file(file_logger);
    logger.print_logs();
}
