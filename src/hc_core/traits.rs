use indexmap::IndexMap;
use itertools::Itertools;

use crate::types::{
    TraitImpl,
    TraitRef,
};

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enum_iterator::Sequence)]
pub enum CoreTrait {
    /// ==, !=
    Equal,
    /// <, >
    Compare,
    /// +
    Add,
    /// -
    Subtract,
    /// *
    Multiply,
    /// /
    Divide,
    /// %
    Remainder,
    /// and, or, xor, not
    Bitwise,
}

impl CoreTrait {
    pub fn parameters(&self) -> usize {
        1
    }
}

impl Symbol for CoreTrait {
    fn path(&self) -> Path {
        Path::core(match self {
            CoreTrait::Equal => "equal",
            CoreTrait::Compare => "compare",
            CoreTrait::Add => "add",
            CoreTrait::Subtract => "subtract",
            CoreTrait::Multiply => "multiply",
            CoreTrait::Divide => "divide",
            CoreTrait::Remainder => "remainder",
            CoreTrait::Bitwise => "bitwise",
        })
    }

    fn symbol_kind(&self) -> SymbolKind {
        let binary = Type::curry(&[Type::v(0), Type::v(0), Type::v(0)]).scheme();
        let binary_bool = Type::curry(&[Type::v(0), Type::v(0), Type::Boolean]).scheme();
        let unary = Type::func(Type::v(0), Type::v(0)).scheme();
        let methods = |items: &[(Path, TypeScheme)]| items.iter().cloned().collect();
        use {
            BinaryOp as b,
            UnaryOp as u,
        };
        SymbolKind::TraitDef(TraitDef {
            name: self.path(),
            parameters: self.parameters(),
            methods: match self {
                CoreTrait::Equal => {
                    methods(&[
                        (b::DoubleEqual.path(), binary_bool.clone()),
                        (b::BangEqual.path(), binary_bool.clone()),
                    ])
                }
                CoreTrait::Compare => {
                    methods(&[
                        (b::Less.path(), binary_bool.clone()),
                        (b::Greater.path(), binary_bool.clone()),
                    ])
                }
                CoreTrait::Add => methods(&[(b::Plus.path(), binary.clone())]),
                CoreTrait::Subtract => {
                    methods(&[
                        (b::Minus.path(), binary.clone()),
                        (u::Minus.path(), unary.clone()),
                    ])
                }
                CoreTrait::Multiply => methods(&[(b::Star.path(), binary.clone())]),
                CoreTrait::Divide => methods(&[(b::Slash.path(), binary.clone())]),
                CoreTrait::Remainder => methods(&[(b::Percent.path(), binary.clone())]),
                CoreTrait::Bitwise => {
                    methods(&[
                        (b::And.path(), binary.clone()),
                        (b::Or.path(), binary.clone()),
                        (b::Xor.path(), binary.clone()),
                        (u::Not.path(), unary.clone()),
                    ])
                }
            },
        })
    }
}

fn implement<Tr: IntoIterator<Item = CoreTrait>, Ty: IntoIterator<Item = Type>>(
    traits: Tr,
    types: Ty,
) -> Vec<TraitImpl>
where
    <Ty as std::iter::IntoIterator>::IntoIter: std::clone::Clone,
{
    traits
        .into_iter()
        .cartesian_product(types)
        .map(|(tr, ty)| {
            let methods = trait_method_paths(tr)
                .into_iter()
                .map(|method_path| {
                    let impl_path = core_impl_method_path(&method_path, &ty);
                    (method_path, impl_path)
                })
                .collect::<IndexMap<_, _>>();
            TraitImpl {
                parameters: tr.parameters(),
                head: TraitRef {
                    trait_name: tr.path(),
                    arguments: vec![ty],
                },
                predicates: vec![],
                methods,
            }
        })
        .collect()
}

fn trait_method_paths(tr: CoreTrait) -> Vec<Path> {
    use {
        BinaryOp as b,
        UnaryOp as u,
    };
    match tr {
        CoreTrait::Equal => vec![b::DoubleEqual.path(), b::BangEqual.path()],
        CoreTrait::Compare => vec![b::Less.path(), b::Greater.path()],
        CoreTrait::Add => vec![b::Plus.path()],
        CoreTrait::Subtract => vec![b::Minus.path(), u::Minus.path()],
        CoreTrait::Multiply => vec![b::Star.path()],
        CoreTrait::Divide => vec![b::Slash.path()],
        CoreTrait::Remainder => vec![b::Percent.path()],
        CoreTrait::Bitwise => vec![b::And.path(), b::Or.path(), b::Xor.path(), u::Not.path()],
    }
}

pub fn core_impls() -> Vec<TraitImpl> {
    use CoreTrait::*;
    use Type::*;
    let mut impls = vec![];
    impls.extend(implement(
        [Equal, Compare],
        [Unit, Integer, Real, Boolean, String, Glyph],
    ));
    impls.extend(implement(
        [Add, Subtract, Multiply, Divide],
        [Integer, Real],
    ));
    impls.extend(implement([Add], [String, Type::array()]));
    impls.extend(implement([Remainder], [Integer]));
    impls.extend(implement([Bitwise], [Integer, Boolean]));
    impls
}
