use rowan::GreenNodeBuilder;

use super::{
    SyntaxKind,
    SyntaxNode,
};
use crate::logging::{
    FileLogger,
    Span,
    WithContext,
};
use crate::token::Token;

/// A token together with its `SyntaxKind`, source text slice, and
/// original `Span` for error reporting.
pub(super) struct LexToken<'src> {
    pub kind: SyntaxKind,
    pub text: &'src str,
    pub span: Span,
}

/// Marker for a node whose `start_node` has been called but whose
/// `finish_node` has not yet been called. Prevents forgetting to close
/// a node by making the finish step explicit.
pub(super) struct Marker {
    _pos: usize,
}

pub(super) struct Parser<'src, 'log> {
    tokens: Vec<LexToken<'src>>,
    pos: usize,
    builder: GreenNodeBuilder<'static>,
    logger: &'log mut FileLogger,
}

impl<'src, 'log> Parser<'src, 'log> {
    pub fn new(
        tokens: &[Token],
        source: &'src str,
        logger: &'log mut FileLogger,
    ) -> Self {
        let lex_tokens: Vec<LexToken<'src>> = tokens
            .iter()
            .map(|t| {
                let Span::Source { start, width, .. } = t.span else {
                    unreachable!()
                };
                LexToken {
                    kind: SyntaxKind::from(&t.inner),
                    text: source.get(start..(start + width)).unwrap_or(""),
                    span: t.span,
                }
            })
            .collect();
        Self {
            tokens: lex_tokens,
            pos: 0,
            builder: GreenNodeBuilder::new(),
            logger,
        }
    }

    /// Finish building and return the root `SyntaxNode`.
    pub fn finish(self) -> SyntaxNode {
        let green = self.builder.finish();
        SyntaxNode::new_root(green)
    }

    // ── Span helpers ─────────────────────────────────────────────────

    /// The `Span` of the current non-trivia token, or a zero-width span
    /// at EOF.
    fn current_span(&self) -> Span {
        self.nth_span(0)
    }

