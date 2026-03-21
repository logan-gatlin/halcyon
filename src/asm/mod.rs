/*!
    # Closures
    Closures are implemented as boxed structs:
    {
        captured_args: [any],
        function: any -> any
    }
    This is because WASM does not support type variables, and so types like
    'a -> 'a are unrepresentable. After a closure is called, it is necessary
    to cast the result to its appropriate type. This should never fail.
*/
pub mod custom_section;
mod encode;
mod lower;
pub mod module_section;
pub mod pretty_print;
mod resolve;
mod verify;

#[cfg(test)]
mod tests;

use custom_section::*;
use indexmap::IndexMap;
use std::collections::HashMap;

use crate::ir::{
    ElaborationResult,
    ImmediateValue,
    Path,
    ScopeKind,
    Statement,
    wasm,
};
use crate::logging::FileId;
use crate::types::SymbolTable;
use crate::{
    Artifact,
    Span,
};

pub use encode::{
    encode,
    encode_with_options,
};
pub(crate) use lower::ConstructorTable;
pub use lower::lower_type;
pub(crate) use resolve::resolve_module;
pub(crate) use verify::verify_module;

#[derive(Debug, Clone)]
pub struct BackendError {
    pub function: Option<Path>,
    pub op_index: Option<usize>,
    pub origin: Option<SourceOrigin>,
    pub message: String,
}

impl BackendError {
    /// Handles module.
    pub fn module(message: impl Into<String>) -> Self {
        Self {
            function: None,
            op_index: None,
            origin: None,
            message: message.into(),
        }
    }

