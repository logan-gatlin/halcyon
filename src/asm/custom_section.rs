use std::collections::HashMap;

use wasm_encoder::{
    CustomSection,
    Encode,
};

use indexmap::{
    IndexMap,
    IndexSet,
};

use super::*;
use crate::types::{
    Kind,
    StructMatch,
    TraitConstraint,
    TraitRef,
    Type as SemanticType,
    TypeDefinition,
    TypeDefinitionKind,
    TypeScheme,
    TypeTransform,
};

/// A cursor for decoding LEB128-encoded data
struct Decoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn decode_usize(&mut self) -> Option<usize> {
        let mut result = 0usize;
        let mut shift = 0;
        loop {
            let byte = *self.data.get(self.pos)?;
            self.pos += 1;
            result |= ((byte & 0x7F) as usize) << shift;
            if byte & 0x80 == 0 {
                return Some(result);
            }
            shift += 7;
            if shift >= usize::BITS {
                return None;
            }
        }
    }

    #[cfg(test)]
    fn decode_u64(&mut self) -> Option<u64> {
        let mut result = 0u64;
        let mut shift = 0;
        loop {
            let byte = *self.data.get(self.pos)?;
            self.pos += 1;
            result |= ((byte & 0x7F) as u64) << shift;
            if byte & 0x80 == 0 {
                return Some(result);
            }
            shift += 7;
            if shift >= u64::BITS {
                return None;
            }
        }
    }

    fn decode_u32(&mut self) -> Option<u32> {
        let mut result = 0u32;
        let mut shift = 0;
        loop {
            let byte = *self.data.get(self.pos)?;
            self.pos += 1;
            result |= ((byte & 0x7F) as u32) << shift;
            if byte & 0x80 == 0 {
                return Some(result);
            }
            shift += 7;
            if shift >= u32::BITS {
                return None;
            }
        }
    }

    fn decode_string(&mut self) -> Option<String> {
        let len = self.decode_usize()?;
        let end = self.pos.checked_add(len)?;
        let bytes = self.data.get(self.pos..end)?;
        let s = std::str::from_utf8(bytes).ok()?.to_string();
        self.pos = end;
        Some(s)
    }

    fn decode_path(&mut self) -> Option<Path> {
        Some(Path {
            major: self.decode_string()?,
            minor: self.decode_string()?,
        })
    }
}

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
pub struct TypeSignatureSection {
    /// Types referenced in this module, but defined elsewhere
    pub imported_types: Vec<Path>,
    /// An ordered map of type declarations from this module
    pub defined_types: IndexMap<Path, TypeDefinition>,
    pub defined_terms: IndexMap<Path, TypeScheme>,
    /// The index of each referenced type in this section
    index_map: HashMap<Path, usize>,
}

