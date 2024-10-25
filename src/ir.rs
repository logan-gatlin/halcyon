use crate::{
  BinaryOp, Expression, ExpressionKind, Immediate, Statement, StatementKind,
  UnaryOp,
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
  EndFunc,
  ReturnType { type_: Type },
  NewParam { uid: uid, type_: Type },
  Return,
  Call { uid: uid },
  Drop,
}

impl std::fmt::Display for IR {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use IR::*;
    match self {
      BinOp { op, type_ } => write!(f, "{op} ({type_})"),
      UnOp { op, type_ } => write!(f, "{op}, {type_}"),
      Imm(immediate) => write!(f, "push {immediate}"),
      NewLocal { uid, type_ } => write!(f, "local ${uid} = {type_}"),
      AssignLocal { uid } => write!(f, "pop local ${uid}"),
      GetLocal { uid } => write!(f, "push local ${uid}"),
      NewGlobal { uid, type_ } => write!(f, "global ${uid} = {type_}"),
      AssignGlobal { uid } => write!(f, "pop global ${uid}"),
      GetGlobal { uid } => write!(f, "push global ${uid}"),
      StartFunc { uid } => write!(f, "<function id=${uid}>"),
      EndFunc => write!(f, "</function>"),
      NewParam { uid, type_ } => write!(f, "param ${uid} = {type_}"),
      Return => write!(f, "return"),
      Call { uid } => write!(f, "call ${uid}"),
      Drop => write!(f, "pop"),
      ReturnType { type_ } => write!(f, "result {type_}"),
    }
  }
}

pub struct Compiler {
  pub ir: Vec<IR>,
}

impl Compiler {
  pub fn new() -> Self {
    Self { ir: vec![] }
  }

  pub fn compile(&mut self, block: Vec<Statement>) {
    for s in block {
      self.statement(s);
    }
    self.ir = self.hoist_functions();
  }

  fn statement(&mut self, statement: Statement) {
    use StatementKind::*;
    match statement.kind {
      Declaration {
        type_actual,
        value,
        varkind,
        ..
      } => {
        match varkind {
          VarKind::Global(uid) => self.ir.push(IR::NewGlobal {
            uid,
            type_: type_actual,
          }),
          VarKind::Local(uid) => self.ir.push(IR::NewLocal {
            uid,
            type_: type_actual,
          }),
          _ => {},
        }
        self.expression(value);
        match varkind {
          VarKind::Global(uid) => self.ir.push(IR::AssignGlobal { uid }),
          VarKind::Local(uid) => self.ir.push(IR::AssignLocal { uid }),
          _ => {},
        };
      },
      Assignment {
        name,
        value,
        varkind,
      } => {
        self.expression(value);
        match varkind {
          VarKind::Global(uid) => self.ir.push(IR::AssignGlobal { uid }),
          VarKind::Local(uid) => self.ir.push(IR::AssignLocal { uid }),
          _ => {},
        }
      },
      If {
        predicate,
        block,
        else_,
      } => todo!(),
      While { predicate, block } => todo!(),
      Print(expression) => todo!(),
      Expression(expression) => {
        if expression.type_ == Type::Nothing {
          self.expression(expression);
        } else {
          self.expression(expression);
          self.ir.push(IR::Drop);
        }
      },
      Block(statements) => {
        for s in statements {
          self.statement(s);
        }
      },
      Error(diagnostic) => {
        panic!("{}", diagnostic);
      },
      Return(expression) => {
        if let Some(e) = expression {
          self.expression(e);
        }
        self.ir.push(IR::Return);
      },
    }
  }

  fn expression(&mut self, expression: Expression) {
    use ExpressionKind::*;
    match expression.kind {
      Immediate(immediate) => {
        self.ir.push(IR::Imm(immediate));
      },
      Identifier(_, var_kind) => match var_kind {
        VarKind::Global(uid) => self.ir.push(IR::GetGlobal { uid }),
        VarKind::Local(uid) => self.ir.push(IR::GetLocal { uid }),
        VarKind::Function(_) => {},
        VarKind::Undefined => panic!("Undefined var not caught by typecheck"),
      },
      Binary {
        op,
        mut left,
        mut right,
      } => {
        left.type_ = Type::coerce(&expression.type_, &left.type_).unwrap();
        right.type_ = Type::coerce(&expression.type_, &right.type_).unwrap();
        assert!(&left.type_ == &right.type_);
        self.expression(*left.clone());
        self.expression(*right);
        self.ir.push(IR::BinOp {
          op,
          type_: expression.type_,
        });
      },
      Unary { op, mut child } => {
        child.type_ = Type::coerce(&expression.type_, &child.type_).unwrap();
        self.expression(*child);
        self.ir.push(IR::UnOp {
          op,
          type_: expression.type_,
        })
      },
      Parenthesis(mut e) => {
        e.type_ = Type::coerce(&expression.type_, &e.type_).unwrap();
        self.expression(*e);
      },
      FunctionDef {
        params,
        returns_actual,
        body,
        id,
        ..
      } => {
        self.ir.push(IR::StartFunc { uid: id });
        for (i, p) in params.iter().enumerate() {
          self.ir.push(IR::NewParam {
            uid: i as uid,
            type_: p.type_actual.clone(),
          })
        }
        self.ir.push(IR::ReturnType {
          type_: returns_actual,
        });
        for s in body {
          self.statement(s);
        }
        self.ir.push(IR::EndFunc);
      },
      FunctionCall { callee, args } => {
        let Type::FunctionDef { id, .. } = callee.type_ else {
          panic!()
        };
        for arg in args {
          self.expression(arg);
        }
        self.ir.push(IR::Call { uid: id });
      },
      StructDef(_) => {},
      StructLiteral { name, args } => todo!(),
      Field { namespace, field } => todo!(),
    }
  }

  fn hoist_functions(&self) -> Vec<IR> {
    let mut functions = vec![(vec![], vec![])];
    let mut result = vec![];
    for index in 0..self.ir.len() {
      let ir = self.ir.get(index).unwrap();
      match ir {
        IR::StartFunc { .. } => {
          functions.push((vec![], vec![]));
        },
        IR::EndFunc => {
          let (inits, instr) = functions.pop().unwrap();
          for ir in inits {
            result.push(ir);
          }
          for ir in instr {
            result.push(ir);
          }
          result.push(IR::EndFunc);
          continue;
        },
        _ => {},
      }
      // Push instruction to correct stack
      let (inits, instr) = functions.last_mut().unwrap();
      match ir {
        IR::NewLocal { .. }
        | IR::NewGlobal { .. }
        | IR::NewParam { .. }
        | IR::StartFunc { .. } => {
          inits.push(ir.clone());
        },
        _ => instr.push(ir.clone()),
      }
    }
    // Initialize globals
    let (inits, instr) = functions.pop().unwrap();
    for ir in inits {
      result.push(ir);
    }
    // The main function (index 0)
    result.push(IR::StartFunc { uid: 0 });
    for ir in instr {
      result.push(ir);
    }
    result.push(IR::EndFunc);
    result
  }
}
