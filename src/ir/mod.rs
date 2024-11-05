/*
mod generate;
mod wasm;

use std::io::Write;

use crate::{
  BinaryOp, Immediate, Statement, UnaryOp,
  semantic::{Primitive, SymbolTable, Type, UID},
};

#[derive(Debug, Clone)]
pub enum IR {
  BinOp {
    op: BinaryOp,
    type_: Type,
  },
  UnOp {
    op: UnaryOp,
    type_: Type,
  },
  Push {
    prim: Primitive,
    value: Immediate,
  },
  New {
    uid: UID,
    type_: Type,
    mutable: bool,
    global: bool,
  },
  Set {
    uid: UID,
    type_: Type,
    global: bool,
  },
  Get {
    uid: UID,
    type_: Type,
    global: bool,
  },
  StartFunc {
    uid: UID,
    params: Vec<(UID, Type)>,
    returns: Type,
  },
  EndFunc,
  Return {
    type_: Type,
  },
  Call {
    uid: UID,
  },
  Drop {
    type_: Type,
  },
  Print {
    type_: Type,
  },
}

impl std::fmt::Display for IR {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use IR::*;
    match self {
      BinOp { op, type_ } => write!(f, "{op} ({type_})"),
      UnOp { op, type_ } => write!(f, "{op}, {type_}"),
      Push { prim, value } => write!(f, "push {value} ({prim})"),
      New {
        uid,
        type_,
        mutable,
        ..
      } => write!(
        f,
        "let {}{uid} : {type_}",
        if *mutable { "mut " } else { "" }
      ),
      Set { uid, .. } => write!(f, "set local {uid}"),
      Get { uid, .. } => write!(f, "get local {uid}"),
      StartFunc {
        uid,
        params,
        returns,
      } => write!(f, "<function id={uid} params={params:?} returns={returns}>"),
      EndFunc => write!(f, "</function>"),
      Call { uid } => write!(f, "call {uid}"),
      Return { type_ } => write!(f, "result {type_}"),
      Print { type_ } => write!(f, "print {type_} [DEBUG]"),
      Drop { .. } => write!(f, "pop"),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Compiler {
  ir: Vec<IR>,
  table: SymbolTable,
  tmp_num: usize,
}

impl Compiler {
  pub fn new(table: SymbolTable) -> Self {
    Self {
      ir: vec![],
      table,
      tmp_num: 0,
    }
  }

  pub fn compile(&mut self, statements: Vec<Statement>) {
    self.generate(statements);
    self.hoist();
    for ir in &self.ir {
      println!("{ir}");
    }
    let mut s = String::new();
    for ir in &self.ir {
      s.push_str(&self.ir_to_wat(ir.clone()).unwrap());
    }
    let assembly = format!("(module\n{s})");
    println!("--------");
    println!("{assembly}");
    std::fs::File::create("test.wat")
      .unwrap()
      .write_all(assembly.as_bytes())
      .unwrap();
    let binary = wat::parse_str(assembly).unwrap();
    std::fs::File::create("test.wasm")
      .unwrap()
      .write_all(&binary)
      .unwrap();
  }

  pub fn push(&mut self, ir: IR) {
    self.ir.push(ir);
  }

  fn hoist(&mut self) {
    // Declarations, instructions
    let mut functions = vec![(vec![], vec![])];
    // Final IR output
    let mut result = vec![];
    for ir in &self.ir {
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
        IR::New { .. } | IR::StartFunc { .. } => {
          inits.push(ir.clone());
        },
        _ => instr.push(ir.clone()),
      }
    }
    // Initialize globals
    let (inits, instr) = functions.pop().unwrap();
    let mut main_locals = vec![];
    for ir in inits {
      match ir {
        IR::New { global: true, .. } => {
          result.push(ir);
        },
        _ => main_locals.push(ir),
      }
    }
    // The main function (index 0)
    result.push(IR::StartFunc {
      uid: "$$main".into(),
      params: vec![],
      returns: Type::Nothing,
    });
    for ir in main_locals {
      result.push(ir);
    }
    for ir in instr {
      result.push(ir);
    }
    result.push(IR::EndFunc);
    self.ir = result;
  }
}
*/
