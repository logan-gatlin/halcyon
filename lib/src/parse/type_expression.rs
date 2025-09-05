use std::collections::HashSet;

use super::*;

pub type TypeDefinition = Expression<TypeDefinitionKind>;
pub type TypeExpression = Expression<TypeExpressionKind>;

#[derive(Debug, Clone, sx::SXRepr)]
pub enum TypeDefinitionKind {
    TypeFunction {
        arguments: Vec<String>,
        body: Box<TypeDefinition>,
    },
    Structure {
        lhs: Vec<String>,
        rhs: Vec<TypeExpression>,
    },
    Sum {
        variant_names: Vec<String>,
        variant_types: Vec<TypeExpression>,
    },
    Expression(TypeExpression),
}

#[derive(Debug, Clone, sx::SXRepr)]
pub enum TypeExpressionKind {
    Function(Box<TypeExpression>, Box<TypeExpression>),
    Call(Box<TypeExpression>, Box<TypeExpression>),
    Identifier(String),
    Product(Vec<TypeExpression>),
    ModulePath(Vec<String>),
    Unit,
}

pub fn parse_type_definition(iter: it!()) -> Result<TypeDefinition> {
    iter.start_span();
    let next = iter.peek(0).ok_or(lint(
        ParseLint::ExpectedExpression,
        iter.span_after_this(),
        [],
    ))?;
    Ok(match &*next {
        // Function
        Fn => {
            iter.skip(1);
            let mut arguments = vec![];
            let mut argument_spans = vec![];
            loop {
                if iter.eat(FatArrow).is_some() {
                    break;
                }
                arguments.push(iter.eat_ident()?);
                argument_spans.push(iter.last_span);
            }
            let mut parameter_set = HashSet::new();
            for i in 0..arguments.len() {
                if !parameter_set.insert(&arguments[i]) {
                    return Err(lint(
                        NameLint::ParamRedefinition,
                        argument_spans[i],
                        ["function".to_string(), arguments[i].clone()],
                    ));
                }
            }
            let body = Box::new(parse_type_definition(iter)?);
            TypeDefinitionKind::TypeFunction { arguments, body }
        }
        // Structure
        LeftBrace => {
            let mut lhs = vec![];
            let mut rhs = vec![];
            iter.skip(1);
            loop {
                if iter.eat(RightBrace).is_some() {
                    break;
                }
                let name = iter.eat_ident()?;
                lhs.push(name);
                iter.eat_or_error(Colon)?;
                let expr = parse_type_expression(iter, 0)?;
                rhs.push(expr);
                if iter.eat(Comma).is_none() && iter.peek(0).is_none_or(|t| *t != RightBrace) {
                    iter.start_span();
                    return Err(if iter.peek_or_error(0, Identifier("".into())).is_ok() {
                        iter.report_error(ExpectedToken, [format!("{Comma}")])
                    } else {
                        iter.report_error(ExpectedToken, [format!("{RightBrace}")])
                    });
                }
            }
            TypeDefinitionKind::Structure { lhs, rhs }
        }
        // Sum
        Identifier(name) if iter.peek_or_error(1, Of).is_ok() => {
            let mut variant_names = vec![name.clone()];
            iter.skip(2);
            let mut variant_types = vec![parse_type_expression(iter, 0)?];
            loop {
                if iter.eat(Pipe).is_none() {
                    break;
                }
                variant_names.push(iter.eat_ident()?);
                iter.eat_or_error(Of)?;
                variant_types.push(parse_type_expression(iter, 0)?);
            }
            TypeDefinitionKind::Sum {
                variant_names,
                variant_types,
            }
        }
        // Other expression
        _ => TypeDefinitionKind::Expression(parse_type_expression(iter, 0)?),
    }
    .with_span(iter.end_span()))
}

fn parse_primary(iter: it!()) -> Result<TypeExpression> {
    iter.start_span();
    let Some(next) = iter.next().map(without_span) else {
        return Err(iter.report_error(ExpectedExpression, []));
    };
    Ok(match next {
        LeftParen if iter.eat(RightParen).is_some() => TypeExpressionKind::Unit,
        // Parenthesis
        LeftParen => {
            let mut inner = parse_type_expression(iter, 0)?;
            inner.span = iter.end_span();
            iter.eat_or_error(RightParen)?;
            return Ok(inner);
        }
        // Module path
        Identifier(name) if iter.peek(0).is_some_and(|t| *t == DoubleColon) => {
            let mut path = vec![name];
            while iter.eat(DoubleColon).is_some() {
                path.push(iter.eat_ident()?);
            }
            TypeExpressionKind::ModulePath(path)
        }
        Identifier(name) => TypeExpressionKind::Identifier(name),
        _ => return Err(iter.report_error(ExpectedExpression, [])),
    }
    .with_span(iter.end_span()))
}

pub fn parse_type_expression(iter: it!(), precedence: Precedence) -> Result<TypeExpression> {
    let mut current = parse_primary(iter)?;
    while let Some(next) = iter.peek(0) {
        const TERMINAL_TOKENS: [TokenKind; 8] =
            [Equal, RightParen, RightBrace, End, Comma, Pipe, Let, Type];
        if TERMINAL_TOKENS.contains(&*next) {
            break;
        }
        iter.start_span();
        // Binary op
        if let Ok(op) = BinaryTypeOp::try_from(&*next) {
            let new_precedence = op.precedence();
            // End precedence climb
            if ((op.assoc() == LEFT_ASSOC) && new_precedence <= precedence)
                || (new_precedence < precedence)
            {
                iter.end_span();
                return Ok(current);
            }
            iter.skip(1);
            current = TypeExpression {
                inner: match op {
                    BinaryTypeOp::Arrow => TypeExpressionKind::Function(
                        Box::new(current),
                        Box::new(parse_type_expression(iter, new_precedence)?),
                    ),
                },
                span: iter.end_span(),
            };
        }
        // Tuple
        else if precedence < TYPE_STAR_PREC && *next == Star {
            let mut tuple = vec![current];
            while iter.eat(Star).is_some() {
                tuple.push(parse_type_expression(iter, TYPE_STAR_PREC)?);
            }
            current = TypeExpressionKind::Product(tuple).with_span(iter.end_span());
        } else if precedence < CALL_PREC && !TERMINAL_TOKENS.contains(&*next) {
            current = TypeExpressionKind::Call(
                Box::new(current),
                Box::new(parse_type_expression(iter, CALL_PREC)?),
            )
            .with_span(iter.end_span());
        } else {
            break;
        }
    }
    Ok(current)
}
