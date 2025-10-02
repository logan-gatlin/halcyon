use super::*;

#[derive(Debug, Clone, sx::SXRepr)]
pub enum Literal {
    Unit,
    Integer(String, Base),
    Real(String),
    String(String),
    Glyph(char),
    Boolean(bool),
}

#[derive(Debug, Clone, sx::SXRepr)]
pub enum ValueExpressionKind {
    Let {
        assignee: PatternExpression,
        value: Box<ValueExpression>,
        in_: Box<ValueExpression>,
    },
    Literal(Literal),
    Identifier(String),
    Binary {
        op: BinaryOp,
        left: Box<ValueExpression>,
        right: Box<ValueExpression>,
    },
    BinaryOp(BinaryOp),
    Unary {
        op: UnaryOp,
        child: Box<ValueExpression>,
    },
    UnaryOp(UnaryOp),
    FunctionDef {
        parameters: Vec<Spanned<String>>,
        types: Vec<Option<TypeExpression>>,
        body: Box<ValueExpression>,
    },
    /// fn | ...
    FunctionShorthand {
        predicates: Vec<PatternExpression>,
        branches: Vec<ValueExpression>,
    },
    FunctionCall {
        callee: Box<ValueExpression>,
        argument: Box<ValueExpression>,
    },
    If {
        predicate: Box<ValueExpression>,
        then: Box<ValueExpression>,
        else_: Box<ValueExpression>,
    },
    Match {
        scrutinee: Box<ValueExpression>,
        predicates: Vec<PatternExpression>,
        branches: Vec<ValueExpression>,
    },
    Tuple(Vec<ValueExpression>),
    Array(Vec<ArrayInner>),
    StructureLiteral {
        lhs: Vec<String>,
        rhs: Vec<ValueExpression>,
    },
    Field {
        lhs: Box<ValueExpression>,
        rhs: String,
    },
    ModulePath(Vec<String>),
}

#[derive(Debug, Clone, sx::SXRepr)]
pub enum ArrayInner {
    Splat(ValueExpression),
    Single(ValueExpression),
}

pub type ValueExpression = Expression<ValueExpressionKind>;

fn primary(logger: &mut Logger, p: p!()) -> LResult<ValueExpression> {
    use ValueExpressionKind as e;
    let next = p.next()?;
    let mut span = next.span;
    Ok(match next.inner {
        LeftParen if p.eat(RightParen).is_ok() => e::Literal(Literal::Unit),
        IntegerLiteral(i, base) => e::Literal(Literal::Integer(i, base)),
        RealLiteral(r) => e::Literal(Literal::Real(r)),
        StringLiteral(s) => e::Literal(Literal::String(s)),
        GlyphLiteral(g) => e::Literal(Literal::Glyph(g)),
        True => e::Literal(Literal::Boolean(true)),
        False => e::Literal(Literal::Boolean(false)),
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
        Fn if p.eat(Pipe).is_ok() => {
            let mut predicates = vec![];
            let mut branches = vec![];
            loop {
                predicates.push(parse_pattern(logger, p)?);
                p.eat(FatArrow)?;
                branches.push(parse_value_expression(logger, p, 0)?);
                if p.eat(Pipe).is_err() {
                    break;
                }
            }
            e::FunctionShorthand {
                predicates,
                branches,
            }
        }
        Fn => {
            let mut parameters = vec![];
            let mut types = vec![];
            loop {
                // Typed parameter
                if p.eat(LeftParen).is_ok() {
                    let ident = p.eat_ident()?;
                    parameters.push(ident.with_span(p.last_span));
                    p.eat(Colon)?;
                    // parse type
                    let type_ = parse_type_expression(logger, p, 0)?;
                    types.push(Some(type_));
                    p.eat(RightParen)?;
                } else if let Ok(ident) = p.eat_ident() {
                    parameters.push(ident.with_span(p.last_span));
                    types.push(None);
                }
                // End of parameters
                else {
                    p.eat(FatArrow)?;
                    break;
                }
            }
            let body = parse_value_expression(logger, p, 0)?.into();
            e::FunctionDef {
                parameters,
                types,
                body,
            }
        }
        Let => e::Let {
            assignee: parse_pattern(logger, p)?,
            value: {
                p.eat(Equal)?;
                Box::new(parse_value_expression(logger, p, 0)?)
            },
            in_: {
                p.eat(In)?;
                Box::new(parse_value_expression(logger, p, 0)?)
            },
        },
        LeftBrace => {
            let mut rhs = vec![];
            let mut lhs = vec![];
            loop {
                if let Ok(ident) = p.eat_ident() {
                    lhs.push(ident);
                    p.eat(Colon)?;
                    rhs.push(parse_value_expression(logger, p, 0)?);
                    if p.eat(Comma).is_err() {
                        p.eat(RightBrace)?;
                        break;
                    }
                } else {
                    p.eat(RightBrace)?;
                    break;
                }
            }
            e::StructureLiteral { lhs, rhs }
        }
        LeftSquare => {
            let mut items = vec![];
            loop {
                if p.eat(RightSquare).is_ok() {
                    break;
                } else if p.eat(DotDot).is_ok() {
                    items.push(ArrayInner::Splat(parse_value_expression(logger, p, 0)?));
                } else {
                    items.push(ArrayInner::Single(parse_value_expression(logger, p, 0)?));
                }
                if p.eat(Comma).is_err() {
                    p.eat(RightSquare)?;
                    break;
                }
            }
            e::Array(items)
        }
        If => e::If {
            predicate: parse_value_expression(logger, p, 0)?.into(),
            then: {
                p.eat(Then)?;
                parse_value_expression(logger, p, 0)?.into()
            },
            else_: {
                p.eat(Else)?;
                parse_value_expression(logger, p, 0)?.into()
            },
        },
        Match => {
            let scrutinee = parse_value_expression(logger, p, 0)?.into();
            p.eat(With)?;
            let _ = p.eat(Pipe); // Intentionally ignore error
            let mut predicates = vec![];
            let mut branches = vec![];
            loop {
                predicates.push(parse_pattern(logger, p)?);
                p.eat(FatArrow)?;
                branches.push(parse_value_expression(logger, p, 0)?);
                if p.eat(Pipe).is_err() {
                    break;
                }
            }
            e::Match {
                scrutinee,
                predicates,
                branches,
            }
        }
        LeftParen
            if let Ok(Ok(op)) = p.peek().map(|o| BinaryOp::try_from(&o.inner))
                && p.peek_nth(1).is_ok_and(|t| *t == RightParen) =>
        {
            p.skip();
            p.skip();
            e::BinaryOp(op)
        }
        LeftParen
            if p.peek().is_ok_and(|t| t.inner == Not)
                && p.peek_nth(1).is_ok_and(|t| *t == RightParen) =>
        {
            p.skip();
            p.skip();
            e::UnaryOp(UnaryOp::Not)
        }
        LeftParen => {
            let mut inner = vec![];
            let mut is_tuple = false;
            loop {
                if p.eat(RightParen).is_ok() {
                    break;
                }
                inner.push(parse_value_expression(logger, p, 0)?);
                if p.eat(Comma).is_ok() {
                    is_tuple = true;
                } else {
                    p.eat(RightParen)?;
                    break;
                }
            }
            if is_tuple {
                e::Tuple(inner)
            } else {
                span += p.last_span;
                let mut inner = inner[0].clone();
                inner.span = span;
                return Ok(inner);
            }
        }
        _ => return Err(err("Expected expression here").span(span)),
    }
    .with_span(span + p.last_span))
}

