use std::rc::Rc;

use super::*;
use crate::{lint::*, parse::*};

pub fn build_ir(
    module: ParsedModule,
    context: &HashMap<Path, ModuleInterface>,
) -> Result<IrModule> {
    let mut items = vec![];
    let mut ir = vec![];
    let mut value_ns = ValueNameSpace::new(Path::from(module.name.as_str()));
    let mut type_ns = TypeNameSpace::new(Path::from(module.name.as_str()));
    for expr in module.contents {
        use ModuleExpressionKind as e;
        match expr.kind {
            // Recursive let
            e::Let {
                assignee,
                value:
                    value @ box Expression {
                        kind: ValueExpressionKind::FunctionDef { .. },
                        ..
                    },
            } => {
                let mangle = pattern(&mut value_ns, assignee, true)?;
                let value = value_expr(&mut ir, &mut value_ns, &type_ns, *value)?;
                items.push(ModuleItem::Let(mangle, value));
            }
            e::Let { assignee, value } => {
                let value = value_expr(&mut ir, &mut value_ns, &type_ns, *value)?;
                let mangle = pattern(&mut value_ns, assignee, true)?;
                items.push(ModuleItem::Let(mangle, value));
            }
            e::Type {
                assignee,
                assignee_span,
                value,
            } => {
                // Recursive type
                if matches!(value.kind, TypeDefinitionKind::Sum { .. }) {
                    let mangle = type_ns
                        .define_global(&assignee, Type::TypeVariable(0).into())
                        .span(assignee_span)?;
                    let type_ = type_def(&mut type_ns, *value)?;
                    let weak = Type::Weak(Rc::downgrade(&type_));
                    type_.borrow_mut().unify(0, &weak);
                    type_ns.update_type(&mangle, type_.clone());
                    items.push(ModuleItem::Type(mangle, type_));
                }
                // Non-recursive type
                else {
                    let type_ = type_def(&mut type_ns, *value)?;
                    let mangle = type_ns
                        .define_global(&assignee, type_.clone())
                        .span(expr.span)?;
                    items.push(ModuleItem::Type(mangle, type_));
                }
            }
            e::Import { name } => {
                let interface = context.get(&name.clone().into()).ok_or(lint(
                    NameLint::NoSuchModule,
                    expr.span,
                    [name.clone()],
                ))?;
                value_ns.import_module(interface.values.clone());
                type_ns.import_module(interface.types.clone());
            }
        }
    }
    Ok(IrModule {
        module_name: module.name.into(),
        universe: type_ns.to_universe(),
        items,
        nodes: ir,
    })
}

fn lit(literal: Literal) -> Result<ConstValue> {
    fn int(value: &str, base: u32) -> Result<i64> {
        i64::from_str_radix(value, base).lint(TokenLint::InvalidInteger)
    }
    fn real(value: &str) -> Result<f64> {
        value.parse().lint(TokenLint::InvalidReal)
    }

    Ok(match literal {
        crate::parse::Literal::Unit => ConstValue::Unit,
        crate::parse::Literal::Integer(i, base) => ConstValue::Integer(int(&i, base as u32)?),
        crate::parse::Literal::Real(r) => ConstValue::Real(real(&r)?),
        crate::parse::Literal::String(s) => ConstValue::String(s),
        crate::parse::Literal::Glyph(g) => ConstValue::Glyph(g),
        crate::parse::Literal::Boolean(b) => ConstValue::Boolean(b),
    })
}

