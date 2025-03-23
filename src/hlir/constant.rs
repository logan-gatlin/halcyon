use super::*;

#[derive(Clone, Debug)]
pub enum ConstValue {
  Nothing,
  Never,
  Integer(i64),
  Real(f64),
  Boolean(bool),
  String {
    address: PtrT,
    length: PtrT,
  },
  Glyph(char),
  Function {
    name: Mangle,
    parameters: Vec<Type>,
    returns: Type,
  },
  StructLiteral {
    member_names: Vec<String>,
    member_values: Vec<ConstValue>,
  },
  Type(Type),
}

impl ConstValue {
  pub fn type_of(&self) -> Type {
    use Primitive::*;
    match self {
      ConstValue::Nothing => nothing.promote(),
      ConstValue::Never => unreachable.promote(),
      ConstValue::Integer(_) => integer.promote(),
      ConstValue::Real(_) => real.promote(),
      ConstValue::Boolean(_) => boolean.promote(),
      ConstValue::String { .. } => string.promote(),
      ConstValue::Glyph(_) => glyph.promote(),
      ConstValue::Function {
        parameters,
        returns,
        ..
      } => Type::Function {
        param_types: parameters.clone(),
        return_type: returns.clone().into(),
      },
      ConstValue::StructLiteral {
        member_names,
        member_values,
      } => Type::Struct {
        member_names: member_names.clone(),
        member_types: member_values.iter().map(|v| v.type_of()).collect(),
      },
      ConstValue::Type(_) => Type::Type,
    }
  }
}

impl std::fmt::Display for ConstValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ConstValue::Nothing => write!(f, "()"),
      ConstValue::Never => write!(f, "!"),
      ConstValue::String { .. } => write!(f, "(string)"),
      ConstValue::Function { name, .. } => write!(f, "{name}"),
      ConstValue::StructLiteral { .. } => write!(f, "(struct)"),
      ConstValue::Type(val) => write!(f, "{val}"),
      ConstValue::Integer(val) => write!(f, "{val}"),
      ConstValue::Real(val) => write!(f, "{val}"),
      ConstValue::Glyph(val) => write!(f, "{val}"),
      ConstValue::Boolean(val) => write!(f, "{val}"),
    }
  }
}
