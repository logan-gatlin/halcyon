use crate::std_hc::BUILTIN_MODULE_NAME;

use super::*;

fn cast(state: &mut ModuleEncoder, to: impl Into<TypeRef>) -> Instruction<'static> {
    let to = to.into();
    if let Type::TypeVariable(_) = (*to.borrow()).clone() {
        cast_any()
    } else {
        let type_id = state.get_asm_type(to).id;
        Instruction::RefCastNonNull(HeapType::Concrete(type_id))
    }
}

fn cast_any() -> Instruction<'static> {
    Instruction::RefCastNonNull(HeapType::ANY)
}

pub fn lower(nodes: &mut IrModule, ptr: IrPtr, state: &mut ModuleEncoder, f: u32) {
    macro_rules! asm {
    ($($e:expr);*;) => {
      let __temp = [$($e,)*];
      state.func_mut(f).extend(&__temp);
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
                .func_mut(f)
                .new_local(assignee, ValType::Ref(RefType::ANYREF));
            lower(nodes, value, state, f);
            asm! { LocalSet(local); }
            if let Some(in_) = in_ {
                lower(nodes, in_, state, f);
            } else {
                asm! { state.make_struct(Type::Unit); }
            }
        }
        h::Immediate(const_value) => state.push_constant(f, const_value),
        h::Identifier(mangle) => {
            state.get_symbol(f, &mangle);
            asm! { cast(state, this_t); }
        }
        h::Tuple(items)
        | h::StructLiteral {
            field_values: items,
            ..
        } => {
            items.into_iter().for_each(|i| lower(nodes, i, state, f));
            asm! { state.make_struct(this_t); }
        }
        h::Field { of, index } => {
            lower(nodes, of, state, f);
            asm! {
              StructGet {
                struct_type_index: state
                  .get_asm_type(nodes[of].type_.clone()).id,
                field_index: nodes[of].type_
                  .borrow()
                  .field_index(&index)
                  .unwrap(),
              };
            }
        }
        h::Binary {
            op: BinaryOp::Semicolon,
            left,
            right,
        } => {
            lower(nodes, left, state, f);
            state.push(f, Drop);
            lower(nodes, right, state, f);
        }
        h::Binary { op, left, right } => {
            let operator_type = state.get_asm_type(op.get_type());
            let operator_temporary = state.func_mut(f).new_temporary(operator_type.val);
            let curried_operator_type = state.get_asm_type(op.get_curry_type());
            let curried_operator_temporary =
                state.func_mut(f).new_temporary(curried_operator_type.val);
            lower(nodes, left, state, f);
            state.get_symbol(f, &Path::from(BUILTIN_MODULE_NAME).child(op));
            asm! {
              LocalTee(operator_temporary);
              StructGet {
                struct_type_index: operator_type.id,
                field_index: 1,
              };
              LocalGet(operator_temporary);
              StructGet {
                struct_type_index: operator_type.id,
                field_index: 0,
              };
              CallIndirect {
                type_index: operator_type.raw_id,
                table_index: 0,
              };
              LocalSet(curried_operator_temporary);
            }
            lower(nodes, right, state, f);
            asm! {
              LocalGet(curried_operator_temporary);
              StructGet {
                struct_type_index: curried_operator_type.id,
                field_index: 1,
              };
              LocalGet(curried_operator_temporary);
              StructGet {
                struct_type_index: curried_operator_type.id,
                field_index: 0,
              };
              CallIndirect {
                type_index: curried_operator_type.raw_id,
                table_index: 0,
              };
            }
        }
        h::Unary { op, child } => {
            lower(nodes, child, state, f);
            let operator_type = state.get_asm_type(op.get_type());
            let temporary = state.func_mut(f).new_temporary(operator_type.val);
            state.get_symbol(f, &Path::from(BUILTIN_MODULE_NAME).child(op));
            asm! {
              LocalTee(temporary);
              StructGet {
                struct_type_index: operator_type.id,
                field_index: 1,
              };
              LocalGet(temporary);
              StructGet {
                struct_type_index: operator_type.id,
                field_index: 0,
              };
              CallIndirect {
                type_index: operator_type.raw_id,
                table_index: 0,
              };
            };
        }
        h::If {
            predicate,
            then,
            else_,
        } => {
            lower(nodes, predicate, state, f);
            asm! {
              state.make_unwrap_primitive(Type::Boolean);
              If(BlockType::Result(state.get_valtype(&this_t, false)));
            }
            lower(nodes, then, state, f);
            asm! { Else; }
            if let Some(else_) = else_ {
                lower(nodes, else_, state, f);
            } else {
                asm! { state.make_struct(Type::Unit); }
            }
            asm! { End; }
        }
        h::FunctionDef {
            parameter_name,
            body,
            captures,
            capture_types,
            ..
        } => {
            let new_func = state.new_function(
                this_t.clone(),
                parameter_name.unwrap_or("unit".into()),
                captures.clone(),
                capture_types.clone(),
            );
            lower(nodes, body, state, new_func);
            asm! { I32Const(new_func as i32); }
            // Push all captures
            for c in &captures {
                asm! {
                  state.get_local(f, c.clone());
                  cast_any();
                }
            }
            asm! {
              state.make_capture(captures.len() as u32);
              state.make_struct(this_t);
            }
        }
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
                this_t,
                parameter_name.unwrap_or("unit".into()),
                captures.clone(),
                capture_types.clone(),
            );
            lower(nodes, body, state, new_func);
            asm! { I32Const(new_func as i32); }
            // Push all captures
            for c in &captures {
                asm! {
                  state.get_local(f, c.clone());
                  cast_any();
                }
            }
            let local = state
                .func_mut(f)
                .new_local(assignee, ValType::Ref(RefType::ANYREF));
            asm! {
             state.make_capture(captures.len() as u32);
             state.make_struct(function_type);
            // </Copied from FunctionDef>
            // <Copied from Declaration>
             LocalSet(local);
            }
            if let Some(in_) = in_ {
                lower(nodes, in_, state, f);
            } else {
                asm! { state.make_struct(Type::Unit); }
            }
            // </Copied from Declaration>
        }
        h::FunctionCall {
            callee,
            argument: arguments,
            ..
        } => {
            let callee_type = state.get_asm_type(nodes[callee].type_.clone());
            let function_temporary = state.func_mut(f).new_temporary(callee_type.val);
            lower(nodes, callee, state, f);
            asm! { LocalSet(function_temporary); }
            lower(nodes, arguments, state, f);
            asm! {
              LocalGet(function_temporary);
              StructGet {
                struct_type_index: callee_type.id,
                field_index: 1,
              };
              LocalGet(function_temporary);
              StructGet {
                struct_type_index: callee_type.id,
                field_index: 0,
              };
              CallIndirect {
                type_index: callee_type.raw_id,
                table_index: 0,
              };
              cast(state, this_t);
            }
        }
        h::ImportedSymbol(mangle, _) => {
            state.get_symbol(f, &mangle);
        }
    }
}
