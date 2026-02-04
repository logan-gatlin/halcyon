use crate::Visit;
use crate::ir::{
    Constructor,
    Glob,
    IrNode,
    Pattern,
};
use crate::operator::BinaryOp;
use crate::semantic::{
    Typed,
    WithType,
};

use super::*;

pub fn lower_type(
    type_: &semantic::Type,
    symbols: &SymbolTable,
) -> Type {
    use semantic::Type::*;
    match type_ {
        Any | Variable(_) => Type::Any,
        Unit => Type::Struct([].into()),
        Integer => Type::Struct([Type::I64].into()),
        Real => Type::Struct([Type::F64].into()),
        Glyph | Boolean => Type::Struct([Type::I32].into()),
        String => Type::Array(Type::I32.into()),
        Struct { fields, .. } => {
            Type::Struct(fields.values().map(|v| lower_type(v, symbols)).collect())
        }
        Array(t) => Type::Array(lower_type(t, symbols).into()),
        Tuple(items) => Type::Struct(items.iter().map(|i| lower_type(i, symbols)).collect()),
        Sum { .. } => Type::Struct([Type::I32, Type::Any].into()),
        Function(..) => Type::closure_type(),
        Instantiation(path, items) => {
            lower_type(
                &symbols
                    .get_type(path)
                    .clone()
                    .instantiate(items)
                    .unwrap_or_else(|_| unreachable!()),
                symbols,
            )
        }
    }
}

