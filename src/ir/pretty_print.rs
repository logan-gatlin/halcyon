use crate::types::Type;

use crate::asm::{
    Instruction as WasmInstruction,
    NumberOperation,
    Type as WasmType,
};
use indexmap::IndexMap;

use super::{
    wasm,
    *,
};

const INDENT_WIDTH: usize = 2;
const LINE_LIMIT: usize = 96;

impl std::fmt::Display for Module<Type> {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let mut printer = Printer::new(Some(self.name.clone()));
        printer.module(self);
        write!(f, "{}", printer.finish())
    }
}

pub fn pretty_typed_term(term: &Term<Type>) -> String {
    let mut printer = Printer::new(None);
    printer.term(term);
    printer.finish()
}

pub fn pretty_typed_pattern(pattern: &Pattern<Type>) -> String {
    let mut printer = Printer::new(None);
    printer.line(printer.format_pattern_typed(pattern));
    printer.finish()
}

struct MatchArm<'a> {
    pattern: &'a Pattern<Type>,
    body: &'a Term<Type>,
}

struct MatchView<'a> {
    scrutinee: &'a Term<Type>,
    arms: Vec<MatchArm<'a>>,
}

struct Printer {
    indent: usize,
    output: String,
    module_name: Option<String>,
}

impl Printer {
    fn new(module_name: Option<String>) -> Self {
        Self {
            indent: 0,
            output: String::new(),
            module_name,
        }
    }

    fn finish(self) -> String {
        self.output
    }

    fn line(
        &mut self,
        text: impl AsRef<str>,
    ) {
        let text = text.as_ref();
        if !text.is_empty() {
            self.output.push_str(&" ".repeat(self.indent));
            self.output.push_str(text);
        }
        self.output.push('\n');
    }

    fn indented(
        &mut self,
        f: impl FnOnce(&mut Self),
    ) {
        self.indent += INDENT_WIDTH;
        f(self);
        self.indent = self.indent.saturating_sub(INDENT_WIDTH);
    }

    fn module(
        &mut self,
        module: &Module<Type>,
    ) {
        self.line(format!("module {} =", module.name));
        self.indented(|printer| {
            if module.statements.is_empty() {
                printer.line("<empty>");
                return;
            }
            for (index, statement) in module.statements.iter().enumerate() {
                if index > 0 {
                    printer.line("");
                }
                printer.statement(statement);
            }
        });
        self.line("end");
    }

    fn statement(
        &mut self,
        statement: &Statement<Type>,
    ) {
        match statement {
            Statement::Term(term) => self.term(term),
            Statement::Type {
                path,
                parameters,
                def,
            } => self.type_statement(path, parameters, def),
            Statement::Trait {
                path,
                parameters,
                methods,
            } => self.trait_statement(path, parameters, methods),
            Statement::Impl {
                trait_path,
                arguments,
                methods,
            } => self.impl_statement(trait_path, arguments, methods),
            Statement::Wasm(declarations) => self.wasm_statement(declarations),
        }
    }

    fn trait_statement(
        &mut self,
        path: &Path,
        parameters: &[Path],
        methods: &[TraitMethodDecl],
    ) {
        let params = if parameters.is_empty() {
            String::new()
        } else {
            let names = parameters
                .iter()
                .map(|param| self.format_path(param))
                .collect::<Vec<_>>()
                .join(" ");
            format!(" : {names}")
        };
        self.line(format!("trait {}{params} =", self.format_path(path)));
        self.indented(|printer| {
            for method in methods {
                printer.line(format!(
                    "let {} : {}",
                    printer.format_path(&method.path),
                    printer.format_type_expr(&method.type_expr)
                ));
            }
        });
        self.line("end");
    }

    fn impl_statement(
        &mut self,
        trait_path: &Path,
        arguments: &[TypeExpr],
        methods: &[ImplMethod<Type>],
    ) {
        let args = arguments
            .iter()
            .map(|arg| self.format_type_expr(arg))
            .collect::<Vec<_>>()
            .join(", ");
        self.line(format!("impl {} : {args} =", self.format_path(trait_path)));
        self.indented(|printer| {
            for method in methods {
                let method_name = printer.format_path(&method.trait_method);
                if let Some(value) = printer.format_term_inline_expr(&method.value) {
                    let line = format!("let {method_name} = {value}");
                    if line.len() <= LINE_LIMIT {
                        printer.line(line);
                        continue;
                    }
                }
                printer.line(format!("let {method_name} ="));
                printer.indented(|printer| printer.term(&method.value));
            }
        });
        self.line("end");
    }

