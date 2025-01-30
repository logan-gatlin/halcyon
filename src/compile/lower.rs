use crate::{
  Immediate as i,
  err::*,
  error,
  semantic::{
    Mangle, Type,
    consteval::ConstValue,
    ir::{Node, NodeKind},
  },
};

use super::{AsmType as aty, Compiler, Wasm as asm};

impl ConstValue {
  pub fn lower(&self) -> Vec<asm> {
    match self {
      ConstValue::Nothing => vec![],
      ConstValue::Integer(val) => {
        vec![asm::constant(aty::i64, val.to_string())]
      },
      ConstValue::Real(val) => vec![asm::constant(aty::f64, val.to_string())],
      ConstValue::Boolean(val) => {
        vec![asm::constant(aty::i32, (*val as i32).to_string())]
      },
      ConstValue::String(_) => vec![asm::nop],
      ConstValue::Glyph(val) => {
        vec![asm::constant(aty::i32, (*val as u32).to_string())]
      },
      ConstValue::Struct { member_values, .. } => {
        member_values.into_iter().flat_map(|v| v.lower()).collect()
      },
    }
  }
}

// TODO resolve literal values before this stage?

impl Compiler {
  fn new_name(&mut self) -> String {
    let name = format!("$__asm{}", self.unique_salt);
    self.unique_salt += 1;
    name
  }

  fn make_register_init(
    mangle: Mangle,
    type_: &Type,
    regs: &mut Vec<asm>,
    global: bool,
    inits: Vec<asm>,
  ) {
    type_
      .register_types()
      .into_iter()
      .zip(inits.into_iter())
      .enumerate()
      .for_each(|(id, (t, i))| {
        regs.push(asm::reg {
          type_: t,
          ident: format!("${mangle}${id}"),
          global,
          initial: Some(Box::new(i)),
        })
      });
  }

  fn make_register(
    mangle: Mangle,
    type_: &Type,
    regs: &mut Vec<asm>,
    global: bool,
  ) {
    type_
      .register_types()
      .into_iter()
      .enumerate()
      .for_each(|(id, t)| {
        regs.push(asm::reg {
          type_: t,
          ident: format!("${mangle}${id}"),
          global,
          initial: None,
        })
      });
  }

  fn get_register(
    mangle: Mangle,
    type_: &Type,
    instrs: &mut Vec<asm>,
    global: bool,
  ) {
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
  ) -> Result<()> {
    use NodeKind::*;
    match node.kind {
      Declaration {
        mangle,
        value,
        global,
        is_constant,
        ..
      } => {
        if let Type::Function { .. } = value.type_.clone() {
          self.lower(*value.clone(), regs, instrs)?;
        } else if is_constant {
          let consteval = value.constant_evaluate()?.lower();
          Self::make_register_init(
            mangle,
            &value.type_,
            regs,
            global,
            consteval,
          )
        } else {
          Self::make_register(mangle.clone(), &value.type_, regs, global);
          self.lower(*value.clone(), regs, instrs)?;
          Self::set_register(mangle.clone(), &value.type_, instrs, global);
        }
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
        i::String(_) => return error!("Strings are not yet implemented"),
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
        if let Type::Function { .. } = node.type_ {
        } else {
          Self::get_register(mangle, &node.type_, instrs, global);
        }
      },
      BinaryOp {
        opdef, left, right, ..
      } => {
        self.lower(*left, regs, instrs)?;
        self.lower(*right, regs, instrs)?;
        instrs.extend(opdef.asm);
      },
      UnaryOp { opdef, child, .. } => {
        self.lower(*child, regs, instrs)?;
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
        self.lower(*predicate, regs, instrs)?;
        let mut then_block = vec![];
        let mut else_block = vec![];
        self.lower(*then, regs, &mut then_block)?;
        if let Some(else_) = else_ {
          self.lower(*else_, regs, &mut else_block)?;
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
          .map(|p| self.lower(p, regs, instrs))
          .try_collect::<Vec<_>>()?;
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
          .zip(
            param_types
              .into_iter()
              .map(|t| t.unwrap_type_name().unwrap().register_types()),
          )
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
        self.lower(*nodes, &mut regs_local, &mut instrs_local)?;
        regs_local.extend(instrs_local);
        let body = regs_local;
        instrs.push(asm::function {
          ident: mangle,
          params,
          results: return_type.unwrap_type_name().unwrap().register_types(),
          body,
        })
      },
      Block { nodes } => {
        nodes
          .into_iter()
          .map(|n| self.lower(n, regs, instrs))
          .try_collect::<Vec<_>>()?;
      },
      Remainder { node } => {
        self.lower(*node, regs, instrs)?;
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
          .map(|i| self.lower(i, regs, instrs))
          .try_collect::<Vec<_>>()?;
        let block_name = self.new_name();
        self.break_stack.push(block_name.clone());
        let mut loop_body = vec![];
        self.lower(*body, regs, &mut loop_body)?;
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
        self.lower(*expr, regs, instrs)?;
        let current_block = self.break_stack.last().unwrap().clone();
        instrs.push(asm::branch(current_block));
      },
      StructLiteral { names, values } => {
        let Type::Struct {
          member_names: ordered_names,
          ..
        } = &node.type_
        else {
          panic!("Struct does not have struct type");
        };
        for name in ordered_names.iter().rev() {
          let pos = names.iter().position(|n| n == name).unwrap();
          let val = values[pos].clone();
          self.lower(val, regs, instrs)?;
        }
      },
    };
    Ok(())
  }
}
