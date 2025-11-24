mod module;
mod pattern;
mod type_expression;
mod value_expression;

pub use module::*;
pub use pattern::*;
pub use type_expression::*;
pub use value_expression::*;

const ERR_MSG: &str = "Syntax error";

use multipeek::{
    MultiPeek,
    multipeek,
};

use crate::operator::*;
use crate::token::*;
use crate::{
    Logger,
    Span,
    Spanned,
    WithContext,
    WithSpan,
};
pub use RecoveryBehavior::*;
use TokenKind::*;

pub type Expression<K> = Spanned<K>;

/// How to recover from a parsing error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryBehavior {
    /// No recovery necessary
    NoRecovery,
    /// Skip until this exact token is next
    UntilKind(TokenKind),
    /// Skip until this category of token is next
    UntilCategory(TokenCategory),
    /// Until the beginning of the next module statement
    UntilNextStatement,
}

type Result<T> = std::result::Result<T, RecoveryBehavior>;

pub struct Parser<'a, I: Iterator<Item = Token>> {
    logger: &'a mut Logger,
    iter: MultiPeek<I>,
    last_span: Span,
}

impl<'a, I: Iterator<Item = Token>> Parser<'a, I> {
    pub fn new(
        logger: &'a mut Logger,
        iter: impl IntoIterator<IntoIter = I>,
    ) -> Self {
        Self {
            last_span: logger.new_span(0, 0),
            logger,
            iter: multipeek(iter),
        }
    }
    pub fn peek(&mut self) -> Option<Token> {
        self.iter.peek().cloned()
    }
    pub fn peek_or_err(&mut self) -> Option<Token> {
        let res = self.peek();
        if res.is_none() {
            let span = self.last_span;
            self.error()
                .primary("Unexpected end of source code", span)
                .done();
        }
        res
    }
    pub fn peek_nth(
        &mut self,
        n: usize,
    ) -> Option<Token> {
        self.iter.peek_nth(n).cloned()
    }
    pub fn next_token(&mut self) -> Option<Token> {
        let next = self.iter.next();
        if let Some(Spanned { span, .. }) = next {
            self.last_span = span;
        }
        next
    }
    pub fn next_token_or_err(&mut self) -> Option<Token> {
        let res = self.next_token();
        if res.is_none() {
            let span = self.last_span;
            self.error()
                .primary("Unexpected end of source code", span)
                .done();
        }
        res
    }
    pub fn skip(&mut self) {
        if let Some(next) = self.iter.next() {
            self.last_span = next.span;
        }
    }
    pub fn eat(
        &mut self,
        tk: &TokenKind,
    ) -> Option<Token> {
        if let Some(next) = self.iter.peek()
            && &next.inner == tk
        {
            let next = next.clone();
            self.skip();
            Some(next)
        } else {
            None
        }
    }
    pub fn eat_or_err(
        &mut self,
        tk: &TokenKind,
    ) -> Option<Token> {
        let res = self.eat(tk);
        if res.is_none() {
            self.error_expected(tk);
        }
        res
    }
    pub fn eat_one_of(
        &mut self,
        items: impl IntoIterator<Item = TokenKind>,
    ) -> Option<usize> {
        let items = items.into_iter().collect::<Vec<_>>();
        for (id, item) in items.iter().enumerate() {
            if self.iter.peek().is_some_and(|t| &t.inner == item) {
                self.skip();
                return Some(id);
            }
        }
        None
    }
    pub fn eat_path(&mut self) -> Option<[Spanned<String>; 2]> {
        let first = self.eat_ident()?;
        self.eat(&DoubleColon)?;
        let second = self.eat_ident()?;
        Some([first, second])
    }
    pub fn eat_path_or_err(&mut self) -> Option<[Spanned<String>; 2]> {
        let res = self.eat_path();
        if res.is_none() {
            let span = self.last_span;
            self.error().primary("Expected a path here", span).done();
        }
        res
    }
    pub fn eat_ident(&mut self) -> Option<Spanned<String>> {
        if let Some(next) = self.iter.peek().cloned()
            && let Identifier(name) = next.inner
        {
            let span = next.span;
            self.skip();
            Some(name.with_span(span))
        } else {
            None
        }
    }
    pub fn eat_ident_or_err(&mut self) -> Option<Spanned<String>> {
        let res = self.eat_ident();
        if res.is_none() {
            self.error_expected(&TokenKind::Identifier("identifier".to_string()));
        }
        res
    }
    pub fn error(&mut self) -> crate::LogBuilder<'_> {
        self.logger.error(ERR_MSG)
    }
    pub fn error_expected(
        &mut self,
        token: &TokenKind,
    ) -> crate::LogBuilder<'_> {
        self.logger
            .error(ERR_MSG)
            .primary(format!("Expected `{token}` here"), self.last_span)
    }
    pub fn recover(
        &mut self,
        bh: RecoveryBehavior,
    ) {
        match bh {
            RecoveryBehavior::NoRecovery => (),
            RecoveryBehavior::UntilKind(token_kind) => {
                while self.peek().is_some_and(|t| t.inner != token_kind) {
                    self.skip()
                }
            }
            RecoveryBehavior::UntilCategory(category) => {
                while self.peek().is_some_and(|t| t.inner.category() != category) {
                    self.skip()
                }
            }
            RecoveryBehavior::UntilNextStatement => {
                while self
                    .peek()
                    .is_some_and(|t| matches!(&t.inner, Let | Type | Do | Module | End | Import))
                {
                    self.skip();
                }
            }
        }
    }
}

pub fn parse(
    logger: &mut Logger,
    iter: impl IntoIterator<Item = Token>,
) -> Vec<ParsedModule> {
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
        if let Some(module) = p.parse_module() {
            modules.push(module);
        }
    }
    modules
}
