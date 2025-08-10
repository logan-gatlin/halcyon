mod build_ir;
pub mod constant;
mod namespace;
mod path;
mod pattern;
pub mod printing;

use std::collections::HashMap;

use crate::{lint::*, operator::*, semantic::*};

pub use build_ir::*;
pub use constant::*;
pub use namespace::*;
pub use path::*;
pub use pattern::*;

pub type IrPtr = usize;

#[derive(Debug, Clone)]
pub enum IrKind {
    Declaration {
        assignee: Pattern,
        value: IrPtr,
        in_: Option<IrPtr>,
    },
    Immediate(ConstValue),
    Identifier(Path),
    Tuple(Vec<IrPtr>),
    StructLiteral {
        field_names: Vec<String>,
        field_values: Vec<IrPtr>,
    },
    Field {
        of: IrPtr,
        index: String,
    },
    Binary {
        op: BinaryOp,
        left: IrPtr,
        right: IrPtr,
    },
    Unary {
        op: UnaryOp,
        child: IrPtr,
    },
    FunctionDef {
        parameter_name: Option<Path>,
        parameter_span: Span,
        parameter_type: Option<TypeRef>,
        captures: Vec<Path>,
        capture_types: Vec<TypeRef>,
        body: IrPtr,
    },
    FunctionCall {
        callee: IrPtr,
        argument: IrPtr,
    },
    If {
        predicate: IrPtr,
        then: IrPtr,
        else_: Option<IrPtr>,
    },
    Match {
        scrutinee: IrPtr,
        predicates: Vec<Pattern>,
        branches: Vec<IrPtr>,
    },
    ImportedSymbol(Path, TypeRef),
}

#[derive(Debug, Clone)]
pub struct IrNode {
    pub kind: IrKind,
    pub span: Span,
    pub type_: TypeRef,
}

impl Unify for IrNode {
    fn unify(&mut self, tv: TypeVariable, type_: &Type) {
        self.type_.borrow_mut().unify(tv, type_);
        match &mut self.kind {
            IrKind::FunctionDef { capture_types, .. } => {
                capture_types.into_iter().for_each(|old_t| {
                    old_t.unify(tv, type_);
                })
            }
            IrKind::Match { predicates, .. } => {
                predicates.into_iter().for_each(|p| p.unify(tv, type_))
            }
            IrKind::Declaration { assignee, .. } => assignee.unify(tv, type_),
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub enum ModuleItem {
    Let(Pattern, IrPtr),
    Type(Path, TypeRef),
    Constructor(Path, Constructor),
}

#[derive(Debug, Clone)]
pub struct IrModule {
    pub module_name: Path,
    pub universe: HashMap<Path, TypeRef>,
    pub items: Vec<ModuleItem>,
    pub nodes: Vec<IrNode>,
}

impl Unify for IrModule {
    fn unify(&mut self, tv: TypeVariable, type_: &Type) {
        self.universe
            .iter_mut()
            .for_each(|(_, t)| t.borrow_mut().unify(tv, type_));
        self.nodes.iter_mut().for_each(|n| n.unify(tv, type_));
    }
}

impl IrModule {
    pub fn ir_range(&self, start: IrPtr) -> std::ops::Range<IrPtr> {
        let mut current = start;
        loop {
            use IrKind::*;
            current = *match &self[current].kind {
                Declaration { value, in_, .. } => {
                    if let Some(in_) = in_ {
                        in_
                    } else {
                        value
                    }
                }
                FunctionCall {
                    argument: arguments,
                    ..
                } => arguments,
                StructLiteral {
                    field_values: items,
                    ..
                }
                | Tuple(items) => {
                    if let Some(last) = items.last() {
                        last
                    } else {
                        break;
                    }
                }
                FunctionDef { body: last, .. }
                | Binary { right: last, .. }
                | Unary { child: last, .. }
                | Field { of: last, .. } => last,
                If { then, else_, .. } => {
                    if let Some(else_) = else_ {
                        else_
                    } else {
                        then
                    }
                }
                Match {
                    scrutinee,
                    branches,
                    ..
                } => {
                    if let Some(last) = branches.last() {
                        last
                    } else {
                        scrutinee
                    }
                }
                ImportedSymbol(..) | Immediate(..) | Identifier(..) => break,
            }
        }
        start..current
    }
}

impl std::ops::Index<usize> for IrModule {
    type Output = IrNode;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl std::ops::IndexMut<usize> for IrModule {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}
