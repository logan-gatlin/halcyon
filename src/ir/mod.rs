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
    FileLogger,
    LogBuilder,
    Span,
    Spanned,
    Visit,
    WithContext,
};
pub use pretty_print::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScopeKind {
    Local,
    Global,
}

type Result<'a, T> = std::result::Result<T, LogBuilder<'a>>;

pub fn build_ir(
    logger: &mut FileLogger,
    symbols: &mut SymbolTable,
    module: ParsedModule,
) -> Module {
    build_ir::Builder::build_ir(logger, symbols, module)
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum IrKind {
    Let {
        /// The pattern to compare against
        assignee: Pattern,
        /// Whether this is a module-level let
        scope: ScopeKind,
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
    Struct(IndexMap<Spanned<String>, IrNode>),
    Field {
        of: Box<IrNode>,
        index: Spanned<String>,
    },
    Function {
        parameter_name: Spanned<Path>,
        parameter_type: Option<Type>,
        captures: Vec<Typed<Path>>,
        body: Box<IrNode>,
    },
    Call {
        callee: Box<IrNode>,
        argument: Box<IrNode>,
    },
    // The `;` operator is singled out because of the opportunity for tail call optimization
    Semicolon(Box<IrNode>, Box<IrNode>),
    Unreachable,
}

impl Visit<IrNode> for IrNode {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut IrNode),
    ) {
        match &mut self.inner.inner {
            IrKind::Let {
                value, then, else_, ..
            } => {
                value._visit(f);
                then._visit(f);
                else_._visit(f);
            }
            IrKind::Tuple(inner) => inner._visit(f),
            IrKind::Struct(map) => map._visit(f),
            IrKind::Field { of, .. } => of._visit(f),
            IrKind::Function { body, .. } => body._visit(f),
            IrKind::Call { callee, argument } => {
                callee._visit(f);
                argument._visit(f);
            }
            IrKind::Semicolon(a, b) => {
                a._visit(f);
                b._visit(f);
            }
            _ => {}
        }
        f(self);
    }
}

impl Visit<Type> for IrNode {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut Type),
    ) {
        self._visit(&mut |n: &mut IrNode| {
            match &mut n.inner.inner {
                IrKind::Let { assignee, .. } => assignee._visit(f),
                IrKind::Function {
                    parameter_type,
                    captures,
                    ..
                } => {
                    parameter_type._visit(f);
                    captures.iter_mut().for_each(|c| c.type_._visit(f));
                }
                _ => {}
            }
            n.type_._visit(f);
        });
    }
}

pub type IrNode = Typed<Spanned<IrKind>>;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub types: IndexMap<Path, AbstractType>,
    pub constructors: HashMap<Path, Constructor>,
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
