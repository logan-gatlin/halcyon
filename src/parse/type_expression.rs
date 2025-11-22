use super::*;

pub type TypeDefinition = Expression<TypeDefinitionKind>;
pub type TypeExpression = Expression<TypeExpressionKind>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDefinitionKind {
    TypeFunction {
        arguments: Vec<Spanned<String>>,
        body: Box<TypeDefinition>,
    },
    Structure {
        lhs: Vec<Spanned<String>>,
        rhs: Vec<TypeExpression>,
    },
    Sum {
        variant_names: Vec<Spanned<String>>,
        variant_types: Vec<Option<TypeExpression>>,
    },
    Expression(TypeExpression),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpressionKind {
    Function(Box<TypeExpression>, Box<TypeExpression>),
    Call(Box<TypeExpression>, Box<TypeExpression>),
    Identifier(String),
    Product(Vec<TypeExpression>),
    ModulePath(String, String),
    Array(Box<TypeExpression>),
    Unit,
}

impl<'a, I: Iterator<Item = Token>> Parser<'a, I> {
    fn type_primary(&mut self) -> Result<TypeExpression> {
        use TypeExpressionKind as e;
        let next = self.next_or_err().ok_or(NoRecovery)?;
        let mut span = next.span;
        Ok(match next.inner {
            // Identifier or path
            Identifier(name1) => {
                // Path
                if self.eat(&DoubleColon).is_some() {
                    if let Some(name2) = self.eat_ident_or_err() {
                        e::ModulePath(name1, name2.inner)
                    } else {
                        e::Identifier(name1)
                    }
                // Identifier
                } else {
                    e::Identifier(name1)
                }
            }
            LeftParen => {
                let mut items = vec![];
                let mut is_tuple = false;
                loop {
                    if self.eat(&RightParen).is_some() {
                        break;
                    }
                    items.push(self.parse_type_expression(0)?);
                    if self.eat(&Comma).is_some() {
                        is_tuple = true;
                    } else {
                        self.eat_or_err(&RightParen)
                            .ok_or(UntilCategory(TokenCategory::EndGrouping))?;
                        break;
                    }
                }
                if is_tuple {
                    e::Product(items)
                } else if items.is_empty() {
                    e::Unit
                } else {
                    span += self.last_span;
                    let mut inner = items[0].clone();
                    inner.span = span;
                    return Ok(inner);
                }
            }
            LeftSquare => {
                let inner = self.parse_type_expression(0)?;
                self.eat_or_err(&RightSquare)
                    .ok_or(UntilCategory(TokenCategory::EndGrouping))?;
                TypeExpressionKind::Array(inner.into())
            }
            _ => {
                self.error().primary("Expected a type here", span).done();
                return Err(UntilNextStatement);
            }
        }
        .with_span(span + self.last_span))
    }

    pub fn parse_type_definition(&mut self) -> Result<TypeDefinition> {
        let next = self.peek_or_err().ok_or(NoRecovery)?;
        let span = next.span;
        Ok(match next.inner {
            Fn => {
                self.skip();
                let mut arguments = vec![];
                loop {
                    if self.eat(&FatArrow).is_some() {
                        break;
                    } else if self.eat(&Equal).is_some() {
                        self.error_expected(&FatArrow);
                        break;
                    }
                    arguments.push(self.eat_ident_or_err().ok_or(UntilNextStatement)?);
                }
                let body = Box::new(self.parse_type_definition()?);
                TypeDefinitionKind::TypeFunction { arguments, body }
            }
            LeftBrace => {
                self.skip();
                let mut lhs = vec![];
                let mut rhs = vec![];
                loop {
                    const ERR: RecoveryBehavior = UntilCategory(TokenCategory::EndGrouping);
                    if let Some(ident) = self.eat_ident() {
                        if self.eat_or_err(&Colon).is_some() {
                            lhs.push(ident);
                            rhs.push(self.parse_type_expression(0)?);
                        } else if self.eat(&Comma).is_some() {
                            continue;
                        } else {
                            return Err(ERR);
                        }
                        if self.eat(&Comma).is_none() {
                            self.eat_or_err(&RightBrace).ok_or(ERR)?;
                            break;
                        }
                    } else {
                        self.eat_or_err(&RightBrace).ok_or(ERR)?;
                        break;
                    }
                }
                TypeDefinitionKind::Structure { lhs, rhs }
            }
            Pipe => {
                self.skip();
                let mut variant_names = vec![];
                let mut variant_types = vec![];
                loop {
                    if self.eat(&Pipe).is_none() {
                        break;
                    }
                    variant_names.push(self.eat_ident_or_err().ok_or(UntilNextStatement)?);
                    variant_types.push(if self.eat(&Of).is_some() {
                        Some(self.parse_type_expression(0)?)
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
                if self
                    .peek_nth(1)
                    .is_some_and(|t| t.inner == Of || t.inner == Pipe) =>
            {
                self.skip();
                let mut variant_names = vec![name.with_span(self.last_span)];
                let mut variant_types = vec![if self.eat(&Of).is_some() {
                    Some(self.parse_type_expression(0)?)
                } else {
                    None
                }];
                loop {
                    if self.eat(&Pipe).is_none() {
                        break;
                    }
                    variant_names.push(self.eat_ident_or_err().ok_or(UntilNextStatement)?);
                    variant_types.push(if self.eat(&Of).is_some() {
                        Some(self.parse_type_expression(0)?)
                    } else {
                        None
                    });
                }
                TypeDefinitionKind::Sum {
                    variant_names,
                    variant_types,
                }
            }
            _ => TypeDefinitionKind::Expression(self.parse_type_expression(0)?),
        }
        .with_span(span + self.last_span))
    }

    pub fn parse_type_expression(
        &mut self,
        precedence: Precedence,
    ) -> Result<TypeExpression> {
        let mut current = self.type_primary()?;
        let span = current.span;
        while let Some(next) = self.peek() {
            // Binary op
            if let Ok(op) = BinaryTypeOp::try_from(&*next) {
                let new_precedence = op.precedence();
                // End precedence climb
                if ((op.assoc() == LEFT_ASSOC) && new_precedence <= precedence)
                    || (new_precedence < precedence)
                {
                    return Ok(current);
                }
                self.skip();
                current = TypeExpression {
                    inner: match op {
                        BinaryTypeOp::Arrow => {
                            TypeExpressionKind::Function(
                                Box::new(current),
                                Box::new(self.parse_type_expression(new_precedence)?),
                            )
                        }
                    },
                    span: span + self.last_span,
                };
            }
            // Function application
            else if precedence < CALL_PREC
                && let Some(t) = self.peek()
                && (t.inner.is_literal() || matches!(t.inner, Identifier(_)))
            {
                current = TypeExpressionKind::Call(
                    Box::new(current),
                    Box::new(self.parse_type_expression(CALL_PREC)?),
                )
                .with_span(span + self.last_span);
            } else {
                break;
            }
        }
        Ok(current)
    }
}
