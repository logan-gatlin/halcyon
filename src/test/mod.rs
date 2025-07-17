use super::*;

macro_rules! test {
  ($($name:ident),*) => {
    $(
      #[test]
      fn $name() {
        let path = "./src/test/".to_string() + stringify!($name) + ".hc";
        let file = std::fs::read_to_string(path).unwrap();
        let linter = Linter::new(file.clone());
        match _compile(&file) {
          Ok(_) => {}
          Err(e) => {
            println!(
              "{}",
              "Failed to Compile".apply_style(Color::Red, Attribute::Underline),
            );
            println!("{}", linter.render(e));
            panic!();
          }
        };
      }
    )*
  }
}

test!(literals, operators, function);
