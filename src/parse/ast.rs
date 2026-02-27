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
    /// Access the underlying untyped node.
    fn syntax(&self) -> &SyntaxNode;

    fn span(&self) -> Span {
        let mut tokens = self
            .syntax()
            .descendants_with_tokens()
            .filter_map(|el| el.into_token())
            .filter(|t| !t.kind().is_trivia());
        let Some(first) = tokens.next() else {
            return Span::Generated;
        };
        rowan::TextRange::new(
            first.text_range().start(),
            tokens
                .last()
                .unwrap_or_else(|| first.clone())
                .text_range()
                .end(),
        )
        .into()
    }
}

// ── Helper traits ────────────────────────────────────────────────────

/// Nodes that carry a name via an `IDENT` child token.
pub trait HasName: AstNode {
    fn name_token(&self) -> Option<SyntaxToken> {
        child_token(self.syntax(), SyntaxKind::IDENT)
    }
    fn name_text(&self) -> Option<String> {
        self.name_token().map(|t| t.text().to_string())
    }
    fn name_text_spanned(&self) -> Option<Spanned<String>> {
        self.name_token()
            .map(|t| t.text().to_string().with_span(t.text_range().into()))
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
fn child_node<N: AstNode>(parent: &SyntaxNode) -> Option<N> {
    parent.children().find_map(N::cast)
}

/// Find all child nodes that can be cast to `N`.
fn child_nodes<N: AstNode>(parent: &SyntaxNode) -> Vec<N> {
    parent.children().filter_map(N::cast).collect()
}

/// Find the nth child node that can be cast to `N` (0-indexed).
fn nth_child_node<N: AstNode>(
    parent: &SyntaxNode,
    n: usize,
) -> Option<N> {
    parent.children().filter_map(N::cast).nth(n)
}

/// Find the first child node that can be cast to `N` after a marker token.
fn child_node_after_token<N: AstNode>(
    parent: &SyntaxNode,
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
                if seen_marker && let Some(cast) = N::cast(node) {
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

// ── Macro to reduce per-node boilerplate ─────────────────────────────

macro_rules! ast_node {
    ($name:ident, $kind:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            syntax: SyntaxNode,
        }

        impl AstNode for $name {
            fn cast(node: SyntaxNode) -> Option<Self> {
                if node.kind() == SyntaxKind::$kind {
                    Some(Self { syntax: node })
                } else {
                    None
                }
            }
            fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }
        }
    };
}

// ═══════════════════════════════════════════════════════════════════════
// Program structure
// ═══════════════════════════════════════════════════════════════════════

ast_node!(SourceFile, SOURCE_FILE);

impl SourceFile {
    pub fn modules(&self) -> Vec<Module> {
        child_nodes(&self.syntax)
    }
}

ast_node!(Module, MODULE);
impl HasName for Module {
}

impl Module {
    pub fn statements(&self) -> Vec<Statement> {
        self.syntax.children().filter_map(Statement::cast).collect()
    }
}

ast_node!(LetStatement, LET_STATEMENT);
impl HasLeadingComments for LetStatement {
}

impl LetStatement {
    pub fn pattern(&self) -> Option<Pattern> {
        child_node(&self.syntax)
    }
    pub fn value(&self) -> Option<Expr> {
        child_node_after_token(&self.syntax, SyntaxKind::EQUAL)
    }
}

ast_node!(TypeStatement, TYPE_STATEMENT);
impl HasName for TypeStatement {
}
impl HasLeadingComments for TypeStatement {
}

impl TypeStatement {
    /// Type parameters declared after `:`, e.g. `type Option: a = ...`.
    /// Returns the IDENT tokens between `:` and `=`.
    pub fn type_params(&self) -> Vec<Spanned<String>> {
        let mut after_colon = false;
        self.syntax
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .filter(|t| !t.kind().is_trivia())
            .filter(move |t| {
                if t.kind() == SyntaxKind::COLON {
                    after_colon = true;
                    return false;
                }
                if t.kind() == SyntaxKind::EQUAL {
                    after_colon = false;
                    return false;
                }
                after_colon && t.kind() == SyntaxKind::IDENT
            })
            .map(|t| t.text().to_string().with_span(t.text_range().into()))
            .collect()
    }

    pub fn type_def(&self) -> Option<TypeDef> {
        self.syntax.children().find_map(TypeDef::cast)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Type definitions
// ═══════════════════════════════════════════════════════════════════════

ast_node!(StructDef, STRUCT_DEF);

impl StructDef {
    pub fn fields(&self) -> Vec<FieldDecl> {
        child_nodes(&self.syntax)
    }
}

ast_node!(SumDef, SUM_DEF);

impl SumDef {
    pub fn variants(&self) -> Vec<Variant> {
        child_nodes(&self.syntax)
    }
}

ast_node!(Variant, VARIANT);
impl HasName for Variant {
}

impl Variant {
    /// The optional payload type expression after the variant name.
    pub fn payload_type(&self) -> Option<TypeExpr> {
        self.syntax.children().find_map(TypeExpr::cast)
    }
}

ast_node!(FieldDecl, FIELD_DECL);
impl HasName for FieldDecl {
}

impl FieldDecl {
    pub fn ty(&self) -> Option<TypeExpr> {
        self.syntax.children().find_map(TypeExpr::cast)
    }
}

ast_node!(TypeAliasDef, TYPE_ALIAS_DEF);

impl TypeAliasDef {
    pub fn type_expr(&self) -> Option<TypeExpr> {
        self.syntax.children().find_map(TypeExpr::cast)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Type expressions
// ═══════════════════════════════════════════════════════════════════════

ast_node!(FunctionType, FUNCTION_TYPE);

impl FunctionType {
    pub fn param_type(&self) -> Option<TypeExpr> {
        nth_type_expr(&self.syntax, 0)
    }
    pub fn return_type(&self) -> Option<TypeExpr> {
        nth_type_expr(&self.syntax, 1)
    }
}

ast_node!(TypeApplication, TYPE_APPLICATION);

impl TypeApplication {
    pub fn base(&self) -> Option<TypeExpr> {
        nth_type_expr(&self.syntax, 0)
    }
    pub fn args(&self) -> Vec<TypeExpr> {
        self.syntax
            .children()
            .filter_map(TypeExpr::cast)
            .skip(1)
            .collect()
    }
}

ast_node!(TupleType, TUPLE_TYPE);

impl TupleType {
    pub fn fields(&self) -> Vec<TypeExpr> {
        self.syntax.children().filter_map(TypeExpr::cast).collect()
    }
}

ast_node!(ArrayType, ARRAY_TYPE);
ast_node!(Unit, UNIT);

/// Helper: get the nth TypeExpr child of a node.
fn nth_type_expr(
    parent: &SyntaxNode,
    n: usize,
) -> Option<TypeExpr> {
    parent.children().filter_map(TypeExpr::cast).nth(n)
}

// ═══════════════════════════════════════════════════════════════════════
// Value expressions
// ═══════════════════════════════════════════════════════════════════════

ast_node!(LetExpr, LET_EXPR);

impl LetExpr {
    pub fn pattern(&self) -> Option<Pattern> {
        child_node(&self.syntax)
    }
    /// The value being bound (first Expr child).
    pub fn value(&self) -> Option<Expr> {
        child_node_after_token(&self.syntax, SyntaxKind::EQUAL)
    }
    /// The body after `in` (second Expr child).
    pub fn body(&self) -> Option<Expr> {
        child_node_after_token(&self.syntax, SyntaxKind::IN_KW)
    }
}

ast_node!(FnExpr, FN_EXPR);

impl FnExpr {
    pub fn params(&self) -> Vec<Param> {
        child_nodes(&self.syntax)
    }
    pub fn body(&self) -> Option<Expr> {
        child_node(&self.syntax)
    }
}

ast_node!(FnShorthandExpr, FN_SHORTHAND_EXPR);

impl FnShorthandExpr {
    pub fn arms(&self) -> Vec<MatchArm> {
        child_nodes(&self.syntax)
    }
}

ast_node!(IfExpr, IF_EXPR);

impl IfExpr {
    pub fn condition(&self) -> Option<Expr> {
        nth_child_node(&self.syntax, 0)
    }
    pub fn then_branch(&self) -> Option<Expr> {
        nth_child_node(&self.syntax, 1)
    }
    pub fn else_branch(&self) -> Option<Expr> {
        nth_child_node(&self.syntax, 2)
    }
}

ast_node!(MatchExpr, MATCH_EXPR);

impl MatchExpr {
    pub fn scrutinee(&self) -> Option<Expr> {
        child_node(&self.syntax)
    }
    pub fn arms(&self) -> Vec<MatchArm> {
        child_nodes(&self.syntax)
    }
}

ast_node!(MatchArm, MATCH_ARM);

impl MatchArm {
    pub fn pattern(&self) -> Option<Pattern> {
        child_node(&self.syntax)
    }
    pub fn body(&self) -> Option<Expr> {
        child_node_after_token(&self.syntax, SyntaxKind::DOUBLE_ARROW)
    }
}

ast_node!(Param, PARAM);
impl HasName for Param {
}

impl Param {
    /// Optional type annotation (present when `(name: type)`).
    pub fn ty(&self) -> Option<TypeExpr> {
        self.syntax.children().find_map(TypeExpr::cast)
    }
}

ast_node!(BinaryExpr, BINARY_EXPR);

impl BinaryExpr {
    pub fn lhs(&self) -> Option<Expr> {
        nth_child_node(&self.syntax, 0)
    }
    pub fn rhs(&self) -> Option<Expr> {
        nth_child_node(&self.syntax, 1)
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
        child_node(&self.syntax)
    }
}

ast_node!(CallExpr, CALL_EXPR);

impl CallExpr {
    pub fn callee(&self) -> Option<Expr> {
        nth_child_node(&self.syntax, 0)
    }
    pub fn arg(&self) -> Option<Expr> {
        nth_child_node(&self.syntax, 1)
    }
}

ast_node!(FieldExpr, FIELD_EXPR);

impl FieldExpr {
    pub fn base(&self) -> Option<Expr> {
        child_node(&self.syntax)
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
    /// - 1 element: grouping
    /// - 2+ elements: tuple
    pub fn inner_exprs(&self) -> Vec<Expr> {
        self.syntax.children().filter_map(Expr::cast).collect()
    }
    /// True if this contains commas (i.e. is a tuple).
    pub fn is_tuple(&self) -> bool {
        self.inner_exprs().len() > 1
    }
    /// If this wraps an operator-as-value, return it.
    pub fn operator(&self) -> Option<OperatorExpr> {
        child_node(&self.syntax)
    }
}

ast_node!(ArrayExpr, ARRAY_EXPR);

impl ArrayExpr {
    /// Non-splat element expressions.
    pub fn exprs(&self) -> Vec<Expr> {
        self.syntax.children().filter_map(Expr::cast).collect()
    }
    /// Splat (`..expr`) elements.
    pub fn splats(&self) -> Vec<ArraySplat> {
        child_nodes(&self.syntax)
    }
}

ast_node!(StructExpr, STRUCT_EXPR);

impl StructExpr {
    pub fn fields(&self) -> Vec<StructField> {
        child_nodes(&self.syntax)
    }
}

ast_node!(StructField, STRUCT_FIELD);
impl HasName for StructField {
}

impl StructField {
    pub fn value(&self) -> Option<Expr> {
        child_node(&self.syntax)
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
    /// All IDENT tokens (1 for simple name, 2 for `Module::name`).
    pub fn segments(&self) -> Vec<SyntaxToken> {
        self.syntax
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .filter(|t| t.kind() == SyntaxKind::IDENT)
            .collect()
    }
    /// For a path like `Module::name`, the module qualifier.
    pub fn qualifier(&self) -> Option<SyntaxToken> {
        let segs = self.segments();
        (segs.len() == 2).then(|| segs[0].clone())
    }
    /// The final name segment.
    pub fn name_text(&self) -> Option<String> {
        self.segments().last().map(|t| t.text().to_string())
    }
}

ast_node!(ArraySplat, ARRAY_SPLAT);

impl ArraySplat {
    pub fn expr(&self) -> Option<Expr> {
        child_node(&self.syntax)
    }
}

ast_node!(OperatorExpr, OPERATOR_EXPR);

impl OperatorExpr {
    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.syntax
            .children_with_tokens()
            .filter_map(|el| el.into_token())
            .find(|t| !t.kind().is_trivia())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Patterns
// ═══════════════════════════════════════════════════════════════════════

ast_node!(PatTuple, PAT_TUPLE);

impl PatTuple {
    pub fn patterns(&self) -> Vec<Pattern> {
        self.syntax.children().filter_map(Pattern::cast).collect()
    }
}

ast_node!(PatArray, PAT_ARRAY);

impl PatArray {
    pub fn patterns(&self) -> Vec<Pattern> {
        self.syntax.children().filter_map(Pattern::cast).collect()
    }
    pub fn rest_patterns(&self) -> Vec<PatRest> {
        child_nodes(&self.syntax)
    }
}

ast_node!(PatStruct, PAT_STRUCT);

impl PatStruct {
    pub fn fields(&self) -> Vec<PatField> {
        child_nodes(&self.syntax)
    }
}

ast_node!(PatConstructor, PAT_CONSTRUCTOR);

impl PatConstructor {
    /// The name of the constructor — either a bare identifier or a
    /// qualified path (`Module::Ctor`).
    pub fn head(&self) -> Option<PathOrIdent> {
        self.syntax.children().find_map(PathOrIdent::cast)
    }
    /// The payload pattern after `of` (second Pattern child).
    pub fn payload(&self) -> Option<Pattern> {
        nth_child_node(&self.syntax, 1)
    }
}

ast_node!(PatTypeHint, PAT_TYPE_HINT);

impl PatTypeHint {
    pub fn pattern(&self) -> Option<Pattern> {
        child_node(&self.syntax)
    }
    pub fn ty(&self) -> Option<TypeExpr> {
        self.syntax.children().find_map(TypeExpr::cast)
    }
}

ast_node!(PatRest, PAT_REST);

impl PatRest {
    /// The optional binding name after `..`.
    pub fn binding_token(&self) -> Option<SyntaxToken> {
        child_token(&self.syntax, SyntaxKind::IDENT)
    }
}

ast_node!(PatField, PAT_FIELD);
impl HasName for PatField {
}

impl PatField {
    /// The bound pattern, if present (e.g. `field = pattern`).
    pub fn pattern(&self) -> Option<Pattern> {
        child_node(&self.syntax)
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

    pub fn name_text(&self) -> Option<String> {
        match self {
            Self::Ident(id) => HasName::name_text(id),
            Self::Path(p) => p.name_text(),
        }
    }

    pub fn qualifier(&self) -> Option<SyntaxToken> {
        match self {
            Self::Ident(_) => None,
            Self::Path(p) => p.qualifier(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Enum groupings
// ═══════════════════════════════════════════════════════════════════════

/// A top-level statement inside a module body.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    Let(LetStatement),
    Type(TypeStatement),
}

impl AstNode for Statement {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::LET_STATEMENT => LetStatement::cast(node).map(Self::Let),
            SyntaxKind::TYPE_STATEMENT => TypeStatement::cast(node).map(Self::Type),
            _ => None,
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Let(n) => n.syntax(),
            Self::Type(n) => n.syntax(),
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
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Struct(n) => n.syntax(),
            Self::Sum(n) => n.syntax(),
            Self::Alias(n) => n.syntax(),
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
            SyntaxKind::UNIT => Unit::cast(node).map(Self::Unit),
            SyntaxKind::PATH => Path::cast(node).map(Self::Path),
            SyntaxKind::IDENT_NODE => Ident::cast(node).map(Self::Ident),
            _ => None,
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Function(n) => n.syntax(),
            Self::Application(n) => n.syntax(),
            Self::Tuple(n) => n.syntax(),
            Self::Array(n) => n.syntax(),
            Self::Unit(n) => n.syntax(),
            Self::Path(n) => n.syntax(),
            Self::Ident(n) => n.syntax(),
        }
    }
}

/// A value expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Let(LetExpr),
    Fn(FnExpr),
    FnShorthand(FnShorthandExpr),
    If(IfExpr),
    Match(MatchExpr),
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
    Operator(OperatorExpr),
}

impl AstNode for Expr {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::LET_EXPR => LetExpr::cast(node).map(Self::Let),
            SyntaxKind::FN_EXPR => FnExpr::cast(node).map(Self::Fn),
            SyntaxKind::FN_SHORTHAND_EXPR => FnShorthandExpr::cast(node).map(Self::FnShorthand),
            SyntaxKind::IF_EXPR => IfExpr::cast(node).map(Self::If),
            SyntaxKind::MATCH_EXPR => MatchExpr::cast(node).map(Self::Match),
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
            SyntaxKind::OPERATOR_EXPR => OperatorExpr::cast(node).map(Self::Operator),
            _ => None,
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Let(n) => n.syntax(),
            Self::Fn(n) => n.syntax(),
            Self::FnShorthand(n) => n.syntax(),
            Self::If(n) => n.syntax(),
            Self::Match(n) => n.syntax(),
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
            Self::Operator(n) => n.syntax(),
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
}
