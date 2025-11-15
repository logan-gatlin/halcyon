use super::*;

impl FunctionEncoder<'_> {
    /// [i32] -> [array]
    pub fn new_array(&mut self, inner_type: Type) -> &mut Self {
        self.encode(Type::Array(inner_type.clone().into()));
        let array_type = self.module_encoder.type_id(&Type::Array(inner_type.into()));
        let temp = self.new_raw_temporary(ValType::I32);
        self.encode(LocalSet(temp))
            .encode(ConstValue::Unit)
            .encode([LocalGet(temp), ArrayNew(array_type)])
    }

    /// [dst(array) dst_offset(i32) src(array)] -> []
    pub fn array_copy_all(&mut self) -> &mut Self {
        let array_type = self
            .module_encoder
            .type_id(&Type::Array(Type::Variable(0).into()));
        let array_temp = self.new_temporary(&Type::Array(Type::Variable(0).into()));
        self.encode([
            LocalTee(array_temp),
            I32Const(0),
            LocalGet(array_temp),
            ArrayLen,
            ArrayCopy {
                array_type_index_dst: array_type,
                array_type_index_src: array_type,
            },
        ])
    }

    /// [array] -> [array]
    pub fn clone_array(&mut self, inner_type: Type) -> &mut Self {
        let array_type = self
            .module_encoder
            .type_id(&Type::Array(inner_type.clone().into()));
        let array_old_temp = self.new_temporary(&Type::Array(inner_type.clone().into()));
        let array_new_temp = self.new_temporary(&Type::Array(inner_type.into()));
        self.encode(LocalSet(array_old_temp))
            .encode(ConstValue::Unit)
            .encode([
                LocalGet(array_old_temp),
                ArrayLen,
                ArrayNew(array_type),
                LocalTee(array_new_temp),
                I32Const(0),
                LocalGet(array_old_temp),
                I32Const(0),
                LocalGet(array_old_temp),
                ArrayLen,
                ArrayCopy {
                    array_type_index_dst: array_type,
                    array_type_index_src: array_type,
                },
                LocalGet(array_new_temp),
            ])
    }
}
