use super::*;
#[derive(Debug, Clone)]
pub enum PatternExpressionKind {
    Literal(super::Literal),
    Identifier(String),
    Tuple(Vec<PatternExpression>),
}

pub type PatternExpression = Expression<PatternExpressionKind>;
use PatternExpressionKind as e;

pub fn parse_pattern(iter: it!()) -> Result<PatternExpression> {
    iter.start_span();
    let Some(Token(next, _)) = iter.next() else {
        return Err(iter.report_error(ExpectedExpression, []));
    };
    let kind = match next {
        // Tuple or unit
        LeftParen => {
            let mut patterns = vec![];
            loop {
                if iter.peek_or_error(0, RightParen).is_ok() {
                    break;
                }
                patterns.push(parse_pattern(iter)?);
                if iter.eat(Comma).is_none() {
                    break;
                }
            }
            iter.eat_or_error(RightParen);
            if patterns.len() == 0 {
                e::Literal(super::Literal::Unit)
            } else {
                e::Tuple(patterns)
            }
        }
        // Identifier
        Identifier(name) => e::Identifier(name),
        // Literals
        IntegerLiteral(i, base) => e::Literal(Literal::Integer(i, base)),
        RealLiteral(r) => e::Literal(Literal::Real(r)),
        True => e::Literal(Literal::Boolean(true)),
        False => e::Literal(Literal::Boolean(false)),
        StringLiteral(s) => e::Literal(Literal::String(s)),
        GlyphLiteral(g) => e::Literal(Literal::Glyph(g)),
        _ => return Err(iter.report_error(ParseLint::ExpectedExpression, [])),
    };
    Ok(PatternExpression {
        kind,
        span: iter.end_span(),
    })
}
