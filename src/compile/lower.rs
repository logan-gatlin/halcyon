use super::*;

fn unwrap_const(c: ConstValue, f: u32, state: &mut ModuleEncoder) {
  use Instruction as i;
  match c {
    ConstValue::Unit => {
      let tid = state.get_type_id(&Type::Unit, false);
      state.func(f).push(i::StructNew(tid));
    },
    ConstValue::Integer(i) => {
      state.func(f).push(i::I64Const(i));
      let tid = state.get_type_id(&Type::Integer, false);
      state.func(f).push(i::StructNew(tid));
    },
    ConstValue::Real(r) => {
      state.func(f).push(i::F64Const((r).into()));
      let tid = state.get_type_id(&Type::Real, false);
      state.func(f).push(i::StructNew(tid));
    },
    ConstValue::Boolean(b) => {
      state.func(f).push(i::I32Const(b as i32));
      let tid = state.get_type_id(&Type::Boolean, false);
      state.func(f).push(i::StructNew(tid));
    },
    ConstValue::String(s) => {
      for b in s.bytes() {
        state.func(f).push(i::I32Const(b as i32));
      }
      let array_type_index = state.get_type_id(&Type::String, false) as u32;
      state.func(f).push(i::ArrayNewFixed {
        array_type_index,
        array_size: s.len() as u32,
      });
    },
    ConstValue::Glyph(g) => {
      state.func(f).push(i::I32Const(g as i32));
      let tid = state.get_type_id(&Type::Glyph, false);
      state.func(f).push(i::StructNew(tid));
    },
    ConstValue::Function { func_index, .. } => {
      state.func(f).push(i::RefFunc(func_index));
    },
    ConstValue::Tuple {
      members: values,
      type_id,
    }
    | ConstValue::StructLiteral {
      member_values: values,
      type_id,
      ..
    } => {
      values.into_iter().for_each(|v| unwrap_const(v, f, state));
      state.func(f).push(i::StructNew(type_id));
    },
    ConstValue::Type(_) => todo!(),
  }
}

fn cast(state: &mut ModuleEncoder, f: u32, to: &Type) {
  if let Type::TypeVariable(_) = to {
    cast_any(state, f);
  } else {
    let type_id = state.get_type_id(to, false);
    state
      .func(f)
      .push(Instruction::RefCastNonNull(HeapType::Concrete(type_id)));
  }
}

fn cast_any(state: &mut ModuleEncoder, f: u32) {
  state
    .func(f)
    .push(Instruction::RefCastNonNull(HeapType::ANY));
}

pub fn lower(
  nodes: &mut HlIrModule,
  ptr: IrPtr,
  state: &mut ModuleEncoder,
  f: u32,
) {
  use Instruction as i;
  let nk = nodes[ptr].kind.clone();
  let this_t = nodes[ptr].type_.clone();
  use HlIrKind as h;
  match nk {
    // Type declarations don't need to be compiled
    h::Declaration {
      is_type: true, in_, ..
    } => {
      if let Some(in_) = in_ {
        lower(nodes, in_, state, f);
      }
    },
    h::Declaration {
      assignee,
      is_type: false,
      value,
      in_,
      ..
    } => {
      let local = state
        .func(f)
        .new_local(assignee, ValType::Ref(RefType::ANYREF));
      lower(nodes, value, state, f);
      state.func(f).push(i::LocalSet(local));
      if let Some(in_) = in_ {
        lower(nodes, in_, state, f);
      } else {
        unwrap_const(ConstValue::Unit, f, state);
      }
    },
    h::Immediate(const_value) => unwrap_const(const_value, f, state),
    h::Identifier(mangle) => {
      state.func(f).get_local(&mangle);
      cast(state, f, &this_t);
    },
    h::Tuple(items)
    | h::StructLiteral {
      field_values: items,
      ..
    } => {
      items.into_iter().for_each(|i| lower(nodes, i, state, f));
      let tid = state.get_type_id(&this_t, false);
      state.func(f).push(i::StructNew(tid));
    },
    h::Field { of, index } => {
      lower(nodes, of, state, f);
      let struct_t = &nodes[of].type_;
      let field_id = struct_t.field_index(&index).unwrap();
      let struct_t = state.get_type_id(&struct_t, false);
      state.func(f).push(i::StructGet {
        struct_type_index: struct_t,
        field_index: field_id,
      })
    },
    h::Binary {
      op: BinaryOp::Semicolon,
      left,
      right,
    } => {
      lower(nodes, left, state, f);
      state.func(f).push(i::Drop);
      lower(nodes, right, state, f);
    },
    h::Binary { op, left, right } => {
      lower(nodes, left, state, f);
      lower(nodes, right, state, f);
      let struct_t = state.get_type_id(
        &(nodes[left].type_.clone() * nodes[right].type_.clone()),
        false,
      );
      state.func(f).push(i::StructNew(struct_t));

      let cid = state.get_type_id(&Type::_ClosureCapture, false);
      state.func(f).push(i::ArrayNewFixed {
        array_type_index: cid,
        array_size: 0,
      });
      let operator_func = state.get_binary_operator(op);
      state.func(f).push(i::Call(operator_func));
    },
    h::Unary { op, child } => {
      lower(nodes, child, state, f);
      let child_t = &nodes[child].type_;
      let operator_func = state.get_unary_operator(op);
      state.func(f).push(i::Call(operator_func));
      todo!()
    },
    h::If {
      predicate,
      then,
      else_,
    } => {
      lower(nodes, predicate, state, f);
      let block_type = BlockType::Result(state.get_valtype(&this_t, false));
      state.func(f).push(i::If(block_type));
      lower(nodes, then, state, f);
      if let Some(else_) = else_ {
        state.func(f).push(i::Else);
        lower(nodes, else_, state, f);
      }
      state.func(f).push(i::End);
    },
    h::FunctionDef {
      parameter_name,
      body,
      captures,
      capture_types,
      ..
    } => {
      let new_func = state.new_function(
        &this_t,
        parameter_name,
        captures.clone(),
        capture_types.clone(),
      );
      lower(nodes, body, state, new_func);
      state.func(f).push(i::RefFunc(new_func));
      // Push all captures
      for c in &captures {
        state.func(f).get_local(c);
        cast_any(state, f);
      }
      let array_type_index = state.get_type_id(&Type::_ClosureCapture, false);
      state.func(f).push(i::ArrayNewFixed {
        array_type_index,
        array_size: captures.len() as u32,
      });
      let tid = state.get_type_id(&this_t, false);
      state.func(f).push(i::StructNew(tid));
    },
    h::FunctionCall {
      callee,
      argument: arguments,
      ..
    } => {
      let callee_type = state.get_valtype(&nodes[callee].type_, false);
      let callee_type_id =
        state.get_type_id(&nodes[callee].type_.clone(), false);
      let callee_raw_type_id = state.get_type_id(&nodes[callee].type_, true);
      let function_temporary = state.func(f).new_temporary(callee_type);
      lower(nodes, callee, state, f);
      state.func(f).push(i::LocalSet(function_temporary));
      lower(nodes, arguments, state, f);
      state.func(f).push(i::LocalGet(function_temporary));
      state.func(f).push(i::StructGet {
        struct_type_index: callee_type_id,
        field_index: 1,
      });
      state.func(f).push(i::LocalGet(function_temporary));
      state.func(f).push(i::StructGet {
        struct_type_index: callee_type_id,
        field_index: 0,
      });
      state.func(f).push(i::CallRef(callee_raw_type_id));
    },
    h::StructDef { .. } => todo!(),
  }
}
