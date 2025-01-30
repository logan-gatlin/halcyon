use std::collections::HashMap;
use std::hash::Hasher;

use crate::compile::{AsmType, Wasm};
use crate::semantic::primitives::Primitive;
use crate::{diagnostic, BinaryOp, UnaryOp};
use crate::{err::*, error};

use super::Type;

#[derive(Clone, Debug)]
struct BinaryOpKey {
  op: BinaryOp,
  left: Type,
  right: Type,
}

struct UnaryOpKey {
  op: UnaryOp,
  on: Type,
}

// Order independent equality
impl PartialEq for BinaryOpKey {
  fn eq(&self, other: &Self) -> bool {
    self.op == other.op
      && ((self.left == other.left && self.right == other.right)
        || (self.left == other.right && self.right == other.left))
  }
}

impl PartialEq for UnaryOpKey {
  fn eq(&self, other: &Self) -> bool {
    self.op == other.op && self.on == other.on
  }
}

impl Eq for BinaryOpKey {}
impl Eq for UnaryOpKey {}

#[derive(Clone, Debug)]
pub struct OpDef {
  pub produces: Type,
  pub asm: Vec<Wasm>,
}

impl Default for OpDef {
  fn default() -> Self {
    OpDef {
      produces: Type::Ambiguous,
      asm: vec![Wasm::nop],
    }
  }
}

impl std::hash::Hash for BinaryOpKey {
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

impl std::hash::Hash for UnaryOpKey {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.op.hash(state);
    self.on.hash(state);
  }
}

