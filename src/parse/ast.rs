//! Typed AST wrappers over the untyped lossless CST.
//!
//! Each struct is a thin newtype around [`SyntaxNode`] that checks the
//! node's [`SyntaxKind`] on construction.  All fields are optional to
//! accommodate incomplete / erroneous source code.
//!
//! Enum types (`Expr`, `Pattern`, `TypeExpr`, `TypeDef`, `Statement`)
//! group related concrete nodes for convenient pattern matching.

use crate::{
    Span,
    Spanned,
    WithSpan,
};

use super::{
    SyntaxKind,
    SyntaxNode,
    SyntaxToken,
};

// ── Core trait ───────────────────────────────────────────────────────

/// Implemented by every typed AST node.
pub trait AstNode: Sized {
    /// Try to interpret an untyped `SyntaxNode` as this type.
    fn cast(node: SyntaxNode) -> Option<Self>;
    fn with_file_id(
        self,
        file_id: usize,
    ) -> Self;
    /// Access the underlying untyped node.
    fn syntax(&self) -> &SyntaxNode;
    fn file_id(&self) -> usize;

    fn span(&self) -> Span {
        let mut tokens = self
            .syntax()
            .descendants_with_tokens()
            .filter_map(|el| el.into_token())
            .filter(|t| !t.kind().is_trivia());
        let Some(first) = tokens.next() else {
            return Span::Generated;
        };
        Span::from(rowan::TextRange::new(
            first.text_range().start(),
            tokens
                .last()
                .unwrap_or_else(|| first.clone())
                .text_range()
                .end(),
        ))
        .with_file_id(self.file_id())
    }
}

// ── Helper traits ────────────────────────────────────────────────────

/// Nodes that carry a name via an `IDENT` child token.
pub trait HasName: AstNode {
    fn name_token(&self) -> Option<SyntaxToken> {
        child_token(self.syntax(), SyntaxKind::IDENT)
    }
    fn name_text(&self) -> Option<String> {
        self.name_text_spanned().map(|n| n.inner)
    }
    fn name_text_spanned(&self) -> Option<Spanned<String>> {
        first_identifier_text_spanned(self.syntax(), self.file_id())
    }
}

/// Nodes whose parser rule uses `start_node_with_leading_comments`,
/// so comment tokens that immediately precede the node (with no blank
/// line in between) are included as leading children.
pub trait HasLeadingComments: AstNode {
    /// All comment tokens attached as leading children of this node.
    fn leading_comments(&self) -> Vec<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .take_while(|t| t.kind().is_trivia())
            .filter(|t| {
                matches!(
                    t.kind(),
                    SyntaxKind::LINE_COMMENT | SyntaxKind::BLOCK_COMMENT
                )
            })
            .collect()
    }

    /// The text of all leading comments, concatenated with newlines.
    /// Trailing whitespace on each comment line is trimmed.
    fn leading_comment_text(&self) -> String {
        self.leading_comments()
            .iter()
            .map(|t| t.text().trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// ── Private helpers ──────────────────────────────────────────────────

/// Find the first child node that can be cast to `N`.
fn child_node<N: AstNode>(
    parent: &SyntaxNode,
    file_id: usize,
) -> Option<N> {
    parent
        .children()
        .find_map(|child| N::cast(child).map(|node| node.with_file_id(file_id)))
}

/// Find all child nodes that can be cast to `N`.
fn child_nodes<N: AstNode>(
    parent: &SyntaxNode,
    file_id: usize,
) -> Vec<N> {
    parent
        .children()
        .filter_map(|child| N::cast(child).map(|node| node.with_file_id(file_id)))
        .collect()
}

/// Find the nth child node that can be cast to `N` (0-indexed).
fn nth_child_node<N: AstNode>(
    parent: &SyntaxNode,
    file_id: usize,
    n: usize,
) -> Option<N> {
    parent
        .children()
        .filter_map(|child| N::cast(child).map(|node| node.with_file_id(file_id)))
        .nth(n)
}

/// Find the first child node that can be cast to `N` after a marker token.
fn child_node_after_token<N: AstNode>(
    parent: &SyntaxNode,
    file_id: usize,
    marker: SyntaxKind,
) -> Option<N> {
    let mut seen_marker = false;
    for el in parent.children_with_tokens() {
        match el {
            rowan::NodeOrToken::Token(tok) => {
                if tok.kind() == marker {
                    seen_marker = true;
                }
            }
            rowan::NodeOrToken::Node(node) => {
                if seen_marker
                    && let Some(cast) = N::cast(node).map(|node| node.with_file_id(file_id))
                {
                    return Some(cast);
                }
            }
        }
    }
    None
}

/// Find the first child token of a given `SyntaxKind`.
fn child_token(
    parent: &SyntaxNode,
    kind: SyntaxKind,
) -> Option<SyntaxToken> {
    parent
        .children_with_tokens()
        .filter_map(|el| el.into_token())
        .find(|t| t.kind() == kind)
}

/// Find the first non-trivia child token whose kind is *not* a node and
/// that satisfies the predicate.
fn first_token_where(
    parent: &SyntaxNode,
    pred: impl Fn(SyntaxKind) -> bool,
) -> Option<SyntaxToken> {
    parent
        .children_with_tokens()
        .filter_map(|el| el.into_token())
        .filter(|t| !t.kind().is_trivia())
        .find(|t| pred(t.kind()))
}

fn non_trivia_tokens(parent: &SyntaxNode) -> Vec<SyntaxToken> {
    parent
        .children_with_tokens()
        .filter_map(|el| el.into_token())
        .filter(|t| !t.kind().is_trivia())
        .collect()
}

fn span_from_text_range(
    file_id: usize,
    range: rowan::TextRange,
) -> Span {
    Span::from(range).with_file_id(file_id)
}

fn first_identifier_text_spanned(
    parent: &SyntaxNode,
    file_id: usize,
) -> Option<Spanned<String>> {
    let tokens = non_trivia_tokens(parent);
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind() == SyntaxKind::IDENT {
            return Some(
                token
                    .text()
                    .to_string()
                    .with_span(span_from_text_range(file_id, token.text_range())),
            );
        }
        if token.kind() == SyntaxKind::L_SQUARE
            && let (Some(op), Some(end)) = (tokens.get(index + 1), tokens.get(index + 2))
            && op.kind().is_operator_token()
            && end.kind() == SyntaxKind::R_SQUARE
        {
            return Some(format!("[{}]", op.text()).with_span(span_from_text_range(
                file_id,
                rowan::TextRange::new(token.text_range().start(), end.text_range().end()),
            )));
        }
        index += 1;
    }
    None
}

fn all_identifier_texts(parent: &SyntaxNode) -> Vec<String> {
    let tokens = non_trivia_tokens(parent);
    let mut names = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind() == SyntaxKind::IDENT {
            names.push(token.text().to_string());
            index += 1;
            continue;
        }
        if token.kind() == SyntaxKind::L_SQUARE
            && let (Some(op), Some(end)) = (tokens.get(index + 1), tokens.get(index + 2))
            && op.kind().is_operator_token()
            && end.kind() == SyntaxKind::R_SQUARE
        {
            names.push(format!("[{}]", op.text()));
            index += 3;
            continue;
        }
        index += 1;
    }
    names
}

fn alias_name_after_as(
    parent: &SyntaxNode,
    file_id: usize,
) -> Option<Spanned<String>> {
    let tokens = non_trivia_tokens(parent);
    let as_index = tokens
        .iter()
        .position(|token| token.kind() == SyntaxKind::AS_KW)?;
    let index = as_index + 1;
    let token = tokens.get(index)?;
    if token.kind() == SyntaxKind::IDENT {
        return Some(
            token
                .text()
                .to_string()
                .with_span(span_from_text_range(file_id, token.text_range())),
        );
    }
    if token.kind() == SyntaxKind::L_SQUARE
        && let (Some(op), Some(end)) = (tokens.get(index + 1), tokens.get(index + 2))
        && op.kind().is_operator_token()
        && end.kind() == SyntaxKind::R_SQUARE
    {
        return Some(format!("[{}]", op.text()).with_span(span_from_text_range(
            file_id,
            rowan::TextRange::new(token.text_range().start(), end.text_range().end()),
        )));
    }
    None
}

