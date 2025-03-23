use std::collections::{HashMap, HashSet};

use crate::{compiler_print, hlir::*, lint::*, mlir::*};

mod checking;
mod sanitize;

pub fn type_check(hlir: &mut HlIrModule, mlir: &MlIrModule) -> Result<()> {
  let mut tc = TypeChecker::new(hlir, mlir);
  tc.check(0)?;
  Ok(())
}

pub fn sanitize(
  hlir: &mut HlIrModule,
  mlir: &MlIrModule,
) -> Result<Option<(HashSet<IrPtr>, Mangle)>> {
  let value = mlir.evaluates_to();
  if let ConstValue::Function {
    name,
    parameters,
    returns,
  } = value
  {
    if parameters.len() != 0 || returns != Primitive::nothing.promote() {
      return Err(lint_nospan(NameLint::InvalidMain));
    }
    let tc = TypeChecker::new(hlir, mlir);
    let ir = tc.sanitize_main(name.clone())?;
    Ok(Some((ir, name)))
  } else {
    compiler_print(format!("{value}"));
    Ok(None)
  }
}

struct TypeChecker<'a> {
  module: &'a mut HlIrModule,
  solution: &'a MlIrModule,
  type_map: HashMap<Mangle, Type>,
  break_stack: Vec<Type>,
}
