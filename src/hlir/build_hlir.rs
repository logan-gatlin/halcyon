use super::*;
use crate::{lint::*, operator::*, parse::*, span::*, token::*};
use std::collections::HashSet;

pub fn parse_int_literal(value: &str, base: u32) -> Result<i64> {
  i64::from_str_radix(value, base).lint(TokenLint::InvalidInteger)
}

pub fn parse_real_literal(value: &str) -> Result<f64> {
  value.parse().lint(TokenLint::InvalidReal)
}
#[derive(Debug, Clone)]
struct Symbol {
  pub mangle: Mangle,
  pub scope_depth: usize,
  pub is_constant: bool,
}

#[derive(Debug, Clone)]
enum Event {
  ScopeStart,
  Modify {
    name: String,
    old_value: Option<Symbol>,
  },
}

fn collect_list(mut expr: Expression) -> Vec<Expression> {
  let mut exprs = vec![];
  use ExpressionKind as e;
  loop {
    match expr.kind {
      e::Binary {
        op: BinaryOp::Comma,
        left,
        right,
      } => {
        exprs
          .extend(collect_list(*right).into_iter().rev().collect::<Vec<_>>());
        expr = *left;
      },
      _ => {
        exprs.push(expr);
        exprs.reverse();
        break exprs;
      },
    }
  }
}

fn collect_parameters(mut expr: Expression) -> Vec<Expression> {
  let mut exprs = vec![];
  use ExpressionKind as e;
  loop {
    match expr.kind {
      e::FunctionCall { callee, arguments } => {
        exprs.extend(
          collect_parameters(*arguments)
            .into_iter()
            .rev()
            .collect::<Vec<_>>(),
        );
        expr = *callee;
      },
      _ => {
        exprs.push(expr);
        exprs.reverse();
        break exprs;
      },
    }
  }
}

fn collect_block(mut expr: Expression) -> Vec<Expression> {
  let mut exprs = vec![];
  use ExpressionKind as e;
  loop {
    match expr.kind {
      e::Binary {
        op: BinaryOp::Semicolon | BinaryOp::DoubleSemicolon,
        left,
        right,
      } => {
        exprs.push(*right);
        expr = *left;
      },
      _ => {
        exprs.push(expr);
        exprs.reverse();
        break exprs;
      },
    }
  }
}

#[derive(Debug, Clone)]
pub struct Canonizer {
  nodes: Vec<Option<HlIrNode>>,
  constants: HashMap<Mangle, IrPtr>,
  scope_depth: usize,
  salt: usize,
  path: Vec<String>,
  event_stack: Vec<Event>,
  memory: Memory,
  _name_to_symbol: HashMap<String, Symbol>,
}

impl Canonizer {
  pub fn new() -> Self {
    let mut this = Self {
      nodes: vec![],
      constants: HashMap::new(),
      path: vec![],
      event_stack: vec![],
      memory: Memory::new(10, 100),
      scope_depth: 0,
      salt: 0,
      _name_to_symbol: HashMap::new(),
    };
    for builtin in Builtin::ALL {
      this.define_builtin(builtin.to_string())
    }
    this
  }

  fn literal_to_const(&mut self, literal: Literal) -> Result<ConstValue> {
    Ok(match literal {
      Literal::Unit => ConstValue::Nothing,
      Literal::Integer(i, base) => {
        ConstValue::Integer(parse_int_literal(&i, base as u32)?)
      },
      Literal::Real(r) => ConstValue::Real(parse_real_literal(&r)?),
      Literal::String(s) => {
        let address = self.memory.static_allocate(s.len() as PtrT);
        ConstValue::String {
          address,
          length: s.len() as PtrT,
        }
      },
      Literal::Glyph(g) => ConstValue::Glyph(g),
      Literal::Boolean(b) => ConstValue::Boolean(b),
    })
  }

  pub fn canonize_expr(mut self, expr: Expression) -> Result<HlIrModule> {
    self.expr(expr)?;
    Ok(HlIrModule {
      nodes: self.nodes.into_iter().map(|n| n.unwrap()).collect(),
      constants: self.constants,
      type_map: HashMap::new(),
      heap: self.memory,
    })
  }

