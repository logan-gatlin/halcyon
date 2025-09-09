use crate::std_hc::STD_MODULE_NAME;
use crate::{ir::Path, semantic::Type, token::*};

pub type Precedence = usize;

macro_rules! op {
  ($name:ident; $prefix:literal; $($op:ident, $prec:expr, $assoc:expr);*;) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum $name {
      $($op,)*
    }

    #[allow(dead_code)]
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

      pub fn path(&self) -> Path {
          Path::from(STD_MODULE_NAME).child(format!("{self}"))
      }
    }

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          $(Self::$op => write!(f, "({}{})", $prefix, TokenKind::$op)),*
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

    impl sx::SXRepr for $name {
        fn sx(self) -> sx::SX {
            sx::SX::Atom(format!("{}{self}", $prefix))
        }
    }
  }
}

pub const RIGHT_ASSOC: bool = true;
pub const LEFT_ASSOC: bool = false;
pub const FIELD_PREC: Precedence = 17;
pub const CALL_PREC: Precedence = 12;

// Name, precedence, associativity;
op! {
  BinaryOp; "";
  Star, 15, LEFT_ASSOC;
  StarDot, 15, LEFT_ASSOC;
  Slash, 15, LEFT_ASSOC;
  SlashDot, 15, LEFT_ASSOC;
  Percent, 15, LEFT_ASSOC;
  Plus, 14, LEFT_ASSOC;
  PlusDot, 14, LEFT_ASSOC;
  Minus, 14, LEFT_ASSOC;
  MinusDot, 14, LEFT_ASSOC;
  ComposeLeft, 10, LEFT_ASSOC;
  ComposeRight, 10, LEFT_ASSOC;
  Xor, 10, LEFT_ASSOC;
  Or, 9, LEFT_ASSOC;
  Apply, 9, LEFT_ASSOC;
  DoubleEqual, 8, LEFT_ASSOC;
  BangEqual, 8, LEFT_ASSOC;
  Less, 8, LEFT_ASSOC;
  LessEqual, 8, LEFT_ASSOC;
  Greater, 8, LEFT_ASSOC;
  GreaterEqual, 8, LEFT_ASSOC;
  And, 7, LEFT_ASSOC;
  Semicolon, 1, LEFT_ASSOC;
}

op! {
  UnaryOp; "unary_";
  Minus, 15, LEFT_ASSOC;
  MinusDot, 15, LEFT_ASSOC;
  Not, 15, LEFT_ASSOC;
}

pub const TYPE_STAR_PREC: Precedence = 15;

op! {
  BinaryTypeOp; "";
  Arrow, 5, RIGHT_ASSOC;
}

#[allow(unused)]
impl BinaryOp {
    pub fn parameter_type(&self) -> Type {
        match self {
            BinaryOp::Minus
            | BinaryOp::Star
            | BinaryOp::Slash
            | BinaryOp::Percent
            | BinaryOp::Plus => Type::Integer,
            BinaryOp::PlusDot | BinaryOp::StarDot | BinaryOp::SlashDot | BinaryOp::MinusDot => {
                Type::Real
            }
            BinaryOp::And | BinaryOp::Xor | BinaryOp::Or => Type::Boolean,
            BinaryOp::DoubleEqual
            | BinaryOp::BangEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual => Type::Variable(0),
            BinaryOp::Apply | BinaryOp::Semicolon => Type::Variable(0),
            BinaryOp::ComposeRight | BinaryOp::ComposeLeft => {
                Type::func(Type::Variable(0), Type::Variable(1))
            }
        }
    }

    pub fn return_type(&self) -> Type {
        match self {
            BinaryOp::Minus
            | BinaryOp::Star
            | BinaryOp::Slash
            | BinaryOp::Percent
            | BinaryOp::Plus => Type::Integer,
            BinaryOp::PlusDot | BinaryOp::StarDot | BinaryOp::SlashDot | BinaryOp::MinusDot => {
                Type::Real
            }
            BinaryOp::DoubleEqual
            | BinaryOp::BangEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual
            | BinaryOp::And
            | BinaryOp::Xor
            | BinaryOp::Or => Type::Boolean,
            BinaryOp::Apply | BinaryOp::Semicolon => Type::Variable(1),
            BinaryOp::ComposeLeft | BinaryOp::ComposeRight => {
                Type::func(Type::Variable(0), Type::Variable(2))
            }
        }
    }

    pub fn get_type(&self) -> Type {
        use BinaryOp::*;
        use Type as t;
        match self {
            Semicolon => t::func(
                Type::Variable(0),
                Type::func(Type::Variable(1), Type::Variable(1)),
            ),
            Apply => t::func(
                Type::Variable(0),
                Type::func(
                    Type::func(Type::Variable(0), Type::Variable(1)),
                    Type::Variable(1),
                ),
            ),
            ComposeRight => t::curry(
                &[
                    t::func(t::Variable(0), t::Variable(1)),
                    t::func(t::Variable(1), t::Variable(2)),
                    t::Variable(0),
                ],
                t::Variable(1),
            ),
            ComposeLeft => t::curry(
                &[
                    t::func(t::Variable(1), t::Variable(2)),
                    t::func(t::Variable(0), t::Variable(1)),
                    t::Variable(0),
                ],
                t::Variable(1),
            ),
            op => Type::curry(
                &[op.parameter_type(), op.parameter_type()],
                op.return_type(),
            ),
        }
    }
}

#[allow(unused)]
impl UnaryOp {
    pub fn get_type(&self) -> Type {
        Type::func(self.parameter_type(), self.parameter_type())
    }

    pub fn parameter_type(&self) -> Type {
        match self {
            UnaryOp::Minus => Type::Integer,
            UnaryOp::MinusDot => Type::Real,
            UnaryOp::Not => Type::Boolean,
        }
    }

    pub fn return_type(&self) -> Type {
        self.parameter_type()
    }
}
