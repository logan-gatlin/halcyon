use super::*;

#[derive(Debug, Clone, sx::SXRepr)]
pub enum ConstructorKind {
    Unitary(Type),
    Function(Type, Type),
}

#[derive(Debug, Clone, sx::SXRepr)]
pub struct Constructor {
    pub variant_id: usize,
    pub kind: ConstructorKind,
}

impl Visit<Type> for Constructor {
    fn _visit(&mut self, f: &mut impl FnMut(&mut Type)) {
        match &mut self.kind {
            ConstructorKind::Unitary(t) => {
                t._visit(f);
            }
            ConstructorKind::Function(a, b) => {
                a._visit(f);
                b._visit(f);
            }
        }
    }
}
