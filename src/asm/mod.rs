mod lower;
mod serialize;

use std::collections::HashMap;

use crate::ir::{
    ConstValue,
    IrModule,
    Path,
};
use crate::{
    SymbolTable,
    semantic,
};

#[derive(Debug, Clone)]
pub enum Type {
    Any,
    I32,
    I64,
    F32,
    F64,
    Struct(Vec<Type>),
    Array(Box<Type>),
    Function,
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
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Set(Path),
    Get(Path),
    Const(ConstValue),
    Func(usize),
    StructNew(Vec<Type>),
    StructGet(usize),
    ArrayGet(usize),
    Call,
    Unreachable,
    Drop,
    If(Option<Type>),
    Else,
    End,
    Loop,
    Block(Option<Type>),
    Break(usize),
    I32Op(NumberOperation),
    I64Op(NumberOperation),
    F32Op(NumberOperation),
    F64Op(NumberOperation),
}

#[derive(Debug, Clone, Default)]
pub struct Function {
    parameters: HashMap<Path, Type>,
    variables: HashMap<Path, Type>,
    ops: Vec<Instruction>,
}

#[derive(Debug, Clone, Default)]
pub struct Module {
    imports: HashMap<Path, Type>,
    globals: HashMap<Path, Type>,
    functions: Vec<Function>,
    init_function: usize,
}

impl Function {
    pub fn new(
        parameter_name: Path,
        parameter_type: Type,
    ) -> Self {
        Self {
            parameters: {
                let mut parameters = HashMap::new();
                parameters.insert(parameter_name, parameter_type);
                parameters
            },
            ..Default::default()
        }
    }
}

impl Module {
    pub fn new_function<'a>(
        &'a mut self,
        paramter_name: Path,
        parameter_type: Type,
    ) -> Encoder<'a> {
        self.functions
            .push(Function::new(paramter_name, parameter_type));
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
    ) -> &mut Self {
        self.module.functions[self.func_index].ops.push(instr);
        self
    }
    pub fn extend(
        &mut self,
        instrs: impl IntoIterator<Item = Instruction>,
    ) -> &mut Self {
        self.module.functions[self.func_index].ops.extend(instrs);
        self
    }
    pub fn define_symbol(
        &mut self,
        symbol: Path,
        is_global: bool,
        type_: Type,
    ) -> &mut Self {
        if is_global {
            assert!(
                self.module.globals.insert(symbol.clone(), type_).is_none(),
                "Redefinition of global symbol {symbol}"
            );
        } else {
            assert!(
                self.module.functions[self.func_index]
                    .variables
                    .insert(symbol.clone(), type_)
                    .is_none(),
                "Redefinition of local symbol {symbol}"
            );
        }
        self
    }

    pub fn define_temporary(
        &mut self,
        type_: Type,
    ) -> Path {
        let temp = Path::new("@", self.temporary_salt.to_string());
        self.module.functions[self.func_index]
            .variables
            .insert(temp.clone(), type_);
        self.temporary_salt += 1;
        temp
    }
}
