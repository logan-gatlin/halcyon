use super::*;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternExpressionKind {
    Literal(super::Literal),
    Identifier(String),
    ModulePath(String, String),
    Tuple(Vec<PatternExpression>),
    Array(Vec<ParsedArrayPattern>),
    Constructor((String, Option<String>), Box<PatternExpression>),
    TypeHint(Box<PatternExpression>, Box<TypeExpression>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedArrayPattern {
    Pattern(PatternExpression),
    ExpansionAssign(Spanned<String>),
    Expansion(Span),
}

pub type PatternExpression = Expression<PatternExpressionKind>;

impl<'a, I: Iterator<Item = Token>> Parser<'a, I> {
    fn pattern_primary(&mut self) -> Result<PatternExpression> {
        use PatternExpressionKind as e;
        let next = self.next_token_or_err().ok_or(UntilNextStatement)?;
        let span = next.span;
        Ok(match next.inner {
            StringLiteral(s) => e::Literal(Literal::String(s)),
            GlyphLiteral(g) => e::Literal(Literal::Glyph(g)),
            IntegerLiteral(i, b) => e::Literal(Literal::Integer(i, b)),
            RealLiteral(r) => e::Literal(Literal::Real(r)),
            True => e::Literal(Literal::Boolean(true)),
            False => e::Literal(Literal::Boolean(false)),
            Identifier(name) => {
                let name1 = name;
                let name2 = if self.eat(&DoubleColon).is_some()
                    && let Some(name2) = self.eat_ident_or_err()
                {
                    Some(name2.inner)
                } else {
                    None
                };
                // Constructor
                if self.eat(&Of).is_some() {
                    let inner = self.parse_pattern()?;
                    e::Constructor((name1, name2), inner.into())
                }
                // Path
                else if let Some(name2) = name2 {
                    e::ModulePath(name1, name2)
                } else {
                    e::Identifier(name1)
                }
            }
            LeftParen => {
                let mut inner = vec![];
                let mut is_tuple = false;
                loop {
                    if self.eat(&RightParen).is_some() {
                        break;
                    }
                    inner.push(self.parse_pattern()?);
                    if self.eat(&Comma).is_some() {
                        is_tuple = true;
                    } else {
                        self.eat_or_err(&RightParen)
                            .ok_or(UntilCategory(TokenCategory::EndGrouping))?;
                        break;
                    }
                }
                if is_tuple {
                    e::Tuple(inner)
                } else if !inner.is_empty() {
                    return Ok(inner[0].clone());
                } else {
                    e::Literal(Literal::Unit)
                }
            }
            LeftSquare => {
                let mut patterns = vec![];
                loop {
                    if self.eat(&RightSquare).is_some() {
                        break;
                    }
                    if self.peek().is_some_and(|t| t.inner == DotDot) {
                        self.skip();
                        if let Some(ident) = self.eat_ident() {
                            patterns.push(ParsedArrayPattern::ExpansionAssign(ident));
                        } else {
                            patterns.push(ParsedArrayPattern::Expansion(self.last_span));
                        }
                    } else {
                        patterns.push(ParsedArrayPattern::Pattern(self.parse_pattern()?));
                    }
                    if self.eat(&Comma).is_none() {
                        self.eat_or_err(&RightSquare)
                            .ok_or(UntilCategory(TokenCategory::EndGrouping))?;
                        break;
                    }
                }
                e::Array(patterns)
            }
            _ => {
                self.error()
                    .primary("Expected a pattern here.", span)
                    .done();
                return Err(UntilNextStatement);
            }
        }
        .with_span(span + self.last_span))
    }

    pub fn parse_pattern(&mut self) -> Result<PatternExpression> {
        use PatternExpressionKind as e;
        let primary = self.pattern_primary()?;
        let span = primary.span;
        if self.eat(&Colon).is_some() {
            let type_ = self.parse_type_expression(0)?;
            Ok(e::TypeHint(primary.into(), type_.into()).with_span(span + self.last_span))
        } else {
            Ok(primary)
        }
    }
}