use Instruction as i;
use indexmap::IndexMap;
impl<'a> Encoder<'a> {
    /// Create a new closure, push a reference to it onto the stack
    pub fn closure(
        &mut self,
        symbols: &SymbolTable,
        parameter: Typed<Path>,
        captures: Vec<Typed<Path>>,
        body: impl for<'b> FnOnce(&mut Encoder<'b>, &SymbolTable),
    ) -> usize {
        let mut new_enc = self.module.new_function();
        let new_func_index = new_enc.func_index;
        let capture_array_name = new_enc.temporary_name("captured_symbols");
        new_enc.new_parameter(capture_array_name.clone(), Type::Array(Type::Any.into()));
        let parameter_type = lower_type(&parameter.type_, symbols);
        let param_anyref_name = new_enc.temporary_name("parameter");
        new_enc.new_parameter(param_anyref_name.clone(), Type::Any);
        new_enc.new_return(Type::Any);
        // Cast the anyref parameter to the actual type and bind to the user's name
        new_enc.new_register(
            parameter.inner.clone(),
            ScopeKind::Local,
            parameter_type.clone(),
        );
        new_enc.push(i::Get(param_anyref_name));
        match &parameter_type {
            Type::Struct(fields) => new_enc.push(i::RefCastStruct(fields.clone())),
            Type::Array(inner) => new_enc.push(i::RefCastArray(inner.clone())),
            Type::Any => {} // No cast needed
            _ => {}         // Primitives don't need casting (i32, i64, etc. shouldn't appear here)
        }
        new_enc.push(i::Set(parameter.inner));
        for (
            id,
            Typed {
                inner: capture_name,
                type_: capture_type,
            },
        ) in captures.clone().into_iter().enumerate()
        {
            let capture_type = lower_type(&capture_type, symbols);
            new_enc.new_register(capture_name.clone(), ScopeKind::Local, capture_type.clone());
            new_enc.extend([
                i::Get(capture_array_name.clone()),
                i::I32Const(id as i32),
                i::ArrayGet(Type::Any),
            ]);
            // Cast anyref from array to the actual capture type
            match &capture_type {
                Type::Struct(fields) => new_enc.push(i::RefCastStruct(fields.clone())),
                Type::Array(inner) => new_enc.push(i::RefCastArray(inner.clone())),
                Type::Any => {} // No cast needed
                _ => {}         // Primitives shouldn't appear here (captured in closures)
            }
            new_enc.push(i::Set(capture_name.clone()));
        }
        body(&mut new_enc, symbols);
        let num_captures = captures.len();
        for capture in captures {
            self.push(i::Get(capture.inner));
        }
        self.extend([
            i::ArrayNewFixed {
                inner_type: Type::Any,
                length: num_captures,
            },
            i::Func(new_func_index),
            i::StructNew([Type::function_capture(), Type::closure_function_type()].into()),
        ]);
        new_func_index
    }
    // Preconditions:
    // * Predicate to be pattern-matched on is top of stack
    // * A br 0 instruction indicates pattern matching has failed
    pub fn lower_pattern(
        &mut self,
        pat: Pattern,
        scope: ScopeKind,
        symbols: &SymbolTable,
    ) {
        use crate::ir::PatternKind as p;
        let type_ = lower_type(&pat.type_, symbols);
        match pat.inner.inner {
            p::Hole => {
                self.push(i::Drop);
            }
            p::Identifier(path) => {
                self.push(i::Set(path));
            }
            p::Tuple(items) => {
                let temporary = self.temporary_name("pattern");
                let semantic::Type::Tuple(types) = &pat.type_ else {
                    unreachable!()
                };
                let types = types
                    .iter()
                    .map(|t| lower_type(t, symbols))
                    .collect::<Box<_>>();
                self.new_register(temporary.clone(), scope, type_.clone());
                self.push(i::Set(temporary.clone()));
                for (index, item) in items.into_iter().enumerate() {
                    self.extend([
                        i::Get(temporary.clone()),
                        i::StructGet(types.clone(), index),
                    ]);
                    self.lower_pattern(item, scope, symbols);
                }
            }
            p::Array {
                starting,
                glob,
                ending,
            } => {
                let semantic::Type::Array(inner_type) = &pat.type_ else {
                    unreachable!()
                };
                let inner_type_lowered = lower_type(inner_type, symbols);
                let temporary = self.temporary_name("array_pattern");
                self.new_register(temporary.clone(), scope, type_.clone());
                self.push(i::Set(temporary.clone()));

                let start_len = starting.len() as i32;
                let end_len = ending.len() as i32;
                let min_len = start_len + end_len;

                let cmp_op = if glob.is_exact() {
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
                    self.lower_pattern(pattern, scope, symbols);
                }

                // Compute middle_len and capture glob if needed
                let middle_len_var = if glob.is_exact() {
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
                    if let Glob::Named(glob_name) = glob {
                        let new_array = self.temporary_name("slice");
                        self.new_register(new_array.clone(), scope, type_.clone());
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
                        self.new_register(glob_name.clone(), scope, type_.clone());
                        self.extend([i::Get(new_array), i::Set(glob_name)]);
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
                    self.lower_pattern(pattern, scope, symbols);
                }
            }
            p::Struct(index_map) => {
                let temporary = self.temporary_name("pattern");
                self.new_register(temporary.clone(), scope, type_.clone());
                self.push(i::Set(temporary.clone()));
                let semantic::Type::Struct {
                    fields: ordered_fields,
                    ..
                } = &pat.type_
                else {
                    unreachable!()
                };
                let types = ordered_fields
                    .values()
                    .map(|t| lower_type(t, symbols))
                    .collect::<Box<_>>();
                for (name, pattern) in index_map {
                    let index = ordered_fields
                        .get_index_of(&name.inner)
                        .unwrap_or_else(|| unreachable!());
                    self.extend([
                        i::Get(temporary.clone()),
                        i::StructGet(types.clone(), index),
                    ]);
                    self.lower_pattern(pattern, scope, symbols);
                }
            }
            p::Constructor(constructor, inner) => {
                use crate::ir::Constructor;
                match constructor {
                    Constructor::Structure(_) => {
                        // Structure constructor is just a type hint, match inner pattern
                        self.lower_pattern(*inner, scope, symbols);
                    }
                    Constructor::SumConstant { tag, .. } | Constructor::SumFunction { tag, .. } => {
                        // Sum type is Struct([I32, Any]) - tag at index 0, value at index 1
                        let sum_type: Box<[Type]> = [Type::I32, Type::Any].into();

                        let temporary = self.temporary_name("constructor_pattern");
                        self.new_register(temporary.clone(), scope, type_.clone());
                        self.push(i::Set(temporary.clone()));

                        // Check if tag matches, break if not
                        self.extend([
                            i::Get(temporary.clone()),
                            i::StructGet(sum_type.clone(), 0),
                            i::I32Const(tag as i32),
                            i::I32Op(NumberOperation::Ne),
                            i::BreakIf(0),
                        ]);

                        // Get value and match inner pattern
                        self.extend([i::Get(temporary), i::StructGet(sum_type, 1)]);
                        // Cast anyref to the expected type
                        let inner_type = lower_type(&inner.type_, symbols);
                        match &inner_type {
                            Type::Struct(fields) => self.push(i::RefCastStruct(fields.clone())),
                            Type::Array(inner) => self.push(i::RefCastArray(inner.clone())),
                            Type::Any => {}
                            _ => {}
                        }
                        self.lower_pattern(*inner, scope, symbols);
                    }
                }
            }
            p::Immediate(const_value) => {
                let double_equal_path = BinaryOp::DoubleEqual.path();
                if double_equal_path.major != self.module.name {
                    self.module
                        .imports
                        .entry(double_equal_path.clone())
                        .or_insert_with(|| lower_type(&BinaryOp::DoubleEqual.get_type(), symbols));
                }
                let Type::Struct(inner_types) = lower_type(&const_value.type_of(), symbols) else {
                    unreachable!()
                };
                self.extend([
                    i::Const(const_value),
                    i::StructNew(inner_types),
                    i::Get(double_equal_path),
                ]);
                self.call_closure();
                self.call_closure();
                let Type::Struct(bool_fields) = lower_type(&semantic::Type::Boolean, symbols)
                else {
                    unreachable!()
                };
                self.extend([
                    i::RefCastStruct(bool_fields.clone()),
                    i::StructGet(bool_fields, 0),
                    i::I32Const(1),
                    i::I32Op(NumberOperation::Xor),
                    i::BreakIf(0),
                ]);
            }
            p::TypeHint(inner, _) => {
                self.lower_pattern(*inner, scope, symbols);
            }
        }
    }
    pub fn lower_ir(
        &mut self,
        ir: IrNode,
        symbols: &SymbolTable,
    ) {
        use crate::ir::IrKind::*;
        match ir.inner.inner {
            Let {
                mut assignee,
                value,
                then,
                else_,
                ..
            } => {
                let skip_pattern = !assignee.is_refutable() && assignee.introduced_names() == 0;
                assignee.visit(|(path, type_)| {
                    self.new_register(path.clone(), ScopeKind::Local, lower_type(type_, symbols));
                });
                let result_type = lower_type(&ir.type_, symbols);
                if !skip_pattern {
                    self.extend([i::Block(Some(result_type)), i::Block(None)]);
                }
                self.lower_ir(*value, symbols);
                if !skip_pattern {
                    self.lower_pattern(assignee, ScopeKind::Local, symbols);
                } else {
                    self.push(i::Drop);
                }
                self.lower_ir(*then, symbols);
                if !skip_pattern {
                    self.extend([i::Break(1), i::End]);
                    self.lower_ir(*else_, symbols);
                    self.push(i::End);
                }
            }
            Immediate(const_value) => {
                let Type::Struct(inner_types) = lower_type(&const_value.type_of(), symbols) else {
                    unreachable!()
                };
                self.extend([i::Const(const_value), i::StructNew(inner_types)]);
            }
            Identifier(path) => {
                // If it is an imported symbol, add it to imports
                if path.major != self.module.name {
                    let type_ = lower_type(&ir.type_, symbols);
                    self.module
                        .imports
                        .entry(path.clone())
                        .or_insert_with(|| type_.clone());
                }
                self.push(i::Get(path));
            }
            Tuple(items) => {
                let types = items
                    .iter()
                    .map(|i| lower_type(&i.type_, symbols))
                    .collect::<Box<[_]>>();
                items.into_iter().for_each(|i| {
                    self.lower_ir(i, symbols);
                });
                self.push(i::StructNew(types));
            }
            Struct(index_map) => {
                let semantic::Type::Struct { fields, .. } = &ir.type_ else {
                    unreachable!()
                };
                // Evaluate fields in order and store in temporaries
                let mut field_temporaries = IndexMap::new();
                for (field_name, field_ir) in index_map {
                    let temp_name = self.temporary_name(&field_name.inner);
                    let temp_type = lower_type(&field_ir.type_, symbols);
                    self.new_register(temp_name.clone(), ScopeKind::Local, temp_type);
                    self.lower_ir(field_ir, symbols);
                    self.push(i::Set(temp_name.clone()));
                    field_temporaries.insert(field_name.inner, temp_name);
                }
                // Re-order fields according to struct type and push onto stack
                let ordered_types = fields
                    .keys()
                    .map(|k| lower_type(&fields[k], symbols))
                    .collect::<Box<[_]>>();
                for field_name in fields.keys() {
                    let temp_name = &field_temporaries[field_name];
                    self.push(i::Get(temp_name.clone()));
                }
                self.push(i::StructNew(ordered_types));
            }
            Field { of, index } => {
                let semantic::Type::Struct { fields, .. } = &of.type_ else {
                    unreachable!()
                };
                let field_index = fields
                    .keys()
                    .position(|k| k == &index.inner)
                    .unwrap_or_else(|| {
                        unreachable!("Missing fields are caught during typechecking")
                    });
                let Type::Struct(inner_types) = lower_type(&of.type_, symbols) else {
                    unreachable!()
                };
                self.lower_ir(*of, symbols);
                self.push(i::StructGet(inner_types, field_index));
            }
            Function {
                parameter_name,
                captures,
                body,
                ..
            } => {
                let semantic::Type::Function(parameter_type, ..) = ir.type_ else {
                    unreachable!()
                };
                self.closure(
                    symbols,
                    parameter_name.inner.clone().with_type(*parameter_type),
                    captures,
                    |new_enc, symbols| {
                        new_enc.lower_ir(*body, symbols);
                    },
                );
            }
            Call { callee, argument } => {
                let callee_name = self.temporary_name("callee");
                // Use Type::Any because call_closure returns anyref for polymorphism
                self.new_register(callee_name.clone(), ScopeKind::Local, Type::Any);
                self.lower_ir(*callee, symbols);
                self.push(i::Set(callee_name.clone()));
                self.lower_ir(*argument, symbols);
                self.push(i::Get(callee_name));
                self.call_closure();
                // Cast the anyref result to the expected type
                let result_type = lower_type(&ir.type_, symbols);
                match result_type {
                    Type::Struct(fields) => self.push(i::RefCastStruct(fields)),
                    Type::Array(inner) => self.push(i::RefCastArray(inner)),
                    Type::Any => {} // No cast needed
                    _ => unreachable!(),
                }
            }
            Semicolon(a, b) => {
                self.lower_ir(*a, symbols);
                self.push(i::Drop);
                self.lower_ir(*b, symbols);
            }
            Unreachable => {
                self.push(i::Unreachable);
            }
        }
    }

    pub fn lower_constructor(
        &mut self,
        path: Path,
        cons: Constructor,
        symbols: &SymbolTable,
    ) {
        let sum_struct_type: Box<[Type]> = [Type::I32, Type::Any].into();

        match cons {
            Constructor::SumConstant { tag, .. } => {
                self.new_register(
                    path.clone(),
                    ScopeKind::Global,
                    Type::Struct(sum_struct_type.clone()),
                );
                self.extend([
                    i::I32Const(tag as i32),
                    i::StructNew([].into()),       // Create unit value ()
                    i::StructNew(sum_struct_type), // Create sum struct { tag, value }
                    i::Set(path),
                ]);
            }
            Constructor::SumFunction {
                tag,
                parameter_type,
                ..
            } => {
                let parameter_name = self.temporary_name("cons_param");
                self.closure(
                    symbols,
                    parameter_name.clone().with_type(parameter_type),
                    vec![],
                    |inner_func, _| {
                        inner_func.extend([
                            i::I32Const(tag as i32),
                            i::Get(parameter_name),
                            i::StructNew(sum_struct_type.clone()),
                        ]);
                    },
                );

                self.new_register(path.clone(), ScopeKind::Global, Type::closure_type());
                self.extend([i::Set(path)]);
            }
            Constructor::Structure(struct_type) => {
                let parameter_name = self.temporary_name("cons_param");
                self.closure(
                    symbols,
                    parameter_name.clone().with_type(struct_type),
                    vec![],
                    |inner_func, _| {
                        inner_func.push(i::Get(parameter_name));
                    },
                );
                // Create closure: { captures: [], func: inner_func }
                self.new_register(path.clone(), ScopeKind::Global, Type::closure_type());
                self.extend([i::Set(path)]);
            }
        }
    }

    /// [argument, closure] -> [result]
    ///
    /// A closure is a struct with fields {captured_args, funcref}.
    /// The calling convention expects [captures, argument] on the stack before funcref.
    pub fn call_closure(&mut self) {
        let closure_struct_type: Box<[Type]> = [
            Type::function_capture(),
            Type::Function {
                parameters: [Type::Array(Type::Any.into()), Type::Any].into(),
                results: [Type::Any].into(),
            },
        ]
        .into();

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

        self.push(i::Call {
            parameters: func_params,
            returns: func_returns,
        });
        // Stack: [result: anyref]
    }
}