    /// Handles in function.
    pub fn in_function(
        function: Path,
        op_index: Option<usize>,
        origin: Option<SourceOrigin>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            function: Some(function),
            op_index,
            origin,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for BackendError {
    /// Formats the value for display.
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "backend error: {}", self.message)?;
        if let Some(function) = &self.function {
            write!(f, " (function: {function}")?;
            if let Some(op_index) = self.op_index {
                write!(f, ", op: {op_index}")?;
            }
            write!(f, ")")?;
        }
        if let Some(origin) = &self.origin {
            write!(
                f,
                " [{}:{}+{}]",
                origin.file_name, origin.start, origin.width
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportPolicy {
    #[default]
    MinorOnly,
    Qualified,
    None,
}

impl ExportPolicy {
    /// Handles global export name.
    fn global_export_name(
        self,
        path: &Path,
    ) -> Option<String> {
        match self {
            Self::MinorOnly => Some(path.minor.clone()),
            Self::Qualified => Some(format!("{path}")),
            Self::None => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceOrigin {
    pub file_name: String,
    pub start: usize,
    pub width: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFileRecord {
    pub file_name: String,
    pub source: String,
}

pub type SourceCatalog = Vec<(FileId, String, String)>;

#[derive(Debug, Clone, Default)]
pub struct EncodedModule {
    pub binary: Vec<u8>,
    pub source_map: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DebugInfoOptions {
    pub emit_source_map: bool,
    pub emit_dwarf: bool,
}

impl DebugInfoOptions {
    pub const fn all() -> Self {
        Self {
            emit_source_map: true,
            emit_dwarf: true,
        }
    }

    pub const fn none() -> Self {
        Self {
            emit_source_map: false,
            emit_dwarf: false,
        }
    }
}

impl Default for DebugInfoOptions {
    fn default() -> Self {
        Self::all()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Any,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Struct(Box<[Type]>),
    Array(Box<Type>),
    Function {
        parameters: Box<[Type]>,
        results: Box<[Type]>,
    },
}

impl std::fmt::Display for Type {
    /// Formats the value for display.
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Type::Any => write!(f, "any"),
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Struct(items) => {
                write!(
                    f,
                    "(struct {})",
                    items
                        .iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
            Type::Array(t) => write!(f, "(array {t})"),
            Type::Function {
                parameters,
                results,
            } => {
                write!(
                    f,
                    "(func {} {})",
                    parameters
                        .into_iter()
                        .map(|p| format!("(param {p})"))
                        .collect::<Vec<_>>()
                        .join(" "),
                    results
                        .into_iter()
                        .map(|p| format!("(result {p})"))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
        }
    }
}

impl Type {
    /// Handles is reftype.
    pub fn is_reftype(&self) -> bool {
        matches!(
            self,
            Type::Struct(_) | Type::Array(_) | Type::Function { .. } | Type::Any
        )
    }
    /// Handles is packed.
    pub fn is_packed(&self) -> bool {
        matches!(self, Type::I8 | Type::I16)
    }
    /// Handles unpack.
    pub fn unpack(self) -> Self {
        /// Handles unpack.
        fn unpack(
            this: Type,
            in_ref: bool,
        ) -> Type {
            match this {
                Type::I8 | Type::I16 if !in_ref => Type::I32,
                Type::Struct(t) => Type::Struct(t.into_iter().map(|t| unpack(t, true)).collect()),
                Type::Array(t) => Type::Array(unpack(*t, true).into()),
                Type::Function {
                    parameters,
                    results,
                } => {
                    Type::Function {
                        parameters: parameters.into_iter().map(|t| unpack(t, false)).collect(),
                        results: results.into_iter().map(|t| unpack(t, false)).collect(),
                    }
                }
                t => t,
            }
        }
        unpack(self, false)
    }
    /// Handles structural eq.
    pub fn structural_eq(
        &self,
        other: &Self,
    ) -> bool {
        match (self, other) {
            (t1, t2) if t1 == t2 => true,
            (Self::Any, t) | (t, Self::Any) if t.is_reftype() => true,
            (Self::Struct(t1), Self::Struct(t2)) => {
                t1.iter()
                    .zip(t2.iter())
                    .all(|(t1, t2)| t1.structural_eq(t2))
            }
            (Self::Array(t1), Self::Array(t2)) => t1.structural_eq(t2),
            // There is no structural equality for functions because of
            // contravariance rules
            _ => false,
        }
    }
}

/// Arithmetic and bitwise operations on numbers.
///
/// Stack: `[left, right] -> [result]`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberOperation {
    /// Equal comparison: returns true if left == right.
    ///
    /// Available for: integers and floats
    Eq,

    /// Not-equal comparison: returns true if left != right.
    ///
    /// Available for: integers and floats
    Ne,

    /// Greater-than comparison: returns true if left > right.
    ///
    /// Available for: integers and floats
    Gt,

    /// Less-than comparison: returns true if left < right.
    ///
    /// Available for: integers and floats
    Lt,

    /// Greater-than-or-equal comparison: returns true if left >= right.
    ///
    /// Available for: integers and floats
    Ge,

    /// Less-than-or-equal comparison: returns true if left <= right.
    ///
    /// Available for: integers and floats
    Le,

    /// Arithmetic addition: left + right.
    ///
    /// Available for: integers and floats
    Add,

    /// Arithmetic subtraction: left - right.
    ///
    /// Available for: integers and floats
    Sub,

    /// Arithmetic multiplication: left * right.
    ///
    /// Available for: integers and floats
    Mul,

    /// Arithmetic division: left / right.
    ///
    /// Available for: integers and floats
    Div,

    /// Arithmetic remainder: left % right.
    ///
    /// Available for: integers only
    Rem,

    /// Bitwise AND: left & right.
    ///
    /// Available for: integers only
    And,

    /// Bitwise OR: left | right.
    ///
    /// Available for: integers only
    Or,

    /// Bitwise XOR: left ^ right.
    ///
    /// Available for: integers only
    Xor,
}

/// High-level WebAssembly instruction abstraction.
///
/// Stack notation: `[input] -> [output]` describes the stack effect.
#[derive(Debug, Clone)]
pub enum Instruction {
    /// Store a value from the stack into a local/global variable.
    ///
    /// Stack: `[value] -> []`
    Set(Path),

    /// Load a value from a local/global variable onto the stack.
    ///
    /// Stack: `[] -> [value]`
    Get(Path),

    /// Push a constant value onto the stack.
    ///
    /// Stack: `[] -> [const_value]`
    Const(ImmediateValue),

    I32Const(i32),

    F32Const(f32),

    /// Push a function reference onto the stack.
    ///
    /// Stack: `[] -> [func_ref]`
    Func(Path),

    /// Create a struct from values on the stack.
    ///
    /// Stack: `[field_0, field_1, ..., field_n] -> [struct]`
    ///
    /// Pops n values (where n = number of types) and creates a struct with those fields.
    StructNew(Box<[Type]>),

    /// Extract a field from a struct.
    ///
    /// Stack: `[struct] -> [field_value]`
    ///
    /// Pops a struct and pushes the value of the field at the given index.
    StructGet(Box<[Type]>, usize),

    /// Load from an array at a dynamic index.
    ///
    /// Stack: `[array, index] -> [value]`
    ArrayGet(Type),

    /// Create a fixed-size array from values on the stack.
    ///
    /// Stack: `[elem_0, elem_1, ..., elem_n] -> [array]`
    ///
    /// Pops n values (where n is the parameter) and creates an array with those elements.
    ArrayNewFixed {
        inner_type: Type,
        length: usize,
    },

    /// Create a new array with default-initialized elements.
    ///
    /// Stack: `[length: i32] -> [array]`
    ArrayNewDefault(Type),

    /// Get the length of an array.
    ///
    /// Stack: `[array] -> [i32]`
    ArrayLen,

    /// Copy elements from source array to destination array.
    ///
    /// Stack: `[dst_array, dst_offset, src_array, src_offset, length] -> []`
    ArrayCopy {
        dst_type: Type,
        src_type: Type,
    },

    /// Call a function reference.
    ///
    /// Stack: `[func_ref, argument] -> [result]`
    CallRef {
        parameters: Box<[Type]>,
        returns: Box<[Type]>,
    },

    /// Call a function by name.
    ///
    /// Stack: depends on function signature
    Call(Path),

    /// Mark a code path as unreachable.
    ///
    /// Stack: `[] -> [bottom]`
    Unreachable,

    /// Discard the top value from the stack.
    ///
    /// Stack: `[value] -> []`
    Drop,

    /// Begin a conditional branch.
    ///
    /// Stack: `[condition] -> []` (condition popped, control flow branches)
    ///
    /// Must be matched with `Else` and `End`.
    If(Option<Type>),

    /// Separate the true and false branches of an `If`.
    ///
    /// Stack: `[] -> []`
    ///
    /// Follows an `If` and precedes `End`.
    Else,

    /// End a block, loop, if, or function.
    ///
    /// Stack: depends on context (produces block result if `Block`/`If` has a type)
    End,

    /// Begin a loop that can be branched back to.
    ///
    /// Stack: `[] -> []`
    ///
    /// Must be matched with `End`. Branches to `Loop` restart the loop.
    Loop,

    /// Begin a labeled block.
    ///
    /// Stack: `[] -> []` (produces block result if type is present)
    ///
    /// Must be matched with `End`. `Break` targets this block.
    Block(Option<Type>),

    /// Unconditional branch to a labeled block.
    ///
    /// Stack: `[result_values...] -> [bottom]`
    ///
    /// Jumps to the block at depth n, popping result values if the block expects a type.
    Break(usize),

    /// Conditional branch to a labeled block.
    ///
    /// Stack: `[condition, result_values...] -> []`
    ///
    /// Pops condition; if true, jumps to block at depth n with result values.
    BreakIf(usize),

    /// 32-bit integer arithmetic/comparison operation.
    ///
    /// Stack: `[left, right] -> [result]` (for binary operations)
    I32Op(NumberOperation),

    /// 64-bit integer arithmetic/comparison operation.
    ///
    /// Stack: `[left, right] -> [result]` (for binary operations)
    I64Op(NumberOperation),

    /// 32-bit floating-point arithmetic/comparison operation.
    ///
    /// Stack: `[left, right] -> [result]` (for binary operations)
    F32Op(NumberOperation),

    /// 64-bit floating-point arithmetic/comparison operation.
    ///
    /// Stack: `[left, right] -> [result]` (for binary operations)
    F64Op(NumberOperation),

    /// Cast a reference to a specific function type.
    ///
    /// Stack: `[funcref] -> [(ref null $func_type)]`
    RefCastFunc {
        parameters: Box<[Type]>,
        returns: Box<[Type]>,
    },

    /// Cast an anyref to a specific struct type.
    ///
    /// Stack: `[anyref] -> [(ref null $struct_type)]`
    RefCastStruct(Box<[Type]>),

    /// Cast an anyref to a specific array type.
    ///
    /// Stack: `[anyref] -> [(ref null $array_type)]`
    RefCastArray(Box<Type>),

    /// Store a byte to linear memory.
    ///
    /// Stack: `[address: i32, value: i32] -> []`
    I32Store8,

    /// Load an i32 from linear memory.
    ///
    /// Stack: `[address: i32] -> [value: i32]`
    I32Load,

    /// Store an i32 to linear memory.
    ///
    /// Stack: `[address: i32, value: i32] -> []`
    I32Store,

    /// Load an i64 from linear memory.
    ///
    /// Stack: `[address: i32] -> [value: i64]`
    I64Load,

    /// Zero-extend i32 to i64.
    ///
    /// Stack: `[value: i32] -> [value: i64]`
    I64ExtendI32U,

    /// Truncate i64 to i32.
    ///
    /// Stack: `[value: i64] -> [value: i32]`
    I32WrapI64,

    /// Truncate f32 to signed i32.
    ///
    /// Stack: `[value: f32] -> [value: i32]`
    I32TruncF32S,

    /// Truncate f32 to unsigned i32.
    ///
    /// Stack: `[value: f32] -> [value: i32]`
    I32TruncF32U,

    /// Truncate f64 to signed i32.
    ///
    /// Stack: `[value: f64] -> [value: i32]`
    I32TruncF64S,

    /// Truncate f64 to unsigned i32.
    ///
    /// Stack: `[value: f64] -> [value: i32]`
    I32TruncF64U,

    /// Truncate f32 to signed i64.
    ///
    /// Stack: `[value: f32] -> [value: i64]`
    I64TruncF32S,

    /// Truncate f32 to unsigned i64.
    ///
    /// Stack: `[value: f32] -> [value: i64]`
    I64TruncF32U,

    /// Truncate f64 to signed i64.
    ///
    /// Stack: `[value: f64] -> [value: i64]`
    I64TruncF64S,

    /// Truncate f64 to unsigned i64.
    ///
    /// Stack: `[value: f64] -> [value: i64]`
    I64TruncF64U,

    /// Demote f64 to f32.
    ///
    /// Stack: `[value: f64] -> [value: f32]`
    F32DemoteF64,

    /// Convert signed i64 to f64.
    ///
    /// Stack: `[value: i64] -> [value: f64]`
    F64ConvertI64S,

    /// Convert unsigned i64 to f64.
    ///
    /// Stack: `[value: i64] -> [value: f64]`
    F64ConvertI64U,
}

#[derive(Debug, Clone, Default)]
pub struct Function {
    pub parameters: IndexMap<Path, Type>,
    pub returns: Vec<Type>,
    pub variables: IndexMap<Path, Type>,
    pub ops: Vec<Instruction>,
    pub op_origins: Vec<Option<SourceOrigin>>,
}

#[derive(Debug, Clone)]
pub struct FunctionImport {
    pub module: String,
    pub name: String,
    pub params: Box<[Type]>,
    pub results: Box<[Type]>,
}

#[derive(Debug, Clone, Default)]
pub struct Module {
    pub name: String,
    pub imports: IndexMap<Path, Type>,
    pub globals: IndexMap<Path, Type>,
    pub functions: IndexMap<Path, Function>,
    pub function_imports: IndexMap<Path, FunctionImport>,
    pub has_memory: bool,
    pub sig: TypeSignatureSection,
    pub export_policy: ExportPolicy,
    pub start: Path,
    pub source_files: IndexMap<String, SourceFileRecord>,
    #[doc(hidden)]
    pub closure_counter: usize,
    #[doc(hidden)]
    pub source_file_lookup: HashMap<FileId, String>,
}

impl Type {
    /// Handles function capture.
    pub fn function_capture() -> Self {
        Self::Array(Self::Any.into())
    }

    /// Handles closure function type.
    pub fn closure_function_type() -> Self {
        Self::Function {
            parameters: [Self::function_capture(), Self::Any].into(),
            results: [Self::Any].into(),
        }
    }

    /// Handles closure type.
    pub fn closure_type() -> Self {
        Self::Struct([Self::function_capture(), Self::closure_function_type()].into())
    }
}

impl Function {
    /// Creates a new instance.
    pub fn new() -> Self {
        Default::default()
    }
}

#[tracing::instrument(skip_all, fields(module = %elaborated.module.name))]
/// Handles lower module.
pub fn lower_module(
    elaborated: ElaborationResult,
    symbols: &SymbolTable,
    source_catalog: &SourceCatalog,
) -> Module {
    let ir_module = elaborated.module;
    let mut module = Module::new(ir_module.name.clone());
    module.ingest_source_catalog(source_catalog);
    let constructor_table = ConstructorTable::from_symbols(symbols);

    let init_name = Path::new(&ir_module.name, "[init]");
    let mut init_func = module.new_function(init_name.clone());

    for (path, info) in constructor_table.constructors_for_module(&ir_module.name) {
        if !symbols.terms().contains_key(&path) {
            continue;
        }
        if symbols.constructor_aliases().contains_key(&path) {
            continue;
        }
        init_func.lower_constructor(path, &info, symbols);
    }

    for trait_def in symbols
        .trait_defs()
        .values()
        .filter(|trait_def| trait_def.name.major == ir_module.name)
    {
        let mut method_paths = trait_def.methods.keys().cloned().collect::<Vec<_>>();
        method_paths.sort_by(|left, right| {
            (left.major.clone(), left.minor.clone())
                .cmp(&(right.major.clone(), right.minor.clone()))
        });
        for method_path in method_paths {
            init_func.lower_trait_method_dispatch(method_path, symbols);
        }
    }

    for statement in ir_module.statements {
        match statement {
            Statement::Term(term) => {
                init_func.lower_ir(term, symbols, &constructor_table);
                init_func.push(Instruction::Drop);
            }
            Statement::ConstructorAlias { path, target, .. } => {
                init_func.lower_constructor_alias(path, target, symbols);
            }
            Statement::Wasm(declarations) => {
                lower_wasm_declarations(init_func.module, &ir_module.name, declarations);
            }
            Statement::Type { .. }
            | Statement::Trait { .. }
            | Statement::TraitAlias { .. }
            | Statement::Impl { .. } => {}
        }
    }
    module.start = init_name;
    module.sig = TypeSignatureSection::new(&ir_module.name, symbols);
    module
}

/// Handles lower wasm declarations.
fn lower_wasm_declarations(
    module: &mut Module,
    module_name: &str,
    declarations: Box<[wasm::Declaration]>,
) {
    for declaration in declarations {
        match declaration {
            wasm::Declaration::Type(_) => {}
            wasm::Declaration::Global(global) => {
                assert!(
                    module
                        .globals
                        .insert(global.name.clone(), global.type_.clone())
                        .is_none(),
                    "Redefinition of global symbol {}",
                    global.name
                );
            }
            wasm::Declaration::Function(function) => {
                let function_path = Path::new(module_name, function.name);
                let op_origin = module.source_origin_for_span(function.span);
                assert!(
                    module
                        .functions
                        .insert(
                            function_path.clone(),
                            Function {
                                parameters: function.parameters,
                                returns: function.results.into_vec(),
                                variables: function.locals,
                                ops: function.body.to_vec(),
                                op_origins: vec![op_origin; function.body.len()],
                            },
                        )
                        .is_none(),
                    "Redefinition of function symbol {function_path}"
                );
            }
            wasm::Declaration::Memory(_) => {
                module.has_memory = true;
            }
            wasm::Declaration::Import(import) => {
                module
                    .function_imports
                    .entry(import.local_name)
                    .or_insert(FunctionImport {
                        module: import.wasm_module,
                        name: import.wasm_name,
                        params: import.params,
                        results: import.results,
                    });
            }
        }
    }
}

#[tracing::instrument(skip_all, fields(module = %elaborated.module.name))]
/// Handles compile module.
pub fn compile_module(
    elaborated: ElaborationResult,
    symbols: &SymbolTable,
    source_catalog: &SourceCatalog,
    debug_info: DebugInfoOptions,
) -> Artifact {
    let _profile_total = crate::profiling::scope("asm.compile_module.total");
    let ir_module = elaborated.module.clone();
    let module_name = ir_module.name.clone();
    let module = {
        let _profile = crate::profiling::scope("asm.compile_module.lower_module");
        lower_module(elaborated, symbols, source_catalog)
    };
    let encoded = {
        let _profile = crate::profiling::scope("asm.compile_module.encode");
        encode_with_options(module, debug_info)
    };
    Artifact {
        module_name,
        ir_module: Some(ir_module),
        binary: encoded.binary,
        source_map: encoded.source_map,
    }
}

impl Module {
    /// Creates a new instance.
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
    /// Handles new function.
    pub fn new_function<'a>(
        &'a mut self,
        name: Path,
    ) -> Encoder<'a> {
        self.functions.insert(name.clone(), Function::new());
        Encoder {
            func_name: name,
            module: self,
            temporary_salt: 0,
            current_origin: None,
            recursive_binding: None,
        }
    }

    /// Handles ingest source catalog.
    fn ingest_source_catalog(
        &mut self,
        source_catalog: &SourceCatalog,
    ) {
        for (file_id, file_name, source) in source_catalog {
            self.source_file_lookup.insert(*file_id, file_name.clone());
            self.source_files
                .entry(file_name.clone())
                .or_insert_with(|| {
                    SourceFileRecord {
                        file_name: file_name.clone(),
                        source: source.clone(),
                    }
                });
        }
    }

    /// Handles source origin for span.
    fn source_origin_for_span(
        &self,
        span: Span,
    ) -> Option<SourceOrigin> {
        let Span::Source {
            start,
            width,
            file_id,
        } = span
        else {
            return None;
        };
        let file_name = self.source_file_lookup.get(&file_id?)?.clone();
        Some(SourceOrigin {
            file_name,
            start,
            width,
        })
    }
}

pub struct Encoder<'a> {
    pub module: &'a mut Module,
    pub func_name: Path,
    pub temporary_salt: usize,
    pub current_origin: Option<SourceOrigin>,
    pub recursive_binding: Option<Path>,
}

impl<'a> Encoder<'a> {
    /// Handles current func.
    fn current_func(&mut self) -> &mut Function {
        self.module
            .functions
            .get_mut(&self.func_name)
            .unwrap_or_else(|| unreachable!("Function {} not found", self.func_name))
    }
    /// Handles push.
    pub fn push(
        &mut self,
        instr: Instruction,
    ) {
        let origin = self.current_origin.clone();
        let function = self.current_func();
        function.ops.push(instr);
        function.op_origins.push(origin);
    }
    /// Handles extend.
    pub fn extend(
        &mut self,
        instrs: impl IntoIterator<Item = Instruction>,
    ) {
        instrs
            .into_iter()
            .for_each(|instruction| self.push(instruction));
    }

    /// Handles with origin.
    pub fn with_origin(
        &mut self,
        origin: Option<SourceOrigin>,
        f: impl FnOnce(&mut Self),
    ) {
        let previous = self.current_origin.clone();
        self.current_origin = origin;
        f(self);
        self.current_origin = previous;
    }
    /// Handles temporary name.
    pub fn temporary_name(
        &mut self,
        name: &str,
    ) -> Path {
        let temp = Path::new("[temp]", format!("{name}#{}", self.temporary_salt));
        self.temporary_salt += 1;
        temp
    }
    /// Handles new parameter.
    pub fn new_parameter(
        &mut self,
        name: Path,
        type_: Type,
    ) {
        self.current_func().parameters.insert(name, type_);
    }
    /// Handles new register.
    pub fn new_register(
        &mut self,
        name: Path,
        scope: ScopeKind,
        type_: Type,
    ) {
        if scope == ScopeKind::Global {
            assert!(
                self.module.globals.insert(name.clone(), type_).is_none(),
                "Redefinition of global symbol {name}"
            );
        } else {
            assert!(
                self.current_func()
                    .variables
                    .insert(name.clone(), type_)
                    .is_none(),
                "Redefinition of local symbol {name}"
            );
        }
    }
    /// Handles new return.
    pub fn new_return(
        &mut self,
        type_: Type,
    ) {
        self.current_func().returns.push(type_);
    }
}
