use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enum_iterator::Sequence)]
pub enum CoreType {
    Unit,
    Integer,
    Real,
    Boolean,
    String,
    Glyph,
    Array,
    Function,
}

impl Symbol for CoreType {
    fn path(&self) -> Path {
        Path::core(match self {
            CoreType::Unit => "unit",
            CoreType::Integer => "integer",
            CoreType::Real => "real",
            CoreType::Boolean => "boolean",
            CoreType::String => "string",
            CoreType::Glyph => "glyph",
            CoreType::Array => "array",
            CoreType::Function => "function",
        })
    }
    fn symbol_kind(&self) -> SymbolKind {
        SymbolKind::Type(match self {
            CoreType::Unit => Type::Unit.def(0),
            CoreType::Integer => Type::Integer.def(0),
            CoreType::Real => Type::Real.def(0),
            CoreType::Boolean => Type::Boolean.def(0),
            CoreType::String => Type::String.def(0),
            CoreType::Glyph => Type::Glyph.def(0),
            CoreType::Array => Type::array().def(1),
            CoreType::Function => Type::function().def(2),
        })
    }
}