impl TypeSignatureSection {
    pub const NAME: &str = "type_signature";
    pub fn new(
        module_name: &str,
        symbols: &SymbolTable,
    ) -> Self {
        let mut imported_types = IndexSet::new();
        let mut defined_types = IndexMap::new();
        let mut defined_terms = IndexMap::new();

        for (path, definition) in symbols.type_definitions().iter() {
            if path.major == module_name {
                defined_types.insert(path.clone(), definition.clone());
            }
        }
        for (path, scheme) in symbols.terms().iter() {
            if path.major == module_name {
                defined_terms.insert(path.clone(), scheme.clone());
            }
        }

        for definition in defined_types.values() {
            collect_imported_types(&definition.body, &defined_types, &mut imported_types);
        }
        for scheme in defined_terms.values() {
            collect_imported_types(&scheme.type_, &defined_types, &mut imported_types);
            for predicate in scheme.predicates.iter() {
                for arg in predicate.arguments.iter() {
                    collect_imported_types(arg, &defined_types, &mut imported_types);
                }
            }
        }
        let imported_types = imported_types.into_iter().collect::<Vec<_>>();
        let mut index_map = HashMap::new();
        for (id, path) in imported_types
            .iter()
            .chain(defined_types.keys())
            .enumerate()
        {
            index_map.insert(path.clone(), id);
        }
        Self {
            imported_types,
            defined_types,
            defined_terms,
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
    /// * TypeVar: '6' u32
    /// * MetaVar: '7' u32
    /// * ForAll: '9' Type
    /// * Named: '11' uint Type
    /// * StructConstraint: '12' (match_mode, [(str, Type)])
    /// * Struct: '13' [(str, Type)]
    /// * Array: '14' Type
    /// * Tuple: '15' [Type]
    /// * Sum: '16' [(str, Type)]
    /// * Function: '17' (Type, Type)
    /// * Apply: '18' Type [Type]
    fn encode_type(
        &self,
        type_: &SemanticType,
        sink: &mut Vec<u8>,
    ) {
        use SemanticType::*;
        match type_ {
            Unit => 0usize.encode(sink),
            Integer => 1usize.encode(sink),
            Real => 2usize.encode(sink),
            Boolean => 3usize.encode(sink),
            String => 4usize.encode(sink),
            Glyph => 5usize.encode(sink),
            TypeVar(id) => {
                6usize.encode(sink);
                id.encode(sink);
            }
            MetaVar(id) => {
                7usize.encode(sink);
                id.encode(sink);
            }
            ForAll(body) => {
                9usize.encode(sink);
                self.encode_type(body, sink);
            }
            Named { name, body } => {
                11usize.encode(sink);
                self.index_map[name].encode(sink);
                self.encode_type(body, sink);
            }
            StructConstraint { fields, mode } => {
                12usize.encode(sink);
                match mode {
                    StructMatch::Exact => 0usize.encode(sink),
                    StructMatch::AtLeast => 1usize.encode(sink),
                }
                fields.len().encode(sink);
                for (name, type_) in fields.iter() {
                    name.encode(sink);
                    self.encode_type(type_, sink);
                }
            }
            Struct { fields } => {
                13usize.encode(sink);
                fields.len().encode(sink);
                for (name, type_) in fields.iter() {
                    name.encode(sink);
                    self.encode_type(type_, sink);
                }
            }
            Array(t) => {
                14usize.encode(sink);
                self.encode_type(t, sink);
            }
            Tuple(items) => {
                15usize.encode(sink);
                items.len().encode(sink);
                for i in items {
                    self.encode_type(i, sink);
                }
            }
            Sum { variants } => {
                16usize.encode(sink);
                variants.len().encode(sink);
                for (name, type_) in variants.iter() {
                    name.encode(sink);
                    self.encode_type(type_, sink);
                }
            }
            Function(param, result) => {
                17usize.encode(sink);
                self.encode_type(param, sink);
                self.encode_type(result, sink);
            }
            Apply {
                constructor,
                arguments,
            } => {
                18usize.encode(sink);
                self.encode_type(constructor, sink);
                arguments.len().encode(sink);
                for arg in arguments {
                    self.encode_type(arg, sink);
                }
            }
        }
    }

    fn encode_scheme(
        &self,
        scheme: &TypeScheme,
        sink: &mut Vec<u8>,
    ) {
        self.encode_type(&scheme.type_, sink);
        scheme.predicates.len().encode(sink);
        for predicate in scheme.predicates.iter() {
            self.encode_predicate(predicate, sink);
        }
    }

    fn encode_predicate(
        &self,
        predicate: &TraitConstraint,
        sink: &mut Vec<u8>,
    ) {
        predicate.trait_name.encode(sink);
        predicate.arguments.len().encode(sink);
        for arg in predicate.arguments.iter() {
            self.encode_type(arg, sink);
        }
    }

    fn encode_kind(
        kind: &Kind,
        sink: &mut Vec<u8>,
    ) {
        match kind {
            Kind::Type => 0usize.encode(sink),
            Kind::Arrow(parameter, result) => {
                1usize.encode(sink);
                Self::encode_kind(parameter, sink);
                Self::encode_kind(result, sink);
            }
        }
    }

    #[cfg(test)]
    fn rebuild_index_map(&mut self) {
        self.index_map.clear();
        for (id, path) in self
            .imported_types
            .iter()
            .chain(self.defined_types.keys())
            .enumerate()
        {
            self.index_map.insert(path.clone(), id);
        }
    }

    /// Decode a SignatureSection from raw bytes (without section id or length prefix)
    /// Decode a complete section including the name prefix.
    /// Used when reading raw encoded bytes (e.g. in roundtrip tests).
    pub fn decode(data: &[u8]) -> Option<Self> {
        let mut dec = Decoder::new(data);
        let name = dec.decode_string()?;
        if name != Self::NAME {
            return None;
        }
        Self::decode_data(&mut dec)
    }

    /// Decode section data without the name prefix.
    /// Used when reading from `wasmparser`, which strips the name.
    pub fn decode_data_slice(data: &[u8]) -> Option<Self> {
        Self::decode_data(&mut Decoder::new(data))
    }

    fn decode_data(dec: &mut Decoder) -> Option<Self> {
        let import_count = dec.decode_usize()?;
        let mut imported_types = Vec::with_capacity(import_count);
        for _ in 0..import_count {
            imported_types.push(dec.decode_path()?);
        }

        let mut reverse_index: Vec<Path> = imported_types.clone();

        let defined_count = dec.decode_usize()?;
        let mut defined_types = IndexMap::new();
        for _ in 0..defined_count {
            let path = dec.decode_path()?;
            reverse_index.push(path.clone());
            let kind = match dec.decode_usize()? {
                0 => TypeDefinitionKind::Named,
                1 => TypeDefinitionKind::Alias,
                _ => return None,
            };
            let parameters = dec.decode_usize()?;
            let mut parameter_kinds = Vec::with_capacity(parameters);
            for _ in 0..parameters {
                parameter_kinds.push(Self::decode_kind(dec)?);
            }
            let body = Self::decode_type(dec, &reverse_index)?;
            defined_types.insert(
                path,
                TypeDefinition {
                    parameters,
                    parameter_kinds,
                    body,
                    kind,
                },
            );
        }

        let defined_count = dec.decode_usize()?;
        let mut defined_terms = IndexMap::new();
        for _ in 0..defined_count {
            let path = dec.decode_path()?;
            let scheme = Self::decode_scheme(dec, &reverse_index)?;
            defined_terms.insert(path, scheme);
        }

        let mut index_map = HashMap::new();
        for (id, path) in imported_types.iter().enumerate() {
            index_map.insert(path.clone(), id);
        }
        for (id, path) in defined_types.keys().enumerate() {
            index_map.insert(path.clone(), id + imported_types.len());
        }

        Some(Self {
            imported_types,
            defined_types,
            defined_terms,
            index_map,
        })
    }

    fn decode_type(
        dec: &mut Decoder,
        reverse_index: &[Path],
    ) -> Option<SemanticType> {
        use SemanticType::*;
        Some(match dec.decode_usize()? {
            0 => Unit,
            1 => Integer,
            2 => Real,
            3 => Boolean,
            4 => String,
            5 => Glyph,
            6 => TypeVar(dec.decode_u32()?),
            7 => MetaVar(dec.decode_u32()?),
            9 => ForAll(Box::new(Self::decode_type(dec, reverse_index)?)),
            11 => {
                let idx = dec.decode_usize()?;
                let name = reverse_index.get(idx)?.clone();
                let body = Self::decode_type(dec, reverse_index)?;
                Named {
                    name,
                    body: Box::new(body),
                }
            }
            12 => {
                let mode = match dec.decode_usize()? {
                    0 => StructMatch::Exact,
                    1 => StructMatch::AtLeast,
                    _ => return None,
                };
                let field_count = dec.decode_usize()?;
                let mut fields = IndexMap::new();
                for _ in 0..field_count {
                    let name = dec.decode_string()?;
                    let type_ = Self::decode_type(dec, reverse_index)?;
                    fields.insert(name, type_);
                }
                StructConstraint { fields, mode }
            }
            13 => {
                let field_count = dec.decode_usize()?;
                let mut fields = IndexMap::new();
                for _ in 0..field_count {
                    let name = dec.decode_string()?;
                    let type_ = Self::decode_type(dec, reverse_index)?;
                    fields.insert(name, type_);
                }
                Struct { fields }
            }
            14 => Array(Box::new(Self::decode_type(dec, reverse_index)?)),
            15 => {
                let item_count = dec.decode_usize()?;
                let mut items = Vec::with_capacity(item_count);
                for _ in 0..item_count {
                    items.push(Self::decode_type(dec, reverse_index)?);
                }
                Tuple(items)
            }
            16 => {
                let variant_count = dec.decode_usize()?;
                let mut variants = IndexMap::new();
                for _ in 0..variant_count {
                    let name = dec.decode_string()?;
                    let type_ = Self::decode_type(dec, reverse_index)?;
                    variants.insert(name, type_);
                }
                Sum { variants }
            }
            17 => {
                let param = Self::decode_type(dec, reverse_index)?;
                let result = Self::decode_type(dec, reverse_index)?;
                Function(Box::new(param), Box::new(result))
            }
            18 => {
                let constructor = Self::decode_type(dec, reverse_index)?;
                let arg_count = dec.decode_usize()?;
                let mut arguments = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    arguments.push(Self::decode_type(dec, reverse_index)?);
                }
                Apply {
                    constructor: Box::new(constructor),
                    arguments,
                }
            }
            _ => return None,
        })
    }

    fn decode_kind(dec: &mut Decoder) -> Option<Kind> {
        Some(match dec.decode_usize()? {
            0 => Kind::Type,
            1 => Kind::arrow(Self::decode_kind(dec)?, Self::decode_kind(dec)?),
            _ => return None,
        })
    }

    fn decode_scheme(
        dec: &mut Decoder,
        reverse_index: &[Path],
    ) -> Option<TypeScheme> {
        let type_ = Self::decode_type(dec, reverse_index)?;
        let predicate_count = dec.decode_usize()?;
        let mut predicates = Vec::with_capacity(predicate_count);
        for _ in 0..predicate_count {
            let trait_name = dec.decode_path()?;
            let arg_count = dec.decode_usize()?;
            let mut arguments = Vec::with_capacity(arg_count);
            for _ in 0..arg_count {
                arguments.push(Self::decode_type(dec, reverse_index)?);
            }
            predicates.push(TraitRef {
                trait_name,
                arguments,
            });
        }
        Some(TypeScheme { predicates, type_ })
    }
}

impl wasm_encoder::Section for TypeSignatureSection {
    fn id(&self) -> u8 {
        0 // Custom section
    }
}

/// Section layout:
/// [Path] (imported type paths)
/// [(Path, parameters, Type)] (type definitions)
/// [(Path, TypeScheme)] (term definitions)
impl Encode for TypeSignatureSection {
    fn encode(
        &self,
        sink: &mut Vec<u8>,
    ) {
        let mut data = vec![];
        self.imported_types.encode(&mut data);
        self.defined_types.len().encode(&mut data);
        for (path, t) in &self.defined_types {
            path.encode(&mut data);
            match t.kind {
                TypeDefinitionKind::Named => 0usize.encode(&mut data),
                TypeDefinitionKind::Alias => 1usize.encode(&mut data),
            }
            t.parameters.encode(&mut data);
            let parameter_kinds = if t.parameter_kinds.len() == t.parameters {
                t.parameter_kinds.clone()
            } else {
                vec![Kind::Type; t.parameters]
            };
            for kind in parameter_kinds.iter() {
                Self::encode_kind(kind, &mut data);
            }
            self.encode_type(&t.body, &mut data);
        }
        self.defined_terms.len().encode(&mut data);
        for (path, t) in &self.defined_terms {
            path.encode(&mut data);
            self.encode_scheme(t, &mut data);
        }
        CustomSection {
            name: Self::NAME.into(),
            data: data.into(),
        }
        .encode(sink);
    }
}

fn collect_imported_types(
    type_: &SemanticType,
    defined_types: &IndexMap<Path, TypeDefinition>,
    imported_types: &mut IndexSet<Path>,
) {
    struct ImportedTypeCollector<'a> {
        defined_types: &'a IndexMap<Path, TypeDefinition>,
        imported_types: &'a mut IndexSet<Path>,
    }

