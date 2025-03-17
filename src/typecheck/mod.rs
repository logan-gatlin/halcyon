use std::collections::HashSet;

use crate::{
  hlir::{CanonizedModule, types::Type},
  lint::*,
  mlir::{IrPtr, solver::Solution},
};

mod checking;
mod sanitize;

pub struct TypeChecker {
  pub module: CanonizedModule,
  pub solution: Solution,
  pub break_stack: Vec<Type>,
}

impl TypeChecker {
  pub fn typecheck(
    module: CanonizedModule,
    solution: Solution,
  ) -> Result<(CanonizedModule, HashSet<IrPtr>)> {
    let mut this = Self::new(module, solution);
    this.check(0)?;
    let functions = this.sanitize_main()?;
    Ok((this.module, functions))
  }
}
