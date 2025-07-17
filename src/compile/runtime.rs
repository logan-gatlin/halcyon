use super::*;

use Instruction as i;

fn unary_op(
  state: &mut ModuleEncoder,
  type_: Type,
  instructions: Vec<Instruction<'static>>,
) -> u32 {
  todo!()
}

pub fn make_unary_operators(
  state: &mut ModuleEncoder,
) -> HashMap<UnaryOp, u32> {
  use UnaryOp::*;
  todo!()
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
  use Instruction as i;
  let parameter_type_id = state.get_type_id(&parameter_type, false);
  ["a", "b"].into_iter().for_each(|local| {
    state.func(tail).get_local(local);
    state.func(tail).push(i::StructGet {
      struct_type_index: parameter_type_id,
      field_index: 0,
    });
  });
  state.func(tail).push(op);
  let return_type_id = state.get_type_id(&return_type, false);
  state.func(tail).push(i::StructNew(return_type_id));
  head
}

pub fn make_binary_operators(
  state: &mut ModuleEncoder,
) -> HashMap<BinaryOp, u32> {
  use BinaryOp::*;
  let mut op_map = HashMap::new();
  [
    // Integer ops
    (Plus, Type::Integer, Type::Integer, i::I64Add),
    (Minus, Type::Integer, Type::Integer, i::I64Sub),
    (Star, Type::Integer, Type::Integer, i::I64Mul),
    (Slash, Type::Integer, Type::Integer, i::I64DivS),
    (Percent, Type::Integer, Type::Integer, i::I64RemS),
    // Real ops
    (PlusDot, Type::Real, Type::Real, i::F64Add),
    (MinusDot, Type::Real, Type::Real, i::F64Sub),
    (StarDot, Type::Real, Type::Real, i::F64Mul),
    (SlashDot, Type::Real, Type::Real, i::F64Div),
    // Boolean ops
    (And, Type::Boolean, Type::Boolean, i::I32And),
    (Or, Type::Boolean, Type::Boolean, i::I32Or),
    (Xor, Type::Boolean, Type::Boolean, i::I32Xor),
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
      (i::I64Eq, i::F64Eq, i::I32Eq, i::I32Eq, i::I32Const(1)),
    ),
    (
      BangEqual,
      (i::I64Ne, i::F64Ne, i::I32Ne, i::I32Ne, i::I32Const(0)),
    ),
    (
      LessEqual,
      (i::I64LeS, i::F64Le, i::I32LeU, i::I32LeU, i::I32Const(1)),
    ),
    (
      GreaterEqual,
      (i::I64GeS, i::F64Ge, i::I32GeU, i::I32GeU, i::I32Const(1)),
    ),
    (
      Less,
      (i::I64LtS, i::F64Lt, i::I32LtU, i::I32LtU, i::I32Const(0)),
    ),
    (
      Greater,
      (i::I64GtS, i::F64Gt, i::I32GtU, i::I32GtU, i::I32Const(0)),
    ),
    /*
     */
  ]
  .into_iter()
  .for_each(
    |(op, (integer_op, real_op, boolean_op, glyph_op, unit_op))| {
      let (head, tail) = state.new_curried_function(
        vec!["a".into(), "b".into()],
        vec![Type::TypeVariable(0), Type::TypeVariable(0)],
        Type::Boolean,
        vec![],
        vec![],
      );
      op_map.insert(op, head);
      let type_ops = [
        (integer_type, integer_op),
        (real_type, real_op),
        (boolean_type, boolean_op),
        (glyph_type, glyph_op),
        (unit_type, unit_op),
      ];
      (0..5).for_each(|_| {
        state
          .func(tail)
          .push(i::Block(BlockType::Result(ValType::Ref(RefType::ANYREF))))
      });
      let br_on = |id, depth| i::BrOnCast {
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
          state.func(tail).push(br_on(t, depth as u32));
        });
      state.func(tail).push(i::Unreachable);
      // Generate basic ops
      type_ops.into_iter().for_each(|(type_, op)| {
        state.func(tail).push(i::Return);
        state.func(tail).push(i::End);
        // Get inner values if not unit
        if type_ != unit_type {
          state
            .func(tail)
            .push(i::RefCastNonNull(HeapType::Concrete(type_)));
          state.func(tail).push(i::StructGet {
            struct_type_index: type_,
            field_index: 0,
          });
          state.func(tail).get_local("b");
          state
            .func(tail)
            .push(i::RefCastNonNull(HeapType::Concrete(type_)));
          state.func(tail).push(i::StructGet {
            struct_type_index: type_,
            field_index: 0,
          });
        }
        // Perform comparison
        state.func(tail).push(op);
        state.func(tail).push(i::StructNew(boolean_type));
      });
      state.func(tail).push(i::Return);
    },
  );

  op_map
}