  fn expr(&mut self, expr: Expression) -> Result<IrPtr> {
    use ExpressionKind as e;
    use HlIrKind as h;
    let node = self.new_node();
    let span = expr.span;
    let kind = match expr.kind {
      e::Literal(literal) => {
        h::Immediate(self.literal_to_const(literal).span(expr.span)?)
      },
      e::Identifier(name) => {
        let symbol = self.name_to_symbol(&name).span(span)?;
        h::Identifier(symbol.mangle.clone())
      },
      // Function def
      e::Binary {
        op: BinaryOp::FatArrow,
        left,
        right,
      } => {
        let parameter_exprs = collect_parameters(*left);
        let parameter_kinds =
          parameter_exprs.iter().map(|a| &a.kind).collect::<Vec<_>>();
        let parameter_spans =
          parameter_exprs.iter().map(|a| a.span).collect::<Vec<_>>();
        let (parameter_names, parameter_spans): (Vec<String>, Vec<Span>) =
          match parameter_kinds.as_slice() {
            [e::Literal(Literal::Unit)] => (vec![], vec![]),
            _ => parameter_exprs
              .iter()
              .map(|e| {
                if let Expression {
                  kind: e::Identifier(name),
                  ..
                } = e
                {
                  Ok((name.clone(), e.span))
                } else {
                  Err(lint(ParseLint::InvalidLambdaParameter, span, &[]))
                }
              })
              .try_collect::<Vec<_>>()?
              .into_iter()
              .unzip(),
          };
        self.start_function();
        self.enscope();
        let parameter_names = parameter_names
          .iter()
          .zip(parameter_spans.iter())
          .map(|(name, span)| self.define_name(name, false).span(*span))
          .try_collect::<Vec<_>>()?;
        let body = self.expr(*right)?;
        self.descope();
        let mangle = self.define_unique("function");
        self.constants.insert(mangle.clone(), node);
        h::FunctionDef {
          name: mangle,
          parameter_names,
          parameter_spans,
          body,
        }
      },
      e::Let {
        is_type,
        is_recursive: is_constant,
        assignee_span,
        assignee,
        value,
        in_,
      } => {
        self.enscope();
        let (assignee, value) = if is_constant {
          (
            self.define_name(assignee, true).span(assignee_span)?,
            self.expr(*value)?,
          )
        } else {
          let t = (
            self.expr(*value)?,
            self.define_name(assignee, false).span(assignee_span)?,
          );
          (t.1, t.0)
        };
        let in_ = if let Some(in_) = in_ {
          Some(self.expr(*in_)?)
        } else {
          None
        };
        self.descope();
        h::Declaration {
          value,
          assignee,
          is_constant,
          in_,
        }
      },
      // Tuple
      e::Binary {
        op: BinaryOp::Comma,
        ..
      } => h::Tuple(
        collect_list(expr)
          .into_iter()
          .map(|e| self.expr(e))
          .try_collect::<Vec<_>>()?,
      ),
      // Field get
      e::Binary {
        op: BinaryOp::Dot,
        left,
        right,
      } => {
        let e::Identifier(index) = right.kind else {
          return Err(lint(NameLint::FieldNotIdent, right.span, &[]));
        };
        h::Field {
          of: self.expr(*left)?,
          index,
        }
      },
      // Block
      e::Binary {
        op: BinaryOp::Semicolon | BinaryOp::DoubleSemicolon,
        ..
      } => h::Block(
        collect_block(expr)
          .into_iter()
          .map(|e| self.expr(e))
          .try_collect::<Vec<_>>()?,
      ),
      e::Binary { op, left, right } => h::Binary {
        op,
        left: self.expr(*left)?,
        right: self.expr(*right)?,
      },
      e::Unary { op, child } => h::Unary {
        op,
        child: self.expr(*child)?,
      },
      e::FunctionCall { callee, arguments } => h::FunctionCall {
        callee: self.expr(*callee)?,
        callee_name: self.define_unique("callee"),
        arguments: collect_list(*arguments)
          .into_iter()
          .map(|e| self.expr(e))
          .try_collect::<Vec<_>>()?,
      },
      e::If {
        predicate,
        then,
        else_,
      } => {
        self.enscope();
        let kind = h::If {
          predicate: self.expr(*predicate)?,
          then: {
            let body = self.expr(*then)?;
            body
          },
          else_: if let Some(else_) = else_ {
            let body = self.expr(*else_)?;
            Some(body)
          } else {
            None
          },
        };
        self.descope();
        kind
      },
      e::Structure {
        is_definition,
        lhs,
        rhs,
      } => {
        let field_names = lhs;
        let right = rhs.into_iter().map(|e| self.expr(e)).try_collect()?;
        if is_definition {
          h::StructDef {
            field_names,
            field_types: right,
          }
        } else {
          h::StructLiteral {
            struct_t: None,
            field_names,
            field_values: right,
          }
        }
      },
    };
    self.set_node(
      node,
      HlIrNode {
        kind,
        span,
        type_: Type::Ambiguous,
      },
    );
    Ok(node)
  }

