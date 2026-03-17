use std::collections::BTreeSet;

use super::super::resolve::{
    ResolvedBinding,
    ResolvedInstruction,
};
use super::super::*;
use super::type_section::TypeSection;
use crate::ir::ImmediateValue;
use wasm_encoder::{
    BlockType,
    HeapType,
    Instruction as winstr,
};

/// Handles encode instruction.
pub(crate) fn encode_instruction(
    op: &ResolvedInstruction,
    type_section: &mut TypeSection,
    function_body: &mut wasm_encoder::Function,
    referenced_funcs: &mut BTreeSet<u32>,
) -> usize {
    use ResolvedInstruction as i;
    match op {
        i::Set(ResolvedBinding::Local(idx)) => {
            function_body.instruction(&winstr::LocalSet(*idx));
            1
        }
        i::Set(ResolvedBinding::Global(idx)) => {
            function_body.instruction(&winstr::GlobalSet(*idx));
            1
        }
        i::Get(ResolvedBinding::Local(idx)) => {
            function_body.instruction(&winstr::LocalGet(*idx));
            1
        }
        i::Get(ResolvedBinding::Global(idx)) => {
            function_body.instruction(&winstr::GlobalGet(*idx));
            1
        }
        i::Const(const_value) => {
            let instr = match const_value {
                ImmediateValue::Unit => winstr::Nop,
                ImmediateValue::Integer(value) => winstr::I64Const(*value),
                ImmediateValue::Real(value) => winstr::F64Const((*value).into()),
                ImmediateValue::Boolean(value) => winstr::I32Const(i32::from(*value)),
                ImmediateValue::String(_) => unreachable!("string constants are verified out"),
                ImmediateValue::Glyph(value) => winstr::I32Const(*value as i32),
            };
            function_body.instruction(&instr);
            1
        }
        i::I32Const(value) => {
            function_body.instruction(&winstr::I32Const(*value));
            1
        }
        i::F32Const(value) => {
            function_body.instruction(&winstr::F32Const((*value).into()));
            1
        }
        i::Func(idx) => {
            referenced_funcs.insert(*idx);
            function_body.instruction(&winstr::RefFunc(*idx));
            1
        }
        i::StructNew(items) => {
            function_body.instruction(&winstr::StructNew(type_section.new_struct(items)));
            1
        }
        i::StructGet(fields, field_index) => {
            let struct_type_index = type_section.new_struct(fields);
            function_body.instruction(&winstr::RefCastNullable(HeapType::Concrete(
                struct_type_index,
            )));
            function_body.instruction(&winstr::StructGet {
                struct_type_index,
                field_index: *field_index as u32,
            });
            2
        }
        i::ArrayGet(type_) => {
            let arr_idx = type_section.new_array(type_);
            let instr = match type_ {
                Type::I8 | Type::I16 => winstr::ArrayGetU(arr_idx),
                _ => winstr::ArrayGet(arr_idx),
            };
            function_body.instruction(&instr);
            1
        }
        i::ArrayNewFixed { inner_type, length } => {
            function_body.instruction(&winstr::ArrayNewFixed {
                array_type_index: type_section.new_array(inner_type),
                array_size: *length as u32,
            });
            1
        }
        i::ArrayNewDefault(type_) => {
            function_body.instruction(&winstr::ArrayNewDefault(type_section.new_array(type_)));
            1
        }
        i::ArrayLen => {
            function_body.instruction(&winstr::ArrayLen);
            1
        }
        i::ArrayCopy { dst_type, src_type } => {
            function_body.instruction(&winstr::ArrayCopy {
                array_type_index_dst: type_section.new_array(dst_type),
                array_type_index_src: type_section.new_array(src_type),
            });
            1
        }
        i::CallRef {
            parameters,
            returns,
        } => {
            function_body.instruction(&winstr::CallRef(
                type_section.new_function(parameters, returns),
            ));
            1
        }
        i::Call(idx) => {
            function_body.instruction(&winstr::Call(*idx));
            1
        }
        i::Unreachable => {
            function_body.instruction(&winstr::Unreachable);
            1
        }
        i::Drop => {
            function_body.instruction(&winstr::Drop);
            1
        }
        i::If(result) => {
            function_body.instruction(&winstr::If(match result {
                Some(result) => BlockType::Result(type_section.valtype_of(result)),
                None => BlockType::Empty,
            }));
            1
        }
        i::Else => {
            function_body.instruction(&winstr::Else);
            1
        }
        i::End => {
            function_body.instruction(&winstr::End);
            1
        }
        i::Loop => {
            function_body.instruction(&winstr::Loop(BlockType::Empty));
            1
        }
        i::Block(result) => {
            function_body.instruction(&winstr::Block(match result {
                Some(result) => BlockType::Result(type_section.valtype_of(result)),
                None => BlockType::Empty,
            }));
            1
        }
        i::Break(depth) => {
            function_body.instruction(&winstr::Br(*depth as u32));
            1
        }
        i::BreakIf(depth) => {
            function_body.instruction(&winstr::BrIf(*depth as u32));
            1
        }
        i::I32Op(op) => {
            function_body.instruction(&lower_i32_op(*op));
            1
        }
        i::I64Op(op) => {
            function_body.instruction(&lower_i64_op(*op));
            1
        }
        i::F32Op(op) => {
            function_body.instruction(&lower_f32_op(*op));
            1
        }
        i::F64Op(op) => {
            function_body.instruction(&lower_f64_op(*op));
            1
        }
        i::RefCastFunc {
            parameters,
            returns,
        } => {
            let func_type_idx = type_section.new_function(parameters, returns);
            function_body.instruction(&winstr::RefCastNullable(HeapType::Concrete(func_type_idx)));
            1
        }
        i::RefCastStruct(fields) => {
            let struct_type_idx = type_section.new_struct(fields);
            function_body.instruction(&winstr::RefCastNullable(HeapType::Concrete(
                struct_type_idx,
            )));
            1
        }
        i::RefCastArray(inner) => {
            let array_type_idx = type_section.new_array(inner);
            function_body.instruction(&winstr::RefCastNullable(HeapType::Concrete(array_type_idx)));
            1
        }
        i::I32Store8 => {
            function_body.instruction(&winstr::I32Store8(wasm_encoder::MemArg {
                offset: 0,
                align: 0,
                memory_index: 0,
            }));
            1
        }
        i::I32Load => {
            function_body.instruction(&winstr::I32Load(wasm_encoder::MemArg {
                offset: 0,
                align: 2,
                memory_index: 0,
            }));
            1
        }
        i::I32Store => {
            function_body.instruction(&winstr::I32Store(wasm_encoder::MemArg {
                offset: 0,
                align: 2,
                memory_index: 0,
            }));
            1
        }
        i::I64Load => {
            function_body.instruction(&winstr::I64Load(wasm_encoder::MemArg {
                offset: 0,
                align: 3,
                memory_index: 0,
            }));
            1
        }
        i::I64ExtendI32U => {
            function_body.instruction(&winstr::I64ExtendI32U);
            1
        }
        i::I32WrapI64 => {
            function_body.instruction(&winstr::I32WrapI64);
            1
        }
        i::I32TruncF32S => {
            function_body.instruction(&winstr::I32TruncF32S);
            1
        }
        i::I32TruncF32U => {
            function_body.instruction(&winstr::I32TruncF32U);
            1
        }
        i::I32TruncF64S => {
            function_body.instruction(&winstr::I32TruncF64S);
            1
        }
        i::I32TruncF64U => {
            function_body.instruction(&winstr::I32TruncF64U);
            1
        }
        i::I64TruncF32S => {
            function_body.instruction(&winstr::I64TruncF32S);
            1
        }
        i::I64TruncF32U => {
            function_body.instruction(&winstr::I64TruncF32U);
            1
        }
        i::I64TruncF64S => {
            function_body.instruction(&winstr::I64TruncF64S);
            1
        }
        i::I64TruncF64U => {
            function_body.instruction(&winstr::I64TruncF64U);
            1
        }
        i::F32DemoteF64 => {
            function_body.instruction(&winstr::F32DemoteF64);
            1
        }
    }
}

