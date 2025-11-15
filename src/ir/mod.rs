mod build_ir;
mod build_types;
pub mod constant;
pub mod constructor;
mod namespace;
mod path;
mod pattern;

use std::{collections::HashMap, rc::Rc};

use crate::{
    Log, Spanned, Visit,
    compile::{ForeignFunctionType, FunctionEncoder},
    err, optimize,
    semantic::*,
};

pub use build_ir::*;
use build_types::*;
pub use constant::*;
pub use constructor::*;
pub use namespace::*;
pub use path::*;
pub use pattern::*;

#[derive(Debug, Clone, sx::SXRepr)]
pub enum IrKind {
    Let {
        assignee: Pattern,
        value: Box<IrNode>,
        in_: Box<IrNode>,
    },
    Immediate(ConstValue),
    Identifier(Path),
    Tuple(Vec<IrNode>),
    Struct {
        field_names: Vec<String>,
        field_values: Vec<IrNode>,
    },
    Field {
        of: Box<IrNode>,
        index: String,
    },
    Function {
        parameter_name: Option<Spanned<Path>>,
        parameter_type: Option<Type>,
        captures: Vec<Path>,
        capture_types: Vec<Type>,
        body: Box<IrNode>,
    },
    Call {
        callee: Box<IrNode>,
        argument: Box<IrNode>,
        opt: optimize::CallOptimization,
    },
    If {
        predicate: Box<IrNode>,
        then: Box<IrNode>,
        else_: Box<IrNode>,
    },
    Match {
        scrutinee: Box<IrNode>,
        predicates: Vec<Pattern>,
        branches: Vec<IrNode>,
    },
    Semicolon(Box<IrNode>, Box<IrNode>),
    AsmLiteral(AsmLiteral),
    ImportedSymbol(Path, Type),
}

#[derive(Clone)]
pub struct AsmLiteral(pub Rc<dyn Fn(&mut FunctionEncoder)>);

impl std::fmt::Debug for AsmLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AsmLiteral")
    }
}

impl sx::SXRepr for AsmLiteral {
    fn sx(self) -> sx::SX {
        sx::SX::Nil
    }
}

pub type IrNode = Typed<Spanned<IrKind>>;

#[derive(Debug, Clone, sx::SXRepr)]
pub enum ModuleItem {
    Let(Pattern, Box<IrNode>),
    Type(Path),
    Constructor(Path, Constructor),
    Import {
        path: Path,
        type_: ForeignFunctionType,
        major: String,
        minor: String,
    },
}

#[derive(Debug, Clone, sx::SXRepr)]
pub struct IrModule {
    pub module_name: Path,
    pub items: Vec<ModuleItem>,
}

impl Visit<IrNode> for IrNode {
    fn _visit(&mut self, f: &mut impl FnMut(&mut IrNode)) {
        use IrKind::*;
        match &mut *self.inner {
            Let { value, in_, .. } => {
                value._visit(f);
                in_._visit(f);
            }
            Semicolon(a, b) => {
                a._visit(f);
                b._visit(f);
            }
            Field { of, .. } => of._visit(f),
            Call {
                callee, argument, ..
            } => {
                callee._visit(f);
                argument._visit(f);
            }
            Function { body, .. } => {
                body._visit(f);
            }
            If {
                predicate,
                then,
                else_,
            } => {
                predicate._visit(f);
                then._visit(f);
                else_._visit(f);
            }
            Match {
                scrutinee,
                branches,
                ..
            } => {
                scrutinee._visit(f);
                branches._visit(f);
            }
            Tuple(items)
            | Struct {
                field_values: items,
                ..
            } => items._visit(f),
            Immediate(_) => {}
            Identifier(_) => {}
            AsmLiteral(_) => {}
            ImportedSymbol(_, _) => {}
        }
        f(self);
    }
}

impl Visit<Type> for IrNode {
    fn _visit(&mut self, mut f: &mut impl FnMut(&mut Type)) {
        self._visit(&mut |n: &mut IrNode| {
            n.type_.visit(&mut f);
            match &mut ***n {
                IrKind::Let { assignee, .. } => assignee._visit(f),
                IrKind::Function {
                    parameter_type,
                    capture_types,
                    ..
                } => {
                    parameter_type._visit(f);
                    capture_types._visit(f);
                }
                IrKind::Match { predicates, .. } => predicates._visit(f),
                IrKind::ImportedSymbol(_, t) => t._visit(f),
                _ => {}
            }
        })
    }
}

impl Visit<Type> for ModuleItem {
    fn _visit(&mut self, f: &mut impl FnMut(&mut Type)) {
        match self {
            ModuleItem::Let(pattern, node) => {
                pattern._visit(f);
                node._visit(f);
            }
            ModuleItem::Type(_) => {}
            ModuleItem::Constructor(_, cons) => cons._visit(f),
            ModuleItem::Import { type_, .. } => {
                let mut type_: Type = type_.clone().into();
                type_._visit(f);
            }
        }
    }
}

impl Visit<Type> for IrModule {
    fn _visit(&mut self, f: &mut impl FnMut(&mut Type)) {
        self.items._visit(f);
    }
}
