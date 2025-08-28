use super::*;

impl FunctionEncoder<'_> {
    fn lower_pattern(&mut self, pattern: Pattern, temporary: u32) {
        match pattern.inner.inner {
            PatternKind::Name(path) => {
                self.encode(LocalGet(temporary))
                    .new_local(&path, &pattern.type_)
                    .set_symbol(&path);
            }
            PatternKind::Tuple(patterns) => {
                let next_temporary = self.new_temporary(&pattern.type_);
                let struct_type_id = self.module_encoder.type_id(&pattern.type_);
                for (index, pattern) in patterns.into_iter().enumerate() {
                    self.encode([
                        LocalGet(temporary),
                        StructGet {
                            struct_type_index: struct_type_id,
                            field_index: index as u32,
                        },
                        LocalSet(next_temporary),
                    ])
                    .lower_pattern(pattern, next_temporary);
                }
            }
            PatternKind::Constructor(
                Constructor {
                    variant,
                    in_type,
                    out_type,
                },
                pattern,
            ) => {
                let next_temporary = self.new_temporary(&pattern.type_);
                let out_type_id = self.module_encoder.type_id(&out_type);
                let in_type_id = self.module_encoder.type_id(&in_type);
                self.encode([
                    // Check sum type tag
                    I32Const(variant as i32),
                    LocalGet(temporary),
                    StructGet {
                        struct_type_index: out_type_id,
                        field_index: 0,
                    },
                    I32Ne,
                    BrIf(0),
                    // Pass on the inner value
                    LocalGet(temporary),
                    StructGet {
                        struct_type_index: out_type_id,
                        field_index: 1,
                    },
                    RefCastNonNull(HeapType::Concrete(in_type_id)),
                    LocalSet(next_temporary),
                ])
                .lower_pattern(*pattern, next_temporary);
            }
            PatternKind::Literal(const_value) => {
                todo!()
            }
        };
    }
}

impl Encode<Pattern> for FunctionEncoder<'_> {
    fn encode(&mut self, pattern: Pattern) -> &mut Self {
        let temporary = self.new_temporary(&pattern.type_);
        self.encode(LocalSet(temporary))
            .lower_pattern(pattern, temporary);
        self
    }
}

impl Encode<ConstValue> for FunctionEncoder<'_> {
    fn encode(&mut self, obj: ConstValue) -> &mut Self {
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

impl Encode<IrNode> for FunctionEncoder<'_> {
    fn encode(&mut self, node: IrNode) -> &mut Self {
        self.encode(node.type_.clone());
        match node.inner.inner {
            IrKind::Let {
                assignee,
                value,
                in_,
            } => self.encode(value).encode(assignee).encode(in_),
            IrKind::Immediate(const_value) => self.encode(const_value),
            IrKind::Identifier(path) => {
                self.get_symbol(&path);
                if !matches!(node.type_, Type::Variable(_)) {
                    let this_type_id = self.module_encoder.type_id(&node.type_);
                    self.encode(RefCastNonNull(HeapType::Concrete(this_type_id)));
                }
                self
            }
            IrKind::Struct {
                field_values: items,
                ..
            }
            | IrKind::Tuple(items) => {
                let type_id = self.module_encoder.type_id(&node.type_);
                self.encode(items.as_slice()).encode(StructNew(type_id))
            }
            IrKind::Field { of, index } => todo!(),
            IrKind::Function {
                parameter_name,
                parameter_type,
                captures,
                capture_types,
                body,
            } => {
                todo!()
            }
            IrKind::Call {
                callee,
                argument,
                argument_first,
            } => todo!(),
            IrKind::If {
                predicate,
                then,
                else_,
            } => {
                let boolean_type_id = self.module_encoder.type_id(&Type::Boolean);
                let result_valtype = self.module_encoder.valtype(&node.type_);
                self.encode(predicate)
                    .encode([
                        StructGet {
                            struct_type_index: boolean_type_id,
                            field_index: 0,
                        },
                        If(BlockType::Result(result_valtype)),
                    ])
                    .encode(then)
                    .encode(Else)
                    .encode(else_)
                    .encode(End)
            }
            IrKind::Match {
                scrutinee,
                predicates,
                branches,
            } => todo!(),
            IrKind::ImportedSymbol(path, _) => todo!(),
        }
    }
}
