use super::*;

macro_rules! it {
  () => {
    &mut MultiPeek<impl Iterator<Item = Token>>
  };
}

fn expression(iter: it!()) {
  use ExpressionKind as e;
  use TokenKind as t;
  let next = iter.peek_nth(0);
}