    fn wasm_statement(
        &mut self,
        declarations: &[wasm::Declaration],
    ) {
        if declarations.is_empty() {
            self.line("wasm => ()");
            return;
        }
        if declarations.len() == 1 {
            self.line("wasm =>");
            self.indented(|printer| {
                printer.wasm_declaration(&declarations[0]);
            });
            return;
        }
        self.line("wasm => (");
        self.indented(|printer| {
            for declaration in declarations {
                printer.wasm_declaration(declaration);
            }
        });
        self.line(")");
    }

    fn wasm_declaration(
        &mut self,
        declaration: &wasm::Declaration,
    ) {
        match declaration {
            wasm::Declaration::Type(type_def) => self.wasm_type_definition(type_def),
            wasm::Declaration::Global(global) => self.wasm_global(global),
            wasm::Declaration::Function(function) => self.wasm_function(function),
            wasm::Declaration::Memory(memory) => self.wasm_memory(memory),
        }
    }

    fn wasm_type_definition(
        &mut self,
        type_def: &wasm::TypeDefinition,
    ) {
        let name = self.format_wasm_name(&type_def.name);
        let type_expr = self.format_wasm_type(&type_def.type_);
        self.line(format!("(type {name} {type_expr})"));
    }

    fn wasm_global(
        &mut self,
        global: &wasm::Global,
    ) {
        let name = self.format_wasm_global_name(&global.name);
        let type_expr = self.format_wasm_type(&global.type_);
        self.line(format!("(global {name} {type_expr})"));
    }

    fn wasm_function(
        &mut self,
        function: &wasm::Function,
    ) {
        let name = self.format_wasm_name(&function.name);
        self.line(format!("(func {name}"));
        self.indented(|printer| {
            for line in printer.wasm_named_types_lines("param", &function.parameters) {
                printer.line(line);
            }
            for line in printer.wasm_result_lines(&function.results) {
                printer.line(line);
            }
            for line in printer.wasm_named_types_lines("local", &function.locals) {
                printer.line(line);
            }
            for instruction in function.body.iter() {
                printer.line(printer.format_wasm_instruction(instruction));
            }
        });
        self.line(")");
    }

    fn wasm_memory(
        &mut self,
        memory: &wasm::Memory,
    ) {
        let name = self.format_wasm_name(&memory.name);
        let line = match memory.maximum_size {
            Some(max) => format!("(memory {name} {} {max})", memory.initial_size),
            None => format!("(memory {name} {})", memory.initial_size),
        };
        self.line(line);
    }

    fn wasm_named_types_lines(
        &self,
        keyword: &str,
        items: &IndexMap<Path, WasmType>,
    ) -> Vec<String> {
        if items.is_empty() {
            return Vec::new();
        }
        let inline = format!(
            "({keyword} {})",
            items
                .iter()
                .map(|(name, ty)| {
                    format!(
                        "{} {}",
                        self.format_wasm_local_name(name),
                        self.format_wasm_type(ty)
                    )
                })
                .collect::<Vec<_>>()
                .join(" ")
        );
        if inline.len() <= LINE_LIMIT {
            return vec![inline];
        }
        items
            .iter()
            .map(|(name, ty)| {
                format!(
                    "({keyword} {} {})",
                    self.format_wasm_local_name(name),
                    self.format_wasm_type(ty)
                )
            })
            .collect()
    }

    fn wasm_result_lines(
        &self,
        results: &[WasmType],
    ) -> Vec<String> {
        if results.is_empty() {
            return Vec::new();
        }
        let inline = format!(
            "(result {})",
            results
                .iter()
                .map(|ty| self.format_wasm_type(ty))
                .collect::<Vec<_>>()
                .join(" ")
        );
        if inline.len() <= LINE_LIMIT {
            return vec![inline];
        }
        results
            .iter()
            .map(|ty| format!("(result {})", self.format_wasm_type(ty)))
            .collect()
    }

