use rowan::GreenNodeBuilder;

use super::lexer::LexToken;
use super::{
    SyntaxKind,
    SyntaxNode,
};
use crate::logging::{
    FileLogger,
    Span,
    WithContext,
};

#[allow(dead_code)]
pub(super) struct Marker(usize);

pub(super) struct Parser<'src, 'log> {
    tokens: Vec<LexToken>,
    pos: usize,
    builder: GreenNodeBuilder<'static>,
    logger: &'log mut FileLogger,
    source: &'src str,
}

impl<'src, 'log> Parser<'src, 'log> {
    pub fn new(
        tokens: &[LexToken],
        source: &'src str,
        logger: &'log mut FileLogger,
    ) -> Self {
        Self {
            tokens: tokens.to_vec(),
            pos: 0,
            builder: GreenNodeBuilder::new(),
            logger,
            source,
        }
    }

    pub fn finish(self) -> SyntaxNode {
        let green = self.builder.finish();
        SyntaxNode::new_root(green)
    }

    fn current_span(&self) -> Span {
        self.nth_span(0)
    }

    fn nth_span(
        &self,
        n: usize,
    ) -> Span {
        let mut i = self.pos;
        let mut remaining = n;
        while i < self.tokens.len() {
            let kind = self.tokens[i].inner;
            if !kind.is_trivia() {
                if remaining == 0 {
                    return self.tokens[i].span;
                }
                remaining -= 1;
            }
            i += 1;
        }
        if let Some(last) = self.tokens.last() {
            match last.span {
                Span::Source { start, width } => {
                    Span::Source {
                        start: start + width,
                        width: 0,
                    }
                }
                Span::Generated => Span::Generated,
            }
        } else {
            Span::Generated
        }
    }

    pub fn current(&self) -> Option<SyntaxKind> {
        self.nth(0)
    }

    pub fn nth(
        &self,
        n: usize,
    ) -> Option<SyntaxKind> {
        let mut i = self.pos;
        let mut remaining = n;
        while i < self.tokens.len() {
            let kind = self.tokens[i].inner;
            if !kind.is_trivia() {
                if remaining == 0 {
                    return Some(kind);
                }
                remaining -= 1;
            }
            i += 1;
        }
        None
    }

    pub fn at(
        &self,
        kind: SyntaxKind,
    ) -> bool {
        self.current() == Some(kind)
    }

    pub fn at_any(
        &self,
        set: &[SyntaxKind],
    ) -> bool {
        self.current().is_some_and(|k| set.contains(&k))
    }

    pub fn at_end(&self) -> bool {
        self.current().is_none()
    }

    pub fn skip_trivia(&mut self) {
        while self.pos < self.tokens.len() && self.tokens[self.pos].inner.is_trivia() {
            self.do_bump();
        }
    }

    fn do_bump(&mut self) {
        let tok = &self.tokens[self.pos];
        let text = self.get_token_text(tok);
        self.builder
            .token(rowan::SyntaxKind(tok.inner as u16), text);
        self.pos += 1;
    }

    fn get_token_text(
        &self,
        tok: &LexToken,
    ) -> &'src str {
        match tok.span {
            Span::Source { start, width } => self.source.get(start..(start + width)).unwrap_or(""),
            Span::Generated => "",
        }
    }

    pub fn bump(&mut self) {
        self.skip_trivia();
        if self.pos < self.tokens.len() {
            self.do_bump();
        }
    }

    pub fn eat(
        &mut self,
        kind: SyntaxKind,
    ) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub fn expect(
        &mut self,
        kind: SyntaxKind,
    ) {
        if !self.eat(kind) {
            let span = self.current_span();
            self.logger
                .error("Syntax error")
                .primary(format!("Expected `{kind}` here"), span)
                .done();
        }
    }

    pub fn start_node(
        &mut self,
        kind: SyntaxKind,
    ) -> Marker {
        self.skip_trivia();
        self.builder.start_node(rowan::SyntaxKind(kind as u16));
        Marker(self.pos)
    }

    pub fn start_node_with_leading_comments(
        &mut self,
        kind: SyntaxKind,
    ) -> Marker {
        let split = self.leading_comment_split();

        while self.pos < split {
            self.do_bump();
        }

        self.builder.start_node(rowan::SyntaxKind(kind as u16));
        let marker = Marker(self.pos);

        while self.pos < self.tokens.len() && self.tokens[self.pos].inner.is_trivia() {
            self.do_bump();
        }

        marker
    }

    fn leading_comment_split(&self) -> usize {
        let mut split = self.pos;
        let mut prev_ended_with_newline = false;

        for i in self.pos..self.tokens.len() {
            let tok = &self.tokens[i];
            if !tok.inner.is_trivia() {
                break;
            }
            let text = self.get_token_text(tok);
            match tok.inner {
                SyntaxKind::WHITESPACE => {
                    if contains_blank_line(text) || (prev_ended_with_newline && text.contains('\n'))
                    {
                        split = i + 1;
                    }
                }
                SyntaxKind::LINE_COMMENT | SyntaxKind::BLOCK_COMMENT => {}
                _ => {}
            }

            prev_ended_with_newline = text.ends_with('\n');
        }
        split
    }

    pub fn start_node_before_trivia(
        &mut self,
        kind: SyntaxKind,
    ) -> Marker {
        self.builder.start_node(rowan::SyntaxKind(kind as u16));
        Marker(self.pos)
    }

    pub fn finish_node(
        &mut self,
        _marker: Marker,
    ) {
        self.builder.finish_node();
    }

    pub fn checkpoint(&mut self) -> rowan::Checkpoint {
        self.skip_trivia();
        self.builder.checkpoint()
    }

    pub fn start_node_at(
        &mut self,
        checkpoint: rowan::Checkpoint,
        kind: SyntaxKind,
    ) -> Marker {
        self.builder
            .start_node_at(checkpoint, rowan::SyntaxKind(kind as u16));
        Marker(0)
    }

    pub fn error_at_current(
        &mut self,
        message: impl Into<String>,
    ) {
        let span = self.current_span();
        self.logger
            .error("Syntax error")
            .primary(message, span)
            .done();
    }

    pub fn error_and_bump(
        &mut self,
        message: impl Into<String>,
    ) {
        let m = self.start_node(SyntaxKind::ERROR);
        self.error_at_current(message);
        self.bump();
        self.finish_node(m);
    }

    pub fn error_recover(
        &mut self,
        message: impl Into<String>,
        recovery: &[SyntaxKind],
    ) {
        if self.at_any(recovery) || self.at_end() {
            self.error_at_current(message);
            return;
        }
        let m = self.start_node(SyntaxKind::ERROR);
        self.error_at_current(message);
        while !self.at_any(recovery) && !self.at_end() {
            self.bump();
        }
        self.finish_node(m);
    }
}

fn contains_blank_line(text: &str) -> bool {
    let mut saw_newline = false;
    for ch in text.chars() {
        if ch == '\n' {
            if saw_newline {
                return true;
            }
            saw_newline = true;
        } else if !ch.is_whitespace() {
            saw_newline = false;
        }
    }
    false
}
