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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enum_iterator::Sequence)]
pub enum CoreImpl {
    BinaryOp(BinaryOp),
    UnaryOp(UnaryOp),
}
