/*!
    The `core` module contains symbols that are required by the compiler.
    These include the standard types, operators, and wrappers for important
    WebAssembly functionality.
*/

mod terms;
mod traits;
mod types;

pub use terms::CoreTerm;
pub use traits::{
    CoreImpl,
    CoreTrait,
};
pub use types::CoreType;

use crate::types::TypeScheme;
use enum_iterator::all;

use crate::ir::Path;
use crate::operator::{
    BinaryOp,
    Operator,
    UnaryOp,
};
use crate::types::symbol_table::{
    Symbol,
    SymbolKind,
};
use crate::types::{
    SymbolTable,
    TraitDef,
    Type,
};

pub const CORE_MODULE_NAME: &str = "core";

pub fn compile_core_module(symbols: &mut SymbolTable) {
    all::<CoreTerm>().for_each(|s| {
        symbols.insert(s);
    });
    all::<CoreType>().for_each(|s| {
        symbols.insert(s);
    });
    all::<CoreTrait>().for_each(|s| {
        symbols.insert(s);
    });
}