    /// The `Span` of the nth non-trivia token ahead (0-indexed), or a
    /// zero-width span at EOF.
    fn nth_span(
        &self,
        n: usize,
    ) -> Span {
        let mut i = self.pos;
        let mut remaining = n;
        while i < self.tokens.len() {
            let kind = self.tokens[i].kind;
            if !kind.is_trivia() {
                if remaining == 0 {
                    return self.tokens[i].span;
                }
                remaining -= 1;
            }
            i += 1;
        }
        // EOF: zero-width span at end of last token
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

    // ── Lookahead ────────────────────────────────────────────────────

    /// The `SyntaxKind` of the current non-trivia token, or `None` at EOF.
    pub fn current(&self) -> Option<SyntaxKind> {
        self.nth(0)
    }

    /// Lookahead by `n` non-trivia tokens (0-indexed).
    pub fn nth(
        &self,
        n: usize,
    ) -> Option<SyntaxKind> {
        let mut i = self.pos;
        let mut remaining = n;
        while i < self.tokens.len() {
            let kind = self.tokens[i].kind;
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

    /// Check whether the current non-trivia token is `kind`.
    pub fn at(
        &self,
        kind: SyntaxKind,
    ) -> bool {
        self.current() == Some(kind)
    }

    /// Check whether the current non-trivia token is in `set`.
    pub fn at_any(
        &self,
        set: &[SyntaxKind],
    ) -> bool {
        self.current().is_some_and(|k| set.contains(&k))
    }

    /// True when there are no more non-trivia tokens.
    pub fn at_end(&self) -> bool {
        self.current().is_none()
    }

    // ── Consuming tokens ─────────────────────────────────────────────

    /// Eat leading trivia, attaching them to the current open node.
    pub fn skip_trivia(&mut self) {
        while self.pos < self.tokens.len() && self.tokens[self.pos].kind.is_trivia() {
            self.do_bump();
        }
    }

    /// Advance past exactly one token (including trivia) and add it to
    /// the green tree.
    fn do_bump(&mut self) {
        let tok = &self.tokens[self.pos];
        self.builder
            .token(rowan::SyntaxKind(tok.kind as u16), tok.text);
        self.pos += 1;
    }

    /// Consume leading trivia then the next non-trivia token.
    pub fn bump(&mut self) {
        self.skip_trivia();
        if self.pos < self.tokens.len() {
            self.do_bump();
        }
    }

    /// Consume the current token if it matches `kind`. Returns `true`
    /// on success.
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

    /// Consume the current token if it matches `kind`, otherwise record
    /// an error via the file logger.
    pub fn expect(
        &mut self,
        kind: SyntaxKind,
    ) {
        if !self.eat(kind) {
            let span = self.current_span();
            self.logger
                .error("Syntax error")
                .primary(format!("Expected `{kind:?}` here"), span)
                .done();
        }
    }

    // ── Node construction ────────────────────────────────────────────

    /// Open a new node of the given kind. Must be paired with
    /// `finish_node`.
    pub fn start_node(
        &mut self,
        kind: SyntaxKind,
    ) -> Marker {
        self.skip_trivia();
        self.builder.start_node(rowan::SyntaxKind(kind as u16));
        Marker { _pos: self.pos }
    }

    /// Open a new node, attaching any immediately-preceding comment
    /// tokens as leading children of the node.
    ///
    /// "Immediately preceding" means there is no blank line between the
    /// comments and the node's first real token.  Trivia *before* the
    /// last blank line is emitted outside the node; only comments after
    /// that point are pulled in.
    pub fn start_node_with_leading_comments(
        &mut self,
        kind: SyntaxKind,
    ) -> Marker {
        // Scan the upcoming trivia run to find where to split.
        // We want to emit trivia up to (and including) the last blank
        // line *outside* the node, then start the node, then emit the
        // remaining trivia (comments + non-blank-line whitespace) *inside*.
        let split = self.leading_comment_split();

        // Emit trivia that precedes the last blank line (outside node).
        while self.pos < split {
            self.do_bump();
        }

        // Open the node.
        self.builder.start_node(rowan::SyntaxKind(kind as u16));
        let marker = Marker { _pos: self.pos };

        // Emit remaining trivia (comments + whitespace) inside the node.
        while self.pos < self.tokens.len() && self.tokens[self.pos].kind.is_trivia() {
            self.do_bump();
        }

        marker
    }

    /// Find the token index where we should split the leading trivia
    /// run: trivia before `split` goes outside the node, trivia from
    /// `split` onward goes inside.
    ///
    /// The split point is right after the last blank-line boundary in
    /// the trivia run.  If there is no blank line, the split is at
    /// `self.pos` (all trivia goes inside).
    ///
    /// A "blank line" is two consecutive newlines in the source.
    /// Because line-comment tokens include their trailing `\n`, a blank
    /// line can span a token boundary: the `\n` at the end of a
    /// LINE_COMMENT followed by a WHITESPACE that contains another `\n`.
    /// We track this with `prev_ended_with_newline` and only scan
    /// whitespace tokens for blank-line boundaries.
    fn leading_comment_split(&self) -> usize {
        let mut split = self.pos;
        let mut prev_ended_with_newline = false;

        for i in self.pos..self.tokens.len() {
            let tok = &self.tokens[i];
            if !tok.kind.is_trivia() {
                break;
            }
            let text = tok.text;
            match tok.kind {
                SyntaxKind::WHITESPACE => {
                    // A blank line occurs when:
                    // (a) this whitespace token alone contains two newlines, OR
                    // (b) previous token ended with \n and this whitespace has any \n.
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

    /// Open a node *before* any trivia-skipping — used when the node
    /// should contain leading trivia (e.g. `SOURCE_FILE`).
    pub fn start_node_before_trivia(
        &mut self,
        kind: SyntaxKind,
    ) -> Marker {
        self.builder.start_node(rowan::SyntaxKind(kind as u16));
        Marker { _pos: self.pos }
    }

    /// Close the most recently opened node.
    pub fn finish_node(
        &mut self,
        _marker: Marker,
    ) {
        self.builder.finish_node();
    }

    /// Create a checkpoint for retroactive node wrapping.
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
        Marker { _pos: 0 }
    }

    // ── Errors ───────────────────────────────────────────────────────

    /// Report an error at the current token position.
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

    /// Wrap the current token in an `ERROR` node, report an error, and
    /// skip past it.
    pub fn error_and_bump(
        &mut self,
        message: impl Into<String>,
    ) {
        let m = self.start_node(SyntaxKind::ERROR);
        self.error_at_current(message);
        self.bump();
        self.finish_node(m);
    }

    /// Skip tokens until we see one in `recovery` (or hit EOF), wrapping
    /// them all in an `ERROR` node.
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

/// True if a whitespace string contains a blank line — two newlines
/// with only whitespace (spaces/tabs) between them.
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
