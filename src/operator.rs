use crate::ir::Path;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enum_iterator::Sequence)]
pub enum BinaryOp {
    Star,
    StarDot,
    Slash,
    SlashDot,
    Percent,
    Plus,
    PlusDot,
    Minus,
    MinusDot,
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
            Self::StarDot => 15,
            Self::Slash => 15,
            Self::SlashDot => 15,
            Self::Percent => 15,
            Self::Plus => 14,
            Self::PlusDot => 14,
            Self::Minus => 14,
            Self::MinusDot => 14,
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
}

impl std::fmt::Display for BinaryOp {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(
            f,
            "({})",
            match self {
                BinaryOp::Star => " * ",
                BinaryOp::StarDot => " *. ",
                BinaryOp::Slash => "/",
                BinaryOp::SlashDot => "/.",
                BinaryOp::Percent => "%",
                BinaryOp::Plus => "+",
                BinaryOp::PlusDot => "+.",
                BinaryOp::Minus => "-",
                BinaryOp::MinusDot => "-.",
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
    MinusDot,
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
}

impl std::fmt::Display for UnaryOp {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(
            f,
            "(unary {})",
            match self {
                UnaryOp::Minus => "-",
                UnaryOp::MinusDot => "-.",
                UnaryOp::Not => "not",
            }
        )
    }
}
