use crate::ir::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreSymbol {
    ArrayEmpty,
    ArrayConcat,
    ArrayPush,
    Default,
    PrintString,
    TraitAdd,
    TraitSubtract,
    TraitMultiply,
    TraitDivide,
    TraitRemainder,
    TraitBitwise,
    TraitEqual,
    TraitCompare,
    TraitDefault,
}

impl CoreSymbol {
    pub fn path(&self) -> Path {
        Path::core(match self {
            CoreSymbol::ArrayEmpty => "array_empty",
            CoreSymbol::ArrayConcat => "array_concat",
            CoreSymbol::ArrayPush => "array_push",
            CoreSymbol::Default => "default",
            CoreSymbol::PrintString => "print_string",
            CoreSymbol::TraitAdd => "ops::Add",
            CoreSymbol::TraitSubtract => "ops::Subtract",
            CoreSymbol::TraitMultiply => "ops::Multiply",
            CoreSymbol::TraitDivide => "ops::Divide",
            CoreSymbol::TraitRemainder => "ops::Remainder",
            CoreSymbol::TraitBitwise => "ops::Bitwise",
            CoreSymbol::TraitEqual => "ops::Equal",
            CoreSymbol::TraitCompare => "ops::Compare",
            CoreSymbol::TraitDefault => "Default",
        })
    }
}
