use crate::ir::{
    Glob,
    ImmediateValue,
    Pattern,
    PatternKind,
    Term,
    TermKind,
};
use crate::types::{
    Type as SemanticType,
    TypeDefinition,
    TypeScheme,
    ordered_trait_methods,
};

use super::*;

/// Handles lower type.
pub fn lower_type(
    type_: &SemanticType,
    symbols: &SymbolTable,
) -> Type {
    use SemanticType::*;
    match type_ {
        Unit => Type::Struct([].into()),
        Integer | Natural => Type::Struct([Type::I64].into()),
        Real => Type::Struct([Type::F64].into()),
        Glyph | Boolean => Type::Struct([Type::I32].into()),
        String => Type::Array(Type::I8.into()),
        TypeVar(_) | MetaVar(_) => Type::Any,
        ForAll { body, .. } => lower_type(body, symbols),
        Named { name, body } => lower_type(&resolve_named_body(name, body, symbols), symbols),
        StructConstraint { fields, .. } => {
            Type::Struct(fields.values().map(|v| lower_type(v, symbols)).collect())
        }
        Struct { fields } => {
            Type::Struct(fields.values().map(|v| lower_type(v, symbols)).collect())
        }
        Array(_) => Type::Array(Type::Any.into()),
        Tuple(items) => Type::Struct(items.iter().map(|item| lower_type(item, symbols)).collect()),
        Sum { .. } => Type::Struct([Type::I32, Type::Any].into()),
        Function(..) => Type::closure_type(),
        Apply {
            constructor,
            arguments,
        } => {
            if let Some(applied) = apply_type(constructor, arguments, symbols) {
                lower_type(&applied, symbols)
            } else {
                Type::Any
            }
        }
    }
}

