pub mod builtins;
pub mod canon;
pub mod control_flow;

use std::collections::HashMap;

use builtins::Builtin;

use crate::Span;
use crate::assembly::operators::OpDef;
use crate::diagnostic;
use crate::err::*;
use crate::error;
use crate::ir::ConstValue;
use crate::ir::IrPtr;
use crate::ir::types::Primitive;
use crate::ir::types::Type;
use crate::parse::BinaryOp;
use crate::parse::Statement;
use crate::parse::UnaryOp;

pub type Mangle = String;

/// Name mangle syntax:
/// mangle ::= "$" path salt
/// path ::= {path-element}*
/// path-element ::= length ident
/// ident ::= _a-zA-Z {_a-zA-Z0-9}*
/// length ::= {0-9}+
/// salt ::= {a-zA-Z}*
pub fn mangle_name(path: Vec<String>, salt: &str) -> Mangle {
  let mut buf: Vec<u8> = vec![];
  for p in path {
    let puny = punycode::encode(&p).unwrap();
    let bytes = format!("{}{puny}", puny.len());
    buf.extend_from_slice(bytes.as_bytes());
  }
  buf.extend_from_slice(salt.as_bytes());
  String::from_utf8(buf).unwrap()
}

/// Builtin mangle syntax:
/// "$" {ident}
pub fn mangle_builtin(name: impl std::fmt::Display) -> Mangle {
  format!("_{name}")
}

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
#[derive(Debug, Clone)]
pub struct CanonNode {
  pub kind: CanonKind,
  pub span: Span,
  pub type_: Type,
}

#[derive(Debug, Clone)]
pub enum CanonKind {
  Remainder(IrPtr),
  Declaration {
    assignee: Mangle,
    is_constant: bool,
    type_assert: Option<IrPtr>,
    value: IrPtr,
  },
  Immediate(ConstValue),
  Block(Vec<IrPtr>),
  Identifier(Mangle),
  StructDef {
    fields: Vec<String>,
    types: Vec<IrPtr>,
  },
  StructLiteral {
    struct_t: Option<(IrPtr, Mangle)>,
    field_names: Vec<String>,
    field_values: Vec<IrPtr>,
  },
  Field {
    of: IrPtr,
    index: String,
  },
  Binary {
    op: BinaryOp,
    opdef: OpDef,
    left: IrPtr,
    right: IrPtr,
  },
  Unary {
    op: UnaryOp,
    opdef: OpDef,
    child: IrPtr,
  },
  FunctionDef {
    name: Mangle,
    parameter_names: Vec<Mangle>,
    parameter_types: Vec<IrPtr>,
    returns: Option<(IrPtr, Mangle)>,
    body: IrPtr,
  },
  FunctionCall {
    callee: IrPtr,
    callee_name: Mangle,
    arguments: Vec<IrPtr>,
  },
  If {
    predicate: IrPtr,
    then: IrPtr,
    else_: Option<IrPtr>,
  },
  Loop {
    parameter_names: Vec<Mangle>,
    parameter_values: Vec<IrPtr>,
    body: IrPtr,
  },
  Break(Option<IrPtr>),
}

pub struct CanonizedModule {
  pub nodes: Vec<CanonNode>,
  pub functions: HashMap<Mangle, IrPtr>,
  pub heap: Vec<Vec<u8>>,
  pub main: Option<Mangle>,
}

#[derive(Debug, Clone)]
pub struct Canonizer {
  pub nodes: Vec<Option<CanonNode>>,
  pub functions: HashMap<Mangle, IrPtr>,
  pub scope_depth: usize,
  pub salt: usize,
  pub path: Vec<String>,
  pub event_stack: Vec<Event>,
  pub heap: Vec<Vec<u8>>,
  pub main: Option<Mangle>,
  _name_to_symbol: HashMap<String, Symbol>,
}

impl Canonizer {
  fn new() -> Self {
    let mut this = Self {
      nodes: vec![],
      functions: HashMap::new(),
      path: vec![],
      event_stack: vec![],
      heap: vec![],
      scope_depth: 0,
      salt: 0,
      main: None,
      _name_to_symbol: HashMap::new(),
    };
    for prim in Primitive::ALL {
      this.define_builtin(format!("{prim}"));
    }
    for builtin in Builtin::ALL {
      this.define_builtin(builtin.to_string())
    }
    this
  }

  pub fn canonize_ast(stmts: Vec<Statement>) -> Result<CanonizedModule> {
    let mut this = Self::new();
    let top_node = this.new_node();
    let top_nodes = this.canon_block(stmts)?;
    this.set_node(
      top_node,
      CanonNode {
        kind: CanonKind::Block(top_nodes),
        span: Span { row: 0, column: 0 },
        type_: Type::default(),
      },
    );
    this
      .nodes
      .clone()
      .into_iter()
      .map(|ir| ir.ok_or(diagnostic!("Empty node in IR array")))
      .try_collect::<Vec<_>>()
      .map(|m| CanonizedModule {
        nodes: m,
        functions: this.functions,
        heap: this.heap,
        main: this.main,
      })
  }

  pub(crate) fn new_node(&mut self) -> IrPtr {
    self.nodes.push(None);
    self.nodes.len() - 1
  }

  pub(crate) fn set_node(&mut self, position: IrPtr, node: CanonNode) {
    assert!(self.nodes[position].is_none());
    self.nodes[position] = Some(node);
  }

  pub(crate) fn name_to_symbol(&self, name: &str) -> Result<&Symbol> {
    self
      ._name_to_symbol
      .get(name)
      .ok_or(diagnostic!("The symbol '{name}' is undefined"))
  }

  pub(crate) fn next_salt(&mut self) -> String {
    let returned_salt = self.salt.to_string();
    self.salt += 1;
    returned_salt
  }

  pub(crate) fn allocate(&mut self, bytes: &[u8]) -> usize {
    self.heap.push(bytes.into());
    self.heap.len() - 1
  }

  pub(crate) fn define_unique(&mut self, hint: &str) -> Mangle {
    let name = String::from(hint);
    let mut path = self.path.clone();
    path.push(name.clone());
    let salt = self.next_salt();
    let mangle = mangle_name(path, &salt);
    mangle
  }

  pub(crate) fn define_builtin(&mut self, name: impl Into<String>) {
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

  pub(crate) fn define_name(
    &mut self,
    name: impl Into<String>,
    is_constant: bool,
  ) -> Result<Mangle> {
    let name = name.into();
    let mut path = self.path.clone();
    path.push(name.clone());
    let salt = self.next_salt();
    let mangle = mangle_name(path, &salt);
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

  pub(crate) fn enscope(&mut self) {
    self.event_stack.push(Event::ScopeStart);
    self.scope_depth += 1;
  }

  pub(crate) fn descope(&mut self) {
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

  pub(crate) fn start_function(&mut self) {
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
