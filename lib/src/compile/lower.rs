use crate::{WithSpan, operator::BinaryOp, optimize::CallOptimization};

use super::*;

impl FunctionEncoder<'_> {
    fn lower_pattern(&mut self, pattern: Pattern, scope: ScopeKind) {
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

impl Encode<(Pattern, ScopeKind)> for FunctionEncoder<'_> {
    fn encode(&mut self, (pattern, scope): (Pattern, ScopeKind)) -> &mut Self {
        self.lower_pattern(pattern, scope);
        self
    }
}

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

impl Encode<ModuleItem> for FunctionEncoder<'_> {
    fn encode(&mut self, item: ModuleItem) -> &mut Self {
        match item {
            ModuleItem::Let(mut pattern, node) => {
                pattern.visit(|(path, type_)| {
                    self.module_encoder.new_global(path, type_);
                });
                self.encode(node.type_.clone())
                    .encode(node)
                    .encode((pattern, ScopeKind::Global))
            }
            ModuleItem::Constructor(
                path,
                Constructor {
                    variant_id,
                    kind: ConstructorKind::Unitary(t),
                },
            ) => {
                self.encode(t.clone());
                let type_id = self.module_encoder.type_id(&t);
                self.module_encoder.new_global(&path, &t);
                self.encode(I32Const(variant_id as i32))
                    .encode(ConstValue::Unit)
                    .encode(StructNew(type_id))
                    .set_symbol(&path)
            }
            ModuleItem::Constructor(
                path,
                Constructor {
                    variant_id,
                    kind: ConstructorKind::Function(in_type, out_type),
                },
            ) => {
                self.encode([
                    in_type.clone(),
                    out_type.clone(),
                    Type::func(in_type.clone(), out_type.clone()),
                ]);
                let function_type = Type::func(in_type.clone(), out_type.clone());
                let parameter_name = Path::from("_");
                let struct_type_id = self.module_encoder.reduced_type_id(&ReducedType::Sum);
                let capture_type_id = self.module_encoder.reduced_type_id(&ReducedType::capture());
                let function_id = self
                    .module_encoder
                    .new_global(&path, &function_type)
                    .function(parameter_name.clone(), &in_type)
                    .encode(I32Const(variant_id as i32))
                    .get_symbol(&parameter_name)
                    .encode(StructNew(struct_type_id))
                    .finish();
                self.encode([
                    RefFunc(function_id),
                    ArrayNewFixed {
                        array_type_index: capture_type_id,
                        array_size: 0,
                    },
                    StructNew(self.module_encoder.type_id(&function_type)),
                ])
                .set_symbol(&path)
            }
            ModuleItem::Import {
                path,
                type_,
                major,
                minor,
            } => {
                self.module_encoder.new_global(&path, &type_.into());
                self.new_import_function(major, minor, type_)
                    .set_symbol(&path)
            }
            ModuleItem::Type(_) => self,
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

    pub fn call_function(&mut self, argument_type: Type, return_type: Type) -> &mut Self {
        self.call_function_maybe_tail(argument_type, return_type, false)
    }

    pub fn tail_call_function(&mut self, argument_type: Type, return_type: Type) -> &mut Self {
        self.call_function_maybe_tail(argument_type, return_type, true)
    }
}

impl Encode<IrNode> for FunctionEncoder<'_> {
    fn encode(&mut self, node: IrNode) -> &mut Self {
        self.encode(node.type_.clone());
        match node.inner.inner {
            IrKind::Let {
                mut assignee,
                value,
                in_,
            } => {
                assignee.visit(|(path, type_)| {
                    if let Type::Function(..) = type_ {
                        self.module_encoder.new_global(path, type_);
                    } else {
                        self.new_local(path, type_);
                    }
                });
                self.encode(value)
                    .encode((assignee, ScopeKind::Local))
                    .encode(in_)
            }
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
            IrKind::Field { .. } => todo!(),
            IrKind::Function {
                parameter_name,
                captures,
                capture_types,
                body,
                ..
            } => {
                let parameter_name = parameter_name
                    .unwrap_or(Path::from("_").with_default_span())
                    .inner;
                let Type::Function(parameter_type, _) = node.type_.clone() else {
                    panic!()
                };
                let (captures, capture_types): (Vec<_>, Vec<_>) = captures
                    .into_iter()
                    .zip(capture_types)
                    .filter(|(c, _)| !self.module_encoder.global_exists(c))
                    .unzip();
                let function = RefFunc(
                    self.module_encoder
                        .function(parameter_name, &parameter_type)
                        .with_capture(&captures, &capture_types)
                        .encode(body)
                        .finish(),
                );
                self.encode(function);
                for capture in &captures {
                    self.get_symbol(capture);
                }
                self.encode([
                    ArrayNewFixed {
                        array_type_index: self
                            .module_encoder
                            .reduced_type_id(&ReducedType::capture()),
                        array_size: captures.len() as u32,
                    },
                    StructNew(self.module_encoder.type_id(&node.type_)),
                ])
            }
            IrKind::Call {
                callee,
                argument,
                opt,
            } => match opt {
                CallOptimization::None => {
                    let return_type = node.type_.clone().reduce();
                    let cast = if return_type != ReducedType::AnyRef {
                        Some(RefCastNonNull(HeapType::Concrete(
                            self.module_encoder.type_id(&node.type_.clone()),
                        )))
                    } else {
                        None
                    };
                    let temporary = self.new_temporary(&callee.type_);
                    self.encode(callee)
                        .encode([LocalSet(temporary)])
                        .encode(argument.clone())
                        .encode(LocalGet(temporary))
                        .call_function(argument.type_, node.type_.clone())
                        .encode(cast)
                }
                CallOptimization::Tail => {
                    let temporary = self.new_temporary(&callee.type_);
                    self.encode(callee)
                        .encode([LocalSet(temporary)])
                        .encode(argument.clone())
                        .encode(LocalGet(temporary))
                        .tail_call_function(argument.type_, node.type_.clone())
                }
            },
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
            } => {
                let branches_valtype = self.module_encoder.valtype(&node.type_);
                let scrutinee_temp = self.new_temporary(&scrutinee.type_);
                self.encode(scrutinee).encode([
                    LocalSet(scrutinee_temp),
                    Block(BlockType::Result(branches_valtype)),
                ]);
                for (mut pattern, branch) in predicates.into_iter().zip(branches) {
                    pattern.visit(|(path, type_)| {
                        self.new_local(path, type_);
                    });
                    self.encode(Block(BlockType::Empty))
                        .encode(LocalGet(scrutinee_temp))
                        .encode((pattern, ScopeKind::Local))
                        .encode(branch)
                        .encode([Br(1), End]);
                }
                self.encode([Unreachable, End])
            }
            IrKind::ImportedSymbol(path, _) => {
                self.get_symbol(&path);
                if !matches!(node.type_, Type::Variable(_)) {
                    let this_type_id = self.module_encoder.type_id(&node.type_);
                    self.encode(RefCastNonNull(HeapType::Concrete(this_type_id)));
                }
                self
            }
            IrKind::AsmLiteral(f) => {
                f.0(self);
                self
            }
            IrKind::Semicolon(a, b) => self.encode(a).encode(Drop).encode(b),
        }
    }
}
