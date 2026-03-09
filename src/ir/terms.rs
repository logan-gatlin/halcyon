use crate::asm::{
    Instruction as WasmInstruction,
    Type as WasmType,
};
use crate::operator::{
    BinaryOp,
    Operator,
    UnaryOp,
};
use crate::parse::ast::AstNode;
use crate::types::Type;
use crate::{
    WithContext,
    WithSpan,
};

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ImmediateValue {
    Unit,
    Integer(i64),
    Real(f64),
    Boolean(bool),
    String(String),
    Glyph(char),
}

impl ImmediateValue {
    pub fn type_of(&self) -> Type {
        match self {
            ImmediateValue::Unit => Type::Unit,
            ImmediateValue::Integer(_) => Type::Integer,
            ImmediateValue::Real(_) => Type::Real,
            ImmediateValue::Boolean(_) => Type::Boolean,
            ImmediateValue::String(_) => Type::String,
            ImmediateValue::Glyph(_) => Type::Glyph,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Default)]
pub enum TermKind<T> {
    Let {
        assignee: Pattern<T>,
        scope: ScopeKind,
        value: Box<Term<T>>,
        then: Box<Term<T>>,
        else_: Box<Term<T>>,
    },
    Immediate(ImmediateValue),
    Identifier(Path),
    Tuple(Vec<Term<T>>),
    Struct(IndexMap<Spanned<String>, Term<T>>),
    Field {
        of: Box<Term<T>>,
        index: Spanned<String>,
    },
    Function {
        parameter_name: Spanned<Path>,
        parameter_type: Option<TypeExpr>,
        captures: Box<[(Path, T)]>,
        body: Box<Term<T>>,
    },
    InlineWasm {
        asserted_type: TypeExpr,
        definitions: IndexMap<Path, WasmType>,
        instructions: Box<[WasmInstruction]>,
    },
    Call {
        callee: Box<Term<T>>,
        argument: Box<Term<T>>,
    },
    Semicolon(Box<Term<T>>, Box<Term<T>>),
    #[default]
    Unreachable,
}

#[derive(Debug, Clone, Default)]
pub struct Term<T> {
    pub comments: String,
    pub kind: TermKind<T>,
    pub span: Span,
    pub type_: T,
}

impl Term<()> {
    fn dummy(kind: TermKind<()>) -> Self {
        Self {
            comments: String::new(),
            kind,
            span: Span::Generated,
            type_: (),
        }
    }
    pub fn unit() -> Self {
        Self::dummy(TermKind::Immediate(ImmediateValue::Unit))
    }
    pub fn unreachable() -> Self {
        Self::dummy(TermKind::Unreachable)
    }
}

pub type UntypedTerm = Term<()>;

pub fn immediate(
    logger: &mut FileLogger,
    lit: ast::Literal,
) -> Option<ImmediateValue> {
    let token = lit.token()?;
    Some(match token.kind() {
        SyntaxKind::INTEGER => {
            let raw_text = token.text();
            let text = raw_text.replace('_', "");
            let (digits, radix) = if let Some(hex) =
                text.strip_prefix("0x").or_else(|| text.strip_prefix("0X"))
            {
                (hex, 16)
            } else if let Some(oct) = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O")) {
                (oct, 8)
            } else if let Some(bin) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
                (bin, 2)
            } else if let Some(dec) = text.strip_prefix("0d").or_else(|| text.strip_prefix("0D")) {
                (dec, 10)
            } else {
                (text.as_str(), 10)
            };
            match i64::from_str_radix(digits, radix) {
                Ok(i) => ImmediateValue::Integer(i),
                Err(_) => {
                    let span: Span = token.text_range().into();
                    logger
                        .error("Failed to parse integer literal.")
                        .primary(
                            format!(
                                "The literal `{raw_text}` is not a valid base {radix} integer."
                            ),
                            span,
                        )
                        .note("Integer literals must fit within a signed 64-bit value.")
                        .done();
                    return None;
                }
            }
        }
        SyntaxKind::REAL => {
            let raw_text = token.text();
            ImmediateValue::Real(match raw_text.replace('_', "").parse() {
                Ok(r) => r,
                Err(_) => {
                    let span: Span = token.text_range().into();
                    logger
                        .error("Failed to parse real literal.")
                        .primary(
                            format!("The literal `{raw_text}` is not a valid real number."),
                            span,
                        )
                        .note("Real literals must fit within an IEEE-754 64-bit float.")
                        .done();
                    return None;
                }
            })
        }
        SyntaxKind::STRING => {
            let text = token.text();
            // Strip surrounding quotes
            let inner = text.strip_prefix('"')?.strip_suffix('"')?;
            ImmediateValue::String(inner.to_string())
        }
        SyntaxKind::GLYPH => {
            let text = token.text();
            let inner = text.strip_prefix('\'')?;
            let inner = inner.strip_suffix('\'')?;
            let mut chars = inner.chars();
            let ch = chars.next()?;
            ImmediateValue::Glyph(ch)
        }
        SyntaxKind::TRUE_KW => ImmediateValue::Boolean(true),
        SyntaxKind::FALSE_KW => ImmediateValue::Boolean(false),
        _ => return None,
    })
}

