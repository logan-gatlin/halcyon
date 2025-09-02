use super::*;

#[derive(Debug, Clone, sx::SXRepr)]
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
        type_: Box<TypeExpression>,
        major: String,
        minor: String,
    },
}

pub type ModuleExpression = Expression<ModuleExpressionKind>;

#[allow(dead_code)]
#[derive(Debug, Clone, sx::SXRepr)]
pub struct _ParsedModule {
    pub name: Spanned<String>,
    pub contents: Vec<ModuleExpression>,
}

pub type ParsedModule = Spanned<_ParsedModule>;

pub fn parse_module(iter: it!()) -> Result<ParsedModule> {
    iter.start_span();
    // module keyword
    iter.eat_or_error(Module)?;
    // name
    let name = iter.eat_ident()?.with_span(iter.last_span);
    iter.eat_or_error(Equal)?;
    let mut contents = vec![];
    // top-level expressions
    loop {
        match iter.peek(0) {
            None
            | Some(Token {
                inner: End | Eof, ..
            }) => break,
            Some(_) => contents.push(parse_module_expression(iter)?),
        };
    }
    // end keyword
    iter.eat_or_error(End)?;
    let module_span = iter.end_span();
    Ok(_ParsedModule { name, contents }.with_span(module_span))
}

pub fn parse_module_expression(iter: it!()) -> Result<ModuleExpression> {
    iter.start_span();
    let variant = iter.eat_one_of([Let, Type, Import])?;
    Ok(match variant {
        // Let
        0 => {
            let assignee = parse_pattern(iter)?;
            iter.eat_or_error(Equal)?;
            ModuleExpressionKind::Let {
                assignee,
                value: Box::new(parse_value_expression(iter, 0)?),
            }
        }
        // Type
        1 => {
            let assignee = iter.eat_ident()?;
            let assignee_span = iter.last_span;
            iter.eat_or_error(Equal)?;
            ModuleExpressionKind::Type {
                assignee,
                assignee_span,
                value: Box::new(parse_type_definition(iter)?),
            }
        }
        // Import
        2 => {
            let name = iter.eat_ident()?;
            iter.eat_or_error(Colon)?;
            let type_ = parse_type_expression(iter, 0)?.into();
            iter.eat_or_error(Equal)?;
            let major = iter.eat_ident()?;
            let minor = iter.eat_ident()?;
            ModuleExpressionKind::Import {
                name,
                type_,
                major,
                minor,
            }
        }
        _ => unreachable!(),
    }
    .with_span(iter.end_span()))
}
