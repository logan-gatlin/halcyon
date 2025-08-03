mod build_ir;
pub mod constant;
mod namespace;
mod path;
pub mod printing;
pub mod types;

use std::collections::HashMap;

use crate::{lint::*, operator::*, semantic::ModuleInterface};

pub use build_ir::*;
pub use constant::*;
use namespace::*;
pub use path::*;
pub use types::*;

pub type IrPtr = usize;

#[derive(Debug, Clone)]
pub enum IrKind {
    Declaration {
        assignee: Path,
        value: IrPtr,
        in_: Option<IrPtr>,
    },
    RecursiveDeclaration {
        assignee: Path,
        parameter_name: Option<Path>,
        parameter_span: Span,
        parameter_type: Option<TypeRef>,
        captures: Vec<Path>,
        capture_types: Vec<TypeRef>,
        function_type: TypeRef,
        body: IrPtr,
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
    ImportedSymbol(Path, TypeRef),
}

#[derive(Debug, Clone)]
pub struct IrNode {
    pub kind: IrKind,
    pub span: Span,
    pub type_: TypeRef,
}

#[derive(Debug, Clone)]
pub enum ModuleItem {
    Let(Path, IrPtr),
    Type(Path, TypeRef),
}

#[derive(Debug, Clone)]
pub struct IrModule {
    pub module_name: Path,
    pub universe: HashMap<Path, TypeRef>,
    pub items: Vec<ModuleItem>,
    pub nodes: Vec<IrNode>,
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
                RecursiveDeclaration { body, in_, .. } => {
                    if let Some(in_) = in_ {
                        in_
                    } else {
                        body
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
