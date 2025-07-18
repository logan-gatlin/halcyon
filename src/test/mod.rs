use super::*;

macro_rules! test {
  ($($name:ident),*) => {
    $(
      #[test]
      fn $name() {
        let path = "./src/test/".to_string() + stringify!($name) + ".hc";
        let file = std::fs::read_to_string(path).unwrap();
        let linter = Linter::new(file.clone());
        let _wasm = match _compile(&file) {
          Ok(wasm) => {wasm}
          Err(e) => {
            println!(
              "{}",
              "Failed to Compile".apply_style(Color::Red, Attribute::Underline),
            );
            println!("{}", linter.render(e));
            panic!();
          }
        };
        /*
        let mut config = wasmtime::Config::default();
        config.wasm_gc(true);
        config.wasm_function_references(true);
        let engine = wasmtime::Engine::new(&config).unwrap();
        let module = wasmtime::Module::new(&engine, &wasm).unwrap();
        let linker = wasmtime::Linker::new(&engine);
        let mut store = wasmtime::Store::new(&engine, ());
        let _instance = linker.instantiate(&mut store, &module).unwrap();
        */
      }
    )*
  }
}

test!(literals, operators, function);
