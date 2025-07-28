use super::*;

use wasm_encoder::Instruction::*;
use wasm_encoder::*;

pub fn make_builtin_module(encoder: &mut ModuleEncoder) -> ModuleInterface {
  let mut interface = ModuleInterface::default();
  primitive_types(&mut interface);
  operator_assembly(encoder, &mut interface);
  primitive_funcs(encoder, &mut interface);
  interface
}

fn make_function(
  encoder: &mut ModuleEncoder,
  name: &str,
  parameter_types: Vec<TypeRef>,
  return_type: TypeRef,
) -> u32 {
  let this_type = Type::curry(&parameter_types, return_type.clone());
  let (head, tail) = encoder.new_curried_function(
    (0..parameter_types.len())
      .into_iter()
      .map(|i| format!("{i}").into())
      .collect(),
    parameter_types,
    return_type,
  );
  let path = Path::from(BUILTIN_MODULE_NAME).child(name);
  encoder.push(encoder.main_fn, I32Const(head as i32));
  encoder.new_capture(encoder.main_fn, 0u32);
  encoder.new_struct(encoder.main_fn, &this_type);
  let global_id = encoder.new_global(path, &this_type);
  encoder.push(encoder.main_fn, GlobalSet(global_id));
  tail
}

fn primitive_funcs(
  encoder: &mut ModuleEncoder,
  interface: &mut ModuleInterface,
) {
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
      e.func(f).extend(&__temp);
    };
  }
  // Panic
  func! { fn panic(Type::Unit) -> (Type::TypeVariable(0)) };
  asm!(Unreachable;);

  // String length
  func! { fn string_length(Type::String) -> (Type::Integer) };
  asm! {
    LocalGet(0);
    ArrayLen;
    I64ExtendI32U;
  }
  e.new_struct(f, &Type::Integer.to_ref());

  // Memory set
  {
    func! { fn string_copy(Type::String, Type::Integer) -> (Type::Unit) };
    let string_param = 0;
    let int_param = 1;
    let index = e.func(f).new_local("index".into(), ValType::I32);
    let length = e.func(f).new_local("length".into(), ValType::I32);
    asm! {
      I32Const(0);
      LocalSet(index);
      // let index = 0
      LocalGet(string_param);
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
          LocalGet(string_param);
          LocalGet(index);
          ArrayGetU(string_param);
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
    }
  }

  // Print string
  {
    func! { fn print_string (Type::String) -> (Type::Unit) };
    let (import_id, import_type) =
      e.new_import("sys", "print_string", [ValType::I32; 2], []);
    let param = 0;
    let index = e.func(f).new_local("index".into(), ValType::I32);
    let length = e.func(f).new_local("length".into(), ValType::I32);
    let string_type = e.get_type_id(&Type::String.to_ref(), false);
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
}

fn primitive_types(interface: &mut ModuleInterface) {
  [
    ("unit", Type::Unit),
    ("integer", Type::Integer),
    ("real", Type::Real),
    ("boolean", Type::Boolean),
    ("string", Type::String),
    ("glyph", Type::Glyph),
  ]
  .into_iter()
  .for_each(|(name, type_)| {
    interface
      .types
      .insert(Path::from(BUILTIN_MODULE_NAME).child(name), type_.to_ref());
  });
}