pub struct OpTable {
  binary_map: HashMap<BinaryOpKey, OpDef>,
  unary_map: HashMap<UnaryOpKey, OpDef>,
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
    use AsmType::*;
    use Primitive::*;
    use Wasm::*;
    {
      use BinaryOp::*;
      let mut b = |op: BinaryOp, p1: Primitive, p2: Primitive, prod: Primitive, asm: Vec<Wasm>| {
        self
          .define_binary(op, Type::Prim(p1), Type::Prim(p2), Type::Prim(prod), asm)
          .unwrap();
      };
      // math
      b(Plus, integer, integer, integer, vec![add(i64)]);
      b(Plus, real, real, real, vec![add(f64)]);
      b(Minus, integer, integer, integer, vec![subtract(i64)]);
      b(Minus, real, real, real, vec![subtract(f64)]);
      b(Star, integer, integer, integer, vec![multiply(i64)]);
      b(Star, real, real, real, vec![multiply(f64)]);
      b(Slash, integer, integer, integer, vec![divide(i64)]);
      b(Slash, real, real, real, vec![divide(f64)]);
      b(Percent, integer, integer, integer, vec![remainder(i64)]);
      // logical
      b(And, boolean, boolean, boolean, vec![and(i32)]);
      b(And, integer, integer, integer, vec![and(i64)]);
      b(Or, boolean, boolean, boolean, vec![or(i32)]);
      b(Or, integer, integer, integer, vec![or(i64)]);
      b(Xor, boolean, boolean, boolean, vec![or(i32)]);
      b(Xor, integer, integer, integer, vec![xor(i64)]);
      b(
        Nand,
        boolean,
        boolean,
        boolean,
        vec![and(i32), constant(i32, "1".into()), xor(i32)],
      );
      b(
        Nand,
        integer,
        integer,
        integer,
        vec![and(i64), constant(i64, "-1".into()), xor(i64)],
      );
      b(Xnor, boolean, boolean, boolean, vec![equal(i32)]);
      b(Xnor, integer, integer, integer, vec![equal(i64)]);
      b(
        Nor,
        boolean,
        boolean,
        boolean,
        vec![or(i32), constant(i32, "1".into()), xor(i32)],
      );
      b(
        Nor,
        integer,
        integer,
        integer,
        vec![or(i64), constant(i64, "-1".into()), xor(i64)],
      );
      // Relative value
      b(DoubleEqual, boolean, boolean, boolean, vec![equal(i32)]);
      b(DoubleEqual, integer, integer, boolean, vec![equal(i64)]);
      b(DoubleEqual, real, real, boolean, vec![equal(f64)]);
      b(
        DoubleEqual,
        nothing,
        nothing,
        boolean,
        vec![constant(i32, "1".into())],
      );
      b(DoubleEqual, glyph, glyph, boolean, vec![equal(i32)]);
      b(Less, integer, integer, boolean, vec![lesser_s(i64)]);
      b(Less, glyph, glyph, boolean, vec![lesser_u(i32)]);
      b(Less, real, real, boolean, vec![lesser_s(f64)]);
      b(Greater, integer, integer, boolean, vec![greater_s(i64)]);
      b(Greater, glyph, glyph, boolean, vec![greater_u(i32)]);
      b(Greater, real, real, boolean, vec![greater_s(f64)]);
      b(
        LessEqual,
        integer,
        integer,
        boolean,
        vec![lesserequal_s(i64)],
      );
      b(LessEqual, glyph, glyph, boolean, vec![lesserequal_u(i32)]);
      b(LessEqual, real, real, boolean, vec![lesserequal_s(f64)]);
      b(
        GreaterEqual,
        integer,
        integer,
        boolean,
        vec![greaterequal_s(i64)],
      );
      b(
        GreaterEqual,
        glyph,
        glyph,
        boolean,
        vec![greaterequal_u(i32)],
      );
      b(GreaterEqual, real, real, boolean, vec![greaterequal_s(f64)]);
      b(BangEqual, boolean, boolean, boolean, vec![unequal(i32)]);
      b(BangEqual, integer, integer, boolean, vec![unequal(i64)]);
      b(BangEqual, glyph, glyph, boolean, vec![unequal(i32)]);
      b(BangEqual, real, real, boolean, vec![unequal(f64)]);
      b(
        BangEqual,
        nothing,
        nothing,
        boolean,
        vec![constant(i32, "0".into())],
      );
    }
    {
      use UnaryOp::*;
      let mut u = |op: UnaryOp, p1: Primitive, prod: Primitive, asm: Vec<Wasm>| {
        self
          .define_unary(op, Type::Prim(p1), Type::Prim(prod), asm)
          .unwrap();
      };
      u(
        Minus,
        integer,
        integer,
        vec![
          constant(i64, "-1".into()),
          xor(i64),
          constant(i64, "1".into()),
          add(i64),
        ],
      );
      u(Minus, real, real, vec![negate(f64)]);
      u(
        Not,
        integer,
        integer,
        vec![constant(i64, "-1".into()), xor(i64)],
      );
      u(
        Not,
        boolean,
        boolean,
        vec![constant(i32, "1".into()), xor(i32)],
      );
    }
  }

  pub fn define_binary(
    &mut self,
    op: BinaryOp,
    left: Type,
    right: Type,
    produces: Type,
    asm: Vec<Wasm>,
  ) -> Result<()> {
    let err: Result<()> = error!(
      "Operator {op} is already defined for types '{}' and '{}'",
      left, &right
    );
    let key = BinaryOpKey { op, left, right };
    if self.binary_map.contains_key(&key) {
      return err;
    }
    let value = OpDef { produces, asm };
    self.binary_map.insert(key, value);
    Ok(())
  }

  pub fn define_unary(
    &mut self,
    op: UnaryOp,
    on: Type,
    produces: Type,
    asm: Vec<Wasm>,
  ) -> Result<()> {
    let err = error!("Operator {op} is already defined for type '{}'", &on);
    let old = self
      .unary_map
      .insert(UnaryOpKey { op, on }, OpDef { produces, asm });
    if old.is_some() {
      err
    } else {
      Ok(())
    }
  }

  pub fn try_binary(&self, op: BinaryOp, left: &Type, right: &Type) -> Result<OpDef> {
    self
      .binary_map
      .get(
        &(BinaryOpKey {
          op,
          left: left.clone(),
          right: right.clone(),
        }),
      )
      .ok_or(diagnostic!(
        "Operator {op} is not defined for types '{left}' and '{right}'",
      ))
      .cloned()
  }

  pub fn try_unary(&self, op: UnaryOp, on: &Type) -> Result<OpDef> {
    self
      .unary_map
      .get(&UnaryOpKey { op, on: on.clone() })
      .ok_or(diagnostic!("Operator {op} is not defined for type '{on}'"))
      .cloned()
  }
}
