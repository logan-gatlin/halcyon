use indexmap::IndexMap;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Unit,
    Integer(String, Base),
    Real(String),
    String(String),
    Glyph(char),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueExpressionKind {
    Let {
        assignee: PatternExpression,
        is_global: bool,
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
    StructureLiteral(IndexMap<Spanned<String>, ValueExpression>),
    Field {
        lhs: Box<ValueExpression>,
        rhs: Spanned<String>,
    },
    ModulePath(String, String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayInner {
    Splat(ValueExpression),
    Single(ValueExpression),
}

pub type ValueExpression = Expression<ValueExpressionKind>;

impl<'a, I: Iterator<Item = Token>> Parser<'a, I> {
    fn primary(&mut self) -> Result<ValueExpression> {
        use ValueExpressionKind as e;
        let next = self.next_token_or_err().ok_or(NoRecovery)?;
        let mut span = next.span;
        Ok(match next.inner {
            LeftParen if self.eat(&RightParen).is_some() => e::Literal(Literal::Unit),
            IntegerLiteral(i, base) => e::Literal(Literal::Integer(i, base)),
            RealLiteral(r) => e::Literal(Literal::Real(r)),
            StringLiteral(s) => e::Literal(Literal::String(s)),
            GlyphLiteral(g) => e::Literal(Literal::Glyph(g)),
            True => e::Literal(Literal::Boolean(true)),
            False => e::Literal(Literal::Boolean(false)),
            // Identifier or path
            Identifier(name1) => {
                // Path
                if self.eat(&DoubleColon).is_some()
                    && let Some(name2) = self.eat_ident_or_err()
                {
                    e::ModulePath(name1, name2.inner)
                // Identifier
                } else {
                    e::Identifier(name1)
                }
            }
            Fn if self.eat(&Pipe).is_some() => {
                let mut predicates = vec![];
                let mut branches = vec![];
                loop {
                    predicates.push(self.parse_pattern()?);
                    self.eat_or_err(&FatArrow)
                        .or_else(|| self.eat(&Equal))
                        .ok_or(UntilNextStatement)?;
                    branches.push(self.parse_value_expression(0)?);
                    if self.eat(&Pipe).is_none() {
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
                    if self.eat(&LeftParen).is_some() {
                        let ident = self.eat_ident_or_err().ok_or(UntilNextStatement)?;
                        parameters.push(ident);
                        if self.eat_or_err(&Colon).is_some() {
                            let type_ = self.parse_type_expression(0)?;
                            types.push(Some(type_));
                        }
                        self.eat_or_err(&RightParen).ok_or(UntilNextStatement)?;
                    } else if let Some(ident) = self.eat_ident() {
                        parameters.push(ident);
                        types.push(None);
                    }
                    // End of parameters
                    else {
                        self.eat_or_err(&FatArrow)
                            .or_else(|| self.eat(&Equal))
                            .ok_or(UntilNextStatement)?;
                        break;
                    }
                }
                let body = self.parse_value_expression(0)?.into();
                e::FunctionDef {
                    parameters,
                    types,
                    body,
                }
            }
            Let => {
                e::Let {
                    assignee: self.parse_pattern()?,
                    is_global: false,
                    value: {
                        self.eat_or_err(&Equal).ok_or(UntilNextStatement)?;
                        Box::new(self.parse_value_expression(0)?)
                    },
                    in_: {
                        self.eat_or_err(&In).ok_or(UntilNextStatement)?;
                        Box::new(self.parse_value_expression(0)?)
                    },
                }
            }
            LeftBrace => {
                let mut span_map = std::collections::HashMap::new();
                let mut value_map = indexmap::IndexMap::new();
                loop {
                    if self.eat(&RightBrace).is_some() {
                        break;
                    }
                    let name = self
                        .eat_ident_or_err()
                        .ok_or(UntilCategory(TokenCategory::EndGrouping))?;
                    if self
                        .eat_or_err(&Equal)
                        .or_else(|| self.eat(&Colon))
                        .is_some()
                    {
                        let value = self.parse_value_expression(0)?;
                        if let Some(old_span) = span_map.insert(name.inner.clone(), name.span) {
                            self.error()
                                .primary("This key is used more than once", old_span)
                                .secondary("Second use is here", name.span)
                                .note("Keys in a structure must be unique")
                                .done();
                        } else {
                            value_map.insert(name, value);
                        }
                    }
                    if self.eat(&Comma).is_none() {
                        self.eat_or_err(&RightBrace)
                            .ok_or(UntilCategory(TokenCategory::EndGrouping))?;
                        break;
                    }
                }
                e::StructureLiteral(value_map)
            }
            LeftSquare => {
                let mut items = vec![];
                loop {
                    if self.eat(&RightSquare).is_some() {
                        break;
                    } else if self.eat(&DotDot).is_some() {
                        items.push(ArrayInner::Splat(self.parse_value_expression(0)?));
                    } else {
                        items.push(ArrayInner::Single(self.parse_value_expression(0)?));
                    }
                    if self.eat(&Comma).is_none() {
                        self.eat_or_err(&RightSquare)
                            .ok_or(UntilCategory(TokenCategory::EndGrouping))?;
                        break;
                    }
                }
                e::Array(items)
            }
            If => {
                e::If {
                    predicate: self.parse_value_expression(0)?.into(),
                    then: {
                        let _ = self.eat_or_err(&Then);
                        self.parse_value_expression(0)?.into()
                    },
                    else_: {
                        let _ = self.eat(&Else);
                        self.parse_value_expression(0)?.into()
                    },
                }
            }
            Match => {
                let scrutinee = self.parse_value_expression(0)?.into();
                self.eat(&With).ok_or(UntilNextStatement)?;
                let _ = self.eat(&Pipe);
                let mut predicates = vec![];
                let mut branches = vec![];
                loop {
                    predicates.push(self.parse_pattern()?);
                    self.eat_or_err(&FatArrow)
                        .or_else(|| self.eat(&Equal))
                        .ok_or(UntilNextStatement)?;
                    branches.push(self.parse_value_expression(0)?);
                    if self.eat(&Pipe).is_none() {
                        break;
                    }
                }
                e::Match {
                    scrutinee,
                    predicates,
                    branches,
                }
            }
            LeftParen => {
                // Binary op
                if let Some(Ok(op)) = self.peek().map(|o| BinaryOp::try_from(&o.inner))
                    && self.peek_nth(1).is_some_and(|t| *t == RightParen)
                {
                    self.skip();
                    self.skip();
                    e::BinaryOp(op)
                }
                // Unary op
                else if self.peek().is_some_and(|t| t.inner == Not)
                    && self.peek_nth(1).is_some_and(|t| *t == RightParen)
                {
                    self.skip();
                    self.skip();
                    e::UnaryOp(UnaryOp::Not)
                }
                // Tuple or parenthesis
                else {
                    let mut inner = vec![];
                    let mut is_tuple = false;
                    loop {
                        if self.eat(&RightParen).is_some() {
                            break;
                        }
                        inner.push(self.parse_value_expression(0)?);
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
                    } else {
                        span += self.last_span;
                        let mut inner = inner[0].clone();
                        inner.span = span;
                        return Ok(inner);
                    }
                }
            }
            _ => {
                self.error()
                    .primary("Expected an expression here.", span)
                    .done();
                return Err(UntilNextStatement);
            }
        }
        .with_span(span + self.last_span))
    }

    pub fn parse_value_expression(
        &mut self,
        precedence: Precedence,
    ) -> Result<ValueExpression> {
        use ValueExpressionKind as e;
        let unary_ops = [Minus, MinusDot, Not];
        let span;
        let mut current = if let Some(id) = self.eat_one_of(unary_ops.clone()) {
            span = self.last_span;
            let op = UnaryOp::try_from(&unary_ops[id]).unwrap_or_else(|_| unreachable!());
            let operand = self.parse_value_expression(op.precedence())?;
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
            let p = self.primary()?;
            span = p.span;
            p
        };
        // Precedence climbing loop
        while let Some(next) = self.peek() {
            // Binary operator
            if let Ok(op) = BinaryOp::try_from(&*next) {
                let new_precedence = op.precedence();
                // End precedence climb
                if ((op.assoc() == LEFT_ASSOC) && new_precedence <= precedence)
                    || (new_precedence < precedence)
                {
                    return Ok(current);
                }
                self.skip();
                current = ValueExpression {
                    inner: e::Binary {
                        op,
                        left: current.into(),
                        right: Box::new(self.parse_value_expression(new_precedence)?),
                    },
                    span: span + self.last_span,
                }
            }
            // Field
            else if precedence < FIELD_PREC && self.eat(&Dot).is_some() {
                let rhs = self.eat_ident_or_err().ok_or(UntilNextStatement)?;
                current = ValueExpression {
                    inner: e::Field {
                        lhs: current.into(),
                        rhs,
                    },
                    span: span + self.last_span,
                }
            }
            // Function call
            else if precedence < CALL_PREC
                && let Some(next) = self.peek()
                && (next.inner.is_literal()
                    || next.inner == LeftParen
                    || next.inner == LeftSquare
                    || next.inner == LeftBrace
                    || matches!(next.inner, Identifier(_)))
            {
                current = ValueExpression {
                    inner: e::FunctionCall {
                        callee: current.into(),
                        argument: Box::new(self.parse_value_expression(CALL_PREC)?),
                    },
                    span: span + self.last_span,
                }
            } else {
                break;
            }
        }
        Ok(current)
    }
}
