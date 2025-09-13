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
        tail: Vec<PatternExpression>,
    },
}

pub type PatternExpression = Expression<PatternExpressionKind>;
use PatternExpressionKind as e;

pub fn parse_pattern(iter: it!()) -> Result<PatternExpression> {
    iter.start_span();
    let Some(next) = iter.next().map(without_span) else {
        return Err(iter.report_error(ExpectedExpression, []));
    };
    let mut current = match next {
        // Tuple or unit
        LeftParen => {
            let mut is_tuple = false;
            let mut patterns = vec![];
            loop {
                if iter.peek_or_error(0, RightParen).is_ok() {
                    break;
                }
                patterns.push(parse_pattern(iter)?);
                if iter.eat(Comma).is_none() {
                    break;
                }
                is_tuple = true;
            }
            iter.eat_or_error(RightParen)?;
            if patterns.is_empty() {
                e::Literal(super::Literal::Unit)
            } else if is_tuple {
                e::Tuple(patterns)
            } else {
                patterns[0].inner.clone()
            }
        }
        // Array
        /*
        LeftSquare => {
            let mut pattern = ParsedArrayPattern::Exact(vec![]);
            loop {
                use ParsedArrayPattern::*;
                if iter.peek_or_error(0, RightSquare).is_ok() {
                    break;
                }
                let next_pat = parse_pattern(iter)?;
                let is_splat = iter.eat(DotDot).is_some();
                if iter.eat(Comma).is_none() {
                    break;
                }
            }
            iter.eat_or_error(RightSquare)?;
            e::Array(pattern.into())
        }
        */
        // Identifier, path, or constructor
        Identifier(name) => {
            let mut path = vec![name];
            while iter.eat(DoubleColon).is_some() {
                path.push(iter.eat_ident()?);
            }
            match iter.eat_or_error(Of) {
                Ok(_) => e::Constructor(path, Box::new(parse_pattern(iter)?)),
                Err(_) if path.len() == 1 => e::Identifier(path[0].clone()),
                Err(_) => e::ModulePath(path),
            }
        }
        // Literals
        IntegerLiteral(i, base) => e::Literal(Literal::Integer(i, base)),
        RealLiteral(r) => e::Literal(Literal::Real(r)),
        Minus if let Some(IntegerLiteral(num, base)) = iter.peek(0).map(|t| t.inner) => {
            iter.skip(1);
            e::Literal(Literal::Integer(format!("-{num}"), base))
        }
        Minus if let Some(RealLiteral(num)) = iter.peek(0).map(|t| t.inner) => {
            iter.skip(1);
            e::Literal(Literal::Real(format!("-{num}")))
        }
        True => e::Literal(Literal::Boolean(true)),
        False => e::Literal(Literal::Boolean(false)),
        StringLiteral(s) => e::Literal(Literal::String(s)),
        GlyphLiteral(g) => e::Literal(Literal::Glyph(g)),
        _ => return Err(iter.report_error(ParseLint::ExpectedExpression, [])),
    }
    .with_span(iter.end_span());
    if iter.eat(Colon).is_some() {
        let span = current.span;
        let type_ = parse_type_expression(iter, 0)?;
        current = e::TypeHint(current.into(), type_.into()).with_span(span + iter.last_span);
    }
    Ok(current)
}
