use super::*;

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
                        self.module_encoder.new_global(path, &type_);
                    } else {
                        self.new_local(path, &type_);
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
                self.encode(items).encode(StructNew(type_id))
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
