use super::*;
#[derive(Debug, Clone, sx::SXRepr)]
pub enum PatternExpressionKind {
    Literal(super::Literal),
    Identifier(String),
    ModulePath(Vec<String>),
    Tuple(Vec<PatternExpression>),
    Array(Box<ParsedArrayPattern>),
    Constructor(Vec<String>, Box<PatternExpression>),
    TypeHint(Box<PatternExpression>, Box<TypeExpression>),
}

#[derive(Debug, Clone, sx::SXRepr)]
pub enum ParsedArrayPattern {
    Exact(Vec<PatternExpression>),
    Leading {
        head: Vec<PatternExpression>,
        tail: Option<String>,
    },
    Trailing {
        head: Option<String>,
        tail: Vec<PatternExpression>,
    },
    LeadingAndTrailing {
        head: Vec<PatternExpression>,
        middle: Option<String>,
        tail: Vec<PatternExpression>,
    },
}

pub type PatternExpression = Expression<PatternExpressionKind>;

fn primary(logger: &mut Logger, p: p!()) -> LResult<PatternExpression> {
    use PatternExpressionKind as e;
    let next = p.next()?;
    let span = next.span;
    Ok(match next.inner {
        StringLiteral(s) => e::Literal(Literal::String(s)),
        GlyphLiteral(g) => e::Literal(Literal::Glyph(g)),
        IntegerLiteral(i, b) => e::Literal(Literal::Integer(i, b)),
        RealLiteral(r) => e::Literal(Literal::Real(r)),
        True => e::Literal(Literal::Boolean(true)),
        False => e::Literal(Literal::Boolean(false)),
        Identifier(name) => {
            let mut path = vec![name];
            if p.eat(DoubleColon).is_ok() {
                loop {
                    let s = p.eat_ident()?;
                    path.push(s);
                    if p.eat(DoubleColon).is_err() {
                        break;
                    }
                }
            }
            // Constructor
            if p.eat(Of).is_ok() {
                let inner = parse_pattern(logger, p)?;
                e::Constructor(path, inner.into())
            }
            // Path
            else if path.len() > 1 {
                e::ModulePath(path)
            } else {
                e::Identifier(path[0].clone())
            }
        }
        LeftParen => {
            let mut inner = vec![];
            let mut is_tuple = false;
            loop {
                if p.eat(RightParen).is_ok() {
                    break;
                }
                inner.push(parse_pattern(logger, p)?);
                if p.eat(Comma).is_ok() {
                    is_tuple = true;
                } else {
                    p.eat(RightParen)?;
                    break;
                }
            }
            if is_tuple {
                e::Tuple(inner)
            } else if inner.len() > 0 {
                return Ok(inner[0].clone());
            } else {
                e::Literal(Literal::Unit)
            }
        }
        LeftSquare => {
            use ParsedArrayPattern as p;
            let mut current = ParsedArrayPattern::Exact(vec![]);
            loop {
                if p.eat(RightSquare).is_ok() {
                    break;
                }
                let is_glob = p.eat(DotDot).is_ok();
                let glob_ident = if is_glob { p.eat_ident().ok() } else { None };
                current = match current {
                    p::Exact(mut head) => {
                        if is_glob && head.is_empty() {
                            p::Trailing {
                                head: glob_ident,
                                tail: vec![],
                            }
                        } else if is_glob {
                            p::Leading {
                                head,
                                tail: glob_ident,
                            }
                        } else {
                            head.push(parse_pattern(logger, p)?);
                            p::Exact(head)
                        }
                    }
                    p::Leading { head, tail } => {
                        if is_glob {
                            panic!()
                        } else {
                            p::LeadingAndTrailing {
                                head,
                                middle: tail,
                                tail: vec![parse_pattern(logger, p)?],
                            }
                        }
                    }
                    p::Trailing { head, mut tail } => {
                        if is_glob {
                            panic!()
                        } else {
                            tail.push(parse_pattern(logger, p)?);
                            p::Trailing { head, tail }
                        }
                    }
                    p::LeadingAndTrailing {
                        head,
                        middle,
                        mut tail,
                    } => {
                        if is_glob {
                            panic!()
                        } else {
                            tail.push(parse_pattern(logger, p)?);
                            p::LeadingAndTrailing { head, middle, tail }
                        }
                    }
                };
                if p.eat(Comma).is_err() {
                    p.eat(RightSquare)?;
                    break;
                }
            }
            e::Array(current.into())
        }
        _ => return Err(err("Expected pattern here").span(span)),
    }
    .with_span(span + p.last_span))
}

pub fn parse_pattern(logger: &mut Logger, p: p!()) -> LResult<PatternExpression> {
    use PatternExpressionKind as e;
    let primary = primary(logger, p)?;
    let span = primary.span;
    Ok(if p.eat(Colon).is_ok() {
        let type_ = parse_type_expression(logger, p, 0)?;
        e::TypeHint(primary.into(), type_.into()).with_span(span + p.last_span)
    } else {
        primary
    })
}
