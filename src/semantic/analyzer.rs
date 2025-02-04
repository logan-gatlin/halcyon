use std::collections::HashMap;

use crate::{
  Span, Statement, StatementKind, diagnostic,
  err::*,
  error,
  semantic::{ConstValue, primitives::Primitive},
};

use super::{
  Analyzer, Mangle, Type, ir::*, mangle_builtin, operators::OpTable,
};
use NodeKind as n;

#[derive(Debug, Clone)]
pub struct Symbol {
  pub mangle: Mangle,
  pub scope_depth: usize,
  pub is_constant: bool,
}

#[derive(Debug, Clone)]
pub enum Event {
  ScopeStart,
  Modify {
    name: String,
    old_value: Option<Symbol>,
  },
}

impl Analyzer {
  pub fn analyze(block: impl Iterator<Item = Statement>) -> Result<Module> {
    let mut analyzer = Analyzer::new();
    analyzer.analyze_module(block)
  }

  pub fn new() -> Self {
    let mut this = Self {
      scope_depth: 0,
      salt: 0,
      path: vec![],
      _name_to_symbol: HashMap::new(),
      event_stack: vec![],
      op_table: OpTable::new(),
      data_segment: vec![],
      data_offset: 0,
      constants: HashMap::new(),
      main: None,
    };
    this.prelude();
    this
  }

  pub fn static_allocate(&mut self, bytes: &[u8]) -> usize {
    let old_offset = self.data_offset;
    self.data_offset += bytes.len();
    self.data_segment.extend(bytes);
    old_offset
  }

  pub fn prelude(&mut self) {
    // Define primitive types
    let mut define_type = |name: String, value: Primitive| {
      let mangle = mangle_builtin(&name);
      self._name_to_symbol.insert(
        name,
        Symbol {
          mangle: mangle.clone(),
          scope_depth: 0,
          is_constant: true,
        },
      );
      self.constants.insert(
        mangle,
        Node {
          span: Span { row: 0, column: 0 },
          type_: Type::Type,
          kind: n::ConstValue(ConstValue::Type(Type::Prim(value))),
        },
      )
    };
    for p in Primitive::ALL {
      define_type(p.to_string(), p);
    }
    // Primitive standard library
    const PRINT_STRING: &str = "print_string";
    self._name_to_symbol.insert(
      PRINT_STRING.into(),
      Symbol {
        mangle: mangle_builtin(PRINT_STRING),
        scope_depth: 0,
        is_constant: true,
      },
    );
  }

  pub(crate) fn name_to_symbol(&self, name: &str) -> Result<&Symbol> {
    self
      ._name_to_symbol
      .get(name)
      .ok_or(diagnostic!("The symbol '{name}' is undefined"))
  }

  fn next_salt(&mut self) -> String {
    let returned_salt = self.salt.to_string();
    self.salt += 1;
    returned_salt
  }

  pub fn define_name(
    &mut self,
    name: impl Into<String>,
    is_constant: bool,
  ) -> Result<Mangle> {
    let name = name.into();
    let mut path = self.path.clone();
    path.push(name.clone());
    let salt = self.next_salt();
    let mangle = super::mangle_name(path, &salt);
    let old_value = self.name_to_symbol(&name).ok().cloned();
    if let Some(old) = &old_value {
      if old.scope_depth == self.scope_depth && is_constant && old.is_constant {
        return error!("Conflicting definitions of '{name}' in the same scope");
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

  pub fn define_anonymous(&mut self) -> Mangle {
    let name = String::from("anon");
    let mut path = self.path.clone();
    path.push(name.clone());
    let salt = self.next_salt();
    let mangle = super::mangle_name(path, &salt);
    mangle
  }

  pub fn enscope(&mut self) {
    self.event_stack.push(Event::ScopeStart);
    self.scope_depth += 1;
  }

  pub fn descope(&mut self) {
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

  pub fn start_function(&mut self) {
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

  pub fn analyze_module(
    &mut self,
    block: impl Iterator<Item = Statement>,
  ) -> Result<Module> {
    let Node {
      kind: NodeKind::Block { mut nodes },
      ..
    } = self.analyze_scope(block)?
    else {
      panic!()
    };
    nodes
      .iter_mut()
      .map(|n| {
        if let NodeKind::Lifted = n.kind {
          Ok(())
        } else {
          error!("Only constant declarations are allowed in global scope")
            .span(&n.span)
        }
      })
      .try_collect::<Vec<_>>()?;
    Ok(Module {
      data: self.data_segment.clone(),
      constants: self.constants.clone(),
      main: self.main.clone(),
    })
  }

  pub fn analyze_scope(
    &mut self,
    block: impl Iterator<Item = Statement>,
  ) -> Result<Node> {
    let nodes = block
      // Pass 1 - define constant names
      .map(|stmt| {
        if let StatementKind::Declaration {
          name,
          is_constant: true,
          ..
        } = &stmt.kind
        {
          self.define_name(name, true).span(&stmt.span).map(|_| stmt)
        } else {
          Ok(stmt)
        }
      })
      .try_collect::<Vec<_>>()?
      .into_iter()
      // Pass 2 - construct ir
      .map(|stmt| self.analyze_statement(stmt))
      .try_collect::<Vec<Node>>()?
      .into_iter()
      .filter(|n| {
        if let NodeKind::Lifted = n.kind {
          false
        } else {
          true
        }
      })
      .collect::<Vec<_>>();
    let mut span = Span { row: 0, column: 0 };
    for n in nodes.iter() {
      span = span + n.span;
    }
    Ok(Node {
      span,
      type_: Type::Ambiguous,
      kind: NodeKind::Block { nodes },
    })
  }

  pub fn analyze_statement(&mut self, stmt: Statement) -> Result<Node> {
    use StatementKind as s;
    Ok(match stmt.kind {
      s::Declaration {
        name,
        type_,
        value,
        is_constant,
      } => {
        let value = self.analyze_expression(value)?;
        let mangle = if is_constant {
          self.name_to_symbol(&name).map(|s| s.mangle.clone())
        } else {
          self.define_name(name.clone(), false)
        }
        .span(&stmt.span)?;
        if name == "main" && self.scope_depth == 0 && is_constant {
          self.main = Some(mangle.clone());
        }
        let type_assert = if let Some(type_) = type_ {
          let type_actual = self.analyze_expression(type_)?;
          Some(type_actual.into())
        } else {
          None
        };
        if is_constant {
          self.constants.insert(mangle, value);
          Node {
            span: stmt.span,
            type_: Primitive::nothing.promote(),
            kind: n::Lifted,
          }
        } else {
          Node {
            span: stmt.span,
            type_: Type::Prim(Primitive::nothing),
            kind: n::Declaration {
              name,
              global: self.scope_depth == 0,
              mangle,
              type_assert,
              value: value.into(),
            },
          }
        }
      },
      s::Expression(expression) => self.analyze_expression(expression)?,
      s::Remainder(expression) => {
        let node = self.analyze_expression(expression)?;
        Node {
          span: stmt.span,
          type_: node.type_.clone(),
          kind: n::Remainder { node: node.into() },
        }
      },
      s::Error(diagnostic) => return Err(diagnostic),
    })
  }
}
