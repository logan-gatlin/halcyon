use crate::builtin::Builtin;

use super::*;

use Instruction::*;

pub fn make_builtins(
  state: &mut ModuleEncoder,
) -> HashMap<Mangle, (Type, u32)> {
  Builtin::ALL
    .into_iter()
    .map(|bt| {
      let f = match bt {
        Builtin::Assert => {
          let f =
            state.new_function(&bt.get_type(), "a".into(), vec![], vec![]);
          state.func(f).get_local("a");
          let boolean_type = state.get_type_id(&Type::Boolean, false);
          state.func(f).push(StructGet {
            struct_type_index: boolean_type,
            field_index: 0,
          });
          state.func(f).push(I32Eqz);
          state.func(f).push(If(BlockType::Empty));
          state.func(f).push(Unreachable);
          state.func(f).push(End);
          let unit_type = state.get_type_id(&Type::Unit, false);
          state.func(f).push(StructNew(unit_type));
          f
        },
      };
      (bt.get_mangle(), (bt.get_type(), f))
    })
    .collect()
}

pub fn make_unary_ops(state: &mut ModuleEncoder) -> HashMap<UnaryOp, u32> {
  [UnaryOp::Minus, UnaryOp::MinusDot, UnaryOp::Not]
    .into_iter()
    .map(|op| {
      let f = state.new_function(&op.get_type(), "a".into(), vec![], vec![]);
      match op {
        UnaryOp::Minus => {
          state.func(f).push(I64Const(0));
          state.func(f).get_local("a");
          let integer_type = state.get_type_id(&Type::Integer, false);
          state.func(f).push(StructGet {
            struct_type_index: integer_type,
            field_index: 0,
          });
          state.func(f).push(I64Sub);
          state.func(f).push(StructNew(integer_type));
        },
        UnaryOp::MinusDot => {
          state.func(f).get_local("a");
          let real_type = state.get_type_id(&Type::Real, false);
          state.func(f).push(StructGet {
            struct_type_index: real_type,
            field_index: 0,
          });
          state.func(f).push(F64Neg);
          state.func(f).push(StructNew(real_type));
        },
        UnaryOp::Not => {
          state.func(f).get_local("a");
          let boolean_type = state.get_type_id(&Type::Boolean, false);
          state.func(f).push(StructGet {
            struct_type_index: boolean_type,
            field_index: 0,
          });
          state.func(f).push(I32Eqz);
          state.func(f).push(StructNew(boolean_type));
        },
      };
      (op, f)
    })
    .collect()
}

fn binary_op(
  state: &mut ModuleEncoder,
  parameter_type: Type,
  return_type: Type,
  op: Instruction<'static>,
) -> u32 {
  let (head, tail) = state.new_curried_function(
    vec!["a".into(), "b".into()],
    vec![parameter_type.clone(), parameter_type.clone()],
    return_type.clone(),
    vec![],
    vec![],
  );
  let parameter_type_id = state.get_type_id(&parameter_type, false);
  ["a", "b"].into_iter().for_each(|local| {
    state.func(tail).get_local(local);
    state.func(tail).push(StructGet {
      struct_type_index: parameter_type_id,
      field_index: 0,
    });
  });
  state.func(tail).push(op);
  let return_type_id = state.get_type_id(&return_type, false);
  state.func(tail).push(StructNew(return_type_id));
  head
}

pub fn make_binary_operators(
  state: &mut ModuleEncoder,
) -> HashMap<BinaryOp, u32> {
  use BinaryOp::*;
  let mut op_map = HashMap::new();
  [
    // Integer ops
    (Plus, Type::Integer, Type::Integer, I64Add),
    (Minus, Type::Integer, Type::Integer, I64Sub),
    (Star, Type::Integer, Type::Integer, I64Mul),
    (Slash, Type::Integer, Type::Integer, I64DivS),
    (Percent, Type::Integer, Type::Integer, I64RemS),
    // Real ops
    (PlusDot, Type::Real, Type::Real, F64Add),
    (MinusDot, Type::Real, Type::Real, F64Sub),
    (StarDot, Type::Real, Type::Real, F64Mul),
    (SlashDot, Type::Real, Type::Real, F64Div),
    // Boolean ops
    (And, Type::Boolean, Type::Boolean, I32And),
    (Or, Type::Boolean, Type::Boolean, I32Or),
    (Xor, Type::Boolean, Type::Boolean, I32Xor),
    /*
     */
  ]
  .into_iter()
  .for_each(|(op, parameter, returns, i)| {
    op_map.insert(op, binary_op(state, parameter, returns, i));
  });
  let integer_type = state.get_type_id(&Type::Integer, false);
  let real_type = state.get_type_id(&Type::Real, false);
  let boolean_type = state.get_type_id(&Type::Boolean, false);
  let glyph_type = state.get_type_id(&Type::Glyph, false);
  let unit_type = state.get_type_id(&Type::Unit, false);
  let string_type = state.get_type_id(&Type::String, false);
  // Integer, Real, Boolean, Glyph, Unit
  [
    (
      DoubleEqual,
      (I64Eq, F64Eq, I32Eq, I32Eq, I32Const(1)),
    ),
    (
      BangEqual,
      (I64Ne, F64Ne, I32Ne, I32Ne, I32Const(0)),
    ),
    (
      LessEqual,
      (I64LeS, F64Le, I32LeU, I32LeU, I32Const(1)),
    ),
    (
      GreaterEqual,
      (I64GeS, F64Ge, I32GeU, I32GeU, I32Const(1)),
    ),
    (
      Less,
      (I64LtS, F64Lt, I32LtU, I32LtU, I32Const(0)),
    ),
    (
      Greater,
      (I64GtS, F64Gt, I32GtU, I32GtU, I32Const(0)),
    ),
    /*
     */
  ]
  .into_iter()
  .for_each(
    |(op, (integer_op, real_op, boolean_op, glyph_op, unit_op))| {
      const TRUE: Instruction = I32Const(1);
      const FALSE: Instruction = I32Const(0);
      let (head, tail) = state.new_curried_function(
        vec!["a".into(), "b".into()],
        vec![Type::TypeVariable(0), Type::TypeVariable(0)],
        Type::Boolean,
        vec![],
        vec![],
      );
      let a = state.func(tail).get_local_id("a");
      let b = state.func(tail).get_local_id("b");
      macro_rules! asm {
        ($($e:expr);*;) => {
          $(state.func(tail).push($e));*
        };
      }
      op_map.insert(op, head);
      let type_ops = [
        (integer_type, integer_op),
        (real_type, real_op),
        (boolean_type, boolean_op),
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
      state.func(tail).get_local("a");
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
      let index = state.func(tail).new_local("index".into(), ValType::I32);
      let length = state.func(tail).new_local("index".into(), ValType::I32);
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
    },
  );

  op_map
}
