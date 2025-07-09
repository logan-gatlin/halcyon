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
    use Primitive::*;
    match self {
      Self::Type => ConstValue::Type(Type::Type),
      Self::Nothing => ConstValue::Type(nothing.promote()),
      Self::Integer => ConstValue::Type(integer.promote()),
      Self::Real => ConstValue::Type(real.promote()),
      Self::Boolean => ConstValue::Type(boolean.promote()),
      Self::String => ConstValue::Type(string.promote()),
      Self::Glyph => ConstValue::Type(glyph.promote()),
    }
  }

  pub fn type_(&self) -> Type {
    Type::Type
  }
}
