//pub mod assembly;
use crate::{ir::Type, token::*};

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
pub const FIELD_PREC: Precedence = 17;
pub const CALL_PREC: Precedence = 12;

// Name, precedence, associativity;
op! {
  BinaryOp;
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
  Semicolon, 1, LEFT_ASSOC;
}

op! {
  UnaryOp;
  Minus, 15, LEFT_ASSOC;
  MinusDot, 15, LEFT_ASSOC;
  Not, 15, LEFT_ASSOC;
}

op! {
  BinaryTypeOp;
  Star, 15, LEFT_ASSOC;
  Plus, 14, LEFT_ASSOC;
  Arrow, 5, RIGHT_ASSOC;
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
    use BinaryOp::*;
    use Type as t;
    let f = |t: Type, r: Type| Type::func(t.clone(), Type::func(t, r));
    match self {
      Minus => f(t::Integer, t::Integer),
      Plus => f(t::Integer, t::Integer),
      Star => f(t::Integer, t::Integer),
      Slash => f(t::Integer, t::Integer),
      Percent => f(t::Integer, t::Integer),
      StarDot => f(t::Real, t::Real),
      SlashDot => f(t::Real, t::Real),
      PlusDot => f(t::Real, t::Real),
      MinusDot => f(t::Real, t::Real),
      Xor => f(t::Boolean, t::Boolean),
      Or => f(t::Boolean, t::Boolean),
      And => f(t::Boolean, t::Boolean),
      DoubleEqual => f(t::TypeVariable(0), Type::Boolean),
      BangEqual => f(t::TypeVariable(0), Type::Boolean),
      Less => f(t::TypeVariable(0), Type::Boolean),
      LessEqual => f(t::TypeVariable(0), Type::Boolean),
      Greater => f(t::TypeVariable(0), Type::Boolean),
      GreaterEqual => f(t::TypeVariable(0), Type::Boolean),
      Semicolon => t::func(
        Type::TypeVariable(0),
        Type::func(Type::TypeVariable(1), Type::TypeVariable(1)),
      ),
      Dot => todo!(),
      Apply => todo!(),
    }
  }
}

impl UnaryOp {
  pub fn get_type(&self) -> Type {
    use UnaryOp::*;
    match self {
      Minus => Type::func(Type::Integer, Type::Integer),
      MinusDot => Type::func(Type::Real, Type::Real),
      Not => Type::func(Type::Boolean, Type::Boolean),
    }
  }
}
