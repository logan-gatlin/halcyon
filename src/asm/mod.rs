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
mod encode;
mod lower;
pub mod pretty_print;
pub mod type_encoder;

use indexmap::IndexMap;
use type_encoder::*;

use crate::ir::{
    ConstValue,
    Path,
};
use crate::{
    SymbolTable,
    semantic,
};

pub use encode::encode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScopeKind {
    Local,
    Global,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Any,
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
    Const(ConstValue),

    I32Const(i32),

    /// Push a function reference onto the stack.
    ///
    /// Stack: `[] -> [func_ref]`
    Func(usize),

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

    /// Call a function.
    ///
    /// Stack: `[func_ref, argument] -> [result]`
    Call {
        parameters: Box<[Type]>,
        returns: Box<[Type]>,
    },

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
}

#[derive(Debug, Clone, Default)]
pub struct Function {
    parameters: IndexMap<Path, Type>,
    returns: Vec<Type>,
    variables: IndexMap<Path, Type>,
    ops: Vec<Instruction>,
}

#[derive(Debug, Clone, Default)]
pub struct Module {
    pub name: String,
    pub imports: IndexMap<Path, Type>,
    pub globals: IndexMap<Path, Type>,
    pub functions: Vec<Function>,
    pub sig: SignatureSection,
}

impl Type {
    pub fn function_capture() -> Self {
        Self::Array(Self::Any.into())
    }

    pub fn closure_function_type() -> Self {
        Self::Function {
            parameters: [Self::function_capture(), Self::Any].into(),
            results: [Self::Any].into(),
        }
    }

    pub fn closure_type() -> Self {
        Self::Struct([Self::function_capture(), Self::closure_function_type()].into())
    }
}

impl Function {
    pub fn new() -> Self {
        Default::default()
    }
}

pub fn lower_module(
    ir_module: crate::ir::Module,
    symbols: &SymbolTable,
) -> Module {
    let mut module = Module::new(ir_module.name.clone());
    let mut init_func = module.new_function();

    // Lower constructors first so they're available as globals
    for (path, cons) in ir_module.constructors {
        init_func.lower_constructor(path, cons, symbols);
    }

    for code in ir_module.code {
        init_func.lower_ir(code, symbols);
        init_func.push(Instruction::Drop);
    }
    module.sig = SignatureSection::new(&ir_module.name, symbols);
    module
}

impl Module {
    fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
    fn new_function<'a>(&'a mut self) -> Encoder<'a> {
        self.functions.push(Function::new());
        Encoder {
            func_index: self.functions.len() - 1,
            module: self,
            temporary_salt: 0,
        }
    }
}

pub struct Encoder<'a> {
    pub module: &'a mut Module,
    pub func_index: usize,
    pub temporary_salt: usize,
}

impl<'a> Encoder<'a> {
    pub fn push(
        &mut self,
        instr: Instruction,
    ) {
        self.module.functions[self.func_index].ops.push(instr);
    }
    pub fn extend(
        &mut self,
        instrs: impl IntoIterator<Item = Instruction>,
    ) {
        self.module.functions[self.func_index].ops.extend(instrs);
    }
    pub fn temporary_name(
        &mut self,
        name: &str,
    ) -> Path {
        let temp = Path::new("[temp]", format!("{name}#{}", self.temporary_salt));
        self.temporary_salt += 1;
        temp
    }
    pub fn new_parameter(
        &mut self,
        name: Path,
        type_: Type,
    ) {
        self.module.functions[self.func_index]
            .parameters
            .insert(name, type_);
    }
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
                self.module.functions[self.func_index]
                    .variables
                    .insert(name.clone(), type_)
                    .is_none(),
                "Redefinition of local symbol {name}"
            );
        }
    }
    pub fn new_return(
        &mut self,
        type_: Type,
    ) {
        self.module.functions[self.func_index].returns.push(type_);
    }
}