fn binary_op_path(kind: SyntaxKind) -> Option<Path> {
    Some(match kind {
        SyntaxKind::STAR => BinaryOp::Star.path(),
        SyntaxKind::SLASH => BinaryOp::Slash.path(),
        SyntaxKind::PERCENT => BinaryOp::Percent.path(),
        SyntaxKind::PLUS => BinaryOp::Plus.path(),
        SyntaxKind::MINUS => BinaryOp::Minus.path(),
        SyntaxKind::COMPOSE_LEFT => BinaryOp::ComposeLeft.path(),
        SyntaxKind::COMPOSE_RIGHT => BinaryOp::ComposeRight.path(),
        SyntaxKind::XOR_KW => BinaryOp::Xor.path(),
        SyntaxKind::OR_KW => BinaryOp::Or.path(),
        SyntaxKind::PIPE_ARROW => BinaryOp::Apply.path(),
        SyntaxKind::DOUBLE_EQUAL => BinaryOp::DoubleEqual.path(),
        SyntaxKind::BANG_EQUAL => BinaryOp::BangEqual.path(),
        SyntaxKind::LESS => BinaryOp::Less.path(),
        SyntaxKind::LESS_EQUAL => BinaryOp::LessEqual.path(),
        SyntaxKind::GREATER => BinaryOp::Greater.path(),
        SyntaxKind::GREATER_EQUAL => BinaryOp::GreaterEqual.path(),
        SyntaxKind::AND_KW => BinaryOp::And.path(),
        SyntaxKind::SEMICOLON => BinaryOp::Semicolon.path(),
        _ => return None,
    })
}

fn unary_op_path(kind: SyntaxKind) -> Option<Path> {
    Some(match kind {
        SyntaxKind::MINUS => UnaryOp::Minus.path(),
        SyntaxKind::NOT_KW => UnaryOp::Not.path(),
        _ => return None,
    })
}

fn mk(
    kind: TermKind<()>,
    span: Span,
) -> UntypedTerm {
    Term {
        comments: String::new(),
        kind,
        span,
        type_: (),
    }
}

fn curry(
    scope: &mut impl Scope,
    wasm_type_defs: &IndexMap<String, WasmType>,
    logger: &mut FileLogger,
    mut params: impl Iterator<Item = ast::Param>,
    body: ast::Expr,
    span: Span,
) -> Option<UntypedTerm> {
    match params.next() {
        Some(param) => {
            let param_name = param.name_text_spanned()?;
            let param_span = param_name.span;
            let parameter_type = match param.ty() {
                Some(type_expr_node) => Some(type_expr(scope, type_expr_node)?),
                None => None,
            };
            let mut inner_scope = scope.nest_function_scope();
            let path = inner_scope.define(param_name, NameSpace::Term);
            let body = curry(&mut inner_scope, wasm_type_defs, logger, params, body, span)?;
            Some(mk(
                TermKind::Function {
                    parameter_name: path.with_span(param_span),
                    parameter_type,
                    captures: inner_scope
                        .into_captures()
                        .into_iter()
                        .map(|c| (c, ()))
                        .collect(),
                    body: body.into(),
                },
                span,
            ))
        }
        None => term(scope, wasm_type_defs, logger, body),
    }
}

