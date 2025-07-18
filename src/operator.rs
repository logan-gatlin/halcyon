//pub mod assembly;
use crate::{hlir::Type, token::*};

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
  Or, 9, LEFT_ASSOC;
  Apply, 9, LEFT_ASSOC;
  DoubleEqual, 8, LEFT_ASSOC;
  BangEqual, 8, LEFT_ASSOC;
  Less, 8, LEFT_ASSOC;
  LessEqual, 8, LEFT_ASSOC;
  Greater, 8, LEFT_ASSOC;
  GreaterEqual, 8, LEFT_ASSOC;
  And, 7, LEFT_ASSOC;
  Arrow, 5, RIGHT_ASSOC;
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
  pub const POLYMORPHIC: [Self; 6] = [
    Self::DoubleEqual,
    Self::BangEqual,
    Self::LessEqual,
    Self::GreaterEqual,
    Self::Less,
    Self::Greater,
  ];

  pub fn get_curry_type(&self) -> Type {
    let Type::Function(_, box Type::Function(a, b)) = self.get_type() else {
      panic!()
    };
    Type::Function(a, b)
  }

  pub fn get_type(&self) -> Type {
    use Type as t;
    let f = |t: Type, r: Type| Type::func(t.clone(), Type::func(t, r));
    match self {
      BinaryOp::Minus => f(t::Integer, t::Integer),
      BinaryOp::Plus => f(t::Integer, t::Integer),
      BinaryOp::Star => f(t::Integer, t::Integer),
      BinaryOp::Slash => f(t::Integer, t::Integer),
      BinaryOp::Percent => f(t::Integer, t::Integer),
      BinaryOp::StarDot => f(t::Real, t::Real),
      BinaryOp::SlashDot => f(t::Real, t::Real),
      BinaryOp::PlusDot => f(t::Real, t::Real),
      BinaryOp::MinusDot => f(t::Real, t::Real),
      BinaryOp::Xor => f(t::Boolean, t::Boolean),
      BinaryOp::Or => f(t::Boolean, t::Boolean),
      BinaryOp::And => f(t::Boolean, t::Boolean),
      BinaryOp::DoubleEqual => f(t::TypeVariable(0), Type::Boolean),
      BinaryOp::BangEqual => f(t::TypeVariable(0), Type::Boolean),
      BinaryOp::Less => f(t::TypeVariable(0), Type::Boolean),
      BinaryOp::LessEqual => f(t::TypeVariable(0), Type::Boolean),
      BinaryOp::Greater => f(t::TypeVariable(0), Type::Boolean),
      BinaryOp::GreaterEqual => f(t::TypeVariable(0), Type::Boolean),
      BinaryOp::Arrow => f(t::Type, t::Type),
      BinaryOp::Semicolon => t::func(
        Type::TypeVariable(0),
        Type::func(Type::TypeVariable(1), Type::TypeVariable(1)),
      ),
      BinaryOp::Dot => todo!(),
      BinaryOp::Comma => todo!(),
      BinaryOp::Apply => todo!(),
    }
  }
}

impl UnaryOp {
  pub fn get_type(&self) -> Type {
    match self {
      UnaryOp::Minus => Type::func(Type::Integer, Type::Integer),
      UnaryOp::MinusDot => Type::func(Type::Real, Type::Real),
      UnaryOp::Not => Type::func(Type::Boolean, Type::Boolean),
    }
  }
}