use Instruction as i;
use indexmap::IndexMap;
impl<'a> Encoder<'a> {
    /// Create a new closure, push a reference to it onto the stack
    pub fn create_closure(
        &mut self,
        symbols: &SymbolTable,
        parameter: Path,
        parameter_type: SemanticType,
        captures: Vec<(Path, SemanticType)>,
        recursive_binding: Option<Path>,
        body: impl for<'b> FnOnce(&mut Encoder<'b>, &SymbolTable),
    ) -> Path {
        let id = self.module.closure_counter;
        self.module.closure_counter += 1;
        let func_name = Path::new("[temp]", format!("closure#{id}"));
        let has_recursive_capture = recursive_binding.as_ref().is_some_and(|binding| {
            captures
                .iter()
                .any(|(capture_name, _)| capture_name == binding)
        });
        let captures_for_env = captures
            .iter()
            .filter(|(capture_name, _)| {
                recursive_binding
                    .as_ref()
                    .is_none_or(|binding| capture_name != binding)
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut new_enc = self.module.new_function(func_name.clone());
        let capture_array_name = new_enc.temporary_name("captured_symbols");
        new_enc.new_parameter(capture_array_name.clone(), Type::Array(Type::Any.into()));
        let parameter_type = lower_type(&parameter_type, symbols);
        let param_anyref_name = new_enc.temporary_name("parameter");
        new_enc.new_parameter(param_anyref_name.clone(), Type::Any);
        new_enc.new_return(Type::Any);
        // Cast the anyref parameter to the actual type and bind to the user's name
        new_enc.new_register(parameter.clone(), ScopeKind::Local, parameter_type.clone());
        new_enc.push(i::Get(param_anyref_name));
        new_enc.ref_cast_if_needed(&parameter_type);
        new_enc.push(i::Set(parameter));
        for (id, (capture_name, capture_type)) in captures_for_env.clone().into_iter().enumerate() {
            let capture_type = lower_type(&capture_type, symbols);
            new_enc.new_register(capture_name.clone(), ScopeKind::Local, capture_type.clone());
            new_enc.extend([
                i::Get(capture_array_name.clone()),
                i::I32Const(id as i32),
                i::ArrayGet(Type::Any),
            ]);
            // Cast anyref from array to the actual capture type
            new_enc.ref_cast_if_needed(&capture_type);
            new_enc.push(i::Set(capture_name.clone()));
        }
        if has_recursive_capture {
            let recursive_binding = recursive_binding.unwrap_or_else(|| unreachable!());
            new_enc.new_register(
                recursive_binding.clone(),
                ScopeKind::Local,
                Type::closure_type(),
            );
            new_enc.extend([
                i::Get(capture_array_name.clone()),
                i::Func(func_name.clone()),
                i::StructNew([Type::function_capture(), Type::closure_function_type()].into()),
                i::Set(recursive_binding),
            ]);
        }
        body(&mut new_enc, symbols);
        let num_captures = captures_for_env.len();
        for (capture, _) in captures_for_env {
            self.push(i::Get(capture));
        }
        self.extend([
            i::ArrayNewFixed {
                inner_type: Type::Any,
                length: num_captures,
            },
            i::Func(func_name.clone()),
            i::StructNew([Type::function_capture(), Type::closure_function_type()].into()),
        ]);
        func_name
    }
    // Preconditions:
    // * Predicate to be pattern-matched on is top of stack
    // * A br 0 instruction indicates pattern matching has failed
    /// Handles lower pattern.
    pub(crate) fn lower_pattern(
        &mut self,
        pat: Pattern<SemanticType>,
        scope: ScopeKind,
        symbols: &SymbolTable,
        constructors: &ConstructorTable,
    ) {
        let Pattern {
            kind, type_, span, ..
        } = pat;
        let previous_origin = self.current_origin.clone();
        if let Some(origin) = self.module.source_origin_for_span(span) {
            self.current_origin = Some(origin);
        }
        let lowered_type = lower_type(&type_, symbols);
        match kind {
            PatternKind::Hole => {
                self.push(i::Drop);
            }
            PatternKind::Identifier(path) => {
                self.ref_cast_if_needed(&lowered_type);
                self.push(i::Set(path));
            }
            PatternKind::Tuple(items) => {
                let temporary = self.temporary_name("pattern");
                let SemanticType::Tuple(_) = &type_ else {
                    unreachable!()
                };
                let Type::Struct(types) = lowered_type.clone() else {
                    unreachable!()
                };
                self.new_register(temporary.clone(), scope, lowered_type.clone());
                self.ref_cast_if_needed(&lowered_type);
                self.push(i::Set(temporary.clone()));
                for (index, item) in items.into_iter().enumerate() {
                    self.extend([
                        i::Get(temporary.clone()),
                        i::StructGet(types.clone(), index),
                    ]);
                    self.lower_pattern(item, scope, symbols, constructors);
                }
            }
            PatternKind::Array {
                starting,
                glob,
                ending,
            } => {
                let SemanticType::Array(_) = &type_ else {
                    unreachable!()
                };
                let Type::Array(inner_type_lowered) = lowered_type.clone() else {
                    unreachable!()
                };
                let inner_type_lowered = *inner_type_lowered;
                let temporary = self.temporary_name("array_pattern");
                self.new_register(temporary.clone(), scope, lowered_type.clone());
                self.ref_cast_if_needed(&lowered_type);
                self.push(i::Set(temporary.clone()));

                let start_len = starting.len() as i32;
                let end_len = ending.len() as i32;
                let min_len = start_len + end_len;

                let cmp_op = if matches!(glob, Glob::None) {
                    NumberOperation::Ne
                } else {
                    NumberOperation::Lt
                };
                self.extend([
                    i::Get(temporary.clone()),
                    i::ArrayLen,
                    i::I32Const(min_len),
                    i::I32Op(cmp_op),
                    i::BreakIf(0),
                ]);

                // Match starting patterns at indices 0, 1, 2, ...
                for (index, pattern) in starting.into_iter().enumerate() {
                    self.push(i::Get(temporary.clone()));
                    self.push(i::I32Const(index as i32));
                    self.push(i::ArrayGet(inner_type_lowered.clone()));
                    self.lower_pattern(pattern, scope, symbols, constructors);
                }

                // Compute middle_len and capture glob if needed
                let middle_len_var = if matches!(glob, Glob::None) {
                    None
                } else {
                    let var = self.temporary_name("middle_len");
                    self.new_register(var.clone(), scope, Type::I32);
                    self.extend([
                        i::Get(temporary.clone()),
                        i::ArrayLen,
                        i::I32Const(min_len),
                        i::I32Op(NumberOperation::Sub),
                        i::Set(var.clone()),
                    ]);

                    // Capture glob slice if named
                    if let Glob::Named(glob_name) = &glob {
                        let new_array = self.temporary_name("slice");
                        self.new_register(new_array.clone(), scope, lowered_type.clone());
                        self.extend([
                            i::Get(var.clone()),
                            i::ArrayNewDefault(inner_type_lowered.clone()),
                            i::Set(new_array.clone()),
                        ]);
                        // Copy elements: [dst, dst_offset, src, src_offset, length]
                        self.extend([
                            i::Get(new_array.clone()),
                            i::I32Const(0),
                            i::Get(temporary.clone()),
                            i::I32Const(start_len),
                            i::Get(var.clone()),
                            i::ArrayCopy {
                                dst_type: inner_type_lowered.clone(),
                                src_type: inner_type_lowered.clone(),
                            },
                        ]);
                        // Bind to glob name
                        self.extend([i::Get(new_array), i::Set(glob_name.clone())]);
                    }

                    Some(var)
                };

                // Match ending patterns at (start_len + middle_len + index)
                for (index, pattern) in ending.into_iter().enumerate() {
                    self.push(i::Get(temporary.clone()));
                    if let Some(ref mlv) = middle_len_var {
                        // Dynamic offset: start_len + middle_len + index
                        self.extend([
                            i::I32Const(start_len),
                            i::Get(mlv.clone()),
                            i::I32Op(NumberOperation::Add),
                            i::I32Const(index as i32),
                            i::I32Op(NumberOperation::Add),
                        ]);
                    } else {
                        // Static offset (exact match): start_len + index
                        self.push(i::I32Const(start_len + index as i32));
                    }
                    self.push(i::ArrayGet(inner_type_lowered.clone()));
                    self.lower_pattern(pattern, scope, symbols, constructors);
                }
            }
            PatternKind::Struct(index_map) => {
                let temporary = self.temporary_name("pattern");
                self.new_register(temporary.clone(), scope, lowered_type.clone());
                self.ref_cast_if_needed(&lowered_type);
                self.push(i::Set(temporary.clone()));
                let ordered_fields = struct_fields_for_type(&type_, symbols).unwrap_or_else(|| {
                    index_map
                        .iter()
                        .map(|(name, pattern)| (name.inner.clone(), pattern.type_.clone()))
                        .collect()
                });
                let types = ordered_fields
                    .values()
                    .map(|t| lower_type(t, symbols))
                    .collect::<Box<[Type]>>();
                for (name, pattern) in index_map {
                    let index = ordered_fields
                        .get_index_of(&name.inner)
                        .unwrap_or_else(|| unreachable!());
                    self.extend([
                        i::Get(temporary.clone()),
                        i::StructGet(types.clone(), index),
                    ]);
                    self.lower_pattern(pattern, scope, symbols, constructors);
                }
            }
            PatternKind::ConstConstructor(path) => {
                let info = constructors
                    .get(&path)
                    .unwrap_or_else(|| unreachable!("Unknown constructor: {path}"));
                match &info.kind {
                    ConstructorKind::SumVariant { tag, .. } => {
                        let sum_type: Box<[Type]> = [Type::I32, Type::Any].into();
                        let temporary = self.temporary_name("constructor_pattern");
                        self.new_register(temporary.clone(), scope, lowered_type.clone());
                        self.ref_cast_if_needed(&lowered_type);
                        self.push(i::Set(temporary.clone()));
                        self.extend([
                            i::Get(temporary),
                            i::StructGet(sum_type.clone(), 0),
                            i::I32Const(*tag as i32),
                            i::I32Op(NumberOperation::Ne),
                            i::BreakIf(0),
                        ]);
                    }
                    ConstructorKind::Wrap { .. } => {
                        self.ref_cast_if_needed(&lowered_type);
                        self.push(i::Drop);
                    }
                }
            }
            PatternKind::Constructor(path, inner) => {
                let info = constructors
                    .get(&path)
                    .unwrap_or_else(|| unreachable!("Unknown constructor: {path}"));
                match &info.kind {
                    ConstructorKind::SumVariant { tag, .. } => {
                        let sum_type: Box<[Type]> = [Type::I32, Type::Any].into();

                        let temporary = self.temporary_name("constructor_pattern");
                        self.new_register(temporary.clone(), scope, lowered_type.clone());
                        self.ref_cast_if_needed(&lowered_type);
                        self.push(i::Set(temporary.clone()));

                        self.extend([
                            i::Get(temporary.clone()),
                            i::StructGet(sum_type.clone(), 0),
                            i::I32Const(*tag as i32),
                            i::I32Op(NumberOperation::Ne),
                            i::BreakIf(0),
                        ]);

                        self.extend([i::Get(temporary), i::StructGet(sum_type, 1)]);
                        let inner_type = lower_type(&inner.type_, symbols);
                        self.ref_cast_if_needed(&inner_type);
                        self.lower_pattern(*inner, scope, symbols, constructors);
                    }
                    ConstructorKind::Wrap { .. } => {
                        let inner_type = lower_type(&inner.type_, symbols);
                        self.ref_cast_if_needed(&inner_type);
                        self.lower_pattern(*inner, scope, symbols, constructors);
                    }
                }
            }
            PatternKind::Immediate(const_value) => {
                match const_value {
                    ImmediateValue::Unit => {
                        self.push(i::Drop);
                    }
                    ImmediateValue::Integer(value) => {
                        let temp = self.temporary_name("const_pattern");
                        let fields = lowered_struct_fields(&SemanticType::Integer, symbols)
                            .unwrap_or_else(|| unreachable!());
                        self.new_register(temp.clone(), scope, lowered_type.clone());
                        self.ref_cast_if_needed(&lowered_type);
                        self.push(i::Set(temp.clone()));
                        self.extend([
                            i::Get(temp),
                            i::StructGet(fields.clone(), 0),
                            i::Const(ImmediateValue::Integer(value)),
                            i::I64Op(NumberOperation::Eq),
                            i::I32Const(1),
                            i::I32Op(NumberOperation::Xor),
                            i::BreakIf(0),
                        ]);
                    }
                    ImmediateValue::Natural(value) => {
                        let temp = self.temporary_name("const_pattern");
                        let fields = lowered_struct_fields(&SemanticType::Natural, symbols)
                            .unwrap_or_else(|| unreachable!());
                        self.new_register(temp.clone(), scope, lowered_type.clone());
                        self.ref_cast_if_needed(&lowered_type);
                        self.push(i::Set(temp.clone()));
                        self.extend([
                            i::Get(temp),
                            i::StructGet(fields.clone(), 0),
                            i::Const(ImmediateValue::Natural(value)),
                            i::I64Op(NumberOperation::Eq),
                            i::I32Const(1),
                            i::I32Op(NumberOperation::Xor),
                            i::BreakIf(0),
                        ]);
                    }
                    ImmediateValue::Real(value) => {
                        let temp = self.temporary_name("const_pattern");
                        let fields = lowered_struct_fields(&SemanticType::Real, symbols)
                            .unwrap_or_else(|| unreachable!());
                        self.new_register(temp.clone(), scope, lowered_type.clone());
                        self.ref_cast_if_needed(&lowered_type);
                        self.push(i::Set(temp.clone()));
                        self.extend([
                            i::Get(temp),
                            i::StructGet(fields.clone(), 0),
                            i::Const(ImmediateValue::Real(value)),
                            i::F64Op(NumberOperation::Eq),
                            i::I32Const(1),
                            i::I32Op(NumberOperation::Xor),
                            i::BreakIf(0),
                        ]);
                    }
                    ImmediateValue::Boolean(value) => {
                        let temp = self.temporary_name("const_pattern");
                        let fields = lowered_struct_fields(&SemanticType::Boolean, symbols)
                            .unwrap_or_else(|| unreachable!());
                        self.new_register(temp.clone(), scope, lowered_type.clone());
                        self.ref_cast_if_needed(&lowered_type);
                        self.push(i::Set(temp.clone()));
                        self.extend([
                            i::Get(temp),
                            i::StructGet(fields.clone(), 0),
                            i::I32Const(i32::from(value)),
                            i::I32Op(NumberOperation::Eq),
                            i::I32Const(1),
                            i::I32Op(NumberOperation::Xor),
                            i::BreakIf(0),
                        ]);
                    }
                    ImmediateValue::Glyph(value) => {
                        let temp = self.temporary_name("const_pattern");
                        let fields = lowered_struct_fields(&SemanticType::Glyph, symbols)
                            .unwrap_or_else(|| unreachable!());
                        self.new_register(temp.clone(), scope, lowered_type.clone());
                        self.ref_cast_if_needed(&lowered_type);
                        self.push(i::Set(temp.clone()));
                        self.extend([
                            i::Get(temp),
                            i::StructGet(fields.clone(), 0),
                            i::I32Const(value as i32),
                            i::I32Op(NumberOperation::Eq),
                            i::I32Const(1),
                            i::I32Op(NumberOperation::Xor),
                            i::BreakIf(0),
                        ]);
                    }
                    ImmediateValue::String(value) => {
                        let temp = self.temporary_name("const_pattern");
                        let const_string = self.temporary_name("const_string");
                        let bool_fields = bool_fields(symbols);
                        let array_type = lower_type(&SemanticType::String, symbols);
                        self.new_register(temp.clone(), scope, lowered_type.clone());
                        self.new_register(const_string.clone(), scope, array_type);
                        self.ref_cast_if_needed(&lowered_type);
                        self.push(i::Set(temp.clone()));
                        self.extend(value.bytes().map(|b| i::I32Const(b as i32)));
                        self.push(i::ArrayNewFixed {
                            inner_type: Type::I8,
                            length: value.len(),
                        });
                        self.push(i::Set(const_string.clone()));
                        emit_string_compare(
                            self,
                            &temp,
                            &const_string,
                            NumberOperation::Eq,
                            bool_fields.clone(),
                        );
                        self.extend([
                            i::StructGet(bool_fields.clone(), 0),
                            i::I32Const(1),
                            i::I32Op(NumberOperation::Xor),
                            i::BreakIf(0),
                        ]);
                    }
                }
            }
            PatternKind::TypeHint(inner, _) => {
                self.lower_pattern(*inner, scope, symbols, constructors);
            }
        }
        self.current_origin = previous_origin;
    }
    /// Handles lower ir.
    pub(crate) fn lower_ir(
        &mut self,
        term: Term<SemanticType>,
        symbols: &SymbolTable,
        constructors: &ConstructorTable,
    ) {
        let Term {
            kind, type_, span, ..
        } = term;
        let previous_origin = self.current_origin.clone();
        if let Some(origin) = self.module.source_origin_for_span(span) {
            self.current_origin = Some(origin);
        }
        match kind {
            TermKind::Let {
                assignee,
                scope,
                value,
                then,
                else_,
            } => {
                let skip_pattern =
                    !pattern_is_refutable(&assignee) && pattern_introduced_names(&assignee) == 0;
                for (path, type_) in collect_pattern_bindings(&assignee) {
                    self.new_register(path, scope, lower_type(&type_, symbols));
                }
                let result_type = lower_type(&type_, symbols);
                if !skip_pattern {
                    self.extend([i::Block(Some(result_type)), i::Block(None)]);
                }
                let recursive_binding = match (&assignee.kind, &value.kind) {
                    (PatternKind::Identifier(path), TermKind::Function { .. }) => {
                        Some(path.clone())
                    }
                    _ => None,
                };
                let previous_recursive_binding =
                    std::mem::replace(&mut self.recursive_binding, recursive_binding);
                self.lower_ir(*value, symbols, constructors);
                self.recursive_binding = previous_recursive_binding;
                if !skip_pattern {
                    self.lower_pattern(assignee, scope, symbols, constructors);
                } else {
                    self.push(i::Drop);
                }
                self.lower_ir(*then, symbols, constructors);
                if !skip_pattern {
                    self.extend([i::Break(1), i::End]);
                    self.lower_ir(*else_, symbols, constructors);
                    self.push(i::End);
                }
            }
            TermKind::Immediate(ImmediateValue::String(s)) => {
                self.extend(s.bytes().map(|b| i::I32Const(b as i32)));
                self.push(i::ArrayNewFixed {
                    inner_type: Type::I8,
                    length: s.len(),
                })
            }
            TermKind::Immediate(const_value) => {
                let Type::Struct(inner_types) = lower_type(&const_value.type_of(), symbols) else {
                    unreachable!()
                };
                self.extend([i::Const(const_value), i::StructNew(inner_types)]);
            }
            TermKind::Identifier(path) => {
                let result_type = lower_type(&type_, symbols);
                if path.major != self.module.name {
                    let is_trait_dispatch_symbol = symbols
                        .trait_defs()
                        .values()
                        .any(|def| def.methods.contains_key(&path));
                    let scheme_type = symbols
                        .terms()
                        .get(&path)
                        .map(|scheme| lower_type(&scheme.type_, symbols));
                    let import_type = if is_trait_dispatch_symbol {
                        Type::closure_type()
                    } else {
                        match scheme_type {
                            Some(Type::Any) => result_type.clone(),
                            Some(type_) => type_,
                            None => result_type.clone(),
                        }
                    };
                    self.module
                        .imports
                        .entry(path.clone())
                        .or_insert_with(|| import_type);
                }
                self.push(i::Get(path));
                self.ref_cast_if_needed(&result_type);
            }
            TermKind::Tuple(items) => {
                let types = items
                    .iter()
                    .map(|i| lower_type(&i.type_, symbols))
                    .collect::<Box<[Type]>>();
                items
                    .into_iter()
                    .for_each(|i| self.lower_ir(i, symbols, constructors));
                self.push(i::StructNew(types));
            }
            TermKind::Struct(index_map) => {
                let ordered_fields = struct_fields_for_type(&type_, symbols).unwrap_or_else(|| {
                    index_map
                        .iter()
                        .map(|(name, value)| (name.inner.clone(), value.type_.clone()))
                        .collect()
                });
                let mut field_temporaries = IndexMap::new();
                for (field_name, field_ir) in index_map {
                    let temp_name = self.temporary_name(&field_name.inner);
                    let temp_type = lower_type(&field_ir.type_, symbols);
                    self.new_register(temp_name.clone(), ScopeKind::Local, temp_type);
                    self.lower_ir(field_ir, symbols, constructors);
                    self.push(i::Set(temp_name.clone()));
                    field_temporaries.insert(field_name.inner, temp_name);
                }
                let ordered_types = ordered_fields
                    .values()
                    .map(|t| lower_type(t, symbols))
                    .collect::<Box<[Type]>>();
                for field_name in ordered_fields.keys() {
                    let temp_name = &field_temporaries[field_name];
                    self.push(i::Get(temp_name.clone()));
                }
                self.push(i::StructNew(ordered_types));
            }
            TermKind::Field { of, index } => {
                let ordered_fields =
                    struct_fields_for_type(&of.type_, symbols).unwrap_or_else(|| {
                        unreachable!("Missing fields are caught during typechecking")
                    });
                let field_index = ordered_fields
                    .keys()
                    .position(|k| k == &index.inner)
                    .unwrap_or_else(|| {
                        unreachable!("Missing fields are caught during typechecking")
                    });
                let inner_types = ordered_fields
                    .values()
                    .map(|t| lower_type(t, symbols))
                    .collect::<Box<[Type]>>();
                self.lower_ir(*of, symbols, constructors);
                self.push(i::StructGet(inner_types, field_index));
                let result_type = lower_type(&type_, symbols);
                self.ref_cast_if_needed(&result_type);
            }
            TermKind::Function {
                parameter_name,
                captures,
                body,
                ..
            } => {
                let SemanticType::Function(parameter_type, _) = type_ else {
                    unreachable!("expected function type during lowering, got `{type_}`")
                };
                let captures = Vec::from(captures);
                self.create_closure(
                    symbols,
                    parameter_name.inner.clone(),
                    *parameter_type,
                    captures,
                    self.recursive_binding.clone(),
                    |new_enc: &mut Encoder<'_>, symbols: &SymbolTable| {
                        new_enc.lower_ir(*body, symbols, constructors);
                    },
                );
            }
            TermKind::InlineWasm {
                definitions,
                instructions,
                ..
            } => {
                for (name, type_) in definitions {
                    self.new_register(name, ScopeKind::Local, type_);
                }
                self.extend(instructions);
            }
            TermKind::Call { callee, argument } => {
                let callee_name = self.temporary_name("callee");
                self.new_register(callee_name.clone(), ScopeKind::Local, Type::Any);
                self.lower_ir(*callee, symbols, constructors);
                self.push(i::Set(callee_name.clone()));
                self.lower_ir(*argument, symbols, constructors);
                self.push(i::Get(callee_name));
                self.call_closure();
                let result_type = lower_type(&type_, symbols);
                self.ref_cast_if_needed(&result_type);
            }
            TermKind::Semicolon(a, b) => {
                self.lower_ir(*a, symbols, constructors);
                self.push(i::Drop);
                self.lower_ir(*b, symbols, constructors);
            }
            TermKind::Unreachable => {
                self.push(i::Unreachable);
            }
        }
        self.current_origin = previous_origin;
    }

    /// Handles lower constructor.
    pub(crate) fn lower_constructor(
        &mut self,
        path: Path,
        info: &ConstructorInfo,
        symbols: &SymbolTable,
    ) {
        match &info.kind {
            ConstructorKind::SumVariant { tag, payload } => {
                let sum_struct_type: Box<[Type]> = [Type::I32, Type::Any].into();
                if payload.is_none() {
                    self.new_register(
                        path.clone(),
                        ScopeKind::Global,
                        Type::Struct(sum_struct_type.clone()),
                    );
                    self.extend([
                        i::I32Const(*tag as i32),
                        i::StructNew([].into()),       // Create unit value ()
                        i::StructNew(sum_struct_type), // Create sum struct { tag, value }
                        i::Set(path),
                    ]);
                } else {
                    let parameter_type = payload.clone().unwrap_or_else(|| SemanticType::Unit);
                    let parameter_name = self.temporary_name("cons_param");
                    self.create_closure(
                        symbols,
                        parameter_name.clone(),
                        parameter_type,
                        vec![],
                        None,
                        |inner_func: &mut Encoder<'_>, _symbols: &SymbolTable| {
                            inner_func.extend([
                                i::I32Const(*tag as i32),
                                i::Get(parameter_name),
                                i::StructNew(sum_struct_type.clone()),
                            ]);
                        },
                    );

                    self.new_register(path.clone(), ScopeKind::Global, Type::closure_type());
                    self.extend([i::Set(path)]);
                }
            }
            ConstructorKind::Wrap { payload } => {
                if payload.is_none() {
                    let register_type = symbols
                        .terms()
                        .get(&path)
                        .map(|scheme| lower_type(&scheme.type_, symbols))
                        .unwrap_or(Type::Any);
                    self.new_register(path.clone(), ScopeKind::Global, register_type);
                    self.extend([i::StructNew([].into()), i::Set(path)]);
                } else {
                    let parameter_type = payload.clone().unwrap_or_else(|| SemanticType::Unit);
                    let parameter_name = self.temporary_name("wrap_param");
                    self.create_closure(
                        symbols,
                        parameter_name.clone(),
                        parameter_type,
                        vec![],
                        None,
                        |inner_func: &mut Encoder<'_>, _symbols: &SymbolTable| {
                            inner_func.extend([i::Get(parameter_name)]);
                        },
                    );
                    self.new_register(path.clone(), ScopeKind::Global, Type::closure_type());
                    self.extend([i::Set(path)]);
                }
            }
        }
    }

    /// Handles lower constructor alias.
    pub(crate) fn lower_constructor_alias(
        &mut self,
        path: Path,
        target: Path,
        symbols: &SymbolTable,
    ) {
        let register_type = symbols
            .terms()
            .get(&path)
            .map(|scheme| lower_type(&scheme.type_, symbols))
            .unwrap_or(Type::Any);
        self.new_register(path.clone(), ScopeKind::Global, register_type);
        self.extend([i::Get(target), i::Set(path)]);
    }

    /// Handles lower trait method dispatch.
    pub(crate) fn lower_trait_method_dispatch(
        &mut self,
        method_path: Path,
        symbols: &SymbolTable,
    ) {
        let Some((methods, method_index)) = ordered_trait_methods_for_path(symbols, &method_path)
        else {
            return;
        };
        let dict_type = SemanticType::Struct {
            fields: methods
                .iter()
                .map(|(path, scheme)| (path.minor.clone(), scheme.type_.clone()))
                .collect(),
        };
        let dict_fields = match lower_type(&dict_type, symbols) {
            Type::Struct(fields) => fields,
            _ => unreachable!(),
        };
        let dict_name = self.temporary_name("dict");
        self.create_closure(
            symbols,
            dict_name.clone(),
            dict_type,
            vec![],
            None,
            move |inner, _symbols| {
                inner.extend([
                    i::Get(dict_name.clone()),
                    i::StructGet(dict_fields.clone(), method_index),
                ]);
            },
        );
        self.new_register(method_path.clone(), ScopeKind::Global, Type::closure_type());
        self.push(i::Set(method_path));
    }

    /// [argument, closure] -> [result]
    ///
    /// A closure is a struct with fields {captured_args, funcref}.
    /// The calling convention expects [captures, argument] on the stack before funcref.
    pub fn call_closure(&mut self) {
        let closure_struct_type: Box<[Type]> =
            [Type::function_capture(), Type::closure_function_type()].into();

        // Stack before: [argument, closure: anyref]
        // Cast closure from anyref to the specific closure struct type
        self.push(i::RefCastStruct(closure_struct_type.clone()));
        // Stack: [argument, closure: (ref null closure_struct)]

        // Save closure to local
        let closure_name = self.temporary_name("closure");
        self.new_register(
            closure_name.clone(),
            ScopeKind::Local,
            Type::Struct(closure_struct_type.clone()),
        );
        self.push(i::Set(closure_name.clone()));
        // Stack: [argument]

        // Save argument to local
        let argument_name = self.temporary_name("argument");
        self.new_register(argument_name.clone(), ScopeKind::Local, Type::Any);
        self.push(i::Set(argument_name.clone()));
        // Stack: []

        let func_params: Box<[Type]> = [Type::function_capture(), Type::Any].into();
        let func_returns: Box<[Type]> = [Type::Any].into();

        // Push captures (first param)
        self.extend([
            i::Get(closure_name.clone()),
            i::StructGet(closure_struct_type.clone(), 0),
        ]);
        // Stack: [captures]

        // Push argument (second param)
        self.push(i::Get(argument_name));
        // Stack: [captures, argument]

        // Get funcref and cast to the expected type
        self.extend([
            i::Get(closure_name),
            i::StructGet(closure_struct_type, 1),
            i::RefCastFunc {
                parameters: func_params.clone(),
                returns: func_returns.clone(),
            },
        ]);
        // Stack: [captures, argument, typed_funcref]

        self.push(i::CallRef {
            parameters: func_params,
            returns: func_returns,
        });
        // Stack: [result: anyref]
    }

    /// Handles ref cast if needed.
    fn ref_cast_if_needed(
        &mut self,
        type_: &Type,
    ) {
        match type_ {
            Type::Struct(fields) => self.push(i::RefCastStruct(fields.clone())),
            Type::Array(inner) => self.push(i::RefCastArray(inner.clone())),
            Type::Any => {}
            _ => {}
        }
    }
}

