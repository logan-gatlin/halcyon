use wasmtime::*;

pub fn run(bin: &[u8]) {
  let engine = Engine::new(Config::new().wasm_gc(true)).unwrap();
  let module = Module::from_binary(&engine, bin).unwrap();
  let mut linker = Linker::new(&engine);
  linker
    .func_wrap("sys", "print", |caller: Caller<'_, u32>, param: i32| {
      println!("WASM :: {}", param);
    })
    .unwrap();
}
