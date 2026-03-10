use crate::hc_core::CoreSymbol;
use crate::ir::Path;
use crate::types::{
    TraitRef,
    Type,
    TypeScheme,
};

pub type Precedence = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Associativity {
    Left,
    Right,
}

pub trait Operator {
    fn precedence(&self) -> Precedence;
    fn associative(&self) -> Associativity;
    fn path(&self) -> Path;
    fn type_scheme(&self) -> TypeScheme;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enum_iterator::Sequence)]
pub enum BinaryOp {
    Star,
    Slash,
    Percent,
    Plus,
    Minus,
    ComposeLeft,
    ComposeRight,
    Xor,
    Or,
    Apply,
    DoubleEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Semicolon,
}

impl Operator for BinaryOp {
    fn precedence(&self) -> Precedence {
        match self {
            Self::Star => 15,
            Self::Slash => 15,
            Self::Percent => 15,
            Self::Plus => 14,
            Self::Minus => 14,
            Self::ComposeLeft => 10,
            Self::ComposeRight => 10,
            Self::Xor => 10,
            Self::Or => 9,
            Self::Apply => 9,
            Self::DoubleEqual => 8,
            Self::BangEqual => 8,
            Self::Less => 8,
            Self::LessEqual => 8,
            Self::Greater => 8,
            Self::GreaterEqual => 8,
            Self::And => 7,
            Self::Semicolon => 1,
        }
    }
    fn associative(&self) -> Associativity {
        Associativity::Left
    }
    fn path(&self) -> Path {
        Path::core(format!("{self}"))
    }
    fn type_scheme(&self) -> TypeScheme {
        match self {
            Self::Star => binary_trait_scheme(CoreSymbol::TraitMultiply),
            Self::Slash => binary_trait_scheme(CoreSymbol::TraitDivide),
            Self::Percent => binary_trait_scheme(CoreSymbol::TraitRemainder),
            Self::Plus => binary_trait_scheme(CoreSymbol::TraitAdd),
            Self::Minus => binary_trait_scheme(CoreSymbol::TraitSubtract),
            Self::ComposeLeft => {
                Type::curry(&[
                    Type::func(Type::v(2), Type::v(1)),
                    Type::func(Type::v(1), Type::v(0)),
                    Type::func(Type::v(2), Type::v(0)),
                ])
                .for_all(3)
                .scheme()
            }
            Self::ComposeRight => {
                Type::curry(&[
                    Type::func(Type::v(1), Type::v(0)),
                    Type::func(Type::v(2), Type::v(1)),
                    Type::func(Type::v(2), Type::v(0)),
                ])
                .for_all(3)
                .scheme()
            }
            Self::Xor => binary_trait_scheme(CoreSymbol::TraitBitwise),
            Self::Or => binary_trait_scheme(CoreSymbol::TraitBitwise),
            Self::Apply => {
                Type::curry(&[Type::v(1), Type::func(Type::v(1), Type::v(0)), Type::v(0)])
                    .for_all(2)
                    .scheme()
            }
            Self::DoubleEqual => binary_trait_scheme_result(CoreSymbol::TraitEqual, Type::Boolean),
            Self::BangEqual => binary_trait_scheme_result(CoreSymbol::TraitEqual, Type::Boolean),
            Self::Less => binary_trait_scheme_result(CoreSymbol::TraitCompare, Type::Boolean),
            Self::LessEqual => {
                binary_trait_scheme_result_with(
                    &[CoreSymbol::TraitCompare, CoreSymbol::TraitEqual],
                    Type::Boolean,
                )
            }
            Self::Greater => binary_trait_scheme_result(CoreSymbol::TraitCompare, Type::Boolean),
            Self::GreaterEqual => {
                binary_trait_scheme_result_with(
                    &[CoreSymbol::TraitCompare, CoreSymbol::TraitEqual],
                    Type::Boolean,
                )
            }
            Self::And => binary_trait_scheme(CoreSymbol::TraitBitwise),
            Self::Semicolon => {
                Type::curry(&[Type::Unit, Type::v(0), Type::v(0)])
                    .for_all(1)
                    .scheme()
            }
        }
    }
}

impl std::fmt::Display for BinaryOp {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            match self {
                BinaryOp::Star => "*",
                BinaryOp::Slash => "/",
                BinaryOp::Percent => "%",
                BinaryOp::Plus => "+",
                BinaryOp::Minus => "-",
                BinaryOp::ComposeLeft => ">>",
                BinaryOp::ComposeRight => "<<",
                BinaryOp::Xor => "xor",
                BinaryOp::Or => "or",
                BinaryOp::Apply => "|>",
                BinaryOp::DoubleEqual => "==",
                BinaryOp::BangEqual => "!=",
                BinaryOp::Less => "<",
                BinaryOp::LessEqual => "<=",
                BinaryOp::Greater => ">",
                BinaryOp::GreaterEqual => ">=",
                BinaryOp::And => "and",
                BinaryOp::Semicolon => ";",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enum_iterator::Sequence)]
pub enum UnaryOp {
    Minus,
    Not,
}

impl Operator for UnaryOp {
    fn precedence(&self) -> Precedence {
        15
    }
    fn associative(&self) -> Associativity {
        Associativity::Left
    }
    fn path(&self) -> Path {
        Path::core(format!("{self}"))
    }
    fn type_scheme(&self) -> TypeScheme {
        match self {
            Self::Minus => unary_trait_scheme(CoreSymbol::TraitSubtract),
            Self::Not => unary_trait_scheme(CoreSymbol::TraitBitwise),
        }
    }
}

impl std::fmt::Display for UnaryOp {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            match self {
                UnaryOp::Minus => "~",
                UnaryOp::Not => "not",
            }
        )
    }
}

fn binary_trait_scheme(trait_symbol: CoreSymbol) -> TypeScheme {
    binary_trait_scheme_result(trait_symbol, Type::v(0))
}

fn binary_trait_scheme_result(
    trait_symbol: CoreSymbol,
    result: Type,
) -> TypeScheme {
    binary_trait_scheme_result_with(&[trait_symbol], result)
}

fn binary_trait_scheme_result_with(
    trait_symbols: &[CoreSymbol],
    result: Type,
) -> TypeScheme {
    Type::curry(&[Type::v(0), Type::v(0), result])
        .for_all(1)
        .scheme_with_predicates(
            trait_symbols
                .iter()
                .map(|symbol| TraitRef::new(symbol.path(), vec![Type::v(0)]))
                .collect(),
        )
}

fn unary_trait_scheme(trait_symbol: CoreSymbol) -> TypeScheme {
    Type::func(Type::v(0), Type::v(0))
        .for_all(1)
        .scheme_with_predicates(vec![TraitRef::new(trait_symbol.path(), vec![Type::v(0)])])
}
