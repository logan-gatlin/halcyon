use std::collections::HashMap;

use crate::ir::Path;

use super::Type;

#[derive(Debug, Clone, Default)]
pub struct TypeCatalog {
    definitions: HashMap<Path, TypeDefinition>,
}

impl TypeCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        path: Path,
        definition: TypeDefinition,
    ) -> Option<TypeDefinition> {
        self.definitions.insert(path, definition)
    }

    pub fn get(
        &self,
        path: &Path,
    ) -> Option<&TypeDefinition> {
        self.definitions.get(path)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Path, &TypeDefinition)> {
        self.definitions.iter()
    }
}

#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub parameters: usize,
    pub body: Type,
}
