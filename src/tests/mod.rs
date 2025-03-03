use crate::{
  parse::{Parser, StatementKind},
  token::Tokenizer,
};

#[test]
fn parse_test() {
  let src = r#"
        a + b - -c * d;
        () {};
        (a: integer + real) -> -nothing& {};
        const_assign :: foo;
        const_assign : type_assert : foo;
        rt_assign = foo;
        rt_assign : type_assert = foo;
        if condition {foo} else if other_condition {} else {};
        loop i: bar {};
        struct {
          field: type,
          field2: type2
        };
    "#;
  let tokenizer = Tokenizer::new(src.chars()).filter(|t| t.0.is_meaningful());
  let parse_tree = Parser::new(tokenizer).collect::<Vec<_>>();
  for node in parse_tree {
    if let StatementKind::Error(e) = &node.kind {
      panic!("{}", e)
    }
  }
}