    fn format_wasm_instruction(
        &self,
        instruction: &WasmInstruction,
    ) -> String {
        match instruction {
            WasmInstruction::Set(path) => format!("set {}", self.format_wasm_path(path)),
            WasmInstruction::Get(path) => format!("get {}", self.format_wasm_path(path)),
            WasmInstruction::Const(value) => {
                format!("const {}", self.format_wasm_immediate(value))
            }
            WasmInstruction::I32Const(value) => format!("i32.const {value}"),
            WasmInstruction::F32Const(value) => format!("f32.const {value}"),
            WasmInstruction::Func(path) => format!("func {}", self.format_wasm_path(path)),
            WasmInstruction::StructNew(fields) => {
                format!("struct.new {}", self.format_wasm_struct_type(fields))
            }
            WasmInstruction::StructGet(fields, index) => {
                format!(
                    "struct.get {} {index}",
                    self.format_wasm_struct_type(fields)
                )
            }
            WasmInstruction::ArrayGet(inner) => {
                format!("array.get {}", self.format_wasm_type(inner))
            }
            WasmInstruction::ArrayNewFixed { inner_type, length } => {
                format!(
                    "array.new_fixed {} {length}",
                    self.format_wasm_type(inner_type)
                )
            }
            WasmInstruction::ArrayNewDefault(inner) => {
                format!("array.new_default {}", self.format_wasm_type(inner))
            }
            WasmInstruction::ArrayLen => "array.len".to_string(),
            WasmInstruction::ArrayCopy { dst_type, src_type } => {
                format!(
                    "array.copy {} {}",
                    self.format_wasm_type(dst_type),
                    self.format_wasm_type(src_type)
                )
            }
            WasmInstruction::CallRef {
                parameters,
                returns,
            } => {
                let func_type = WasmType::Function {
                    parameters: parameters.clone(),
                    results: returns.clone(),
                };
                format!("call.ref {}", self.format_wasm_type(&func_type))
            }
            WasmInstruction::Call(path) => format!("call {}", self.format_wasm_path(path)),
            WasmInstruction::Unreachable => "unreachable".to_string(),
            WasmInstruction::Drop => "drop".to_string(),
            WasmInstruction::If(type_) => {
                match type_ {
                    Some(type_) => format!("if {}", self.format_wasm_type(type_)),
                    None => "if".to_string(),
                }
            }
            WasmInstruction::Else => "else".to_string(),
            WasmInstruction::End => "end".to_string(),
            WasmInstruction::Loop => "loop".to_string(),
            WasmInstruction::Block(type_) => {
                match type_ {
                    Some(type_) => format!("block {}", self.format_wasm_type(type_)),
                    None => "block".to_string(),
                }
            }
            WasmInstruction::Break(depth) => format!("break {depth}"),
            WasmInstruction::BreakIf(depth) => format!("break.if {depth}"),
            WasmInstruction::I32Op(op) => {
                format!("i32.{}", self.format_wasm_number_op(*op))
            }
            WasmInstruction::I64Op(op) => {
                format!("i64.{}", self.format_wasm_number_op(*op))
            }
            WasmInstruction::F32Op(op) => {
                format!("f32.{}", self.format_wasm_number_op(*op))
            }
            WasmInstruction::F64Op(op) => {
                format!("f64.{}", self.format_wasm_number_op(*op))
            }
            WasmInstruction::RefCastFunc {
                parameters,
                returns,
            } => {
                let func_type = WasmType::Function {
                    parameters: parameters.clone(),
                    results: returns.clone(),
                };
                format!("ref.cast.func {}", self.format_wasm_type(&func_type))
            }
            WasmInstruction::RefCastStruct(fields) => {
                format!("ref.cast.struct {}", self.format_wasm_struct_type(fields))
            }
            WasmInstruction::RefCastArray(inner) => {
                format!("ref.cast.array {}", self.format_wasm_type(inner))
            }
            WasmInstruction::I32Store8 => "i32.store8".to_string(),
            WasmInstruction::I32Store => "i32.store".to_string(),
        }
    }

    fn format_wasm_type(
        &self,
        ty: &WasmType,
    ) -> String {
        match ty {
            WasmType::Any => "any".to_string(),
            WasmType::I8 => "i8".to_string(),
            WasmType::I16 => "i16".to_string(),
            WasmType::I32 => "i32".to_string(),
            WasmType::I64 => "i64".to_string(),
            WasmType::F32 => "f32".to_string(),
            WasmType::F64 => "f64".to_string(),
            WasmType::Struct(fields) => self.format_wasm_struct_type(fields),
            WasmType::Array(inner) => {
                format!("(array {})", self.format_wasm_type(inner))
            }
            WasmType::Function {
                parameters,
                results,
            } => {
                let mut parts = Vec::new();
                if !parameters.is_empty() {
                    parts.push(format!(
                        "(param {})",
                        parameters
                            .iter()
                            .map(|ty| self.format_wasm_type(ty))
                            .collect::<Vec<_>>()
                            .join(" ")
                    ));
                }
                if !results.is_empty() {
                    parts.push(format!(
                        "(result {})",
                        results
                            .iter()
                            .map(|ty| self.format_wasm_type(ty))
                            .collect::<Vec<_>>()
                            .join(" ")
                    ));
                }
                if parts.is_empty() {
                    return "(func)".to_string();
                }
                format!("(func {})", parts.join(" "))
            }
        }
    }

