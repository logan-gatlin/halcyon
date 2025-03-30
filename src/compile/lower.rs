use crate::{compile::*, hlir::*, lint::*, mlir::*};

use Wasm as asm;
use WasmType as aty;

use super::Compiler;
impl Compiler {
  fn new_name(&mut self) -> String {
    let name = format!("_{}", self.unique_salt);
    self.unique_salt += 1;
    name
  }

  fn make_register(mangle: Mangle, type_: &Type, regs: &mut Vec<asm>) {
    type_
      .register_types()
      .into_iter()
      .enumerate()
      .for_each(|(id, t)| regs.push(asm::Local(t, format!("{mangle}${id}"))));
  }

  fn get_register(mangle: Mangle, type_: &Type, instrs: &mut Vec<asm>) {
    type_
      .register_types()
      .into_iter()
      .enumerate()
      .for_each(|(id, _)| instrs.push(asm::LocalGet(format!("{mangle}${id}"))));
  }

  fn set_register(mangle: Mangle, type_: &Type, instrs: &mut Vec<asm>) {
    type_
      .register_types()
      .into_iter()
      .enumerate()
      .rev()
      .for_each(|(id, _)| instrs.push(asm::LocalSet(format!("{mangle}${id}"))));
  }

  pub fn lower(&mut self, node: IrPtr, regs: &mut Vec<asm>, instrs: &mut Vec<asm>) -> Result<()> {
    use HlIrKind::*;
    let type_ = self.module.type_of(node);
    match self.module.nodes[node].clone().kind {
      Declaration {
        assignee,
        value,
        is_constant,
        ..
      } => {
        if !is_constant {
          Self::make_register(assignee.clone(), &self.module.type_of(value), regs);
          self.lower(value, regs, instrs)?;
          Self::set_register(assignee.clone(), &self.module.type_of(value), instrs);
        }
      }
      Immediate(im) => instrs.extend(im.to_wasm_value().into_iter().map(|v| asm::Constant(v))),
      Identifier(mangle) => {
        if let Type::Function { .. } = type_ {
        } else {
          Self::get_register(mangle, &type_, instrs);
        }
      }
      Binary {
        opdef, left, right, ..
      } => {
        self.lower(left, regs, instrs)?;
        self.lower(right, regs, instrs)?;
        instrs.extend(opdef.asm);
      }
      Unary { opdef, child, .. } => {
        self.lower(child, regs, instrs)?;
        instrs.extend(opdef.asm);
      }
      Field { of, index } => {
        let Type::Struct {
          member_names,
          member_types,
          ..
        } = self.module.type_of(of)
        else {
          panic!("Field of non-struct accessed");
        };
        self.lower(of, regs, instrs)?;
        let pos = member_names.iter().position(|n| n == &index).unwrap();
        let temporary_name = self.new_name();
        for (id, t) in member_types.into_iter().enumerate() {
          // Drop everything that comes before and after
          if id != pos {
            (0..t.count_registers()).for_each(|_| instrs.push(asm::Drop))
          }
          // Save desired value to register
          else {
            Self::make_register(temporary_name.clone(), &type_, regs);
            Self::set_register(temporary_name.clone(), &type_, instrs);
          }
        }
        Self::get_register(temporary_name, &type_, instrs);
      }
      FunctionCall {
        callee, arguments, ..
      } => {
        let name = match self.module.nodes[callee].kind.clone() {
          FunctionDef { name, .. }
          | Immediate(ConstValue::Function { name, .. })
          | Identifier(name) => name,
          _ => panic!(),
        };
        arguments
          .into_iter()
          .map(|p| self.lower(p, regs, instrs))
          .try_collect::<Vec<_>>()?;
        instrs.push(asm::Call(name.clone()))
      }
      FunctionDef {
        name,
        parameter_names,
        body,
        ..
      } => {
        let Type::Function {
          param_types,
          return_type,
          ..
        } = type_
        else {
          panic!("Function does not have function type");
        };
        let params = parameter_names
          .into_iter()
          .zip(param_types.into_iter().map(|t| t.register_types()))
          .flat_map(|(mangle, types)| {
            types
              .into_iter()
              .enumerate()
              .map(|(id, t)| (format!("{mangle}${id}"), t))
              .collect::<Vec<_>>()
          })
          .collect::<Vec<_>>();
        let mut regs_local = vec![];
        let mut instrs_local = vec![];
        self.lower(body, &mut regs_local, &mut instrs_local)?;
        instrs_local.push(asm::Return);
        regs_local.extend(instrs_local);
        let body = regs_local;
        instrs.push(asm::Function {
          ident: format!("{name}"),
          params,
          results: return_type.register_types(),
          body,
        })
      }
      Block(nodes) => {
        nodes
          .into_iter()
          .map(|n| self.lower(n, regs, instrs))
          .try_collect::<Vec<_>>()?;
      }
      If {
        predicate,
        then,
        else_,
      } => {
        self.lower(predicate, regs, instrs)?;
        let result_name = if type_.count_registers() == 0 {
          "".to_string()
        } else {
          self.new_name()
        };
        Self::make_register(result_name.clone(), &type_, regs);
        instrs.push(asm::If);
        self.lower(then, regs, instrs)?;
        Self::set_register(result_name.clone(), &type_, instrs);
        if let Some(else_) = else_ {
          instrs.push(asm::Else);
          self.lower(else_, regs, instrs)?;
          Self::set_register(result_name.clone(), &type_, instrs);
        }
        instrs.push(asm::End);
        Self::get_register(result_name, &type_, instrs);
      }
      Loop {
        parameter_names,
        parameter_values,
        parameter_spans,
        body,
      } => {
        let result_name = if type_.count_registers() == 0 {
          "".to_string()
        } else {
          self.new_name()
        };
        Self::make_register(result_name.clone(), &type_, regs);
        let loop_registers: Vec<_> = parameter_names
          .iter()
          .cloned()
          .zip(parameter_values.iter().map(|i| self.module.type_of(*i)))
          .collect();
        loop_registers
          .iter()
          .for_each(|(name, type_)| Self::make_register(name.clone(), &type_, regs));
        parameter_values
          .into_iter()
          .rev()
          .map(|i| self.lower(i, regs, instrs))
          .try_collect::<Vec<_>>()?;
        loop_registers
          .iter()
          .for_each(|reg| Self::set_register(reg.0.clone(), &reg.1, instrs));
        let block_name = self.new_name();
        instrs.push(asm::Block(block_name.clone()));
        self.break_stack.push(BreakTarget {
          block_name,
          result_name: result_name.clone(),
        });
        let loop_name = self.new_name();
        instrs.push(asm::Loop(loop_name.clone()));
        self.lower(body, regs, instrs)?;
        self.break_stack.pop();
        loop_registers
          .iter()
          .for_each(|(name, type_)| Self::set_register(name.clone(), type_, instrs));
        instrs.push(asm::Branch(loop_name.clone()));
        instrs.push(asm::End);
        instrs.push(asm::End);
        Self::get_register(result_name, &type_, instrs);
      }
      Break(expr) => {
        let type_ = if let Some(expr) = expr {
          self.lower(expr, regs, instrs)?;
          self.module.type_of(expr)
        } else {
          Type::Primitive(Primitive::nothing)
        };
        let BreakTarget {
          block_name,
          result_name,
        } = self.break_stack.last().unwrap().clone();
        Self::set_register(result_name, &type_, instrs);
        instrs.push(asm::Branch(block_name));
      }
      StructLiteral { field_values, .. } => {
        for value in field_values {
          self.lower(value, regs, instrs)?;
        }
      }
      StructDef { .. } => {}
      Tuple(items) => todo!(),
    };
    Ok(())
  }
}
