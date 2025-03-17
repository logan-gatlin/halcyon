use std::collections::HashSet;

use crate::{
  ir::{IrPtr, solver::Solution, types::Type},
  lint::*,
  naming::CanonizedModule,
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
