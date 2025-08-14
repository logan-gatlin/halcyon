use crate::semantic::{Type, TypeRef};

#[derive(Clone, Debug)]
pub enum ConstValue {
    Unit,
    Integer(i64),
    Real(f64),
    Boolean(bool),
    String(String),
    Glyph(char),
}

impl std::fmt::Display for ConstValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstValue::Unit => write!(f, "()"),
            ConstValue::String { .. } => write!(f, "<string>"),
            ConstValue::Integer(val) => write!(f, "{val}"),
            ConstValue::Real(val) => write!(f, "{val}"),
            ConstValue::Glyph(val) => write!(f, "{val}"),
            ConstValue::Boolean(val) => write!(f, "{val}"),
        }
    }
}

impl ConstValue {
    pub fn type_of(&self) -> TypeRef {
        match self {
            ConstValue::Unit => Type::Unit,
            ConstValue::Integer(_) => Type::Integer,
            ConstValue::Real(_) => Type::Real,
            ConstValue::Boolean(_) => Type::Boolean,
            ConstValue::String(_) => Type::String,
            ConstValue::Glyph(_) => Type::Glyph,
        }
    }
}
