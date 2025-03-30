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
        exprs.push(*right);
        expr = *left;
      }
      _ => {
        exprs.push(expr);
        exprs.reverse();
        break exprs;
      }
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
      e::Literal(literal) => match literal {
        Literal::Unit => h::Immediate(ConstValue::Nothing),
        Literal::Integer(i, base) => h::Immediate(ConstValue::Integer(
          parse_int_literal(&i, base as u32).span(span)?,
        )),
        Literal::Real(r) => h::Immediate(ConstValue::Real(parse_real_literal(&r).span(span)?)),
        Literal::String(s) => {
          let address = self.memory.static_allocate(s.len() as PtrT);
          h::Immediate(ConstValue::String {
            address,
            length: s.len() as PtrT,
          })
        }
        Literal::Glyph(g) => h::Immediate(ConstValue::Glyph(g)),
        Literal::Boolean(b) => h::Immediate(ConstValue::Boolean(b)),
      },
      e::Identifier(name) => {
        let symbol = self.name_to_symbol(&name).span(span)?;
        h::Identifier(symbol.mangle.clone())
      }
      e::Binary {
        op: op @ (BinaryOp::Equal | BinaryOp::DoubleColon),
        left:
          box Expression {
            kind: e::Identifier(name),
            span: name_span,
          },
        right,
      } => {
        let is_constant = op == BinaryOp::DoubleColon;
        h::Declaration {
          assignee: self.define_name(name, is_constant).span(name_span)?,
          is_constant,
          type_assert: None,
          value: self.expr(*right)?,
        }
      }
      e::Binary {
        op: BinaryOp::Equal | BinaryOp::DoubleColon,
        left,
        ..
      } => return Err(lint(ParseLint::AssignToExpression, left.span, &[])),
      e::Binary {
        op: BinaryOp::Comma,
        ..
      } => {
        let list = collect_list(expr);
        // A list of items could be a struct literal, a type definition, or a tuple.
        // This checks which of the choices it is, and makes sure it is not ambiguous
        let (is_literal, is_definition, is_tuple) =
          list.iter().fold((None, None, None), |acc, x| {
            let is_literal = matches!(x, Expression {
              kind: e::Binary {
                op: BinaryOp::Equal,
                left: box Expression {
                  kind: e::Identifier(_),
                  ..
                },
                ..
              },
              ..
            });
            let is_definition = matches!(x, Expression {
              kind: e::Binary {
                op: BinaryOp::Colon,
                left: box Expression {
                  kind: e::Identifier(_),
                  ..
                },
                ..
              },
              ..
            });
            (
              acc.0.or(if is_literal { Some(x.span) } else { None }),
              acc.1.or(if is_definition { Some(x.span) } else { None }),
              acc.2.or(if !(is_literal || is_definition) {
                Some(x.span)
              } else {
                None
              }),
            )
          });
        match (is_literal, is_definition, is_tuple) {
          // Struct literal
          (Some(_), None, None) => {
            let (field_names, field_values): (Vec<String>, Vec<Expression>) = list
              .into_iter()
              .map(|e| {
                let Expression {
                  kind:
                    e::Binary {
                      op: BinaryOp::Equal,
                      left:
                        box Expression {
                          kind: e::Identifier(name),
                          ..
                        },
                      right,
                    },
                  ..
                } = e
                else {
                  panic!()
                };
                (name, *right)
              })
              .unzip();
            let field_values = field_values
              .into_iter()
              .map(|v| self.expr(v))
              .try_collect::<Vec<_>>()?;
            h::StructLiteral {
              struct_t: None,
              field_names,
              field_values,
            }
          }
          // Struct definition
          (None, Some(_), None) => {
            let (field_names, field_types): (Vec<String>, Vec<Expression>) = list
              .into_iter()
              .map(|e| {
                let Expression {
                  kind:
                    e::Binary {
                      op: BinaryOp::Colon,
                      left:
                        box Expression {
                          kind: e::Identifier(name),
                          ..
                        },
                      right,
                    },
                  ..
                } = e
                else {
                  panic!()
                };
                (name, *right)
              })
              .unzip();
            let field_types = field_types
              .into_iter()
              .map(|v| self.expr(v))
              .try_collect::<Vec<_>>()?;
            h::StructDef {
              field_names,
              field_types,
            }
          }
          // Tuple
          (None, None, Some(_)) => h::Tuple(
            list
              .into_iter()
              .map(|e| self.expr(e))
              .try_collect::<Vec<_>>()?,
          ),
          (s1, s2, s3) => {
            return Err(lint(
              ParseLint::AmbiguousList,
              s1.or(s2).or(s3).unwrap(),
              &[],
            ));
          }
        }
      }
      e::Binary { op, left, right } => h::Binary {
        op,
        opdef: OpDef::default(),
        left: self.expr(*left)?,
        right: self.expr(*right)?,
      },
      e::Unary { op, child } => h::Unary {
        op,
        opdef: OpDef::default(),
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
      e::Block(expressions) => {}
      e::If {
        predicate,
        then,
        else_,
      } => todo!(),
      e::Guard {
        predicates,
        branches,
        else_branch,
      } => todo!(),
      e::Loop { parameters, body } => todo!(),
    };
    self.set_node(node, HlIrNode {
      kind,
      span,
      type_: Type::Ambiguous,
    });
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
        .insert(name.clone(), Symbol {
          mangle,
          scope_depth: 0,
          is_constant: true,
        },)
        .is_none(),
      "Multiple definitions of builtin {name}"
    );
  }

  fn define_name(&mut self, name: impl Into<String>, is_constant: bool) -> Result<Mangle> {
    let name = name.into();
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
    self._name_to_symbol.insert(name.clone(), Symbol {
      mangle: mangle.clone(),
      scope_depth: self.scope_depth,
      is_constant,
    });
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
        }
        Event::Modify { name, old_value } => {
          if let Some(old) = old_value {
            self._name_to_symbol.insert(name, old);
          } else {
            self._name_to_symbol.remove(&name);
          }
        }
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
