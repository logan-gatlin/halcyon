use crate::{operator::BinaryOp, optimize::CallOptimization};

pub use super::*;

pub mod irnode;
pub mod module_item;
pub mod pattern;

impl Encode<ConstValue> for FunctionEncoder<'_> {
    fn encode(&mut self, obj: ConstValue) -> &mut Self {
        self.encode(obj.type_of());
        let type_id = self.module_encoder.type_id(&obj.type_of());
        match obj {
            ConstValue::Unit => self.encode(StructNew(type_id)),
            ConstValue::Integer(i) => self.encode([I64Const(i), StructNew(type_id)]),
            ConstValue::Real(r) => self.encode([F64Const(r.into()), StructNew(type_id)]),
            ConstValue::Boolean(b) => self.encode([I32Const(b as i32), StructNew(type_id)]),
            ConstValue::Glyph(g) => self.encode([I32Const(g as i32), StructNew(type_id)]),
            ConstValue::String(s) => {
                for b in s.bytes() {
                    self.encode(I32Const(b as i32));
                }
                self.encode(ArrayNewFixed {
                    array_type_index: type_id,
                    array_size: s.len() as u32,
                })
            }
        }
    }
}

impl FunctionEncoder<'_> {
    /// Expects [argument, function] on the stack
    pub fn call_function_maybe_tail(
        &mut self,
        argument_type: Type,
        return_type: Type,
        tail_call: bool,
    ) -> &mut Self {
        let callee_type = Type::func(argument_type.clone(), return_type.clone());
        let function_temporary = self.new_temporary(&callee_type);
        let function_type_id = self.module_encoder.function_type_id();
        let function_wrapper_id = self.module_encoder.type_id(&callee_type);
        let return_type = return_type.reduce();
        let cast = if return_type == ReducedType::AnyRef || tail_call {
            None
        } else {
            let id = self.module_encoder.reduced_type_id(&return_type);
            Some(RefCastNonNull(HeapType::Concrete(id)))
        };
        self.encode([
            LocalTee(function_temporary),
            // Get capture
            StructGet {
                struct_type_index: function_wrapper_id,
                field_index: 1,
            },
            LocalGet(function_temporary),
            // Get funcref
            StructGet {
                struct_type_index: function_wrapper_id,
                field_index: 0,
            },
            if tail_call {
                ReturnCallRef(function_type_id)
            } else {
                CallRef(function_type_id)
            },
        ])
        .encode(cast)
    }

    /// Expects [argument, function] on the stack
    pub fn call_function(&mut self, argument_type: Type, return_type: Type) -> &mut Self {
        self.call_function_maybe_tail(argument_type, return_type, false)
    }

    /// Expects [argument, function] on the stack
    pub fn tail_call_function(&mut self, argument_type: Type, return_type: Type) -> &mut Self {
        self.call_function_maybe_tail(argument_type, return_type, true)
    }
}
