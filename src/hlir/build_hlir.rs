use std::collections::HashSet;

use super::*;
use crate::{lint::*, operator::*, parse::*};

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

fn collect_block(mut expr: Expression) -> Vec<Expression> {
  let mut exprs = vec![];
  use ExpressionKind as e;
  loop {
    match expr.kind {
      e::Binary {
        op: BinaryOp::Semicolon,
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
  func_index: u32,
  scope_depth: usize,
  salt: usize,
  event_stack: Vec<Event>,
  _name_to_symbol: HashMap<String, Symbol>,
  exports: HashSet<String>,
}

impl Canonizer {
  pub fn new() -> Self {
    let mut this = Self {
      nodes: vec![],
      func_index: 0,
      exports: HashSet::new(),
      event_stack: vec![],
      scope_depth: 0,
      salt: 0,
      _name_to_symbol: HashMap::new(),
    };
    Type::primitives()
      .iter()
      .flat_map(Type::primitive_mangle)
      .map(|m| this.define_builtin(m));
    this
  }

  fn literal_to_const(&mut self, literal: Literal) -> Result<ConstValue> {
    Ok(match literal {
      Literal::Unit => ConstValue::Unit,
      Literal::Integer(i, base) => {
        ConstValue::Integer(parse_int_literal(&i, base as u32)?)
      },
      Literal::Real(r) => ConstValue::Real(parse_real_literal(&r)?),
      Literal::String(s) => ConstValue::String(s),
      Literal::Glyph(g) => ConstValue::Glyph(g),
      Literal::Boolean(b) => ConstValue::Boolean(b),
    })
  }

  pub fn canonize_expr(mut self, expr: Expression) -> Result<HlIrModule> {
    self.expr(expr)?;
    Ok(HlIrModule {
      nodes: self.nodes.into_iter().map(|n| n.unwrap()).collect(),
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
      e::FunctionDef {
        export_name,
        arguments,
        argument_spans,
        types,
        body,
      } => {
        if let Some(export) = &export_name {
          if !self.exports.insert(export.clone()) {
            return Err(lint(NameLint::NonUniqueExport, span, &[]));
          }
        }
        self.start_function();
        self.enscope();
        let parameter_names = arguments
          .iter()
          .zip(argument_spans.iter())
          .map(|(name, span)| self.define_name(name, false).span(*span))
          .try_collect::<Vec<_>>()?;
        let parameter_types = types
          .into_iter()
          .map(|t| t.map(|t| self.expr(t)))
          .map(|t| match t {
            Some(Ok(e)) => Ok(Some(e)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
          })
          .try_collect::<Vec<_>>()?;
        let body = self.expr(*body)?;
        let id = self.func_index;
        self.func_index += 1;
        self.descope();
        h::FunctionDef {
          export_name,
          parameter_names,
          parameter_spans: argument_spans,
          parameter_types,
          body,
          id,
        }
      },
      e::Let {
        is_type,
        is_recursive,
        assignee_span,
        assignee,
        value,
        in_,
      } => {
        self.enscope();
        let (assignee, value) = if is_recursive {
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
          is_type,
          is_recursive,
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
        op: BinaryOp::Semicolon,
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
        type_: Type::Any,
      },
    );
    Ok(node)
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
    let path = vec![name.clone()];
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
