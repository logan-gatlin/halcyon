use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Builtin {
  Type,
  Nothing,
  Integer,
  Real,
  Boolean,
  String,
  Glyph,
}

impl Builtin {
  pub const ALL: [Builtin; 7] = [
    Self::Type,
    Self::Nothing,
    Self::Integer,
    Self::Real,
    Self::Boolean,
    Self::String,
    Self::Glyph,
  ];

  pub fn to_string(&self) -> &'static str {
    match self {
      Self::Type => "type",
      Self::Nothing => "nothing",
      Self::Integer => "integer",
      Self::Real => "real",
      Self::Boolean => "boolean",
      Self::String => "string",
      Self::Glyph => "glyph",
    }
  }

  pub fn to_mangle(&self) -> String {
    mangle_builtin(self.to_string())
  }

  #[allow(dead_code)]
  pub fn from_mangle(mangle: &Mangle) -> Option<Self> {
    for b in Self::ALL {
      if &mangle_builtin(b.to_string()) == mangle {
        return Some(b);
      }
    }
    None
  }

  pub fn value(&self) -> ConstValue {
    match self {
      Self::Type => ConstValue::Type(Type::Type),
      Self::Nothing => ConstValue::Type(Type::Unit),
      Self::Integer => ConstValue::Type(Type::Integer),
      Self::Real => ConstValue::Type(Type::Real),
      Self::Boolean => ConstValue::Type(Type::Boolean),
      Self::String => ConstValue::Type(Type::String),
      Self::Glyph => ConstValue::Type(Type::Glyph),
    }
  }

  pub fn type_(&self) -> Type {
    Type::Type
  }
}
