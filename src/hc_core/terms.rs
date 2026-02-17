use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreSymbols {
    EmptyArray,
    ArrayPush,
    ArrayConcat,
}

impl CoreSymbols {
    pub fn path(&self) -> Path {
        match self {
            CoreSymbols::EmptyArray => Path::core("array_empty"),
            CoreSymbols::ArrayPush => Path::core("array_push"),
            CoreSymbols::ArrayConcat => Path::core("array_concat"),
        }
    }
}

/*
pub fn operator_definitions(
    enc: &mut Encoder,
    syms: &mut SymbolTable,
) {
    syms.terms.extend(
        BinaryOp::all()
            .into_iter()
            .map(|b| (b.path(), b.get_type())),
    );
    syms.terms
        .extend(UnaryOp::all().into_iter().map(|u| (u.path(), u.get_type())));

    use NumberOperation::*;
    use {
        BinaryOp as b,
        Instruction as i,
    };
    let p1 = Path::new("[temp]", "p1");
    let p2 = Path::new("[temp]", "p2");
    [
        (b::Plus, i::I64Op(Add)),
        (b::Minus, i::I64Op(Sub)),
        (b::Star, i::I64Op(Mul)),
        (b::Slash, i::I64Op(Div)),
        (b::PlusDot, i::F64Op(Add)),
        (b::MinusDot, i::F64Op(Sub)),
        (b::StarDot, i::F64Op(Mul)),
        (b::SlashDot, i::F64Op(Div)),
        (b::And, i::I32Op(And)),
        (b::Or, i::I32Op(Or)),
        (b::Xor, i::I32Op(Xor)),
    ]
    .into_iter()
    .for_each(|(op, instr)| {
        enc.create_curried_closure(
            syms,
            &[
                p1.clone().with_type(op.parameter_type()),
                p2.clone().with_type(op.parameter_type()),
            ],
            vec![],
            |enc, syms| {
                let asm::Type::Struct(fields) = lower_type(&op.parameter_type(), syms) else {
                    unreachable!("operator parameter type must lower to a struct");
                };
                enc.extend([
                    i::Get(p1.clone()),
                    i::StructGet(fields.clone(), 0),
                    i::Get(p2.clone()),
                    i::StructGet(fields.clone(), 0),
                    instr,
                    i::StructNew(fields),
                ]);
            },
        );
        enc.new_register(
            op.path(),
            ScopeKind::Global,
            lower_type(&op.get_type(), syms),
        );
        enc.push(i::Set(op.path()));
    });
}

/// Register `put_str : string -> unit` and emit its WASI-backed implementation.
pub fn put_str_definition(
    enc: &mut Encoder,
    syms: &mut SymbolTable,
) {
    use Instruction as i;
    use NumberOperation::*;

    let put_str_type = Type::Function(Box::new(Type::String), Box::new(Type::Unit));
    let put_str_path = core("put_str");
    syms.terms
        .insert(put_str_path.clone(), put_str_type.clone());

    // Register fd_write as a function import
    let fd_write_path = Path::new("wasi_snapshot_preview1", "fd_write");
    enc.module.function_imports.insert(
        fd_write_path.clone(),
        FunctionImport {
            module: "wasi_snapshot_preview1".into(),
            name: "fd_write".into(),
            params: [ValType::I32, ValType::I32, ValType::I32, ValType::I32].into(),
            results: [ValType::I32].into(),
        },
    );
    enc.module.has_memory = true;

    let param = Path::new("[temp]", "str_param");
    enc.create_closure(
        syms,
        param.clone().with_type(Type::String),
        vec![],
        |enc, _syms| {
            // param is an (array i8) on the local stack
            // Memory layout:
            //   [0..4)  = iovec.buf pointer (will be 12)
            //   [4..8)  = iovec.buf_len
            //   [8..12) = nwritten (output)
            //   [12..)  = string data

            // Store iovec.buf = 12 at memory offset 0
            enc.extend([i::I32Const(0), i::I32Const(12), i::I32Store]);

            // Store iovec.buf_len = array.len(param) at memory offset 4
            enc.extend([
                i::I32Const(4),
                i::Get(param.clone()),
                i::ArrayLen,
                i::I32Store,
            ]);

            // Copy loop: for counter in 0..len, memory[12+counter] = param[counter]
            let counter = enc.temporary_name("counter");
            enc.new_register(counter.clone(), ScopeKind::Local, asm::Type::I32);
            enc.extend([i::I32Const(0), i::Set(counter.clone())]);

            // Only loop if length > 0
            enc.extend([
                i::Get(param.clone()),
                i::ArrayLen,
                i::I32Const(0),
                i::I32Op(Gt),
                i::If(None),
                i::Loop,
            ]);

            // address = 12 + counter
            enc.extend([i::I32Const(12), i::Get(counter.clone()), i::I32Op(Add)]);
            // value = param[counter]
            enc.extend([
                i::Get(param.clone()),
                i::Get(counter.clone()),
                i::ArrayGet(asm::Type::I8),
            ]);
            // store byte
            enc.push(i::I32Store8);

            // counter = counter + 1
            enc.extend([
                i::Get(counter.clone()),
                i::I32Const(1),
                i::I32Op(Add),
                i::Set(counter.clone()),
            ]);

            // branch back to loop if counter < array.len(param)
            enc.extend([
                i::Get(counter.clone()),
                i::Get(param.clone()),
                i::ArrayLen,
                i::I32Op(Lt),
                i::BreakIf(0), // branch to Loop
                i::End,        // end Loop
                i::End,        // end If
            ]);

            // Call fd_write(stdout=1, iovs_ptr=0, iovs_count=1, nwritten_ptr=8)
            enc.extend([
                i::I32Const(1), // fd: stdout
                i::I32Const(0), // iovs pointer
                i::I32Const(1), // iovs count
                i::I32Const(8), // nwritten pointer
                i::Call(fd_write_path.clone()),
                i::Drop, // discard errno
            ]);

            // Return unit
            enc.push(i::StructNew([].into()));
        },
    );

    enc.new_register(
        put_str_path.clone(),
        ScopeKind::Global,
        lower_type(&put_str_type, syms),
    );
    enc.push(i::Set(put_str_path));
}
*/