fn operator_assembly(
  encoder: &mut ModuleEncoder,
  interface: &mut ModuleInterface,
) {
  let e = encoder;
  // Unary operators
  {
    use UnaryOp::*;
    // Integer negate (-)
    let type_ = Type::Integer.to_ref();
    let f =
      make_function(e, &format!("{Minus}"), vec![type_.clone()], type_.clone());
    e.push(f, I64Const(0));
    e.get_symbol(f, &Path::from("0"));
    e.unwrap_primitive(f, &type_);
    e.push(f, I64Sub);
    e.new_struct(f, &type_);
    // Real negate (-)
    let type_ = Type::Real.to_ref();
    let f = make_function(
      e,
      &format!("{MinusDot}"),
      vec![type_.clone()],
      type_.clone(),
    );
    e.push(f, F64Const(0.0.into()));
    e.get_symbol(f, &Path::from("0"));
    e.unwrap_primitive(f, &type_);
    e.push(f, F64Sub);
    e.new_struct(f, &type_);
    // Boolean negate (not)
    let type_ = Type::Boolean.to_ref();
    let f =
      make_function(e, &format!("{Not}"), vec![type_.clone()], type_.clone());
    e.get_symbol(f, &Path::from("0"));
    e.unwrap_primitive(f, &type_);
    e.push(f, I32Eqz);
    e.new_struct(f, &type_);
  }

  use BinaryOp::*;
  // Binary arithmetic
  [
    (Plus, I64Add),
    (Minus, I64Sub),
    (Star, I64Mul),
    (Slash, I64DivS),
    (Percent, I64RemS),
    (PlusDot, F64Add),
    (MinusDot, F64Sub),
    (StarDot, F64Mul),
    (SlashDot, F64Div),
    (And, I32And),
    (Or, I32Or),
    (Xor, I32Xor),
  ]
  .into_iter()
  .for_each(|(op, instr)| {
    interface
      .values
      .insert(Path::from(BUILTIN_MODULE_NAME).child(op), op.get_type());
    let f = make_function(
      e,
      &format!("{op}"),
      vec![op.parameter_type(); 2],
      op.return_type(),
    );
    e.get_symbol(f, &Path::from("0"));
    e.unwrap_primitive(f, &op.parameter_type());
    e.get_symbol(f, &Path::from("1"));
    e.unwrap_primitive(f, &op.parameter_type());
    e.push(f, instr);
    e.new_struct(f, &op.return_type());
  });
  // Binary comparison ops
  {
    let integer_type = e.get_type_id(&Type::Integer.into(), false);
    let real_type = e.get_type_id(&Type::Real.into(), false);
    let boolean_type = e.get_type_id(&Type::Boolean.into(), false);
    let glyph_type = e.get_type_id(&Type::Glyph.into(), false);
    let unit_type = e.get_type_id(&Type::Unit.into(), false);
    let string_type = e.get_type_id(&Type::String.into(), false);
    [
      (DoubleEqual, I64Eq, F64Eq, I32Eq, TRUE),
      (BangEqual, I64Ne, F64Ne, I32Ne, FALSE),
      (LessEqual, I64LeS, F64Le, I32LeS, TRUE),
      (GreaterEqual, I64GeS, F64Ge, I32GeS, TRUE),
      (Less, I64LtS, F64Lt, I32LtS, FALSE),
      (Greater, I64GtS, F64Gt, I32GtS, FALSE),
    ]
    .into_iter()
    .for_each(|(op, integer_op, real_op, glyph_op, unit_op)| {
      let f = make_function(
        e,
        &format!("{op}"),
        vec![
          Type::TypeVariable(0).to_ref(),
          Type::TypeVariable(0).to_ref(),
        ],
        Type::Boolean.to_ref(),
      );

      let a = e.func(f).get_local_id(&Path::from("0"));
      let b = e.func(f).get_local_id(&Path::from("1"));
      macro_rules! asm {
        ($($e:expr);*;) => {
          let __temp = [$($e,)*];
          e.func(f).extend(&__temp);
        };
      }
      let type_ops = [
        (integer_type, integer_op),
        (real_type, real_op),
        (boolean_type, glyph_op.clone()), /* Glyph and boolean are always
                                           * the
                                           * same */
        (glyph_type, glyph_op),
        (unit_type, unit_op),
      ];
      (0..6).for_each(|_| {
        asm!(Block(BlockType::Result(ValType::Ref(RefType::ANYREF))););
      });
      let br_on = |id, depth| BrOnCast {
        relative_depth: depth,
        from_ref_type: RefType::ANYREF,
        to_ref_type: RefType {
          nullable: false,
          heap_type: HeapType::Concrete(id),
        },
      };
      // Jump on cast
      e.func(f).get_local(&Path::from("0"));
      type_ops
        .clone()
        .into_iter()
        .map(|(t, _)| t)
        .enumerate()
        .for_each(|(depth, t)| {
          asm!(br_on(t, depth as u32););
        });
      asm!(
        br_on(string_type, 5);
        Unreachable;
        End;
      );
      // Basic comparisons
      type_ops.into_iter().for_each(|(type_, op)| {
        // Get inner values if not unit
        if type_ != unit_type {
          asm!(
            RefCastNonNull(HeapType::Concrete(type_));
            StructGet {
              struct_type_index: type_,
              field_index: 0,
            };
            LocalGet(b);
            RefCastNonNull(HeapType::Concrete(type_));
            StructGet {
              struct_type_index: type_,
              field_index: 0,
            };
          );
        }
        asm!(
          // Perform comparison
          op;
          StructNew(boolean_type);
          Return;
          End;
        );
      });
      // String comparison
      asm!(
        // If len(first) > len(second)
        RefCastNonNull(HeapType::Concrete(string_type));
        ArrayLen;
        LocalGet(b);
        RefCastNonNull(HeapType::Concrete(string_type));
        ArrayLen;
        I32GtU;
        If(BlockType::Empty);
        match op {
          BinaryOp::DoubleEqual | BinaryOp::LessEqual | BinaryOp::Less => {
            FALSE
          },
          BinaryOp::Greater | BinaryOp::BangEqual | BinaryOp::GreaterEqual => {
            TRUE
          },
          _ => unreachable!(),
        };
        StructNew(boolean_type);
        Return;
        End;
        // If len(second) > len(first)
        LocalGet(a);
        RefCastNonNull(HeapType::Concrete(string_type));
        ArrayLen;
        LocalGet(b);
        RefCastNonNull(HeapType::Concrete(string_type));
        ArrayLen;
        I32LtU;
        If(BlockType::Empty);
        match op {
          BinaryOp::DoubleEqual | BinaryOp::GreaterEqual | BinaryOp::Greater => {
            FALSE
          },
          BinaryOp::Less | BinaryOp::BangEqual | BinaryOp::LessEqual => {
            TRUE
          },
          _ => unreachable!(),
        };
        StructNew(boolean_type);
        Return;
        End;
      );
      let index = e.func(f).new_local("index".into(), ValType::I32);
      let length = e.func(f).new_local("index".into(), ValType::I32);
      asm!(
        I32Const(0);
        LocalSet(index);
        LocalGet(a);
        RefCastNonNull(HeapType::Concrete(string_type));
        ArrayLen;
        LocalSet(length);
        // Lexical comparison
        Loop(BlockType::Empty);
        LocalGet(a);
        RefCastNonNull(HeapType::Concrete(string_type));
        LocalGet(index);
        ArrayGetU(string_type);
        LocalGet(b);
        RefCastNonNull(HeapType::Concrete(string_type));
        LocalGet(index);
        ArrayGetU(string_type);
        match op {
          BinaryOp::DoubleEqual => I32Ne,
          BinaryOp::BangEqual => I32Eq,
          BinaryOp::LessEqual => I32GtU,
          BinaryOp::GreaterEqual => I32LtU,
          BinaryOp::Less => I32GeU,
          BinaryOp::Greater => I32LeU,
          _ => unreachable!(),
        };
        If(BlockType::Empty);
        FALSE;
        StructNew(boolean_type);
        Return;
        End;
        LocalGet(index);
        I32Const(1);
        I32Add;
        LocalTee(index);
        LocalGet(length);
        I32LtU;
        BrIf(0);
        End;
        TRUE;
        StructNew(boolean_type);
        Return;
      );
    });
  }
}
