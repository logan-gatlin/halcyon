use super::*;

pub type TypeDefinition = Expression<TypeDefinitionKind>;
pub type TypeExpression = Expression<TypeExpressionKind>;

#[derive(Debug, Clone)]
pub enum TypeDefinitionKind {
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

#[derive(Debug, Clone)]
pub enum TypeExpressionKind {
  Identifier(String),
  Product(Vec<TypeExpression>),
  ModulePath(Vec<String>),
}

pub fn parse_type_definition(iter: it!()) -> Result<TypeDefinition> {
  iter.start_span();
  let next = iter.peek(0).ok_or(lint(
    ParseLint::ExpectedExpression,
    iter.span_after_this(),
    [],
  ))?;
  let kind = match &next.0 {
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
        let expr = parse_type_expression(iter)?;
        rhs.push(expr);
        if iter.eat(Comma).is_none()
          && iter.peek(0).is_none_or(|t| t.0 != RightBrace)
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
      TypeDefinitionKind::Structure { lhs, rhs }
    },
    // Sum
    Identifier(name) if iter.peek_or_error(1, Of).is_ok() => {
      let mut variant_names = vec![name.clone()];
      iter.skip(2);
      let mut variant_types = vec![parse_type_expression(iter)?];
      loop {
        if iter.eat(Pipe).is_none() {
          break;
        }
        variant_names.push(iter.eat_ident()?);
        iter.eat_or_error(Of)?;
        variant_types.push(parse_type_expression(iter)?);
      }
      TypeDefinitionKind::Sum {
        variant_names,
        variant_types,
      }
    },
    // Other expression
    _ => TypeDefinitionKind::Expression(parse_type_expression(iter)?),
  };
  Ok(TypeDefinition {
    kind,
    span: iter.end_span(),
  })
}

pub fn parse_type_expression(iter: it!()) -> Result<TypeExpression> {
  fn primary(iter: it!()) -> Result<TypeExpression> {
    iter.start_span();
    let kind = match iter.eat_ident()? {
      // Module field
      name if iter.peek_or_error(0, Colon).is_ok() => {
        let mut path = vec![name];
        while iter.eat(Colon).is_some() {
          path.push(iter.eat_ident()?);
        }
        TypeExpressionKind::ModulePath(path)
      },
      // Basic ident
      name => TypeExpressionKind::Identifier(name),
    };
    Ok(TypeExpression {
      kind,
      span: iter.end_span(),
    })
  }
  iter.start_span();
  let mut product = vec![primary(iter)?];
  while iter.eat(Star).is_some() {
    product.push(primary(iter)?);
  }
  match &product[..] {
    [] => unreachable!(),
    [primary] => {
      iter.end_span();
      return Ok(primary.clone());
    },
    [..] => Ok(TypeExpression {
      kind: TypeExpressionKind::Product(product),
      span: iter.end_span(),
    }),
  }
}