/// Handles lower i32 op.
fn lower_i32_op(op: NumberOperation) -> winstr<'static> {
    match op {
        NumberOperation::Eq => winstr::I32Eq,
        NumberOperation::Ne => winstr::I32Ne,
        NumberOperation::Gt => winstr::I32GtS,
        NumberOperation::Lt => winstr::I32LtS,
        NumberOperation::Ge => winstr::I32GeS,
        NumberOperation::Le => winstr::I32LeS,
        NumberOperation::Add => winstr::I32Add,
        NumberOperation::Sub => winstr::I32Sub,
        NumberOperation::Mul => winstr::I32Mul,
        NumberOperation::Div => winstr::I32DivS,
        NumberOperation::Rem => winstr::I32RemS,
        NumberOperation::And => winstr::I32And,
        NumberOperation::Or => winstr::I32Or,
        NumberOperation::Xor => winstr::I32Xor,
    }
}

/// Handles lower i64 op.
fn lower_i64_op(op: NumberOperation) -> winstr<'static> {
    match op {
        NumberOperation::Eq => winstr::I64Eq,
        NumberOperation::Ne => winstr::I64Ne,
        NumberOperation::Gt => winstr::I64GtS,
        NumberOperation::Lt => winstr::I64LtS,
        NumberOperation::Ge => winstr::I64GeS,
        NumberOperation::Le => winstr::I64LeS,
        NumberOperation::Add => winstr::I64Add,
        NumberOperation::Sub => winstr::I64Sub,
        NumberOperation::Mul => winstr::I64Mul,
        NumberOperation::Div => winstr::I64DivS,
        NumberOperation::Rem => winstr::I64RemS,
        NumberOperation::And => winstr::I64And,
        NumberOperation::Or => winstr::I64Or,
        NumberOperation::Xor => winstr::I64Xor,
    }
}