    fn format_wasm_struct_type(
        &self,
        fields: &[WasmType],
    ) -> String {
        let fields = fields
            .iter()
            .map(|ty| self.format_wasm_type(ty))
            .collect::<Vec<_>>()
            .join(" ");
        if fields.is_empty() {
            "(struct)".to_string()
        } else {
            format!("(struct {fields})")
        }
    }

    fn format_wasm_number_op(
        &self,
        op: NumberOperation,
    ) -> String {
        match op {
            NumberOperation::Eq => "eq".to_string(),
            NumberOperation::Ne => "ne".to_string(),
            NumberOperation::Gt => "gt".to_string(),
            NumberOperation::Lt => "lt".to_string(),
            NumberOperation::Ge => "ge".to_string(),
            NumberOperation::Le => "le".to_string(),
            NumberOperation::Add => "add".to_string(),
            NumberOperation::Sub => "sub".to_string(),
            NumberOperation::Mul => "mul".to_string(),
            NumberOperation::Div => "div".to_string(),
            NumberOperation::Rem => "rem".to_string(),
            NumberOperation::And => "and".to_string(),
            NumberOperation::Or => "or".to_string(),
            NumberOperation::Xor => "xor".to_string(),
        }
    }

    fn format_wasm_immediate(
        &self,
        value: &ImmediateValue,
    ) -> String {
        match value {
            ImmediateValue::Unit => "()".to_string(),
            ImmediateValue::Integer(value) => value.to_string(),
            ImmediateValue::Real(value) => value.to_string(),
            ImmediateValue::Boolean(value) => value.to_string(),
            ImmediateValue::String(value) => {
                format!("\"{}\"", escape_sexpr_string(value))
            }
            ImmediateValue::Glyph(value) => {
                format!("'{}'", escape_sexpr_glyph(*value))
            }
        }
    }

    fn format_wasm_path(
        &self,
        path: &Path,
    ) -> String {
        if path.major == "[local]" {
            return self.format_wasm_symbol_name(&path.minor);
        }
        if path.major.is_empty() {
            return self.format_wasm_symbol_name(&path.minor);
        }
        if is_sexpr_ident(&path.major) && is_sexpr_ident(&path.minor) {
            return format!("${}::{}", path.major, path.minor);
        }
        let escaped = escape_sexpr_string(&format!("{}::{}", path.major, path.minor));
        format!("$\"{escaped}\"")
    }

    fn format_wasm_global_name(
        &self,
        path: &Path,
    ) -> String {
        if self
            .module_name
            .as_ref()
            .is_some_and(|name| name == &path.major)
        {
            return self.format_wasm_symbol_name(&path.minor);
        }
        self.format_wasm_path(path)
    }

    fn format_wasm_local_name(
        &self,
        path: &Path,
    ) -> String {
        if path.major == "[local]" || path.major.is_empty() {
            return self.format_wasm_symbol_name(&path.minor);
        }
        self.format_wasm_path(path)
    }

    fn format_wasm_symbol_name(
        &self,
        name: &str,
    ) -> String {
        if is_sexpr_ident(name) {
            format!("${name}")
        } else {
            format!("$\"{}\"", escape_sexpr_string(name))
        }
    }

    fn format_wasm_name(
        &self,
        name: &str,
    ) -> String {
        if is_sexpr_ident(name) {
            name.to_string()
        } else {
            format!("\"{}\"", escape_sexpr_string(name))
        }
    }

    fn type_statement(
        &mut self,
        path: &Path,
        parameters: &[Path],
        def: &TypeDef,
    ) {
        let params = if parameters.is_empty() {
            String::new()
        } else {
            let names = parameters
                .iter()
                .map(|param| self.format_path(param))
                .collect::<Vec<_>>()
                .join(" ");
            format!(" : {names}")
        };
        if let Some(def_inline) = self.format_type_def_inline(def) {
            let line = format!("type {}{params} = {def_inline}", self.format_path(path));
            if line.len() <= LINE_LIMIT {
                self.line(line);
                return;
            }
        }
        self.line(format!("type {}{params} =", self.format_path(path)));
        self.indented(|printer| printer.type_def(def));
    }