/// Handles lowered struct fields.
fn lowered_struct_fields(
    type_: &SemanticType,
    symbols: &SymbolTable,
) -> Option<Box<[Type]>> {
    match lower_type(type_, symbols) {
        Type::Struct(fields) => Some(fields),
        _ => None,
    }
}

/// Handles bool fields.
fn bool_fields(symbols: &SymbolTable) -> Box<[Type]> {
    match lower_type(&SemanticType::Boolean, symbols) {
        Type::Struct(fields) => fields,
        _ => unreachable!(),
    }
}

/// Handles emit string compare.
pub(crate) fn emit_string_compare(
    encoder: &mut Encoder<'_>,
    left: &Path,
    right: &Path,
    op: NumberOperation,
    bool_fields: Box<[Type]>,
) {
    let left_len = encoder.temporary_name("left_len");
    let right_len = encoder.temporary_name("right_len");
    let min_len = encoder.temporary_name("min_len");
    let index = encoder.temporary_name("index");
    let cmp = encoder.temporary_name("cmp");
    let left_byte = encoder.temporary_name("left_byte");
    let right_byte = encoder.temporary_name("right_byte");

    encoder.new_register(left_len.clone(), ScopeKind::Local, Type::I32);
    encoder.new_register(right_len.clone(), ScopeKind::Local, Type::I32);
    encoder.new_register(min_len.clone(), ScopeKind::Local, Type::I32);
    encoder.new_register(index.clone(), ScopeKind::Local, Type::I32);
    encoder.new_register(cmp.clone(), ScopeKind::Local, Type::I32);
    encoder.new_register(left_byte.clone(), ScopeKind::Local, Type::I32);
    encoder.new_register(right_byte.clone(), ScopeKind::Local, Type::I32);

    encoder.extend([
        i::Get(left.clone()),
        i::ArrayLen,
        i::Set(left_len.clone()),
        i::Get(right.clone()),
        i::ArrayLen,
        i::Set(right_len.clone()),
        i::Get(left_len.clone()),
        i::Set(min_len.clone()),
        i::Get(right_len.clone()),
        i::Get(left_len.clone()),
        i::I32Op(NumberOperation::Lt),
        i::If(None),
        i::Get(right_len.clone()),
        i::Set(min_len.clone()),
        i::End,
        i::I32Const(0),
        i::Set(index.clone()),
        i::I32Const(0),
        i::Set(cmp.clone()),
    ]);

    encoder.extend([i::Block(None), i::Loop]);
    encoder.extend([
        i::Get(index.clone()),
        i::Get(min_len.clone()),
        i::I32Op(NumberOperation::Eq),
        i::BreakIf(1),
        i::Get(left.clone()),
        i::Get(index.clone()),
        i::ArrayGet(Type::I8),
        i::Set(left_byte.clone()),
        i::Get(right.clone()),
        i::Get(index.clone()),
        i::ArrayGet(Type::I8),
        i::Set(right_byte.clone()),
        i::Get(left_byte.clone()),
        i::Get(right_byte.clone()),
        i::I32Op(NumberOperation::Lt),
        i::If(None),
        i::I32Const(-1),
        i::Set(cmp.clone()),
        i::Break(2),
        i::End,
        i::Get(left_byte.clone()),
        i::Get(right_byte.clone()),
        i::I32Op(NumberOperation::Gt),
        i::If(None),
        i::I32Const(1),
        i::Set(cmp.clone()),
        i::Break(2),
        i::End,
        i::Get(index.clone()),
        i::I32Const(1),
        i::I32Op(NumberOperation::Add),
        i::Set(index.clone()),
        i::Break(0),
    ]);
    encoder.extend([i::End, i::End]);

    encoder.extend([
        i::Get(cmp.clone()),
        i::I32Const(0),
        i::I32Op(NumberOperation::Eq),
        i::If(None),
        i::Get(left_len.clone()),
        i::Get(right_len.clone()),
        i::I32Op(NumberOperation::Lt),
        i::If(None),
        i::I32Const(-1),
        i::Set(cmp.clone()),
        i::Else,
        i::Get(left_len.clone()),
        i::Get(right_len.clone()),
        i::I32Op(NumberOperation::Gt),
        i::If(None),
        i::I32Const(1),
        i::Set(cmp.clone()),
        i::End,
        i::End,
        i::End,
    ]);

    encoder.extend([
        i::Get(cmp),
        i::I32Const(0),
        i::I32Op(op),
        i::StructNew(bool_fields.clone()),
    ]);
}

