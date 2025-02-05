use crate::semantic::{Node, NodeKind};
use crate::{err::*, error};

use super::ExWasm;

pub fn lower(node: Node) -> Result<Vec<ExWasm>> {
  use ExWasm as a;
  let mut regs = vec![];
  let mut instrs = vec![];
  node.map(&mut |node| {
    use NodeKind as n;
    match &node.kind {
      n::Loop {
        names,
        initials,
        body,
      } => todo!(),
      n::Break { expr } => todo!(),
      n::ConstValue(const_value) => {
        instrs.push(a::push(const_value.clone()));
        todo!();
      }
      n::Identifier {
        name,
        constant,
        mangle,
      } => {
        return error!("Unresolved identifier '{name}' when lowering. This should not happen!")
          .span(&node.span)
      }
      n::StructDef {
        mangle,
        member_names,
        member_types,
      } => todo!(),
      n::StructLiteral {
        struct_t,
        param_names,
        param_values,
      } => todo!(),
      n::BinaryOp {
        op,
        opdef,
        left,
        right,
      } => todo!(),
      n::UnaryOp { op, opdef, child } => todo!(),
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
        param_mangles,
        param_types,
        returns,
        nodes,
      } => todo!(),
      n::Declaration {
        name,
        global,
        mangle,
        type_assert,
        value,
      } => todo!(),
      n::Block { nodes } => todo!(),
      n::Remainder { node } => todo!(),
      n::Lifted => todo!(),
    }
    Ok(())
  })?;
  regs.extend(instrs);
  Ok(regs)
}
