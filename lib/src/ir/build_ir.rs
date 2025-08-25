use super::*;
use crate::{lint::*, parse::*};

pub fn build_ir(
    module: ParsedModule,
    context: &HashMap<Path, ModuleInterface>,
) -> Result<IrModule> {
    let module_name = Path::from((*module.name).clone());
    let mut ns = ModuleNameSpace::new(module_name.clone());
    let mut items = vec![];
    for item in module.contents.clone() {
        module_expr(&mut ns, context, item, &mut items)?;
    }
    Ok(IrModule { module_name, items })
}

fn module_expr(
    ns: &mut ModuleNameSpace,
    context: &HashMap<Path, ModuleInterface>,
    e: ModuleExpression,
    items: &mut Vec<ModuleItem>,
) -> Result<()> {
    match e.inner {
        ModuleExpressionKind::Let { assignee, value } => {
            let assignee = pattern_expr(ns, assignee, true)?;
            let value = value_expr(ns, *value)?;
            items.push(ModuleItem::Let(assignee, Box::new(value)));
        }
        ModuleExpressionKind::Type {
            assignee,
            assignee_span,
            value,
        } => {
            type_def(ns, assignee, assignee_span, *value, items)?;
        }
        ModuleExpressionKind::Import { name } => {
            let interface = context.get(&name.clone().into()).ok_or(lint(
                NameLint::NoSuchModule,
                e.span,
                [name],
            ))?;
            ns.import_module(interface).span(e.span)?;
        }
    }
    Ok(())
}

fn lit(literal: Literal) -> Result<ConstValue> {
    fn int(value: &str, base: u32) -> Result<i64> {
        i64::from_str_radix(value, base).lint(TokenLint::InvalidInteger)
    }
    fn real(value: &str) -> Result<f64> {
        value.parse().lint(TokenLint::InvalidReal)
    }

    Ok(match literal {
        Literal::Unit => ConstValue::Unit,
        Literal::Integer(i, base) => ConstValue::Integer(int(&i, base as u32)?),
        Literal::Real(r) => ConstValue::Real(real(&r)?),
        Literal::String(s) => ConstValue::String(s),
        Literal::Glyph(g) => ConstValue::Glyph(g),
        Literal::Boolean(b) => ConstValue::Boolean(b),
    })
}

