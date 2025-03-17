use crate::ir::types::{Primitive, Type, TypeLint};
use crate::{BinaryOp, UnaryOp, lint::*};
use std::collections::HashMap;
use std::hash::Hasher;

use super::{Wasm, WasmType, WasmValue};

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

impl Eq for BinaryOpKey {
}
impl Eq for UnaryOpKey {
}

#[derive(Clone, Debug)]
pub struct OpDef {
  pub produces: Type,
  pub asm: Vec<Wasm>,
}

impl Default for OpDef {
  fn default() -> Self {
    OpDef {
      produces: Type::Ambiguous,
      asm: vec![Wasm::Nop],
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
    use Primitive::*;
    use Wasm as w;
    use WasmType::*;
    use WasmValue as v;
    {
      use BinaryOp::*;
      let mut b = |op: BinaryOp,
                   p1: Primitive,
                   p2: Primitive,
                   prod: Primitive,
                   asm: Vec<Wasm>| {
        self.define_binary(
          op,
          Type::Primitive(p1),
          Type::Primitive(p2),
          Type::Primitive(prod),
          asm,
        );
      };
      // math
      b(Plus, integer, integer, integer, vec![w::Add(I64)]);
      b(Plus, real, real, real, vec![w::Add(F64)]);
      b(Minus, integer, integer, integer, vec![w::Subtract(I64)]);
      b(Minus, real, real, real, vec![w::Subtract(F64)]);
      b(Star, integer, integer, integer, vec![w::Multiply(I64)]);
      b(Star, real, real, real, vec![w::Multiply(F64)]);
      b(Slash, integer, integer, integer, vec![w::Divide(I64)]);
      b(Slash, real, real, real, vec![w::Divide(F64)]);
      b(Percent, integer, integer, integer, vec![w::Remainder(I64)]);
      // logical
      b(And, boolean, boolean, boolean, vec![w::And(I32)]);
      b(And, integer, integer, integer, vec![w::And(I64)]);
      b(Or, boolean, boolean, boolean, vec![w::Or(I32)]);
      b(Or, integer, integer, integer, vec![w::Or(I64)]);
      b(Xor, boolean, boolean, boolean, vec![w::Or(I32)]);
      b(Xor, integer, integer, integer, vec![w::Xor(I64)]);
      b(
        Nand,
        boolean,
        boolean,
        boolean,
        vec![w::And(I64), w::Constant(v::I32(1)), w::Xor(I32)],
      );
      b(
        Nand,
        integer,
        integer,
        integer,
        vec![w::And(I64), w::Constant(v::I64(-1)), w::Xor(I64)],
      );
      b(Xnor, boolean, boolean, boolean, vec![w::Equal(I32)]);
      b(Xnor, integer, integer, integer, vec![w::Equal(I64)]);
      b(
        Nor,
        boolean,
        boolean,
        boolean,
        vec![w::Or(I64), w::Constant(v::I32(1)), w::Xor(I32)],
      );
      b(
        Nor,
        integer,
        integer,
        integer,
        vec![w::Or(I64), w::Constant(v::I64(-1)), w::Xor(I64)],
      );
      // Relative value
      b(DoubleEqual, boolean, boolean, boolean, vec![w::Equal(I32)]);
      b(DoubleEqual, integer, integer, boolean, vec![w::Equal(I64)]);
      b(DoubleEqual, real, real, boolean, vec![w::Equal(F64)]);
      b(
        DoubleEqual,
        nothing,
        nothing,
        boolean,
        vec![w::Constant(v::I32(1))],
      );
      b(DoubleEqual, glyph, glyph, boolean, vec![w::Equal(I64)]);
      b(Less, integer, integer, boolean, vec![w::LesserSigned(I64)]);
      b(Less, glyph, glyph, boolean, vec![w::LesserUnsigned(I64)]);
      b(Less, real, real, boolean, vec![w::LesserSigned(F64)]);
      b(
        Greater,
        integer,
        integer,
        boolean,
        vec![w::GreaterSigned(I64)],
      );
      b(
        Greater,
        glyph,
        glyph,
        boolean,
        vec![w::GreaterUnsigned(I64)],
      );
      b(Greater, real, real, boolean, vec![w::GreaterSigned(F64)]);
      b(
        LessEqual,
        integer,
        integer,
        boolean,
        vec![w::LesserEqualSigned(I64)],
      );
      b(
        LessEqual,
        glyph,
        glyph,
        boolean,
        vec![w::LesserEqualUnsigned(I64)],
      );
      b(
        LessEqual,
        real,
        real,
        boolean,
        vec![w::LesserEqualSigned(F64)],
      );
      b(
        GreaterEqual,
        integer,
        integer,
        boolean,
        vec![w::GreaterEqualSigned(I64)],
      );
      b(
        GreaterEqual,
        glyph,
        glyph,
        boolean,
        vec![w::GreaterEqualUnsigned(I64)],
      );
      b(
        GreaterEqual,
        real,
        real,
        boolean,
        vec![w::GreaterEqualSigned(F64)],
      );
      b(BangEqual, boolean, boolean, boolean, vec![w::Unequal(I32)]);
      b(BangEqual, integer, integer, boolean, vec![w::Unequal(I64)]);
      b(BangEqual, glyph, glyph, boolean, vec![w::Unequal(I64)]);
      b(BangEqual, real, real, boolean, vec![w::Unequal(F64)]);
      b(
        BangEqual,
        nothing,
        nothing,
        boolean,
        vec![w::Constant(v::I32(0))],
      );
    }
    {
      use UnaryOp::*;
      let mut u = |op: UnaryOp,
                   p1: Primitive,
                   prod: Primitive,
                   asm: Vec<Wasm>| {
        self.define_unary(op, Type::Primitive(p1), Type::Primitive(prod), asm);
      };
      u(
        Minus,
        integer,
        integer,
        vec![
          w::Constant(v::I64(-1)),
          w::Xor(I64),
          w::Constant(v::I64(1)),
          w::Add(I64),
        ],
      );
      u(Minus, real, real, vec![w::Negate(F64)]);
      u(
        Not,
        integer,
        integer,
        vec![w::Constant(v::I64(-1)), w::Xor(I64)],
      );
      u(
        Not,
        boolean,
        boolean,
        vec![w::Constant(v::I32(1)), w::Xor(I32)],
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
  ) {
    let key = BinaryOpKey { op, left, right };
    if self.binary_map.contains_key(&key) {
      panic!();
    }
    let value = OpDef { produces, asm };
    self.binary_map.insert(key, value);
  }

  pub fn define_unary(
    &mut self,
    op: UnaryOp,
    on: Type,
    produces: Type,
    asm: Vec<Wasm>,
  ) {
    let old = self
      .unary_map
      .insert(UnaryOpKey { op, on }, OpDef { produces, asm });
    if old.is_some() {
      panic!()
    }
  }

  pub fn try_binary(
    &self,
    op: BinaryOp,
    left: &Type,
    right: &Type,
  ) -> Result<OpDef> {
    self
      .binary_map
      .get(
        &(BinaryOpKey {
          op,
          left: left.clone(),
          right: right.clone(),
        }),
      )
      .ok_or(lint_nospan(TypeLint::BinaryOpUndefined as LintKind))
      .context(format!("{op}"))
      .context(format!("{left}"))
      .context(format!("{right}"))
      .cloned()
  }

  pub fn try_unary(&self, op: UnaryOp, on: &Type) -> Result<OpDef> {
    if let UnaryOp::Tilda = op
      && on != &Type::Primitive(Primitive::nothing)
    {
      return Ok(OpDef {
        produces: Primitive::nothing.promote(),
        // TODO revisit this
        asm: vec![],
      });
    }
    self
      .unary_map
      .get(&UnaryOpKey { op, on: on.clone() })
      .ok_or(lint_nospan(TypeLint::UnaryOpUndefined as LintKind))
      .context(format!("{op}"))
      .context(format!("{on}"))
      .cloned()
  }
}