  fn pattern(&mut self, expr: Expression) -> Result<Pattern> {
    let span = expr.span;
    use ExpressionKind as e;
    Ok(match expr.kind {
      e::Identifier(name) => {
        let mangle = self.define_name(name, false).span(span)?;
        Pattern {
          kind: PatternKind::Wildcard(mangle),
          span,
        }
      },
      e::Literal(literal) => Pattern {
        kind: PatternKind::Const(self.literal_to_const(literal).span(span)?),
        span,
      },
      e::Binary {
        op: BinaryOp::Comma,
        ..
      } => Pattern {
        kind: PatternKind::Tuple(
          collect_list(expr)
            .into_iter()
            .map(|e| self.pattern(e))
            .try_collect()?,
        ),
        span,
      },
      _ => return Err(lint(ParseLint::InvalidPattern, span, &[])),
    })
  }

  fn new_node(&mut self) -> IrPtr {
    self.nodes.push(None);
    self.nodes.len() - 1
  }

  fn set_node(&mut self, position: IrPtr, node: HlIrNode) {
    assert!(self.nodes[position].is_none());
    self.nodes[position] = Some(node);
  }

  fn name_to_symbol(&self, name: &str) -> Result<&Symbol> {
    self
      ._name_to_symbol
      .get(name)
      .ok_or(lint_nospan(NameLint::UndefinedName))
      .context(name)
  }

  fn next_salt(&mut self) -> String {
    let returned_salt = self.salt.to_string();
    self.salt += 1;
    returned_salt
  }

  fn allocate(&mut self, bytes: &[u8]) -> PtrT {
    let address = self.memory.static_allocate(bytes.len() as PtrT);
    for (i, b) in bytes.iter().enumerate() {
      self.memory.store(address + i as PtrT, b);
    }
    address
  }

  fn define_unique(&mut self, hint: &str) -> Mangle {
    let name = String::from(hint);
    let mut path = self.path.clone();
    path.push(name.clone());
    let salt = self.next_salt();
    let mangle = mangle_name(path, &salt);
    mangle
  }

  fn define_builtin(&mut self, name: impl Into<String>) {
    let name = name.into();
    let mangle = mangle_builtin(&name);
    assert!(
      self
        ._name_to_symbol
        .insert(
          name.clone(),
          Symbol {
            mangle,
            scope_depth: 0,
            is_constant: true,
          },
        )
        .is_none(),
      "Multiple definitions of builtin {name}"
    );
  }

  fn define_name(
    &mut self,
    name: impl Into<String>,
    is_constant: bool,
  ) -> Result<Mangle> {
    let name = name.into();
    if name == "_" {
      return Ok("_".to_string());
    }
    let mut path = self.path.clone();
    path.push(name.clone());
    let salt = self.next_salt();
    let mangle = mangle_name(path, &salt);
    let old_value = self.name_to_symbol(&name).ok().cloned();
    if let Some(old) = &old_value {
      if old.scope_depth == self.scope_depth && is_constant == old.is_constant {
        return Err(lint_nospan(NameLint::ConstRedefinition)).context(name);
      }
    }
    let event = Event::Modify {
      old_value,
      name: name.clone(),
    };
    self.event_stack.push(event);
    self._name_to_symbol.insert(
      name.clone(),
      Symbol {
        mangle: mangle.clone(),
        scope_depth: self.scope_depth,
        is_constant,
      },
    );
    Ok(mangle)
  }

  fn enscope(&mut self) {
    self.event_stack.push(Event::ScopeStart);
    self.scope_depth += 1;
  }

  fn descope(&mut self) {
    while let Some(e) = self.event_stack.pop() {
      match e {
        Event::ScopeStart => {
          self.scope_depth -= 1;
          break;
        },
        Event::Modify { name, old_value } => {
          if let Some(old) = old_value {
            self._name_to_symbol.insert(name, old);
          } else {
            self._name_to_symbol.remove(&name);
          }
        },
      }
    }
  }

  fn start_function(&mut self) {
    let mut to_reset = vec![];
    for (name, symbol) in self._name_to_symbol.iter() {
      if !symbol.is_constant {
        self.event_stack.push(Event::Modify {
          name: name.clone(),
          old_value: Some(symbol.clone()),
        });
        to_reset.push(name.clone())
      }
    }
    for name in to_reset {
      self._name_to_symbol.remove(&name);
    }
  }
}
