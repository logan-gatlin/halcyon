macro_rules! p {
    () => {
        &mut Parser<impl Iterator<Item = Token>>
    }
}

mod module;
mod pattern;
mod type_expression;
mod value_expression;

pub use module::*;
pub use pattern::*;
pub use type_expression::*;
pub use value_expression::*;

use multipeek::{MultiPeek, multipeek};

use crate::{Logger, Span, Spanned, WithSpan, operator::*, token::*};
use TokenKind::*;

pub type Expression<K> = Spanned<K>;

type PResult<T> = Result<T, Spanned<String>>;

pub struct Parser<I: Iterator<Item = Token>> {
    iter: MultiPeek<I>,
    last_span: Span,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter: multipeek(iter),
            last_span: Span { start: 0, width: 0 },
        }
    }

    fn peek(&mut self) -> PResult<Token> {
        self.iter.peek().cloned().ok_or_else(|| {
            "Unexpected end of input"
                .to_string()
                .with_span(self.last_span)
        })
    }

    fn peek_nth(&mut self, n: usize) -> PResult<Token> {
        self.iter.peek_nth(n).cloned().ok_or_else(|| {
            "Unexpected end of input"
                .to_string()
                .with_span(self.last_span)
        })
    }

    fn next(&mut self) -> PResult<Token> {
        self.iter.next().ok_or_else(|| {
            "Unexpected end of input"
                .to_string()
                .with_span(self.last_span)
        })
    }

    fn skip(&mut self) {
        if let Some(next) = self.iter.next() {
            self.last_span = next.span;
        }
    }

    fn eat(&mut self, tk: TokenKind) -> PResult<()> {
        if let Some(next) = self.iter.peek()
            && next.inner == tk
        {
            self.skip();
            Ok(())
        } else {
            Err(format!("Expected {tk} after this").with_span(self.last_span))
        }
    }

    fn eat_one_of(&mut self, items: impl IntoIterator<Item = TokenKind>) -> PResult<usize> {
        let items = items.into_iter().collect::<Vec<_>>();
        for (id, item) in items.iter().enumerate() {
            if self.iter.peek().is_some_and(|t| &t.inner == item) {
                self.skip();
                return Ok(id);
            }
        }
        return Err(format!(
            "Expected one of these: {}",
            items
                .iter()
                .map(|t| format!("{t}"))
                .collect::<Vec<_>>()
                .join(",")
        )
        .with_span(self.last_span));
    }

    fn eat_ident(&mut self) -> PResult<String> {
        if let Some(next) = self.iter.peek().cloned()
            && let Identifier(name) = next.inner
        {
            self.skip();
            Ok(name)
        } else {
            Err(format!("Expected identifier after this").with_span(self.last_span))
        }
    }
}

pub fn parse(logger: &mut Logger, iter: impl IntoIterator<Item = Token>) -> Vec<ParsedModule> {
    let mut p = Parser {
        iter: multipeek(iter.into_iter().filter(|t| {
            !matches!(
                t.inner,
                TokenKind::LineComment(_) | TokenKind::BlockComment(_)
            )
        })),
        last_span: Span::default(),
    };
    let mut modules = vec![];
    while p.peek().is_ok() {
        modules.push(parse_module(logger, &mut p));
    }
    modules
}
