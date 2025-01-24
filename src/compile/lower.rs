use crate::{
  semantic::ir::{Node, NodeKind},
  Immediate as i,
};

use super::{Asm as asm, AsmType as aty, Compiler};

// TODO resolve literal values before this stage?

impl Compiler {
  pub fn lower(&self, node: Node) -> Vec<asm> {
    let mut nodes = vec![];
    use NodeKind::*;
    match node.kind {
      Declaration {
        name,
        mangle,
        is_constant,
        type_assert,
        value,
      } => {}
      Loop {
        names,
        initials,
        body,
      } => todo!(),
      Break { expr } => todo!(),
      Immediate(immediate) => match immediate {
        i::Unit => {}
        i::Integer(string, base) => {
          let int_value = i64::from_str_radix(&string, base as u32).unwrap();
          let node = asm::constant(aty::i64, int_value.to_string());
          nodes.push(node);
        }
        i::Real(r) => {
          let real_value: f64 = r.parse().unwrap();
          let node = asm::constant(aty::f64, real_value.to_string());
          nodes.push(node);
        }
        i::String(_) => todo!(),
        i::Glyph(g) => {
          let node = asm::constant(aty::i32, (g as u32).to_string());
          nodes.push(node);
        }
        i::Boolean(b) => {
          let node = asm::constant(aty::i32, if b { 1 } else { 0 }.to_string());
          nodes.push(node);
        }
      },
      Identifier { name, mangle } => todo!(),
      StructLiteral { names, values } => todo!(),
      BinaryOp {
        op,
        opdef,
        left,
        right,
      } => todo!(),
      UnaryOp { op, opdef, child } => todo!(),
      Field { namespace, index } => todo!(),
      If {
        predicate,
        then,
        else_,
      } => todo!(),
      Call {
        mangle,
        callee,
        params,
      } => todo!(),
      Function {
        mangle,
        arguments,
        nodes,
      } => todo!(),
      Block { nodes } => todo!(),
      Remainder { node } => todo!(),
    };
    nodes
  }
}
