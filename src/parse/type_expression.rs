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
  ModuleField {
    lhs: Box<TypeExpression>,
    rhs: String,
  },
}

pub type TypeExpression = Expression<TypeExpressionKind>;

fn parse_primary(iter: it!()) -> Result<TypeExpression> {
  iter.start_span();
  let Some(Token(next, _)) = iter.next() else {
    return Err(iter.report_error(ExpectedExpression, []));
  };
  let kind = match next {
    Identifier(ident) => e::Identifier(ident),
    LeftParen => {
      let mut inner = parse_type_expression(iter, 0)?;
      iter.eat_or_error(RightParen)?;
      inner.span = iter.end_span();
      return Ok(inner);
    },
    LeftBrace => {
      let mut rhs = vec![];
      let mut lhs = vec![];
      loop {
        if iter.eat(RightBrace).is_some() {
          break;
        }
        lhs.push(iter.eat_ident()?);
        iter.eat_or_error(Colon)?;
        rhs.push(parse_type_expression(iter, 0)?);
        if iter.eat(Comma).is_none()
          && iter.peek_or_error(0, RightBrace).is_err()
        {
          iter.start_span();
          return Err(
            if iter.peek_or_error(0, Identifier("".into())).is_ok() {
              iter.report_error(ExpectedToken, [format!("{Comma}")])
            } else {
              iter.report_error(ExpectedToken, [format!("{RightBrace}")])
            },
          );
        }
      }
      e::StructureDefinition { lhs, rhs }
    },
    _ => return Err(iter.report_error(ExpectedExpression, [])),
  };
  Ok(TypeExpression {
    kind,
    span: iter.end_span(),
  })
}

pub fn parse_type_expression(
  iter: it!(),
  precedence: Precedence,
) -> Result<TypeExpression> {
  let mut current = parse_primary(iter)?;

  const TERMINAL_TOKENS: [TokenKind; 4] = [Let, Type, End, RightParen];
  while let Some(next) = iter.peek(0) {
    if TERMINAL_TOKENS.contains(&next.0) {
      break;
    }
    iter.start_span();
    if let Ok(op) = BinaryTypeOp::try_from(&next.0) {
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
        kind: e::Binary {
          op,
          left: current.into(),
          right: Box::new(parse_type_expression(iter, new_precedence)?),
        },
        span: iter.end_span(),
      }
    } else if precedence < MODULE_FIELD_PREC && iter.eat(Colon).is_some() {
      let rhs = iter.eat_ident()?;
      current = TypeExpression {
        kind: e::ModuleField {
          lhs: current.into(),
          rhs,
        },
        span: iter.end_span(),
      }
    } else {
      break;
    }
  }
  Ok(current)
}