    impl TypeTransform for ImportedTypeCollector<'_> {
        fn visit(
            &mut self,
            type_: &SemanticType,
        ) {
            if let SemanticType::Named { name, .. } = type_
                && !self.defined_types.contains_key(name)
            {
                self.imported_types.insert(name.clone());
            }
        }
    }

    ImportedTypeCollector {
        defined_types,
        imported_types,
    }
    .walk(type_);
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::hc_core::CoreSymbol;

    fn decode_section(encoded: &[u8]) -> TypeSignatureSection {
        let mut pos = 0;
        while encoded[pos] & 0x80 != 0 {
            pos += 1;
        }
        pos += 1;
        TypeSignatureSection::decode(&encoded[pos..]).unwrap()
    }

    fn roundtrip_type(type_: SemanticType) {
        let path = Path::new("test", "MyType");
        let mut section = TypeSignatureSection::default();
        section.defined_types.insert(
            path.clone(),
            TypeDefinition {
                parameters: 0,
                parameter_kinds: Vec::new(),
                body: type_.clone(),
                kind: TypeDefinitionKind::Named,
            },
        );
        section.rebuild_index_map();

        let mut encoded = vec![];
        section.encode(&mut encoded);
        let decoded = decode_section(&encoded);

        let original_type = &section.defined_types.values().next().unwrap().body;
        let decoded_type = &decoded.defined_types.values().next().unwrap().body;
        assert_eq!(format!("{original_type:?}"), format!("{decoded_type:?}"));
    }

    #[test]
    fn roundtrip_primitives() {
        roundtrip_type(SemanticType::Unit);
        roundtrip_type(SemanticType::Integer);
        roundtrip_type(SemanticType::Real);
        roundtrip_type(SemanticType::Boolean);
        roundtrip_type(SemanticType::String);
        roundtrip_type(SemanticType::Glyph);
    }

    #[test]
    fn roundtrip_vars_and_binders() {
        roundtrip_type(SemanticType::TypeVar(2));
        roundtrip_type(SemanticType::MetaVar(3));
        roundtrip_type(SemanticType::ForAll(Box::new(SemanticType::TypeVar(0))));
    }

    #[test]
    fn roundtrip_struct_and_constraint() {
        let mut fields = IndexMap::new();
        fields.insert("x".into(), SemanticType::Integer);
        fields.insert("y".into(), SemanticType::Real);
        roundtrip_type(SemanticType::Struct {
            fields: fields.clone(),
        });
        roundtrip_type(SemanticType::StructConstraint {
            fields,
            mode: StructMatch::AtLeast,
        });
    }

    #[test]
    fn roundtrip_compound() {
        roundtrip_type(SemanticType::Array(Box::new(SemanticType::Integer)));
        roundtrip_type(SemanticType::Tuple(vec![
            SemanticType::Integer,
            SemanticType::String,
            SemanticType::Boolean,
        ]));
        roundtrip_type(SemanticType::Sum {
            variants: {
                let mut variants = IndexMap::new();
                variants.insert("None".into(), SemanticType::Unit);
                variants.insert("Some".into(), SemanticType::Integer);
                variants
            },
        });
        roundtrip_type(SemanticType::Function(
            Box::new(SemanticType::Integer),
            Box::new(SemanticType::String),
        ));
    }

    #[test]
    fn roundtrip_named_apply_and_scheme() {
        let path = Path::new("test", "Box");
        let body = SemanticType::ForAll(Box::new(SemanticType::Struct {
            fields: {
                let mut fields = IndexMap::new();
                fields.insert("value".into(), SemanticType::TypeVar(0));
                fields
            },
        }));
        let applied = SemanticType::Apply {
            constructor: Box::new(SemanticType::Named {
                name: path.clone(),
                body: Box::new(body.clone()),
            }),
            arguments: vec![SemanticType::Integer],
        };

        let mut section = TypeSignatureSection::default();
        section.defined_types.insert(
            path.clone(),
            TypeDefinition {
                parameters: 1,
                parameter_kinds: vec![crate::types::Kind::Type],
                body: body.clone(),
                kind: TypeDefinitionKind::Named,
            },
        );
        section.defined_terms.insert(
            Path::new("test", "make"),
            TypeScheme::with_predicates(
                applied,
                vec![TraitRef {
                    trait_name: CoreSymbol::TraitEqual.path(),
                    arguments: vec![SemanticType::Integer],
                }],
            ),
        );
        section.rebuild_index_map();

        let mut encoded = vec![];
        section.encode(&mut encoded);
        let decoded = decode_section(&encoded);

        let orig_term = section.defined_terms.values().next().unwrap();
        let dec_term = decoded.defined_terms.values().next().unwrap();
        assert_eq!(format!("{orig_term:?}"), format!("{dec_term:?}"));
    }

    #[test]
    fn decode_empty_returns_none() {
        assert!(TypeSignatureSection::decode(&[]).is_none());
    }

    #[test]
    fn decode_wrong_name_returns_none() {
        let mut data = vec![];
        "not_signature".encode(&mut data);
        assert!(TypeSignatureSection::decode(&data).is_none());
    }

    #[test]
    fn decode_truncated_after_name_returns_none() {
        let mut data = vec![];
        TypeSignatureSection::NAME.encode(&mut data);
        assert!(TypeSignatureSection::decode(&data).is_none());
    }

    #[test]
    fn decode_truncated_leb128_returns_none() {
        assert!(TypeSignatureSection::decode(&[0x80]).is_none());
    }

    #[test]
    fn decode_oversized_leb128_returns_none() {
        let data: Vec<u8> = std::iter::repeat_n(0x80, 10).chain([0x01]).collect();
        let mut dec = Decoder::new(&data);
        assert!(dec.decode_u64().is_none());
    }

    #[test]
    fn decode_invalid_utf8_returns_none() {
        let data = [0x02, 0xFF, 0xFE];
        let mut dec = Decoder::new(&data);
        assert!(dec.decode_string().is_none());
    }

    #[test]
    fn decode_string_length_overflow_returns_none() {
        let data = [0xFF, 0x01, 0x41, 0x42];
        let mut dec = Decoder::new(&data);
        assert!(dec.decode_string().is_none());
    }

    #[test]
    fn decode_unknown_type_tag_returns_none() {
        let mut data = vec![];
        TypeSignatureSection::NAME.encode(&mut data);
        0usize.encode(&mut data); // 0 imports
        1usize.encode(&mut data); // 1 defined type
        "test".encode(&mut data); // path.major
        "T".encode(&mut data); // path.minor
        0usize.encode(&mut data); // kind: named
        0usize.encode(&mut data); // 0 parameters
        99usize.encode(&mut data); // invalid type tag
        0usize.encode(&mut data); // 0 defined terms
        assert!(TypeSignatureSection::decode(&data).is_none());
    }

    #[test]
    fn decode_out_of_bounds_named_index_returns_none() {
        let mut data = vec![];
        TypeSignatureSection::NAME.encode(&mut data);
        0usize.encode(&mut data); // 0 imports
        1usize.encode(&mut data); // 1 defined type
        "test".encode(&mut data); // path.major
        "T".encode(&mut data); // path.minor
        0usize.encode(&mut data); // kind: named
        0usize.encode(&mut data); // 0 parameters
        11usize.encode(&mut data); // Named tag
        999usize.encode(&mut data); // out-of-bounds index into reverse_index
        0usize.encode(&mut data); // body tag: unit
        0usize.encode(&mut data); // 0 defined terms
        assert!(TypeSignatureSection::decode(&data).is_none());
    }

    #[test]
    fn decode_truncated_terms_returns_none() {
        let mut data = vec![];
        TypeSignatureSection::NAME.encode(&mut data);
        0usize.encode(&mut data); // 0 imports
        0usize.encode(&mut data); // 0 defined types
        1usize.encode(&mut data); // 1 defined term (but no term data follows)
        assert!(TypeSignatureSection::decode(&data).is_none());
    }
}
