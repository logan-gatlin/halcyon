use super::*;

#[derive(Debug, Clone)]
pub enum ModuleStatementKind {
    DocComment(String),
    Let {
        assignee: PatternExpression,
        value: Box<ValueExpression>,
    },
    Type {
        assignee: Spanned<String>,
        value: Box<TypeDefinition>,
    },
}

pub type ModuleStatement = Expression<ModuleStatementKind>;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InnerParsedModule {
    pub name: Spanned<String>,
    pub contents: Vec<ModuleStatement>,
}

pub type ParsedModule = Spanned<InnerParsedModule>;

impl<'a, I: Iterator<Item = Token>> Parser<'a, I> {
    pub fn parse_module(&mut self) -> ParsedModule {
        macro_rules! recover {
            () => {
                loop {
                    match self.peek().map(|t| t.inner) {
                        Some(Let | Type | Do | Import | End | Module | DocComment(_)) => break,
                        _ => {}
                    }
                    self.skip();
                }
            };
        }
        macro_rules! try_ {
            ($e:expr) => {
                match $e {
                    Some(v) => v,
                    None => {
                        recover!();
                        continue;
                    }
                }
            };

            ($e:expr, $note:expr) => {
                match $e {
                    Some(v) => v,
                    None => {
                        self.logger
                            .error(ERR_MSG)
                            .primary($note, self.last_span)
                            .done();
                        // Error recovery
                        recover!();
                        continue;
                    }
                }
            };
        }
        use ModuleStatementKind as m;
        loop {
            if self.eat(Module).is_some() {
                break;
            } else {
                self.error_expected(Module)
                    .note("Expressions are only allowed inside of modules.")
                    .done();
                recover!()
            }
        }
        let span = self.last_span;
        let name = self
            .eat_ident()
            .unwrap_or_else(|| {
                self.error_expected(Identifier("identifier".to_string()))
                    .done();
                "_".to_string()
            })
            .with_span(self.last_span);
        p.eat(Equal).unwrap_or_else(|e| logger.log(e));
        let mut contents = vec![];
        loop {
            let next = if let Ok(next) = p.next() {
                next
            } else {
                error!(logger, p.last_span, "Expected `end` after this");
                break;
            };
            match next.inner {
                End => break,
                DocComment(s) => {
                    let s = s.trim().to_string();
                    contents.push(m::DocComment(s).with_span(p.last_span))
                }
                Let => {
                    let assignee = try_! {parse_pattern(logger, p), ""};
                    let span = assignee.span;
                    try_! {p.eat(Equal), ""};
                    let value = Box::new(try_!(parse_value_expression(logger, p, 0)));
                    contents.push(m::Let { assignee, value }.with_span(span + p.last_span));
                }
                Type => {
                    let assignee = try_!(p.eat_ident()).with_span(p.last_span);
                    try_!(p.eat(Equal));
                    contents.push(
                        m::Type {
                            assignee,
                            value: Box::new(try_!(parse_type_definition(logger, p))),
                        }
                        .with_span(span + p.last_span),
                    )
                }
                _ => {
                    error!(
                        logger,
                        p.last_span, "Expected a statement beginning with `let` or `type` here"
                    );
                    recover!();
                }
            }
        }
        InnerParsedModule { name, contents }.with_span(span + p.last_span)
    }
}
