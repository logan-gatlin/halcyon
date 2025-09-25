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

pub fn parse_type_definition(logger: &mut Logger, p: it!()) -> PResult<TypeDefinition> {
    todo!()
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

fn primary(logger: &mut Logger, p: it!()) -> PResult<TypeExpression> {
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
        _ => todo!(),
    }
    .with_span(span + p.last_span))
}

fn parse_type_expression(
    logger: &mut Logger,
    p: it!(),
    precedence: Precedence,
) -> PResult<TypeExpression> {
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