fn array_term(
    scope: &mut impl Scope,
    wasm_type_defs: &IndexMap<String, WasmType>,
    logger: &mut FileLogger,
    array_expr: ast::ArrayExpr,
) -> Option<UntypedTerm> {
    let span = array_expr.span();
    let empty = Path::core("array_empty");
    let mut current = mk(TermKind::Identifier(empty), span);

    for child in array_expr.syntax().children() {
        if let Some(splat) = ast::ArraySplat::cast(child.clone()) {
            let concat_path = Path::core("array_concat");
            let elem = term(scope, wasm_type_defs, logger, splat.expr()?)?;
            let elem_span = elem.span;
            current = mk(
                TermKind::Call {
                    callee: mk(
                        TermKind::Call {
                            callee: mk(TermKind::Identifier(concat_path), elem_span).into(),
                            argument: elem.into(),
                        },
                        elem_span,
                    )
                    .into(),
                    argument: current.into(),
                },
                elem_span,
            );
        } else if let Some(expr) = ast::Expr::cast(child) {
            let push_path = Path::core("array_push");
            let elem = term(scope, wasm_type_defs, logger, expr)?;
            let elem_span = elem.span;
            current = mk(
                TermKind::Call {
                    callee: mk(
                        TermKind::Call {
                            callee: mk(TermKind::Identifier(push_path), elem_span).into(),
                            argument: elem.into(),
                        },
                        elem_span,
                    )
                    .into(),
                    argument: current.into(),
                },
                elem_span,
            );
        }
    }
    Some(current)
}

