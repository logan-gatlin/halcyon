pub mod assembly;
use crate::token::*;

pub use assembly::*;

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
pub const GUARD_PREC: Precedence = 10;
pub const LOOP_PREC: Precedence = 7;
pub const MATCH_PREC: Precedence = 7;
pub const IF_ELSE_PREC: Precedence = 1;

// Name, precedence, associativity;
op! {
  BinaryOp;
  Dot, 17, LEFT_ASSOC;
  Star, 15, LEFT_ASSOC;
  Slash, 15, LEFT_ASSOC;
  Percent, 15, LEFT_ASSOC;
  Plus, 14, LEFT_ASSOC;
  Minus, 14, LEFT_ASSOC;
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
  Semicolon, 7, LEFT_ASSOC;
  FatArrow, 6, RIGHT_ASSOC;
  Colon, 5, RIGHT_ASSOC;
  Arrow, 5, RIGHT_ASSOC;
  Equal, 4, RIGHT_ASSOC;
  DoubleColon, 4, RIGHT_ASSOC;
  Comma, 3, LEFT_ASSOC;
  DoubleSemicolon, 1, LEFT_ASSOC;
}

op! {
  UnaryOp;
  Ampersand, 15, RIGHT_ASSOC;
  Tilda, 15, RIGHT_ASSOC;
  Minus, 15, LEFT_ASSOC;
  Not, 15, LEFT_ASSOC;
  Break, 2, LEFT_ASSOC;
}
