use crate::Visit;
use crate::ir::{
    IrNode,
    Pattern,
};
use crate::operator::BinaryOp;

use super::*;

pub fn lower_type(
    type_: &semantic::Type,
    symbols: &SymbolTable,
) -> Type {
    use semantic::Type::*;
    match type_ {
        Any | Variable(_) => Type::Any,
        Unit => Type::Struct([].into()),
        Integer => Type::Struct([Type::I64].into()),
        Real => Type::Struct([Type::F64].into()),
        Glyph | Boolean => Type::Struct([Type::I32].into()),
        String => Type::Array(Type::I32.into()),
        Struct { fields, .. } => {
            Type::Struct(fields.values().map(|v| lower_type(v, symbols)).collect())
        }
        Array(t) => Type::Array(lower_type(t, symbols).into()),
        Tuple(items) => Type::Struct(items.iter().map(|i| lower_type(i, symbols)).collect()),
        Sum { .. } => Type::Struct([Type::I32, Type::Any].into()),
        Function(..) => Type::Struct([Type::Array(Type::Any.into()), Type::Function].into()),
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

use Instruction as i;
use indexmap::IndexMap;
impl<'a> Encoder<'a> {
    // Preconditions:
    // * Predicate to be pattern-matched on is top of stack
    // * A br 0 instruction indicates pattern matching has failed
    pub fn lower_pattern(
        &mut self,
        pat: Pattern,
        scope: ScopeKind,
        symbols: &SymbolTable,
    ) {
        use crate::ir::PatternKind as p;
        let type_ = lower_type(&pat.type_, symbols);
        match pat.inner.inner {
            p::Hole => {
                self.push(i::Drop);
            }
            p::Identifier(path) => {
                self.push(i::Set(path));
            }
            p::Tuple(items) => {
                let temporary = self.temporary_name("pattern");
                let semantic::Type::Tuple(types) = &pat.type_ else {
                    unreachable!()
                };
                let types = types
                    .iter()
                    .map(|t| lower_type(t, symbols))
                    .collect::<Box<_>>();
                self.new_register(temporary.clone(), scope, type_.clone());
                self.push(i::Set(temporary.clone()));
                for (index, item) in items.into_iter().enumerate() {
                    self.extend([
                        i::Get(temporary.clone()),
                        i::StructGet(types.clone(), index),
                    ]);
                    self.lower_pattern(item, scope, symbols);
                }
            }
            p::Array {
                starting,
                glob,
                ending,
                is_exact,
            } => todo!(),
            p::Struct(index_map) => {
                let temporary = self.temporary_name("pattern");
                self.new_register(temporary.clone(), scope, type_.clone());
                self.push(i::Set(temporary.clone()));
                let semantic::Type::Struct {
                    fields: ordered_fields,
                    ..
                } = &pat.type_
                else {
                    unreachable!()
                };
                let types = ordered_fields
                    .values()
                    .map(|t| lower_type(t, symbols))
                    .collect::<Box<_>>();
                for (name, pattern) in index_map {
                    let index = ordered_fields
                        .get_index_of(&name.inner)
                        .unwrap_or_else(|| unreachable!());
                    self.extend([
                        i::Get(temporary.clone()),
                        i::StructGet(types.clone(), index),
                    ]);
                    self.lower_pattern(pattern, scope, symbols);
                }
            }
            p::Constructor(constructor, inner) => todo!(),
            p::Immediate(const_value) => {
                self.extend([i::Const(const_value), i::Get(BinaryOp::DoubleEqual.path())]);
                self.call_closure();
                self.call_closure();
                self.extend([
                    i::StructGet([lower_type(&semantic::Type::Boolean, symbols)].into(), 0),
                    i::I32Op(NumberOperation::Xor),
                    i::BreakIf(0),
                ]);
            }
            p::TypeHint(inner, _) => {
                self.lower_pattern(*inner, scope, symbols);
            }
        }
    }
    pub fn lower_ir(
        &mut self,
        ir: IrNode,
        symbols: &SymbolTable,
    ) {
        use crate::ir::IrKind::*;
        match ir.inner.inner {
            Let {
                mut assignee,
                value,
                then,
                else_,
                ..
            } => {
                assignee.visit(|(path, type_)| {
                    self.new_register(path.clone(), ScopeKind::Local, lower_type(type_, symbols));
                });
                let result_type = lower_type(&ir.type_, symbols);
                self.extend([i::Block(Some(result_type)), i::Block(None)]);
                self.lower_ir(*value, symbols);
                self.lower_pattern(assignee, ScopeKind::Local, symbols);
                self.lower_ir(*then, symbols);
                self.extend([i::Break(1), i::End]);
                self.lower_ir(*else_, symbols);
                self.push(i::End);
            }
            Immediate(const_value) => {
                let type_ = lower_type(&const_value.type_of(), symbols);
                self.extend([i::Const(const_value), i::StructNew([type_].into())]);
            }
            Identifier(path) => {
                self.push(i::Get(path));
            }
            Tuple(items) => {
                let types = items
                    .iter()
                    .map(|i| lower_type(&i.type_, symbols))
                    .collect::<Box<[_]>>();
                items.into_iter().for_each(|i| {
                    self.lower_ir(i, symbols);
                });
                self.push(i::StructNew(types));
            }
            Struct(index_map) => {
                let semantic::Type::Struct { fields, .. } = &ir.type_ else {
                    unreachable!()
                };
                // Evaluate fields in order and store in temporaries
                let mut field_temporaries = IndexMap::new();
                for (field_name, field_ir) in index_map {
                    let temp_name = self.temporary_name(&field_name.inner);
                    let temp_type = lower_type(&field_ir.type_, symbols);
                    self.new_register(temp_name.clone(), ScopeKind::Local, temp_type);
                    self.lower_ir(field_ir, symbols);
                    self.push(i::Set(temp_name.clone()));
                    field_temporaries.insert(field_name.inner, temp_name);
                }
                // Re-order fields according to struct type and push onto stack
                let ordered_types = fields
                    .keys()
                    .map(|k| lower_type(&fields[k], symbols))
                    .collect::<Box<[_]>>();
                for field_name in fields.keys() {
                    let temp_name = &field_temporaries[field_name];
                    self.push(i::Get(temp_name.clone()));
                }
                self.push(i::StructNew(ordered_types));
            }
            Field { of, index } => {
                let semantic::Type::Struct { fields, .. } = &of.type_ else {
                    unreachable!()
                };
                let field_index = fields
                    .keys()
                    .position(|k| k == &index.inner)
                    .unwrap_or_else(|| {
                        unreachable!("Missing fields are caught during typechecking")
                    });
                let Type::Struct(inner_types) = lower_type(&of.type_, symbols) else {
                    unreachable!()
                };
                self.lower_ir(*of, symbols);
                self.push(i::StructGet(inner_types, field_index));
            }
            Function {
                parameter_name,
                captures: capture_names,
                capture_types,
                body,
                ..
            } => {
                let mut new_enc = self.module.new_function();
                let new_func_index = new_enc.func_index;
                let capture_array_name = new_enc.temporary_name("captured_symbols");
                new_enc.new_parameter(capture_array_name.clone(), Type::Array(Type::Any.into()));
                let semantic::Type::Function(parameter_type, return_type) = ir.type_ else {
                    unreachable!("Previously checked to be function type")
                };
                let parameter_type = lower_type(&parameter_type, symbols);
                let return_type = lower_type(&return_type, symbols);
                new_enc.new_parameter(parameter_name.inner, parameter_type);
                new_enc.new_return(return_type);
                for (id, (capture_name, capture_type)) in capture_names
                    .clone()
                    .into_iter()
                    .zip(capture_types)
                    .enumerate()
                {
                    let capture_type = lower_type(&capture_type, symbols);
                    new_enc.new_register(capture_name.clone(), ScopeKind::Local, capture_type);
                    new_enc.extend([
                        i::Get(capture_array_name.clone()),
                        i::Const(ConstValue::Integer(id as i64)),
                        i::ArrayGet(Type::Any),
                        i::Set(capture_name.clone()),
                    ]);
                }
                new_enc.lower_ir(*body, symbols);
                let num_captures = capture_names.len();
                for capture_name in capture_names {
                    self.push(i::Get(capture_name));
                }
                self.extend([
                    i::ArrayNewFixed {
                        inner_type: Type::Any,
                        length: num_captures,
                    },
                    i::Func(new_func_index),
                    i::StructNew([Type::function_capture(), Type::Function].into()),
                ])
            }
            Call { callee, argument } => {
                let callee_name = self.temporary_name("callee");
                let callee_type = lower_type(&callee.type_, symbols);
                self.new_register(callee_name.clone(), ScopeKind::Local, callee_type);
                self.lower_ir(*callee, symbols);
                self.push(i::Set(callee_name.clone()));
                self.lower_ir(*argument, symbols);
                self.push(i::Get(callee_name));
                self.call_closure();
            }
            Semicolon(a, b) => {
                self.lower_ir(*a, symbols);
                self.push(i::Drop);
                self.lower_ir(*b, symbols);
            }
            Unreachable => {
                self.push(i::Unreachable);
            }
        }
    }

    /// Expected stack before: [closure, argument]
    /// Expected stack after: [result]
    ///
    /// A closure is a struct with fields {captured_args, funcref}.
    /// The calling convention expects [argument, captured_args] on the stack.
    pub fn call_closure(&mut self) {
        let closure_type = Type::function_capture();
        let closure_name = self.temporary_name("closure");
        self.new_register(closure_name.clone(), ScopeKind::Local, closure_type.clone());
        self.push(i::Set(closure_name.clone()));

        let funcref_name = self.temporary_name("funcref");
        let funcref_type = Type::Function;
        self.new_register(funcref_name.clone(), ScopeKind::Local, funcref_type);
        self.extend([
            i::Get(closure_name.clone()),
            i::StructGet([Type::Any].into(), 1),
            i::Set(funcref_name.clone()),
        ]);

        self.extend([
            i::Get(closure_name),
            i::StructGet([Type::Any].into(), 0),
            i::Get(funcref_name),
            i::Call {
                parameters: [Type::Array(Type::Any.into()), Type::Any].into(),
                returns: [Type::Any].into(),
            },
        ]);
    }
}
