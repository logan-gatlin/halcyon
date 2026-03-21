use indexmap::IndexMap;

use super::*;

#[derive(Debug, Clone)]
pub enum ResolvedBinding {
    Local(u32),
    Global(u32),
}

#[derive(Debug, Clone)]
pub enum ResolvedInstruction {
    Set(ResolvedBinding),
    Get(ResolvedBinding),
    Const(ImmediateValue),
    I32Const(i32),
    F32Const(f32),
    Func(u32),
    StructNew(Box<[Type]>),
    StructGet(Box<[Type]>, usize),
    ArrayGet(Type),
    ArrayNewFixed {
        inner_type: Type,
        length: usize,
    },
    ArrayNewDefault(Type),
    ArrayLen,
    ArrayCopy {
        dst_type: Type,
        src_type: Type,
    },
    CallRef {
        parameters: Box<[Type]>,
        returns: Box<[Type]>,
    },
    Call(u32),
    Unreachable,
    Drop,
    If(Option<Type>),
    Else,
    End,
    Loop,
    Block(Option<Type>),
    Break(usize),
    BreakIf(usize),
    I32Op(NumberOperation),
    I64Op(NumberOperation),
    F32Op(NumberOperation),
    F64Op(NumberOperation),
    RefCastFunc {
        parameters: Box<[Type]>,
        returns: Box<[Type]>,
    },
    RefCastStruct(Box<[Type]>),
    RefCastArray(Box<Type>),
    I32Store8,
    I32Load,
    I32Store,
    I64Load,
    I64ExtendI32U,
    I32WrapI64,
    I32TruncF32S,
    I32TruncF32U,
    I32TruncF64S,
    I32TruncF64U,
    I64TruncF32S,
    I64TruncF32U,
    I64TruncF64S,
    I64TruncF64U,
    F32DemoteF64,
    F64ConvertI64S,
    F64ConvertI64U,
}

#[derive(Debug, Clone)]
pub struct ResolvedFunction {
    pub parameters: IndexMap<Path, Type>,
    pub returns: Vec<Type>,
    pub variables: IndexMap<Path, Type>,
    pub ops: Vec<ResolvedInstruction>,
    pub op_origins: Vec<Option<SourceOrigin>>,
}

#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub lowered: Module,
    pub function_indices: IndexMap<Path, u32>,
    pub start_function_index: u32,
    pub functions: Vec<ResolvedFunction>,
}

/// Handles resolve module.
pub fn resolve_module(mut module: Module) -> Result<ResolvedModule, BackendError> {
    inject_group_binding_imports(&mut module);

    let mut function_indices = IndexMap::new();
    for (idx, (path, _)) in module.function_imports.iter().enumerate() {
        function_indices.insert(path.clone(), idx as u32);
    }
    let function_import_count = module.function_imports.len() as u32;
    for (idx, (path, _)) in module.functions.iter().enumerate() {
        function_indices.insert(path.clone(), idx as u32 + function_import_count);
    }

    let mut global_indices = IndexMap::new();
    for (idx, (path, _)) in module.imports.iter().enumerate() {
        global_indices.insert(path.clone(), idx as u32);
    }
    let global_import_count = module.imports.len() as u32;
    for (idx, (path, _)) in module.globals.iter().enumerate() {
        global_indices.insert(path.clone(), idx as u32 + global_import_count);
    }

    let start_function_index = *function_indices.get(&module.start).ok_or_else(|| {
        BackendError::module(format!("Missing start function `{}`", module.start))
    })?;

    let mut functions = Vec::with_capacity(module.functions.len());
    for (function_path, function) in &module.functions {
        if function.op_origins.len() != function.ops.len() {
            return Err(BackendError::in_function(
                function_path.clone(),
                None,
                None,
                format!(
                    "origin count mismatch: {} ops but {} origins",
                    function.ops.len(),
                    function.op_origins.len()
                ),
            ));
        }

        let local_indices = function
            .parameters
            .iter()
            .chain(function.variables.iter())
            .enumerate()
            .map(|(index, (name, _))| (name.clone(), index as u32))
            .collect::<IndexMap<_, _>>();

        let mut resolved_ops = Vec::with_capacity(function.ops.len());
        for (op_index, op) in function.ops.iter().enumerate() {
            let origin = function.op_origins.get(op_index).cloned().unwrap_or(None);
            resolved_ops.push(resolve_instruction(
                op,
                &local_indices,
                &global_indices,
                &function_indices,
                function_path,
                op_index,
                origin,
            )?);
        }

        functions.push(ResolvedFunction {
            parameters: function.parameters.clone(),
            returns: function.returns.clone(),
            variables: function.variables.clone(),
            ops: resolved_ops,
            op_origins: function.op_origins.clone(),
        });
    }

    Ok(ResolvedModule {
        lowered: module,
        function_indices,
        start_function_index,
        functions,
    })
}

