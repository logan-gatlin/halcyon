use std::collections::HashMap;

use crate::{
  Handle, compile::ModuleEncoder, ir::*, operator::BinaryOp,
  semantic::ModuleInterface,
};

pub const BUILTIN_MODULE_NAME: &str = "builtin";
pub const STDLIB_MODULE_NAME: &str = "std";

const TRUE: wasm_encoder::Instruction<'static> =
  wasm_encoder::Instruction::I32Const(1);
const FALSE: wasm_encoder::Instruction<'static> =
  wasm_encoder::Instruction::I32Const(0);

#[allow(redundant_semicolons)]
#[allow(unused_assignments)]
pub fn make_std_module(
  encoder: &mut ModuleEncoder,
  interfaces: &mut HashMap<Mangle, ModuleInterface>,
) {
  // Set up interface
  let mut interface = ModuleInterface::default();
  let e = encoder;
  let mut f = 0;

  macro_rules! asm {
    ($($e:expr;)*) => {{
      let temp = [$($e,)*];
      e.func(f).extend(&temp);
    }};
  }

  macro_rules! define {
    ($(type $name:ident = $t:expr;)*) => {$({
      let mangle = mangle_global(&[BUILTIN_MODULE_NAME], stringify!($name));
      interface.types.insert(mangle, $t.to_ref());
    })*};

    // Define function, add it to signature
    (pub fn ($name:expr)
      ($($param_name:ident : $param_type:expr),*)
      -> ($return_type:expr)
      {
        $(let $local_name:ident : $local_type:expr;)*
        $($body:expr;)*
      }) => {{
      let this_type = crate::ir::Type::curry(&[$($param_type.into(),)*], $return_type.into());
      let mangle = mangle_global(&[BUILTIN_MODULE_NAME], format!("{}", $name));
      interface.values.insert(mangle.clone(), this_type.clone());
      define! {
        fn ($name) ($($param_name : $param_type),*) -> ($return_type) {
          $(let $local_name : $local_type;)*
          $($body;)*
        }
      }
    }};
    // Same as above, but for simple identifier names
    (pub fn $name:ident
      ($($param_name:ident : $param_type:expr),*)
      -> ($return_type:expr)
      {
        $(let $local_name:ident : $local_type:expr;)*
        $($body:expr;)*
      }) => {
      define! {
        pub fn (stringify!{$name}) ($($param_name : $param_type),*) -> ($return_type) {
          $(let $local_name : $local_type;)*
          $($body;)*
        }
      }
    };
    // Define function that does not appear in signature
    (fn ($name:expr)
      ($($param_name:ident : $param_type:expr),*)
       -> ($return_type:expr)
       {
         $(let $local_name:ident : $local_type:expr;)*
         $($body:expr;)*
       })  => {{
      let this_type = crate::ir::Type::curry(&[$($param_type.into(),)*], $return_type.into());
      let mangle = mangle_global(&[BUILTIN_MODULE_NAME], format!("{}", $name));
      let (head, tail) = e.new_curried_function(
        vec![$(stringify!($param_name).to_string(),)*],
        vec![$($param_type.into(),)*],
        $return_type.into()
      );
      e.push(e.main_fn, I32Const(head as i32));
      e.new_capture(e.main_fn, 0u32);
      e.new_struct(e.main_fn, &this_type);
      let global_id = e.new_global(mangle, &this_type);
      e.push(e.main_fn, GlobalSet(global_id));
      f = tail;
      $(
        let type_ = e.get_valtype(&$local_type.to_ref(), false);
        e.func(f).new_local(stringify!($local_name).to_string(), type_);
      )*
      asm!($($body;)*)
    }};
    // Same as above, but for simple identifier names
    (fn $name:ident
      ($($param_name:ident : $param_type:expr),*)
        -> ($return_type:expr)
        {
        $(let $local_name:ident : $local_type:expr;)*
        $($body:expr;)*
        }) => {
      define! {
        fn (stringify!{$name}) ($($param_name : $param_type),*) -> ($return_type) {
          $(
            let type_ = e.get_valtype(&$local_type.to_ref(), false);
            e.func(f).new_local(stringify!($local_name).to_string(), type_);
          )*
          $($body;)*
        }
      }
    };
    // Many case
    ($(pub fn $name:ident
      ($($param_name:ident : $param_type:expr),*)
       -> ($return_type:expr)
       {
         $(let $local_name:ident : $local_type:expr;)*
         $($body:expr;)*
       })*) => {
       $(define! {
         pub fn $name ($($param_name: $param_type),*) -> ($return_type) {
          $(let $local_name : $local_type;)*
          $($body;)*
         }
       })*
     };
  }

  macro_rules! syn {
    (set $name:ident) => {
      LocalSet(e.func(f).get_local_id(stringify! {$name}))
    };
    (get $name:ident) => {
      LocalGet(e.func(f).get_local_id(stringify! {$name}))
    };
    (tee $name:ident) => {
      LocalTee(e.func(f).get_local_id(stringify! {$name}))
    };
    (struct $t:expr) => {
      StructNew(e.get_type_id(&$t.to_ref(), false))
    };
    (unwrap $t:expr) => {
      StructGet {
        struct_type_index: e.get_type_id(&$t.to_ref(), false),
        field_index: 0,
      }
    };
  }

  macro_rules! binary_arithmetic {
    ($($op:expr, $instr:expr;)*) => {$({
      let name = format!("{}", $op);
      define! {
        fn (name)(a: $op.parameter_type(), b: $op.parameter_type()) -> ($op.return_type()) {}
      }
      // Push both parameters to the stack, unwrapping them
      e.get_symbol(f, "a");
      e.unwrap_primitive(f, &$op.parameter_type());
      e.get_symbol(f, "b");
      e.unwrap_primitive(f, &$op.parameter_type());
      e.push(f, $instr);
      e.new_struct(f, &$op.return_type());
    })*};
  }
  use crate::ConstValue as c;
  use crate::ir::Type::*;
  use wasm_encoder::Instruction::*;
  use wasm_encoder::*;
  // Primitive types
  define! {
    type unit = Unit;
    type integer = Integer;
    type real = Real;
    type boolean = Boolean;
    type string = String;
    type glyph = Glyph;
  }
  // Binary operators
  {
    use BinaryOp::*;
    binary_arithmetic!(
      Plus, I64Add;
      Minus, I64Sub;
      Star, I64Mul;
      Slash, I64DivS;
      Percent, I64RemS;

      PlusDot, F64Add;
      MinusDot, F64Sub;
      StarDot, F64Mul;
      SlashDot, F64Div;

      And, I32And;
      Or, I32Or;
      Xor, I32Xor;
    );
    macro_rules! comparison_ops {
      ($($op:expr, $int:expr, $real:expr, $glyph:expr, $unit:expr;)*) => {
        $(
          define! {
            fn ($op)(a: TypeVariable(0), b: TypeVariable(0)) -> (Boolean) {}
          }
          make_comparison_ops(e, f, $op, $int, $real, $glyph, $unit);
        )*
      };
    }
    comparison_ops! {
      DoubleEqual, I64Eq, F64Eq, I32Eq, TRUE;
      BangEqual, I64Ne, F64Ne, I32Ne, FALSE;
      LessEqual, I64LeS, F64Le, I32LeS, TRUE;
      GreaterEqual, I64GeS, F64Ge, I32GeS, TRUE;
      Less, I64LtS, F64Lt, I32LtS, FALSE;
      Greater, I64GtS, F64Gt, I32GtS, FALSE;
    }
  }

  // Builtins
  define! {
    pub fn panic(nothing: Unit) -> (TypeVariable(0)) {
      let a : Integer;
      Unreachable;
    }
  }

  // Finalize builtins
  interfaces.insert(BUILTIN_MODULE_NAME.to_string(), interface.clone());

  // Create standard library
  let input = include_str!("stdlib.hc");
  let linter = crate::Linter::new(input.to_string());
  let tokens = crate::tokenize(input.chars()).handle(&linter);
  let parsed_module = crate::parse(tokens)
    .handle(&linter)
    .first()
    .expect("stdlib.hc is empty")
    .clone();
  let mut ir_module =
    crate::build_ir(parsed_module, interfaces).handle(&linter);
  let std_interface = crate::type_solve(&mut ir_module).handle(&linter);
  e.encode_ir(ir_module);
  // Finalize stdlib
  interfaces.insert(STDLIB_MODULE_NAME.to_string(), std_interface);
}

