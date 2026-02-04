use std::collections::HashMap;

use wasm_encoder::{
    Encode,
    Section,
};

use crate::map::*;
use crate::semantic::AbstractType;

use super::*;

impl Encode for Path {
    fn encode(
        &self,
        sink: &mut Vec<u8>,
    ) {
        self.major.encode(sink);
        self.minor.encode(sink);
    }
}

/// Currently, WASM does not allow type imports. Even if it did, WASM types
/// are only a simplification of Halcyon types. This section provides semantic
/// information to linkers about a module's type signature
#[derive(Debug, Clone, Default)]
pub struct SignatureSection {
    /// Types referenced in this module, but defined elsewhere
    pub imported_types: Vec<Path>,
    /// An ordered map of type declarations from this module
    pub defined_types: IndexMap<Path, AbstractType>,
    /// The index of each referenced type in this section
    index_map: HashMap<Path, usize>,
}

impl SignatureSection {
    pub fn new(
        module_name: &str,
        symbols: &SymbolTable,
    ) -> Self {
        let mut imported_types = vec![];
        let mut defined_types = IndexMap::new();
        // PERFORMANCE:
        // This is O(n^2) because it iterates over every type, and is called for
        // every module in the symbol table. To fix, the symbol table will need
        // to be changed to a doubly nested indexmap.
        for (path, t) in &symbols.types {
            if path.major == module_name {
                defined_types.insert(path.clone(), t.clone());
                t.clone().visit(|t: &mut semantic::Type| {
                    if let semantic::Type::Instantiation(path, _) = t {
                        imported_types.push(path.clone());
                    }
                });
            }
        }
        let mut index_map = HashMap::new();
        for (id, path) in imported_types.iter().enumerate() {
            index_map.insert(path.clone(), id);
        }
        for (id, path) in defined_types.keys().enumerate() {
            index_map.insert(path.clone(), id);
        }
        Self {
            imported_types,
            defined_types,
            index_map,
        }
    }

    /// Type layout:
    /// * Unit: '0'
    /// * Integer: '1'
    /// * Real: '2'
    /// * Boolean: '3'
    /// * String: '4'
    /// * Glyph: '5'
    /// * Variable: '6' uint
    /// * Struct: '7' [(str, Type)]
    /// * Array: '8' Type
    /// * Tuple: '9' [Type]
    /// * Sum: '10' [(str, Type)]
    /// * Function: '11' (Type, Type)
    /// * Instantiation: '12' uint [Type]
    fn encode_type(
        &self,
        type_: &semantic::Type,
        sink: &mut Vec<u8>,
    ) {
        use semantic::Type::*;
        match type_ {
            Any => unreachable!(),
            Unit => 0usize.encode(sink),
            Integer => 1usize.encode(sink),
            Real => 2usize.encode(sink),
            Boolean => 3usize.encode(sink),
            String => 4usize.encode(sink),
            Glyph => 5usize.encode(sink),
            Variable(id) => {
                6usize.encode(sink);
                id.encode(sink);
            }
            Struct { fields, .. } => {
                7usize.encode(sink);
                fields.len().encode(sink);
                for f in fields {
                    f.0.encode(sink);
                    self.encode_type(f.1, sink);
                }
            }
            Array(t) => {
                8usize.encode(sink);
                self.encode_type(t, sink);
            }
            Tuple(items) => {
                9usize.encode(sink);
                items.len().encode(sink);
                for i in items {
                    self.encode_type(i, sink);
                }
            }
            Sum {
                variant_names,
                variant_types,
                ..
            } => {
                10usize.encode(sink);
                for (name, type_) in variant_names.iter().zip(variant_types) {
                    name.encode(sink);
                    self.encode_type(type_, sink);
                }
            }
            Function(param, result) => {
                11usize.encode(sink);
                self.encode_type(param, sink);
                self.encode_type(result, sink);
            }
            Instantiation(path, items) => {
                12usize.encode(sink);
                path.encode(sink);
                items.len().encode(sink);
                for item in items {
                    self.encode_type(item, sink);
                }
            }
        }
    }
}

impl wasm_encoder::Section for SignatureSection {
    fn id(&self) -> u8 {
        0 // Custom section
    }
}

/// Section layout:
/// [Path] (imported type paths)
/// [(Path, Type, [uint])] (Type definitions)
impl Encode for SignatureSection {
    fn encode(
        &self,
        sink: &mut Vec<u8>,
    ) {
        self.imported_types.encode(sink);
        self.defined_types.len().encode(sink);
        for (path, t) in &self.defined_types {
            path.encode(sink);
            self.encode_type(&t.base, sink);
            t.variables.encode(sink);
        }
    }
}
