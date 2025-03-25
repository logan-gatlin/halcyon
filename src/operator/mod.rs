//pub mod assembly;
use crate::token::*;

//pub use assembly::*;

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
  Dot, 13, LEFT_ASSOC;
  Star, 11, LEFT_ASSOC;
  Slash, 11, LEFT_ASSOC;
  Percent, 11, LEFT_ASSOC;
  Plus, 10, LEFT_ASSOC;
  Minus, 10, LEFT_ASSOC;
  Nand, 9, LEFT_ASSOC;
  Xor, 8, LEFT_ASSOC;
  Xnor, 8, LEFT_ASSOC;
  Or, 7, LEFT_ASSOC;
  Nor, 7, LEFT_ASSOC;
  DoubleEqual, 6, LEFT_ASSOC;
  BangEqual, 6, LEFT_ASSOC;
  Less, 6, LEFT_ASSOC;
  LessEqual, 6, LEFT_ASSOC;
  Greater, 6, LEFT_ASSOC;
  GreaterEqual, 6, LEFT_ASSOC;
  And, 5, LEFT_ASSOC;
  Colon, 4, LEFT_ASSOC;
  Equal, 3, LEFT_ASSOC;
  Comma, 2, LEFT_ASSOC;
  Arrow, 1, RIGHT_ASSOC;
  FatArrow, 1, RIGHT_ASSOC;
}

op! {
  UnaryOp;
  Ampersand, 12, RIGHT_ASSOC;
  Tilda, 12, RIGHT_ASSOC;
  Minus, 12, LEFT_ASSOC;
  Not, 12, LEFT_ASSOC;
  Break, 0, LEFT_ASSOC;
}
