use crate::asm::{
    Encoder,
    Instruction,
    NumberOperation,
    lower_type,
};
use crate::operator::{
    BinaryOp,
    UnaryOp,
};
use crate::semantic::WithType;

use super::*;

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
        (b::Minus, i::I64Op(Add)),
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
                let t = lower_type(&op.parameter_type(), syms);
                enc.extend([
                    i::Get(p1.clone()),
                    i::StructGet([t.clone()].into(), 0),
                    i::Get(p2.clone()),
                    i::StructGet([t.clone()].into(), 0),
                    instr,
                    i::StructNew([t.clone()].into()),
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
