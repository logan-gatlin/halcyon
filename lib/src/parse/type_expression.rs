use super::*;

pub type TypeDefinition = Expression<TypeDefinitionKind>;
pub type TypeExpression = Expression<TypeExpressionKind>;

#[derive(Debug, Clone, sx::SXRepr)]
pub enum TypeDefinitionKind {
    TypeFunction {
        arguments: Vec<String>,
        body: Box<TypeDefinition>,
    },
    Structure {
        lhs: Vec<String>,
        rhs: Vec<TypeExpression>,
    },
    Sum {
        variant_names: Vec<String>,
        variant_types: Vec<Option<TypeExpression>>,
    },
    Expression(TypeExpression),
}

pub fn parse_type_definition(logger: &mut Logger, p: p!()) -> LResult<TypeDefinition> {
    let next = p.peek()?;
    let span = next.span;
    Ok(match next.inner {
        Fn => {
            p.skip();
            let mut arguments = vec![];
            loop {
                if p.eat(FatArrow).is_ok() {
                    break;
                }
                arguments.push(p.eat_ident()?);
            }
            let body = Box::new(parse_type_definition(logger, p)?);
            TypeDefinitionKind::TypeFunction { arguments, body }
        }
        LeftBrace => {
            p.skip();
            let mut lhs = vec![];
            let mut rhs = vec![];
            loop {
                if let Ok(ident) = p.eat_ident() {
                    lhs.push(ident);
                    p.eat(Colon)?;
                    rhs.push(parse_type_expression(logger, p, 0)?);
                    if p.eat(Comma).is_err() {
                        p.eat(RightBrace)?;
                        break;
                    }
                } else {
                    p.eat(RightBrace)?;
                    break;
                }
            }
            TypeDefinitionKind::Structure { lhs, rhs }
        }
        Pipe => {
            p.skip();
            let mut variant_names = vec![];
            let mut variant_types = vec![];
            loop {
                if p.eat(Pipe).is_err() {
                    break;
                }
                variant_names.push(p.eat_ident()?);
                variant_types.push(if p.eat(Of).is_ok() {
                    Some(parse_type_expression(logger, p, 0)?)
                } else {
                    None
                });
            }
            TypeDefinitionKind::Sum {
                variant_names,
                variant_types,
            }
        }
        Identifier(name)
            if p.peek_nth(1)
                .is_ok_and(|t| t.inner == Of || t.inner == Pipe) =>
        {
            p.skip();
            let mut variant_names = vec![name];
            let mut variant_types = vec![if p.eat(Of).is_ok() {
                Some(parse_type_expression(logger, p, 0)?)
            } else {
                None
            }];
            loop {
                if p.eat(Pipe).is_err() {
                    break;
                }
                variant_names.push(p.eat_ident()?);
                variant_types.push(if p.eat(Of).is_ok() {
                    Some(parse_type_expression(logger, p, 0)?)
                } else {
                    None
                });
            }
            TypeDefinitionKind::Sum {
                variant_names,
                variant_types,
            }
        }
        _ => TypeDefinitionKind::Expression(parse_type_expression(logger, p, 0)?),
    }
    .with_span(span + p.last_span))
}

#[derive(Debug, Clone, sx::SXRepr)]
pub enum TypeExpressionKind {
    Function(Box<TypeExpression>, Box<TypeExpression>),
    Call(Box<TypeExpression>, Box<TypeExpression>),
    Identifier(String),
    Product(Vec<TypeExpression>),
    ModulePath(Vec<String>),
    Array(Box<TypeExpression>),
    Unit,
}

fn primary(logger: &mut Logger, p: p!()) -> LResult<TypeExpression> {
    use TypeExpressionKind as e;
    let next = p.next()?;
    let mut span = next.span;
    Ok(match next.inner {
        // Identifier or path
        Identifier(name) => {
            // Path
            if p.eat(DoubleColon).is_ok() {
                let mut path = vec![name];
                loop {
                    let s = p.eat_ident()?;
                    path.push(s);
                    if p.eat(DoubleColon).is_err() {
                        break;
                    } else {
                        span += p.last_span;
                    }
                }
                e::ModulePath(path)
            // Identifier
            } else {
                e::Identifier(name)
            }
        }
        LeftParen => {
            let mut items = vec![];
            let mut is_tuple = false;
            loop {
                if p.eat(RightParen).is_ok() {
                    break;
                }
                items.push(parse_type_expression(logger, p, 0)?);
                if p.eat(Comma).is_ok() {
                    is_tuple = true;
                } else {
                    p.eat(RightParen)?;
                    break;
                }
            }
            if is_tuple {
                e::Product(items)
            } else if items.is_empty() {
                e::Unit
            } else {
                span += p.last_span;
                let mut inner = items[0].clone();
                inner.span = span;
                return Ok(inner);
            }
        }
        LeftSquare => {
            let inner = parse_type_expression(logger, p, 0)?;
            p.eat(RightSquare)?;
            TypeExpressionKind::Array(inner.into())
        }
        _ => return Err(err("Expected type expression here").span(span)),
    }
    .with_span(span + p.last_span))
}

pub fn parse_type_expression(
    logger: &mut Logger,
    p: p!(),
    precedence: Precedence,
) -> LResult<TypeExpression> {
    let mut current = primary(logger, p)?;
    let span = current.span;
    while let Ok(next) = p.peek() {
        // Binary op
        if let Ok(op) = BinaryTypeOp::try_from(&*next) {
            let new_precedence = op.precedence();
            // End precedence climb
            if ((op.assoc() == LEFT_ASSOC) && new_precedence <= precedence)
                || (new_precedence < precedence)
            {
                return Ok(current);
            }
            p.skip();
            current = TypeExpression {
                inner: match op {
                    BinaryTypeOp::Arrow => TypeExpressionKind::Function(
                        Box::new(current),
                        Box::new(parse_type_expression(logger, p, new_precedence)?),
                    ),
                },
                span: span + p.last_span,
            };
        }
        // Function application
        else if precedence < CALL_PREC
            && let Ok(t) = p.peek()
            && (t.inner.is_literal() || matches!(t.inner, Identifier(_)))
        {
            current = TypeExpressionKind::Call(
                Box::new(current),
                Box::new(parse_type_expression(logger, p, CALL_PREC)?),
            )
            .with_span(span + p.last_span);
        } else {
            break;
        }
    }
    Ok(current)
}
