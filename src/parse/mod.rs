macro_rules! it {
  () => {
    &mut StatefulIter<impl Iterator<Item = Token>>
  };
}

mod module;
mod pattern;
mod printing;
mod type_expression;
mod value_expression;

pub use module::*;
pub use pattern::*;
pub use type_expression::*;
pub use value_expression::*;

use multipeek::MultiPeek;

use crate::{lint::*, operator::*, token::*};
use ParseLint::*;
use TokenKind::*;

#[derive(Debug, Clone)]
pub struct Expression<K> {
    pub kind: K,
    pub span: Span,
}

pub struct StatefulIter<I: Iterator<Item = Token>> {
    iter: MultiPeek<I>,
    last_span: Span,
    span_stack: Vec<Span>,
}

impl<I: Iterator<Item = Token>> StatefulIter<I> {
    pub fn start_span(&mut self) {
        let span = match self.peek(0).map(|t| t.1) {
            Some(span) => span,
            None => Span {
                start: self.last_span.start + self.last_span.width,
                width: 1,
            },
        };
        self.span_stack.push(span);
    }

    pub fn end_span(&mut self) -> Span {
        self.span_stack.pop().unwrap()
    }

    pub fn span_after_this(&self) -> Span {
        Span {
            start: self.last_span.start + self.last_span.width,
            width: 1,
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        match self.iter.next() {
            Some(tok) => {
                self.last_span = tok.1;
                self.span_stack.iter_mut().for_each(|s| {
                    *s += tok.1;
                });
                Some(tok)
            }
            None => None,
        }
    }

    pub fn peek(&mut self, n: usize) -> Option<Token> {
        self.iter.peek_nth(n).cloned()
    }

    pub fn eat(&mut self, expect: TokenKind) -> Option<Token> {
        if self.peek(0).map(|t| t.0 == expect) == Some(true) {
            self.next()
        } else {
            None
        }
    }

    pub fn skip(&mut self, n: usize) {
        for _ in 0..n {
            self.next();
        }
    }

    pub fn peek_or_error(&mut self, n: usize, expect: TokenKind) -> Result<Token> {
        if let Some(next) = self.peek(n)
            && next.0 == expect
        {
            Ok(next)
        } else {
            Err(lint(
                ExpectedToken,
                self.span_after_this(),
                [format!("{expect}")],
            ))
        }
    }

    pub fn eat_or_error(&mut self, expect: TokenKind) -> Result<Token> {
        self.eat(expect.clone()).ok_or(lint(
            ExpectedToken,
            self.span_after_this(),
            [format!("{expect}")],
        ))
    }

    pub fn eat_ident(&mut self) -> Result<String> {
        let Token(Identifier(assignee), _) = self.eat_or_error(Identifier("".into()))? else {
            unreachable!();
        };
        Ok(assignee)
    }

    pub fn eat_one_of(&mut self, kinds: impl IntoIterator<Item = TokenKind>) -> Result<usize> {
        let kinds = kinds.into_iter().collect::<Vec<_>>();
        let kinds_str = kinds
            .iter()
            .map(|k| format!("{k}"))
            .collect::<Vec<_>>()
            .join(", ");
        let Some(next) = self.peek(0) else {
            return Err(lint(
                ExpectedOneOf,
                self.last_span,
                [format!("{kinds_str}",)],
            ));
        };
        if let Some(pos) = kinds.iter().position(|k| k == &next.0) {
            self.skip(1);
            Ok(pos)
        } else {
            Err(lint(ExpectedOneOf, next.1, [format!("{kinds_str}")]))
        }
    }

    pub fn report_error(
        &mut self,
        lint_kind: ParseLint,
        context: impl IntoIterator<Item = String>,
    ) -> Lint {
        let span = self.end_span();
        lint(lint_kind, span, context)
    }
}

pub fn parse(iter: impl IntoIterator<Item = Token>) -> Result<Vec<ParsedModule>> {
    let mut iter = StatefulIter {
        iter: multipeek::multipeek(iter),
        last_span: Span { start: 0, width: 1 },
        span_stack: vec![],
    };
    let mut modules = vec![];
    while let Some(tok) = iter.peek(0)
        && tok.0 != EOF
    {
        modules.push(parse_module(&mut iter)?);
    }
    Ok(modules)
}
