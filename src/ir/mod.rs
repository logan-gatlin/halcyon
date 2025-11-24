mod build_ir;
mod build_types;
mod names;
mod pattern;
mod pretty_print;
mod symbol_table;
use indexmap::IndexMap;
pub use names::*;
pub use pattern::*;
pub use symbol_table::*;

use std::collections::HashMap;

use crate::parse::ParsedModule;
use crate::semantic::*;
use crate::{
    LogBuilder,
    Logger,
    Span,
    Spanned,
    Visit,
    WithContext,
};
pub use pretty_print::*;

type Result<'a, T> = std::result::Result<T, LogBuilder<'a>>;

pub fn build_ir(
    logger: &mut Logger,
    symbols: &mut SymbolTable,
    module: ParsedModule,
) -> IrModule {
    build_ir::Builder::build_ir(logger, symbols, module)
}

#[derive(Debug, Clone)]
pub enum IrKind {
    Let {
        /// The pattern to compare against
        assignee: Pattern,
        /// Whether this is a module-level let
        is_global: bool,
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
        index: Spanned<String>,
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
    pub code: Vec<IrNode>,
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