fn alias_name_after_pipe(
    parent: &SyntaxNode,
    file_id: usize,
) -> Option<Spanned<String>> {
    let tokens = non_trivia_tokens(parent);
    let pipe_index = tokens
        .iter()
        .position(|token| token.kind() == SyntaxKind::PIPE)?;
    let index = pipe_index + 1;
    let token = tokens.get(index)?;
    if token.kind() == SyntaxKind::IDENT {
        return Some(
            token
                .text()
                .to_string()
                .with_span(span_from_text_range(file_id, token.text_range())),
        );
    }
    if token.kind() == SyntaxKind::L_SQUARE
        && let (Some(op), Some(end)) = (tokens.get(index + 1), tokens.get(index + 2))
        && op.kind().is_operator_token()
        && end.kind() == SyntaxKind::R_SQUARE
    {
        return Some(format!("[{}]", op.text()).with_span(span_from_text_range(
            file_id,
            rowan::TextRange::new(token.text_range().start(), end.text_range().end()),
        )));
    }
    None
}

// ── Macro to reduce per-node boilerplate ─────────────────────────────

macro_rules! ast_node {
    ($name:ident, $kind:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            syntax: SyntaxNode,
            file_id: usize,
        }

        impl AstNode for $name {
            fn cast(node: SyntaxNode) -> Option<Self> {
                if node.kind() == SyntaxKind::$kind {
                    Some(Self {
                        syntax: node,
                        file_id: 0,
                    })
                } else {
                    None
                }
            }
            fn with_file_id(
                mut self,
                file_id: usize,
            ) -> Self {
                self.file_id = file_id;
                self
            }
            fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }
            fn file_id(&self) -> usize {
                self.file_id
            }
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════
// Program structure
// ═══════════════════════════════════════════════════════════════════════

ast_node!(SourceFile, SOURCE_FILE);

impl SourceFile {
    pub fn items(&self) -> Vec<Statement> {
        child_nodes(&self.syntax, self.file_id())
    }

    pub fn bundle_declaration(&self) -> Option<BundleDeclaration> {
        self.items().into_iter().find_map(|statement| {
            if let Statement::Bundle(bundle_declaration) = statement {
                Some(bundle_declaration)
            } else {
                None
            }
        })
    }

    pub fn modules(&self) -> Vec<Module> {
        child_nodes(&self.syntax, self.file_id())
    }

    pub fn statements(&self) -> Vec<Statement> {
        self.items()
    }

    pub fn imports(&self) -> Vec<ImportStatement> {
        self.items()
            .into_iter()
            .filter_map(|statement| {
                if let Statement::Import(import_statement) = statement {
                    Some(import_statement)
                } else {
                    None
                }
            })
            .collect()
    }
}

ast_node!(BundleDeclaration, BUNDLE_DECLARATION);
impl HasName for BundleDeclaration {
}

ast_node!(Module, MODULE);
impl HasName for Module {
}

impl Module {
    pub fn statements(&self) -> Vec<Statement> {
        self.syntax
            .children()
            .filter_map(|node| Statement::cast(node).map(|node| node.with_file_id(self.file_id())))
            .collect()
    }
}

ast_node!(ImportStatement, IMPORT_STATEMENT);
impl HasLeadingComments for ImportStatement {
}

impl ImportStatement {
    pub fn path_literals(&self) -> Vec<Spanned<String>> {
        non_trivia_tokens(&self.syntax)
            .into_iter()
            .filter(|token| token.kind() == SyntaxKind::STRING)
            .map(|token| {
                token
                    .text()
                    .to_string()
                    .with_span(span_from_text_range(self.file_id(), token.text_range()))
            })
            .collect()
    }
}

ast_node!(UseStatement, USE_STATEMENT);
impl HasLeadingComments for UseStatement {
}

impl UseStatement {
    pub fn target(&self) -> Option<PathOrIdent> {
        self.syntax.children().find_map(PathOrIdent::cast)
    }

    pub fn alias_name_spanned(&self) -> Option<Spanned<String>> {
        alias_name_after_as(&self.syntax, self.file_id())
    }
}

ast_node!(LetStatement, LET_STATEMENT);
impl HasLeadingComments for LetStatement {
}

impl LetStatement {
    pub fn is_pattern_alias(&self) -> bool {
        let tokens = non_trivia_tokens(&self.syntax);
        matches!(
            (
                tokens.first().map(|token| token.kind()),
                tokens.get(1).map(|token| token.kind())
            ),
            (Some(SyntaxKind::LET_KW), Some(SyntaxKind::PIPE))
        )
    }

    pub fn alias_name_spanned(&self) -> Option<Spanned<String>> {
        if !self.is_pattern_alias() {
            return None;
        }
        alias_name_after_pipe(&self.syntax, self.file_id())
    }

    pub fn alias_target(&self) -> Option<PathOrIdent> {
        if !self.is_pattern_alias() {
            return None;
        }
        let equal = child_token(&self.syntax, SyntaxKind::EQUAL)?;
        self.syntax
            .children()
            .filter_map(PathOrIdent::cast)
            .find(|target| target.syntax().text_range().start() >= equal.text_range().end())
    }

    pub fn pattern(&self) -> Option<Pattern> {
        child_node(&self.syntax, self.file_id())
    }
    pub fn value(&self) -> Option<Expr> {
        child_node_after_token(&self.syntax, self.file_id(), SyntaxKind::EQUAL)
    }
}

ast_node!(DoStatement, DO_STATEMENT);
impl HasLeadingComments for DoStatement {
}

impl DoStatement {
    pub fn value(&self) -> Option<Expr> {
        child_node_after_token(&self.syntax, self.file_id(), SyntaxKind::DO_KW)
    }
}

ast_node!(TypeStatement, TYPE_STATEMENT);
impl HasName for TypeStatement {
}
impl HasLeadingComments for TypeStatement {
}

impl TypeStatement {
    pub fn is_alias(&self) -> bool {
        non_trivia_tokens(&self.syntax)
            .into_iter()
            .take_while(|token| token.kind() != SyntaxKind::EQUAL)
            .any(|token| token.kind() == SyntaxKind::TILDE)
    }

    /// Type parameters declared after `:`, e.g. `type Option: a = ...`.
    /// Returns canonical parameter names between `:` and `=`.
    pub fn type_params(&self) -> Vec<Spanned<String>> {
        let tokens = non_trivia_tokens(&self.syntax);
        let mut params = Vec::new();
        let Some(mut index) = tokens
            .iter()
            .position(|token| token.kind() == SyntaxKind::COLON)
        else {
            return params;
        };
        index += 1;

        while index < tokens.len() {
            let token = &tokens[index];
            if token.kind() == SyntaxKind::EQUAL {
                break;
            }

            if token.kind() == SyntaxKind::IDENT {
                params.push(
                    token
                        .text()
                        .to_string()
                        .with_span(span_from_text_range(self.file_id(), token.text_range())),
                );
                index += 1;
                continue;
            }

            if token.kind() == SyntaxKind::L_SQUARE
                && let (Some(op), Some(end)) = (tokens.get(index + 1), tokens.get(index + 2))
                && op.kind().is_operator_token()
                && end.kind() == SyntaxKind::R_SQUARE
            {
                params.push(format!("[{}]", op.text()).with_span(span_from_text_range(
                    self.file_id(),
                    rowan::TextRange::new(token.text_range().start(), end.text_range().end()),
                )));
                index += 3;
                continue;
            }

            index += 1;
        }

        params
    }

    pub fn type_def(&self) -> Option<TypeDef> {
        self.syntax
            .children()
            .find_map(|node| TypeDef::cast(node).map(|node| node.with_file_id(self.file_id())))
    }
}

ast_node!(TraitStatement, TRAIT_STATEMENT);
impl HasName for TraitStatement {
}
impl HasLeadingComments for TraitStatement {
}

impl TraitStatement {
    pub fn is_alias(&self) -> bool {
        non_trivia_tokens(&self.syntax)
            .into_iter()
            .take_while(|token| token.kind() != SyntaxKind::EQUAL)
            .any(|token| token.kind() == SyntaxKind::TILDE)
    }

    pub fn alias_target(&self) -> Option<PathOrIdent> {
        let equal = child_token(&self.syntax, SyntaxKind::EQUAL)?;
        self.syntax
            .children()
            .filter_map(PathOrIdent::cast)
            .find(|target| target.syntax().text_range().start() >= equal.text_range().end())
    }

    pub fn trait_params(&self) -> Vec<Spanned<String>> {
        let tokens = non_trivia_tokens(&self.syntax);
        let mut params = Vec::new();
        let Some(mut index) = tokens
            .iter()
            .position(|token| token.kind() == SyntaxKind::COLON)
        else {
            return params;
        };
        index += 1;

        while index < tokens.len() {
            let token = &tokens[index];
            if token.kind() == SyntaxKind::EQUAL {
                break;
            }

            if token.kind() == SyntaxKind::IDENT {
                params.push(
                    token
                        .text()
                        .to_string()
                        .with_span(span_from_text_range(self.file_id(), token.text_range())),
                );
                index += 1;
                continue;
            }

            if token.kind() == SyntaxKind::L_SQUARE
                && let (Some(op), Some(end)) = (tokens.get(index + 1), tokens.get(index + 2))
                && op.kind().is_operator_token()
                && end.kind() == SyntaxKind::R_SQUARE
            {
                params.push(format!("[{}]", op.text()).with_span(span_from_text_range(
                    self.file_id(),
                    rowan::TextRange::new(token.text_range().start(), end.text_range().end()),
                )));
                index += 3;
                continue;
            }

            index += 1;
        }

        params
    }

    pub fn methods(&self) -> Vec<TraitMethodDecl> {
        child_nodes(&self.syntax, self.file_id())
    }
}

ast_node!(ImplStatement, IMPL_STATEMENT);
impl HasLeadingComments for ImplStatement {
}

impl ImplStatement {
    pub fn trait_name(&self) -> Option<PathOrIdent> {
        self.syntax.children().find_map(PathOrIdent::cast)
    }

    pub fn type_args(&self) -> Vec<TypeExpr> {
        let Some(trait_name) = self.trait_name() else {
            return Vec::new();
        };
        let Some(equal) = child_token(&self.syntax, SyntaxKind::EQUAL) else {
            return Vec::new();
        };
        let start = trait_name.syntax().text_range().end();
        let end = equal.text_range().start();
        self.syntax
            .children()
            .filter_map(|node| TypeExpr::cast(node).map(|node| node.with_file_id(self.file_id())))
            .filter(|type_expr| {
                let range = type_expr.syntax().text_range();
                range.start() >= start && range.end() <= end
            })
            .collect()
    }

    pub fn methods(&self) -> Vec<ImplMethodDef> {
        child_nodes(&self.syntax, self.file_id())
    }
}

ast_node!(WasmStatement, WASM_STATEMENT);
impl HasLeadingComments for WasmStatement {
}

impl WasmStatement {
    pub fn sexpr(&self) -> Option<Sexpr> {
        self.syntax
            .children()
            .find_map(|node| Sexpr::cast(node).map(|node| node.with_file_id(self.file_id())))
    }
}

ast_node!(TraitMethodDecl, TRAIT_METHOD_DECL);
impl HasName for TraitMethodDecl {
}

impl TraitMethodDecl {
    pub fn ty(&self) -> Option<TypeExpr> {
        self.syntax
            .children()
            .find_map(|node| TypeExpr::cast(node).map(|node| node.with_file_id(self.file_id())))
    }
}

ast_node!(ImplMethodDef, IMPL_METHOD_DEF);
impl HasName for ImplMethodDef {
}

impl ImplMethodDef {
    pub fn value(&self) -> Option<Expr> {
        child_node_after_token(&self.syntax, self.file_id(), SyntaxKind::EQUAL)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Type definitions
// ═══════════════════════════════════════════════════════════════════════

ast_node!(StructDef, STRUCT_DEF);

impl StructDef {
    pub fn members(&self) -> Vec<StructTypeMemberDecl> {
        child_nodes(&self.syntax, self.file_id())
    }

    pub fn fields(&self) -> Vec<FieldDecl> {
        self.syntax
            .children()
            .filter_map(|node| FieldDecl::cast(node).map(|node| node.with_file_id(self.file_id())))
            .collect()
    }

    pub fn spreads(&self) -> Vec<StructSpreadDecl> {
        self.syntax
            .children()
            .filter_map(|node| {
                StructSpreadDecl::cast(node).map(|node| node.with_file_id(self.file_id()))
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StructTypeMemberDecl {
    Field(FieldDecl),
    Spread(StructSpreadDecl),
}

impl AstNode for StructTypeMemberDecl {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::FIELD_DECL => FieldDecl::cast(node).map(Self::Field),
            SyntaxKind::STRUCT_SPREAD_DECL => StructSpreadDecl::cast(node).map(Self::Spread),
            _ => None,
        }
    }

    fn with_file_id(
        self,
        file_id: usize,
    ) -> Self {
        match self {
            Self::Field(field) => Self::Field(field.with_file_id(file_id)),
            Self::Spread(spread) => Self::Spread(spread.with_file_id(file_id)),
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Field(field) => field.syntax(),
            Self::Spread(spread) => spread.syntax(),
        }
    }

    fn file_id(&self) -> usize {
        match self {
            Self::Field(field) => field.file_id(),
            Self::Spread(spread) => spread.file_id(),
        }
    }
}

ast_node!(SumDef, SUM_DEF);

impl SumDef {
    pub fn variants(&self) -> Vec<Variant> {
        child_nodes(&self.syntax, self.file_id())
    }
}

ast_node!(Variant, VARIANT);
impl HasName for Variant {
}

impl Variant {
    /// The optional payload type expression after the variant name.
    pub fn payload_type(&self) -> Option<TypeExpr> {
        self.syntax
            .children()
            .find_map(|node| TypeExpr::cast(node).map(|node| node.with_file_id(self.file_id())))
    }
}

ast_node!(FieldDecl, FIELD_DECL);
impl HasName for FieldDecl {
}

impl FieldDecl {
    pub fn ty(&self) -> Option<TypeExpr> {
        self.syntax
            .children()
            .find_map(|node| TypeExpr::cast(node).map(|node| node.with_file_id(self.file_id())))
    }
}

ast_node!(StructSpreadDecl, STRUCT_SPREAD_DECL);

impl StructSpreadDecl {
    pub fn ty(&self) -> Option<TypeExpr> {
        self.syntax
            .children()
            .find_map(|node| TypeExpr::cast(node).map(|node| node.with_file_id(self.file_id())))
    }
}

ast_node!(TypeAliasDef, TYPE_ALIAS_DEF);

impl TypeAliasDef {
    pub fn type_expr(&self) -> Option<TypeExpr> {
        self.syntax
            .children()
            .find_map(|node| TypeExpr::cast(node).map(|node| node.with_file_id(self.file_id())))
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Type expressions
// ═══════════════════════════════════════════════════════════════════════

ast_node!(FunctionType, FUNCTION_TYPE);

impl FunctionType {
    pub fn param_type(&self) -> Option<TypeExpr> {
        nth_type_expr(&self.syntax, self.file_id(), 0)
    }
    pub fn return_type(&self) -> Option<TypeExpr> {
        nth_type_expr(&self.syntax, self.file_id(), 1)
    }
}

ast_node!(TypeApplication, TYPE_APPLICATION);

impl TypeApplication {
    pub fn base(&self) -> Option<TypeExpr> {
        nth_type_expr(&self.syntax, self.file_id(), 0)
    }
    pub fn args(&self) -> Vec<TypeExpr> {
        self.syntax
            .children()
            .filter_map(|node| TypeExpr::cast(node).map(|node| node.with_file_id(self.file_id())))
            .skip(1)
            .collect()
    }
}

ast_node!(TupleType, TUPLE_TYPE);

impl TupleType {
    pub fn fields(&self) -> Vec<TypeExpr> {
        self.syntax
            .children()
            .filter_map(|node| TypeExpr::cast(node).map(|node| node.with_file_id(self.file_id())))
            .collect()
    }

    pub fn is_tuple(&self) -> bool {
        self.fields().len() > 1 || node_has_token(&self.syntax, SyntaxKind::COMMA)
    }
}

ast_node!(ArrayType, ARRAY_TYPE);
ast_node!(ForAllType, FORALL_TYPE);
ast_node!(TypeConstraint, TYPE_CONSTRAINT);

impl TypeConstraint {
    pub fn trait_name(&self) -> Option<PathOrIdent> {
        self.syntax.children().find_map(PathOrIdent::cast)
    }

    pub fn args(&self) -> Vec<TypeExpr> {
        let Some(trait_name) = self.trait_name() else {
            return Vec::new();
        };
        let args_start = match trait_name {
            PathOrIdent::Ident(ident) => ident.syntax().text_range().end(),
            PathOrIdent::Path(path) => path.syntax().text_range().end(),
        };
        self.syntax
            .children()
            .filter_map(|node| TypeExpr::cast(node).map(|node| node.with_file_id(self.file_id())))
            .filter(|type_expr| type_expr.syntax().text_range().start() >= args_start)
            .collect()
    }
}

impl ForAllType {
    /// The type variable identifiers declared after `for`.
    pub fn params(&self) -> Vec<Ident> {
        let Some(for_kw) = child_token(&self.syntax, SyntaxKind::FOR_KW) else {
            return Vec::new();
        };
        let Some(in_kw) = child_token(&self.syntax, SyntaxKind::IN_KW) else {
            return Vec::new();
        };
        let for_end = for_kw.text_range().end();
        let param_end = in_kw.text_range().start();
        self.syntax
            .children()
            .filter_map(|node| Ident::cast(node).map(|node| node.with_file_id(self.file_id())))
            .filter(|ident| {
                let range = ident.syntax().text_range();
                range.start() >= for_end && range.end() <= param_end
            })
            .collect()
    }

    pub fn constraints(&self) -> Vec<TypeConstraint> {
        let Some(where_kw) = child_token(&self.syntax, SyntaxKind::WHERE_KW) else {
            return Vec::new();
        };
        let constraints_start = where_kw.text_range().end();
        self.syntax
            .children()
            .filter_map(|node| {
                TypeConstraint::cast(node).map(|node| node.with_file_id(self.file_id()))
            })
            .filter(|constraint| {
                let range = constraint.syntax().text_range();
                range.start() >= constraints_start
            })
            .collect()
    }

    /// The body type expression after `in`.
    pub fn body(&self) -> Option<TypeExpr> {
        let in_kw = child_token(&self.syntax, SyntaxKind::IN_KW)?;
        let in_end = in_kw.text_range().end();
        self.syntax
            .children()
            .filter_map(|node| TypeExpr::cast(node).map(|node| node.with_file_id(self.file_id())))
            .find(|type_expr| type_expr.syntax().text_range().start() >= in_end)
    }
}

ast_node!(Unit, UNIT);

/// Helper: get the nth TypeExpr child of a node.
fn nth_type_expr(
    parent: &SyntaxNode,
    file_id: usize,
    n: usize,
) -> Option<TypeExpr> {
    parent
        .children()
        .filter_map(|node| TypeExpr::cast(node).map(|node| node.with_file_id(file_id)))
        .nth(n)
}

// ═══════════════════════════════════════════════════════════════════════
// Value expressions
// ═══════════════════════════════════════════════════════════════════════

ast_node!(LetExpr, LET_EXPR);

impl LetExpr {
    pub fn pattern(&self) -> Option<Pattern> {
        child_node(&self.syntax, self.file_id())
    }
    /// The value being bound (first Expr child).
    pub fn value(&self) -> Option<Expr> {
        child_node_after_token(&self.syntax, self.file_id(), SyntaxKind::EQUAL)
    }
    /// The body after `in` (second Expr child).
    pub fn body(&self) -> Option<Expr> {
        child_node_after_token(&self.syntax, self.file_id(), SyntaxKind::IN_KW)
    }
}

ast_node!(UseExpr, USE_EXPR);

impl UseExpr {
    pub fn target(&self) -> Option<PathOrIdent> {
        self.syntax.children().find_map(PathOrIdent::cast)
    }

    pub fn alias_name_spanned(&self) -> Option<Spanned<String>> {
        alias_name_after_as(&self.syntax, self.file_id())
    }

    pub fn body(&self) -> Option<Expr> {
        child_node_after_token(&self.syntax, self.file_id(), SyntaxKind::IN_KW)
    }
}

ast_node!(FnExpr, FN_EXPR);

impl FnExpr {
    pub fn params(&self) -> Vec<Param> {
        child_nodes(&self.syntax, self.file_id())
    }
    pub fn body(&self) -> Option<Expr> {
        child_node(&self.syntax, self.file_id())
    }
}

ast_node!(FnShorthandExpr, FN_SHORTHAND_EXPR);

impl FnShorthandExpr {
    pub fn arms(&self) -> Vec<MatchArm> {
        child_nodes(&self.syntax, self.file_id())
    }
}

ast_node!(IfExpr, IF_EXPR);

impl IfExpr {
    pub fn condition(&self) -> Option<Expr> {
        nth_child_node(&self.syntax, self.file_id(), 0)
    }
    pub fn then_branch(&self) -> Option<Expr> {
        nth_child_node(&self.syntax, self.file_id(), 1)
    }
    pub fn else_branch(&self) -> Option<Expr> {
        nth_child_node(&self.syntax, self.file_id(), 2)
    }
}

ast_node!(MatchExpr, MATCH_EXPR);

impl MatchExpr {
    pub fn scrutinee(&self) -> Option<Expr> {
        child_node(&self.syntax, self.file_id())
    }
    pub fn arms(&self) -> Vec<MatchArm> {
        child_nodes(&self.syntax, self.file_id())
    }
}

ast_node!(InlineWasmExpr, INLINE_WASM_EXPR);

impl InlineWasmExpr {
    pub fn asserted_type(&self) -> Option<TypeExpr> {
        child_node_after_token(&self.syntax, self.file_id(), SyntaxKind::COLON)
    }

    pub fn instructions(&self) -> Option<Sexpr> {
        self.syntax.children().find_map(Sexpr::cast)
    }
}

ast_node!(MatchArm, MATCH_ARM);

impl MatchArm {
    pub fn pattern(&self) -> Option<Pattern> {
        child_node(&self.syntax, self.file_id())
    }
    pub fn body(&self) -> Option<Expr> {
        child_node_after_token(&self.syntax, self.file_id(), SyntaxKind::DOUBLE_ARROW)
    }
}

ast_node!(Param, PARAM);
impl HasName for Param {
}

impl Param {
    /// Optional type annotation (present when `(name: type)`).
    pub fn ty(&self) -> Option<TypeExpr> {
        self.syntax
            .children()
            .find_map(|node| TypeExpr::cast(node).map(|node| node.with_file_id(self.file_id())))
    }
}

ast_node!(BinaryExpr, BINARY_EXPR);

impl BinaryExpr {
    pub fn lhs(&self) -> Option<Expr> {
        nth_child_node(&self.syntax, self.file_id(), 0)
    }
    pub fn rhs(&self) -> Option<Expr> {
        nth_child_node(&self.syntax, self.file_id(), 1)
    }
    /// The operator token (e.g. `+`, `*`, `==`).
    pub fn op_token(&self) -> Option<SyntaxToken> {
        first_token_where(&self.syntax, |k| {
            !matches!(
                k,
                SyntaxKind::IDENT
                    | SyntaxKind::INTEGER
                    | SyntaxKind::REAL
                    | SyntaxKind::STRING
                    | SyntaxKind::GLYPH
                    | SyntaxKind::TRUE_KW
                    | SyntaxKind::FALSE_KW
                    | SyntaxKind::L_PAREN
                    | SyntaxKind::R_PAREN
                    | SyntaxKind::L_SQUARE
                    | SyntaxKind::R_SQUARE
                    | SyntaxKind::L_BRACE
                    | SyntaxKind::R_BRACE
                    | SyntaxKind::COMMA
                    | SyntaxKind::COLON
                    | SyntaxKind::DOT
                    | SyntaxKind::DOT_DOT
                    | SyntaxKind::DOUBLE_COLON
                    | SyntaxKind::DOUBLE_ARROW
                    | SyntaxKind::ARROW
                    | SyntaxKind::EQUAL
            )
        })
    }
}

ast_node!(UnaryExpr, UNARY_EXPR);

impl UnaryExpr {
    /// The operator token (e.g. `-`, `-.`, `not`).
    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.syntax
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| !t.kind().is_trivia())
    }
    pub fn operand(&self) -> Option<Expr> {
        child_node(&self.syntax, self.file_id())
    }
}

ast_node!(CallExpr, CALL_EXPR);

impl CallExpr {
    pub fn callee(&self) -> Option<Expr> {
        nth_child_node(&self.syntax, self.file_id(), 0)
    }
    pub fn arg(&self) -> Option<Expr> {
        nth_child_node(&self.syntax, self.file_id(), 1)
    }
}

ast_node!(FieldExpr, FIELD_EXPR);

impl FieldExpr {
    pub fn base(&self) -> Option<Expr> {
        child_node(&self.syntax, self.file_id())
    }

    pub fn field_name_spanned(&self) -> Option<Spanned<String>> {
        let tokens = non_trivia_tokens(&self.syntax);
        let mut index = 0;
        while index < tokens.len() {
            if tokens[index].kind() != SyntaxKind::DOT {
                index += 1;
                continue;
            }
            let next = tokens.get(index + 1)?;
            if next.kind() == SyntaxKind::IDENT {
                return Some(
                    next.text()
                        .to_string()
                        .with_span(span_from_text_range(self.file_id(), next.text_range())),
                );
            }
            if next.kind() == SyntaxKind::L_SQUARE
                && let (Some(op), Some(end)) = (tokens.get(index + 2), tokens.get(index + 3))
                && op.kind().is_operator_token()
                && end.kind() == SyntaxKind::R_SQUARE
            {
                return Some(format!("[{}]", op.text()).with_span(span_from_text_range(
                    self.file_id(),
                    rowan::TextRange::new(next.text_range().start(), end.text_range().end()),
                )));
            }
            return None;
        }
        None
    }

    /// The field name token after `.`.
    pub fn field_token(&self) -> Option<SyntaxToken> {
        // The IDENT token that follows the DOT
        let mut after_dot = false;
        for el in self.syntax.children_with_tokens() {
            if let Some(tok) = el.into_token() {
                if tok.kind() == SyntaxKind::DOT {
                    after_dot = true;
                } else if after_dot && tok.kind() == SyntaxKind::IDENT {
                    return Some(tok);
                }
            }
        }
        None
    }
}

ast_node!(ParenExpr, PAREN_EXPR);

impl ParenExpr {
    /// The expressions inside the parentheses.
    /// - 1 element without comma: grouping
    /// - 1 element with comma: singleton tuple
    /// - 2+ elements: tuple
    pub fn inner_exprs(&self) -> Vec<Expr> {
        self.syntax
            .children()
            .filter_map(|node| Expr::cast(node).map(|node| node.with_file_id(self.file_id())))
            .collect()
    }
    /// True if this contains commas (i.e. is a tuple).
    pub fn is_tuple(&self) -> bool {
        self.inner_exprs().len() > 1 || node_has_token(&self.syntax, SyntaxKind::COMMA)
    }
}

ast_node!(ArrayExpr, ARRAY_EXPR);

impl ArrayExpr {
    /// Non-splat element expressions.
    pub fn exprs(&self) -> Vec<Expr> {
        self.syntax
            .children()
            .filter_map(|node| Expr::cast(node).map(|node| node.with_file_id(self.file_id())))
            .collect()
    }
    /// Splat (`..expr`) elements.
    pub fn splats(&self) -> Vec<ArraySplat> {
        child_nodes(&self.syntax, self.file_id())
    }
}

ast_node!(StructExpr, STRUCT_EXPR);

impl StructExpr {
    pub fn fields(&self) -> Vec<StructField> {
        child_nodes(&self.syntax, self.file_id())
    }
}

ast_node!(StructField, STRUCT_FIELD);
impl HasName for StructField {
}

impl StructField {
    pub fn value(&self) -> Option<Expr> {
        child_node(&self.syntax, self.file_id())
    }
}

ast_node!(Literal, LITERAL);

impl Literal {
    /// The single token inside the LITERAL node.
    pub fn token(&self) -> Option<SyntaxToken> {
        self.syntax
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| !t.kind().is_trivia())
    }
}

ast_node!(Ident, IDENT_NODE);
impl HasName for Ident {
}

ast_node!(Path, PATH);

impl Path {
    /// All path segments without the optional `root` prefix.
    /// Bracketed operators are normalized to `[<op>]`.
    pub fn segments(&self) -> Vec<String> {
        all_identifier_texts(&self.syntax)
    }

    pub fn is_rooted(&self) -> bool {
        non_trivia_tokens(&self.syntax)
            .first()
            .is_some_and(|token| token.kind() == SyntaxKind::ROOT_KW)
    }

    pub fn is_bundle_rooted(&self) -> bool {
        non_trivia_tokens(&self.syntax)
            .first()
            .is_some_and(|token| token.kind() == SyntaxKind::BUNDLE_KW)
    }

    /// For a path like `A::B::name`, returns `A::B`.
    pub fn qualifier(&self) -> Option<String> {
        let segs = self.segments();
        (segs.len() >= 2).then(|| segs[..segs.len() - 1].join("::"))
    }
    /// The final name segment.
    pub fn name_text(&self) -> Option<String> {
        self.segments().last().cloned()
    }

    pub fn has_dollar_prefix(&self) -> bool {
        self.syntax
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .any(|t| t.kind() == SyntaxKind::DOLLAR)
    }
}

ast_node!(ArraySplat, ARRAY_SPLAT);

impl ArraySplat {
    pub fn expr(&self) -> Option<Expr> {
        child_node(&self.syntax, self.file_id())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Patterns
// ═══════════════════════════════════════════════════════════════════════

ast_node!(PatTuple, PAT_TUPLE);

impl PatTuple {
    pub fn patterns(&self) -> Vec<Pattern> {
        self.syntax
            .children()
            .filter_map(|node| Pattern::cast(node).map(|node| node.with_file_id(self.file_id())))
            .collect()
    }

    pub fn is_tuple(&self) -> bool {
        self.patterns().len() > 1 || node_has_token(&self.syntax, SyntaxKind::COMMA)
    }
}

ast_node!(PatArray, PAT_ARRAY);

impl PatArray {
    pub fn patterns(&self) -> Vec<Pattern> {
        self.syntax
            .children()
            .filter_map(|node| Pattern::cast(node).map(|node| node.with_file_id(self.file_id())))
            .collect()
    }
    pub fn rest_patterns(&self) -> Vec<PatRest> {
        child_nodes(&self.syntax, self.file_id())
    }
}

ast_node!(PatStruct, PAT_STRUCT);

impl PatStruct {
    pub fn fields(&self) -> Vec<PatField> {
        child_nodes(&self.syntax, self.file_id())
    }
}

ast_node!(PatConstructor, PAT_CONSTRUCTOR);

impl PatConstructor {
    /// The name of the constructor — either a bare identifier or a
    /// qualified path (`Module::Ctor`).
    pub fn head(&self) -> Option<PathOrIdent> {
        self.syntax
            .children()
            .find_map(|node| PathOrIdent::cast(node).map(|node| node.with_file_id(self.file_id())))
    }
    /// The payload pattern after the constructor head (second Pattern child).
    pub fn payload(&self) -> Option<Pattern> {
        nth_child_node(&self.syntax, self.file_id(), 1)
    }
}

ast_node!(PatTypeHint, PAT_TYPE_HINT);

impl PatTypeHint {
    pub fn pattern(&self) -> Option<Pattern> {
        child_node(&self.syntax, self.file_id())
    }
    pub fn ty(&self) -> Option<TypeExpr> {
        child_node_after_token(&self.syntax, self.file_id(), SyntaxKind::COLON)
    }
}

ast_node!(PatRest, PAT_REST);

impl PatRest {
    /// The optional binding name after `..`.
    pub fn binding_token(&self) -> Option<SyntaxToken> {
        child_token(&self.syntax, SyntaxKind::IDENT)
    }

    pub fn binding_name_spanned(&self) -> Option<Spanned<String>> {
        first_identifier_text_spanned(&self.syntax, self.file_id())
    }
}

ast_node!(PatField, PAT_FIELD);
impl HasName for PatField {
}

impl PatField {
    /// The bound pattern, if present (e.g. `field = pattern`).
    pub fn pattern(&self) -> Option<Pattern> {
        child_node(&self.syntax, self.file_id())
    }
}

// ── Helper types ─────────────────────────────────────────────────────

/// A bare identifier or a qualified path (`Module::Name`).
///
/// Used where exactly those two forms are valid, e.g. the head of a
/// constructor pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathOrIdent {
    Ident(Ident),
    Path(Path),
}

impl PathOrIdent {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::IDENT_NODE => Ident::cast(node).map(Self::Ident),
            SyntaxKind::PATH => Path::cast(node).map(Self::Path),
            _ => None,
        }
    }

    fn with_file_id(
        self,
        file_id: usize,
    ) -> Self {
        match self {
            Self::Ident(ident) => Self::Ident(ident.with_file_id(file_id)),
            Self::Path(path) => Self::Path(path.with_file_id(file_id)),
        }
    }

    pub fn name_text(&self) -> Option<String> {
        match self {
            Self::Ident(id) => HasName::name_text(id),
            Self::Path(p) => p.name_text(),
        }
    }

    pub fn qualifier(&self) -> Option<String> {
        match self {
            Self::Ident(_) => None,
            Self::Path(p) => p.qualifier(),
        }
    }

    pub fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Ident(ident) => ident.syntax(),
            Self::Path(path) => path.syntax(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Enum groupings
// ═══════════════════════════════════════════════════════════════════════

/// A top-level statement inside a module body.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    Bundle(BundleDeclaration),
    Import(ImportStatement),
    Use(UseStatement),
    Let(LetStatement),
    Do(DoStatement),
    Type(TypeStatement),
    Trait(TraitStatement),
    Impl(ImplStatement),
    Module(Module),
    Wasm(WasmStatement),
}

impl AstNode for Statement {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::BUNDLE_DECLARATION => BundleDeclaration::cast(node).map(Self::Bundle),
            SyntaxKind::IMPORT_STATEMENT => ImportStatement::cast(node).map(Self::Import),
            SyntaxKind::USE_STATEMENT => UseStatement::cast(node).map(Self::Use),
            SyntaxKind::LET_STATEMENT => LetStatement::cast(node).map(Self::Let),
            SyntaxKind::DO_STATEMENT => DoStatement::cast(node).map(Self::Do),
            SyntaxKind::TYPE_STATEMENT => TypeStatement::cast(node).map(Self::Type),
            SyntaxKind::TRAIT_STATEMENT => TraitStatement::cast(node).map(Self::Trait),
            SyntaxKind::IMPL_STATEMENT => ImplStatement::cast(node).map(Self::Impl),
            SyntaxKind::MODULE => Module::cast(node).map(Self::Module),
            SyntaxKind::WASM_STATEMENT => WasmStatement::cast(node).map(Self::Wasm),
            _ => None,
        }
    }

    fn with_file_id(
        self,
        file_id: usize,
    ) -> Self {
        match self {
            Self::Bundle(node) => Self::Bundle(node.with_file_id(file_id)),
            Self::Import(node) => Self::Import(node.with_file_id(file_id)),
            Self::Use(node) => Self::Use(node.with_file_id(file_id)),
            Self::Let(node) => Self::Let(node.with_file_id(file_id)),
            Self::Do(node) => Self::Do(node.with_file_id(file_id)),
            Self::Type(node) => Self::Type(node.with_file_id(file_id)),
            Self::Trait(node) => Self::Trait(node.with_file_id(file_id)),
            Self::Impl(node) => Self::Impl(node.with_file_id(file_id)),
            Self::Module(node) => Self::Module(node.with_file_id(file_id)),
            Self::Wasm(node) => Self::Wasm(node.with_file_id(file_id)),
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Bundle(n) => n.syntax(),
            Self::Import(n) => n.syntax(),
            Self::Use(n) => n.syntax(),
            Self::Let(n) => n.syntax(),
            Self::Do(n) => n.syntax(),
            Self::Type(n) => n.syntax(),
            Self::Trait(n) => n.syntax(),
            Self::Impl(n) => n.syntax(),
            Self::Module(n) => n.syntax(),
            Self::Wasm(n) => n.syntax(),
        }
    }

    fn file_id(&self) -> usize {
        match self {
            Self::Bundle(node) => node.file_id(),
            Self::Import(node) => node.file_id(),
            Self::Use(node) => node.file_id(),
            Self::Let(node) => node.file_id(),
            Self::Do(node) => node.file_id(),
            Self::Type(node) => node.file_id(),
            Self::Trait(node) => node.file_id(),
            Self::Impl(node) => node.file_id(),
            Self::Module(node) => node.file_id(),
            Self::Wasm(node) => node.file_id(),
        }
    }
}

impl HasLeadingComments for Statement {
}

/// A type definition (right-hand side of `type Name = ...`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeDef {
    Struct(StructDef),
    Sum(SumDef),
    Alias(TypeAliasDef),
}

impl AstNode for TypeDef {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::STRUCT_DEF => StructDef::cast(node).map(Self::Struct),
            SyntaxKind::SUM_DEF => SumDef::cast(node).map(Self::Sum),
            SyntaxKind::TYPE_ALIAS_DEF => TypeAliasDef::cast(node).map(Self::Alias),
            _ => None,
        }
    }

    fn with_file_id(
        self,
        file_id: usize,
    ) -> Self {
        match self {
            Self::Struct(node) => Self::Struct(node.with_file_id(file_id)),
            Self::Sum(node) => Self::Sum(node.with_file_id(file_id)),
            Self::Alias(node) => Self::Alias(node.with_file_id(file_id)),
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Struct(n) => n.syntax(),
            Self::Sum(n) => n.syntax(),
            Self::Alias(n) => n.syntax(),
        }
    }

    fn file_id(&self) -> usize {
        match self {
            Self::Struct(node) => node.file_id(),
            Self::Sum(node) => node.file_id(),
            Self::Alias(node) => node.file_id(),
        }
    }
}

