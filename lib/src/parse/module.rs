use super::*;

#[derive(Debug, Clone, sx::SXRepr)]
pub enum ModuleStatementKind {
    DocComment(String),
    Let {
        assignee: PatternExpression,
        value: Box<ValueExpression>,
    },
    Do(Box<ValueExpression>),
    Type {
        assignee: String,
        assignee_span: Span,
        value: Box<TypeDefinition>,
    },
    Import {
        name: String,
        type_: Box<TypeExpression>,
        major: String,
        minor: String,
    },
}

pub type ModuleStatement = Expression<ModuleStatementKind>;

#[allow(dead_code)]
#[derive(Debug, Clone, sx::SXRepr)]
pub struct InnerParsedModule {
    pub name: Spanned<String>,
    pub contents: Vec<ModuleStatement>,
}

pub type ParsedModule = Spanned<InnerParsedModule>;

pub fn parse_module(logger: &mut Logger, p: p!()) -> ParsedModule {
    macro_rules! try_ {
        ($e:expr) => {
            match $e {
                Ok(v) => v,
                Err(e) => {
                    error!(logger, e.span, "{}", e.inner);
                    // Error recovery
                    loop {
                        match p.peek().map(|t| t.inner) {
                            Ok(Let | Type | Do | Import | DocComment(_)) | Err(_) => break,
                            _ => {}
                        }
                        p.skip();
                    }
                    continue;
                }
            }
        };
    }
    use ModuleStatementKind as m;
    p.eat(Module).unwrap_or_else(|_| {
        error!(
            logger,
            p.last_span, "Expected `module` here. Statements are not allowed outside of a module"
        );
    });
    let span = p.last_span;
    let name = p
        .eat_ident()
        .unwrap_or_else(|e| {
            error!(logger, e.span, "{}", e.inner);
            "".to_string()
        })
        .with_span(p.last_span);
    p.eat(Equal).unwrap_or_else(|e| {
        error!(logger, e.span, "{}", e.inner);
    });
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
                let assignee = try_! {parse_pattern(logger, p)};
                let span = assignee.span;
                try_! {p.eat(Equal)};
                let value = Box::new(try_!(parse_value_expression(logger, p, 0)));
                contents.push(m::Let { assignee, value }.with_span(span + p.last_span));
            }
            Type => {
                let assignee = try_!(p.eat_ident());
                let span = p.last_span;
                try_!(p.eat(Equal));
                contents.push(
                    m::Type {
                        assignee,
                        assignee_span: span,
                        value: Box::new(try_!(parse_type_definition(logger, p))),
                    }
                    .with_span(span + p.last_span),
                )
            }
            Do => {
                let span = p.last_span;
                let value = try_!(parse_value_expression(logger, p, 0));
                contents.push(m::Do(Box::new(value)).with_span(span + p.last_span))
            }
            Import => {
                let span = p.last_span;
                let name = try_!(p.eat_ident());
                try_!(p.eat(Colon));
                let type_ = try_!(parse_type_expression(logger, p, 0)).into();
                try_!(p.eat(Equal));
                let major = try_!(p.eat_ident());
                try_!(p.eat(DoubleColon));
                let minor = try_!(p.eat_ident());
                contents.push(
                    m::Import {
                        name,
                        type_,
                        major,
                        minor,
                    }
                    .with_span(span + p.last_span),
                )
            }
            _ => {
                error!(logger, p.last_span, "Expected a module statement here");
            }
        }
    }
    InnerParsedModule { name, contents }.with_span(span + p.last_span)
}
