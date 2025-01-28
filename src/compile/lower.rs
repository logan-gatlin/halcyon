use crate::{
  Immediate as i,
  semantic::{
    Mangle, Type,
    ir::{Node, NodeKind},
  },
};

use super::{Asm as asm, AsmType as aty, Compiler};

// TODO resolve literal values before this stage?

impl Compiler {
  fn new_name(&mut self) -> String {
    let name = format!("$__asm{}", self.unique_salt);
    self.unique_salt += 1;
    name
  }

  fn make_register(
    mangle: Mangle,
    type_: &Type,
    regs: &mut Vec<asm>,
    global: bool,
  ) {
    regs.push(asm::comment(format!("declare {mangle}")));
    type_
      .register_types()
      .into_iter()
      .enumerate()
      .for_each(|(id, t)| {
        regs.push(asm::reg {
          type_: t,
          ident: format!("${mangle}${id}"),
          global,
        })
      });
  }

  fn get_register(
    mangle: Mangle,
    type_: &Type,
    instrs: &mut Vec<asm>,
    global: bool,
  ) {
    instrs.push(asm::comment(format!("get {mangle}")));
    type_
      .register_types()
      .into_iter()
      .rev()
      .enumerate()
      .for_each(|(id, _)| {
        instrs.push(asm::regget {
          ident: format!("${mangle}${id}"),
          global,
        })
      });
  }

  fn set_register(
    mangle: Mangle,
    type_: &Type,
    instrs: &mut Vec<asm>,
    global: bool,
  ) {
    instrs.push(asm::comment(format!("set {mangle}")));
    type_
      .register_types()
      .into_iter()
      .enumerate()
      .for_each(|(id, _)| {
        instrs.push(asm::regset {
          ident: format!("${mangle}${id}"),
          global,
        })
      });
  }

  pub fn lower(
    &mut self,
    node: Node,
    regs: &mut Vec<asm>,
    instrs: &mut Vec<asm>,
  ) {
    use NodeKind::*;
    match node.kind {
      Declaration {
        mangle,
        value,
        global,
        ..
      } => {
        Self::make_register(mangle.clone(), &node.type_.clone(), regs, global);
        self.lower(*value, regs, instrs);
        Self::set_register(mangle.clone(), &node.type_.clone(), instrs, global);
      },
      Immediate(immediate) => match immediate {
        i::Unit => {},
        i::Integer(string, base) => {
          let int_value = i64::from_str_radix(&string, base as u32).unwrap();
          let node = asm::constant(aty::i64, int_value.to_string());
          instrs.push(node);
        },
        i::Real(r) => {
          let real_value: f64 = r.parse().unwrap();
          let node = asm::constant(aty::f64, real_value.to_string());
          instrs.push(node);
        },
        i::String(_) => todo!(),
        i::Glyph(g) => {
          let node = asm::constant(aty::i32, (g as u32).to_string());
          instrs.push(node);
        },
        i::Boolean(b) => {
          let node = asm::constant(aty::i32, if b { 1 } else { 0 }.to_string());
          instrs.push(node);
        },
      },
      Identifier { mangle, global, .. } => {
        Self::get_register(mangle, &node.type_, instrs, global);
      },
      BinaryOp {
        opdef, left, right, ..
      } => {
        self.lower(*left, regs, instrs);
        self.lower(*right, regs, instrs);
        instrs.extend(opdef.asm);
      },
      UnaryOp { opdef, child, .. } => {
        self.lower(*child, regs, instrs);
        instrs.extend(opdef.asm);
      },
      Field { namespace, index } => {
        todo!()
      },
      If {
        predicate,
        then,
        else_,
      } => {
        self.lower(*predicate, regs, instrs);
        let mut then_block = vec![];
        let mut else_block = vec![];
        self.lower(*then, regs, &mut then_block);
        if let Some(else_) = else_ {
          self.lower(*else_, regs, &mut else_block);
        }
        instrs.push(asm::ifelse {
          then: then_block,
          else_: else_block,
        });
      },
      Call { mangle, params, .. } => {
        // TODO perform in expected left->right order here
        params
          .into_iter()
          .rev()
          .for_each(|p| self.lower(p, regs, instrs));
        instrs.push(asm::call(mangle))
      },
      Function {
        mangle,
        param_mangles,
        nodes,
      } => {
        let Type::Function {
          param_types,
          return_type,
          ..
        } = node.type_
        else {
          panic!("Function does not have function type");
        };
        let params = param_mangles
          .into_iter()
          .zip(param_types.into_iter().map(|t| t.register_types()))
          .flat_map(|(mangle, types)| {
            types
              .into_iter()
              .enumerate()
              .map(|(id, t)| (format!("${mangle}${id}"), t))
              .collect::<Vec<_>>()
          })
          .collect::<Vec<_>>();
        let mut regs_local = vec![];
        let mut instrs_local = vec![];
        self.lower(*nodes, &mut regs_local, &mut instrs_local);
        regs_local.extend(instrs_local);
        let body = regs_local;
        instrs.push(asm::function {
          ident: mangle,
          params,
          results: return_type.register_types(),
          body,
        })
      },
      Block { nodes } => {
        nodes.into_iter().for_each(|n| self.lower(n, regs, instrs))
      },
      Remainder { node } => {
        self.lower(*node, regs, instrs);
      },
      Loop {
        names,
        initials,
        body,
      } => {
        names
          .into_iter()
          .zip(initials.iter().map(|i| i.type_.clone()))
          .for_each(|(name, type_)| {
            Self::make_register(name, &type_, regs, false)
          });
        initials
          .into_iter()
          .rev()
          .for_each(|i| self.lower(i, regs, instrs));
        let block_name = self.new_name();
        self.break_stack.push(block_name.clone());
        let mut loop_body = vec![];
        self.lower(*body, regs, &mut loop_body);
        self.break_stack.pop();
        let loop_name = self.new_name();
        loop_body.push(asm::branch(loop_name.clone()));
        let asm_loop = asm::loop_ {
          name: loop_name,
          body: loop_body,
        };
        let asm_block = asm::block {
          name: block_name,
          body: vec![asm_loop],
        };
        instrs.push(asm_block);
      },
      Break { expr } => {
        self.lower(*expr, regs, instrs);
        let current_block = self.break_stack.last().unwrap().clone();
        instrs.push(asm::branch(current_block));
      },
      StructLiteral { names, values } => todo!(),
    };
  }
}
