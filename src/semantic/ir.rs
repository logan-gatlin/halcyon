use operators::OpDef;
//use operators::OpDef;

use crate::{BinaryOp, Span, UnaryOp};

use super::*;

pub fn parse_int_literal(value: &str, base: u32) -> Result<i64> {
  i64::from_str_radix(value, base)
    .reason(format!("Failed to parse integer literal '{value}'"))
}

pub fn parse_real_literal(value: &str) -> Result<f64> {
  value
    .parse()
    .ok()
    .reason(format!("Failed to parse real literal '{value}'"))
}

#[derive(Clone, Debug)]
pub enum ConstValue {
  Nothing,
  Integer(i64),
  Real(f64),
  Boolean(bool),
  String {
    address: usize,
    length: usize,
  },
  Glyph(char),
  Function(Mangle),
  StructLiteral {
    member_names: Vec<String>,
    member_values: Vec<ConstValue>,
  },
  Type(Type),
}

#[derive(Debug, Clone)]
pub struct Module {
  pub heap: Vec<Vec<u8>>,
  pub constants: HashMap<Mangle, Node>,
  pub main: Option<Mangle>,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
  Loop {
    names: Vec<Mangle>,
    initials: Vec<Node>,
    body: Box<Node>,
  },
  Break {
    expr: Box<Node>,
  },
  ConstValue(ConstValue),
  Identifier {
    name: String,
    constant: bool,
    mangle: Mangle,
  },
  StructDef {
    mangle: String,
    member_names: Vec<String>,
    member_types: Vec<Node>,
  },
  StructLiteral {
    struct_t: Box<Node>,
    param_names: Vec<String>,
    param_values: Vec<Node>,
  },
  BinaryOp {
    op: BinaryOp,
    opdef: OpDef,
    left: Box<Node>,
    right: Box<Node>,
  },
  UnaryOp {
    op: UnaryOp,
    opdef: OpDef,
    child: Box<Node>,
  },
  Field {
    namespace: Box<Node>,
    index: String,
  },
  If {
    predicate: Box<Node>,
    then: Box<Node>,
    else_: Option<Box<Node>>,
  },
  Call {
    callee: Box<Node>,
    params: Vec<Node>,
  },
  Function {
    mangle: Mangle,
    param_mangles: Vec<Mangle>,
    param_types: Vec<Node>,
    returns: Box<Node>,
    nodes: Box<Node>,
  },
  Declaration {
    name: String,
    global: bool,
    mangle: Mangle,
    type_assert: Option<Box<Node>>,
    value: Box<Node>,
  },
  Block {
    nodes: Vec<Node>,
  },
  Remainder {
    node: Box<Node>,
  },
  /// Constant declaration that got lifted to global scope
  Lifted,
}

#[derive(Debug, Clone)]
pub struct Node {
  pub span: Span,
  pub type_: Type,
  pub kind: NodeKind,
}

impl Node {
  pub fn map(&self, op: &mut impl FnMut(&Self) -> Result<()>) -> Result<()> {
    use NodeKind as n;
    let mut it = |n: &Node| n.map(op);
    match &self.kind {
      n::Loop {
        names,
        initials,
        body,
      } => {
        initials.into_iter().try_for_each(|i| it(i))?;
        it(body)?;
      },
      n::Break { expr } => it(expr)?,
      n::ConstValue(const_value) => {},
      n::Identifier {
        name,
        constant,
        mangle,
      } => {},
      n::StructDef {
        mangle,
        member_names,
        member_types,
      } => member_types.into_iter().try_for_each(|t| it(t))?,
      n::StructLiteral {
        struct_t,
        param_names,
        param_values,
      } => {
        it(struct_t)?;
        param_values.into_iter().try_for_each(|v| it(v))?
      },
      n::BinaryOp { left, right, .. } => {
        it(left)?;
        it(right)?;
      },
      n::UnaryOp { child, .. } => it(child)?,
      n::Field { namespace, index } => it(namespace)?,
      n::If {
        predicate,
        then,
        else_,
      } => {
        it(predicate)?;
        it(then)?;
        if let Some(else_) = else_ {
          it(else_)?;
        }
      },
      n::Call { callee, params } => {
        it(callee)?;
        params.into_iter().try_for_each(|p| it(p))?;
      },
      n::Function {
        mangle,
        param_mangles,
        param_types,
        returns,
        nodes,
      } => {
        param_types.into_iter().try_for_each(|p| it(p))?;
        it(returns)?;
        it(nodes)?;
      },
      n::Declaration {
        name,
        global,
        mangle,
        type_assert,
        value,
      } => {
        if let Some(type_assert) = type_assert {
          it(type_assert)?;
        }
        it(value)?;
      },
      n::Block { nodes } => {
        nodes.into_iter().try_for_each(|n| it(n))?;
      },
      n::Remainder { node } => it(node)?,
      n::Lifted => {},
    };
    drop(it);
    op(self)?;
    Ok(())
  }

  pub fn map_mut(
    &mut self,
    op: &mut impl FnMut(&mut Self) -> Result<()>,
  ) -> Result<()> {
    use NodeKind as n;
    let mut it = |n: &mut Node| n.map_mut(op);
    match &mut self.kind {
      n::Loop {
        names,
        initials,
        body,
      } => {
        initials.into_iter().try_for_each(|i| it(i))?;
        it(body)?;
      },
      n::Break { expr } => it(expr)?,
      n::ConstValue(const_value) => {},
      n::Identifier {
        name,
        constant,
        mangle,
      } => {},
      n::StructDef {
        mangle,
        member_names,
        member_types,
      } => member_types.into_iter().try_for_each(|t| it(t))?,
      n::StructLiteral {
        struct_t,
        param_names,
        param_values,
      } => {
        it(struct_t)?;
        param_values.into_iter().try_for_each(|v| it(v))?
      },
      n::BinaryOp { left, right, .. } => {
        it(left)?;
        it(right)?;
      },
      n::UnaryOp { child, .. } => it(child)?,
      n::Field { namespace, index } => it(namespace)?,
      n::If {
        predicate,
        then,
        else_,
      } => {
        it(predicate)?;
        it(then)?;
        if let Some(else_) = else_ {
          it(else_)?;
        }
      },
      n::Call { callee, params } => {
        it(callee)?;
        params.into_iter().try_for_each(|p| it(p))?;
      },
      n::Function {
        mangle,
        param_mangles,
        param_types,
        returns,
        nodes,
      } => {
        param_types.into_iter().try_for_each(|p| it(p))?;
        it(returns)?;
        it(nodes)?;
      },
      n::Declaration {
        name,
        global,
        mangle,
        type_assert,
        value,
      } => {
        if let Some(type_assert) = type_assert {
          it(type_assert)?;
        }
        it(value)?;
      },
      n::Block { nodes } => {
        nodes.into_iter().try_for_each(|n| it(n))?;
      },
      n::Remainder { node } => it(node)?,
      n::Lifted => {},
    };
    drop(it);
    op(self)?;
    Ok(())
  }
}
