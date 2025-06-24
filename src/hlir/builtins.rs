use super::*;
//use crate::compile::*;

pub const GLOBAL_SCOPE_MANGLE: &str = "_global";
/*
macro_rules! count {
    () => (0usize);
    ($x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

macro_rules! builtins {
  ($($name:ident, $repr:literal, $type:expr, $value:expr, $import:expr);*;) => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Builtin {
      $($name),*
    }

    impl Builtin {
      pub const ALL: [Builtin; count!($($name)*,) - 1] = [$(Builtin::$name),*];

      pub fn to_string(&self) -> &'static str {
        match self {
          $(Self::$name => $repr ),*
        }
      }

      pub fn to_mangle(&self) -> Mangle {
        match self {
          $(Self::$name => mangle_builtin($repr)),*
        }
      }

      pub fn type_of(&self) -> Type {
        match self {
          $(Self::$name => $type),*
        }
      }

      pub fn value(&self) -> ConstValue {
        match self {
          $(Self::$name => $value),*
        }
      }

      pub fn import(&self) -> Option<Wasm> {
        None
      }
    }
  };
}

const fn p(primitive: Primitive) -> Type {
  Type::Primitive(primitive)
}

const fn c(primitive: Primitive) -> ConstValue {
  ConstValue::Type(Type::Primitive(primitive))
}

use ConstValue as c;
use Primitive::*;
use Type as t;
*/
/*
builtins! {
  // Primitive types
  Type, "type", t::Type, c::Type(t::Type), None;
  Nothing, "nothing", p(nothing), c(nothing), None;
  Unreachable, "unreachable", p(unreachable), c(unreachable), None;
  Integer, "integer", p(integer), c(integer), None;
  Real, "real", p(real), c(real), None;
  Boolean, "bool", p(boolean), c(boolean), None;
  String, "string", p(string), c(string), None;
  Glyph, "glyph", p(glyph), c(glyph), None;
  // Print functions
  PrintString, "print_string",
    Type::Function {
      param_types: vec![p(string)],
      return_type: p(nothing).into(),
    },
    ConstValue::Function {
      name: mangle_builtin("print_string"),
      parameters: vec![p(string)],
      returns: p(nothing).into(),
    }, None;
}
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Builtin {
  Type,
  Nothing,
  Unreachable,
  Integer,
  Real,
  Boolean,
  String,
  Glyph,
  PrintString,
  PrintReal,
  PrintGlyph,
  PrintInteger,
  PrintBoolean,
  PrintType,
}

impl Builtin {
  pub const ALL: [Builtin; 14] = [
    Self::Type,
    Self::Nothing,
    Self::Unreachable,
    Self::Integer,
    Self::Real,
    Self::Boolean,
    Self::String,
    Self::Glyph,
    Self::PrintString,
    Self::PrintReal,
    Self::PrintGlyph,
    Self::PrintInteger,
    Self::PrintBoolean,
    Self::PrintType,
  ];

  pub fn to_string(&self) -> &'static str {
    match self {
      Self::Type => "type",
      Self::Nothing => "nothing",
      Self::Unreachable => "unreachable",
      Self::Integer => "integer",
      Self::Real => "real",
      Self::Boolean => "boolean",
      Self::String => "string",
      Self::Glyph => "glyph",
      Self::PrintString => "print_string",
      Self::PrintReal => "print_real",
      Self::PrintGlyph => "print_glyph",
      Self::PrintInteger => "print_integer",
      Self::PrintBoolean => "print_boolean",
      Self::PrintType => "print_type",
    }
  }

  pub fn to_mangle(&self) -> String {
    mangle_builtin(self.to_string())
  }

  pub fn from_mangle(mangle: &Mangle) -> Option<Self> {
    for b in Self::ALL {
      if &mangle_builtin(b.to_string()) == mangle {
        return Some(b);
      }
    }
    None
  }

  /*
  pub fn value(&self) -> ConstValue {
    use Primitive::*;
    match self {
      Self::PrintString
      | Self::PrintGlyph
      | Self::PrintReal
      | Self::PrintInteger
      | Self::PrintBoolean
      | Self::PrintType => {
        let Type::Function {
          param_types,
          return_type,
        } = self.type_()
        else {
          panic!()
        };
        ConstValue::Function {
          name: mangle_builtin(self.to_string()),
          parameters: param_types,
          returns: *return_type,
        }
      },
      //ConstValue::Function(mangle_builtin(self.to_string())),
      Self::Type => ConstValue::Type(Type::Type),
      Self::Nothing => ConstValue::Type(nothing.promote()),
      Self::Unreachable => ConstValue::Type(never.promote()),
      Self::Integer => ConstValue::Type(integer.promote()),
      Self::Real => ConstValue::Type(real.promote()),
      Self::Boolean => ConstValue::Type(boolean.promote()),
      Self::String => ConstValue::Type(string.promote()),
      Self::Glyph => ConstValue::Type(glyph.promote()),
    }
  }
  */

  pub fn type_(&self) -> Type {
    use Primitive as p;
    let param = match self {
      Self::PrintString => p::string.promote(),
      Self::PrintReal => p::real.promote(),
      Self::PrintGlyph => p::glyph.promote(),
      Self::PrintInteger => p::integer.promote(),
      Self::PrintBoolean => p::boolean.promote(),
      Self::PrintType => Type::Type,
      Self::Type
      | Self::Nothing
      | Self::Unreachable
      | Self::Integer
      | Self::Real
      | Self::Boolean
      | Self::String
      | Self::Glyph => return Type::Type,
    };
    Type::Function {
      param_types: vec![param],
      return_type: Box::new(p::nothing.promote()),
    }
  }

  pub fn sanitary(&self) -> bool {
    self != &Self::PrintType
  }
  /*

  pub fn import(&self) -> Option<Wasm> {
    match self {
      Builtin::PrintString => Some(Wasm::Import {
        ns1: "js".to_string(),
        ns2: self.to_string().into(),
        object: Wasm::Function {
          ident: self.to_mangle(),
          params: vec![("".into(), WasmType::I32), ("".into(), WasmType::I32)],
          body: vec![],
          results: vec![],
        }
        .into(),
      }),
      Builtin::PrintReal => Some(Wasm::Import {
        ns1: "js".to_string(),
        ns2: self.to_string().into(),
        object: Wasm::Function {
          ident: self.to_mangle(),
          params: vec![("".into(), WasmType::F64)],
          body: vec![],
          results: vec![],
        }
        .into(),
      }),
      Builtin::PrintGlyph => Some(Wasm::Import {
        ns1: "js".to_string(),
        ns2: self.to_string().into(),
        object: Wasm::Function {
          ident: self.to_mangle(),
          params: vec![("".into(), WasmType::I32)],
          body: vec![],
          results: vec![],
        }
        .into(),
      }),
      Builtin::PrintInteger => Some(Wasm::Import {
        ns1: "js".to_string(),
        ns2: self.to_string().into(),
        object: Wasm::Function {
          ident: self.to_mangle(),
          params: vec![("".into(), WasmType::I64)],
          body: vec![],
          results: vec![],
        }
        .into(),
      }),
      Builtin::PrintBoolean => Some(Wasm::Import {
        ns1: "js".to_string(),
        ns2: self.to_string().into(),
        object: Wasm::Function {
          ident: self.to_mangle(),
          params: vec![("".into(), WasmType::I32)],
          body: vec![],
          results: vec![],
        }
        .into(),
      }),
      _ => None,
    }
  }
  */
}
