use super::*;
use crate::{Span, lint::*, operator::*, parse::*, token::*};
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

#[derive(Debug, Clone)]
pub struct Canonizer {
  nodes: Vec<Option<HlIrNode>>,
  functions: HashMap<Mangle, IrPtr>,
  scope_depth: usize,
  salt: usize,
  path: Vec<String>,
  event_stack: Vec<Event>,
  virtual_memory: Vec<Vec<u8>>,
  main: Option<Mangle>,
  _name_to_symbol: HashMap<String, Symbol>,
}

impl Canonizer {
  fn new() -> Self {
    let mut this = Self {
      nodes: vec![],
      functions: HashMap::new(),
      path: vec![],
      event_stack: vec![],
      virtual_memory: vec![],
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

  pub fn canonize_ast(stmts: Vec<Statement>) -> Result<HlIrModule> {
    let mut this = Self::new();
    let top_node = this.new_node();
    let top_nodes = this.canon_block(stmts)?;
    this.set_node(
      top_node,
      HlIrNode {
        kind: HlIrKind::Block(top_nodes),
        span: Span::default(),
        type_: Type::default(),
      },
    );
    let nodes = this
      .nodes
      .clone()
      .into_iter()
      .map(|ir| ir.unwrap())
      .collect::<Vec<_>>();
    Ok(HlIrModule {
      nodes,
      functions: this.functions,
      heap: this.virtual_memory,
      main: this.main,
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

  fn allocate(&mut self, bytes: &[u8]) -> usize {
    self.virtual_memory.push(bytes.into());
    self.virtual_memory.len() - 1
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
    let mut path = self.path.clone();
    path.push(name.clone());
    let salt = self.next_salt();
    let mangle = mangle_name(path, &salt);
    let old_value = self.name_to_symbol(&name).ok().cloned();
    if let Some(old) = &old_value {
      if old.scope_depth == self.scope_depth && is_constant && old.is_constant {
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

  fn validate_parameters<'a>(
    &mut self,
    error_hint: &str,
    parameters: &Parameters,
  ) -> Result<()> {
    let names = parameters.names.clone();
    let spans = &parameters.spans;
    if let Some(pos) = {
      let mut unique = HashSet::new();
      let mut set = names.iter();
      set.position(move |x| !unique.insert(x))
    } {
      return Err(lint(
        NameLint::ParamRedefinition,
        spans[pos],
        &[error_hint.to_string(), names[pos].clone()],
      ));
    }
    Ok(())
  }

  pub(super) fn canon_block(
    &mut self,
    stmts: Vec<Statement>,
  ) -> Result<Vec<IrPtr>> {
    stmts
      .iter()
      .map(|s| {
        if let StatementKind::Declaration {
          name,
          is_constant: true,
          ..
        } = &s.kind
        {
          self.define_name(name, true).map(|_| {})
        } else {
          Ok(())
        }
      })
      .try_collect::<Vec<_>>()?;
    let v = stmts
      .into_iter()
      .map(|s| self.canon_statement(s))
      .try_collect::<Vec<_>>();
    v
  }

  fn canon_statement(&mut self, stmt: Statement) -> Result<IrPtr> {
    use HlIrKind as k;
    let node = self.new_node();
    let kind = match stmt.kind {
      StatementKind::Declaration {
        name,
        type_,
        value,
        is_constant,
      } => {
        let assignee = if is_constant {
          self.name_to_symbol(&name).span(stmt.span)?.mangle.clone()
        } else {
          self
            .define_name(name.clone(), is_constant)
            .span(stmt.span)?
        };
        let type_assert = if let Some(type_) = type_ {
          Some(self.canon_expr(type_)?)
        } else {
          None
        };
        let value = self.canon_expr(value)?;
        // Hook main
        if is_constant && name == "main" && self.scope_depth == 0 {
          if let HlIrKind::FunctionDef { name, .. } =
            &self.nodes[value].clone().unwrap().kind
          {
            self.main = Some(name.clone());
          } else {
            return Err(lint(NameLint::InvalidMain, stmt.span, &[]));
          }
        }
        k::Declaration {
          assignee,
          is_constant,
          type_assert,
          value,
        }
      },
      StatementKind::Expression(expression) => {
        self.nodes.pop();
        return self.canon_expr(expression);
      },
      StatementKind::Error(diagnostic) => return Err(diagnostic),
    };
    self.set_node(
      node,
      HlIrNode {
        kind,
        span: stmt.span,
        type_: Type::default(),
      },
    );
    Ok(node)
  }

  fn canon_expr(&mut self, expr: Expression) -> Result<IrPtr> {
    use ExpressionKind as e;
    use HlIrKind as k;
    let node = self.new_node();
    let kind = match expr.kind {
      e::Immediate(immediate) => match immediate {
        Immediate::Unit => k::Immediate(ConstValue::Nothing),
        Immediate::Integer(val, base) => k::Immediate(ConstValue::Integer(
          parse_int_literal(&val, base as u32)?,
        )),
        Immediate::Real(val) => {
          k::Immediate(ConstValue::Real(parse_real_literal(&val)?))
        },
        Immediate::String(val) => {
          let bytes = val.into_bytes();
          let address = self.allocate(&bytes);
          k::Immediate(ConstValue::String {
            virtual_address: address,
            length: bytes.len(),
          })
        },
        Immediate::Glyph(val) => k::Immediate(ConstValue::Glyph(val)),
        Immediate::Boolean(val) => k::Immediate(ConstValue::Boolean(val)),
      },
      e::Identifier { name } => {
        let Symbol { mangle, .. } =
          self.name_to_symbol(&name).span(expr.span)?.clone();
        k::Identifier(mangle)
      },
      e::Binary { op, left, right } => {
        let left = self.canon_expr(*left)?;
        let right = self.canon_expr(*right)?;
        k::Binary {
          op,
          opdef: OpDef::default(),
          left,
          right,
        }
      },
      e::Unary { op, child } => {
        let child = self.canon_expr(*child)?;
        k::Unary {
          op,
          opdef: OpDef::default(),
          child,
        }
      },
      e::Parenthesis(expression) => {
        self.nodes.pop();
        return self.canon_expr(*expression);
      },
      e::FunctionDef {
        parameters,
        returns,
        body,
      } => {
        self.start_function();
        let function_mangle = self.define_unique("function");
        self.functions.insert(function_mangle.clone(), node);
        self.enscope();
        self.validate_parameters("Function", &parameters)?;
        let parameter_names = parameters
          .names
          .iter()
          .map(|n| self.define_name(n, false))
          .try_collect::<Vec<_>>()?;
        let parameter_types = parameters
          .types
          .into_iter()
          .map(|e| self.canon_expr(e))
          .try_collect::<Vec<_>>()?;
        let returns = if let Some(returns) = returns {
          Some((
            self.canon_expr(*returns)?,
            self.define_unique("return_type"),
          ))
        } else {
          None
        };
        let body = self.canon_expr(*body)?;
        self.descope();
        k::FunctionDef {
          name: function_mangle,
          parameter_names,
          parameter_types,
          returns,
          body,
        }
      },
      e::FunctionCall { callee, args } => {
        let callee = self.canon_expr(*callee)?;
        let arguments = args
          .into_iter()
          .map(|a| self.canon_expr(a))
          .try_collect::<Vec<_>>()?;
        k::FunctionCall {
          callee,
          callee_name: self.define_unique("callee"),
          arguments,
        }
      },
      e::StructDef(parameters) => {
        let fields = parameters.names.clone();
        let types = parameters
          .types
          .into_iter()
          .map(|t| self.canon_expr(t))
          .try_collect::<Vec<_>>()?;
        k::StructDef { fields, types }
      },
      e::StructLiteral {
        struct_t,
        parameters,
      } => {
        let struct_t = if let Some(struct_t) = struct_t {
          Some((
            self.canon_expr(*struct_t)?,
            self.define_unique("struct_type"),
          ))
        } else {
          None
        };
        let field_names = parameters.names.clone();
        let field_values = parameters
          .types
          .into_iter()
          .map(|t| self.canon_expr(t))
          .try_collect::<Vec<_>>()?;
        k::StructLiteral {
          struct_t,
          field_names,
          field_values,
        }
      },
      e::Field { namespace, field } => {
        let of = self.canon_expr(*namespace)?;
        let e::Identifier { name: index } = field.kind else {
          return Err(lint(NameLint::FieldNotIdent, field.span, &[]));
        };
        k::Field { of, index }
      },
      e::Block(statements) => {
        self.enscope();
        let body = self.canon_block(statements)?;
        self.descope();
        k::Block(body)
      },
      e::If {
        predicate,
        then,
        else_,
      } => {
        let predicate = self.canon_expr(*predicate)?;
        let then = self.canon_expr(*then)?;
        let else_ = if let Some(else_) = else_ {
          Some(self.canon_expr(*else_)?)
        } else {
          None
        };
        k::If {
          predicate,
          then,
          else_,
        }
      },
      e::Loop { parameters, body } => {
        self.enscope();
        self.validate_parameters("Loop", &parameters)?;
        let parameter_names = parameters
          .names
          .iter()
          .map(|n| self.define_name(n, false))
          .try_collect::<Vec<_>>()?;
        let parameter_values = parameters
          .types
          .into_iter()
          .map(|e| self.canon_expr(e))
          .try_collect::<Vec<_>>()?;
        let body = self.canon_expr(*body)?;
        self.descope();
        k::Loop {
          parameter_names,
          parameter_values,
          body,
        }
      },
      e::Break { expr } => {
        let value = if let Some(expr) = expr {
          Some(self.canon_expr(*expr)?)
        } else {
          None
        };
        k::Break(value)
      },
    };
    self.set_node(
      node,
      HlIrNode {
        kind,
        span: expr.span,
        type_: Type::default(),
      },
    );
    Ok(node)
  }
}