pub fn value_expr(
    module: &mut Vec<IrNode>,
    ns: &mut ValueNameSpace,
    tns: &TypeNameSpace,
    expr: ValueExpression,
) -> Result<IrPtr> {
    use IrKind as ir;
    use ValueExpressionKind::*;
    let span = expr.span;
    let ptr = module.len();
    macro_rules! rec {
        ($e:expr) => {
            value_expr(module, ns, tns, $e)
        };
    }
    module.push(IrNode {
        kind: IrKind::Immediate(ConstValue::Unit),
        span,
        type_: Default::default(),
    });
    let kind = match expr.kind {
        // Recursive let
        Let {
            assignee,
            value:
                value @ box Expression {
                    kind: FunctionDef { .. },
                    ..
                },
            in_,
            ..
        } => {
            let assignee = pattern(ns, assignee, false)?;
            let value = rec!(*value)?;
            let in_ = if let Some(in_) = in_ {
                Some(rec!(*in_)?)
            } else {
                None
            };
            (0..assignee.introduced_names()).for_each(|_| ns.end_local_scope());
            ir::Declaration {
                assignee,
                value,
                in_,
            }
        }
        Let {
            assignee,
            value,
            in_,
            ..
        } => {
            let value = rec!(*value)?;
            let assignee = pattern(ns, assignee, false)?;
            let in_ = if let Some(in_) = in_ {
                Some(rec!(*in_)?)
            } else {
                None
            };
            (0..assignee.introduced_names()).for_each(|_| ns.end_local_scope());
            ir::Declaration {
                assignee,
                value,
                in_,
            }
        }
        Literal(literal) => ir::Immediate(lit(literal).span(span)?),
        Identifier(name) => ir::Identifier(ns.get(&name).span(expr.span)?),
        Binary { op, left, right } => ir::Binary {
            op,
            left: rec!(*left)?,
            right: rec!(*right)?,
        },
        Unary { op, child } => ir::Unary {
            op,
            child: rec!(*child)?,
        },
        FunctionDef {
            mut arguments,
            mut argument_spans,
            mut types,
            body,
        } => {
            if arguments.len() == 0 {
                ns.begin_capture();
                let parameter_span = span;
                let body = rec!(*body)?;
                let captures = ns.end_capture();
                let capture_types = vec![Type::Any.to_ref(); captures.len()];
                ir::FunctionDef {
                    parameter_name: None,
                    parameter_span,
                    parameter_type: None,
                    captures,
                    capture_types,
                    body,
                }
            } else {
                ns.begin_capture();
                let (argument, new_arguments) = arguments.split_first().unwrap();
                let parameter_name = ns.define_local(&argument);
                arguments = new_arguments.to_vec();
                let (parameter_span, new_spans) = argument_spans.split_first().unwrap();
                let parameter_span = parameter_span.clone();
                argument_spans = new_spans.to_vec();
                let (type_, new_type_s) = types.split_first().unwrap();
                let parameter_type = if let Some(type_) = type_.clone() {
                    Some(type_expr(tns, type_)?)
                } else {
                    None
                };
                types = new_type_s.to_vec();
                let body = if arguments.len() == 0 {
                    rec!(*body)?
                } else {
                    rec!(Expression {
                        kind: FunctionDef {
                            arguments,
                            argument_spans,
                            types,
                            body,
                        },
                        span,
                    })?
                };
                let captures = ns.end_capture();
                ns.end_local_scope();
                let capture_types = vec![Type::Any.to_ref(); captures.len()];
                ir::FunctionDef {
                    parameter_name: Some(parameter_name),
                    parameter_span,
                    parameter_type,
                    captures,
                    capture_types,
                    body,
                }
            }
        }
        FunctionCall { callee, argument } => {
            let callee = rec!(*callee)?;
            let argument = rec!(*argument)?;
            ir::FunctionCall { callee, argument }
        }
        If {
            predicate,
            then,
            else_,
        } => ir::If {
            predicate: rec!(*predicate)?,
            then: rec!(*then)?,
            else_: if let Some(else_) = else_ {
                Some(rec!(*else_)?)
            } else {
                None
            },
        },
        Match {
            scrutinee,
            predicates,
            branches,
        } => {
            let scrutinee = rec!(*scrutinee)?;
            let mut ir_predicates = vec![];
            let mut predicate_spans = vec![];
            let mut ir_branches = vec![];
            for (p, b) in predicates.into_iter().zip(branches) {
                predicate_spans.push(p.span);
                let predicate = pattern(ns, p, false)?;
                let introduced_names = predicate.introduced_names();
                ir_predicates.push(predicate);
                let branch = rec!(b)?;
                ir_branches.push(branch);
                (0..introduced_names).for_each(|_| ns.end_local_scope());
            }
            ir::Match {
                scrutinee,
                predicates: ir_predicates,
                branches: ir_branches,
            }
        }
        Tuple(expressions) => ir::Tuple(expressions.into_iter().map(|e| rec!(e)).try_collect()?),
        StructureLiteral { lhs, rhs } => ir::StructLiteral {
            field_names: lhs,
            field_values: rhs.into_iter().map(|e| rec!(e)).try_collect()?,
        },
        Field { lhs, rhs } => ir::Field {
            of: rec!(*lhs)?,
            index: rhs,
        },
        ModuleField(path) => {
            let path = Path::from(path);
            ir::ImportedSymbol(
                path.clone().into(),
                ns.get_import_type(&path.into()).span(span)?,
            )
        }
    };
    module[ptr].kind = kind;
    Ok(ptr)
}

fn pattern(ns: &mut ValueNameSpace, expr: PatternExpression, global: bool) -> Result<Pattern> {
    Ok(Pattern {
        kind: match expr.kind {
            PatternExpressionKind::Literal(literal) => {
                PatternKind::Literal(lit(literal).span(expr.span)?)
            }
            PatternExpressionKind::Identifier(name) if !global => {
                PatternKind::Name(ns.define_local(&name))
            }
            PatternExpressionKind::Identifier(name) => {
                PatternKind::Name(ns.define_global(&name).span(expr.span)?)
            }
            PatternExpressionKind::Tuple(expressions) => PatternKind::Tuple(
                expressions
                    .into_iter()
                    .map(|e| pattern(ns, e, global))
                    .try_collect()?,
            ),
        },
        type_: Type::default().into(),
        span: expr.span,
    })
}

pub fn type_def(ns: &mut TypeNameSpace, expr: TypeDefinition) -> Result<TypeRef> {
    use TypeDefinitionKind::*;
    Ok(match expr.kind {
        Structure { lhs, rhs } => Type::Struct {
            member_names: lhs,
            member_types: rhs.into_iter().map(|e| type_expr(ns, e)).try_collect()?,
        }
        .to_ref(),
        Sum {
            variant_names,
            variant_types,
        } => Type::Sum {
            variant_names,
            variant_types: variant_types
                .into_iter()
                .map(|e| type_expr(ns, e))
                .try_collect()?,
        }
        .to_ref(),
        Expression(expression) => type_expr(ns, expression)?,
    })
}

pub fn type_expr(ns: &TypeNameSpace, expr: TypeExpression) -> Result<TypeRef> {
    use TypeExpressionKind::*;
    Ok(match expr.kind {
        Identifier(name) => ns.get_type(&name).span(expr.span)?,
        Product(expressions) => Type::Product(
            expressions
                .into_iter()
                .map(|e| type_expr(ns, e))
                .try_collect()?,
        )
        .to_ref(),
        ModulePath(items) => ns.get_import_type(&items.into()).span(expr.span)?,
    })
}