/// Handles lower f32 op.
fn lower_f32_op(op: NumberOperation) -> winstr<'static> {
    match op {
        NumberOperation::Eq => winstr::F32Eq,
        NumberOperation::Ne => winstr::F32Ne,
        NumberOperation::Gt => winstr::F32Gt,
        NumberOperation::Lt => winstr::F32Lt,
        NumberOperation::Ge => winstr::F32Ge,
        NumberOperation::Le => winstr::F32Le,
        NumberOperation::Add => winstr::F32Add,
        NumberOperation::Sub => winstr::F32Sub,
        NumberOperation::Mul => winstr::F32Mul,
        NumberOperation::Div => winstr::F32Div,
        NumberOperation::And
        | NumberOperation::Or
        | NumberOperation::Xor
        | NumberOperation::Rem => {
            unreachable!("Bitwise operations not supported on F32")
        }
    }
}

/// Handles lower f64 op.
fn lower_f64_op(op: NumberOperation) -> winstr<'static> {
    match op {
        NumberOperation::Eq => winstr::F64Eq,
        NumberOperation::Ne => winstr::F64Ne,
        NumberOperation::Gt => winstr::F64Gt,
        NumberOperation::Lt => winstr::F64Lt,
        NumberOperation::Ge => winstr::F64Ge,
        NumberOperation::Le => winstr::F64Le,
        NumberOperation::Add => winstr::F64Add,
        NumberOperation::Sub => winstr::F64Sub,
        NumberOperation::Mul => winstr::F64Mul,
        NumberOperation::Div => winstr::F64Div,
        NumberOperation::And
        | NumberOperation::Or
        | NumberOperation::Xor
        | NumberOperation::Rem => {
            unreachable!("Bitwise operations not supported on F64")
        }
    }
}