/// Handles inject group binding imports.
fn inject_group_binding_imports(module: &mut Module) {
    let mut inferred_types = IndexMap::<Path, Type>::new();
    for function in module.functions.values() {
        for (path, type_) in function.parameters.iter().chain(function.variables.iter()) {
            inferred_types
                .entry(path.clone())
                .or_insert_with(|| type_.clone());
        }
    }

    let referenced = module
        .functions
        .values()
        .flat_map(|function| function.ops.iter())
        .filter_map(|instruction| {
            match instruction {
                Instruction::Get(path) | Instruction::Set(path) if is_group_binding(path) => {
                    Some(path.clone())
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();

    for path in referenced {
        if module.imports.contains_key(&path) || module.globals.contains_key(&path) {
            continue;
        }
        let type_ = inferred_types.get(&path).cloned().unwrap_or(Type::Any);
        module.imports.insert(path, type_);
    }
}

/// Handles is group binding.
fn is_group_binding(path: &Path) -> bool {
    path.minor.starts_with("[group binding] #")
}

/// Handles resolve instruction.
fn resolve_instruction(
    op: &Instruction,
    local_indices: &IndexMap<Path, u32>,
    global_indices: &IndexMap<Path, u32>,
    function_indices: &IndexMap<Path, u32>,
    function_path: &Path,
    op_index: usize,
    origin: Option<SourceOrigin>,
) -> Result<ResolvedInstruction, BackendError> {
    Ok(match op {
        Instruction::Set(path) => {
            if let Some(index) = local_indices.get(path) {
                ResolvedInstruction::Set(ResolvedBinding::Local(*index))
            } else if let Some(index) = global_indices.get(path) {
                ResolvedInstruction::Set(ResolvedBinding::Global(*index))
            } else {
                return Err(BackendError::in_function(
                    function_path.clone(),
                    Some(op_index),
                    origin,
                    format!("Unknown set target `{path}`"),
                ));
            }
        }
        Instruction::Get(path) => {
            if let Some(index) = local_indices.get(path) {
                ResolvedInstruction::Get(ResolvedBinding::Local(*index))
            } else if let Some(index) = global_indices.get(path) {
                ResolvedInstruction::Get(ResolvedBinding::Global(*index))
            } else {
                return Err(BackendError::in_function(
                    function_path.clone(),
                    Some(op_index),
                    origin,
                    format!("Unknown get target `{path}`"),
                ));
            }
        }
        Instruction::Func(path) => {
            let index = *function_indices.get(path).ok_or_else(|| {
                BackendError::in_function(
                    function_path.clone(),
                    Some(op_index),
                    origin,
                    format!("Unknown function reference `{path}`"),
                )
            })?;
            ResolvedInstruction::Func(index)
        }
        Instruction::Call(path) => {
            let index = *function_indices.get(path).ok_or_else(|| {
                BackendError::in_function(
                    function_path.clone(),
                    Some(op_index),
                    origin,
                    format!("Unknown call target `{path}`"),
                )
            })?;
            ResolvedInstruction::Call(index)
        }
        Instruction::Const(value) => ResolvedInstruction::Const(value.clone()),
        Instruction::I32Const(value) => ResolvedInstruction::I32Const(*value),
        Instruction::F32Const(value) => ResolvedInstruction::F32Const(*value),
        Instruction::StructNew(fields) => ResolvedInstruction::StructNew(fields.clone()),
        Instruction::StructGet(fields, index) => {
            ResolvedInstruction::StructGet(fields.clone(), *index)
        }
        Instruction::ArrayGet(type_) => ResolvedInstruction::ArrayGet(type_.clone()),
        Instruction::ArrayNewFixed { inner_type, length } => {
            ResolvedInstruction::ArrayNewFixed {
                inner_type: inner_type.clone(),
                length: *length,
            }
        }
        Instruction::ArrayNewDefault(type_) => ResolvedInstruction::ArrayNewDefault(type_.clone()),
        Instruction::ArrayLen => ResolvedInstruction::ArrayLen,
        Instruction::ArrayCopy { dst_type, src_type } => {
            ResolvedInstruction::ArrayCopy {
                dst_type: dst_type.clone(),
                src_type: src_type.clone(),
            }
        }
        Instruction::CallRef {
            parameters,
            returns,
        } => {
            ResolvedInstruction::CallRef {
                parameters: parameters.clone(),
                returns: returns.clone(),
            }
        }
        Instruction::Unreachable => ResolvedInstruction::Unreachable,
        Instruction::Drop => ResolvedInstruction::Drop,
        Instruction::If(result) => ResolvedInstruction::If(result.clone()),
        Instruction::Else => ResolvedInstruction::Else,
        Instruction::End => ResolvedInstruction::End,
        Instruction::Loop => ResolvedInstruction::Loop,
        Instruction::Block(result) => ResolvedInstruction::Block(result.clone()),
        Instruction::Break(depth) => ResolvedInstruction::Break(*depth),
        Instruction::BreakIf(depth) => ResolvedInstruction::BreakIf(*depth),
        Instruction::I32Op(operation) => ResolvedInstruction::I32Op(*operation),
        Instruction::I64Op(operation) => ResolvedInstruction::I64Op(*operation),
        Instruction::F32Op(operation) => ResolvedInstruction::F32Op(*operation),
        Instruction::F64Op(operation) => ResolvedInstruction::F64Op(*operation),
        Instruction::RefCastFunc {
            parameters,
            returns,
        } => {
            ResolvedInstruction::RefCastFunc {
                parameters: parameters.clone(),
                returns: returns.clone(),
            }
        }
        Instruction::RefCastStruct(fields) => ResolvedInstruction::RefCastStruct(fields.clone()),
        Instruction::RefCastArray(inner) => ResolvedInstruction::RefCastArray(inner.clone()),
        Instruction::I32Store8 => ResolvedInstruction::I32Store8,
        Instruction::I32Load => ResolvedInstruction::I32Load,
        Instruction::I32Store => ResolvedInstruction::I32Store,
        Instruction::I64Load => ResolvedInstruction::I64Load,
        Instruction::I64ExtendI32U => ResolvedInstruction::I64ExtendI32U,
        Instruction::I32WrapI64 => ResolvedInstruction::I32WrapI64,
        Instruction::I32TruncF32S => ResolvedInstruction::I32TruncF32S,
        Instruction::I32TruncF32U => ResolvedInstruction::I32TruncF32U,
        Instruction::I32TruncF64S => ResolvedInstruction::I32TruncF64S,
        Instruction::I32TruncF64U => ResolvedInstruction::I32TruncF64U,
        Instruction::I64TruncF32S => ResolvedInstruction::I64TruncF32S,
        Instruction::I64TruncF32U => ResolvedInstruction::I64TruncF32U,
        Instruction::I64TruncF64S => ResolvedInstruction::I64TruncF64S,
        Instruction::I64TruncF64U => ResolvedInstruction::I64TruncF64U,
        Instruction::F32DemoteF64 => ResolvedInstruction::F32DemoteF64,
        Instruction::F64ConvertI64S => ResolvedInstruction::F64ConvertI64S,
        Instruction::F64ConvertI64U => ResolvedInstruction::F64ConvertI64U,
    })
}
