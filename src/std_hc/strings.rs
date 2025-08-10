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
        Type::curry(&[$($param_type),*], $return_type),
      );
      f = make_function(
        e, name, vec![$($param_type,)*], $return_type
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
    {
        func! { fn string_length(Type::String) -> (Type::Integer) };
        let param = e.func(f).get_local_id(&"0".into());
        asm! {
          LocalGet(param);
          ArrayLen;
          I64ExtendI32U;
          e.make_struct(Type::Integer);
        }
    }

    // Print string
    {
        func! { fn print_string (Type::String) -> (Type::Unit) };
        let (import_id, import_type) = e.new_import("sys", "print_string", [ValType::I32; 2], []);
        let param = e.func(f).get_local_id(&"0".into());
        let index = e.func_mut(f).new_local("index", ValType::I32);
        let length = e.func_mut(f).new_local("length", ValType::I32);
        let string_type = e.get_asm_type(Type::String).id.unwrap();
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
    // String concatenate
    {
        func! { fn string_concatenate (Type::String, Type::String) -> (Type::String) };
        let length1 = e.func_mut(f).new_local("length1", ValType::I32);
        let length2 = e.func_mut(f).new_local("length2", ValType::I32);
        let string_type = e.get_asm_type(Type::String);
        let s = e.new_local(f, "s", Type::String);
        // array.copy: dest dest_offset src src_offset len
        asm! {
          e.get_local(f, "0");
          ArrayLen;
          LocalTee(length1);
          e.get_local(f, "1");
          ArrayLen;
          LocalTee(length2);
          I32Add;
          // First copy
          // Destination string
          ArrayNewDefault(string_type.id.unwrap());
          LocalTee(s);
          // Destination offset
          I32Const(0);
          // Source string
          e.get_local(f, "0");
          // Source offset
          I32Const(0);
          // Length
          LocalGet(length1);
          ArrayCopy { array_type_index_dst: string_type.id.unwrap(), array_type_index_src: string_type.id.unwrap() };
          // Second copy
          // Destination string
          LocalGet(s);
          // Destination offset
          LocalGet(length1);
          // Source string
          e.get_local(f, "1");
          // Source offset
          I32Const(0);
          // Length
          LocalGet(length2);
          ArrayCopy { array_type_index_dst: string_type.id.unwrap(), array_type_index_src: string_type.id.unwrap() };
          LocalGet(s);
        }
    }
}
