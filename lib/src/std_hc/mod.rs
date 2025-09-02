mod builtin;

pub use builtin::*;
use std::{collections::HashMap, rc::Rc};

use crate::{
    WithSpan,
    compile::*,
    ir::{AsmLiteral, IrKind, IrNode, Path},
    operator::{BinaryOp, UnaryOp},
    semantic::*,
};

pub fn curry_function(
    parameters: impl IntoIterator<Item = (Path, Type)>,
    returns: Type,
    body: impl Fn(&mut FunctionEncoder) + 'static,
) -> IrNode {
    curry(
        parameters.into_iter(),
        &mut vec![],
        &mut vec![],
        returns,
        IrKind::AsmLiteral(AsmLiteral(Rc::new(body))),
    )
}

fn curry(
    mut parameters: impl Iterator<Item = (Path, Type)>,
    captures: &mut Vec<Path>,
    capture_types: &mut Vec<Type>,
    returns: Type,
    body: IrKind,
) -> IrNode {
    match parameters.next() {
        Some((path, parameter_type)) => {
            let old_captures = captures.clone();
            let old_capture_types = capture_types.clone();
            captures.push(path.clone());
            capture_types.push(parameter_type.clone());
            let body = Box::new(curry(parameters, captures, capture_types, returns, body));
            let return_type = body.type_.clone();
            IrKind::Function {
                parameter_name: Some(path.with_default_span()),
                parameter_type: None,
                captures: old_captures,
                capture_types: old_capture_types,
                body,
            }
            .with_default_span()
            .with_type(Type::func(parameter_type, return_type))
        }
        None => body.with_default_span().with_type(returns),
    }
}
