use std::path::Path;

use crate::{
  err::*,
  semantic::{self},
  Parser, Statement, Tokenizer,
};

#[derive(Debug, Clone)]
pub struct Module {
  file_name: String,
  source: String,
  pub program: Vec<Statement>,
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
    let statements = Parser::new(tokens);
    let program = semantic::Analyzer::typecheck(statements.collect());
    let mut errors = vec![];
    Self {
      file_name,
      source: source.into(),
      program,
      errors,
    }
  }

  pub fn errors(&self) -> &[Diagnostic] {
    &self.errors
  }

  pub fn ok(&self) -> bool {
    self.errors.len() == 0
  }
}
