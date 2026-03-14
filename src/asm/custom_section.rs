use indexmap::{
    IndexMap,
    IndexSet,
};
use serde::{
    Deserialize,
    Serialize,
};
use wasm_encoder::{
    CustomSection,
    Encode,
};

use super::*;
use crate::types::{
    Kind,
    StructMatch,
    TraitRef,
    Type as SemanticType,
    TypeDefinition,
    TypeDefinitionKind,
    TypeScheme,
    TypeTransform,
};

/// Currently, WASM does not allow type imports. Even if it did, WASM types
/// are only a simplification of Halcyon types. This section provides semantic
/// information to linkers about a module's type signature.
#[derive(Debug, Clone, Default)]
pub struct TypeSignatureSection {
    /// Types referenced in this module, but defined elsewhere.
    pub imported_types: Vec<Path>,
    /// An ordered map of type declarations from this module.
    pub defined_types: IndexMap<Path, TypeDefinition>,
    pub defined_terms: IndexMap<Path, TypeScheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TypeSignaturePayload {
    version: u32,
    imported_types: Vec<WirePath>,
    defined_types: Vec<(WirePath, WireTypeDefinition)>,
    defined_terms: Vec<(WirePath, WireTypeScheme)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WirePath {
    major: String,
    minor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WireTypeDefinition {
    parameters: u64,
    parameter_kinds: Vec<WireKind>,
    body: WireSemanticType,
    kind: WireTypeDefinitionKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WireTypeDefinitionKind {
    Named,
    Alias,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WireTypeScheme {
    predicates: Vec<WireTraitRef>,
    type_: WireSemanticType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WireTraitRef {
    trait_name: WirePath,
    arguments: Vec<WireSemanticType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WireKind {
    Type,
    Arrow(Box<WireKind>, Box<WireKind>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WireStructMatch {
    Exact,
    AtLeast,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WireSemanticType {
    Unit,
    Integer,
    Real,
    Boolean,
    String,
    Glyph,
    TypeVar(u32),
    MetaVar(u32),
    ForAll(Box<WireSemanticType>),
    Named {
        name: WirePath,
        body: Box<WireSemanticType>,
    },
    StructConstraint {
        fields: Vec<(String, WireSemanticType)>,
        mode: WireStructMatch,
    },
    Struct {
        fields: Vec<(String, WireSemanticType)>,
    },
    Array(Box<WireSemanticType>),
    Tuple(Vec<WireSemanticType>),
    Sum {
        variants: Vec<(String, WireSemanticType)>,
    },
    Function(Box<WireSemanticType>, Box<WireSemanticType>),
    Apply {
        constructor: Box<WireSemanticType>,
        arguments: Vec<WireSemanticType>,
    },
}

#[derive(Debug, Clone, Copy)]
enum TypeSignatureDecodeError {
    InvalidVersion,
    IntegerOverflow,
}

impl TypeSignatureSection {
    pub const NAME: &str = "type_signature";
    const VERSION: u32 = 2;

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
                for argument in predicate.arguments.iter() {
                    collect_imported_types(argument, &defined_types, &mut imported_types);
                }
            }
        }

        Self {
            imported_types: imported_types.into_iter().collect(),
            defined_types,
            defined_terms,
        }
    }

    pub(crate) fn rebuild_index_map_for_encoding(&mut self) {
    }

    /// Decode a complete custom section payload that includes the section name
    /// prefix.
    pub fn decode(data: &[u8]) -> Option<Self> {
        let (name, payload) = decode_named_custom_section_data(data)?;
        if name != Self::NAME {
            return None;
        }
        Self::decode_data_slice(payload)
    }

    /// Decode section data without the name prefix.
    /// Used when reading from `wasmparser`, which strips the name.
    pub fn decode_data_slice(data: &[u8]) -> Option<Self> {
        let payload = postcard::from_bytes::<TypeSignaturePayload>(data).ok()?;
        Self::from_payload(payload).ok()
    }

    fn to_payload(&self) -> TypeSignaturePayload {
        TypeSignaturePayload {
            version: Self::VERSION,
            imported_types: self.imported_types.iter().map(WirePath::from).collect(),
            defined_types: self
                .defined_types
                .iter()
                .map(|(path, definition)| {
                    (WirePath::from(path), WireTypeDefinition::from(definition))
                })
                .collect(),
            defined_terms: self
                .defined_terms
                .iter()
                .map(|(path, scheme)| (WirePath::from(path), WireTypeScheme::from(scheme)))
                .collect(),
        }
    }

    fn from_payload(payload: TypeSignaturePayload) -> Result<Self, TypeSignatureDecodeError> {
        if payload.version != Self::VERSION {
            return Err(TypeSignatureDecodeError::InvalidVersion);
        }

        let imported_types = payload
            .imported_types
            .into_iter()
            .map(WirePath::into_path)
            .collect::<Vec<_>>();

        let mut defined_types = IndexMap::with_capacity(payload.defined_types.len());
        for (path, definition) in payload.defined_types {
            defined_types.insert(path.into_path(), definition.into_type_definition()?);
        }

        let mut defined_terms = IndexMap::with_capacity(payload.defined_terms.len());
        for (path, scheme) in payload.defined_terms {
            defined_terms.insert(path.into_path(), scheme.into_type_scheme());
        }

        Ok(Self {
            imported_types,
            defined_types,
            defined_terms,
        })
    }
}

impl wasm_encoder::Section for TypeSignatureSection {
    fn id(&self) -> u8 {
        0
    }
}

impl Encode for TypeSignatureSection {
    fn encode(
        &self,
        sink: &mut Vec<u8>,
    ) {
        let payload = self.to_payload();
        let data = postcard::to_stdvec(&payload)
            .unwrap_or_else(|_| unreachable!("serializing type signature payload must succeed"));
        CustomSection {
            name: Self::NAME.into(),
            data: data.into(),
        }
        .encode(sink);
    }
}

impl WirePath {
    fn from(path: &Path) -> Self {
        Self {
            major: path.major.clone(),
            minor: path.minor.clone(),
        }
    }

    fn into_path(self) -> Path {
        Path {
            major: self.major,
            minor: self.minor,
        }
    }
}

impl WireTypeDefinition {
    fn from(type_definition: &TypeDefinition) -> Self {
        Self {
            parameters: type_definition.parameters as u64,
            parameter_kinds: type_definition
                .parameter_kinds
                .iter()
                .map(WireKind::from)
                .collect(),
            body: WireSemanticType::from(&type_definition.body),
            kind: WireTypeDefinitionKind::from(type_definition.kind),
        }
    }

    fn into_type_definition(self) -> Result<TypeDefinition, TypeSignatureDecodeError> {
        Ok(TypeDefinition {
            parameters: usize::try_from(self.parameters)
                .map_err(|_| TypeSignatureDecodeError::IntegerOverflow)?,
            parameter_kinds: self
                .parameter_kinds
                .into_iter()
                .map(WireKind::into_kind)
                .collect(),
            body: self.body.into_semantic_type(),
            kind: self.kind.into_type_definition_kind(),
        })
    }
}

impl WireTypeDefinitionKind {
    fn from(kind: TypeDefinitionKind) -> Self {
        match kind {
            TypeDefinitionKind::Named => Self::Named,
            TypeDefinitionKind::Alias => Self::Alias,
        }
    }

    fn into_type_definition_kind(self) -> TypeDefinitionKind {
        match self {
            Self::Named => TypeDefinitionKind::Named,
            Self::Alias => TypeDefinitionKind::Alias,
        }
    }
}

impl WireTypeScheme {
    fn from(type_scheme: &TypeScheme) -> Self {
        Self {
            predicates: type_scheme
                .predicates
                .iter()
                .map(WireTraitRef::from)
                .collect(),
            type_: WireSemanticType::from(&type_scheme.type_),
        }
    }

    fn into_type_scheme(self) -> TypeScheme {
        TypeScheme {
            predicates: self
                .predicates
                .into_iter()
                .map(WireTraitRef::into_trait_ref)
                .collect(),
            type_: self.type_.into_semantic_type(),
        }
    }
}

impl WireTraitRef {
    fn from(trait_ref: &TraitRef) -> Self {
        Self {
            trait_name: WirePath::from(&trait_ref.trait_name),
            arguments: trait_ref
                .arguments
                .iter()
                .map(WireSemanticType::from)
                .collect(),
        }
    }

    fn into_trait_ref(self) -> TraitRef {
        TraitRef {
            trait_name: self.trait_name.into_path(),
            arguments: self
                .arguments
                .into_iter()
                .map(WireSemanticType::into_semantic_type)
                .collect(),
        }
    }
}

impl WireKind {
    fn from(kind: &Kind) -> Self {
        match kind {
            Kind::Type => Self::Type,
            Kind::Arrow(parameter, result) => {
                Self::Arrow(
                    Box::new(Self::from(parameter)),
                    Box::new(Self::from(result)),
                )
            }
        }
    }

    fn into_kind(self) -> Kind {
        match self {
            Self::Type => Kind::Type,
            Self::Arrow(parameter, result) => {
                Kind::Arrow(
                    Box::new(parameter.into_kind()),
                    Box::new(result.into_kind()),
                )
            }
        }
    }
}

impl WireStructMatch {
    fn from(mode: StructMatch) -> Self {
        match mode {
            StructMatch::Exact => Self::Exact,
            StructMatch::AtLeast => Self::AtLeast,
        }
    }

    fn into_struct_match(self) -> StructMatch {
        match self {
            Self::Exact => StructMatch::Exact,
            Self::AtLeast => StructMatch::AtLeast,
        }
    }
}

impl WireSemanticType {
    fn from(type_: &SemanticType) -> Self {
        match type_ {
            SemanticType::Unit => Self::Unit,
            SemanticType::Integer => Self::Integer,
            SemanticType::Real => Self::Real,
            SemanticType::Boolean => Self::Boolean,
            SemanticType::String => Self::String,
            SemanticType::Glyph => Self::Glyph,
            SemanticType::TypeVar(id) => Self::TypeVar(*id),
            SemanticType::MetaVar(id) => Self::MetaVar(*id),
            SemanticType::ForAll(body) => Self::ForAll(Box::new(Self::from(body))),
            SemanticType::Named { name, body } => {
                Self::Named {
                    name: WirePath::from(name),
                    body: Box::new(Self::from(body)),
                }
            }
            SemanticType::StructConstraint { fields, mode } => {
                Self::StructConstraint {
                    fields: fields
                        .iter()
                        .map(|(name, type_)| (name.clone(), Self::from(type_)))
                        .collect(),
                    mode: WireStructMatch::from(*mode),
                }
            }
            SemanticType::Struct { fields } => {
                Self::Struct {
                    fields: fields
                        .iter()
                        .map(|(name, type_)| (name.clone(), Self::from(type_)))
                        .collect(),
                }
            }
            SemanticType::Array(inner) => Self::Array(Box::new(Self::from(inner))),
            SemanticType::Tuple(items) => Self::Tuple(items.iter().map(Self::from).collect()),
            SemanticType::Sum { variants } => {
                Self::Sum {
                    variants: variants
                        .iter()
                        .map(|(name, type_)| (name.clone(), Self::from(type_)))
                        .collect(),
                }
            }
            SemanticType::Function(parameter, result) => {
                Self::Function(
                    Box::new(Self::from(parameter)),
                    Box::new(Self::from(result)),
                )
            }
            SemanticType::Apply {
                constructor,
                arguments,
            } => {
                Self::Apply {
                    constructor: Box::new(Self::from(constructor)),
                    arguments: arguments.iter().map(Self::from).collect(),
                }
            }
        }
    }

    fn into_semantic_type(self) -> SemanticType {
        match self {
            Self::Unit => SemanticType::Unit,
            Self::Integer => SemanticType::Integer,
            Self::Real => SemanticType::Real,
            Self::Boolean => SemanticType::Boolean,
            Self::String => SemanticType::String,
            Self::Glyph => SemanticType::Glyph,
            Self::TypeVar(id) => SemanticType::TypeVar(id),
            Self::MetaVar(id) => SemanticType::MetaVar(id),
            Self::ForAll(body) => SemanticType::ForAll(Box::new(body.into_semantic_type())),
            Self::Named { name, body } => {
                SemanticType::Named {
                    name: name.into_path(),
                    body: Box::new(body.into_semantic_type()),
                }
            }
            Self::StructConstraint { fields, mode } => {
                SemanticType::StructConstraint {
                    fields: fields
                        .into_iter()
                        .map(|(name, type_)| (name, type_.into_semantic_type()))
                        .collect(),
                    mode: mode.into_struct_match(),
                }
            }
            Self::Struct { fields } => {
                SemanticType::Struct {
                    fields: fields
                        .into_iter()
                        .map(|(name, type_)| (name, type_.into_semantic_type()))
                        .collect(),
                }
            }
            Self::Array(inner) => SemanticType::Array(Box::new(inner.into_semantic_type())),
            Self::Tuple(items) => {
                SemanticType::Tuple(items.into_iter().map(Self::into_semantic_type).collect())
            }
            Self::Sum { variants } => {
                SemanticType::Sum {
                    variants: variants
                        .into_iter()
                        .map(|(name, type_)| (name, type_.into_semantic_type()))
                        .collect(),
                }
            }
            Self::Function(parameter, result) => {
                SemanticType::Function(
                    Box::new(parameter.into_semantic_type()),
                    Box::new(result.into_semantic_type()),
                )
            }
            Self::Apply {
                constructor,
                arguments,
            } => {
                SemanticType::Apply {
                    constructor: Box::new(constructor.into_semantic_type()),
                    arguments: arguments
                        .into_iter()
                        .map(Self::into_semantic_type)
                        .collect(),
                }
            }
        }
    }
}

fn decode_named_custom_section_data(data: &[u8]) -> Option<(&str, &[u8])> {
    let (name_length, name_length_bytes) = decode_leb128_usize(data)?;
    let name_start = name_length_bytes;
    let name_end = name_start.checked_add(name_length)?;
    let name = std::str::from_utf8(data.get(name_start..name_end)?).ok()?;
    Some((name, data.get(name_end..)?))
}

fn decode_leb128_usize(data: &[u8]) -> Option<(usize, usize)> {
    let mut value = 0usize;
    let mut shift = 0;

    for (index, byte) in data.iter().copied().enumerate() {
        value |= ((byte & 0x7F) as usize) << shift;
        if byte & 0x80 == 0 {
            return Some((value, index + 1));
        }
        shift += 7;
        if shift >= usize::BITS {
            return None;
        }
    }

    None
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

    fn decode_section_payload(encoded: &[u8]) -> Vec<u8> {
        let (section_size, section_size_bytes) = decode_leb128_usize(&encoded).unwrap();
        let section_data_start = section_size_bytes;
        let section_data_end = section_data_start + section_size;
        let section_data = &encoded[section_data_start..section_data_end];
        let (_, payload) = decode_named_custom_section_data(section_data).unwrap();
        payload.to_vec()
    }

    fn roundtrip_type(type_: SemanticType) {
        let path = Path::new("test", "MyType");
        let mut section = TypeSignatureSection::default();
        section.defined_types.insert(
            path,
            TypeDefinition {
                parameters: 0,
                parameter_kinds: Vec::new(),
                body: type_.clone(),
                kind: TypeDefinitionKind::Named,
            },
        );

        let mut encoded = vec![];
        section.encode(&mut encoded);
        let payload = decode_section_payload(&encoded);
        let decoded = TypeSignatureSection::decode_data_slice(&payload).unwrap();

        let original_type = &section.defined_types.values().next().unwrap().body;
        let decoded_type = &decoded.defined_types.values().next().unwrap().body;
        assert_eq!(format!("{original_type:?}"), format!("{decoded_type:?}"));
    }

    #[test]
    fn roundtrip_primitives_and_compounds() {
        roundtrip_type(SemanticType::Unit);
        roundtrip_type(SemanticType::Integer);
        roundtrip_type(SemanticType::Real);
        roundtrip_type(SemanticType::Boolean);
        roundtrip_type(SemanticType::String);
        roundtrip_type(SemanticType::Glyph);
        roundtrip_type(SemanticType::TypeVar(2));
        roundtrip_type(SemanticType::MetaVar(3));
        roundtrip_type(SemanticType::ForAll(Box::new(SemanticType::TypeVar(0))));

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

        roundtrip_type(SemanticType::Array(Box::new(SemanticType::Integer)));
        roundtrip_type(SemanticType::Tuple(vec![
            SemanticType::Integer,
            SemanticType::String,
            SemanticType::Boolean,
        ]));
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
            path,
            TypeDefinition {
                parameters: 1,
                parameter_kinds: vec![crate::types::Kind::Type],
                body,
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

        let mut encoded = vec![];
        section.encode(&mut encoded);
        let payload = decode_section_payload(&encoded);
        let decoded = TypeSignatureSection::decode_data_slice(&payload).unwrap();

        let original_term = section.defined_terms.values().next().unwrap();
        let decoded_term = decoded.defined_terms.values().next().unwrap();
        assert_eq!(format!("{original_term:?}"), format!("{decoded_term:?}"));
    }

    #[test]
    fn decode_invalid_payload_returns_none() {
        assert!(TypeSignatureSection::decode_data_slice(&[1, 2, 3]).is_none());
    }

    #[test]
    fn decode_wrong_name_returns_none() {
        let mut data = vec![];
        "not_signature".encode(&mut data);
        assert!(TypeSignatureSection::decode(&data).is_none());
    }
}
