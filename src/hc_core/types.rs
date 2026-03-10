use crate::ir::Path;
use crate::types::symbol_table::{
    Symbol,
    SymbolKind,
};
use crate::types::{
    Type,
    TypeDefinition,
};

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
            CoreType::Unit => "Unit",
            CoreType::Integer => "Integer",
            CoreType::Real => "Real",
            CoreType::Boolean => "Boolean",
            CoreType::String => "String",
            CoreType::Glyph => "Glyph",
            CoreType::Array => "Array",
            CoreType::Function => "Fn",
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

impl CoreType {
    pub fn typedef(&self) -> TypeDefinition {
        match self {
            CoreType::Unit => Type::Unit.def(0),
            CoreType::Integer => Type::Integer.def(0),
            CoreType::Real => Type::Real.def(0),
            CoreType::Boolean => Type::Boolean.def(0),
            CoreType::String => Type::String.def(0),
            CoreType::Glyph => Type::Glyph.def(0),
            CoreType::Array => Type::array().def(1),
            CoreType::Function => Type::function().def(2),
        }
    }
}
