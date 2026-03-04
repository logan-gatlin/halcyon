use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enum_iterator::Sequence)]
pub enum CoreTerm {
    EmptyArray,
    ArrayPush,
    ArrayConcat,
    PrintString,
}

impl Symbol for CoreTerm {
    fn path(&self) -> Path {
        match self {
            CoreTerm::EmptyArray => Path::core("array_empty"),
            CoreTerm::ArrayPush => Path::core("array_push"),
            CoreTerm::ArrayConcat => Path::core("array_concat"),
            CoreTerm::PrintString => Path::core("print_string"),
        }
    }

    fn symbol_kind(&self) -> crate::types::symbol_table::SymbolKind {
        SymbolKind::Term(match self {
            CoreTerm::EmptyArray => Type::array().scheme(),
            CoreTerm::ArrayPush => {
                Type::curry(&[
                    Type::v(0),
                    Type::Array(Type::v(0).into()),
                    Type::Array(Type::v(0).into()),
                ])
                .for_all(1)
                .scheme()
            }
            CoreTerm::ArrayConcat => {
                Type::curry(&[
                    Type::Array(Type::v(0).into()),
                    Type::Array(Type::v(0).into()),
                    Type::Array(Type::v(0).into()),
                ])
                .for_all(1)
                .scheme()
            }
            CoreTerm::PrintString => Type::func(Type::String, Type::Unit).scheme(),
        })
    }
}