/// A type expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeExpr {
    Function(FunctionType),
    Application(TypeApplication),
    Tuple(TupleType),
    Array(ArrayType),
    ForAll(ForAllType),
    Unit(Unit),
    Path(Path),
    Ident(Ident),
}

impl AstNode for TypeExpr {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::FUNCTION_TYPE => FunctionType::cast(node).map(Self::Function),
            SyntaxKind::TYPE_APPLICATION => TypeApplication::cast(node).map(Self::Application),
            SyntaxKind::TUPLE_TYPE => TupleType::cast(node).map(Self::Tuple),
            SyntaxKind::ARRAY_TYPE => ArrayType::cast(node).map(Self::Array),
            SyntaxKind::FORALL_TYPE => ForAllType::cast(node).map(Self::ForAll),
            SyntaxKind::UNIT => Unit::cast(node).map(Self::Unit),
            SyntaxKind::PATH => Path::cast(node).map(Self::Path),
            SyntaxKind::IDENT_NODE => Ident::cast(node).map(Self::Ident),
            _ => None,
        }
    }

    fn with_file_id(
        self,
        file_id: usize,
    ) -> Self {
        match self {
            Self::Function(node) => Self::Function(node.with_file_id(file_id)),
            Self::Application(node) => Self::Application(node.with_file_id(file_id)),
            Self::Tuple(node) => Self::Tuple(node.with_file_id(file_id)),
            Self::Array(node) => Self::Array(node.with_file_id(file_id)),
            Self::ForAll(node) => Self::ForAll(node.with_file_id(file_id)),
            Self::Unit(node) => Self::Unit(node.with_file_id(file_id)),
            Self::Path(node) => Self::Path(node.with_file_id(file_id)),
            Self::Ident(node) => Self::Ident(node.with_file_id(file_id)),
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Function(n) => n.syntax(),
            Self::Application(n) => n.syntax(),
            Self::Tuple(n) => n.syntax(),
            Self::Array(n) => n.syntax(),
            Self::ForAll(n) => n.syntax(),
            Self::Unit(n) => n.syntax(),
            Self::Path(n) => n.syntax(),
            Self::Ident(n) => n.syntax(),
        }
    }

    fn file_id(&self) -> usize {
        match self {
            Self::Function(node) => node.file_id(),
            Self::Application(node) => node.file_id(),
            Self::Tuple(node) => node.file_id(),
            Self::Array(node) => node.file_id(),
            Self::ForAll(node) => node.file_id(),
            Self::Unit(node) => node.file_id(),
            Self::Path(node) => node.file_id(),
            Self::Ident(node) => node.file_id(),
        }
    }
}

