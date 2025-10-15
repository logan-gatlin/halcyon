use super::*;

impl Encode<(Pattern, ScopeKind)> for FunctionEncoder<'_> {
    fn encode(&mut self, (pattern, scope): (Pattern, ScopeKind)) -> &mut Self {
        self.lower_pattern(pattern, scope);
        self
    }
}

impl FunctionEncoder<'_> {
    pub fn lower_pattern(&mut self, pattern: Pattern, scope: ScopeKind) {
        self.encode(pattern.type_.clone());
        match pattern.inner.inner {
            PatternKind::Hole => {
                self.encode(Drop);
            }
            PatternKind::Name(path) => {
                self.set_symbol(&path);
            }
            PatternKind::Tuple(patterns) => {
                let temporary = self.new_temporary(&pattern.type_);
                self.encode(LocalSet(temporary));
                let struct_type_id = self.module_encoder.type_id(&pattern.type_);
                for (index, pattern) in patterns.into_iter().enumerate() {
                    self.encode([
                        LocalGet(temporary),
                        StructGet {
                            struct_type_index: struct_type_id,
                            field_index: index as u32,
                        },
                    ])
                    .lower_pattern(pattern, scope);
                }
            }
            PatternKind::Array(ArrayPattern::Exact(patterns)) => {
                let temporary = self.new_temporary(&pattern.type_);
                // Skip if length is incorrect
                self.encode([
                    LocalTee(temporary),
                    ArrayLen,
                    I32Const(patterns.len() as i32),
                    I32Ne,
                    BrIf(0),
                ]);
                let type_ = self.module_encoder.type_id(&pattern.type_);
                for (id, pat) in patterns.into_iter().enumerate() {
                    let pattern_type = pat.type_.clone().reduce();
                    let convert = if pattern_type == ReducedType::AnyRef {
                        None
                    } else {
                        Some(RefCastNonNull(HeapType::Concrete(
                            self.module_encoder.reduced_type_id(&pattern_type),
                        )))
                    };
                    self.encode([LocalGet(temporary), I32Const(id as i32), ArrayGet(type_)])
                        .encode(convert)
                        .lower_pattern(pat, scope);
                }
            }
            PatternKind::Array(ArrayPattern::Leading { head, tail }) => {
                let head_length = head.len();
                let temporary = self.new_temporary(&pattern.type_);
                self.encode([
                    LocalTee(temporary),
                    ArrayLen,
                    I32Const(head.len() as i32),
                    I32LtU,
                    BrIf(0),
                ]);
                let type_ = self.module_encoder.type_id(&pattern.type_);
                let Type::Array(inner_type) = pattern.type_.clone() else {
                    unreachable!()
                };
                let reduced_inner_type = inner_type.clone().reduce();
                let convert = if reduced_inner_type == ReducedType::AnyRef {
                    None
                } else {
                    Some(RefCastNonNull(HeapType::Concrete(
                        self.module_encoder.reduced_type_id(&reduced_inner_type),
                    )))
                };
                for (id, pat) in head.into_iter().enumerate() {
                    self.encode([LocalGet(temporary), I32Const(id as i32), ArrayGet(type_)])
                        .encode(convert.clone())
                        .lower_pattern(pat, scope);
                }
                if let Some(tail) = tail {
                    let new_array = self.new_temporary(&pattern.type_);
                    let array_len = self.new_raw_temporary(ValType::I32);
                    self.encode([
                        LocalGet(temporary),
                        ArrayLen,
                        I32Const(head_length as i32),
                        I32Sub,
                        LocalTee(array_len),
                    ])
                    // Dest array
                    .new_array(*inner_type)
                    .encode([
                        LocalTee(new_array),
                        // Dest offset
                        I32Const(0),
                        // Src array
                        LocalGet(temporary),
                        // Src offset
                        LocalGet(array_len),
                        I32Const(1),
                        I32Sub,
                        LocalGet(new_array),
                        // Length
                        ArrayLen,
                        ArrayCopy {
                            array_type_index_dst: type_,
                            array_type_index_src: type_,
                        },
                        LocalGet(new_array),
                    ])
                    .set_symbol(&tail);
                }
            }
            PatternKind::Array(ArrayPattern::Trailing { head, tail }) => {
                let tail_len = tail.len();
                let temporary = self.new_temporary(&pattern.type_);
                self.encode([
                    LocalTee(temporary),
                    ArrayLen,
                    I32Const(tail_len as i32),
                    I32LtU,
                    BrIf(0),
                ]);
                let type_ = self.module_encoder.type_id(&pattern.type_);
                let Type::Array(inner_type) = pattern.type_.clone() else {
                    unreachable!()
                };
                let array_len = self.new_raw_temporary(ValType::I32);
                self.encode([
                    LocalGet(temporary),
                    ArrayLen,
                    I32Const(tail_len as i32),
                    I32Sub,
                    LocalSet(array_len),
                ]);
                if let Some(head) = head {
                    let new_array = self.new_temporary(&pattern.type_);
                    // Dest array
                    self.encode(LocalGet(array_len))
                        .new_array(*inner_type.clone())
                        .encode([
                            LocalTee(new_array),
                            // Dest offset
                            I32Const(0),
                            // Src array
                            LocalGet(temporary),
                            // Src offset
                            I32Const(0),
                            LocalGet(new_array),
                            // Length
                            ArrayLen,
                            ArrayCopy {
                                array_type_index_dst: type_,
                                array_type_index_src: type_,
                            },
                            LocalGet(new_array),
                        ])
                        .set_symbol(&head);
                }
                let reduced_inner_type = inner_type.reduce();
                let convert = if reduced_inner_type == ReducedType::AnyRef {
                    None
                } else {
                    Some(RefCastNonNull(HeapType::Concrete(
                        self.module_encoder.reduced_type_id(&reduced_inner_type),
                    )))
                };
                for (id, pat) in tail.into_iter().enumerate() {
                    self.encode([
                        LocalGet(temporary),
                        I32Const(id as i32),
                        LocalGet(array_len),
                        I32Add,
                        ArrayGet(type_),
                    ])
                    .encode(convert.clone())
                    .lower_pattern(pat, scope);
                }
            }
            PatternKind::Array(ArrayPattern::LeadingAndTrailing { head, middle, tail }) => {
                let head_len = head.len();
                let tail_len = tail.len();
                let temporary = self.new_temporary(&pattern.type_);
                self.encode([
                    LocalTee(temporary),
                    ArrayLen,
                    I32Const((head_len + tail_len) as i32),
                    I32LtU,
                    BrIf(0),
                ]);
                let type_ = self.module_encoder.type_id(&pattern.type_);
                let Type::Array(inner_type) = pattern.type_.clone() else {
                    unreachable!()
                };
                let reduced_inner_type = inner_type.clone().reduce();
                let convert = if reduced_inner_type == ReducedType::AnyRef {
                    None
                } else {
                    Some(RefCastNonNull(HeapType::Concrete(
                        self.module_encoder.reduced_type_id(&reduced_inner_type),
                    )))
                };
                for (id, pat) in head.into_iter().enumerate() {
                    self.encode([LocalGet(temporary), I32Const(id as i32), ArrayGet(type_)])
                        .encode(convert.clone())
                        .lower_pattern(pat, scope);
                }
                let array_len = self.new_raw_temporary(ValType::I32);
                self.encode([
                    LocalGet(temporary),
                    ArrayLen,
                    I32Const((head_len + tail_len) as i32),
                    I32Sub,
                    LocalSet(array_len),
                ]);
                if let Some(middle) = middle {
                    let new_array = self.new_temporary(&pattern.type_);
                    // Dest array
                    self.encode(LocalGet(array_len))
                        .new_array(*inner_type)
                        .encode([
                            LocalTee(new_array),
                            // Dest offset
                            I32Const(0),
                            // Src array
                            LocalGet(temporary),
                            // Src offset
                            LocalGet(array_len),
                            I32Const((head_len - 1) as i32),
                            I32Add,
                            LocalGet(new_array),
                            // Length
                            ArrayLen,
                            ArrayCopy {
                                array_type_index_dst: type_,
                                array_type_index_src: type_,
                            },
                            LocalGet(new_array),
                        ])
                        .set_symbol(&middle);
                }
                for (id, pat) in tail.into_iter().enumerate() {
                    self.encode([
                        LocalGet(temporary),
                        I32Const((id + head_len) as i32),
                        LocalGet(array_len),
                        I32Add,
                        ArrayGet(type_),
                    ])
                    .encode(convert.clone())
                    .lower_pattern(pat, scope);
                }
            }
            PatternKind::Constructor(
                Constructor {
                    variant_id,
                    kind: ConstructorKind::Unitary(t),
                },
                _,
            ) => {
                self.encode(t.clone());
                let type_id = self.module_encoder.type_id(&t);
                self.encode([
                    StructGet {
                        struct_type_index: type_id,
                        field_index: 0,
                    },
                    I32Const(variant_id as i32),
                    I32Ne,
                    BrIf(0),
                ]);
            }
            PatternKind::Constructor(
                Constructor {
                    variant_id,
                    kind: ConstructorKind::Function(in_type, out_type),
                },
                next_pattern,
            ) => {
                self.encode([in_type.clone(), out_type.clone()]);
                let temporary = self.new_temporary(&pattern.type_);
                let out_type_id = self.module_encoder.type_id(&out_type);
                let in_type_id_cast = match in_type.reduce() {
                    ReducedType::AnyRef => Nop,
                    t => {
                        RefCastNonNull(HeapType::Concrete(self.module_encoder.reduced_type_id(&t)))
                    }
                };
                self.encode([
                    LocalTee(temporary),
                    // Check sum type tag
                    StructGet {
                        struct_type_index: out_type_id,
                        field_index: 0,
                    },
                    I32Const(variant_id as i32),
                    I32Ne,
                    BrIf(0),
                    // Pass on the inner value
                    LocalGet(temporary),
                    StructGet {
                        struct_type_index: out_type_id,
                        field_index: 1,
                    },
                    in_type_id_cast,
                ])
                .lower_pattern(*next_pattern, scope);
            }
            PatternKind::Literal(ConstValue::Unit) => {
                self.encode(Drop);
            }
            PatternKind::Literal(const_value) => {
                self.encode([
                    const_value.type_of(),
                    Type::func(const_value.type_of(), const_value.type_of()),
                ]);
                let function_temporary =
                    self.new_temporary(&Type::func(Type::Variable(0), Type::Boolean));
                let boolean_type = self.module_encoder.type_id(&Type::Boolean);
                self.get_symbol(&BinaryOp::DoubleEqual.path())
                    .call_function(
                        Type::Variable(0),
                        Type::func(Type::Variable(0), Type::Boolean),
                    )
                    .encode(LocalSet(function_temporary))
                    .encode(const_value)
                    .encode(LocalGet(function_temporary))
                    .call_function(Type::Variable(0), Type::Boolean)
                    .encode([
                        StructGet {
                            struct_type_index: boolean_type,
                            field_index: 0,
                        },
                        I32Const(1),
                        I32Xor,
                        BrIf(0),
                    ]);
            }
            PatternKind::TypeHint(p, _) => {
                self.lower_pattern(*p, scope);
            }
        };
    }
}