pub fn term(
    scope: &mut impl Scope,
    wasm_type_defs: &IndexMap<String, WasmType>,
    logger: &mut FileLogger,
    expr: ast::Expr,
) -> Option<UntypedTerm> {
    let span = expr.span();
    Some(match expr {
        ast::Expr::Let(let_expr) => {
            let value = term(scope, wasm_type_defs, logger, let_expr.value()?)?;
            let mut inner_scope = scope.nest_scope();
            let pat = pattern(&mut inner_scope, logger, let_expr.pattern()?)?;
            let body = term(&mut inner_scope, wasm_type_defs, logger, let_expr.body()?)?;
            mk(
                TermKind::Let {
                    assignee: pat,
                    scope: ScopeKind::Local,
                    value: value.into(),
                    then: body.into(),
                    else_: mk(TermKind::Unreachable, span).into(),
                },
                span,
            )
        }
        ast::Expr::Use(use_expr) => {
            scope.push_use_scope();
            let lowered = (|| {
                scope.register_use(
                    use_expr.target()?,
                    use_expr.alias_name_spanned(),
                    use_expr.span(),
                )?;
                term(scope, wasm_type_defs, logger, use_expr.body()?)
            })();
            scope.pop_use_scope();
            lowered?
        }
        ast::Expr::Fn(fn_expr) => {
            let params = fn_expr.params();
            let body = fn_expr.body()?;
            if params.is_empty() {
                // Unit function: `fn () => body` => curry with a synthetic parameter
                let mut inner_scope = scope.nest_function_scope();
                let param_name = "<parameter>".to_string().with_span(Span::Generated);
                let path = inner_scope.define(param_name, NameSpace::Term);
                let body = term(&mut inner_scope, wasm_type_defs, logger, body)?;
                mk(
                    TermKind::Function {
                        parameter_name: path.with_span(Span::Generated),
                        parameter_type: None,
                        captures: inner_scope
                            .into_captures()
                            .into_iter()
                            .map(|c| (c, ()))
                            .collect(),
                        body: body.into(),
                    },
                    span,
                )
            } else {
                curry(
                    scope,
                    wasm_type_defs,
                    logger,
                    params.into_iter(),
                    body,
                    span,
                )?
            }
        }
        ast::Expr::FnShorthand(fn_shorthand_expr) => {
            // Desugar to: fn <parameter> => match <parameter> with arms
            let mut inner_scope = scope.nest_function_scope();
            let param_name = "<parameter>".to_string().with_span(Span::Generated);
            let path = inner_scope.define(param_name, NameSpace::Term);

            // Build the match chain
            let arms = fn_shorthand_expr.arms();
            let mut current: Box<UntypedTerm> = mk(TermKind::Unreachable, span).into();
            for arm in arms.into_iter().rev() {
                let mut arm_scope = inner_scope.nest_scope();
                let pat = pattern(&mut arm_scope, logger, arm.pattern()?)?;
                let body = term(&mut arm_scope, wasm_type_defs, logger, arm.body()?)?;
                let arm_span = pat.span;
                current = mk(
                    TermKind::Let {
                        assignee: pat,
                        scope: ScopeKind::Local,
                        value: mk(TermKind::Identifier(path.clone()), span).into(),
                        then: body.into(),
                        else_: current,
                    },
                    arm_span,
                )
                .into();
            }
            mk(
                TermKind::Function {
                    parameter_name: path.with_span(Span::Generated),
                    parameter_type: None,
                    captures: inner_scope
                        .into_captures()
                        .into_iter()
                        .map(|c| (c, ()))
                        .collect(),
                    body: current,
                },
                span,
            )
        }
        ast::Expr::If(if_expr) => {
            let condition = term(scope, wasm_type_defs, logger, if_expr.condition()?)?;
            let then_branch = term(scope, wasm_type_defs, logger, if_expr.then_branch()?)?;
            let else_branch = term(scope, wasm_type_defs, logger, if_expr.else_branch()?)?;
            mk(
                TermKind::Let {
                    assignee: Pattern {
                        comments: String::new(),
                        kind: PatternKind::Immediate(ImmediateValue::Boolean(true)),
                        span,
                        type_: (),
                    },
                    scope: ScopeKind::Local,
                    value: condition.into(),
                    then: then_branch.into(),
                    else_: else_branch.into(),
                },
                span,
            )
        }
        ast::Expr::Match(match_expr) => {
            let scrutinee = term(scope, wasm_type_defs, logger, match_expr.scrutinee()?)?;
            let scrutinee_span = scrutinee.span;
            let mut outer_scope = scope.nest_scope();
            let scrutinee_name = "<scrutinee>".to_string().with_span(scrutinee_span);
            let scrutinee_path = outer_scope.define(scrutinee_name, NameSpace::Term);

            let arms = match_expr.arms();
            let mut current: Box<UntypedTerm> = mk(TermKind::Unreachable, span).into();
            for arm in arms.into_iter().rev() {
                let mut arm_scope = outer_scope.nest_scope();
                let pat = pattern(&mut arm_scope, logger, arm.pattern()?)?;
                let body = term(&mut arm_scope, wasm_type_defs, logger, arm.body()?)?;
                let arm_span = pat.span;
                current = mk(
                    TermKind::Let {
                        assignee: pat,
                        scope: ScopeKind::Local,
                        value: mk(TermKind::Identifier(scrutinee_path.clone()), arm_span).into(),
                        then: body.into(),
                        else_: current,
                    },
                    arm_span,
                )
                .into();
            }
            mk(
                TermKind::Let {
                    assignee: Pattern {
                        comments: String::new(),
                        kind: PatternKind::Identifier(scrutinee_path),
                        span: scrutinee_span,
                        type_: (),
                    },
                    scope: ScopeKind::Local,
                    value: scrutinee.into(),
                    then: current,
                    else_: mk(TermKind::Unreachable, span).into(),
                },
                span,
            )
        }
        ast::Expr::InlineWasm(inline_wasm_expr) => {
            let asserted_type = type_expr(scope, inline_wasm_expr.asserted_type()?)?;
            let inline_wasm = {
                let mut inline_scope = scope.nest_scope();
                wasm::build_inline_expression(
                    &inline_wasm_expr.instructions()?,
                    wasm_type_defs,
                    logger,
                    &mut inline_scope,
                )
            };
            mk(
                TermKind::InlineWasm {
                    asserted_type,
                    definitions: inline_wasm.definitions,
                    instructions: inline_wasm.instructions,
                },
                span,
            )
        }
        ast::Expr::Binary(binary_expr) => {
            let op_token = binary_expr.op_token()?;
            let op_kind = op_token.kind();
            if op_kind == SyntaxKind::SEMICOLON {
                let lhs = term(scope, wasm_type_defs, logger, binary_expr.lhs()?)?;
                let rhs = term(scope, wasm_type_defs, logger, binary_expr.rhs()?)?;
                mk(TermKind::Semicolon(lhs.into(), rhs.into()), span)
            } else {
                let op_path = binary_op_path(op_kind)?;
                let op_span: Span = op_token.text_range().into();
                let lhs = term(scope, wasm_type_defs, logger, binary_expr.lhs()?)?;
                let rhs = term(scope, wasm_type_defs, logger, binary_expr.rhs()?)?;
                mk(
                    TermKind::Call {
                        callee: mk(
                            TermKind::Call {
                                callee: mk(TermKind::Identifier(op_path), op_span).into(),
                                argument: lhs.into(),
                            },
                            span,
                        )
                        .into(),
                        argument: rhs.into(),
                    },
                    span,
                )
            }
        }
        ast::Expr::Unary(unary_expr) => {
            let op_token = unary_expr.op_token()?;
            let op_path = unary_op_path(op_token.kind())?;
            let op_span: Span = op_token.text_range().into();
            let operand = term(scope, wasm_type_defs, logger, unary_expr.operand()?)?;
            mk(
                TermKind::Call {
                    callee: mk(TermKind::Identifier(op_path), op_span).into(),
                    argument: operand.into(),
                },
                span,
            )
        }
        ast::Expr::Call(call_expr) => {
            let callee = term(scope, wasm_type_defs, logger, call_expr.callee()?)?;
            let argument = term(scope, wasm_type_defs, logger, call_expr.arg()?)?;
            mk(
                TermKind::Call {
                    callee: callee.into(),
                    argument: argument.into(),
                },
                span,
            )
        }
        ast::Expr::Field(field_expr) => {
            let base = term(scope, wasm_type_defs, logger, field_expr.base()?)?;
            let field_name = field_expr.field_name_spanned()?;
            mk(
                TermKind::Field {
                    of: base.into(),
                    index: field_name,
                },
                span,
            )
        }
        ast::Expr::Unit(_) => mk(TermKind::Immediate(ImmediateValue::Unit), span),
        ast::Expr::Paren(paren_expr) => {
            if paren_expr.is_tuple() {
                let items = paren_expr
                    .inner_exprs()
                    .into_iter()
                    .map(|e| term(scope, wasm_type_defs, logger, e))
                    .collect::<Option<Vec<_>>>()?;
                mk(TermKind::Tuple(items), span)
            } else {
                // Grouping: single inner expression
                let inner = paren_expr.inner_exprs().into_iter().next()?;
                return term(scope, wasm_type_defs, logger, inner);
            }
        }
        ast::Expr::Array(array_expr) => {
            return array_term(scope, wasm_type_defs, logger, array_expr);
        }
        ast::Expr::Struct(struct_expr) => {
            let mut fields = IndexMap::new();
            for field in struct_expr.fields() {
                let name = field.name_text_spanned()?;
                let value = term(scope, wasm_type_defs, logger, field.value()?)?;
                fields.insert(name, value);
            }
            mk(TermKind::Struct(fields), span)
        }
        ast::Expr::Literal(literal) => mk(TermKind::Immediate(immediate(logger, literal)?), span),
        ast::Expr::Ident(ident_expr) => {
            let name = ident_expr.name_text_spanned()?;
            let path = scope.query_string(name, NameSpace::Term);
            mk(TermKind::Identifier(path), span)
        }
        ast::Expr::Path(path_expr) => {
            let path = scope.resolve_path(&path_expr, NameSpace::Term, span)?;
            scope.query_path(path.clone().with_span(span), NameSpace::Term);
            mk(TermKind::Identifier(path), span)
        }
    })
}