/// A value expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Let(LetExpr),
    Use(UseExpr),
    Fn(FnExpr),
    FnShorthand(FnShorthandExpr),
    If(IfExpr),
    Match(MatchExpr),
    InlineWasm(InlineWasmExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Call(CallExpr),
    Field(FieldExpr),
    Paren(ParenExpr),
    Array(ArrayExpr),
    Struct(StructExpr),
    Literal(Literal),
    Unit(Unit),
    Ident(Ident),
    Path(Path),
}

impl AstNode for Expr {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::LET_EXPR => LetExpr::cast(node).map(Self::Let),
            SyntaxKind::USE_EXPR => UseExpr::cast(node).map(Self::Use),
            SyntaxKind::FN_EXPR => FnExpr::cast(node).map(Self::Fn),
            SyntaxKind::FN_SHORTHAND_EXPR => FnShorthandExpr::cast(node).map(Self::FnShorthand),
            SyntaxKind::IF_EXPR => IfExpr::cast(node).map(Self::If),
            SyntaxKind::MATCH_EXPR => MatchExpr::cast(node).map(Self::Match),
            SyntaxKind::INLINE_WASM_EXPR => InlineWasmExpr::cast(node).map(Self::InlineWasm),
            SyntaxKind::BINARY_EXPR => BinaryExpr::cast(node).map(Self::Binary),
            SyntaxKind::UNARY_EXPR => UnaryExpr::cast(node).map(Self::Unary),
            SyntaxKind::CALL_EXPR => CallExpr::cast(node).map(Self::Call),
            SyntaxKind::FIELD_EXPR => FieldExpr::cast(node).map(Self::Field),
            SyntaxKind::PAREN_EXPR => ParenExpr::cast(node).map(Self::Paren),
            SyntaxKind::ARRAY_EXPR => ArrayExpr::cast(node).map(Self::Array),
            SyntaxKind::STRUCT_EXPR => StructExpr::cast(node).map(Self::Struct),
            SyntaxKind::LITERAL => Literal::cast(node).map(Self::Literal),
            SyntaxKind::UNIT => Unit::cast(node).map(Self::Unit),
            SyntaxKind::IDENT_NODE => Ident::cast(node).map(Self::Ident),
            SyntaxKind::PATH => Path::cast(node).map(Self::Path),
            _ => None,
        }
    }

    fn with_file_id(
        self,
        file_id: usize,
    ) -> Self {
        match self {
            Self::Let(node) => Self::Let(node.with_file_id(file_id)),
            Self::Use(node) => Self::Use(node.with_file_id(file_id)),
            Self::Fn(node) => Self::Fn(node.with_file_id(file_id)),
            Self::FnShorthand(node) => Self::FnShorthand(node.with_file_id(file_id)),
            Self::If(node) => Self::If(node.with_file_id(file_id)),
            Self::Match(node) => Self::Match(node.with_file_id(file_id)),
            Self::InlineWasm(node) => Self::InlineWasm(node.with_file_id(file_id)),
            Self::Binary(node) => Self::Binary(node.with_file_id(file_id)),
            Self::Unary(node) => Self::Unary(node.with_file_id(file_id)),
            Self::Call(node) => Self::Call(node.with_file_id(file_id)),
            Self::Field(node) => Self::Field(node.with_file_id(file_id)),
            Self::Paren(node) => Self::Paren(node.with_file_id(file_id)),
            Self::Array(node) => Self::Array(node.with_file_id(file_id)),
            Self::Struct(node) => Self::Struct(node.with_file_id(file_id)),
            Self::Literal(node) => Self::Literal(node.with_file_id(file_id)),
            Self::Unit(node) => Self::Unit(node.with_file_id(file_id)),
            Self::Ident(node) => Self::Ident(node.with_file_id(file_id)),
            Self::Path(node) => Self::Path(node.with_file_id(file_id)),
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Let(n) => n.syntax(),
            Self::Use(n) => n.syntax(),
            Self::Fn(n) => n.syntax(),
            Self::FnShorthand(n) => n.syntax(),
            Self::If(n) => n.syntax(),
            Self::Match(n) => n.syntax(),
            Self::InlineWasm(n) => n.syntax(),
            Self::Binary(n) => n.syntax(),
            Self::Unary(n) => n.syntax(),
            Self::Call(n) => n.syntax(),
            Self::Field(n) => n.syntax(),
            Self::Paren(n) => n.syntax(),
            Self::Array(n) => n.syntax(),
            Self::Struct(n) => n.syntax(),
            Self::Literal(n) => n.syntax(),
            Self::Unit(n) => n.syntax(),
            Self::Ident(n) => n.syntax(),
            Self::Path(n) => n.syntax(),
        }
    }

    fn file_id(&self) -> usize {
        match self {
            Self::Let(node) => node.file_id(),
            Self::Use(node) => node.file_id(),
            Self::Fn(node) => node.file_id(),
            Self::FnShorthand(node) => node.file_id(),
            Self::If(node) => node.file_id(),
            Self::Match(node) => node.file_id(),
            Self::InlineWasm(node) => node.file_id(),
            Self::Binary(node) => node.file_id(),
            Self::Unary(node) => node.file_id(),
            Self::Call(node) => node.file_id(),
            Self::Field(node) => node.file_id(),
            Self::Paren(node) => node.file_id(),
            Self::Array(node) => node.file_id(),
            Self::Struct(node) => node.file_id(),
            Self::Literal(node) => node.file_id(),
            Self::Unit(node) => node.file_id(),
            Self::Ident(node) => node.file_id(),
            Self::Path(node) => node.file_id(),
        }
    }
}

