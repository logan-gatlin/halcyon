use crate::asm::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AbiType {
    Number,
    BigInt,
    ExternRef,
}

impl AbiType {
    pub(crate) fn from_lowered(type_: &Type) -> Self {
        match type_ {
            Type::I8 | Type::I16 | Type::I32 | Type::F32 | Type::F64 => Self::Number,
            Type::I64 => Self::BigInt,
            Type::Any | Type::Struct(_) | Type::Array(_) | Type::Function { .. } => Self::ExternRef,
        }
    }

    pub(crate) const fn ts_name(self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::BigInt => "bigint",
            Self::ExternRef => "ExternRef",
        }
    }
}
