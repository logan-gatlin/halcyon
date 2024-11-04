use crate::{Statement, semantic::SymbolTable};

pub struct Analyzer {
  pub table: SymbolTable,
}

impl Analyzer {
  pub fn new() -> Self {
    Self {
      table: SymbolTable::new(),
    }
  }

  pub fn typecheck(
    &mut self,
    mut statements: Vec<Statement>,
  ) -> Vec<Statement> {
    for s in &mut statements {
      *s = *self.naming_pass_stmt(s.clone().into()).unwrap();
    }
    for s in &mut statements {
      *s = *self.bottom_up_stmt(s.clone().into()).unwrap();
    }
    for s in &mut statements {
      *s = *self.top_down_stmt(s.clone().into()).unwrap();
    }
    println!("-----TABLE------");
    println!("{:#?}", self.table.table);
    println!("-----FUNCS------");
    println!("{:#?}", self.table.functions);
    println!("-----STRUCTS----");
    println!("{:#?}", self.table.structs);
    statements
  }
}