#[derive(Debug, Clone)]
pub(crate) struct ConstructorInfo {
    pub kind: ConstructorKind,
}

#[derive(Debug, Clone)]
pub(crate) enum ConstructorKind {
    SumVariant {
        tag: usize,
        payload: Option<SemanticType>,
    },
    Wrap {
        payload: Option<SemanticType>,
    },
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ConstructorTable {
    constructors: IndexMap<Path, ConstructorInfo>,
}

impl ConstructorTable {
    /// Handles from symbols.
    pub(crate) fn from_symbols(symbols: &SymbolTable) -> Self {
        let mut constructors = IndexMap::new();
        for (path, definition) in symbols.type_definitions().iter() {
            if let Some(variants) = sum_variants(definition) {
                for (tag, (variant, payload_type)) in variants.iter().enumerate() {
                    let payload = if is_unit_type(payload_type, symbols) {
                        None
                    } else {
                        Some(payload_type.clone())
                    };
                    constructors.insert(
                        path.sibling(variant),
                        ConstructorInfo {
                            kind: ConstructorKind::SumVariant { tag, payload },
                        },
                    );
                }
                continue;
            }

            if definition.kind == crate::types::TypeDefinitionKind::Named {
                let args = (0..definition.parameters)
                    .map(|index| SemanticType::v((definition.parameters - 1 - index) as u32))
                    .collect::<Vec<_>>();
                let payload_type = instantiate_named_body(&definition.body, &args)
                    .unwrap_or_else(|| definition.body.clone());
                let payload = if is_unit_type(&payload_type, symbols) {
                    None
                } else {
                    Some(payload_type)
                };
                constructors.insert(
                    path.clone(),
                    ConstructorInfo {
                        kind: ConstructorKind::Wrap { payload },
                    },
                );
            }
        }
        for (alias, target) in symbols.constructor_aliases() {
            let mut current = target;
            let mut visited = std::collections::HashSet::new();
            while let Some(next) = symbols.constructor_aliases().get(current) {
                if !visited.insert(current.clone()) {
                    break;
                }
                current = next;
            }
            if let Some(info) = constructors.get(current).cloned() {
                constructors.insert(alias.clone(), info);
            }
        }
        Self { constructors }
    }

