/*!
    Parser for Halcyon's inline WASM S-expression syntax.

    This is intentionally WAT-like, but not WAT-compatible:

    - Declaration forms are restricted to `(type ...)`, `(global ...)`,
      `(func ...)`, `(memory ...)`, and `(import ...)`.
    - Memory declarations use 32-bit page counts; an optional maximum must not
      be smaller than the initial size.
    - Function/inline-expression bodies are *flat* instruction streams. Nested
      instruction lists (WAT style) are rejected.
    - Names are symbolic and resolved through IR scopes. `$name` resolves in the
      wasm namespace; bare identifiers resolve in the term namespace.
    - Types use the simplified GC-oriented `asm::Type` model, with support for
      primitives (`any`, `i8`, `i16`, `i32`, `i64`, `f32`, `f64`), `(struct
      ...)`, `(array ...)`, `(func (param ...) (result ...))`, and user-defined
      wasm type aliases.

    For inline wasm expressions (`(wasm : T) => (...)`), only `(local ...)`
    declarations are allowed before instructions.
*/

use indexmap::IndexMap;

use crate::asm::{
    Instruction,
    NumberOperation,
    Type,
};
use crate::ir::{
    NameSpace,
    Scope,
};
use crate::parse::ast::{
    AstNode,
    Sexpr,
    SexprAtom,
    SexprItem,
};
use crate::{
    FileLogger,
    Span,
    Spanned,
    WithContext,
    WithSpan,
};

use super::{
    ImmediateValue,
    Path,
};

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub span: Span,
    pub parameters: IndexMap<Path, Type>,
    pub results: Box<[Type]>,
    pub locals: IndexMap<Path, Type>,
    pub body: Box<[Instruction]>,
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub name: String,
    pub initial_size: u32,
    pub maximum_size: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub name: String,
    pub type_: Type,
}

#[derive(Debug, Clone)]
struct FunctionType {
    parameters: Box<[Type]>,
    results: Box<[Type]>,
}

#[derive(Debug, Clone)]
pub struct Global {
    pub name: Path,
    pub type_: Type,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub wasm_module: String,
    pub wasm_name: String,
    pub local_name: Path,
    pub params: Box<[Type]>,
    pub results: Box<[Type]>,
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Type(TypeDefinition),
    Global(Global),
    Function(Function),
    Memory(Memory),
    Import(Import),
}

#[derive(Debug, Clone)]
pub struct InlineExpression {
    pub definitions: IndexMap<Path, Type>,
    pub instructions: Box<[Instruction]>,
}

struct TypeEnv<'a> {
    type_defs: &'a IndexMap<String, Type>,
    defining: Option<&'a str>,
    module_name: &'a str,
}

#[derive(Debug, Clone)]
struct FunctionSections {
    parameters: IndexMap<Path, Type>,
    results: Vec<Type>,
    locals: IndexMap<Path, Type>,
    body_items: Vec<SexprItem>,
}

impl FunctionSections {
    fn new() -> Self {
        Self {
            parameters: IndexMap::new(),
            results: Vec::new(),
            locals: IndexMap::new(),
            body_items: Vec::new(),
        }
    }
}

struct Cursor<'a> {
    items: &'a [SexprItem],
    index: usize,
}

impl<'a> Cursor<'a> {
    fn new(items: &'a [SexprItem]) -> Self {
        Self { items, index: 0 }
    }
    fn is_done(&self) -> bool {
        self.index >= self.items.len()
    }
    fn peek(&self) -> Option<&'a SexprItem> {
        self.items.get(self.index)
    }
    fn next(&mut self) -> Option<&'a SexprItem> {
        let item = self.items.get(self.index);
        if item.is_some() {
            self.index += 1;
        }
        item
    }
}

pub fn build_toplevel(
    sexpr: &Sexpr,
    module_name: &str,
    type_defs: &mut IndexMap<String, Type>,
    logger: &mut FileLogger,
    scope: &mut impl Scope,
) -> Box<[Declaration]> {
    let mut declarations = Vec::new();
    for list in collect_declaration_lists(sexpr, logger) {
        let env = TypeEnv {
            type_defs,
            defining: None,
            module_name,
        };
        let Some(declaration) = parse_declaration(&list, &env, logger, scope) else {
            continue;
        };
        if let Declaration::Type(type_def) = &declaration {
            type_defs.insert(type_def.name.clone(), type_def.type_.clone());
        }
        declarations.push(declaration);
    }
    declarations.into_boxed_slice()
}

pub fn build_inline_expression(
    sexpr: &Sexpr,
    type_defs: &IndexMap<String, Type>,
    logger: &mut FileLogger,
    scope: &mut impl Scope,
) -> InlineExpression {
    let env = TypeEnv {
        type_defs,
        defining: None,
        module_name: "",
    };
    let items = sexpr.items();
    let mut cursor = Cursor::new(&items);
    let sections = parse_inline_expression_sections(&mut cursor, &env, logger, scope);
    let instructions = parse_function_body(&sections.body_items, &env, logger, scope);
    InlineExpression {
        definitions: sections.locals,
        instructions: instructions.into_boxed_slice(),
    }
}

fn parse_inline_expression_sections(
    cursor: &mut Cursor<'_>,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
    scope: &mut impl Scope,
) -> FunctionSections {
    let mut sections = FunctionSections::new();
    while let Some(item) = cursor.next() {
        match item {
            SexprItem::List(list) => {
                let Some(keyword) = declaration_keyword(list) else {
                    logger
                        .error("Unsupported list in inline wasm expression")
                        .primary("Only `(local ...)` is allowed here.", list.span())
                        .done();
                    continue;
                };
                match keyword.as_str() {
                    "local" => {
                        for (name, ty) in parse_named_types_list("local", list, env, logger) {
                            let name = scope.define(name, NameSpace::Wasm);
                            sections.locals.insert(name, ty);
                        }
                    }
                    _ => {
                        logger
                            .error("Unsupported list in inline wasm expression")
                            .primary(
                                format!("`({keyword} ...)` is not supported here."),
                                list.span(),
                            )
                            .note("Only `(local ...)` is allowed here.")
                            .done();
                    }
                }
            }
            other => sections.body_items.push(other.clone()),
        }
    }
    sections
}

