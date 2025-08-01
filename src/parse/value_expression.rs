use std::collections::HashSet;

use super::*;
use ValueExpressionKind as e;

#[derive(Debug, Clone)]
pub enum Literal {
  Unit,
  Integer(String, Base),
  Real(String),
  String(String),
  Glyph(char),
  Boolean(bool),
}

#[derive(Debug, Clone)]
pub enum ValueExpressionKind {
  Let {
    assignee: String,
    value: Box<ValueExpression>,
    in_: Option<Box<ValueExpression>>,
  },
  Literal(Literal),
  Identifier(String),
  Binary {
    op: BinaryOp,
    left: Box<ValueExpression>,
    right: Box<ValueExpression>,
  },
  Unary {
    op: UnaryOp,
    child: Box<ValueExpression>,
  },
  FunctionDef {
    arguments: Vec<String>,
    argument_spans: Vec<Span>,
    types: Vec<Option<TypeExpression>>,
    body: Box<ValueExpression>,
  },
  FunctionCall {
    callee: Box<ValueExpression>,
    argument: Box<ValueExpression>,
  },
  If {
    predicate: Box<ValueExpression>,
    then: Box<ValueExpression>,
    else_: Option<Box<ValueExpression>>,
  },
  Tuple(Vec<ValueExpression>),
  StructureLiteral {
    lhs: Vec<String>,
    rhs: Vec<ValueExpression>,
  },
  Field {
    lhs: Box<ValueExpression>,
    rhs: String,
  },
  ModuleField(Vec<String>),
}

pub type ValueExpression = Expression<ValueExpressionKind>;

fn parse_primary(iter: it!()) -> Result<ValueExpression> {
  iter.start_span();
  let Some(Token(next, _)) = iter.next() else {
    return Err(iter.report_error(ExpectedExpression, []));
  };
  let kind = match next {
    LeftParen if iter.eat(RightParen).is_some() => e::Literal(Literal::Unit),
    IntegerLiteral(value, base) => e::Literal(Literal::Integer(value, base)),
    RealLiteral(value) => e::Literal(Literal::Real(value)),
    StringLiteral(value) => e::Literal(Literal::String(value)),
    GlyphLiteral(value) => e::Literal(Literal::Glyph(value)),
    True => e::Literal(Literal::Boolean(true)),
    False => e::Literal(Literal::Boolean(false)),
    Identifier(ident) if iter.peek(0).is_none_or(|t| t.0 != Colon) => e::Identifier(ident),
    // Module field
    Identifier(ident) => {
      let mut path = vec![ident];
      while iter.eat(Colon).is_some() {
        path.push(iter.eat_ident()?);
      }
      e::ModuleField(path)
    }
    // Function definition
    Fn => {
      let mut arguments = vec![];
      let mut types = vec![];
      let mut spans = vec![];
      loop {
        if iter.eat(FatArrow).is_some() {
          break;
        }
        // Typed parameter
        if iter.eat(LeftParen).is_some() {
          arguments.push(iter.eat_ident()?);
          spans.push(iter.last_span);
          iter.eat_or_error(Colon)?;
          types.push(Some(parse_type_expression(iter)?));
          iter.eat_or_error(RightParen)?;
        }
        // Untyped parameter
        else {
          arguments.push(iter.eat_ident()?);
          spans.push(iter.last_span);
          types.push(None);
        }
      }
      let mut parameter_set = HashSet::new();
      for i in 0..arguments.len() {
        if !parameter_set.insert(&arguments[i]) {
          return Err(lint(
            NameLint::ParamRedefinition,
            spans[i],
            ["function".to_string(), arguments[i].clone()],
          ));
        }
      }
      let body = Box::new(parse_value_expression(iter, 0)?);
      e::FunctionDef {
        arguments,
        argument_spans: spans,
        types,
        body,
      }
    }
    Let => e::Let {
      assignee: iter.eat_ident()?,
      value: {
        iter.eat_or_error(Equal)?;
        Box::new(parse_value_expression(iter, 0)?)
      },
      in_: if iter.eat(In).is_some() {
        Some(Box::new(parse_value_expression(iter, 0)?))
      } else {
        None
      },
    },
    // Struct literal
    LeftBrace => {
      let mut rhs = vec![];
      let mut lhs = vec![];
      loop {
        if iter.eat(RightBrace).is_some() {
          break;
        }
        lhs.push(iter.eat_ident()?);
        iter.eat_or_error(Equal)?;
        rhs.push(parse_value_expression(iter, 0)?);
        if iter.eat(Comma).is_none() && iter.peek_or_error(0, RightBrace).is_err() {
          iter.start_span();
          return Err(if iter.peek_or_error(0, Identifier("".into())).is_ok() {
            iter.report_error(ExpectedToken, [format!("{Comma}")])
          } else {
            iter.report_error(ExpectedToken, [format!("{RightBrace}")])
          });
        }
      }
      e::StructureLiteral { lhs, rhs }
    }
    If => e::If {
      predicate: Box::new(parse_value_expression(iter, 0)?),
      then: {
        iter.eat_or_error(Then)?;
        Box::new(parse_value_expression(iter, 0)?)
      },
      else_: if iter.eat(Else).is_some() {
        Some(Box::new(parse_value_expression(iter, 0)?))
      } else {
        None
      },
    },
    // Tuple or parenthesis
    LeftParen => {
      let mut inner = vec![];
      let mut is_tuple = false;
      loop {
        if iter.eat(RightParen).is_some() {
          break;
        }
        inner.push(parse_value_expression(iter, 0)?);
        if iter.eat(Comma).is_some() {
          is_tuple = true;
        } else if iter.peek_or_error(0, RightParen).is_err() {
          iter.start_span();
          return Err(iter.report_error(ExpectedToken, [format!("{RightParen}")]));
        }
      }
      if is_tuple {
        e::Tuple(inner)
      } else {
        let mut inner = inner[0].clone();
        inner.span = iter.end_span();
        return Ok(inner);
      }
    }
    _ => return Err(iter.report_error(ExpectedExpression, [])),
  };
  Ok(ValueExpression {
    kind,
    span: iter.end_span(),
  })
}

