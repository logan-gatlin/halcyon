// Core terms are now defined in core.hc via inline wasm.
// This module is kept as a placeholder for the CoreTerm type,
// which is currently empty but may be needed for future primitive terms.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enum_iterator::Sequence)]
pub enum CoreTerm {}

impl Symbol for CoreTerm {
    fn path(&self) -> Path {
        match *self {}
    }

    fn symbol_kind(&self) -> crate::types::symbol_table::SymbolKind {
        match *self {}
    }
}
