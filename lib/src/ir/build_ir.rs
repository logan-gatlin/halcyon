use super::*;
use crate::{lint::*, parse::*};

pub fn build_ir(
    module: ParsedModule,
    context: &HashMap<Path, ModuleInterface>,
) -> Result<IrModule> {
    let module_name = Path::from((*module.name).clone());
    let mut ns = ModuleNameSpace::new(module_name.clone());
    for interface in context.values() {
        ns.import_interface(interface).span(module.span)?;
    }
    let mut items = vec![];
    for item in module.contents.clone() {
        module_expr(&mut ns, item, &mut items)?;
    }
    Ok(IrModule { module_name, items })
}

fn module_expr(
    ns: &mut ModuleNameSpace,
    e: ModuleExpression,
    items: &mut Vec<ModuleItem>,
) -> Result<()> {
    match e.inner {
        ModuleExpressionKind::Let { assignee, value } => {
            let mut assignee = pattern_expr(ns, assignee, true)?;
            let value = value_expr(ns, *value)?;
            assignee.visit(|(p, _)| ns.finalize_value(p));
            items.push(ModuleItem::Let(assignee, Box::new(value)));
        }
        ModuleExpressionKind::Type {
            assignee,
            assignee_span,
            value,
        } => {
            type_def(ns, assignee, assignee_span, *value, items, 0)?;
        }
        ModuleExpressionKind::Import {
            name,
            type_,
            major,
            minor,
        } => {
            let type_ = ForeignFunctionType::try_from(type_expr(ns, *type_)?).span(e.span)?;
            let path = ns.new_global_value(&name).span(e.span)?;
            items.push(ModuleItem::Import {
                path,
                type_,
                major,
                minor,
            });
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
                let parameter_name =
                    Some(ns.new_local_value(&argument, true).with_span(argument.span));
                let body = curry(ns, arguments, body, span)?;
                let captures = ns.end_capture();
                ns.end_value_scopes(1);
                IrKind::Function {
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
            let mut assignee = pattern_expr(ns, assignee, false)?;
            if !assignee.is_irrefutable() {
                return Err(lint(TypeLint::NonExhaustive, span, []));
            }
            let value = rec!(value);
            assignee.visit(|(p, _)| {
                ns.finalize_value(p);
            });
            let in_ = rec!(in_);
            ns.end_value_scopes(assignee.introduced_names());
            ir::Let {
                assignee,
                value,
                in_,
            }
        }
        Literal(literal) => ir::Immediate(lit(literal).span(span)?),
        Identifier(ident) => ir::Identifier(ns.get_value(&ident).span(span)?),
        BinaryOp(op) => ir::ImportedSymbol(op.path(), op.get_type()),
        Binary {
            op: crate::operator::BinaryOp::Semicolon,
            left,
            right,
        } => ir::Semicolon(rec!(left), rec!(right)),
        Binary { op, left, right } => {
            let left_span = left.span;
            ir::Call {
                callee: ir::Call {
                    callee: ir::ImportedSymbol(op.path(), op.get_type())
                        .with_span(expr.span)
                        .with_type(Type::Any)
                        .into(),
                    argument: rec!(left),
                    opt: Default::default(),
                }
                .with_span(left_span)
                .with_type(Type::Any)
                .into(),
                argument: rec!(right),
                opt: Default::default(),
            }
        }
        Unary { op, child } => ir::Call {
            callee: ir::ImportedSymbol(op.path(), op.get_type())
                .with_span(expr.span)
                .with_type(Type::Any)
                .into(),
            argument: rec!(child),
            opt: Default::default(),
        },
        FunctionShorthand {
            parameters,
            types,
            predicates,
            branches,
        } => {
            const SHORTHAND_NAME: &str = "~";
            let body = FunctionDef {
                parameters: vec![SHORTHAND_NAME.to_string().with_span(expr.span)],
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
            .with_span(expr.span)
            .into();
            return Ok(*curry(ns, parameters.into_iter().zip(types), body, span)?);
        }
        FunctionDef {
            parameters: arguments,
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
                return Ok(*curry(ns, arguments.into_iter().zip(types), body, span)?);
            }
        }
        FunctionCall { callee, argument } => ir::Call {
            callee: rec!(callee),
            argument: rec!(argument),
            opt: Default::default(),
        },
        If {
            predicate,
            then,
            else_,
        } => ir::If {
            predicate: rec!(predicate),
            then: rec!(then),
            else_: rec!(else_),
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
                let mut predicate = pattern_expr(ns, predicate, false)?;
                predicate.visit(|(p, _)| ns.finalize_value(p));
                let branch = value_expr(ns, branch)?;
                ns.end_value_scopes(predicate.introduced_names());
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
            let t = ns.get_imported_value_type(&path).span(span)?;
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
        ModulePath(path) => {
            let path = Path::from(path);
            let cons = ns.get_constructor_exact(&path).span(span)?;
            if !matches!(cons.kind, ConstructorKind::Unitary(_)) {
                return Err(lint(
                    NameLint::MissingConstructorBinding,
                    span,
                    [format!("{path}")],
                ));
            }
            PatternKind::Constructor(
                cons,
                PatternKind::Literal(ConstValue::Unit)
                    .with_span(span)
                    .with_type(Type::Any)
                    .into(),
            )
        }
        Identifier(id) if id == "_" => PatternKind::Hole,
        Identifier(id) => match ns.get_constructor(&id) {
            // Unitary constructor
            Ok(
                cons @ crate::Constructor {
                    kind: ConstructorKind::Unitary(_),
                    ..
                },
            ) => PatternKind::Constructor(
                cons,
                PatternKind::Literal(ConstValue::Unit)
                    .with_span(span)
                    .with_type(Type::Any)
                    .into(),
            ),
            // Non-unitary constructor
            Ok(_) => return Err(lint(NameLint::MissingConstructorBinding, span, [id])),
            // Regular identifier
            _ => PatternKind::Name(if global {
                ns.new_global_value(&id).span(span)?
            } else {
                ns.new_local_value(&id, false)
            }),
        },
        Tuple(expressions) => PatternKind::Tuple(
            expressions
                .into_iter()
                .map(|e| pattern_expr(ns, e, global))
                .try_collect()?,
        ),
        Constructor(items, expression) => {
            let cons = if items.len() == 1 {
                ns.get_constructor(&items[0])
            } else {
                ns.get_constructor_exact(&Path::from(items))
            }
            .span(span)?;
            PatternKind::Constructor(cons, Box::new(pattern_expr(ns, *expression, global)?))
        }
        TypeHint(pat, type_) => {
            let pat = pattern_expr(ns, *pat, global)?;
            let type_ = type_expr(ns, *type_)?;
            PatternKind::TypeHint(pat.into(), type_)
        }
    }
    .with_span(span)
    .with_type(Type::Any))
}