/// A pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    Ident(Ident),
    Literal(Literal),
    Unit(Unit),
    Tuple(PatTuple),
    Array(PatArray),
    Struct(PatStruct),
    Constructor(PatConstructor),
    TypeHint(PatTypeHint),
    Path(Path),
}

impl AstNode for Pattern {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::IDENT_NODE => Ident::cast(node).map(Self::Ident),
            SyntaxKind::LITERAL => Literal::cast(node).map(Self::Literal),
            SyntaxKind::UNIT => Unit::cast(node).map(Self::Unit),
            SyntaxKind::PAT_TUPLE => PatTuple::cast(node).map(Self::Tuple),
            SyntaxKind::PAT_ARRAY => PatArray::cast(node).map(Self::Array),
            SyntaxKind::PAT_STRUCT => PatStruct::cast(node).map(Self::Struct),
            SyntaxKind::PAT_CONSTRUCTOR => PatConstructor::cast(node).map(Self::Constructor),
            SyntaxKind::PAT_TYPE_HINT => PatTypeHint::cast(node).map(Self::TypeHint),
            SyntaxKind::PATH => Path::cast(node).map(Self::Path),
            _ => None,
        }
    }

    fn with_file_id(
        self,
        file_id: usize,
    ) -> Self {
        match self {
            Self::Ident(node) => Self::Ident(node.with_file_id(file_id)),
            Self::Literal(node) => Self::Literal(node.with_file_id(file_id)),
            Self::Unit(node) => Self::Unit(node.with_file_id(file_id)),
            Self::Tuple(node) => Self::Tuple(node.with_file_id(file_id)),
            Self::Array(node) => Self::Array(node.with_file_id(file_id)),
            Self::Struct(node) => Self::Struct(node.with_file_id(file_id)),
            Self::Constructor(node) => Self::Constructor(node.with_file_id(file_id)),
            Self::TypeHint(node) => Self::TypeHint(node.with_file_id(file_id)),
            Self::Path(node) => Self::Path(node.with_file_id(file_id)),
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Ident(n) => n.syntax(),
            Self::Literal(n) => n.syntax(),
            Self::Unit(n) => n.syntax(),
            Self::Tuple(n) => n.syntax(),
            Self::Array(n) => n.syntax(),
            Self::Struct(n) => n.syntax(),
            Self::Constructor(n) => n.syntax(),
            Self::TypeHint(n) => n.syntax(),
            Self::Path(n) => n.syntax(),
        }
    }

    fn file_id(&self) -> usize {
        match self {
            Self::Ident(node) => node.file_id(),
            Self::Literal(node) => node.file_id(),
            Self::Unit(node) => node.file_id(),
            Self::Tuple(node) => node.file_id(),
            Self::Array(node) => node.file_id(),
            Self::Struct(node) => node.file_id(),
            Self::Constructor(node) => node.file_id(),
            Self::TypeHint(node) => node.file_id(),
            Self::Path(node) => node.file_id(),
        }
    }
}

