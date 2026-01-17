use std::sync::atomic::AtomicU64;

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
    pub fn fresh_tv(&mut self) -> TypeVariable {
        let tv = self
            .current_type_variable
            .load(std::sync::atomic::Ordering::Relaxed);
        self.current_type_variable
            .store(tv + 1, std::sync::atomic::Ordering::Relaxed);
        tv
    }
    pub fn fresh_tv_source(&mut self) -> impl FnMut() -> TypeVariable {
        || self.fresh_tv()
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
            .unwrap_or_else(|| unreachable!("Accessed {path}"))
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
