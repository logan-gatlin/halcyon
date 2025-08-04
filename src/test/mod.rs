use super::*;

macro_rules! test {
  ($($name:ident),*) => {
    $(
      #[test]
      fn $name() {
        let path = "./src/test/".to_string() + stringify!($name) + ".hc";
        let file = std::fs::read_to_string(path).unwrap();
        compile(&file);
      }
    )*
  }
}

test!(literals, operators, function, control_flow, types, fizzbuzz);
