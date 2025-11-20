mod build_ir;
mod names;
mod pattern;
mod pretty_print;
use indexmap::IndexMap;
pub use names::*;
pub use pattern::*;

use std::collections::HashMap;

use crate::parse::ParsedModule;
use crate::semantic::*;
use crate::{
    Logger,
    Spanned,
    Visit,
};
pub use pretty_print::*;

pub fn build_ir(
    logger: &mut Logger,
    module: ParsedModule,
) -> IrModule {
    build_ir::Builder::build_ir(logger, module)
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    pub terms: HashMap<Path, Type>,
    pub types: HashMap<Path, AbstractType>,
    pub constructors: HashMap<Path, Constructor>,
}

#[derive(Debug, Clone)]
pub enum IrKind {
    Let {
        /// The pattern to compare against
        assignee: Pattern,
        /// The value which is compared with the value
        value: Box<IrNode>,
        /// The branch which is taken if the pattern binding succeeds
        then: Box<IrNode>,
        /// The branch which is taken if the pattern binding fails
        else_: Box<IrNode>,
    },
    Immediate(ConstValue),
    Identifier(Path),
    Tuple(Vec<IrNode>),
    Struct(IndexMap<String, IrNode>),
    Field {
        of: Box<IrNode>,
        index: String,
    },
    Function {
        parameter_name: Spanned<Path>,
        parameter_type: Option<Type>,
        captures: Vec<Path>,
        capture_types: Vec<Type>,
        body: Box<IrNode>,
    },
    Call {
        callee: Box<IrNode>,
        argument: Box<IrNode>,
    },
    // The `;` operator is singled out because of the opportunity for tail call optimization
    Semicolon(Box<IrNode>, Box<IrNode>),
}

pub type IrNode = Typed<Spanned<IrKind>>;

#[derive(Debug, Clone)]
pub struct IrModule {
    pub module_name: String,
    pub constructors: Vec<Constructor>,
    pub type_definitions: Vec<Typed<Spanned<Path>>>,
    pub let_definitions: Vec<(Pattern, IrNode)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConstValue {
    Unit,
    Integer(i64),
    Real(f64),
    Boolean(bool),
    String(String),
    Glyph(char),
}

impl std::fmt::Display for ConstValue {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            ConstValue::Unit => write!(f, "()"),
            ConstValue::String(s) => write!(f, "\"{s}\""),
            ConstValue::Integer(val) => write!(f, "{val}"),
            ConstValue::Real(val) => write!(f, "{val}"),
            ConstValue::Glyph(val) => write!(f, "'{val}'"),
            ConstValue::Boolean(val) => write!(f, "{val}"),
        }
    }
}

impl ConstValue {
    pub fn type_of(&self) -> Type {
        match self {
            ConstValue::Unit => Type::Unit,
            ConstValue::Integer(_) => Type::Integer,
            ConstValue::Real(_) => Type::Real,
            ConstValue::Boolean(_) => Type::Boolean,
            ConstValue::String(_) => Type::String,
            ConstValue::Glyph(_) => Type::Glyph,
        }
    }
}
