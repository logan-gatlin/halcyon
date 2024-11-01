use crate::{Statement, semantic::SymbolTable};

pub struct Analyzer {
  pub table: SymbolTable,
}

impl Analyzer {
  pub fn typecheck(mut statements: Vec<Statement>) -> Vec<Statement> {
    let mut this = Self {
      table: SymbolTable::new(),
    };
    for s in &mut statements {
      *s = *this.naming_pass_stmt(s.clone().into()).unwrap();
    }
    for s in &mut statements {
      *s = *this.bottom_up_stmt(s.clone().into()).unwrap();
    }
    for s in &mut statements {
      *s = *this.top_down_stmt(s.clone().into()).unwrap();
    }
    println!("-----TABLE------");
    println!("{:#?}", this.table.table);
    println!("-----FUNCS------");
    println!("{:#?}", this.table.functions);
    println!("-----STRUCTS----");
    println!("{:#?}", this.table.structs);
    statements
  }
}
