use crate::std_hc::BUILTIN_MODULE_NAME;

use super::*;

fn cast(state: &mut ModuleEncoder, to: Type) -> Instruction<'static> {
    if let Type::TypeVariable(_) = to {
        cast_any()
    } else {
        let type_id = state.get_asm_type(to).id.unwrap();
        Instruction::RefCastNonNull(HeapType::Concrete(type_id))
    }
}

fn cast_any() -> Instruction<'static> {
    Instruction::RefCastNonNull(HeapType::ANY)
}

// Helper functions to insert binary-operator assembly anywhere. Useful for comparisons
// in patterns.
/// Assumes the first operand is on the stack. Returns temporary with the curried op
fn binary_op_first_half(op: BinaryOp, state: &mut ModuleEncoder, f: u32) -> u32 {
    macro_rules! asm {
            ($($e:expr);*;) => {
                let __temp = [$($e,)*];
                state.func_mut(f).extend(&__temp);
            };
        }
    let operator_type = state.get_asm_type(op.get_type());
    let operator_temporary = state.func_mut(f).new_temporary(operator_type.val);
    let curried_operator_type = state.get_asm_type(op.get_curry_type());
    let curried_operator_temporary = state.func_mut(f).new_temporary(curried_operator_type.val);
    state.get_symbol(f, &Path::from(BUILTIN_MODULE_NAME).child(op));
    asm! {
      LocalTee(operator_temporary);
      StructGet {
        struct_type_index: operator_type.id.unwrap(),
        field_index: 1,
      };
      LocalGet(operator_temporary);
      StructGet {
        struct_type_index: operator_type.id.unwrap(),
        field_index: 0,
      };
      CallIndirect {
        type_index: operator_type.raw_id.unwrap(),
        table_index: 0,
      };
      RefCastNonNull(HeapType::Concrete(curried_operator_type.id.unwrap()));
      LocalSet(curried_operator_temporary);
    }
    curried_operator_temporary
}

// Assumes the second operand is on the stack
fn binary_op_second_half(
    op: BinaryOp,
    curried_operator_temporary: u32,
    state: &mut ModuleEncoder,
    f: u32,
) {
    macro_rules! asm {
            ($($e:expr);*;) => {
                let __temp = [$($e,)*];
                state.func_mut(f).extend(&__temp);
            };
        }
    let curried_operator_type = state.get_asm_type(op.get_curry_type());
    let return_type = state.get_asm_type(op.return_type()).id.unwrap();
    asm! {
      LocalGet(curried_operator_temporary);
      StructGet {
        struct_type_index: curried_operator_type.id.unwrap(),
        field_index: 1,
      };
      LocalGet(curried_operator_temporary);
      StructGet {
        struct_type_index: curried_operator_type.id.unwrap(),
        field_index: 0,
      };
      CallIndirect {
        type_index: curried_operator_type.raw_id.unwrap(),
        table_index: 0,
      };
      RefCastNonNull(HeapType::Concrete(return_type));
    }
}

