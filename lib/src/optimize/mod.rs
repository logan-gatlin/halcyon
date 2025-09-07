mod redundancy;
mod tail_call;

use redundancy::*;
use tail_call::*;

use crate::ir::{IrModule, IrNode};

#[derive(Debug, Clone, sx::SXRepr)]
pub enum CallOptimization {
    /// No optimizations
    None,
    /// Possible to tail call optimize
    Tail,
}

impl Default for CallOptimization {
    fn default() -> Self {
        Self::None
    }
}

pub fn optimize_ir(module: &mut IrModule) {
    tail_call(module);
}