fn collect_declaration_lists(
    sexpr: &Sexpr,
    logger: &mut FileLogger,
) -> Vec<Sexpr> {
    let items = sexpr.items();
    if is_declaration_list(&items) {
        return vec![sexpr.clone()];
    }
    items
        .into_iter()
        .filter_map(|item| {
            match item {
                SexprItem::List(list) => Some(list),
                other => {
                    logger
                        .error("Expected a wasm declaration list")
                        .primary(
                            "Only `(type ...)`, `(global ...)`, `(func ...)`, `(memory ...)`, and `(import ...)` are supported here.",
                            other.span(),
                        )
                        .done();
                    None
                }
            }
        })
        .collect()
}

fn is_declaration_list(items: &[SexprItem]) -> bool {
    matches!(
        items.first().and_then(item_keyword),
        Some(ref keyword)
            if keyword == "type"
                || keyword == "global"
                || keyword == "func"
                || keyword == "memory"
                || keyword == "import"
    )
}

fn parse_declaration(
    list: &Sexpr,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
    scope: &mut impl Scope,
) -> Option<Declaration> {
    let Some(keyword) = declaration_keyword(list) else {
        logger
            .error("Expected a wasm declaration")
            .primary(
                "Declarations must start with `type`, `global`, `func`, `memory`, or `import`.",
                list.span(),
            )
            .done();
        return None;
    };
    match keyword.as_str() {
        "type" => parse_type_definition(list, env, logger).map(Declaration::Type),
        "global" => parse_global_definition(list, env, logger, scope).map(Declaration::Global),
        "func" => parse_function(list, env, logger, scope).map(Declaration::Function),
        "memory" => parse_memory(list, logger).map(Declaration::Memory),
        "import" => parse_import(list, env, logger, scope).map(Declaration::Import),
        _ => {
            logger
                .error("Unsupported wasm declaration")
                .primary(format!("`{keyword}` is not supported here."), list.span())
                .note("Only `(type ...)`, `(global ...)`, `(func ...)`, `(memory ...)`, and `(import ...)` are supported.")
                .done();
            None
        }
    }
}

fn declaration_keyword(list: &Sexpr) -> Option<String> {
    list.items().first().and_then(item_keyword)
}

