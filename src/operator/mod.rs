//pub mod assembly;
use crate::{hlir::*, lint::*, token::*};

pub type Precedence = usize;

macro_rules! op {
  ($name:ident; $($op:ident, $prec:expr, $assoc:expr);*;) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum $name {
      $($op,)*
    }

    impl $name {
      pub fn precedence(&self) -> Precedence {
        match self {
          $(Self::$op => $prec),*
        }
      }

      pub fn assoc(&self) -> bool {
        match self {
          $(Self::$op => $assoc),*
        }
      }
    }

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          $(Self::$op => write!(f, "{}", TokenKind::$op)),*
        }
      }
    }

    impl TryFrom<&TokenKind> for $name {
      type Error = ();
      fn try_from(value: &TokenKind) -> std::result::Result<Self, ()> {
        match value {
          $(TokenKind::$op => Ok(Self::$op),)*
          _ => Err(()),
        }
      }
    }
  }
}

pub const RIGHT_ASSOC: bool = true;
pub const LEFT_ASSOC: bool = false;
pub const CALL_PREC: Precedence = 12;

// Name, precedence, associativity;
op! {
  BinaryOp;
  Dot, 17, LEFT_ASSOC;
  Star, 15, LEFT_ASSOC;
  StarDot, 15, LEFT_ASSOC;
  Slash, 15, LEFT_ASSOC;
  SlashDot, 15, LEFT_ASSOC;
  Percent, 15, LEFT_ASSOC;
  Plus, 14, LEFT_ASSOC;
  PlusDot, 14, LEFT_ASSOC;
  Minus, 14, LEFT_ASSOC;
  MinusDot, 14, LEFT_ASSOC;
  Xor, 10, LEFT_ASSOC;
  Xnor, 10, LEFT_ASSOC;
  Or, 9, LEFT_ASSOC;
  Nor, 9, LEFT_ASSOC;
  Apply, 9, LEFT_ASSOC;
  DoubleEqual, 8, LEFT_ASSOC;
  BangEqual, 8, LEFT_ASSOC;
  Less, 8, LEFT_ASSOC;
  LessEqual, 8, LEFT_ASSOC;
  Greater, 8, LEFT_ASSOC;
  GreaterEqual, 8, LEFT_ASSOC;
  Nand, 7, LEFT_ASSOC;
  And, 7, LEFT_ASSOC;
  Arrow, 5, RIGHT_ASSOC;
  DoubleColon, 4, RIGHT_ASSOC;
  Comma, 3, LEFT_ASSOC;
  Semicolon, 1, LEFT_ASSOC;
}

op! {
  UnaryOp;
  Minus, 15, LEFT_ASSOC;
  MinusDot, 15, LEFT_ASSOC;
  Not, 15, LEFT_ASSOC;
}

impl BinaryOp {
  pub fn with(&self, t1: &Type, t2: &Type) -> Result<Type> {
    use BinaryOp::*;
    use Primitive::*;
    use Type::Primitive as p;
    let e = Err(lint_nospan(TypeLint::BinaryOpUndefined))
      .context(format!("{t1}"))
      .context(format!("{t2}"));
    Ok(match (self, t1, t2) {
      // Math
      (Plus, p(integer), p(integer)) => p(integer),
      (PlusDot, p(real), p(real)) => p(real),
      (Minus, p(integer), p(integer)) => p(integer),
      (MinusDot, p(real), p(real)) => p(real),
      (Star, p(integer), p(integer)) => p(integer),
      (StarDot, p(real), p(real)) => p(real),
      (Slash, p(integer), p(integer)) => p(integer),
      (SlashDot, p(real), p(real)) => p(real),
      (Percent, p(integer), p(integer)) => p(integer),
      // Logic
      (And, p(boolean), p(boolean)) => p(boolean),
      (Or, p(boolean), p(boolean)) => p(boolean),
      (Xor, p(boolean), p(boolean)) => p(boolean),
      (Xnor, p(boolean), p(boolean)) => p(boolean),
      (Nand, p(boolean), p(boolean)) => p(boolean),
      (Nor, p(boolean), p(boolean)) => p(boolean),
      // Equivalence
      (DoubleEqual, p(integer), p(integer)) => p(boolean),
      (DoubleEqual, p(real), p(real)) => p(boolean),
      (DoubleEqual, p(boolean), p(boolean)) => p(boolean),
      (DoubleEqual, p(glyph), p(glyph)) => p(boolean),
      (DoubleEqual, p(string), p(string)) => p(boolean),
      (DoubleEqual, p(nothing), p(nothing)) => p(boolean),

      (BangEqual, p(integer), p(integer)) => p(boolean),
      (BangEqual, p(real), p(real)) => p(boolean),
      (BangEqual, p(boolean), p(boolean)) => p(boolean),
      (BangEqual, p(glyph), p(glyph)) => p(boolean),
      (BangEqual, p(string), p(string)) => p(boolean),
      (BangEqual, p(nothing), p(nothing)) => p(boolean),

      (Less, p(integer), p(integer)) => p(boolean),
      (Less, p(real), p(real)) => p(boolean),
      (Less, p(boolean), p(boolean)) => p(boolean),
      (Less, p(glyph), p(glyph)) => p(boolean),
      (Less, p(string), p(string)) => p(boolean),

      (LessEqual, p(integer), p(integer)) => p(boolean),
      (LessEqual, p(real), p(real)) => p(boolean),
      (LessEqual, p(boolean), p(boolean)) => p(boolean),
      (LessEqual, p(glyph), p(glyph)) => p(boolean),
      (LessEqual, p(string), p(string)) => p(boolean),

      (Greater, p(integer), p(integer)) => p(boolean),
      (Greater, p(real), p(real)) => p(boolean),
      (Greater, p(boolean), p(boolean)) => p(boolean),
      (Greater, p(glyph), p(glyph)) => p(boolean),
      (Greater, p(string), p(string)) => p(boolean),

      (GreaterEqual, p(integer), p(integer)) => p(boolean),
      (GreaterEqual, p(real), p(real)) => p(boolean),
      (GreaterEqual, p(boolean), p(boolean)) => p(boolean),
      (GreaterEqual, p(glyph), p(glyph)) => p(boolean),
      (GreaterEqual, p(string), p(string)) => p(boolean),
      _ => return e,
    })
  }
}

impl UnaryOp {
  pub fn with(&self, t1: Type) -> Result<Type> {
    use Primitive::*;
    use Type::Primitive as p;
    use UnaryOp::*;
    let e =
      Err(lint_nospan(TypeLint::UnaryOpUndefined)).context(format!("{t1}"));
    Ok(match (self, t1) {
      (Minus, p(integer)) => p(integer),
      (MinusDot, p(real)) => p(real),
      (Not, p(boolean)) => p(boolean),
      _ => return e,
    })
  }
}