    fn term(
        &mut self,
        term: &Term<Type>,
    ) {
        if let Some(view) = match_view(term) {
            self.term_match(term, view);
            return;
        }
        if let Some(line) = self.format_term_inline(term) {
            self.line(line);
            return;
        }

        match &term.kind {
            TermKind::Let {
                assignee,
                scope,
                value,
                then,
                else_,
            } => self.term_let(term, assignee, *scope, value, then, else_),
            TermKind::Immediate(value) => {
                self.line(format!("{value} : {}", term.type_.pretty()));
            }
            TermKind::Identifier(path) => {
                self.line(format!(
                    "{} : {}",
                    self.format_path(path),
                    term.type_.pretty()
                ));
            }
            TermKind::Tuple(items) => self.term_tuple(term, items),
            TermKind::Struct(fields) => self.term_struct(term, fields),
            TermKind::Field { of, index } => self.term_field(term, of, index),
            TermKind::Function {
                parameter_name,
                parameter_type,
                body,
                ..
            } => self.term_function(term, parameter_name, parameter_type.as_ref(), body),
            TermKind::InlineWasm {
                asserted_type,
                definitions,
                instructions,
            } => self.term_inline_wasm(term, asserted_type, definitions, instructions),
            TermKind::Call { callee, argument } => self.term_call(term, callee, argument),
            TermKind::Semicolon(left, right) => self.term_semicolon(term, left, right),
            TermKind::Unreachable => {
                self.line(format!("unreachable : {}", term.type_.pretty()));
            }
        }
    }

    fn term_let(
        &mut self,
        term: &Term<Type>,
        assignee: &Pattern<Type>,
        scope: ScopeKind,
        value: &Term<Type>,
        then: &Term<Type>,
        else_: &Term<Type>,
    ) {
        if scope == ScopeKind::Global {
            let pattern = self.format_pattern_typed(assignee);
            let header = format!("let {pattern} =");
            if header.len() <= LINE_LIMIT {
                self.line(header);
            } else {
                self.line("let");
                self.indented(|printer| printer.line(format!("{pattern} =")));
            }
            self.indented(|printer| printer.term(value));
            return;
        }
        if scope == ScopeKind::Local
            && matches!(
                assignee.kind,
                PatternKind::Immediate(ImmediateValue::Boolean(true))
            )
            && !is_unreachable(else_)
        {
            let condition = self.format_term_inline_expr(value);
            if let Some(condition) = condition {
                let line = format!("if {condition}");
                if line.len() <= LINE_LIMIT {
                    self.line(line);
                } else {
                    self.line("if");
                    self.indented(|printer| printer.term(value));
                }
            } else {
                self.line("if");
                self.indented(|printer| printer.term(value));
            }
            self.line("then");
            self.indented(|printer| printer.term(then));
            self.line("else");
            self.indented(|printer| printer.term(else_));
            self.line(format!(": {}", term.type_.pretty()));
            return;
        }

        let pattern = self.format_pattern_typed(assignee);
        let header = format!("let {pattern} =");
        if header.len() <= LINE_LIMIT {
            self.line(header);
        } else {
            self.line("let");
            self.indented(|printer| printer.line(format!("{pattern} =")));
        }
        self.indented(|printer| printer.term(value));

        if scope == ScopeKind::Local && is_unreachable(else_) {
            self.line("in");
            self.indented(|printer| printer.term(then));
        } else {
            self.line("then");
            self.indented(|printer| printer.term(then));
            self.line("else");
            self.indented(|printer| printer.term(else_));
        }
        self.line(format!(": {}", term.type_.pretty()));
    }

    fn term_match(
        &mut self,
        term: &Term<Type>,
        view: MatchView<'_>,
    ) {
        let inline = self.format_term_inline_expr(view.scrutinee);
        if let Some(inline) = inline {
            let line = format!("match {inline} with");
            if line.len() <= LINE_LIMIT {
                self.line(line);
            } else {
                self.line("match");
                self.indented(|printer| printer.term(view.scrutinee));
                self.line("with");
            }
        } else {
            self.line("match");
            self.indented(|printer| printer.term(view.scrutinee));
            self.line("with");
        }
        self.indented(|printer| {
            for arm in view.arms {
                let pattern = printer.format_pattern_typed(arm.pattern);
                if let Some(body_inline) = printer.format_term_inline(arm.body) {
                    let line = format!("| {pattern} => {body_inline}");
                    if line.len() <= LINE_LIMIT {
                        printer.line(line);
                        continue;
                    }
                }
                printer.line(format!("| {pattern} =>"));
                printer.indented(|printer| printer.term(arm.body));
            }
        });
        self.line(format!(": {}", term.type_.pretty()));
    }

    fn term_function(
        &mut self,
        term: &Term<Type>,
        parameter_name: &Spanned<Path>,
        parameter_type: Option<&TypeExpr>,
        body: &Term<Type>,
    ) {
        let name = self.format_path(&parameter_name.inner);
        let param = match parameter_type {
            Some(type_expr) => {
                let ty = self.format_type_expr(type_expr);
                format!("({name} : {ty})")
            }
            None => name,
        };
        self.line(format!("fn {param} =>"));
        self.indented(|printer| printer.term(body));
        self.line(format!(": {}", term.type_.pretty()));
    }

