use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enum_iterator::Sequence)]
pub enum CoreTerm {
    EmptyArray,
    ArrayPush,
    ArrayConcat,
}

impl Symbol for CoreTerm {
    fn path(&self) -> Path {
        match self {
            CoreTerm::EmptyArray => Path::core("array_empty"),
            CoreTerm::ArrayPush => Path::core("array_push"),
            CoreTerm::ArrayConcat => Path::core("array_concat"),
        }
    }

    fn symbol_kind(&self) -> crate::types::symbol_table::SymbolKind {
        SymbolKind::Term(match self {
            CoreTerm::EmptyArray => todo!(),
            CoreTerm::ArrayPush => todo!(),
            CoreTerm::ArrayConcat => todo!(),
        })
    }
}
