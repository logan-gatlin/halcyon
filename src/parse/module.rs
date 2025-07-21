use super::*;

#[derive(Debug, Clone)]
pub enum ModuleExpressionKind {
  Let {
    is_recursive: bool,
    assignee: String,
    value: Box<ValueExpression>,
  },
  Type {
    is_recursive: bool,
    assignee: String,
    value: Box<TypeExpression>,
  },
}

pub type ModuleExpression = Expression<ModuleExpressionKind>;

#[derive(Debug, Clone)]
pub struct ParsedModule {
  name: String,
  contents: Vec<ModuleExpression>,
  span: Span,
}

pub fn parse_module(iter: it!()) -> Result<ParsedModule> {
  iter.start_span();
  // module keyword
  iter.eat_or_error(Module)?;
  // name
  let name = iter.eat_ident()?;
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
  let is_let = iter.eat_one_of([Let, Type])? == 0;
  iter.skip(0);
  let assignee = iter.eat_ident()?;
  let is_recursive = iter.eat_one_of([Equal, DoubleColon])? == 1;
  if is_let {
    Ok(ModuleExpression {
      kind: ModuleExpressionKind::Let {
        is_recursive,
        assignee,
        value: Box::new(parse_value_expression(iter, 0)?),
      },
      span: iter.end_span(),
    })
  } else {
    Ok(ModuleExpression {
      kind: ModuleExpressionKind::Type {
        is_recursive,
        assignee,
        value: todo!(),
      },
      span: iter.end_span(),
    })
  }
}
