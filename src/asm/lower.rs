use crate::Visit;
use crate::ir::{
    IrNode,
    Pattern,
};

use super::*;

pub fn lower_type(
    type_: &semantic::Type,
    symbols: &SymbolTable,
) -> Type {
    use semantic::Type::*;
    match type_ {
        Any | Variable(_) => Type::Any,
        Unit => Type::Struct(vec![]),
        Integer => Type::Struct(vec![Type::I64]),
        Real => Type::Struct(vec![Type::F64]),
        Glyph | Boolean => Type::Struct(vec![Type::I32]),
        String => Type::Array(Type::I32.into()),
        Struct { fields, .. } => {
            Type::Struct(fields.values().map(|v| lower_type(v, symbols)).collect())
        }
        Array(t) => Type::Array(lower_type(t, symbols).into()),
        Tuple(items) => Type::Struct(items.iter().map(|i| lower_type(i, symbols)).collect()),
        Sum { .. } => Type::Struct(vec![Type::I32, Type::Any]),
        Function(..) => Type::Function,
        Instantiation(path, items) => {
            lower_type(
                &symbols
                    .get_type(path)
                    .clone()
                    .instantiate(items)
                    .unwrap_or_else(|_| unreachable!()),
                symbols,
            )
        }
    }
}

pub fn lower_pattern(
    pat: Pattern,
    encoder: &mut Encoder,
    symbols: &SymbolTable,
) {
    todo!()
}

impl<'a> Encoder<'a> {
    pub fn lower_ir(
        &mut self,
        ir: IrNode,
        symbols: &SymbolTable,
    ) -> &mut Self {
        use crate::ir::IrKind::*;
        use Instruction as i;
        match ir.inner.inner {
            Let {
                mut assignee,
                is_global,
                value,
                then,
                else_,
            } => {
                todo!()
            }
            Immediate(const_value) => {
                let type_ = lower_type(&const_value.type_of(), symbols);
                self.extend([i::Const(const_value), i::StructNew(vec![type_])]);
            }
            Identifier(path) => {
                self.push(i::Get(path));
            }
            Tuple(items) => {
                let types = items
                    .iter()
                    .map(|i| lower_type(&i.type_, symbols))
                    .collect::<Vec<_>>();
                items.into_iter().for_each(|i| {
                    self.lower_ir(i, symbols);
                });
                self.push(i::StructNew(types));
            }
            Struct(index_map) => todo!(),
            Field { of, index } => todo!(),
            Function {
                parameter_name,
                parameter_type,
                captures,
                capture_types,
                body,
            } => todo!(),
            Call { callee, argument } => {
                let callee_type = lower_type(&callee.type_, symbols);
                let temporary = self.define_temporary(callee_type);
                self.lower_ir(*callee, symbols)
                    .push(i::Set(temporary.clone()))
                    .lower_ir(*argument, symbols)
                    .extend([i::Get(temporary), i::Call]);
            }
            Semicolon(a, b) => {
                self.lower_ir(*a, symbols)
                    .push(i::Drop)
                    .lower_ir(*b, symbols);
            }
            Unreachable => {
                self.push(i::Unreachable);
            }
        }
        self
    }
}