pub fn parse_value_expression(iter: it!(), precedence: Precedence) -> Result<ValueExpression> {
  iter.start_span();
  let unary_ops = [Minus, MinusDot, Not];
  let mut current = if let Ok(id) = iter.eat_one_of(unary_ops.clone()) {
    let op = UnaryOp::try_from(&unary_ops[id]).unwrap();
    if op.assoc() == RIGHT_ASSOC {
      return Err(iter.report_error(ExpectedExpression, []));
    }
    let operand = parse_value_expression(iter, op.precedence())?;
    let op = if let (UnaryOp::Minus, e::Literal(Literal::Real(_))) = (op, &operand.kind) {
      UnaryOp::MinusDot
    } else {
      op
    };
    let span = iter.end_span();
    Expression {
      kind: e::Unary {
        op,
        child: operand.into(),
      },
      span,
    }
  } else {
    iter.end_span();
    parse_primary(iter)?
  };
  const TERMINAL_TOKENS: [TokenKind; 9] = [
    Let, Type, Import, End, RightParen, RightBrace, In, Then, Else,
  ];
  // Precedence climbing loop
  while let Some(next) = iter.peek(0) {
    if TERMINAL_TOKENS.contains(&next.0) {
      break;
    }
    iter.start_span();
    // Binary operator
    if let Ok(op) = BinaryOp::try_from(&next.0) {
      let new_precedence = op.precedence();
      // End precedence climb
      if ((op.assoc() == LEFT_ASSOC) && new_precedence <= precedence)
        || (new_precedence < precedence)
      {
        iter.end_span();
        return Ok(current);
      }
      iter.skip(1);
      current = ValueExpression {
        kind: e::Binary {
          op,
          left: current.into(),
          right: Box::new(parse_value_expression(iter, new_precedence)?),
        },
        span: iter.end_span(),
      }
    }
    // Field
    else if precedence < FIELD_PREC && iter.eat(Dot).is_some() {
      let rhs = iter.eat_ident()?;
      current = ValueExpression {
        kind: e::Field {
          lhs: current.into(),
          rhs,
        },
        span: iter.end_span(),
      }
    }
    // Function call
    else if precedence < CALL_PREC {
      current = ValueExpression {
        kind: e::FunctionCall {
          callee: current.into(),
          argument: Box::new(parse_value_expression(iter, CALL_PREC)?),
        },
        span: iter.end_span(),
      }
    } else {
      break;
    }
  }
  Ok(current)
}
