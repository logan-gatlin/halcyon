use std::{io::Write, path::Path};

use crate::{
  Parser, Tokenizer,
  err::*,
  semantic::{self},
};

#[derive(Debug, Clone)]
pub struct Module {
  file_name: String,
  source: String,
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
    Self {
      file_name,
      source: source.into(),
    }
  }
  /*
  pub fn write_to(&self, path: impl AsRef<Path>) {
    let watpath = path.as_ref().to_owned().with_extension("wat");
    let mut file = std::fs::File::create(watpath).unwrap();
    file.write_all(&self.wat.as_bytes()).unwrap();

    let wasmpath = path.as_ref().to_owned().with_extension("wasm");
    let mut file = std::fs::File::create(wasmpath).unwrap();
    file.write_all(&self.wasm).unwrap();
  }
  */
}