pub fn value_expr(ns: &mut ModuleNameSpace, expr: ValueExpression) -> Result<IrNode> {
    use IrKind as ir;
    use ValueExpressionKind::*;
    let span = expr.span;
    macro_rules! rec {
        ($e:expr) => {
            Box::new(value_expr(ns, *$e)?)
        };
    }
    Ok(match expr.inner {
        Let {
            assignee,
            value,
            in_,
        } => {
            let assignee = pattern_expr(ns, assignee, false)?;
            let value = rec!(value);
            let in_ = match in_ {
                Some(in_) => Some(rec!(in_)),
                None => None,
            };
            ns.values.end_local_scopes(assignee.introduced_names());
            ir::Let {
                assignee,
                value,
                in_,
            }
        }
        Literal(literal) => ir::Immediate(lit(literal).span(span)?),
        Identifier(ident) => ir::Identifier(ns.get_value(&ident).span(span)?),
        BinaryOp(op) => ir::ImportedSymbol(op.path(), op.get_type()),
        Binary { op, left, right } => {
            let left_span = left.span;
            ir::Call {
                callee: ir::Call {
                    callee: ir::ImportedSymbol(op.path(), op.get_type())
                        .with_span(expr.span)
                        .with_type(Type::Any)
                        .into(),
                    argument: rec!(left),
                    argument_first: true,
                }
                .with_span(left_span)
                .with_type(Type::Any)
                .into(),
                argument: rec!(right),
                argument_first: true,
            }
        }
        Unary { op, child } => ir::Call {
            callee: ir::ImportedSymbol(op.path(), op.get_type())
                .with_span(expr.span)
                .with_type(Type::Any)
                .into(),
            argument: rec!(child),
            argument_first: true,
        },
        FunctionShorthand {
            predicates,
            branches,
        } => {
            const SHORTHAND_NAME: &str = "~";
            return value_expr(
                ns,
                FunctionDef {
                    arguments: vec![SHORTHAND_NAME.to_string().with_span(expr.span)],
                    types: vec![None],
                    body: Match {
                        scrutinee: Identifier(SHORTHAND_NAME.into())
                            .with_span(expr.span)
                            .into(),
                        predicates,
                        branches,
                    }
                    .with_span(expr.span)
                    .into(),
                }
                .with_span(expr.span),
            );
        }
        FunctionDef {
            arguments,
            types,
            body,
        } => {
            // Zero parameter function has implicit unit parameter
            if arguments.is_empty() {
                ns.begin_capture();
                let body = rec!(body);
                let captures = ns.end_capture();
                ir::Function {
                    parameter_name: None,
                    parameter_type: Some(Type::Unit),
                    capture_types: vec![Type::Any; captures.len()],
                    captures,
                    body,
                }
            } else {
                fn curry(
                    ns: &mut ModuleNameSpace,
                    mut arguments: impl Iterator<Item = (Spanned<String>, Option<TypeExpression>)>,
                    body: Box<ValueExpression>,
                    span: Span,
                ) -> Result<Box<IrNode>> {
                    Ok(Box::new(
                        match arguments.next() {
                            Some((argument, type_)) => {
                                ns.begin_capture();
                                let parameter_name = Some(
                                    ns.define_local_value(&argument, true)
                                        .with_span(argument.span),
                                );
                                let body = curry(ns, arguments, body, span)?;
                                let captures = ns.end_capture();
                                ns.values.end_local_scopes(1);
                                ir::Function {
                                    parameter_name,
                                    parameter_type: match type_ {
                                        Some(t) => Some(type_expr(ns, t)?),
                                        None => None,
                                    },
                                    capture_types: vec![Type::Any; captures.len()],
                                    captures,
                                    body,
                                }
                            }
                            None => return value_expr(ns, *body).map(Box::new),
                        }
                        .with_span(span)
                        .with_type(Type::Any),
                    ))
                }
                return Ok(*curry(ns, arguments.into_iter().zip(types), body, span)?);
            }
        }
        FunctionCall { callee, argument } => ir::Call {
            callee: rec!(callee),
            argument: rec!(argument),
            argument_first: false,
        },
        If {
            predicate,
            then,
            else_,
        } => ir::If {
            predicate: rec!(predicate),
            then: rec!(then),
            else_: match else_ {
                Some(else_) => Some(rec!(else_)),
                None => None,
            },
        },
        Match {
            scrutinee,
            predicates,
            branches,
        } => {
            let scrutinee = rec!(scrutinee);
            let mut new_predicates = vec![];
            let mut new_branches = vec![];
            for (predicate, branch) in predicates.into_iter().zip(branches) {
                let predicate = pattern_expr(ns, predicate, false)?;
                let branch = value_expr(ns, branch)?;
                ns.values.end_local_scopes(predicate.introduced_names());
                new_predicates.push(predicate);
                new_branches.push(branch);
            }
            ir::Match {
                scrutinee,
                predicates: new_predicates,
                branches: new_branches,
            }
        }
        Tuple(expressions) => ir::Tuple(
            expressions
                .into_iter()
                .map(|e| value_expr(ns, e))
                .try_collect()?,
        ),
        StructureLiteral { lhs, rhs } => ir::Struct {
            field_names: lhs,
            field_values: rhs.into_iter().map(|e| value_expr(ns, e)).try_collect()?,
        },
        Field { lhs, rhs } => ir::Field {
            of: rec!(lhs),
            index: rhs,
        },
        ModuleField(items) => {
            let path = Path::from(items);
            let t = ns.get_import_type(&path).span(span)?;
            ir::ImportedSymbol(path, t)
        }
    }
    .with_span(span)
    .with_type(Type::Any))
}

fn pattern_expr(
    ns: &mut ModuleNameSpace,
    pattern: PatternExpression,
    global: bool,
) -> Result<Pattern> {
    use PatternExpressionKind::*;
    let span = pattern.span;
    Ok(match pattern.inner {
        Literal(literal) => PatternKind::Literal(lit(literal).span(span)?),
        Identifier(id) => PatternKind::Name(if global {
            ns.define_global_value(&id)?
        } else {
            ns.define_local_value(&id, false)
        }),
        Tuple(expressions) => PatternKind::Tuple(
            expressions
                .into_iter()
                .map(|e| pattern_expr(ns, e, global))
                .try_collect()?,
        ),
        Constructor(items, expression) => {
            let cons = if items.len() == 1 {
                ns.constructors.get(&items[0])
            } else {
                ns.constructors.get_exact(&Path::from(items))
            }?;
            PatternKind::Constructor(cons, Box::new(pattern_expr(ns, *expression, global)?))
        }
    }
    .with_span(span)
    .with_type(Type::Any))
}
