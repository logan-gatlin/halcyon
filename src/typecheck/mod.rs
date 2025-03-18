use std::collections::HashSet;

use crate::{hlir::*, lint::*, mlir::*};

mod checking;
mod sanitize;

pub struct TypeChecker {
  pub module: HlIrModule,
  pub solution: Solution,
  pub break_stack: Vec<Type>,
}

impl TypeChecker {
  pub fn typecheck(module: HlIrModule, solution: Solution) -> Result<(HlIrModule, HashSet<IrPtr>)> {
    let mut this = Self::new(module, solution);
    this.check(0)?;
    let functions = this.sanitize_main()?;
    Ok((this.module, functions))
  }
}
