mod lower;
pub mod pretty_print;
mod serialize;

use indexmap::IndexMap;

use crate::ir::{
    ConstValue,
    Path,
};
use crate::{
    semantic,
    SymbolTable,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScopeKind {
    Local,
    Global,
}

#[derive(Debug, Clone)]
pub enum Type {
    Any,
    I32,
    I64,
    F32,
    F64,
    Struct(Box<[Type]>),
    Array(Box<Type>),
    Function,
}

#[derive(Debug, Clone, Copy)]
pub enum MacroKind {
    Call,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberOperation {
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
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Set(Path),
    Get(Path),
    Const(ConstValue),
    Func(usize),
    StructNew(Box<[Type]>),
    StructGet(Type, usize),
    ArrayGet,
    ArrayNewFixed(usize),
    Call,
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
    Macro(MacroKind),
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
    pub globals: IndexMap<Path, Type>,
    pub functions: Vec<Function>,
}

impl Type {
    pub fn function_capture() -> Self {
        Self::Array(Self::Any.into())
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
    let mut module = Module::default();
    let mut init_func = module.new_function();
    for code in ir_module.code {
        init_func.lower_ir(code, symbols);
    }
    module
}

impl Module {
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
