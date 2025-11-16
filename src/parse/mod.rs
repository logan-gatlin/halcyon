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

const ERR_MSG: &str = "Syntax error";

use multipeek::{MultiPeek, multipeek};

use crate::{LoggerT, Span, Spanned, WithSpan, operator::*, token::*};
use TokenKind::*;

pub type Expression<K> = Spanned<K>;

pub struct Parser<'a, I: Iterator<Item = Token>> {
    pub logger: &'a mut LoggerT,
    pub iter: MultiPeek<I>,
    pub last_span: Span,
}

impl<'a, I: Iterator<Item = Token>> Parser<'a, I> {
    fn peek(&mut self) -> Option<Token> {
        self.iter.peek().cloned()
    }

    fn peek_nth(&mut self, n: usize) -> Option<Token> {
        self.iter.peek_nth(n).cloned()
    }

    fn next(&mut self) -> Option<Token> {
        self.iter.next()
    }

    fn skip(&mut self) {
        if let Some(next) = self.iter.next() {
            self.last_span = next.span;
        }
    }

    fn eat(&mut self, tk: TokenKind) -> Option<Token> {
        if let Some(next) = self.iter.peek().cloned()
            && next.inner == tk
        {
            self.skip();
            Some(next)
        } else {
            None
        }
    }

    fn eat_one_of(&mut self, items: impl IntoIterator<Item = TokenKind>) -> Option<usize> {
        let items = items.into_iter().collect::<Vec<_>>();
        for (id, item) in items.iter().enumerate() {
            if self.iter.peek().is_some_and(|t| &t.inner == item) {
                self.skip();
                return Some(id);
            }
        }
        None
    }

    fn eat_path(&mut self) -> Option<[String; 2]> {
        let first = self.eat_ident()?;
        self.eat(DoubleColon)?;
        let second = self.eat_ident()?;
        Some([first, second])
    }

    fn eat_ident(&mut self) -> Option<String> {
        if let Some(next) = self.iter.peek().cloned()
            && let Identifier(name) = next.inner
        {
            self.skip();
            Some(name)
        } else {
            None
        }
    }

    fn error(&mut self) -> crate::LogBuilder<'_, usize> {
        self.logger.error(ERR_MSG)
    }

    fn error_expected(&mut self, token: TokenKind) -> crate::LogBuilder<'_, usize> {
        self.logger
            .error(ERR_MSG)
            .primary(format!("Expected `{token}` here"), self.last_span)
    }
}

pub fn parse(logger: &mut LoggerT, iter: impl IntoIterator<Item = Token>) -> Vec<ParsedModule> {
    let mut p = Parser {
        iter: multipeek(iter.into_iter().filter(|t| {
            !matches!(
                t.inner,
                TokenKind::LineComment(_) | TokenKind::BlockComment(_)
            )
        })),
        last_span: Span::default(),
        logger,
    };
    let mut modules = vec![];
    while p.peek().is_some() {
        modules.push(parse_module(logger, &mut p));
    }
    modules
}