ast_node!(Sexpr, SEXPR);
ast_node!(SexprField, SEXPR_FIELD);

impl Sexpr {
    pub fn items(&self) -> Vec<SexprItem> {
        self.syntax
            .children()
            .filter_map(|node| sexpr_item_from_node(node, self.file_id()))
            .collect()
    }
}

impl SexprField {
    pub fn lhs_token(&self) -> Option<SyntaxToken> {
        self.ident_tokens().first().cloned()
    }
    pub fn rhs_token(&self) -> Option<SyntaxToken> {
        self.ident_tokens().get(1).cloned()
    }
    fn ident_tokens(&self) -> Vec<SyntaxToken> {
        self.syntax
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .filter(|t| !t.kind().is_trivia())
            .filter(|t| is_sexpr_ident_token(t.kind()))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SexprItem {
    List(Sexpr),
    Path(Path),
    Field(SexprField),
    Atom(SexprAtom),
}

impl SexprItem {
    pub fn span(&self) -> Span {
        match self {
            SexprItem::List(sexpr) => sexpr.span(),
            SexprItem::Path(path) => path.span(),
            SexprItem::Field(field) => field.span(),
            SexprItem::Atom(atom) => Span::from(atom.token().text_range()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SexprAtom {
    Ident(SyntaxToken),
    SymbolIdent(SyntaxToken),
    String(SyntaxToken),
    Integer(SyntaxToken),
    Float(SyntaxToken),
    Bool(SyntaxToken, bool),
}

impl SexprAtom {
    pub fn token(&self) -> &SyntaxToken {
        match self {
            SexprAtom::Ident(token)
            | SexprAtom::SymbolIdent(token)
            | SexprAtom::String(token)
            | SexprAtom::Integer(token)
            | SexprAtom::Float(token)
            | SexprAtom::Bool(token, _) => token,
        }
    }
    pub fn bool_value(&self) -> Option<bool> {
        match self {
            SexprAtom::Bool(_, value) => Some(*value),
            _ => None,
        }
    }
}

fn sexpr_item_from_node(
    node: SyntaxNode,
    file_id: usize,
) -> Option<SexprItem> {
    match node.kind() {
        SyntaxKind::SEXPR => {
            Sexpr::cast(node).map(|node| SexprItem::List(node.with_file_id(file_id)))
        }
        SyntaxKind::PATH => {
            Path::cast(node).map(|node| SexprItem::Path(node.with_file_id(file_id)))
        }
        SyntaxKind::SEXPR_FIELD => {
            SexprField::cast(node).map(|node| SexprItem::Field(node.with_file_id(file_id)))
        }
        _ => sexpr_atom_from_node(node).map(SexprItem::Atom),
    }
}

fn sexpr_atom_from_node(node: SyntaxNode) -> Option<SexprAtom> {
    let kind = node.kind();
    let token = if kind == SyntaxKind::IDENT {
        first_token_where(&node, is_sexpr_ident_token)?
    } else {
        child_token(&node, kind)?
    };
    match kind {
        SyntaxKind::IDENT => {
            if node_has_token(&node, SyntaxKind::DOLLAR) {
                Some(SexprAtom::SymbolIdent(token))
            } else {
                Some(SexprAtom::Ident(token))
            }
        }
        SyntaxKind::STRING => Some(SexprAtom::String(token)),
        SyntaxKind::INTEGER => Some(SexprAtom::Integer(token)),
        SyntaxKind::REAL => Some(SexprAtom::Float(token)),
        SyntaxKind::TRUE_KW => Some(SexprAtom::Bool(token, true)),
        SyntaxKind::FALSE_KW => Some(SexprAtom::Bool(token, false)),
        _ => None,
    }
}

fn node_has_token(
    node: &SyntaxNode,
    kind: SyntaxKind,
) -> bool {
    node.children_with_tokens()
        .filter_map(|el| el.into_token())
        .any(|token| token.kind() == kind)
}

fn is_sexpr_ident_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::IDENT
            | SyntaxKind::MINUS
            | SyntaxKind::MODULE_KW
            | SyntaxKind::IMPORT_KW
            | SyntaxKind::USE_KW
            | SyntaxKind::AS_KW
            | SyntaxKind::END_KW
            | SyntaxKind::MATCH_KW
            | SyntaxKind::WITH_KW
            | SyntaxKind::LET_KW
            | SyntaxKind::TYPE_KW
            | SyntaxKind::TRAIT_KW
            | SyntaxKind::IMPL_KW
            | SyntaxKind::DO_KW
            | SyntaxKind::OF_KW
            | SyntaxKind::IN_KW
            | SyntaxKind::IF_KW
            | SyntaxKind::THEN_KW
            | SyntaxKind::ELSE_KW
            | SyntaxKind::AND_KW
            | SyntaxKind::OR_KW
            | SyntaxKind::XOR_KW
            | SyntaxKind::NOT_KW
            | SyntaxKind::FN_KW
            | SyntaxKind::WASM_KW
            | SyntaxKind::FOR_KW
            | SyntaxKind::ROOT_KW
    )
}
