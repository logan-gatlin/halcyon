use super::*;
use crate::ir::{IrKind, ModuleItem};

/// Tail call (aka return call) is an optimization that allows recursive function
/// calls without growing the stack. It can be applied to function calls that may
/// be immediately followed by a function return without altering the normal
/// control flow.

pub fn tail_call(module: &mut IrModule) {
    module.items.iter_mut().for_each(|i| match i {
        ModuleItem::Let(_, node) => look_for_tail_call(node),
        _ => {}
    })
}

use IrKind::*;
/// Look for function nodes where tail call would be applicable
fn look_for_tail_call(node: &mut IrNode) {
    match &mut node.inner.inner {
        Let { value, in_, .. } => {
            look_for_tail_call(value);
            look_for_tail_call(in_);
        }
        Match {
            branches: items, ..
        }
        | Struct {
            field_values: items,
            ..
        }
        | Tuple(items) => items.iter_mut().for_each(look_for_tail_call),
        Field { of, .. } => look_for_tail_call(of),
        Function { body, .. } => tail_call_node(body),
        Semicolon(a, b) => {
            look_for_tail_call(a);
            look_for_tail_call(b);
        }
        Call {
            callee, argument, ..
        } => {
            look_for_tail_call(callee);
            look_for_tail_call(argument);
        }
        If {
            predicate,
            then,
            else_,
        } => {
            look_for_tail_call(predicate);
            look_for_tail_call(then);
            look_for_tail_call(else_);
        }
        _ => {}
    }
}

/// Once inside a function, mark calls that may be optimized
fn tail_call_node(node: &mut IrNode) {
    match &mut node.inner.inner {
        Let { in_, .. } => {
            tail_call_node(in_);
        }
        Function { body, .. } => tail_call_node(body),
        Call { opt, .. } => *opt = CallOptimization::Tail,
        If { then, else_, .. } => {
            tail_call_node(then);
            tail_call_node(else_);
        }
        Semicolon(_, b) => tail_call_node(b),
        Match { branches, .. } => branches.iter_mut().for_each(tail_call_node),
        _ => {}
    }
}
