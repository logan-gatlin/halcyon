use indexmap::IndexMap;
use itertools::Itertools;

use crate::ir::Path;
use crate::operator::{
    BinaryOp,
    Operator,
    UnaryOp,
};
use crate::types::{
    TraitImpl,
    TraitRef,
    Type,
};

use super::core_impl_method_path;

fn trait_path(name: &str) -> Path {
    Path::core(name)
}

fn trait_method_paths(name: &str) -> Vec<Path> {
    use {
        BinaryOp as b,
        UnaryOp as u,
    };
    match name {
        "equal" => vec![b::DoubleEqual.path()],
        "compare" => vec![b::Less.path(), b::Greater.path()],
        "add" => vec![b::Plus.path()],
        "subtract" => vec![b::Minus.path(), u::Minus.path()],
        "multiply" => vec![b::Star.path()],
        "divide" => vec![b::Slash.path()],
        "remainder" => vec![b::Percent.path()],
        "bitwise" => vec![b::And.path(), b::Or.path(), b::Xor.path(), u::Not.path()],
        _ => Vec::new(),
    }
}

fn implement(
    trait_names: &[&str],
    types: &[Type],
) -> Vec<TraitImpl> {
    trait_names
        .iter()
        .cartesian_product(types.iter())
        .map(|(trait_name, type_)| {
            let methods = trait_method_paths(trait_name)
                .into_iter()
                .map(|method_path| {
                    let impl_path = core_impl_method_path(&method_path, type_);
                    (method_path, impl_path)
                })
                .collect::<IndexMap<_, _>>();
            TraitImpl {
                parameters: 1,
                head: TraitRef {
                    trait_name: trait_path(trait_name),
                    arguments: vec![type_.clone()],
                },
                predicates: vec![],
                methods,
            }
        })
        .collect()
}

pub fn core_impls() -> Vec<TraitImpl> {
    implement(&["add"], &[Type::array()])
}
