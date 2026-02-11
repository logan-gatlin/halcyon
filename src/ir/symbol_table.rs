use std::sync::atomic::{
    AtomicU64,
    Ordering,
};

use super::*;

#[derive(Debug, Default)]
pub struct SymbolTable {
    pub spans: HashMap<(Path, NameSpace), Span>,
    pub terms: HashMap<Path, Type>,
    pub types: HashMap<Path, AbstractType>,
    pub constructors: HashMap<Path, Constructor>,
    pub current_type_variable: AtomicU64,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn contains_symbol(
        &self,
        path: &Path,
        namespace: NameSpace,
    ) -> bool {
        match namespace {
            NameSpace::Term => self.terms.contains_key(path),
            NameSpace::Type => self.types.contains_key(path),
            NameSpace::Constructor => self.constructors.contains_key(path),
        }
    }
    pub fn get_term(
        &self,
        path: &Path,
    ) -> &Type {
        self.terms
            .get(path)
            .unwrap_or_else(|| unreachable!("Accessed non-existant term {path}"))
    }
    pub fn get_type(
        &self,
        path: &Path,
    ) -> &AbstractType {
        self.types
            .get(path)
            .unwrap_or_else(|| unreachable!("Accessed {path}"))
    }
    pub fn get_constructor(
        &self,
        path: &Path,
    ) -> &Constructor {
        self.constructors
            .get(path)
            .unwrap_or_else(|| unreachable!("Accessed {path}"))
    }
}

pub trait TypeVariableSource {
    fn fresh_tv(&self) -> TypeVariable;
}

impl TypeVariableSource for SymbolTable {
    fn fresh_tv(&self) -> TypeVariable {
        self.current_type_variable.fresh_tv()
    }
}

impl TypeVariableSource for AtomicU64 {
    fn fresh_tv(&self) -> TypeVariable {
        self.fetch_add(1, Ordering::Relaxed)
    }
}

impl<T> TypeVariableSource for &T
where
    T: TypeVariableSource,
{
    fn fresh_tv(&self) -> TypeVariable {
        TypeVariableSource::fresh_tv(*self)
    }
}

impl<T> TypeVariableSource for &mut T
where
    T: TypeVariableSource,
{
    fn fresh_tv(&self) -> TypeVariable {
        TypeVariableSource::fresh_tv(&**self)
    }
}