    /// Handles get.
    pub(crate) fn get(
        &self,
        path: &Path,
    ) -> Option<&ConstructorInfo> {
        self.constructors.get(path)
    }

    /// Handles constructors for module.
    pub(crate) fn constructors_for_module(
        &self,
        module_name: &str,
    ) -> Vec<(Path, ConstructorInfo)> {
        self.constructors
            .iter()
            .filter(|(path, _)| path.major == module_name)
            .map(|(path, info)| (path.clone(), info.clone()))
            .collect()
    }
}

/// Handles sum variants.
fn sum_variants(definition: &TypeDefinition) -> Option<IndexMap<String, SemanticType>> {
    let mut current = &definition.body;
    loop {
        match current {
            SemanticType::ForAll { body, .. } => current = body,
            SemanticType::Sum { variants } => return Some(variants.clone()),
            _ => return None,
        }
    }
}

/// Handles is unit type.
fn is_unit_type(
    type_: &SemanticType,
    symbols: &SymbolTable,
) -> bool {
    match type_ {
        SemanticType::Unit => true,
        SemanticType::Named { name, body } => {
            matches!(resolve_named_body(name, body, symbols), SemanticType::Unit)
        }
        SemanticType::Apply {
            constructor,
            arguments,
        } => {
            apply_type(constructor, arguments, symbols)
                .as_ref()
                .is_some_and(|t| is_unit_type(t, symbols))
        }
        _ => false,
    }
}

/// Handles resolve named body.
fn resolve_named_body(
    name: &Path,
    body: &SemanticType,
    symbols: &SymbolTable,
) -> SemanticType {
    if matches!(body, SemanticType::Unit)
        && let Some(definition) = symbols.type_definitions().get(name)
    {
        definition.body.clone()
    } else {
        body.clone()
    }
}

/// Handles apply type.
fn apply_type(
    constructor: &SemanticType,
    arguments: &[SemanticType],
    symbols: &SymbolTable,
) -> Option<SemanticType> {
    let mut current = constructor.clone();
    for arg in arguments {
        current = match current {
            SemanticType::ForAll { body, .. } => body.open_forall(arg)?,
            SemanticType::Named { name, body } => {
                let resolved = resolve_named_body(&name, &body, symbols);
                if let SemanticType::ForAll { body: inner, .. } = resolved {
                    inner.open_forall(arg)?
                } else {
                    return None;
                }
            }
            _ => return None,
        };
    }
    Some(current)
}

/// Handles instantiate named body.
fn instantiate_named_body(
    body: &SemanticType,
    arguments: &[SemanticType],
) -> Option<SemanticType> {
    let mut current = body.clone();
    for arg in arguments {
        if let SemanticType::ForAll { body: inner, .. } = current {
            current = inner.open_forall(arg)?;
        } else {
            return None;
        }
    }
    Some(current)
}

/// Handles struct fields for type.
fn struct_fields_for_type(
    type_: &SemanticType,
    symbols: &SymbolTable,
) -> Option<IndexMap<String, SemanticType>> {
    match type_ {
        SemanticType::Struct { fields } => Some(fields.clone()),
        SemanticType::StructConstraint { fields, .. } => Some(fields.clone()),
        SemanticType::Named { name, body } => {
            let resolved = resolve_named_body(name, body, symbols);
            struct_fields_for_type(&resolved, symbols)
        }
        SemanticType::Apply {
            constructor,
            arguments,
        } => {
            if let SemanticType::Named { name, body } = constructor.as_ref() {
                let resolved = resolve_named_body(name, body, symbols);
                let instantiated = instantiate_named_body(&resolved, arguments)?;
                struct_fields_for_type(&instantiated, symbols)
            } else {
                let applied = apply_type(constructor, arguments, symbols)?;
                struct_fields_for_type(&applied, symbols)
            }
        }
        _ => None,
    }
}

/// Handles ordered trait methods for path.
fn ordered_trait_methods_for_path(
    symbols: &SymbolTable,
    method_path: &Path,
) -> Option<(Vec<(Path, TypeScheme)>, usize)> {
    symbols.trait_defs().values().find_map(|def| {
        let methods = ordered_trait_methods(def);
        methods
            .iter()
            .position(|(path, _)| path == method_path)
            .map(|index| (methods, index))
    })
}

/// Handles collect pattern bindings.
fn collect_pattern_bindings(pattern: &Pattern<SemanticType>) -> Vec<(Path, SemanticType)> {
    match &pattern.kind {
        PatternKind::Hole | PatternKind::Immediate(_) | PatternKind::ConstConstructor(_) => {
            Vec::new()
        }
        PatternKind::Identifier(path) => vec![(path.clone(), pattern.type_.clone())],
        PatternKind::Constructor(_, payload) => collect_pattern_bindings(payload),
        PatternKind::Tuple(items) => items.iter().flat_map(collect_pattern_bindings).collect(),
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            let mut bindings = Vec::new();
            bindings.extend(starting.iter().flat_map(collect_pattern_bindings));
            bindings.extend(ending.iter().flat_map(collect_pattern_bindings));
            if let Glob::Named(path) = glob {
                bindings.push((path.clone(), pattern.type_.clone()));
            }
            bindings
        }
        PatternKind::Struct(fields) => fields.values().flat_map(collect_pattern_bindings).collect(),
        PatternKind::TypeHint(inner, _) => collect_pattern_bindings(inner),
    }
}

/// Handles pattern introduced names.
fn pattern_introduced_names(pattern: &Pattern<SemanticType>) -> usize {
    collect_pattern_bindings(pattern).len()
}

/// Handles pattern is refutable.
fn pattern_is_refutable(pattern: &Pattern<SemanticType>) -> bool {
    match &pattern.kind {
        PatternKind::Hole | PatternKind::Identifier(_) => false,
        PatternKind::TypeHint(inner, _) => pattern_is_refutable(inner),
        PatternKind::Tuple(items) => items.iter().any(pattern_is_refutable),
        PatternKind::Struct(fields) => fields.values().any(pattern_is_refutable),
        PatternKind::Array { .. }
        | PatternKind::Immediate(_)
        | PatternKind::ConstConstructor(_)
        | PatternKind::Constructor(..) => true,
    }
}
