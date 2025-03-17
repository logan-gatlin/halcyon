use crate::compile::assembly::*;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Builtin {
  Type,
  PrintString,
  PrintReal,
  PrintGlyph,
  PrintInteger,
  PrintBoolean,
  PrintType,
}

impl Builtin {
  pub const ALL: [Builtin; 7] = [
    Self::Type,
    Self::PrintString,
    Self::PrintReal,
    Self::PrintGlyph,
    Self::PrintInteger,
    Self::PrintBoolean,
    Self::PrintType,
  ];

  pub fn to_string(&self) -> &'static str {
    match self {
      Builtin::Type => "type",
      Builtin::PrintString => "print_string",
      Builtin::PrintReal => "print_real",
      Builtin::PrintGlyph => "print_glyph",
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
      | Builtin::PrintReal
      | Builtin::PrintInteger
      | Builtin::PrintBoolean
      | Builtin::PrintType => ConstValue::Function(mangle_builtin(self.to_string())),
      Builtin::Type => ConstValue::Type(Type::Type),
    }
  }

  pub fn type_(&self) -> Type {
    use Primitive as p;
    let param = match self {
      Builtin::PrintString => p::string.promote(),
      Builtin::PrintReal => p::real.promote(),
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

  pub fn import(&self) -> Option<Wasm> {
    match self {
      Builtin::PrintType | Builtin::Type => None,
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
    }
  }
}
