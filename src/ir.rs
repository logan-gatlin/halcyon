use crate::{
  BinaryOp, Expression, ExpressionKind, Immediate, Statement, StatementKind,
  UnaryOp,
  semantic::{Primitive, Type, UID},
};
/*
#[derive(Debug, Clone)]
pub enum IR {
  BinOp { op: BinaryOp, type_: Type },
  UnOp { op: UnaryOp, type_: Type },
  Push { prim: Primitive, value: Immediate },
  NewLocal { uid: UID, type_: Type },
  AssignLocal { uid: UID },
  GetLocal { uid: UID },
  NewGlobal { uid: UID, type_: Type },
  AssignGlobal { uid: UID },
  GetGlobal { uid: UID },
  StartFunc { uid: UID },
  EndFunc,
  ReturnType { type_: Type },
  NewParam { uid: UID, type_: Type },
  Return,
  Call { uid: UID },
  Drop,
  Print,
}

impl std::fmt::Display for IR {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use IR::*;
    match self {
      BinOp { op, type_ } => write!(f, "{op} ({type_})"),
      UnOp { op, type_ } => write!(f, "{op}, {type_}"),
      Push { prim, value } => write!(f, "push {value} ({prim})"),
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
      Print => write!(f, "print [DEBUG]"),
    }
  }
}

impl IR {
  pub fn to_wat(&self) -> String {
    use BinaryOp as b;
    use IR::*;
    use Primitive as p;
    use Type as t;
    match self {
      // Primitive
      BinOp {
        op,
        type_: t::Prim(p),
      } => match (op, p) {
        // Addition
        (b::Plus, p::i32 | p::w32 | p::integer | p::whole) => "i32.add",
        (b::Plus, p::i64 | p::w64) => "i64.add",
        (b::Plus, p::r32) => "f32.add",
        (b::Plus, p::r64) => "f64.add",
        // Subtraction
        (b::Minus, p::i32 | p::w32 | p::integer | p::whole) => "i32.sub",
        (b::Minus, p::i64 | p::w64) => "i64.sub",
        (b::Minus, p::r32) => "f32.sub",
        (b::Minus, p::r64) => "f64.sub",
        // Multiplication
        (b::Star, p::i32 | p::w32 | p::integer | p::whole) => "i32.mul",
        (b::Star, p::i64 | p::w64) => "i64.mul",
        (b::Star, p::r32) => "f32.mul",
        (b::Star, p::r64) => "f64.mul",
        // Division
        (b::Slash, p::i32 | p::integer) => "i32.div_s",
        (b::Slash, p::w32 | p::whole) => "i32.div_u",
        (b::Slash, p::i64) => "i64.div_s",
        (b::Slash, p::w64) => "i64.div_u",
        (b::Slash, p::r32) => "f32.div",
        (b::Slash, p::r64) => "f64.div",
        _ => todo!(),
      }
      .into(),
      UnOp {
        op,
        type_: t::Prim(p),
      } => todo!(),
      Push { prim, value } => todo!(),
      NewLocal {
        uid,
        type_: t::Prim(p),
      } => format!("(local ${uid} {})", p.as_wat()),
      AssignLocal { uid } => format!("local.set ${uid}"),
      GetLocal { uid } => format!("local.get ${uid}"),
      NewGlobal {
        uid,
        type_: t::Prim(p),
      } => format!(
        "(global ${uid} (mut {}) ({}.const 0))",
        p.as_wat(),
        p.as_wat()
      ),
      AssignGlobal { uid } => format!("global.set ${uid}"),
      GetGlobal { uid } => format!("global.get ${uid}"),
      StartFunc { uid } => format!("(func ${uid}"),
      EndFunc => ")".into(),
      ReturnType { type_: t::Prim(p) } => format!("(result {})", p.as_wat()),
      NewParam {
        uid,
        type_: t::Prim(p),
      } => format!("(param ${uid} {})", p.as_wat()),
      Return => "return".into(),
      Call { uid } => format!("call ${uid}"),
      Drop => "drop".into(),
      Print => "call $log".into(),
      _ => todo!(),
    }
    .into()
  }
}

pub struct Compiler {
  pub ir: Vec<IR>,
}

impl Compiler {
  const IMPORTS: &'static str =
    r#"(import "console" "log" (func $log (param i32)))"#;

  pub fn compile(block: Vec<Statement>) -> (Vec<u8>, String) {
    let mut this = Self { ir: vec![] };
    for s in block {
      this.statement(s);
    }
    this.ir = this.hoist();
    let mut output = String::new();
    for ir in &this.ir {
      //println!("{ir}");
      let s = ir.to_wat();
      output.push_str(&format!("{s}\n"));
    }
    output = format!("(module\n{}\n{output}\n(start $0)\n)", Self::IMPORTS);
    println!("{output}");
    (wat::parse_str(&output).unwrap(), output)
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
      Print(expression) => {
        self.expression(expression);
        self.ir.push(IR::Print);
      },
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
        let Type::Prim(p) = expression.type_ else {
          panic!();
        };
        self.ir.push(IR::Push {
          value: immediate,
          prim: p,
        })
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
        assert!(&left.type_ == &right.type_);
        self.expression(*left.clone());
        self.expression(*right);
        self.ir.push(IR::BinOp {
          op,
          type_: expression.type_,
        });
      },
      Unary { op, mut child } => {
        self.expression(*child);
        self.ir.push(IR::UnOp {
          op,
          type_: expression.type_,
        })
      },
      Parenthesis(mut e) => {
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

  fn hoist(&self) -> Vec<IR> {
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
*/