pub fn lower_pattern(
    pattern: Pattern,
    state: &mut ModuleEncoder,
    temporary: u32,
    f: u32,
    is_global: bool,
) {
    macro_rules! asm {
        ($($e:expr);*;) => {
            let __temp = [$($e,)*];
            state.func_mut(f).extend(&__temp);
        };
    }
    match pattern.kind {
        PatternKind::Name(path) => {
            state.push(f, LocalGet(temporary));
            if !is_global {
                let local = state.new_local(f, path.clone(), pattern.type_);
                state.push(f, LocalSet(local));
            } else {
                let global_id = state.get_global_id(&path);
                state.push(f, GlobalSet(global_id));
            }
        }
        PatternKind::Tuple(patterns) => {
            let struct_t = state.get_asm_type(pattern.type_.clone());
            for (id, p) in patterns.into_iter().enumerate() {
                let temporary_t = state.get_asm_type(p.type_.clone());
                let next_temporary = state.func_mut(f).new_temporary(temporary_t.val);
                asm! {
                    LocalGet(temporary);
                    StructGet {
                        struct_type_index: struct_t.id.unwrap(),
                        field_index: id as u32,
                    };
                    LocalSet(next_temporary);
                }
                lower_pattern(p, state, next_temporary, f, is_global);
            }
        }
        PatternKind::Constructor(
            Constructor {
                variant,
                in_type,
                out_type,
            },
            pat,
        ) => {
            let out_t = state.get_asm_type(out_type);
            let Type::Sum { variant_types, .. } = pattern.type_.clone() else {
                panic!();
            };
            let in_type = variant_types[variant].clone();
            let in_t = state.get_asm_type(in_type.clone());
            let next_temporary = state.func_mut(f).new_temporary(in_t.val);
            asm! {
                I32Const(variant as i32);
                LocalGet(temporary);
                StructGet {
                    struct_type_index: out_t.id.unwrap(),
                    field_index: 0,
                };
                I32Eq;
                I32Const(1);
                I32Xor;
                BrIf(0);
                LocalGet(temporary);
                StructGet {
                    struct_type_index: out_t.id.unwrap(),
                    field_index: 1,
                };
                cast(state, in_type);
                LocalSet(next_temporary);
            }
            lower_pattern(*pat, state, next_temporary, f, is_global);
        }
        PatternKind::Literal(const_value) => {
            state.push(f, LocalGet(temporary));
            let temp = binary_op_first_half(BinaryOp::DoubleEqual, state, f);
            state.push_constant(f, const_value);
            binary_op_second_half(BinaryOp::DoubleEqual, temp, state, f);
            state.unwrap_primitive(f, Type::Boolean);
            state.push(f, I32Const(1));
            state.push(f, I32Xor);
            state.push(f, BrIf(0));
        }
    }
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
            let value_t = state.get_asm_type(nodes[value].type_.clone());
            let temporary = state.func_mut(f).new_temporary(value_t.val);
            lower(nodes, value, state, f);
            asm! {
                LocalSet(temporary);
            }
            lower_pattern(assignee, state, temporary, f, false);
            if let Some(in_) = in_ {
                lower(nodes, in_, state, f);
            } else {
                asm! { state.make_struct(Type::Unit); }
            }
        }
        h::Immediate(const_value) => state.push_constant(f, const_value),
        h::Identifier(mangle) => {
            state.get_symbol(f, &mangle);
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
                  .get_asm_type(nodes[of].type_.clone()).id.unwrap(),
                field_index: nodes[of].type_
                  .field_index(&index)
                  .unwrap_or_else(|| panic!("Struct does not contain field {index}")),
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
        h::Binary {
            op: BinaryOp::Apply,
            left,
            right,
        } => {
            let callee_type = state.get_asm_type(nodes[right].type_.clone());
            let function_temporary = state.func_mut(f).new_temporary(callee_type.val);
            lower(nodes, left, state, f);
            lower(nodes, right, state, f);
            asm! { LocalTee(function_temporary); }
            asm! {
              StructGet {
                struct_type_index: callee_type.id.unwrap(),
                field_index: 1,
              };
              LocalGet(function_temporary);
              StructGet {
                struct_type_index: callee_type.id.unwrap(),
                field_index: 0,
              };
              CallIndirect {
                type_index: callee_type.raw_id.unwrap(),
                table_index: 0,
              };
              cast(state, this_t);
            }
        }
        h::Binary { op, left, right } => {
            lower(nodes, left, state, f);
            let temp = binary_op_first_half(op, state, f);
            lower(nodes, right, state, f);
            binary_op_second_half(op, temp, state, f);
        }
        h::Unary { op, child } => {
            lower(nodes, child, state, f);
            let operator_type = state.get_asm_type(op.get_type());
            let temporary = state.func_mut(f).new_temporary(operator_type.val);
            state.get_symbol(f, &Path::from(BUILTIN_MODULE_NAME).child(op));
            asm! {
              LocalTee(temporary);
              StructGet {
                struct_type_index: operator_type.id.unwrap(),
                field_index: 1,
              };
              LocalGet(temporary);
              StructGet {
                struct_type_index: operator_type.id.unwrap(),
                field_index: 0,
              };
              CallIndirect {
                type_index: operator_type.raw_id.unwrap(),
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
        h::Match {
            scrutinee,
            predicates,
            branches,
        } => {
            let scrutinee_t = state.get_asm_type(nodes[scrutinee].type_.clone());
            let branches_t = state.get_asm_type(this_t);
            lower(nodes, scrutinee, state, f);
            let scrutinee_temp = state.func_mut(f).new_temporary(scrutinee_t.val);
            asm! { LocalSet(scrutinee_temp); }
            asm! { Block(BlockType::Result(branches_t.val)); }
            for (p, b) in predicates.into_iter().zip(branches) {
                asm! { Block(BlockType::Empty); }
                lower_pattern(p, state, scrutinee_temp, f, false);
                lower(nodes, b, state, f);
                asm! {
                    Br(1);
                    End;
                }
            }
            asm! {
               Unreachable;
               End;
            }
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
                struct_type_index: callee_type.id.unwrap(),
                field_index: 1,
              };
              LocalGet(function_temporary);
              StructGet {
                struct_type_index: callee_type.id.unwrap(),
                field_index: 0,
              };
              CallIndirect {
                type_index: callee_type.raw_id.unwrap(),
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
