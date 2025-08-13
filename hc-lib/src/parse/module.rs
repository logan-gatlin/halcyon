use super::*;

#[derive(Debug, Clone)]
pub enum ModuleExpressionKind {
    Let {
        assignee: PatternExpression,
        value: Box<ValueExpression>,
    },
    Type {
        assignee: String,
        assignee_span: Span,
        value: Box<TypeDefinition>,
    },
    Import {
        name: String,
    },
}

pub type ModuleExpression = Expression<ModuleExpressionKind>;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParsedModule {
    pub name: String,
    pub contents: Vec<ModuleExpression>,
    pub span: Span,
}

pub fn parse_module(iter: it!()) -> Result<ParsedModule> {
    iter.start_span();
    // module keyword
    iter.eat_or_error(Module)?;
    // name
    let name = iter.eat_ident()?;
    iter.eat_or_error(Equal)?;
    let mut contents = vec![];
    // top-level expressions
    loop {
        match iter.peek(0) {
            None | Some(Token(End, _)) | Some(Token(EOF, _)) => break,
            Some(_) => contents.push(parse_module_expression(iter)?),
        };
    }
    // end keyword
    iter.eat_or_error(End)?;
    let module_span = iter.end_span();
    Ok(ParsedModule {
        name,
        contents,
        span: module_span,
    })
}

pub fn parse_module_expression(iter: it!()) -> Result<ModuleExpression> {
    iter.start_span();
    let variant = iter.eat_one_of([Let, Type, Import])?;
    match variant {
        // Let
        0 => {
            let assignee = parse_pattern(iter)?;
            iter.eat_or_error(Equal)?;
            Ok(ModuleExpression {
                kind: ModuleExpressionKind::Let {
                    assignee,
                    value: Box::new(parse_value_expression(iter, 0)?),
                },
                span: iter.end_span(),
            })
        }
        // Type
        1 => {
            let assignee = iter.eat_ident()?;
            let assignee_span = iter.last_span;
            iter.eat_or_error(Equal)?;
            Ok(ModuleExpression {
                kind: ModuleExpressionKind::Type {
                    assignee,
                    assignee_span,
                    value: Box::new(parse_type_definition(iter)?),
                },
                span: iter.end_span(),
            })
        }
        // Import
        2 => Ok(ModuleExpression {
            kind: ModuleExpressionKind::Import {
                name: iter.eat_ident()?,
            },
            span: iter.end_span(),
        }),
        _ => unreachable!(),
    }
}
