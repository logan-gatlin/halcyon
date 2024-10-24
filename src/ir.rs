use crate::{
  BinaryOp, Expression, ExpressionKind, Immediate, UnaryOp,
  semantic::{Type, VarKind, uid},
};

#[derive(Debug, Clone)]
pub enum IR {
  BinOp { op: BinaryOp, type_: Type },
  UnOp { op: UnaryOp, type_: Type },
  Imm(Immediate),
  NewLocal { uid: uid, type_: Type },
  AssignLocal { uid: uid },
  GetLocal { uid: uid },
  NewGlobal { uid: uid, type_: Type },
  AssignGlobal { uid: uid },
  GetGlobal { uid: uid },
  StartFunc { uid: uid },
  NewParam { uid: uid, type_: Type },
  EndFunc,
}

pub struct Compiler {
  ir: Vec<IR>,
}

impl Compiler {
  fn expression(&mut self, expression: Expression) {
    use ExpressionKind::*;
    match expression.kind {
      Immediate(immediate) => {
        self.ir.push(IR::Imm(immediate));
      },
      Identifier(name, var_kind) => match var_kind {
        VarKind::Global(uid) => self.ir.push(IR::GetGlobal { uid }),
        VarKind::Local(uid) | VarKind::Param(uid) => {
          self.ir.push(IR::GetLocal { uid })
        },
        VarKind::Function(_) => todo!(),
        VarKind::Undefined => todo!(),
      },
      Binary { op, left, right } => todo!(),
      Unary { op, child } => todo!(),
      Parenthesis(expression) => todo!(),
      FunctionDef {
        params,
        returns_str,
        returns_actual,
        body,
        id,
      } => todo!(),
      FunctionCall { callee, args } => todo!(),
      StructDef(vec) => todo!(),
      StructLiteral { name, args } => todo!(),
      Field { namespace, field } => todo!(),
    }
  }
}