pub fn parse_value_expression(
    logger: &mut Logger,
    iter: p!(),
    precedence: Precedence,
) -> LResult<ValueExpression> {
    use ValueExpressionKind as e;
    let unary_ops = [Minus, MinusDot, Not];
    let span;
    let mut current = if let Ok(id) = iter.eat_one_of(unary_ops.clone()) {
        span = iter.last_span;
        let op = UnaryOp::try_from(&unary_ops[id]).unwrap();
        let operand = parse_value_expression(logger, iter, op.precedence())?;
        let op = if let (UnaryOp::Minus, e::Literal(Literal::Real(_))) = (op, &*operand) {
            UnaryOp::MinusDot
        } else {
            op
        };
        Expression {
            inner: e::Unary {
                op,
                child: operand.into(),
            },
            span,
        }
    } else {
        let p = primary(logger, iter)?;
        span = p.span;
        p
    };
    // Precedence climbing loop
    while let Ok(next) = iter.peek() {
        // Binary operator
        if let Ok(op) = BinaryOp::try_from(&*next) {
            let new_precedence = op.precedence();
            // End precedence climb
            if ((op.assoc() == LEFT_ASSOC) && new_precedence <= precedence)
                || (new_precedence < precedence)
            {
                return Ok(current);
            }
            iter.skip();
            current = ValueExpression {
                inner: e::Binary {
                    op,
                    left: current.into(),
                    right: Box::new(parse_value_expression(logger, iter, new_precedence)?),
                },
                span: span + iter.last_span,
            }
        }
        // Field
        else if precedence < FIELD_PREC && iter.eat(Dot).is_ok() {
            let rhs = iter.eat_ident()?;
            current = ValueExpression {
                inner: e::Field {
                    lhs: current.into(),
                    rhs,
                },
                span: span + iter.last_span,
            }
        }
        // Function call
        else if precedence < CALL_PREC
            && let Ok(next) = iter.peek()
            && (next.inner.is_literal()
                || next.inner == LeftParen
                || next.inner == LeftSquare
                || next.inner == LeftBrace
                || matches!(next.inner, Identifier(_)))
        {
            current = ValueExpression {
                inner: e::FunctionCall {
                    callee: current.into(),
                    argument: Box::new(parse_value_expression(logger, iter, CALL_PREC)?),
                },
                span: span + iter.last_span,
            }
        } else {
            break;
        }
    }
    Ok(current)
}
