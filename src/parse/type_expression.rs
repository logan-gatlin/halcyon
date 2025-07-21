use super::*;

use TypeExpressionKind as e;

#[derive(Debug, Clone)]
pub enum TypeExpressionKind {
  StructureDefinition {
    lhs: Vec<String>,
    rhs: Vec<TypeExpression>,
  },
  Identifier(String),
  Binary {
    op: BinaryTypeOp,
    left: Box<TypeExpression>,
    right: Box<TypeExpression>,
  },
  Unit,
}

pub type TypeExpression = Expression<TypeExpressionKind>;

fn parse_primary(iter: it!()) -> Result<TypeExpression> {
  iter.start_span();
  let Some(Token(next, _)) = iter.next() else {
    return Err(iter.report_error(ExpectedExpression, []));
  };
  let kind = match next {
    Identifier(ident) => e::Identifier(ident),
    LeftParen if iter.eat(RightParen).is_some() => e::Unit,
    _ => return Err(iter.report_error(ExpectedExpression, [])),
  };
  todo!()
}

pub fn parse_type_expression(iter: it!()) -> Result<TypeExpression> {
  todo!()
}