    fn term_call(
        &mut self,
        term: &Term<Type>,
        callee: &Term<Type>,
        argument: &Term<Type>,
    ) {
        self.line("(");
        self.indented(|printer| {
            printer.term(callee);
            printer.term(argument);
        });
        self.line(format!(") : {}", term.type_.pretty()));
    }

    fn term_tuple(
        &mut self,
        term: &Term<Type>,
        items: &[Term<Type>],
    ) {
        self.line("(");
        self.indented(|printer| {
            if items.is_empty() {
                printer.line("()");
                return;
            }
            for item in items.iter() {
                printer.term(item);
            }
        });
        self.line(format!(") : {}", term.type_.pretty()));
    }

    fn term_struct(
        &mut self,
        term: &Term<Type>,
        fields: &IndexMap<Spanned<String>, Term<Type>>,
    ) {
        self.line("{");
        self.indented(|printer| {
            if fields.is_empty() {
                printer.line("{}");
                return;
            }
            for (name, value) in fields.iter() {
                if let Some(inline) = printer.format_term_inline(value) {
                    printer.line(format!("{name} = {inline}"));
                } else {
                    printer.line(format!("{name} ="));
                    printer.indented(|printer| printer.term(value));
                }
            }
        });
        self.line(format!("}} : {}", term.type_.pretty()));
    }

    fn term_field(
        &mut self,
        term: &Term<Type>,
        of: &Term<Type>,
        index: &Spanned<String>,
    ) {
        self.line("(");
        self.indented(|printer| printer.term(of));
        self.line(format!(").{index} : {}", term.type_.pretty()));
    }

    fn term_semicolon(
        &mut self,
        term: &Term<Type>,
        left: &Term<Type>,
        right: &Term<Type>,
    ) {
        self.line("(");
        self.indented(|printer| {
            if let Some(line) = printer.format_term_inline(left) {
                printer.line(format!("{line};"));
            } else {
                printer.term(left);
                printer.line(";");
            }
            printer.term(right);
        });
        self.line(format!(") : {}", term.type_.pretty()));
    }

    fn term_inline_wasm(
        &mut self,
        term: &Term<Type>,
        asserted_type: &TypeExpr,
        definitions: &IndexMap<Path, WasmType>,
        instructions: &[WasmInstruction],
    ) {
        self.line(format!(
            "(wasm : {}) => (",
            self.format_type_expr(asserted_type)
        ));
        self.indented(|printer| {
            for (name, type_) in definitions {
                printer.line(format!(
                    "(local {} {})",
                    printer.format_wasm_local_name(name),
                    printer.format_wasm_type(type_),
                ));
            }
            for instruction in instructions {
                printer.line(printer.format_wasm_instruction(instruction));
            }
        });
        self.line(format!(") : {}", term.type_.pretty()));
    }

    fn type_def(
        &mut self,
        def: &TypeDef,
    ) {
        match def.kind() {
            TypeDefKind::Struct(fields) => {
                self.line("{");
                self.indented(|printer| {
                    if fields.is_empty() {
                        printer.line("{}");
                        return;
                    }
                    for (name, type_expr) in fields.iter() {
                        printer.line(format!("{name}: {}", printer.format_type_expr(type_expr)));
                    }
                });
                self.line("}");
            }
            TypeDefKind::Sum(variants) => {
                if variants.is_empty() {
                    self.line("|");
                    return;
                }
                for (name, type_expr) in variants.iter() {
                    let expr = self.format_type_expr(type_expr);
                    if expr == "()" {
                        self.line(format!("| {name}"));
                    } else {
                        self.line(format!("| {name} {expr}"));
                    }
                }
            }
            TypeDefKind::Expr(expr) => {
                self.line(self.format_type_expr(expr));
            }
        }
    }