pub fn make_comparison_ops(
  e: &mut ModuleEncoder,
  f: u32,
  op: BinaryOp,
  integer_op: wasm_encoder::Instruction<'static>,
  real_op: wasm_encoder::Instruction<'static>,
  glyph_op: wasm_encoder::Instruction<'static>,
  unit_op: wasm_encoder::Instruction<'static>,
) {
  use wasm_encoder::Instruction::*;
  use wasm_encoder::*;
  let integer_type = e.get_type_id(&Type::Integer.into(), false);
  let real_type = e.get_type_id(&Type::Real.into(), false);
  let boolean_type = e.get_type_id(&Type::Boolean.into(), false);
  let glyph_type = e.get_type_id(&Type::Glyph.into(), false);
  let unit_type = e.get_type_id(&Type::Unit.into(), false);
  let string_type = e.get_type_id(&Type::String.into(), false);

  let a = e.func(f).get_local_id("a");
  let b = e.func(f).get_local_id("b");
  macro_rules! asm {
    ($($e:expr);*;) => {
      let __temp = [$($e,)*];
      e.func(f).extend(&__temp);
    };
  }
  let type_ops = [
    (integer_type, integer_op),
    (real_type, real_op),
    (boolean_type, glyph_op.clone()), // Glyph and boolean are always the same
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
  e.func(f).get_local("a");
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
}
