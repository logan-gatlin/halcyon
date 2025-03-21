use crate::compile::assembly::*;

use super::*;

pub const GLOBAL_SCOPE_MANGLE: &str = "_global";

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
      }
      //ConstValue::Function(mangle_builtin(self.to_string())),
      Self::Type => ConstValue::Type(Type::Type),
      Self::Nothing => ConstValue::Type(nothing.promote()),
      Self::Unreachable => ConstValue::Type(unreachable.promote()),
      Self::Integer => ConstValue::Type(integer.promote()),
      Self::Real => ConstValue::Type(real.promote()),
      Self::Boolean => ConstValue::Type(boolean.promote()),
      Self::String => ConstValue::Type(string.promote()),
      Self::Glyph => ConstValue::Type(glyph.promote()),
    }
  }

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

  pub fn import(&self) -> Option<Wasm> {
    match self {
      Builtin::PrintString => Some(Wasm::Import {
        ns1: "js".to_string(),
        ns2: "print_string".to_string(),
        object: Wasm::Function {
          ident: "_print_string".into(),
          params: vec![("".into(), WasmType::I32), ("".into(), WasmType::I32)],
          body: vec![],
          results: vec![],
        }
        .into(),
      }),
      Builtin::PrintReal => Some(Wasm::Import {
        ns1: "js".to_string(),
        ns2: "print_real".to_string(),
        object: Wasm::Function {
          ident: "_print_real".into(),
          params: vec![("".into(), WasmType::F64)],
          body: vec![],
          results: vec![],
        }
        .into(),
      }),
      Builtin::PrintGlyph => Some(Wasm::Import {
        ns1: "js".to_string(),
        ns2: "print_glyph".to_string(),
        object: Wasm::Function {
          ident: "_print_glyph".into(),
          params: vec![("".into(), WasmType::I32)],
          body: vec![],
          results: vec![],
        }
        .into(),
      }),
      Builtin::PrintInteger => Some(Wasm::Import {
        ns1: "js".to_string(),
        ns2: "print_integer".to_string(),
        object: Wasm::Function {
          ident: "_print_integer".into(),
          params: vec![("".into(), WasmType::I64)],
          body: vec![],
          results: vec![],
        }
        .into(),
      }),
      Builtin::PrintBoolean => Some(Wasm::Import {
        ns1: "js".to_string(),
        ns2: "print_boolean".to_string(),
        object: Wasm::Function {
          ident: "_print_boolean".into(),
          params: vec![("".into(), WasmType::I32)],
          body: vec![],
          results: vec![],
        }
        .into(),
      }),
      _ => None,
    }
  }
}