    fn format_type_def_inline(
        &self,
        def: &TypeDef,
    ) -> Option<String> {
        match def.kind() {
            TypeDefKind::Struct(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, type_expr)| {
                        format!("{name}: {}", self.format_type_expr(type_expr))
                    })
                    .collect::<Vec<_>>();
                Some(format!("{{ {} }}", fields.join(", ")))
            }
            TypeDefKind::Sum(variants) => {
                if variants.is_empty() {
                    return Some("|".to_string());
                }
                let variants = variants
                    .iter()
                    .map(|(name, type_expr)| {
                        let expr = self.format_type_expr(type_expr);
                        if expr == "()" {
                            name.clone()
                        } else {
                            format!("{name} {expr}")
                        }
                    })
                    .collect::<Vec<_>>();
                Some(format!("| {}", variants.join(" | ")))
            }
            TypeDefKind::Expr(expr) => Some(self.format_type_expr(expr)),
        }
    }

    #[allow(clippy::missing_asserts_for_indexing)]
    fn format_type_expr(
        &self,
        expr: &TypeExpr,
    ) -> String {
        match &expr.kind {
            TypeExprKind::Tuple(items) => {
                let items = items
                    .iter()
                    .map(|item| self.format_type_expr(item))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({items})")
            }
            TypeExprKind::Instantiation(path, args) => {
                let is_core = path.major == "core";
                if is_core && path.minor == "unit" && args.is_empty() {
                    return "()".to_string();
                }
                if is_core && path.minor == "array" && args.is_empty() {
                    return "[]".to_string();
                }
                if is_core && path.minor == "array" && args.len() == 1 {
                    let inner = self.format_type_expr(&args[0]);
                    return format!("[] {}", self.wrap_type_expr(&inner));
                }
                if is_core && path.minor == "function" && args.len() == 2 {
                    let param = self.wrap_type_expr(&self.format_type_expr(&args[0]));
                    let result = self.format_type_expr(&args[1]);
                    return format!("{param} -> {result}");
                }
                let name = self.format_path(path);
                if args.is_empty() {
                    return name;
                }
                let args = args
                    .iter()
                    .map(|arg| self.wrap_type_expr(&self.format_type_expr(arg)))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("{name} {args}")
            }
        }
    }

    fn format_pattern_typed(
        &self,
        pattern: &Pattern<Type>,
    ) -> String {
        let raw = self.format_pattern(pattern);
        let wrapped = self.wrap_pattern(&raw);
        format!("{wrapped} : {}", pattern.type_.pretty())
    }

    fn format_pattern(
        &self,
        pattern: &Pattern<Type>,
    ) -> String {
        match &pattern.kind {
            PatternKind::Hole => "_".to_string(),
            PatternKind::Identifier(path) => self.format_path(path),
            PatternKind::ConstConstructor(path) => self.format_path(path),
            PatternKind::Constructor(path, payload) => {
                let payload = self.wrap_pattern(&self.format_pattern(payload));
                format!("{} of {payload}", self.format_path(path))
            }
            PatternKind::Tuple(items) => {
                let items = items
                    .iter()
                    .map(|item| self.format_pattern(item))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({items})")
            }
            PatternKind::Array {
                starting,
                glob,
                ending,
            } => {
                let mut parts = Vec::new();
                parts.extend(starting.iter().map(|item| self.format_pattern(item)));
                match glob {
                    Glob::None => {}
                    Glob::Anonymous => parts.push("..".to_string()),
                    Glob::Named(path) => parts.push(format!("..{}", self.format_path(path))),
                }
                parts.extend(ending.iter().map(|item| self.format_pattern(item)));
                format!("[{}]", parts.join(", "))
            }
            PatternKind::Struct(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, value)| format!("{name} = {}", self.format_pattern(value)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {fields} }}")
            }
            PatternKind::Immediate(value) => value.to_string(),
            PatternKind::TypeHint(inner, type_expr) => {
                let inner = self.wrap_pattern(&self.format_pattern(inner));
                let type_expr = self.format_type_expr(type_expr);
                format!("{inner} : {type_expr}")
            }
        }
    }

    fn format_term_inline(
        &self,
        term: &Term<Type>,
    ) -> Option<String> {
        let expr = self.format_term_inline_expr(term)?;
        let line = format!("{expr} : {}", term.type_.pretty());
        (line.len() <= LINE_LIMIT).then_some(line)
    }

    fn format_term_wrapped_expr(
        &self,
        term: &Term<Type>,
    ) -> Option<String> {
        let expr = self.format_term_inline_expr(term)?;
        let expr = self.wrap_inline_expr(&expr);
        (expr.len() <= LINE_LIMIT).then_some(expr)
    }

    fn format_term_inline_expr(
        &self,
        term: &Term<Type>,
    ) -> Option<String> {
        match &term.kind {
            TermKind::Immediate(value) => Some(value.to_string()),
            TermKind::Identifier(path) => Some(self.format_path(path)),
            TermKind::Tuple(items) => {
                let items = items
                    .iter()
                    .map(|item| self.format_term_inline_expr(item))
                    .collect::<Option<Vec<_>>>()?;
                Some(format!("({})", items.join(", ")))
            }
            TermKind::Struct(fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, value)| {
                        Some(format!("{name} = {}", self.format_term_inline_expr(value)?))
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(format!("{{ {} }}", fields.join(", ")))
            }
            TermKind::Field { of, index } => {
                let base = self.format_term_wrapped_expr(of)?;
                Some(format!("{base}.{index}"))
            }
            TermKind::Call { callee, argument } => {
                let callee = self.format_term_wrapped_expr(callee)?;
                let argument = self.format_term_wrapped_expr(argument)?;
                Some(format!("{callee} {argument}"))
            }
            TermKind::Semicolon(left, right) => {
                let left = self.format_term_wrapped_expr(left)?;
                let right = self.format_term_wrapped_expr(right)?;
                Some(format!("{left}; {right}"))
            }
            TermKind::Unreachable => Some("unreachable".to_string()),
            TermKind::Let { .. } | TermKind::Function { .. } | TermKind::InlineWasm { .. } => None,
        }
    }

    fn format_path(
        &self,
        path: &Path,
    ) -> String {
        match &self.module_name {
            Some(module) if module == &path.major => path.minor.clone(),
            _ => path.to_string(),
        }
    }

    fn wrap_pattern(
        &self,
        value: &str,
    ) -> String {
        if value.contains(' ') || value.contains(':') || value.contains('|') {
            format!("({value})")
        } else {
            value.to_string()
        }
    }

    fn wrap_type_expr(
        &self,
        value: &str,
    ) -> String {
        if value.contains(' ') || value.contains("->") || value.contains('|') {
            format!("({value})")
        } else {
            value.to_string()
        }
    }

    fn wrap_inline_expr(
        &self,
        expr: &str,
    ) -> String {
        let trimmed = expr.trim();
        if trimmed.is_empty() {
            return "()".to_string();
        }
        if is_inline_atom(trimmed) {
            trimmed.to_string()
        } else {
            format!("({trimmed})")
        }
    }
}