fn parse_type_definition(
    list: &Sexpr,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<TypeDefinition> {
    let items = list.items();
    let mut cursor = Cursor::new(&items);
    cursor.next();
    let name = parse_type_definition_name(&mut cursor, list.span(), logger)?;
    let Some(type_item) = cursor.next() else {
        log_expected(logger, list.span(), "type expression");
        return None;
    };
    let type_env = TypeEnv {
        type_defs: env.type_defs,
        defining: Some(&name),
        module_name: env.module_name,
    };
    let type_ = parse_type(type_item, &type_env, logger)?;
    if let Some(extra) = cursor.next() {
        log_invalid(
            logger,
            extra.span(),
            "Type definitions take a name and a single type expression.",
        );
    }
    Some(TypeDefinition { name, type_ })
}

fn parse_global_definition(
    list: &Sexpr,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
    scope: &mut impl Scope,
) -> Option<Global> {
    let items = list.items();
    let mut cursor = Cursor::new(&items);
    cursor.next();
    let name = scope.define(
        parse_global_name(&mut cursor, list.span(), logger)?,
        NameSpace::Wasm,
    );
    let Some(type_item) = cursor.next() else {
        log_expected(logger, list.span(), "global type");
        return None;
    };
    let type_ = parse_type(type_item, env, logger)?;
    if let Some(extra) = cursor.next() {
        log_invalid(
            logger,
            extra.span(),
            "Global definitions take a name and a single type expression.",
        );
    }
    Some(Global { name, type_ })
}

fn parse_function(
    list: &Sexpr,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
    scope: &mut impl Scope,
) -> Option<Function> {
    let items = list.items();
    let mut cursor = Cursor::new(&items);
    cursor.next();
    let name = parse_function_name(&mut cursor, list.span(), logger)?;
    let name = scope.define(name, NameSpace::Wasm);
    let mut function_scope = scope.nest_function_scope();
    let sections = parse_function_sections(&mut cursor, env, logger, &mut function_scope);
    let body = parse_function_body(&sections.body_items, env, logger, &mut function_scope);
    Some(Function {
        name: name.minor,
        span: list.span(),
        parameters: sections.parameters,
        results: sections.results.into_boxed_slice(),
        locals: sections.locals,
        body: body.into_boxed_slice(),
    })
}

fn parse_function_name(
    cursor: &mut Cursor<'_>,
    fallback_span: Span,
    logger: &mut FileLogger,
) -> Option<Spanned<String>> {
    let Some(item) = cursor.next() else {
        logger
            .error("Missing function name")
            .primary("Functions require a name after `func`.", fallback_span)
            .done();
        return None;
    };
    let Some(name) = item_symbol_ident(item) else {
        log_invalid(
            logger,
            item.span(),
            "Function names must be `$`-prefixed identifiers.",
        );
        return None;
    };
    Some(name.with_span(item.span()))
}

fn parse_global_name(
    cursor: &mut Cursor<'_>,
    fallback_span: Span,
    logger: &mut FileLogger,
) -> Option<Spanned<String>> {
    let Some(item) = cursor.next() else {
        log_expected(logger, fallback_span, "global name");
        return None;
    };
    let Some(name) = item_symbol_ident(item) else {
        log_invalid(
            logger,
            item.span(),
            "Global names must be `$`-prefixed identifiers.",
        );
        return None;
    };
    Some(name.with_span(item.span()))
}

fn parse_type_definition_name(
    cursor: &mut Cursor<'_>,
    fallback_span: Span,
    logger: &mut FileLogger,
) -> Option<String> {
    let Some(item) = cursor.next() else {
        logger
            .error("Missing type name")
            .primary(
                "Type definitions require a name after `type`.",
                fallback_span,
            )
            .done();
        return None;
    };
    item_symbol_ident(item).or_else(|| {
        logger
            .error("Invalid value")
            .primary("Type names must be `$`-prefixed identifiers.", item.span())
            .done();
        None
    })
}

fn parse_function_sections(
    cursor: &mut Cursor<'_>,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
    scope: &mut impl Scope,
) -> FunctionSections {
    let mut sections = FunctionSections::new();
    while let Some(item) = cursor.next() {
        match item {
            SexprItem::List(list) => {
                let Some(keyword) = declaration_keyword(list) else {
                    logger
                        .error("Unsupported list in function body")
                        .primary(
                            "Only `(param ...)`, `(result ...)`, and `(local ...)` are allowed here.",
                            list.span(),
                        )
                        .done();
                    continue;
                };
                match keyword.as_str() {
                    "param" => {
                        for (name, ty) in parse_named_types_list("parameter", list, env, logger) {
                            let name = scope.define(name, NameSpace::Wasm);
                            sections.parameters.insert(name, ty);
                        }
                    }
                    "result" => {
                        sections
                            .results
                            .extend(parse_result_types_list(list, env, logger));
                    }
                    "local" => {
                        for (name, ty) in parse_named_types_list("local", list, env, logger) {
                            let name = scope.define(name, NameSpace::Wasm);
                            sections.locals.insert(name, ty);
                        }
                    }
                    _ => {
                        logger
                            .error("Unsupported list in function body")
                            .primary(
                                format!("`({keyword} ...)` is not supported here."),
                                list.span(),
                            )
                            .note("Only `(param ...)`, `(result ...)`, and `(local ...)` are supported.")
                            .done();
                    }
                }
            }
            other => sections.body_items.push(other.clone()),
        }
    }
    sections
}

fn parse_named_types_list(
    kind: &str,
    list: &Sexpr,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Vec<(Spanned<String>, Type)> {
    let items = list.items();
    let mut cursor = Cursor::new(&items);
    cursor.next();
    let mut out = Vec::new();
    while !cursor.is_done() {
        let fallback_span = cursor
            .peek()
            .map(SexprItem::span)
            .unwrap_or_else(|| list.span());
        let expected = format!("{kind} name");
        let Some((name, span)) = expect_symbol_name(&mut cursor, &expected, fallback_span, logger)
        else {
            continue;
        };
        let name = name.with_span(span);
        let type_span = cursor
            .peek()
            .map(SexprItem::span)
            .unwrap_or_else(|| list.span());
        let Some(type_item) = cursor.next() else {
            logger
                .error(format!("Missing {kind} type"))
                .primary(format!("`{name}` is missing a type annotation."), type_span)
                .done();
            continue;
        };
        let Some(ty) = parse_type(type_item, env, logger) else {
            continue;
        };
        out.push((name, ty));
    }
    out
}

fn parse_result_types_list(
    list: &Sexpr,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Vec<Type> {
    let items = list.items();
    let mut cursor = Cursor::new(&items);
    cursor.next();
    let mut out = Vec::new();
    while !cursor.is_done() {
        let Some(item) = cursor.next() else {
            break;
        };
        if let Some(ty) = parse_type(item, env, logger) {
            out.push(ty);
        }
    }
    out
}

fn parse_function_body(
    items: &[SexprItem],
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
    scope: &mut impl Scope,
) -> Vec<Instruction> {
    let mut cursor = Cursor::new(items);
    let mut out = Vec::new();
    while let Some(op_item) = cursor.next() {
        if let SexprItem::List(list) = op_item {
            logger
                .error("Unsupported instruction form")
                .primary("Only flat instruction forms are supported.", list.span())
                .done();
            continue;
        }
        let op_span = op_item.span();
        let Some(op) = item_keyword(op_item) else {
            logger
                .error("Expected an instruction")
                .primary("Instruction keywords must be identifiers.", op_span)
                .done();
            continue;
        };
        if let Some(instr) = parse_instruction(&op, &mut cursor, op_span, env, logger, scope) {
            out.push(instr)
        }
    }
    out
}

fn parse_instruction(
    op: &str,
    cursor: &mut Cursor<'_>,
    op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
    scope: &mut impl Scope,
) -> Option<Instruction> {
    match op {
        "set" => {
            parse_path_argument(cursor, op_span, "set target", logger, scope).map(Instruction::Set)
        }
        "get" => {
            parse_path_argument(cursor, op_span, "get target", logger, scope).map(Instruction::Get)
        }
        "const" => parse_const(cursor, op_span, logger),
        "i32.const" => parse_i32_const(cursor, op_span, logger),
        "i64.const" => parse_i64_const(cursor, op_span, logger),
        "f32.const" => parse_f32_const(cursor, op_span, logger),
        "f64.const" => parse_f64_const(cursor, op_span, logger),
        "string.const" => parse_string_const(cursor, op_span, logger),
        "glyph.const" => parse_glyph_const(cursor, op_span, logger),
        "func" => {
            parse_path_argument(cursor, op_span, "function name", logger, scope)
                .map(Instruction::Func)
        }
        "struct.new" => parse_struct_new(cursor, op_span, env, logger),
        "struct.get" => parse_struct_get(cursor, op_span, env, logger),
        "array.get" => parse_array_get(cursor, op_span, env, logger),
        "array.new_fixed" => parse_array_new_fixed(cursor, op_span, env, logger),
        "array.new_default" => parse_array_new_default(cursor, op_span, env, logger),
        "array.len" => Some(Instruction::ArrayLen),
        "array.copy" => parse_array_copy(cursor, op_span, env, logger),
        "call.ref" => parse_call_ref(cursor, op_span, env, logger),
        "call" => {
            parse_path_argument(cursor, op_span, "function name", logger, scope)
                .map(Instruction::Call)
        }
        "unreachable" => Some(Instruction::Unreachable),
        "drop" => Some(Instruction::Drop),
        "if" => parse_if(cursor, op_span, env, logger),
        "else" => Some(Instruction::Else),
        "end" => Some(Instruction::End),
        "loop" => Some(Instruction::Loop),
        "block" => parse_block(cursor, op_span, env, logger),
        "break" => parse_break(cursor, op_span, logger),
        "break.if" => parse_break_if(cursor, op_span, logger),
        "ref.cast_func" => parse_ref_cast_func(cursor, op_span, env, logger),
        "ref.cast_struct" => parse_ref_cast_struct(cursor, op_span, env, logger),
        "ref.cast_array" => parse_ref_cast_array(cursor, op_span, env, logger),
        "i32.store8" => Some(Instruction::I32Store8),
        "i32.load" => Some(Instruction::I32Load),
        "i32.store" => Some(Instruction::I32Store),
        "i64.load" => Some(Instruction::I64Load),
        "i64.extend_i32_u" => Some(Instruction::I64ExtendI32U),
        "i32.wrap_i64" => Some(Instruction::I32WrapI64),
        "i32.trunc_f32_s" => Some(Instruction::I32TruncF32S),
        "i32.trunc_f32_u" => Some(Instruction::I32TruncF32U),
        "i32.trunc_f64_s" => Some(Instruction::I32TruncF64S),
        "i32.trunc_f64_u" => Some(Instruction::I32TruncF64U),
        "i64.trunc_f32_s" => Some(Instruction::I64TruncF32S),
        "i64.trunc_f32_u" => Some(Instruction::I64TruncF32U),
        "i64.trunc_f64_s" => Some(Instruction::I64TruncF64S),
        "i64.trunc_f64_u" => Some(Instruction::I64TruncF64U),
        "f32.demote_f64" => Some(Instruction::F32DemoteF64),
        _ => {
            parse_number_op(op).map_or_else(
                || {
                    logger
                        .error("Unsupported instruction")
                        .primary(format!("`{op}` is not supported."), op_span)
                        .done();
                    None
                },
                Some,
            )
        }
    }
}

fn parse_const(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    parse_immediate_argument(cursor, op_span, logger).map(Instruction::Const)
}

fn parse_i32_const(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let (value, _) = parse_signed_integer_argument(cursor, op_span, "integer literal", logger)?;
    match i32::try_from(value) {
        Ok(value) => Some(Instruction::I32Const(value)),
        Err(_) => {
            log_invalid(logger, op_span, "The literal does not fit in an i32.");
            None
        }
    }
}

fn parse_i64_const(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let (value, _) = parse_signed_integer_argument(cursor, op_span, "integer literal", logger)?;
    Some(Instruction::Const(ImmediateValue::Integer(value)))
}

fn parse_f32_const(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let value = parse_signed_float_argument(cursor, op_span, logger)?;
    Some(Instruction::F32Const(value as f32))
}

fn parse_f64_const(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let value = parse_signed_float_argument(cursor, op_span, logger)?;
    Some(Instruction::Const(ImmediateValue::Real(value)))
}

fn parse_string_const(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let Some(item) = cursor.next() else {
        log_expected(logger, op_span, "string literal");
        return None;
    };
    let value = parse_string_literal_from_item(item, logger)?;
    Some(Instruction::Const(ImmediateValue::String(value)))
}

fn parse_glyph_const(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let Some(item) = cursor.next() else {
        log_expected(logger, op_span, "single-character string literal");
        return None;
    };
    let value = parse_glyph_literal_from_item(item, logger)?;
    Some(Instruction::Const(ImmediateValue::Glyph(value)))
}

fn parse_struct_new(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let fields = parse_struct_type_argument(cursor, op_span, env, logger)?;
    Some(Instruction::StructNew(fields))
}

fn parse_struct_get(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let fields = parse_struct_type_argument(cursor, op_span, env, logger)?;
    let index = parse_usize_argument(cursor, op_span, "struct field index", logger)?;
    Some(Instruction::StructGet(fields, index))
}

fn parse_array_get(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let Some(item) = cursor.next() else {
        log_expected(logger, op_span, "array element type");
        return None;
    };
    let element_type = parse_array_element_type(item, env, logger)?;
    Some(Instruction::ArrayGet(element_type))
}

fn parse_array_new_fixed(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let Some(type_item) = cursor.next() else {
        log_expected(logger, op_span, "array element type");
        return None;
    };
    let element_type = parse_array_element_type(type_item, env, logger)?;
    let length = parse_usize_argument(cursor, op_span, "array length", logger)?;
    Some(Instruction::ArrayNewFixed {
        inner_type: element_type,
        length,
    })
}

fn parse_array_new_default(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let Some(item) = cursor.next() else {
        log_expected(logger, op_span, "array element type");
        return None;
    };
    let element_type = parse_array_element_type(item, env, logger)?;
    Some(Instruction::ArrayNewDefault(element_type))
}

fn parse_array_copy(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let Some(dst_item) = cursor.next() else {
        log_expected(logger, op_span, "destination element type");
        return None;
    };
    let Some(src_item) = cursor.next() else {
        log_expected(logger, op_span, "source element type");
        return None;
    };
    let dst_type = parse_array_element_type(dst_item, env, logger)?;
    let src_type = parse_array_element_type(src_item, env, logger)?;
    Some(Instruction::ArrayCopy { dst_type, src_type })
}

fn parse_call_ref(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let function_type = parse_function_type_argument(cursor, op_span, env, logger)?;
    Some(Instruction::CallRef {
        parameters: function_type.parameters,
        returns: function_type.results,
    })
}

fn parse_if(
    cursor: &mut Cursor<'_>,
    _op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let type_ = parse_optional_type(cursor, env, logger)?;
    Some(Instruction::If(type_))
}

fn parse_block(
    cursor: &mut Cursor<'_>,
    _op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let type_ = parse_optional_type(cursor, env, logger)?;
    Some(Instruction::Block(type_))
}

fn parse_break(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    parse_usize_argument(cursor, op_span, "break depth", logger).map(Instruction::Break)
}

fn parse_break_if(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    parse_usize_argument(cursor, op_span, "break depth", logger).map(Instruction::BreakIf)
}

fn parse_ref_cast_func(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let function_type = parse_function_type_argument(cursor, op_span, env, logger)?;
    Some(Instruction::RefCastFunc {
        parameters: function_type.parameters,
        returns: function_type.results,
    })
}

fn parse_ref_cast_struct(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    parse_struct_type_argument(cursor, op_span, env, logger).map(Instruction::RefCastStruct)
}

fn parse_ref_cast_array(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Instruction> {
    let Some(item) = cursor.next() else {
        log_expected(logger, op_span, "array element type");
        return None;
    };
    let element_type = parse_array_element_type(item, env, logger)?;
    Some(Instruction::RefCastArray(element_type.into()))
}

fn parse_number_op(op: &str) -> Option<Instruction> {
    if let Some(suffix) = op.strip_prefix("i32.") {
        return parse_number_operation(suffix).map(Instruction::I32Op);
    }
    if let Some(suffix) = op.strip_prefix("i64.") {
        return parse_number_operation(suffix).map(Instruction::I64Op);
    }
    if let Some(suffix) = op.strip_prefix("f32.") {
        return parse_number_operation(suffix).map(Instruction::F32Op);
    }
    if let Some(suffix) = op.strip_prefix("f64.") {
        return parse_number_operation(suffix).map(Instruction::F64Op);
    }
    None
}

fn parse_number_operation(name: &str) -> Option<NumberOperation> {
    match name {
        "eq" => Some(NumberOperation::Eq),
        "ne" => Some(NumberOperation::Ne),
        "gt" => Some(NumberOperation::Gt),
        "lt" => Some(NumberOperation::Lt),
        "ge" => Some(NumberOperation::Ge),
        "le" => Some(NumberOperation::Le),
        "add" => Some(NumberOperation::Add),
        "sub" => Some(NumberOperation::Sub),
        "mul" => Some(NumberOperation::Mul),
        "div" => Some(NumberOperation::Div),
        "rem" => Some(NumberOperation::Rem),
        "and" => Some(NumberOperation::And),
        "or" => Some(NumberOperation::Or),
        "xor" => Some(NumberOperation::Xor),
        _ => None,
    }
}

fn parse_struct_type_argument(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Box<[Type]>> {
    let Some(item) = cursor.next() else {
        log_expected(logger, op_span, "struct type");
        return None;
    };
    let span = item.span();
    match parse_type(item, env, logger)? {
        Type::Struct(fields) => Some(fields),
        _ => {
            log_invalid(logger, span, "Expected a struct type.");
            None
        }
    }
}

fn parse_function_type_argument(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<FunctionType> {
    let Some(item) = cursor.next() else {
        log_expected(logger, op_span, "function type");
        return None;
    };
    let span = item.span();
    match parse_type(item, env, logger)? {
        Type::Function {
            parameters,
            results,
        } => {
            Some(FunctionType {
                parameters,
                results,
            })
        }
        _ => {
            log_invalid(logger, span, "Expected a function type.");
            None
        }
    }
}

fn parse_array_element_type(
    item: &SexprItem,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Type> {
    match parse_type(item, env, logger)? {
        Type::Array(inner) => Some(*inner),
        other => Some(other),
    }
}

fn parse_optional_type(
    cursor: &mut Cursor<'_>,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Option<Type>> {
    let Some(item) = cursor.peek() else {
        return Some(None);
    };
    if !is_type_item(item, env) {
        return Some(None);
    }
    let item = cursor.next().unwrap_or_else(|| unreachable!());
    parse_type(item, env, logger).map(Some)
}

fn parse_usize_argument(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    context: &str,
    logger: &mut FileLogger,
) -> Option<usize> {
    let (value, span) = parse_signed_integer_argument(cursor, op_span, context, logger)?;
    if value < 0 {
        log_invalid(logger, span, "Expected a non-negative integer.");
        return None;
    }
    usize::try_from(value).ok().or_else(|| {
        log_invalid(logger, span, "The value does not fit in a usize.");
        None
    })
}

fn parse_path_argument(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    context: &str,
    logger: &mut FileLogger,
    scope: &mut impl Scope,
) -> Option<Path> {
    let Some(item) = cursor.next() else {
        log_expected(logger, op_span, context);
        return None;
    };
    parse_path(item, logger, scope)
}

fn parse_path(
    item: &SexprItem,
    logger: &mut FileLogger,
    scope: &mut impl Scope,
) -> Option<Path> {
    match item {
        SexprItem::Path(path) => {
            let namespace = if path.has_dollar_prefix() {
                NameSpace::Wasm
            } else {
                NameSpace::Term
            };
            let segments = path.segments();
            let [lhs, rhs] = segments.as_slice() else {
                log_invalid(logger, path.span(), "Expected a two-part path.");
                return None;
            };
            let path = Path::new(lhs.clone(), rhs.clone()).with_span(path.span());
            Some(scope.query_path(path, namespace))
        }
        SexprItem::Atom(SexprAtom::SymbolIdent(token)) => {
            Some(scope.query_string(
                token.text().to_string().with_span(item.span()),
                NameSpace::Wasm,
            ))
        }
        SexprItem::Atom(SexprAtom::Ident(token)) => {
            Some(scope.query_string(
                token.text().to_string().with_span(item.span()),
                NameSpace::Term,
            ))
        }
        _ => {
            log_invalid(logger, item.span(), "Symbol must be an identifier or path");
            None
        }
    }
}

fn parse_import(
    list: &Sexpr,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
    scope: &mut impl Scope,
) -> Option<Import> {
    // (import "module" "name" (func $local (param ...) (result ...)))
    let items = list.items();
    let mut cursor = Cursor::new(&items);
    cursor.next(); // skip "import"

    let Some(wasm_module) = cursor
        .next()
        .and_then(|item| parse_string_literal_from_item(item, logger))
    else {
        log_expected(logger, list.span(), "import module string");
        return None;
    };

    let Some(wasm_name) = cursor
        .next()
        .and_then(|item| parse_string_literal_from_item(item, logger))
    else {
        log_expected(logger, list.span(), "import name string");
        return None;
    };

    let Some(SexprItem::List(func_list)) = cursor.next() else {
        log_expected(logger, list.span(), "function descriptor `(func ...)`");
        return None;
    };

    let func_items = func_list.items();
    let Some(kw) = func_items.first().and_then(item_keyword) else {
        log_expected(logger, func_list.span(), "`func` keyword");
        return None;
    };
    if kw != "func" {
        log_invalid(
            logger,
            func_list.span(),
            "Only function imports are supported.",
        );
        return None;
    }

    let mut func_cursor = Cursor::new(&func_items);
    func_cursor.next(); // skip "func"
    let local_name = parse_function_name(&mut func_cursor, func_list.span(), logger)?;
    let local_path = scope.define(local_name, NameSpace::Wasm);

    let mut params = Vec::new();
    let mut results = Vec::new();
    while let Some(SexprItem::List(inner_list)) = func_cursor.next() {
        let Some(keyword) = declaration_keyword(inner_list) else {
            log_invalid(
                logger,
                inner_list.span(),
                "Expected `(param ...)` or `(result ...)`.",
            );
            continue;
        };
        match keyword.as_str() {
            "param" => params.extend(parse_types_from_list(inner_list, env, logger)),
            "result" => results.extend(parse_types_from_list(inner_list, env, logger)),
            _ => {
                log_invalid(
                    logger,
                    inner_list.span(),
                    "Expected `(param ...)` or `(result ...)`.",
                );
            }
        }
    }

    Some(Import {
        wasm_module,
        wasm_name,
        local_name: local_path,
        params: params.into_boxed_slice(),
        results: results.into_boxed_slice(),
    })
}

fn parse_memory(
    list: &Sexpr,
    logger: &mut FileLogger,
) -> Option<Memory> {
    let items = list.items();
    let mut cursor = Cursor::new(&items);
    cursor.next();
    let name = parse_memory_name(&mut cursor, list.span(), logger)?;
    let (initial_size, maximum_size) = parse_memory_limits(&mut cursor, list.span(), logger)?;
    Some(Memory {
        name,
        initial_size,
        maximum_size,
    })
}

fn parse_memory_name(
    cursor: &mut Cursor<'_>,
    fallback_span: Span,
    logger: &mut FileLogger,
) -> Option<String> {
    let Some(item) = cursor.next() else {
        logger
            .error("Missing memory name")
            .primary("Memories require a name after `memory`.", fallback_span)
            .done();
        return None;
    };
    item_symbol_ident(item).or_else(|| {
        log_invalid(
            logger,
            item.span(),
            "Memory names must be `$`-prefixed identifiers.",
        );
        None
    })
}

fn parse_memory_limits(
    cursor: &mut Cursor<'_>,
    fallback_span: Span,
    logger: &mut FileLogger,
) -> Option<(u32, Option<u32>)> {
    let initial_span = cursor.peek().map(SexprItem::span).unwrap_or(fallback_span);
    let initial_size =
        parse_u32_immediate(cursor.next(), "initial memory size", initial_span, logger)?;
    let mut maximum_span = fallback_span;
    let maximum_size = if cursor.is_done() {
        None
    } else {
        maximum_span = cursor.peek().map(SexprItem::span).unwrap_or(fallback_span);
        Some(parse_u32_immediate(
            cursor.next(),
            "maximum memory size",
            maximum_span,
            logger,
        )?)
    };
    if let Some(maximum_size) = maximum_size
        && maximum_size < initial_size
    {
        logger
            .error("Invalid memory limits")
            .primary(
                "Maximum memory size cannot be smaller than the initial size.",
                maximum_span,
            )
            .done();
        return None;
    }
    if let Some(extra) = cursor.peek() {
        logger
            .error("Too many memory limits")
            .primary("Expected at most two memory size values.", extra.span())
            .done();
    }
    Some((initial_size, maximum_size))
}

fn parse_immediate_argument(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    logger: &mut FileLogger,
) -> Option<ImmediateValue> {
    let Some(item) = cursor.next() else {
        log_expected(logger, op_span, "immediate value");
        return None;
    };
    if is_minus_item(item) {
        let Some(next) = cursor.next() else {
            log_expected(logger, op_span, "immediate value");
            return None;
        };
        return match parse_immediate(next, logger)? {
            ImmediateValue::Integer(value) => {
                value
                    .checked_neg()
                    .map(ImmediateValue::Integer)
                    .or_else(|| {
                        log_invalid(logger, next.span(), "Integer literal is out of range.");
                        None
                    })
            }
            ImmediateValue::Real(value) => Some(ImmediateValue::Real(-value)),
            _ => {
                log_invalid(
                    logger,
                    next.span(),
                    "Only numeric immediates can be negative.",
                );
                None
            }
        };
    }
    parse_immediate(item, logger)
}

fn parse_signed_integer_argument(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    expected: &str,
    logger: &mut FileLogger,
) -> Option<(i64, Span)> {
    let Some(item) = cursor.next() else {
        log_expected(logger, op_span, expected);
        return None;
    };
    if is_minus_item(item) {
        let Some(next) = cursor.next() else {
            log_expected(logger, op_span, expected);
            return None;
        };
        let value = parse_integer_literal_from_item(next, logger)?;
        return value
            .checked_neg()
            .map(|value| (value, item.span()))
            .or_else(|| {
                log_invalid(logger, next.span(), "Integer literal is out of range.");
                None
            });
    }
    parse_integer_literal_from_item(item, logger).map(|value| (value, item.span()))
}

fn parse_signed_float_argument(
    cursor: &mut Cursor<'_>,
    op_span: Span,
    logger: &mut FileLogger,
) -> Option<f64> {
    let Some(item) = cursor.next() else {
        log_expected(logger, op_span, "numeric literal");
        return None;
    };
    if is_minus_item(item) {
        let Some(next) = cursor.next() else {
            log_expected(logger, op_span, "numeric literal");
            return None;
        };
        return parse_float_literal_from_item(next, logger).map(|value| -value);
    }
    parse_float_literal_from_item(item, logger)
}

fn is_minus_item(item: &SexprItem) -> bool {
    matches!(item, SexprItem::Atom(SexprAtom::Ident(token)) if token.text() == "-")
}

fn parse_immediate(
    item: &SexprItem,
    logger: &mut FileLogger,
) -> Option<ImmediateValue> {
    match item {
        SexprItem::Atom(SexprAtom::Integer(token)) => {
            parse_integer_literal(token.text())
                .map(ImmediateValue::Integer)
                .or_else(|| {
                    log_invalid(logger, item.span(), "Invalid integer literal.");
                    None
                })
        }
        SexprItem::Atom(SexprAtom::Float(token)) => {
            parse_float_literal(token.text())
                .map(ImmediateValue::Real)
                .or_else(|| {
                    log_invalid(logger, item.span(), "Invalid float literal.");
                    None
                })
        }
        SexprItem::Atom(SexprAtom::String(token)) => {
            string_token_value(token.text())
                .map(ImmediateValue::String)
                .or_else(|| {
                    log_invalid(logger, item.span(), "Invalid string literal.");
                    None
                })
        }
        SexprItem::Atom(SexprAtom::Bool(_, value)) => Some(ImmediateValue::Boolean(*value)),
        SexprItem::List(list) if list.items().is_empty() => Some(ImmediateValue::Unit),
        _ => {
            log_invalid(logger, item.span(), "Unsupported immediate value.");
            None
        }
    }
}

fn parse_integer_literal_from_item(
    item: &SexprItem,
    logger: &mut FileLogger,
) -> Option<i64> {
    let SexprItem::Atom(SexprAtom::Integer(token)) = item else {
        log_invalid(logger, item.span(), "Expected an integer literal.");
        return None;
    };
    parse_integer_literal(token.text()).or_else(|| {
        log_invalid(logger, item.span(), "Invalid integer literal.");
        None
    })
}

fn parse_float_literal_from_item(
    item: &SexprItem,
    logger: &mut FileLogger,
) -> Option<f64> {
    let raw = match item {
        SexprItem::Atom(SexprAtom::Float(token)) => token.text(),
        SexprItem::Atom(SexprAtom::Integer(token)) => token.text(),
        _ => {
            log_invalid(logger, item.span(), "Expected a numeric literal.");
            return None;
        }
    };
    parse_float_literal(raw).or_else(|| {
        log_invalid(logger, item.span(), "Invalid float literal.");
        None
    })
}

fn parse_string_literal_from_item(
    item: &SexprItem,
    logger: &mut FileLogger,
) -> Option<String> {
    let SexprItem::Atom(SexprAtom::String(token)) = item else {
        log_invalid(logger, item.span(), "Expected a string literal.");
        return None;
    };
    string_token_value(token.text()).or_else(|| {
        log_invalid(logger, item.span(), "Invalid string literal.");
        None
    })
}

fn parse_glyph_literal_from_item(
    item: &SexprItem,
    logger: &mut FileLogger,
) -> Option<char> {
    let value = parse_string_literal_from_item(item, logger)?;
    let mut chars = value.chars();
    let Some(ch) = chars.next() else {
        log_invalid(logger, item.span(), "Glyphs cannot be empty.");
        return None;
    };
    if chars.next().is_some() {
        log_invalid(logger, item.span(), "Glyphs must be a single character.");
        return None;
    }
    Some(ch)
}

fn parse_u32_immediate(
    item: Option<&SexprItem>,
    context: &str,
    span: Span,
    logger: &mut FileLogger,
) -> Option<u32> {
    let Some(item) = item else {
        logger
            .error(format!("Missing {context}"))
            .primary(format!("Expected {context} as an integer literal."), span)
            .done();
        return None;
    };
    let SexprItem::Atom(SexprAtom::Integer(token)) = item else {
        logger
            .error(format!("Invalid {context}"))
            .primary(
                format!("Expected {context} as an integer literal."),
                item.span(),
            )
            .done();
        return None;
    };
    let value = parse_integer_literal(token.text()).or_else(|| {
        logger
            .error(format!("Invalid {context}"))
            .primary(
                format!("`{}` is not a valid integer.", token.text()),
                item.span(),
            )
            .done();
        None
    })?;
    u32::try_from(value).ok().or_else(|| {
        logger
            .error(format!("Invalid {context}"))
            .primary("The value does not fit in a u32.", item.span())
            .done();
        None
    })
}

fn parse_type(
    item: &SexprItem,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Type> {
    match item {
        SexprItem::Atom(SexprAtom::Ident(token))
        | SexprItem::Atom(SexprAtom::SymbolIdent(token)) => {
            parse_type_ident(token.text(), item.span(), env, logger)
        }
        SexprItem::List(list) => parse_type_list(list, env, logger),
        SexprItem::Path(_) => {
            log_invalid(
                logger,
                item.span(),
                "Paths are not allowed in type positions.",
            );
            None
        }
        _ => {
            log_invalid(logger, item.span(), "Expected a type.");
            None
        }
    }
}

fn parse_type_ident(
    name: &str,
    span: Span,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Type> {
    if env.defining == Some(name) {
        log_invalid(logger, span, "Recursive type definitions are not allowed.");
        return None;
    }
    if let Some(definition) = env.type_defs.get(name) {
        return Some(definition.clone());
    }
    match name {
        "any" => Some(Type::Any),
        "i8" => Some(Type::I8),
        "i16" => Some(Type::I16),
        "i32" => Some(Type::I32),
        "i64" => Some(Type::I64),
        "f32" => Some(Type::F32),
        "f64" => Some(Type::F64),
        _ => {
            log_invalid(logger, span, "Unknown type identifier.");
            None
        }
    }
}

fn parse_type_list(
    list: &Sexpr,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<Type> {
    let items = list.items();
    let Some(keyword) = items.first().and_then(item_keyword) else {
        log_invalid(logger, list.span(), "Expected a type expression.");
        return None;
    };
    match keyword.as_str() {
        "struct" => {
            let fields = items
                .iter()
                .skip(1)
                .filter_map(|item| parse_type(item, env, logger))
                .collect::<Vec<_>>();
            Some(Type::Struct(fields.into_boxed_slice()))
        }
        "array" => {
            let inner = items.get(1).and_then(|item| parse_type(item, env, logger));
            let Some(inner) = inner else {
                log_expected(logger, list.span(), "array element type");
                return None;
            };
            if items.len() > 2 {
                log_invalid(logger, list.span(), "Arrays take a single element type.");
            }
            Some(Type::Array(Box::new(inner)))
        }
        "func" => {
            parse_function_type(list, env, logger).map(|function_type| {
                Type::Function {
                    parameters: function_type.parameters,
                    results: function_type.results,
                }
            })
        }
        _ => {
            log_invalid(logger, list.span(), "Unsupported type expression.");
            None
        }
    }
}

fn parse_function_type(
    list: &Sexpr,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Option<FunctionType> {
    let items = list.items();
    let mut parameters = Vec::new();
    let mut results = Vec::new();
    for item in items.iter().skip(1) {
        let SexprItem::List(inner_list) = item else {
            log_invalid(
                logger,
                item.span(),
                "Function types require `(param ...)` or `(result ...)`.",
            );
            continue;
        };
        let Some(keyword) = declaration_keyword(inner_list) else {
            log_invalid(
                logger,
                inner_list.span(),
                "Expected `(param ...)` or `(result ...)`.",
            );
            continue;
        };
        match keyword.as_str() {
            "param" => {
                parameters.extend(parse_types_from_list(inner_list, env, logger));
            }
            "result" => {
                results.extend(parse_types_from_list(inner_list, env, logger));
            }
            _ => {
                log_invalid(
                    logger,
                    inner_list.span(),
                    "Expected `(param ...)` or `(result ...)`.",
                );
            }
        }
    }
    Some(FunctionType {
        parameters: parameters.into_boxed_slice(),
        results: results.into_boxed_slice(),
    })
}

fn parse_types_from_list(
    list: &Sexpr,
    env: &TypeEnv<'_>,
    logger: &mut FileLogger,
) -> Vec<Type> {
    list.items()
        .iter()
        .skip(1)
        .filter_map(|item| parse_type(item, env, logger))
        .collect()
}

fn is_type_item(
    item: &SexprItem,
    env: &TypeEnv<'_>,
) -> bool {
    match item {
        SexprItem::Atom(SexprAtom::Ident(token))
        | SexprItem::Atom(SexprAtom::SymbolIdent(token)) => {
            let name = token.text();
            env.type_defs.contains_key(name)
                || matches!(name, "any" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64")
        }
        SexprItem::List(list) => {
            matches!(
                declaration_keyword(list).as_deref(),
                Some("struct" | "array" | "func")
            )
        }
        _ => false,
    }
}

fn item_keyword(item: &SexprItem) -> Option<String> {
    match item {
        SexprItem::Atom(SexprAtom::Ident(token)) => Some(token.text().to_string()),
        SexprItem::Field(field) => {
            let lhs = field.lhs_token()?.text().to_string();
            let rhs = field.rhs_token()?.text().to_string();
            Some(format!("{lhs}.{rhs}"))
        }
        _ => None,
    }
}

fn item_symbol_ident(item: &SexprItem) -> Option<String> {
    match item {
        SexprItem::Atom(SexprAtom::SymbolIdent(token)) => Some(token.text().to_string()),
        _ => None,
    }
}

fn expect_symbol_name(
    cursor: &mut Cursor<'_>,
    expected: &str,
    fallback_span: Span,
    logger: &mut FileLogger,
) -> Option<(String, Span)> {
    let Some(item) = cursor.next() else {
        log_expected(logger, fallback_span, expected);
        return None;
    };
    let span = item.span();
    let Some(name) = item_symbol_ident(item) else {
        log_invalid(
            logger,
            span,
            &format!("{expected} must be a `$`-prefixed identifier."),
        );
        return None;
    };
    Some((name, span))
}

fn log_expected(
    logger: &mut FileLogger,
    span: Span,
    expected: &str,
) {
    logger
        .error("Expected value")
        .primary(format!("Expected {expected}."), span)
        .done();
}

fn log_invalid(
    logger: &mut FileLogger,
    span: Span,
    message: &str,
) {
    logger.error("Invalid value").primary(message, span).done();
}

fn string_token_value(text: &str) -> Option<String> {
    crate::parse::lexer::decode_quoted_string_literal(text)
}

fn parse_integer_literal(text: &str) -> Option<i64> {
    crate::parse::lexer::parse_integer_literal(text)
}

fn parse_float_literal(text: &str) -> Option<f64> {
    crate::parse::lexer::parse_real_literal(text)
}
