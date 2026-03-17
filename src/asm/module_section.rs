use indexmap::IndexMap;
use serde::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use wasm_encoder::{
    CustomSection,
    Encode,
};

use super::*;
use crate::asm::custom_section::TypeSignatureSection;
use crate::ir::ImmediateValue;

#[derive(Debug, Clone)]
pub struct LoweredModuleSection {
    module: Module,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModulePayload {
    name: String,
    imports: Vec<(WirePath, WireType)>,
    globals: Vec<(WirePath, WireType)>,
    functions: Vec<(WirePath, WireFunction)>,
    function_imports: Vec<(WirePath, WireFunctionImport)>,
    has_memory: bool,
    start: WirePath,
    closure_counter: u64,
    export_policy: WireExportPolicy,
    source_files: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WireFunction {
    parameters: Vec<(WirePath, WireType)>,
    returns: Vec<WireType>,
    variables: Vec<(WirePath, WireType)>,
    ops: Vec<WireInstruction>,
    op_origins: Vec<Option<WireSourceOrigin>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WireSourceOrigin {
    file_name: String,
    start: u64,
    width: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WireFunctionImport {
    module: String,
    name: String,
    params: Vec<WireType>,
    results: Vec<WireType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WirePath {
    major: String,
    minor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WireType {
    Any,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Struct(Vec<WireType>),
    Array(Box<WireType>),
    Function {
        parameters: Vec<WireType>,
        results: Vec<WireType>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WireExportPolicy {
    MinorOnly,
    Qualified,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WireNumberOperation {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WireImmediateValue {
    Unit,
    Integer(i64),
    Real(f64),
    Boolean(bool),
    String(String),
    Glyph(char),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WireInstruction {
    Set(WirePath),
    Get(WirePath),
    Const(WireImmediateValue),
    I32Const(i32),
    F32Const(f32),
    Func(WirePath),
    StructNew(Vec<WireType>),
    StructGet(Vec<WireType>, u64),
    ArrayGet(WireType),
    ArrayNewFixed {
        inner_type: WireType,
        length: u64,
    },
    ArrayNewDefault(WireType),
    ArrayLen,
    ArrayCopy {
        dst_type: WireType,
        src_type: WireType,
    },
    CallRef {
        parameters: Vec<WireType>,
        returns: Vec<WireType>,
    },
    Call(WirePath),
    Unreachable,
    Drop,
    If(Option<WireType>),
    Else,
    End,
    Loop,
    Block(Option<WireType>),
    Break(u64),
    BreakIf(u64),
    I32Op(WireNumberOperation),
    I64Op(WireNumberOperation),
    F32Op(WireNumberOperation),
    F64Op(WireNumberOperation),
    RefCastFunc {
        parameters: Vec<WireType>,
        returns: Vec<WireType>,
    },
    RefCastStruct(Vec<WireType>),
    RefCastArray(Box<WireType>),
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
}

#[derive(Debug, Clone, Copy)]
enum ModulePayloadDecodeError {
    IntegerOverflow,
}

impl LoweredModuleSection {
    pub const NAME: &str = "halcyon.asm";

    /// Creates a new instance.
    pub fn new(module: &Module) -> Self {
        Self {
            module: module.clone(),
        }
    }

    /// Handles decode.
    pub fn decode(data: &[u8]) -> Option<Module> {
        let (name, payload) = decode_named_custom_section_data(data)?;
        if name != Self::NAME {
            return None;
        }
        Self::decode_data_slice(payload)
    }

    /// Handles decode data slice.
    pub fn decode_data_slice(data: &[u8]) -> Option<Module> {
        let payload = postcard::from_bytes::<ModulePayload>(data).ok()?;
        payload.try_into_module().ok()
    }
}

impl wasm_encoder::Section for LoweredModuleSection {
    /// Returns the identifier for this value.
    fn id(&self) -> u8 {
        0
    }
}

impl Encode for LoweredModuleSection {
    /// Handles encode.
    fn encode(
        &self,
        sink: &mut Vec<u8>,
    ) {
        let payload = ModulePayload::from_module(&self.module);
        let data = postcard::to_stdvec(&payload)
            .unwrap_or_else(|_| unreachable!("serializing lowered module payload must succeed"));
        CustomSection {
            name: Self::NAME.into(),
            data: data.into(),
        }
        .encode(sink);
    }
}

impl ModulePayload {
    /// Handles from module.
    fn from_module(module: &Module) -> Self {
        Self {
            name: module.name.clone(),
            imports: module
                .imports
                .iter()
                .map(|(path, type_)| (WirePath::from(path), WireType::from(type_)))
                .collect(),
            globals: module
                .globals
                .iter()
                .map(|(path, type_)| (WirePath::from(path), WireType::from(type_)))
                .collect(),
            functions: module
                .functions
                .iter()
                .map(|(path, function)| (WirePath::from(path), WireFunction::from(function)))
                .collect(),
            function_imports: module
                .function_imports
                .iter()
                .map(|(path, import)| (WirePath::from(path), WireFunctionImport::from(import)))
                .collect(),
            has_memory: module.has_memory,
            start: WirePath::from(&module.start),
            closure_counter: module.closure_counter as u64,
            export_policy: WireExportPolicy::from(module.export_policy),
            source_files: module
                .source_files
                .values()
                .map(|record| (record.file_name.clone(), record.source.clone()))
                .collect(),
        }
    }

    /// Handles try into module.
    fn try_into_module(self) -> Result<Module, ModulePayloadDecodeError> {
        let mut imports = IndexMap::with_capacity(self.imports.len());
        for (path, type_) in self.imports {
            imports.insert(path.into_path(), type_.into_type()?);
        }

        let mut globals = IndexMap::with_capacity(self.globals.len());
        for (path, type_) in self.globals {
            globals.insert(path.into_path(), type_.into_type()?);
        }

        let mut functions = IndexMap::with_capacity(self.functions.len());
        for (path, function) in self.functions {
            functions.insert(path.into_path(), function.try_into_function()?);
        }

        let mut function_imports = IndexMap::with_capacity(self.function_imports.len());
        for (path, function_import) in self.function_imports {
            function_imports.insert(path.into_path(), function_import.into_function_import()?);
        }

        let closure_counter = usize::try_from(self.closure_counter)
            .map_err(|_| ModulePayloadDecodeError::IntegerOverflow)?;
        let source_files = self
            .source_files
            .into_iter()
            .map(|(file_name, source)| (file_name.clone(), SourceFileRecord { file_name, source }))
            .collect::<IndexMap<_, _>>();

        Ok(Module {
            name: self.name,
            imports,
            globals,
            functions,
            function_imports,
            has_memory: self.has_memory,
            sig: TypeSignatureSection::default(),
            export_policy: self.export_policy.into_export_policy(),
            start: self.start.into_path(),
            source_files,
            closure_counter,
            source_file_lookup: HashMap::new(),
        })
    }
}

impl WireFunction {
    /// Converts from one representation to another.
    fn from(function: &Function) -> Self {
        Self {
            parameters: function
                .parameters
                .iter()
                .map(|(path, type_)| (WirePath::from(path), WireType::from(type_)))
                .collect(),
            returns: function.returns.iter().map(WireType::from).collect(),
            variables: function
                .variables
                .iter()
                .map(|(path, type_)| (WirePath::from(path), WireType::from(type_)))
                .collect(),
            ops: function.ops.iter().map(WireInstruction::from).collect(),
            op_origins: function
                .op_origins
                .iter()
                .map(|origin| origin.as_ref().map(WireSourceOrigin::from))
                .collect(),
        }
    }

    /// Handles try into function.
    fn try_into_function(self) -> Result<Function, ModulePayloadDecodeError> {
        let mut parameters = IndexMap::with_capacity(self.parameters.len());
        for (path, type_) in self.parameters {
            parameters.insert(path.into_path(), type_.into_type()?);
        }

        let mut variables = IndexMap::with_capacity(self.variables.len());
        for (path, type_) in self.variables {
            variables.insert(path.into_path(), type_.into_type()?);
        }

        let returns = self
            .returns
            .into_iter()
            .map(WireType::into_type)
            .collect::<Result<Vec<_>, _>>()?;
        let ops = self
            .ops
            .into_iter()
            .map(WireInstruction::into_instruction)
            .collect::<Result<Vec<_>, _>>()?;
        let op_origins = self
            .op_origins
            .into_iter()
            .map(|origin| origin.map(WireSourceOrigin::into_source_origin).transpose())
            .collect::<Result<Vec<_>, _>>()?;
        let op_origins = if op_origins.len() == ops.len() {
            op_origins
        } else {
            vec![None; ops.len()]
        };

        Ok(Function {
            parameters,
            returns,
            variables,
            ops,
            op_origins,
        })
    }
}

impl WireSourceOrigin {
    /// Converts from one representation to another.
    fn from(origin: &SourceOrigin) -> Self {
        Self {
            file_name: origin.file_name.clone(),
            start: origin.start as u64,
            width: origin.width as u64,
        }
    }

    /// Handles into source origin.
    fn into_source_origin(self) -> Result<SourceOrigin, ModulePayloadDecodeError> {
        Ok(SourceOrigin {
            file_name: self.file_name,
            start: usize::try_from(self.start)
                .map_err(|_| ModulePayloadDecodeError::IntegerOverflow)?,
            width: usize::try_from(self.width)
                .map_err(|_| ModulePayloadDecodeError::IntegerOverflow)?,
        })
    }
}

impl WireFunctionImport {
    /// Converts from one representation to another.
    fn from(function_import: &FunctionImport) -> Self {
        Self {
            module: function_import.module.clone(),
            name: function_import.name.clone(),
            params: function_import.params.iter().map(WireType::from).collect(),
            results: function_import.results.iter().map(WireType::from).collect(),
        }
    }

    /// Handles into function import.
    fn into_function_import(self) -> Result<FunctionImport, ModulePayloadDecodeError> {
        let params = self
            .params
            .into_iter()
            .map(WireType::into_type)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        let results = self
            .results
            .into_iter()
            .map(WireType::into_type)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        Ok(FunctionImport {
            module: self.module,
            name: self.name,
            params,
            results,
        })
    }
}

impl WirePath {
    /// Converts from one representation to another.
    fn from(path: &Path) -> Self {
        Self {
            major: path.major.clone(),
            minor: path.minor.clone(),
        }
    }

    /// Handles into path.
    fn into_path(self) -> Path {
        Path {
            major: self.major,
            minor: self.minor,
        }
    }
}

impl WireType {
    /// Converts from one representation to another.
    fn from(type_: &Type) -> Self {
        match type_ {
            Type::Any => Self::Any,
            Type::I8 => Self::I8,
            Type::I16 => Self::I16,
            Type::I32 => Self::I32,
            Type::I64 => Self::I64,
            Type::F32 => Self::F32,
            Type::F64 => Self::F64,
            Type::Struct(fields) => Self::Struct(fields.iter().map(Self::from).collect()),
            Type::Array(inner) => Self::Array(Box::new(Self::from(inner))),
            Type::Function {
                parameters,
                results,
            } => {
                Self::Function {
                    parameters: parameters.iter().map(Self::from).collect(),
                    results: results.iter().map(Self::from).collect(),
                }
            }
        }
    }

    /// Handles into type.
    fn into_type(self) -> Result<Type, ModulePayloadDecodeError> {
        Ok(match self {
            Self::Any => Type::Any,
            Self::I8 => Type::I8,
            Self::I16 => Type::I16,
            Self::I32 => Type::I32,
            Self::I64 => Type::I64,
            Self::F32 => Type::F32,
            Self::F64 => Type::F64,
            Self::Struct(fields) => {
                Type::Struct(
                    fields
                        .into_iter()
                        .map(Self::into_type)
                        .collect::<Result<Vec<_>, _>>()?
                        .into_boxed_slice(),
                )
            }
            Self::Array(inner) => Type::Array(Box::new(inner.into_type()?)),
            Self::Function {
                parameters,
                results,
            } => {
                Type::Function {
                    parameters: parameters
                        .into_iter()
                        .map(Self::into_type)
                        .collect::<Result<Vec<_>, _>>()?
                        .into_boxed_slice(),
                    results: results
                        .into_iter()
                        .map(Self::into_type)
                        .collect::<Result<Vec<_>, _>>()?
                        .into_boxed_slice(),
                }
            }
        })
    }
}

impl WireExportPolicy {
    /// Converts from one representation to another.
    fn from(export_policy: ExportPolicy) -> Self {
        match export_policy {
            ExportPolicy::MinorOnly => Self::MinorOnly,
            ExportPolicy::Qualified => Self::Qualified,
            ExportPolicy::None => Self::None,
        }
    }

    /// Handles into export policy.
    fn into_export_policy(self) -> ExportPolicy {
        match self {
            Self::MinorOnly => ExportPolicy::MinorOnly,
            Self::Qualified => ExportPolicy::Qualified,
            Self::None => ExportPolicy::None,
        }
    }
}

impl WireNumberOperation {
    /// Converts from one representation to another.
    fn from(number_operation: NumberOperation) -> Self {
        match number_operation {
            NumberOperation::Eq => Self::Eq,
            NumberOperation::Ne => Self::Ne,
            NumberOperation::Gt => Self::Gt,
            NumberOperation::Lt => Self::Lt,
            NumberOperation::Ge => Self::Ge,
            NumberOperation::Le => Self::Le,
            NumberOperation::Add => Self::Add,
            NumberOperation::Sub => Self::Sub,
            NumberOperation::Mul => Self::Mul,
            NumberOperation::Div => Self::Div,
            NumberOperation::Rem => Self::Rem,
            NumberOperation::And => Self::And,
            NumberOperation::Or => Self::Or,
            NumberOperation::Xor => Self::Xor,
        }
    }

    /// Handles into number operation.
    fn into_number_operation(self) -> NumberOperation {
        match self {
            Self::Eq => NumberOperation::Eq,
            Self::Ne => NumberOperation::Ne,
            Self::Gt => NumberOperation::Gt,
            Self::Lt => NumberOperation::Lt,
            Self::Ge => NumberOperation::Ge,
            Self::Le => NumberOperation::Le,
            Self::Add => NumberOperation::Add,
            Self::Sub => NumberOperation::Sub,
            Self::Mul => NumberOperation::Mul,
            Self::Div => NumberOperation::Div,
            Self::Rem => NumberOperation::Rem,
            Self::And => NumberOperation::And,
            Self::Or => NumberOperation::Or,
            Self::Xor => NumberOperation::Xor,
        }
    }
}

impl WireImmediateValue {
    /// Converts from one representation to another.
    fn from(immediate_value: &ImmediateValue) -> Self {
        match immediate_value {
            ImmediateValue::Unit => Self::Unit,
            ImmediateValue::Integer(value) => Self::Integer(*value),
            ImmediateValue::Real(value) => Self::Real(*value),
            ImmediateValue::Boolean(value) => Self::Boolean(*value),
            ImmediateValue::String(value) => Self::String(value.clone()),
            ImmediateValue::Glyph(value) => Self::Glyph(*value),
        }
    }

    /// Handles into immediate value.
    fn into_immediate_value(self) -> ImmediateValue {
        match self {
            Self::Unit => ImmediateValue::Unit,
            Self::Integer(value) => ImmediateValue::Integer(value),
            Self::Real(value) => ImmediateValue::Real(value),
            Self::Boolean(value) => ImmediateValue::Boolean(value),
            Self::String(value) => ImmediateValue::String(value),
            Self::Glyph(value) => ImmediateValue::Glyph(value),
        }
    }
}

impl WireInstruction {
    /// Converts from one representation to another.
    fn from(instruction: &Instruction) -> Self {
        match instruction {
            Instruction::Set(path) => Self::Set(WirePath::from(path)),
            Instruction::Get(path) => Self::Get(WirePath::from(path)),
            Instruction::Const(value) => Self::Const(WireImmediateValue::from(value)),
            Instruction::I32Const(value) => Self::I32Const(*value),
            Instruction::F32Const(value) => Self::F32Const(*value),
            Instruction::Func(path) => Self::Func(WirePath::from(path)),
            Instruction::StructNew(fields) => {
                Self::StructNew(fields.iter().map(WireType::from).collect())
            }
            Instruction::StructGet(fields, index) => {
                Self::StructGet(fields.iter().map(WireType::from).collect(), *index as u64)
            }
            Instruction::ArrayGet(type_) => Self::ArrayGet(WireType::from(type_)),
            Instruction::ArrayNewFixed { inner_type, length } => {
                Self::ArrayNewFixed {
                    inner_type: WireType::from(inner_type),
                    length: *length as u64,
                }
            }
            Instruction::ArrayNewDefault(type_) => Self::ArrayNewDefault(WireType::from(type_)),
            Instruction::ArrayLen => Self::ArrayLen,
            Instruction::ArrayCopy { dst_type, src_type } => {
                Self::ArrayCopy {
                    dst_type: WireType::from(dst_type),
                    src_type: WireType::from(src_type),
                }
            }
            Instruction::CallRef {
                parameters,
                returns,
            } => {
                Self::CallRef {
                    parameters: parameters.iter().map(WireType::from).collect(),
                    returns: returns.iter().map(WireType::from).collect(),
                }
            }
            Instruction::Call(path) => Self::Call(WirePath::from(path)),
            Instruction::Unreachable => Self::Unreachable,
            Instruction::Drop => Self::Drop,
            Instruction::If(result) => Self::If(result.as_ref().map(WireType::from)),
            Instruction::Else => Self::Else,
            Instruction::End => Self::End,
            Instruction::Loop => Self::Loop,
            Instruction::Block(result) => Self::Block(result.as_ref().map(WireType::from)),
            Instruction::Break(depth) => Self::Break(*depth as u64),
            Instruction::BreakIf(depth) => Self::BreakIf(*depth as u64),
            Instruction::I32Op(operation) => Self::I32Op(WireNumberOperation::from(*operation)),
            Instruction::I64Op(operation) => Self::I64Op(WireNumberOperation::from(*operation)),
            Instruction::F32Op(operation) => Self::F32Op(WireNumberOperation::from(*operation)),
            Instruction::F64Op(operation) => Self::F64Op(WireNumberOperation::from(*operation)),
            Instruction::RefCastFunc {
                parameters,
                returns,
            } => {
                Self::RefCastFunc {
                    parameters: parameters.iter().map(WireType::from).collect(),
                    returns: returns.iter().map(WireType::from).collect(),
                }
            }
            Instruction::RefCastStruct(fields) => {
                Self::RefCastStruct(fields.iter().map(WireType::from).collect())
            }
            Instruction::RefCastArray(inner) => Self::RefCastArray(Box::new(WireType::from(inner))),
            Instruction::I32Store8 => Self::I32Store8,
            Instruction::I32Load => Self::I32Load,
            Instruction::I32Store => Self::I32Store,
            Instruction::I64Load => Self::I64Load,
            Instruction::I64ExtendI32U => Self::I64ExtendI32U,
            Instruction::I32WrapI64 => Self::I32WrapI64,
            Instruction::I32TruncF32S => Self::I32TruncF32S,
            Instruction::I32TruncF32U => Self::I32TruncF32U,
            Instruction::I32TruncF64S => Self::I32TruncF64S,
            Instruction::I32TruncF64U => Self::I32TruncF64U,
            Instruction::I64TruncF32S => Self::I64TruncF32S,
            Instruction::I64TruncF32U => Self::I64TruncF32U,
            Instruction::I64TruncF64S => Self::I64TruncF64S,
            Instruction::I64TruncF64U => Self::I64TruncF64U,
            Instruction::F32DemoteF64 => Self::F32DemoteF64,
        }
    }

    /// Handles into instruction.
    fn into_instruction(self) -> Result<Instruction, ModulePayloadDecodeError> {
        Ok(match self {
            Self::Set(path) => Instruction::Set(path.into_path()),
            Self::Get(path) => Instruction::Get(path.into_path()),
            Self::Const(value) => Instruction::Const(value.into_immediate_value()),
            Self::I32Const(value) => Instruction::I32Const(value),
            Self::F32Const(value) => Instruction::F32Const(value),
            Self::Func(path) => Instruction::Func(path.into_path()),
            Self::StructNew(fields) => {
                Instruction::StructNew(
                    fields
                        .into_iter()
                        .map(WireType::into_type)
                        .collect::<Result<Vec<_>, _>>()?
                        .into_boxed_slice(),
                )
            }
            Self::StructGet(fields, index) => {
                Instruction::StructGet(
                    fields
                        .into_iter()
                        .map(WireType::into_type)
                        .collect::<Result<Vec<_>, _>>()?
                        .into_boxed_slice(),
                    usize::try_from(index)
                        .map_err(|_| ModulePayloadDecodeError::IntegerOverflow)?,
                )
            }
            Self::ArrayGet(type_) => Instruction::ArrayGet(type_.into_type()?),
            Self::ArrayNewFixed { inner_type, length } => {
                Instruction::ArrayNewFixed {
                    inner_type: inner_type.into_type()?,
                    length: usize::try_from(length)
                        .map_err(|_| ModulePayloadDecodeError::IntegerOverflow)?,
                }
            }
            Self::ArrayNewDefault(type_) => Instruction::ArrayNewDefault(type_.into_type()?),
            Self::ArrayLen => Instruction::ArrayLen,
            Self::ArrayCopy { dst_type, src_type } => {
                Instruction::ArrayCopy {
                    dst_type: dst_type.into_type()?,
                    src_type: src_type.into_type()?,
                }
            }
            Self::CallRef {
                parameters,
                returns,
            } => {
                Instruction::CallRef {
                    parameters: parameters
                        .into_iter()
                        .map(WireType::into_type)
                        .collect::<Result<Vec<_>, _>>()?
                        .into_boxed_slice(),
                    returns: returns
                        .into_iter()
                        .map(WireType::into_type)
                        .collect::<Result<Vec<_>, _>>()?
                        .into_boxed_slice(),
                }
            }
            Self::Call(path) => Instruction::Call(path.into_path()),
            Self::Unreachable => Instruction::Unreachable,
            Self::Drop => Instruction::Drop,
            Self::If(result) => Instruction::If(result.map(WireType::into_type).transpose()?),
            Self::Else => Instruction::Else,
            Self::End => Instruction::End,
            Self::Loop => Instruction::Loop,
            Self::Block(result) => Instruction::Block(result.map(WireType::into_type).transpose()?),
            Self::Break(depth) => {
                Instruction::Break(
                    usize::try_from(depth)
                        .map_err(|_| ModulePayloadDecodeError::IntegerOverflow)?,
                )
            }
            Self::BreakIf(depth) => {
                Instruction::BreakIf(
                    usize::try_from(depth)
                        .map_err(|_| ModulePayloadDecodeError::IntegerOverflow)?,
                )
            }
            Self::I32Op(operation) => Instruction::I32Op(operation.into_number_operation()),
            Self::I64Op(operation) => Instruction::I64Op(operation.into_number_operation()),
            Self::F32Op(operation) => Instruction::F32Op(operation.into_number_operation()),
            Self::F64Op(operation) => Instruction::F64Op(operation.into_number_operation()),
            Self::RefCastFunc {
                parameters,
                returns,
            } => {
                Instruction::RefCastFunc {
                    parameters: parameters
                        .into_iter()
                        .map(WireType::into_type)
                        .collect::<Result<Vec<_>, _>>()?
                        .into_boxed_slice(),
                    returns: returns
                        .into_iter()
                        .map(WireType::into_type)
                        .collect::<Result<Vec<_>, _>>()?
                        .into_boxed_slice(),
                }
            }
            Self::RefCastStruct(fields) => {
                Instruction::RefCastStruct(
                    fields
                        .into_iter()
                        .map(WireType::into_type)
                        .collect::<Result<Vec<_>, _>>()?
                        .into_boxed_slice(),
                )
            }
            Self::RefCastArray(inner) => Instruction::RefCastArray(Box::new(inner.into_type()?)),
            Self::I32Store8 => Instruction::I32Store8,
            Self::I32Load => Instruction::I32Load,
            Self::I32Store => Instruction::I32Store,
            Self::I64Load => Instruction::I64Load,
            Self::I64ExtendI32U => Instruction::I64ExtendI32U,
            Self::I32WrapI64 => Instruction::I32WrapI64,
            Self::I32TruncF32S => Instruction::I32TruncF32S,
            Self::I32TruncF32U => Instruction::I32TruncF32U,
            Self::I32TruncF64S => Instruction::I32TruncF64S,
            Self::I32TruncF64U => Instruction::I32TruncF64U,
            Self::I64TruncF32S => Instruction::I64TruncF32S,
            Self::I64TruncF32U => Instruction::I64TruncF32U,
            Self::I64TruncF64S => Instruction::I64TruncF64S,
            Self::I64TruncF64U => Instruction::I64TruncF64U,
            Self::F32DemoteF64 => Instruction::F32DemoteF64,
        })
    }
}

/// Handles decode named custom section data.
fn decode_named_custom_section_data(data: &[u8]) -> Option<(&str, &[u8])> {
    let (name_length, name_length_bytes) = decode_leb128_usize(data)?;
    let name_start = name_length_bytes;
    let name_end = name_start.checked_add(name_length)?;
    let name = std::str::from_utf8(data.get(name_start..name_end)?).ok()?;
    Some((name, data.get(name_end..)?))
}

/// Handles decode leb128 usize.
fn decode_leb128_usize(data: &[u8]) -> Option<(usize, usize)> {
    let mut value = 0usize;
    let mut shift = 0;

    for (index, byte) in data.iter().copied().enumerate() {
        value |= ((byte & 0x7F) as usize) << shift;
        if byte & 0x80 == 0 {
            return Some((value, index + 1));
        }
        shift += 7;
        if shift >= usize::BITS {
            return None;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    /// Handles roundtrip module section.
    fn roundtrip_module_section() {
        let mut module = Module::new("demo".to_string());
        module.export_policy = ExportPolicy::Qualified;
        module.start = Path::new("demo", "[init]");
        module
            .globals
            .insert(Path::new("demo", "value"), Type::Struct([Type::I64].into()));
        module.imports.insert(
            Path::new("core", "default"),
            Type::Struct([Type::Any].into()),
        );
        module.functions.insert(
            Path::new("demo", "[init]"),
            Function {
                parameters: IndexMap::new(),
                returns: Vec::new(),
                variables: IndexMap::new(),
                ops: vec![
                    Instruction::Get(Path::new("demo", "value")),
                    Instruction::Drop,
                ],
                op_origins: vec![None, None],
            },
        );

        let section = LoweredModuleSection::new(&module);
        let mut encoded = Vec::new();
        section.encode(&mut encoded);

        let (section_size, section_size_bytes) = decode_leb128_usize(&encoded).unwrap();
        let section_data_start = section_size_bytes;
        let section_data_end = section_data_start + section_size;
        let section_data = &encoded[section_data_start..section_data_end];
        let (_, payload_data) = decode_named_custom_section_data(section_data).unwrap();
        let decoded = LoweredModuleSection::decode_data_slice(payload_data).unwrap();

        assert_eq!(decoded.name, module.name);
        assert_eq!(decoded.start, module.start);
        assert_eq!(decoded.export_policy, module.export_policy);
        assert_eq!(decoded.globals, module.globals);
        assert_eq!(decoded.imports, module.imports);
        assert_eq!(decoded.functions.len(), module.functions.len());
    }

    #[test]
    /// Handles decode invalid data returns none.
    fn decode_invalid_data_returns_none() {
        assert!(LoweredModuleSection::decode_data_slice(&[1, 2, 3]).is_none());
    }
}
