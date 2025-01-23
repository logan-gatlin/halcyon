use std::collections::HashMap;
use std::hash::Hasher;

use crate::semantic::primitives::Primitive;
use crate::{err::*, error};
use crate::{BinaryOp, UnaryOp};

use super::Type;

#[derive(Clone, Debug)]
pub struct BinaryOpDef {
  pub op: BinaryOp,
  pub left: Type,
  pub right: Type,
  pub asm: String,
}

#[derive(Clone, Debug)]
pub struct UnaryOpDef {
  pub op: UnaryOp,
  pub on: Type,
  pub asm: String,
}

impl Default for BinaryOpDef {
  fn default() -> Self {
    BinaryOpDef {
      op: BinaryOp::Plus,
      left: Type::Ambiguous,
      right: Type::Ambiguous,
      asm: "".into(),
    }
  }
}

impl Default for UnaryOpDef {
  fn default() -> Self {
    UnaryOpDef {
      op: UnaryOp::Plus,
      on: Type::Ambiguous,
      asm: "".into(),
    }
  }
}

// Order independent equality
impl PartialEq for BinaryOpDef {
  fn eq(&self, other: &Self) -> bool {
    self.op == other.op
      && ((self.left == other.left && self.right == other.right)
        || (self.left == other.right && self.right == other.left))
  }
}

impl PartialEq for UnaryOpDef {
  fn eq(&self, other: &Self) -> bool {
    self.op == other.op && self.on == other.on
  }
}

impl Eq for BinaryOpDef {}
impl Eq for UnaryOpDef {}

impl std::hash::Hash for BinaryOpDef {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.op.hash(state);
    // Order independent hash of left and right
    let mut h1 = std::hash::DefaultHasher::new();
    let mut h2 = std::hash::DefaultHasher::new();
    self.left.hash(&mut h1);
    self.right.hash(&mut h2);
    (h1.finish() ^ h2.finish()).hash(state)
  }
}

impl std::hash::Hash for UnaryOpDef {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.op.hash(state);
    self.on.hash(state);
  }
}

pub struct OpTable {
  binary_map: HashMap<BinaryOpDef, Type>,
  unary_map: HashMap<UnaryOpDef, Type>,
}

impl OpTable {
  pub fn new() -> Self {
    let mut this = Self {
      binary_map: HashMap::new(),
      unary_map: HashMap::new(),
    };
    this.prelude();
    this
  }

