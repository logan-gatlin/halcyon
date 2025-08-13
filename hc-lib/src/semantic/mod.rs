mod checking;
mod constraint;
mod environment;
mod inference;
mod types;
use std::collections::HashMap;

use checking::*;
use inference::*;

pub use inference::infer_types as type_solve;

use crate::lint::*;
pub use constraint::*;
pub use environment::*;
pub use types::*;

use crate::ir::*;

#[derive(Debug, Clone, Default)]
pub struct ModuleInterface {
    pub types: HashMap<Path, TypeRef>,
    pub values: HashMap<Path, TypeRef>,
    pub constructors: HashMap<Path, Constructor>,
}
