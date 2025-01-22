pub struct Compiler {
  output: String,
}

impl Compiler {
  pub fn new() -> Self {
    Self {
      output: String::new(),
    }
  }
}
