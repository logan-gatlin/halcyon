use super::*;

macro_rules! test {
  ($($name:ident,)*) => {
    $(
      #[allow(unused)]
      #[test]
      fn $name() {
        let path = "./src/test/".to_string() + stringify!($name) + ".hc";
        let file = std::fs::read_to_string(path).unwrap();
        let wasm = match compile(&file) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("{e}");
                panic!()
            }
        };
        execute(wasm);
      }
    )*
  }
}

test!(
    literals,
    operators,
    function,
    control_flow,
    types,
    fizzbuzz,
    demo,
    stdtest,
);

#[allow(unused)]
fn execute(wasm: Vec<u8>) {
    use wasmtime::*;
    let mut config = Config::default();
    config.wasm_gc(true);
    config.wasm_function_references(true);
    let engine = Engine::new(&config).unwrap();
    let module = Module::new(&engine, &wasm).unwrap();
    let mut linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    let memory = Memory::new(&mut store, MemoryType::new(1, None)).unwrap();
    linker
        .func_wrap(
            "sys",
            "print_string",
            move |_callee: Caller<'_, ()>, ptr: i64, len: i64| {
                println!("CALLED PRINTLN");
                let mut buffer = vec![0; len as usize];
                memory.read(_callee, ptr as usize, &mut buffer).unwrap();
                let s = String::from_utf8(buffer).unwrap();
                println!("{s}");
            },
        )
        .unwrap()
        .define(&mut store, "sys", "memory", Extern::Memory(memory))
        .unwrap();
    let _instance = linker.instantiate(&mut store, &module).unwrap();
}
