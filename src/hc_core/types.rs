use crate::types::{
    TraitDef,
    Type,
};

use super::*;

#[derive(Copy, Clone, enum_iterator::Sequence)]
pub enum CoreTypes {
    Unit,
    Integer,
    Real,
    Boolean,
    String,
    Glyph,
    Array,
    Function,
}

impl CoreTypes {
    pub fn path(&self) -> Path {
        Path::core(match self {
            CoreTypes::Unit => "unit",
            CoreTypes::Integer => "integer",
            CoreTypes::Real => "real",
            CoreTypes::Boolean => "boolean",
            CoreTypes::String => "string",
            CoreTypes::Glyph => "glyph",
            CoreTypes::Array => "array",
            CoreTypes::Function => "function",
        })
    }
    pub fn type_(&self) -> Type {
        match self {
            CoreTypes::Unit => Type::Unit,
            CoreTypes::Integer => Type::Integer,
            CoreTypes::Real => Type::Real,
            CoreTypes::Boolean => Type::Boolean,
            CoreTypes::String => Type::String,
            CoreTypes::Glyph => Type::Glyph,
            CoreTypes::Array => Type::ForAll(Type::Array(Type::TypeVar(0).into()).into()),
            CoreTypes::Function => {
                Type::ForAll(
                    Type::ForAll(
                        Type::Function(Type::TypeVar(1).into(), Type::TypeVar(0).into()).into(),
                    )
                    .into(),
                )
            }
        }
    }
}

#[derive(Copy, Clone, enum_iterator::Sequence)]
pub enum CoreTraits {
    Equal,
    Compare,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

impl CoreTraits {
    pub fn path(&self) -> Path {
        Path::core(match self {
            CoreTraits::Equal => "equal",
            CoreTraits::Compare => "compare",
            CoreTraits::Add => "add",
            CoreTraits::Subtract => "subtract",
            CoreTraits::Multiply => "multiply",
            CoreTraits::Divide => "divide",
            CoreTraits::Remainder => "remainder",
        })
    }
    pub fn parameters(&self) -> usize {
        1
    }

    pub fn trait_(&self) -> TraitDef {
        TraitDef {
            name: self.path(),
            parameters: self.parameters(),
        }
    }
}
