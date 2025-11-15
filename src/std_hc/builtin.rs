use wasm_encoder::Instruction;

use super::*;

pub fn compile_builtin(enc: &mut FunctionEncoder, interface: &mut ModuleInterface) {
    primitive_types(enc, interface);
    operator_assembly(enc, interface);
}

fn primitive_types(enc: &mut FunctionEncoder, interface: &mut ModuleInterface) {
    [
        ("unit", Type::Unit),
        ("integer", Type::Integer),
        ("real", Type::Real),
        ("boolean", Type::Boolean),
        ("string", Type::String),
        ("glyph", Type::Glyph),
    ]
    .into_iter()
    .for_each(|(name, type_)| {
        enc.encode(type_.clone());
        let path = Path::from(STD_MODULE_NAME).child(name);
        interface.types.insert(path.clone());
        Universe::get().new_named_type(path, type_);
    });
}

fn operator_assembly(encoder: &mut FunctionEncoder, interface: &mut ModuleInterface) {
    // Unary operators
    let p1 = Path::from("a");
    // Unary - -.
    [
        (UnaryOp::Minus, I64Const(0), I64Sub),
        (UnaryOp::MinusDot, F64Const(0.0.into()), F64Sub),
        (UnaryOp::Not, I32Const(1), I32Xor),
    ]
    .into_iter()
    .for_each(|(op, zero, sub): (UnaryOp, Instruction, Instruction)| {
        interface.values.insert(op.path(), op.get_type());
        encoder.encode(op.get_type());
        encoder
            .module_encoder
            .new_global(&op.path(), &op.get_type());
        let capture_t = encoder
            .module_encoder
            .reduced_type_id(&ReducedType::capture());
        let type_id = encoder.module_encoder.type_id(&op.parameter_type());
        let func_t = encoder.module_encoder.type_id(&op.get_type());
        let id = encoder
            .module_encoder
            .function(p1.clone(), &op.parameter_type())
            .encode(zero)
            .get_symbol(&p1)
            .encode([
                StructGet {
                    struct_type_index: type_id,
                    field_index: 0,
                },
                sub,
                StructNew(type_id),
            ])
            .finish();
        encoder
            .encode([
                RefFunc(id),
                ArrayNewFixed {
                    array_type_index: capture_t,
                    array_size: 0,
                },
                StructNew(func_t),
            ])
            .set_symbol(&op.path());
    });
    let p2 = Path::from("b");
    // Binary |>
    {
        const OP: BinaryOp = BinaryOp::Apply;
        interface.values.insert(OP.path(), OP.get_type());
        encoder.encode(OP.get_type());
        encoder
            .module_encoder
            .new_global(&OP.path(), &OP.get_type());
        encoder
            .encode(curry_function_with_node(
                [
                    (p1.clone(), Type::Variable(0)),
                    (p2.clone(), Type::func(Type::Variable(0), Type::Variable(1))),
                ]
                .into_iter(),
                Type::Variable(1),
                IrKind::Call {
                    callee: IrKind::Identifier(p2.clone())
                        .with_default_span()
                        .with_type(Type::func(Type::Variable(0), Type::Variable(1)))
                        .into(),
                    argument: IrKind::Identifier(p1.clone())
                        .with_default_span()
                        .with_type(Type::Variable(0))
                        .into(),
                    opt: Default::default(),
                },
            ))
            .set_symbol(&OP.path());
    }
    // Binary >> (compose right)
    {
        let p1 = p1.clone();
        let p2 = p2.clone();
        let p3 = Path::from("c");
        let p1t = Type::func(Type::Variable(0), Type::Variable(1));
        let p2t = Type::func(Type::Variable(1), Type::Variable(2));
        let p3t = Type::Variable(0);
        n_params(
            encoder,
            interface,
            BinaryOp::ComposeRight.path(),
            [p1t.clone(), p2t.clone(), p3t.clone()],
            Type::Variable(1),
            move |e| {
                e.get_symbol(&p3)
                    .get_symbol(&p1)
                    .call_function(Type::Variable(0), Type::Variable(1))
                    .get_symbol(&p2)
                    .call_function(Type::Variable(1), Type::Variable(2));
            },
        );
    }
    // Binary << (compose right)
    {
        let p1 = p1.clone();
        let p2 = p2.clone();
        let p3 = Path::from("c");
        let p1t = Type::func(Type::Variable(1), Type::Variable(2));
        let p2t = Type::func(Type::Variable(0), Type::Variable(1));
        let p3t = Type::Variable(0);
        n_params(
            encoder,
            interface,
            BinaryOp::ComposeLeft.path(),
            [p1t.clone(), p2t.clone(), p3t.clone()],
            Type::Variable(1),
            move |e| {
                e.get_symbol(&p3)
                    .get_symbol(&p2)
                    .call_function(Type::Variable(0), Type::Variable(1))
                    .get_symbol(&p1)
                    .call_function(Type::Variable(1), Type::Variable(2));
            },
        );
    }
    // Binary ;
    {
        const OP: BinaryOp = BinaryOp::Semicolon;
        interface.values.insert(OP.path(), OP.get_type());
        encoder.encode(OP.get_type());
        encoder
            .module_encoder
            .new_global(&OP.path(), &OP.get_type());
        encoder
            .encode(curry_function_with_node(
                [
                    (p1.clone(), Type::Variable(0)),
                    (p2.clone(), Type::Variable(1)),
                ]
                .into_iter(),
                Type::Variable(1),
                IrKind::Identifier(p2.clone()),
            ))
            .set_symbol(&OP.path());
    }
    use BinaryOp::*;
    // Binary + - * / % +. -. *. /. and or xor
    [
        (Plus, I64Add),
        (Slash, I64DivS),
        (Minus, I64Sub),
        (Star, I64Mul),
        (Percent, I64RemS),
        (PlusDot, F64Add),
        (MinusDot, F64Sub),
        (StarDot, F64Mul),
        (SlashDot, F64Div),
        (And, I32And),
        (Or, I32Or),
        (Xor, I32Xor),
    ]
    .into_iter()
    .for_each(|(op, instr): (BinaryOp, Instruction)| {
        interface.values.insert(op.path(), op.get_type());
        encoder.encode(op.get_type());
        encoder
            .module_encoder
            .new_global(&op.path(), &op.get_type());
        let p1 = p1.clone();
        let p2 = p2.clone();
        encoder
            .encode(curry_function(
                [
                    (p1.clone(), op.parameter_type()),
                    (p2.clone(), op.parameter_type()),
                ],
                op.return_type(),
                move |enc| {
                    let struct_type = enc.module_encoder.type_id(&op.parameter_type());
                    enc.get_symbol(&p1)
                        .encode(StructGet {
                            struct_type_index: struct_type,
                            field_index: 0,
                        })
                        .get_symbol(&p2)
                        .encode([
                            StructGet {
                                struct_type_index: struct_type,
                                field_index: 0,
                            },
                            instr.clone(),
                            StructNew(struct_type),
                        ]);
                },
            ))
            .set_symbol(&op.path());
    });
    [
        (DoubleEqual, I64Eq, F64Eq, I32Eq, true),
        (BangEqual, I64Ne, F64Ne, I32Ne, false),
        (LessEqual, I64LeS, F64Le, I32LeS, true),
        (GreaterEqual, I64GeS, F64Ge, I32GeS, true),
        (Less, I64LtS, F64Lt, I32LtS, false),
        (Greater, I64GtS, F64Gt, I32GtS, false),
    ]
    .into_iter()
    .for_each(
        |(op, integer_op, real_op, glyph_op, unit_op): (
            BinaryOp,
            Instruction,
            Instruction,
            Instruction,
            bool,
        )| {
            interface.values.insert(op.path(), op.get_type());
            encoder.encode(op.get_type());
            encoder
                .module_encoder
                .new_global(&op.path(), &op.get_type());
            let p1 = p1.clone();
            let p2 = p2.clone();
            encoder
                .encode(curry_function(
                    [
                        (p1.clone(), Type::Variable(0)),
                        (p2.clone(), Type::Variable(0)),
                    ],
                    Type::Boolean,
                    move |enc| {
                        (0..6).for_each(|_| {
                            enc.encode(Block(BlockType::Result(ValType::Ref(RefType::ANYREF))));
                        });
                        let integer_type = enc.module_encoder.type_id(&Type::Integer);
                        let real_type = enc.module_encoder.type_id(&Type::Real);
                        let boolean_type = enc.module_encoder.type_id(&Type::Boolean);
                        let glyph_type = enc.module_encoder.type_id(&Type::Glyph);
                        let unit_type = enc.module_encoder.type_id(&Type::Unit);
                        let type_ops = [
                            (integer_type, integer_op.clone()),
                            (real_type, real_op.clone()),
                            (boolean_type, glyph_op.clone()), /* Glyph and boolean are always
                                                               * the
                                                               * same */
                            (glyph_type, glyph_op.clone()),
                            (unit_type, I32Const(unit_op as i32)),
                        ];
                        let br_on = |id, depth| BrOnCast {
                            relative_depth: depth,
                            from_ref_type: RefType::ANYREF,
                            to_ref_type: RefType {
                                nullable: false,
                                heap_type: HeapType::Concrete(id),
                            },
                        };
                        enc.get_symbol(&p1);
                        type_ops
                            .clone()
                            .into_iter()
                            .map(|(t, _)| t)
                            .enumerate()
                            .for_each(|(depth, t)| {
                                enc.encode(br_on(t, depth as u32));
                            });
                        let string_type = enc.module_encoder.type_id(&Type::String);
                        enc.encode([br_on(string_type.clone(), 5), Unreachable, End]);
                        // Basic comparisons
                        type_ops.clone().into_iter().for_each(|(type_, op)| {
                            // Get inner if not unit
                            if type_ != unit_type {
                                enc.encode([
                                    RefCastNonNull(HeapType::Concrete(type_)),
                                    StructGet {
                                        struct_type_index: type_,
                                        field_index: 0,
                                    },
                                ])
                                .get_symbol(&p2)
                                .encode([
                                    RefCastNonNull(HeapType::Concrete(type_)),
                                    StructGet {
                                        struct_type_index: type_,
                                        field_index: 0,
                                    },
                                ]);
                            }
                            // Do comparison
                            enc.encode([op, StructNew(boolean_type), Return, End]);
                        });
                        // String comparison
                        enc.encode([RefCastNonNull(HeapType::Concrete(string_type)), ArrayLen])
                            .get_symbol(&p2)
                            .encode([
                                RefCastNonNull(HeapType::Concrete(string_type)),
                                ArrayLen,
                                I32GtU,
                                // Early exit if len(first) > len(second)
                                If(BlockType::Empty),
                                match op {
                                    DoubleEqual | LessEqual | Less => I32Const(0),
                                    BangEqual | GreaterEqual | Greater => I32Const(1),
                                    _ => unreachable!(),
                                },
                                StructNew(boolean_type),
                                Return,
                                End,
                            ])
                            .get_symbol(&p1)
                            .encode([RefCastNonNull(HeapType::Concrete(string_type)), ArrayLen])
                            .get_symbol(&p2)
                            .encode([
                                RefCastNonNull(HeapType::Concrete(string_type)),
                                ArrayLen,
                                I32GtU,
                                // Early exit if len(second) > len(first)
                                If(BlockType::Empty),
                                match op {
                                    DoubleEqual | GreaterEqual | Greater => I32Const(0),
                                    BangEqual | LessEqual | Less => I32Const(1),
                                    _ => unreachable!(),
                                },
                                StructNew(boolean_type),
                                Return,
                                End,
                            ]);
                        let index = enc.new_raw_temporary(ValType::I32);
                        let length = enc.new_raw_temporary(ValType::I32);
                        enc.encode([I32Const(0), LocalSet(index)])
                            .get_symbol(&p1)
                            .encode([
                                RefCastNonNull(HeapType::Concrete(string_type)),
                                ArrayLen,
                                LocalSet(length),
                                // Lexical comparison
                                Loop(BlockType::Empty),
                            ])
                            .get_symbol(&p1)
                            .encode([
                                RefCastNonNull(HeapType::Concrete(string_type)),
                                LocalGet(index),
                                ArrayGetU(string_type),
                            ])
                            .get_symbol(&p2)
                            .encode([
                                RefCastNonNull(HeapType::Concrete(string_type)),
                                LocalGet(index),
                                ArrayGetU(string_type),
                                match op {
                                    BinaryOp::DoubleEqual => I32Ne,
                                    BinaryOp::BangEqual => I32Eq,
                                    BinaryOp::LessEqual => I32GtU,
                                    BinaryOp::GreaterEqual => I32LtU,
                                    BinaryOp::Less => I32GeU,
                                    BinaryOp::Greater => I32LeU,
                                    _ => unreachable!(),
                                },
                                If(BlockType::Empty),
                                I32Const(0),
                                StructNew(boolean_type),
                                Return,
                                End,
                                LocalGet(index),
                                I32Const(1),
                                I32Add,
                                LocalTee(index),
                                LocalGet(length),
                                I32LtU,
                                BrIf(0),
                                End,
                                I32Const(1),
                                StructNew(boolean_type),
                                Return,
                            ]);
                    },
                ))
                .set_symbol(&op.path());
        },
    );
}
