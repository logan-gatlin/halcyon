use super::*;
use wasm_encoder::Instruction::*;
use wasm_encoder::*;

pub fn make(encoder: &mut ModuleEncoder, interface: &mut ModuleInterface) {
  let e = encoder;
  let mut f;
  macro_rules! func {
    (fn $name:ident ($($param_type:expr),*) -> ($return_type:expr)) => {
      let name = stringify! {$name};
      interface.values.insert(
        Path::from(BUILTIN_MODULE_NAME).child(name),
        Type::curry(&[$($param_type.to_ref()),*], $return_type.to_ref()),
      );
      f = make_function(
        e, name, vec![$($param_type.to_ref(),)*], $return_type.to_ref()
      );
    };
  }
  macro_rules! asm {
    ($($e:expr);*;) => {
      let __temp = [$($e,)*];
      e.func_mut(f).extend(&__temp);
    };
  }
  // String length
  func! { fn string_length(Type::String) -> (Type::Integer) };
  asm! {
    LocalGet(0);
    ArrayLen;
    I64ExtendI32U;
    e.make_struct(Type::Integer);
  }

  // String copy (dest, dest_offset, src, src_offset, size)
  {
    func! { fn copy_string (Type::String, Type::String, Type::String) -> (Type::Unit) }
    asm! {
      Unreachable;
    }
  }
  // Print string
  {
    func! { fn print_string (Type::String) -> (Type::Unit) };
    let (import_id, import_type) = e.new_import("sys", "print_string", [ValType::I32; 2], []);
    let param = 0;
    let index = e.func_mut(f).new_local("index", ValType::I32);
    let length = e.func_mut(f).new_local("length", ValType::I32);
    let string_type = e.get_asm_type(Type::String).id;
    asm! {
      I32Const(0);
      LocalSet(index);
      // let index = 0
      LocalGet(param);
      ArrayLen;
      LocalSet(length);
      // let length = len(string)
      Loop(BlockType::Empty);
        LocalGet(index);
        LocalGet(length);
        I32LtU;
        // if index < length
        If(BlockType::Empty);
          LocalGet(index);
          LocalGet(param);
          LocalGet(index);
          ArrayGetU(string_type);
          I32Store8(MemArg { offset: 0, align: 0, memory_index: 0 });
          // *ptr = string[index]
          LocalGet(index);
          I32Const(1);
          I32Add;
          LocalSet(index);
          // index += 1
          // Continue
          Br(1);
        End;
      End;
      I32Const(0);
      LocalGet(length);
      I32Const(import_id as i32);
      CallIndirect { type_index: import_type, table_index: 0 };
      // print_string(0, len(string))
    }
    e.push_constant(f, ConstValue::Unit);
  }
  // New zeroed string with given length
  {
    func! { fn zeroed_string (Type::Integer) -> (Type::String) };
    asm! {
      LocalGet(0);
      e.make_unwrap_primitive(Type::Integer);
      I32WrapI64;
      ArrayNewDefault(e.get_asm_type(Type::String).id);
    }
  }
  // String concatenate
  {
    func! { fn string_concatenate (Type::String, Type::String) -> (Type::String) };
    let index = e.func_mut(f).new_local("index", ValType::I32);
    let length1 = e.func_mut(f).new_local("length1", ValType::I32);
    let length2 = e.func_mut(f).new_local("length2", ValType::I32);
    let length3 = e.func_mut(f).new_local("length3", ValType::I32);
    let s = e.new_local(f, "s", Type::String);
    asm! {
      e.get_local(f, "0");
      ArrayLen;
      LocalTee(length1);
      e.get_local(f, "1");
      ArrayLen;
      LocalTee(length2);
      I32Add;
      LocalTee(length3);
      ArrayNewDefault(e.get_asm_type(Type::String).id);
      Unreachable;
    }
  }
}
