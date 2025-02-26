use crate::ir::{
  ConstValue,
  types::{Primitive, Type},
};

use super::{Mangle, mangle_builtin};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Builtin {
  Type,
  PrintString,
  PrintGlyph,
  PrintInteger,
  PrintBoolean,
  PrintType,
}

impl Builtin {
  pub const ALL: [Builtin; 6] = [
    Self::Type,
    Self::PrintString,
    Self::PrintGlyph,
    Self::PrintInteger,
    Self::PrintBoolean,
    Self::PrintType,
  ];

  pub fn to_string(&self) -> &'static str {
    match self {
      Builtin::Type => "type",
      Builtin::PrintString => "print_string",
      Builtin::PrintGlyph => "pring_glyph",
      Builtin::PrintInteger => "print_integer",
      Builtin::PrintBoolean => "print_boolean",
      Builtin::PrintType => "print_type",
    }
  }

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
      Builtin::PrintString
      | Builtin::PrintGlyph
      | Builtin::PrintInteger
      | Builtin::PrintBoolean
      | Builtin::PrintType => {
        ConstValue::Function(mangle_builtin(self.to_string()))
      },
      Builtin::Type => ConstValue::Type(Type::Type),
    }
  }

  pub fn type_(&self) -> Type {
    use Primitive as p;
    let param = match self {
      Builtin::PrintString => p::string.promote(),
      Builtin::PrintGlyph => p::glyph.promote(),
      Builtin::PrintInteger => p::integer.promote(),
      Builtin::PrintBoolean => p::boolean.promote(),
      Builtin::PrintType => Type::Type,
      Builtin::Type => return Type::Type,
    };
    Type::Function {
      param_types: vec![param],
      return_type: Box::new(p::nothing.promote()),
    }
  }

  pub fn sanitary(&self) -> bool {
    self != &Self::PrintType
  }
}
