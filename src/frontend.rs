use std::path::Path;

use crate::{
  err::*, treewalk::Interpreter, Parser, Statement, StatementKind, Token,
  Tokenizer,
};

#[derive(Debug, Clone)]
pub struct Module {
  file_name: String,
  source: String,
  ast: Vec<Statement>,
  errors: Vec<Diagnostic>,
}

impl Module {
  pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
    let file = std::fs::read(&path).trace(format!(
      "while attempting to open file '{}'",
      &path.as_ref().display()
    ))?;
    let source = String::from_utf8_lossy(&file);
    Ok(Self::from_string(
      format!("{}", path.as_ref().display()),
      source.into(),
    ))
  }

  pub fn from_string(file_name: String, source: String) -> Self {
    let tokens = Tokenizer::new(source.chars()).filter(|t| t.0.is_meaningful());
    let mut ast = vec![];
    let mut errors = vec![];
    for statement in Parser::new(tokens) {
      use StatementKind as s;
      match statement.kind {
        s::Error(e) => errors.push(e),
        _ => ast.push(statement),
      }
    }
    Self {
      file_name,
      source: source.into(),
      ast,
      errors,
    }
  }

  pub fn errors(&self) -> &[Diagnostic] {
    &self.errors
  }

  pub fn ok(&self) -> bool {
    self.errors.len() == 0
  }

  pub fn execute(&self) {
    if !self.ok() {
      for e in self.errors() {
        eprintln!("{e}");
      }
      return;
    }
    Interpreter::new(self.ast.clone().into_iter())
      .run()
      .unwrap();
  }
}
