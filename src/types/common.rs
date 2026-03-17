//! Shared helpers reused across type-system submodules.

use crate::Span;
use crate::ir::{
    Glob,
    Path,
    Pattern,
    PatternKind,
};

use super::{
    Kind,
    Type,
};

pub(crate) fn normalize_parameter_kinds(
    mut kinds: Vec<Kind>,
    parameter_count: usize,
) -> Vec<Kind> {
    if kinds.len() < parameter_count {
        kinds.extend(std::iter::repeat_n(
            Kind::Type,
            parameter_count - kinds.len(),
        ));
    }
    kinds.truncate(parameter_count);
    kinds
}

pub(crate) fn split_applied_type(type_: Type) -> (Type, Vec<Type>) {
    match type_ {
        Type::Apply {
            constructor,
            arguments,
        } => {
            let (base, mut constructor_arguments) = split_applied_type(*constructor);
            constructor_arguments.extend(arguments);
            (base, constructor_arguments)
        }
        other => (other, Vec::new()),
    }
}

pub(crate) fn split_applied_type_ref(type_: &Type) -> (Type, Vec<Type>) {
    match type_ {
        Type::Apply {
            constructor,
            arguments,
        } => {
            let (base, mut constructor_arguments) = split_applied_type_ref(constructor);
            constructor_arguments.extend(arguments.iter().cloned());
            (base, constructor_arguments)
        }
        other => (other.clone(), Vec::new()),
    }
}

pub(crate) fn for_each_pattern_binding<T>(
    pattern: &Pattern<T>,
    mut visit: impl FnMut(&Path, Span),
) {
    for_each_pattern_binding_impl(pattern, &mut visit);
}

fn for_each_pattern_binding_impl<T>(
    pattern: &Pattern<T>,
    visit: &mut impl FnMut(&Path, Span),
) {
    match &pattern.kind {
        PatternKind::Hole | PatternKind::Immediate(_) | PatternKind::ConstConstructor(_) => {}
        PatternKind::Identifier(path) => visit(path, pattern.span),
        PatternKind::Constructor(_, payload) => for_each_pattern_binding_impl(payload, visit),
        PatternKind::Tuple(items) => {
            for item in items.iter() {
                for_each_pattern_binding_impl(item, visit);
            }
        }
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            for item in starting.iter() {
                for_each_pattern_binding_impl(item, visit);
            }
            for item in ending.iter() {
                for_each_pattern_binding_impl(item, visit);
            }
            if let Glob::Named(path) = glob {
                visit(path, pattern.span);
            }
        }
        PatternKind::Struct(fields) => {
            for value in fields.values() {
                for_each_pattern_binding_impl(value, visit);
            }
        }
        PatternKind::TypeHint(inner, _) => for_each_pattern_binding_impl(inner, visit),
    }
}