  pub fn prelude(&mut self) {
    use Primitive::*;
    {
      use BinaryOp::*;
      let mut b = |op: BinaryOp, p1: Primitive, p2: Primitive, prod: Primitive, asm: &str| {
        let asm = format!("{asm}\n");
        self
          .define_binary(op, Type::Prim(p1), Type::Prim(p2), Type::Prim(prod), asm)
          .unwrap();
      };
      // math
      b(Plus, integer, integer, integer, "i64.add");
      b(Plus, real, real, real, "f64.add");
      b(Minus, integer, integer, integer, "i64.sub");
      b(Minus, real, real, real, "f64.sub");
      b(Star, integer, integer, integer, "i64.mul");
      b(Star, real, real, real, "f64.mul");
      b(Slash, integer, integer, integer, "i64.div");
      b(Slash, real, real, real, "f64.div");
      b(Percent, integer, integer, integer, "i64.rem");
      // logical
      b(And, boolean, boolean, boolean, "i32.and");
      b(And, integer, integer, integer, "i64.and");
      b(Or, boolean, boolean, boolean, "i32.or");
      b(Or, integer, integer, integer, "i64.or");
      b(Xor, boolean, boolean, boolean, "i32.xor");
      b(Xor, integer, integer, integer, "i64.xor");
      b(
        Nand,
        boolean,
        boolean,
        boolean,
        "i32.and\ni32.const 1\ni32.xor",
      );
      b(
        Nand,
        integer,
        integer,
        integer,
        "i64.and\ni64.const 0xFFFFFFFF\ni64.xor",
      );
      b(Xnor, boolean, boolean, boolean, "i32.eq");
      b(Xnor, integer, integer, integer, "i64.eq");
      b(
        Nor,
        boolean,
        boolean,
        boolean,
        "i32.or\ni32.const 1\ni32.xor",
      );
      b(
        Nor,
        integer,
        integer,
        integer,
        "i64.or\ni64.const 0xFFFFFFFF\ni64.xor",
      );
      // Relative value
      b(DoubleEqual, boolean, boolean, boolean, "i32.eq");
      b(DoubleEqual, integer, integer, integer, "i64.eq");
      b(DoubleEqual, real, real, real, "f64.eq");
      b(DoubleEqual, nothing, nothing, nothing, "i32.const 1");
      b(DoubleEqual, glyph, glyph, glyph, "i32.eq");
      b(Less, integer, integer, integer, "i64.lt");
      b(Less, glyph, glyph, glyph, "i32.lt");
      b(Less, real, real, real, "f64.lt");
      b(Greater, integer, integer, integer, "i64.gt");
      b(Greater, glyph, glyph, glyph, "i32.gt");
      b(Greater, real, real, real, "f64.gt");
      b(LessEqual, integer, integer, integer, "i64.le");
      b(LessEqual, glyph, glyph, glyph, "i32.le");
      b(LessEqual, real, real, real, "f64.le");
      b(GreaterEqual, integer, integer, integer, "i64.ge");
      b(GreaterEqual, real, real, real, "f64.ge");
      b(GreaterEqual, glyph, glyph, glyph, "i32.ge");
      b(BangEqual, boolean, boolean, boolean, "i32.ne");
      b(BangEqual, integer, integer, integer, "i64.ne");
      b(BangEqual, glyph, glyph, glyph, "i32.ne");
      b(BangEqual, real, real, real, "f64.ne");
      b(BangEqual, nothing, nothing, nothing, "i32.const 0");
    }
    {
      use UnaryOp::*;
      let mut u = |op: UnaryOp, p1: Primitive, prod: Primitive, asm: &str| {
        let asm = format!("{asm}\n");
        self
          .define_unary(op, Type::Prim(p1), Type::Prim(prod), asm)
          .unwrap();
      };
      u(Minus, integer, integer, "i64.neg");
      u(Minus, real, real, "f64.neg");
      u(Not, integer, integer, "i64.const 0xFFFFFFFF\ni64.xor");
      u(Not, boolean, boolean, "i32.const 1\ni32.xor");
    }
  }

  pub fn define_binary(
    &mut self,
    op: BinaryOp,
    left: Type,
    right: Type,
    produces: Type,
    asm: String,
  ) -> Result<()> {
    let err: Result<()> = error!(
      "Operator {op} is already defined for types '{}' and '{}'",
      left, &right
    );
    let opdef = BinaryOpDef {
      op,
      left,
      right,
      asm,
    };
    if self.binary_map.contains_key(&opdef) {
      return err;
    }
    self.binary_map.insert(opdef, produces);
    Ok(())
  }

  pub fn define_unary(&mut self, op: UnaryOp, on: Type, produces: Type, asm: String) -> Result<()> {
    let err = error!("Operator {op} is already defined for type '{}'", &on);
    let old = self.unary_map.insert(UnaryOpDef { op, on, asm }, produces);
    if old.is_some() {
      err
    } else {
      Ok(())
    }
  }

  pub fn try_binary(&self, op: BinaryOp, left: &Type, right: &Type) -> Result<Type> {
    let err = error!("Operator {op} is not defined for types '{left}' and '{right}'",);
    let left = left.clone();
    let right = right.clone();
    let opdef = BinaryOpDef {
      op,
      left,
      right,
      asm: "".into(),
    };
    if let Some(t) = self.binary_map.get(&opdef) {
      Ok(t.clone())
    } else {
      err
    }
  }

  pub fn try_unary(&self, op: UnaryOp, on: &Type) -> Result<Type> {
    let err = error!("Operator {op} is not defined for type '{on}'");
    match self.unary_map.get(&UnaryOpDef {
      op,
      on: on.clone(),
      asm: "".into(),
    }) {
      Some(t) => Ok(t.clone()),
      None => err,
    }
  }
}
