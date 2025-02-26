use std::collections::HashSet;

use crate::{
  err::*,
  ir::{IrPtr, solver::Solution},
  naming::CanonizedModule,
};

mod checking;
mod sanitize;

pub struct TypeChecker {
  pub module: CanonizedModule,
  pub solution: Solution,
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
