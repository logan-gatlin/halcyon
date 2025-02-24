use std::collections::HashMap;

use crate::{
  Span, diagnostic,
  err::*,
  error,
  ir::{
    ConstValue, IrPtr,
    types::{Primitive, Type},
  },
  naming::{
    build_ir::{parse_int_literal, parse_real_literal},
    mangle_builtin,
  },
  parse::{BinaryOp, Expression, ExpressionKind, Immediate, UnaryOp},
};

use super::{Event, Mangle, Symbol, mangle_name};

#[derive(Debug, Clone)]
pub struct CanonNode {
  pub kind: CanonKind,
  pub span: Span,
  pub type_: Type,
}

#[derive(Debug, Clone)]
pub enum CanonKind {
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
    field_names: Vec<String>,
    field_values: Vec<IrPtr>,
  },
  Binary {
    op: BinaryOp,
    left: IrPtr,
    right: IrPtr,
  },
  Unary {
    op: UnaryOp,
    child: IrPtr,
  },
  FunctionDef {
    parameters: Vec<Mangle>,
    parameter_types: Vec<IrPtr>,
  },
  FunctionCall {
    callee: IrPtr,
    arguments: Vec<IrPtr>,
  },
  If {
    predicate: IrPtr,
    then: IrPtr,
    else_: IrPtr,
  },
  Loop {
    parameters: Vec<Mangle>,
    parameter_values: Vec<IrPtr>,
  },
  Break(Option<IrPtr>),
}

pub struct Canonizer {
  pub ir: Vec<Option<CanonNode>>,
  pub scope_depth: usize,
  pub salt: usize,
  pub path: Vec<String>,
  pub event_stack: Vec<Event>,
  pub heap: Vec<Vec<u8>>,
  // Constant expressions to evaluate
  pub constants: HashMap<Mangle, IrPtr>,
  // Parameter types to evaluate
  pub type_assertions: HashMap<Mangle, IrPtr>,
  pub functions: HashMap<Mangle, IrPtr>,
  pub main: Option<Mangle>,
  pub break_targets: Vec<IrPtr>,
  _name_to_symbol: HashMap<String, Symbol>,
}

impl Canonizer {
  fn canon_expr(&mut self, expr: Expression) -> Result<IrPtr> {
    use CanonKind as k;
    use ExpressionKind as e;
    let kind = match expr.kind {
      e::Immediate(immediate) => match immediate {
        Immediate::Unit => CanonKind::Immediate(ConstValue::Nothing),
        Immediate::Integer(val, base) => CanonKind::Immediate(
          ConstValue::Integer(parse_int_literal(&val, base as u32)?),
        ),
        Immediate::Real(val) => {
          CanonKind::Immediate(ConstValue::Real(parse_real_literal(&val)?))
        },
        Immediate::String(val) => {
          let bytes = val.into_bytes();
          let address = self.allocate(&bytes);
          CanonKind::Immediate(ConstValue::String {
            virtual_address: address,
            length: bytes.len(),
          })
        },
        Immediate::Glyph(val) => CanonKind::Immediate(ConstValue::Glyph(val)),
        Immediate::Boolean(val) => {
          CanonKind::Immediate(ConstValue::Boolean(val))
        },
      },
      e::Identifier { name } => {
        let Symbol { mangle, .. } =
          self.name_to_symbol(&name).span(&expr.span)?.clone();
        k::Identifier(mangle)
      },
      e::Binary { op, left, right } => {
        let node = self.new_node();
        let left = self.canon_expr(*left)?;
        let right = self.canon_expr(*right)?;
        k::Binary { op, left, right }
      },
      e::Unary { op, child } => {
        let child = self.canon_expr(*child)?;
        k::Unary { op, child }
      },
      e::Parenthesis(expression) => return self.canon_expr(*expression),
      e::FunctionDef {
        params,
        returns,
        body,
      } => todo!(),
      e::FunctionCall { callee, args } => {
        let callee = self.canon_expr(*callee)?;
        let arguments = args
          .into_iter()
          .map(|a| self.canon_expr(a))
          .try_collect::<Vec<_>>()?;
        k::FunctionCall { callee, arguments }
      },
      e::StructDef(parameters) => todo!(),
      e::StructLiteral { struct_t, params } => todo!(),
      e::Field { namespace, field } => todo!(),
      e::Block(statements) => todo!(),
      e::If {
        predicate,
        then,
        else_,
      } => todo!(),
      e::Loop { params, body } => todo!(),
      e::Break { expr } => todo!(),
    };
    todo!()
  }

  fn new() -> Self {
    let mut this = Self {
      ir: vec![],
      path: vec![],
      event_stack: vec![],
      heap: vec![],
      constants: HashMap::new(),
      type_assertions: HashMap::new(),
      functions: HashMap::new(),
      scope_depth: 0,
      salt: 0,
      main: None,
      break_targets: vec![],
      _name_to_symbol: HashMap::new(),
    };
    for prim in Primitive::ALL {
      this.define_builtin(format!("{prim}"));
    }
    this.define_builtin(format!("{}", Type::Type));
    this.define_builtin("print_string");
    this
  }

  pub(crate) fn new_node(&mut self) -> IrPtr {
    self.ir.push(None);
    self.ir.len() - 1
  }

  pub(crate) fn set_node(&mut self, position: IrPtr, node: CanonNode) {
    assert!(self.ir[position].is_none());
    self.ir[position] = Some(node);
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
    let path = vec![name.clone()];
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
