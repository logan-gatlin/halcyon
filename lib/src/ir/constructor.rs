use super::*;

#[derive(Debug, Clone, sx::SXRepr)]
pub struct Constructor {
    pub variant: usize,
    pub in_type: Type,
    pub out_type: Type,
}

impl Constructor {
    pub fn function_type(&self) -> Type {
        Type::func(self.in_type.clone(), self.out_type.clone())
    }
}
