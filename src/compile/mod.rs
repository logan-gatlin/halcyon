use crate::semantic::primitives::Primitive;
use crate::semantic::Type;
use crate::{err::*, BinaryOp};
use crate::{
  semantic::ir::{Node, NodeKind},
  Immediate,
};

pub struct Compiler {}

impl Compiler {
  pub fn compile(&mut self, node: Node, buffer: &mut String) -> Result<()> {
    let mut push = |kind, value| {
      buffer.push_str(&format!("{}.const {}\n", kind, value));
    };
    use NodeKind as n;
    match node.kind {
      n::Immediate(immediate) => match immediate {
        Immediate::Unit => {}
        Immediate::Integer(s, base) => {
          let value = i64::from_str_radix(&s, base as u32)?;
          push("i64", value.to_string());
        }
        Immediate::Real(r) => {
          let value: f64 = r
            .parse()
            .ok()
            .reason(format!("Failed to parse float '{r}'\n"))
            .span(&node.span)?;
          push("f64", value.to_string());
        }
        Immediate::Glyph(c) => push("i32", c.to_string()),
        Immediate::String(_) => {
          todo!()
        }
        Immediate::Boolean(b) => push("i32", if b { "1" } else { "2" }.to_string()),
      },
      n::BinaryOp {
        op,
        opdef,
        left,
        right,
      } => {
        use BinaryOp as o;
        use Primitive as p;
        use Type::Prim as ty;
        match (left.type_, op, right.type_) {
          _ => panic!(),
        }
      }
      n::UnaryOp { op, opdef, child } => todo!(),
      n::Identifier { name, mangle } => todo!(),
      n::StructLiteral { names, values } => todo!(),
      n::Field { namespace, index } => todo!(),
      n::If {
        predicate,
        then,
        else_,
      } => todo!(),
      n::Call {
        mangle,
        callee,
        params,
      } => todo!(),
      n::Function {
        mangle,
        arguments,
        nodes,
      } => todo!(),
      n::Declaration {
        name,
        mangle,
        is_constant,
        type_assert,
        value,
      } => todo!(),
      n::Block { nodes } => todo!(),
      n::Remainder { node } => todo!(),
      n::Loop {
        names,
        initials,
        body,
      } => todo!(),
      n::Break { expr } => todo!(),
    }
    Ok(())
  }
}