fn is_unreachable(term: &Term<Type>) -> bool {
    matches!(term.kind, TermKind::Unreachable)
}

fn match_view<'a>(term: &'a Term<Type>) -> Option<MatchView<'a>> {
    let TermKind::Let {
        assignee,
        scope: ScopeKind::Local,
        value,
        then,
        else_,
    } = &term.kind
    else {
        return None;
    };
    let PatternKind::Identifier(scrutinee_path) = &assignee.kind else {
        return None;
    };
    if !is_unreachable(else_) {
        return None;
    }
    let mut arms = Vec::new();
    let mut current = then.as_ref();
    loop {
        let TermKind::Let {
            assignee,
            scope: ScopeKind::Local,
            value,
            then,
            else_,
        } = &current.kind
        else {
            return None;
        };
        let TermKind::Identifier(candidate) = &value.kind else {
            return None;
        };
        if candidate != scrutinee_path {
            return None;
        }
        arms.push(MatchArm {
            pattern: assignee,
            body: then,
        });
        if is_unreachable(else_) {
            break;
        }
        current = else_.as_ref();
    }
    if arms.len() <= 1 {
        return None;
    }
    Some(MatchView {
        scrutinee: value,
        arms,
    })
}

fn is_inline_atom(expr: &str) -> bool {
    if expr.starts_with('(') || expr.starts_with('{') || expr.starts_with('[') {
        return true;
    }
    if expr.contains(' ') || expr.contains(';') || expr.contains("=>") {
        return false;
    }
    expr.chars().all(|ch| {
        ch.is_ascii_alphanumeric()
            || matches!(
                ch,
                '_' | ':' | '#' | '-' | '+' | '*' | '/' | '%' | '<' | '>' | '=' | '!'
            )
            || ch == '.'
            || ch == '\''
            || ch == '"'
    })
}

fn is_sexpr_ident(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !is_sexpr_ident_start(first) {
        return false;
    }
    chars.all(is_sexpr_ident_continue)
}

fn is_sexpr_ident_start(ch: char) -> bool {
    (!ch.is_ascii_punctuation() || ch == '_') && !ch.is_whitespace()
}

fn is_sexpr_ident_continue(ch: char) -> bool {
    (!ch.is_ascii_punctuation() || ch == '_' || ch == '-') && !ch.is_whitespace()
}

fn escape_sexpr_string(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\x08' => escaped.push_str("\\b"),
            '\0' => escaped.push_str("\\0"),
            c if c.is_control() => {
                escaped.push_str(&format!("\\x{code:02x}", code = c as u32));
            }
            c => escaped.push(c),
        }
    }
    escaped
}

fn escape_sexpr_glyph(value: char) -> String {
    match value {
        '\\' => "\\\\".to_string(),
        '\'' => "\\'".to_string(),
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        '\x08' => "\\b".to_string(),
        '\0' => "\\0".to_string(),
        c if c.is_control() => format!("\\x{code:02x}", code = c as u32),
        c => c.to_string(),
    }
}
