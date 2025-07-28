use crate::std_hc::BUILTIN_MODULE_NAME;

use super::*;

fn cast(state: &mut ModuleEncoder, f: u32, to: &TypeRef) {
  if let Type::TypeVariable(_) = (*to.borrow()).clone() {
    cast_any(state, f);
  } else {
    let type_id = state.get_type_id(to, false);
    state.push(f, Instruction::RefCastNonNull(HeapType::Concrete(type_id)));
  }
}

fn cast_any(state: &mut ModuleEncoder, f: u32) {
  state.push(f, Instruction::RefCastNonNull(HeapType::ANY));
}

pub fn lower(
  nodes: &mut IrModule,
  ptr: IrPtr,
  state: &mut ModuleEncoder,
  f: u32,
) {
  macro_rules! asm {
    ($($e:expr);*;) => {
      let __temp = [$($e,)*];
      state.func(f).extend(&__temp);
    };
  }

  let nk = nodes[ptr].kind.clone();
  let this_t = nodes[ptr].type_.clone();
  use IrKind as h;
  match nk {
    h::Declaration {
      assignee,
      value,
      in_,
    } => {
      let local = state
        .func(f)
        .new_local(assignee, ValType::Ref(RefType::ANYREF));
      lower(nodes, value, state, f);
      state.push(f, LocalSet(local));
      if let Some(in_) = in_ {
        lower(nodes, in_, state, f);
      } else {
        state.push_constant(f, ConstValue::Unit);
      }
    },
    h::Immediate(const_value) => state.push_constant(f, const_value),
    h::Identifier(mangle) => {
      state.get_symbol(f, &mangle);
      cast(state, f, &this_t);
    },
    h::Tuple(items)
    | h::StructLiteral {
      field_values: items,
      ..
    } => {
      items.into_iter().for_each(|i| lower(nodes, i, state, f));
      let tid = state.get_type_id(&this_t, false);
      state.push(f, StructNew(tid));
    },
    h::Field { of, index } => {
      lower(nodes, of, state, f);
      let struct_t = &nodes[of].type_;
      let field_id = struct_t.borrow().field_index(&index).unwrap();
      let struct_t = state.get_type_id(&struct_t, false);
      state.push(
        f,
        StructGet {
          struct_type_index: struct_t,
          field_index: field_id,
        },
      )
    },
    h::Binary {
      op: BinaryOp::Semicolon,
      left,
      right,
    } => {
      lower(nodes, left, state, f);
      state.push(f, Drop);
      lower(nodes, right, state, f);
    },
    h::Binary { op, left, right } => {
      lower(nodes, left, state, f);
      // Stack: Arg1
      state.get_symbol(f, &Path::from(BUILTIN_MODULE_NAME).child(op));
      // Stack: Arg1 Closure
      let op_func_type = state.get_valtype(&op.get_type(), false);
      let temporary = state.func(f).new_temporary(op_func_type);
      state.push(f, LocalTee(temporary));
      // Stack: Arg1 Closure
      let op_func_type_id = state.get_type_id(&op.get_type(), false);
      state.push(
        f,
        StructGet {
          struct_type_index: op_func_type_id,
          field_index: 1,
        },
      );
      // Stack: Arg1 Capture
      state.push(f, LocalGet(temporary));
      state.push(
        f,
        StructGet {
          struct_type_index: op_func_type_id,
          field_index: 0,
        },
      );
      // Stack: Arg1 Capture FunctionPtr
      let raw_op_func_type = state.get_type_id(&op.get_type(), true);
      state.push(
        f,
        CallIndirect {
          type_index: raw_op_func_type,
          table_index: 0,
        },
      );
      // Stack: Closure
      let closure_type = op.get_curry_type();
      let closure_valtype = state.get_valtype(&closure_type, false);
      let temporary = state.func(f).new_temporary(closure_valtype);
      state.push(f, LocalSet(temporary));
      // Stack: (empty)
      lower(nodes, right, state, f);
      // Stack: Arg2
      state.push(f, LocalGet(temporary));
      let closure_type_id = state.get_type_id(&closure_type, false);
      // Stack: Arg2 Closure
      state.push(
        f,
        StructGet {
          struct_type_index: closure_type_id,
          field_index: 1,
        },
      );
      // Stack: Arg2 Capture
      state.push(f, LocalGet(temporary));
      // Stack: Arg2 Capture Closure
      state.push(
        f,
        StructGet {
          struct_type_index: closure_type_id,
          field_index: 0,
        },
      );
      // Stack: Arg2 Capture Function
      let raw_function_type = state.get_type_id(&closure_type, true);
      state.push(
        f,
        CallIndirect {
          type_index: raw_function_type,
          table_index: 0,
        },
      );
    },
    h::Unary { op, child } => {
      lower(nodes, child, state, f);
      state.get_symbol(f, &Path::from(BUILTIN_MODULE_NAME).child(op));
      let op_func_type = state.get_valtype(&op.get_type(), false);
      let temporary = state.func(f).new_temporary(op_func_type);
      state.push(f, LocalTee(temporary));
      // Stack: Arg1 Closure
      let op_func_type_id = state.get_type_id(&op.get_type(), false);
      state.push(
        f,
        StructGet {
          struct_type_index: op_func_type_id,
          field_index: 1,
        },
      );
      // Stack: Arg1 Capture
      state.push(f, LocalGet(temporary));
      state.push(
        f,
        StructGet {
          struct_type_index: op_func_type_id,
          field_index: 0,
        },
      );
      // Stack: Arg1 Capture FunctionPtr
      let raw_op_func_type = state.get_type_id(&op.get_type(), true);
      state.push(
        f,
        CallIndirect {
          type_index: raw_op_func_type,
          table_index: 0,
        },
      );
    },
    h::If {
      predicate,
      then,
      else_,
    } => {
      lower(nodes, predicate, state, f);
      state.unwrap_primitive(f, &Type::Boolean.into());
      let block_type = BlockType::Result(state.get_valtype(&this_t, false));
      state.push(f, If(block_type));
      lower(nodes, then, state, f);
      state.push(f, Else);
      if let Some(else_) = else_ {
        lower(nodes, else_, state, f);
      } else {
        state.push_constant(f, ConstValue::Unit);
      }
      state.push(f, End);
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
        parameter_name.unwrap_or("unit".into()),
        captures.clone(),
        capture_types.clone(),
      );
      lower(nodes, body, state, new_func);
      state.push(f, I32Const(new_func as i32));
      // Push all captures
      for c in &captures {
        state.func(f).get_local(c);
        cast_any(state, f);
      }
      asm! {
        state.make_new_capture(captures.len() as u32);
        state.make_new_struct(&this_t);
      }
    },
    h::RecursiveDeclaration {
      assignee,
      parameter_name,
      captures,
      capture_types,
      function_type,
      body,
      in_,
      ..
    } => {
      // <Copied from FunctionDef>
      let new_func = state.new_function(
        &this_t,
        parameter_name.unwrap_or("unit".into()),
        captures.clone(),
        capture_types.clone(),
      );
      lower(nodes, body, state, new_func);
      state.push(f, I32Const(new_func as i32));
      // Push all captures
      for c in &captures {
        state.func(f).get_local(c);
        cast_any(state, f);
      }
      state.new_capture(f, captures.len() as u32);
      state.new_struct(f, &function_type);
      // </Copied from FunctionDef>
      // <Copied from Declaration>
      let local = state
        .func(f)
        .new_local(assignee, ValType::Ref(RefType::ANYREF));
      state.push(f, LocalSet(local));
      if let Some(in_) = in_ {
        lower(nodes, in_, state, f);
      } else {
        state.push_constant(f, ConstValue::Unit);
      }
      // </Copied from Declaration>
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
      state.push(f, LocalSet(function_temporary));
      lower(nodes, arguments, state, f);
      state.push(f, LocalGet(function_temporary));
      state.push(
        f,
        StructGet {
          struct_type_index: callee_type_id,
          field_index: 1,
        },
      );
      state.push(f, LocalGet(function_temporary));
      state.push(
        f,
        StructGet {
          struct_type_index: callee_type_id,
          field_index: 0,
        },
      );
      state.push(
        f,
        CallIndirect {
          type_index: callee_raw_type_id,
          table_index: 0,
        },
      );
      cast(state, f, &this_t);
    },
    h::ImportedSymbol(mangle, _) => {
      state.get_symbol(f, &mangle);
    },
  }
}
