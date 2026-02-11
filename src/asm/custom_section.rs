use std::collections::HashMap;

use wasm_encoder::{
    CustomSection,
    Encode,
};

use crate::map::*;
use crate::semantic::AbstractType;

use super::*;

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
pub struct SignatureSection {
    /// Types referenced in this module, but defined elsewhere
    pub imported_types: Vec<Path>,
    /// An ordered map of type declarations from this module
    pub defined_types: IndexMap<Path, AbstractType>,
    pub defined_terms: IndexMap<Path, semantic::Type>,
    /// The index of each referenced type in this section
    index_map: HashMap<Path, usize>,
}

impl SignatureSection {
    pub const NAME: &str = "type_signature";
    pub fn new(
        module_name: &str,
        symbols: &SymbolTable,
    ) -> Self {
        let mut imported_types = vec![];
        let mut defined_types = IndexMap::new();
        let mut defined_terms = IndexMap::new();

        // PERFORMANCE:
        // This is O(n^2) because it iterates over every type, and is called for
        // every module in the symbol table. To fix, the symbol table will need
        // to be changed to a doubly nested indexmap.
        for (path, t) in &symbols.types {
            if path.major == module_name {
                defined_types.insert(path.clone(), t.clone());
                t.clone().visit(|t: &mut semantic::Type| {
                    if let semantic::Type::Instantiation(path, _) = t
                        && !defined_types.contains_key(path)
                    {
                        imported_types.push(path.clone());
                    }
                });
            }
        }
        for (path, t) in &symbols.terms {
            if path.major == module_name {
                defined_terms.insert(path.clone(), t.clone());
                t.clone().visit(|t: &mut semantic::Type| {
                    if let semantic::Type::Instantiation(path, _) = t
                        && !defined_types.contains_key(path)
                    {
                        imported_types.push(path.clone());
                    }
                });
            }
        }
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
                variant_names.len().encode(sink);
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
                self.index_map[path].encode(sink);
                items.len().encode(sink);
                for item in items {
                    self.encode_type(item, sink);
                }
            }
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
        // Decode imported types
        let import_count = dec.decode_usize()?;
        let mut imported_types = Vec::with_capacity(import_count);
        for _ in 0..import_count {
            imported_types.push(dec.decode_path()?);
        }

        // Build reverse index for decoding Instantiation references
        let mut reverse_index: Vec<Path> = imported_types.clone();

        // Decode defined types
        let defined_count = dec.decode_usize()?;
        let mut defined_types = IndexMap::new();
        for _ in 0..defined_count {
            let path = dec.decode_path()?;
            reverse_index.push(path.clone());
            let base = Self::decode_type(dec, &path, &reverse_index)?;
            let var_count = dec.decode_usize()?;
            let mut variables = Vec::with_capacity(var_count);
            for _ in 0..var_count {
                variables.push(dec.decode_u64()?);
            }
            defined_types.insert(
                path,
                AbstractType {
                    variables: variables.into_boxed_slice(),
                    base,
                },
            );
        }

        let defined_count = dec.decode_usize()?;
        let mut defined_terms = IndexMap::new();
        for _ in 0..defined_count {
            let path = dec.decode_path()?;
            let type_ = Self::decode_type(dec, &path, &reverse_index)?;
            defined_terms.insert(path, type_);
        }

        // Rebuild index_map
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
        this_path: &Path,
        reverse_index: &[Path],
    ) -> Option<semantic::Type> {
        use semantic::Type::*;
        Some(match dec.decode_usize()? {
            0 => Unit,
            1 => Integer,
            2 => Real,
            3 => Boolean,
            4 => String,
            5 => Glyph,
            6 => Variable(dec.decode_u64()?),
            7 => {
                let field_count = dec.decode_usize()?;
                let mut fields = IndexMap::new();
                for _ in 0..field_count {
                    let name = dec.decode_string()?;
                    let type_ = Self::decode_type(dec, this_path, reverse_index)?;
                    fields.insert(name, type_);
                }
                Struct {
                    name: this_path.clone(),
                    fields,
                }
            }
            8 => Array(Box::new(Self::decode_type(dec, this_path, reverse_index)?)),
            9 => {
                let item_count = dec.decode_usize()?;
                let mut items = Vec::with_capacity(item_count);
                for _ in 0..item_count {
                    items.push(Self::decode_type(dec, this_path, reverse_index)?);
                }
                Tuple(items)
            }
            10 => {
                let variant_count = dec.decode_usize()?;
                let mut variant_names = Vec::with_capacity(variant_count);
                let mut variant_types = Vec::with_capacity(variant_count);
                for _ in 0..variant_count {
                    variant_names.push(dec.decode_string()?);
                    variant_types.push(Self::decode_type(dec, this_path, reverse_index)?);
                }
                Sum {
                    name: Path::default(),
                    variant_names,
                    variant_types,
                }
            }
            11 => {
                let param = Self::decode_type(dec, this_path, reverse_index)?;
                let result = Self::decode_type(dec, this_path, reverse_index)?;
                Function(Box::new(param), Box::new(result))
            }
            12 => {
                let idx = dec.decode_usize()?;
                let path = reverse_index.get(idx)?.clone();
                let item_count = dec.decode_usize()?;
                let mut items = Vec::with_capacity(item_count);
                for _ in 0..item_count {
                    items.push(Self::decode_type(dec, this_path, reverse_index)?);
                }
                Instantiation(path, items)
            }
            _ => return None,
        })
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
        let mut data = vec![];
        self.imported_types.encode(&mut data);
        self.defined_types.len().encode(&mut data);
        for (path, t) in &self.defined_types {
            path.encode(&mut data);
            self.encode_type(&t.base, &mut data);
            t.variables.encode(&mut data);
        }
        self.defined_terms.len().encode(&mut data);
        for (path, t) in &self.defined_terms {
            path.encode(&mut data);
            self.encode_type(t, &mut data);
        }
        CustomSection {
            name: Self::NAME.into(),
            data: data.into(),
        }
        .encode(sink);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn roundtrip_type(type_: semantic::Type) {
        let path = Path {
            major: "test".into(),
            minor: "MyType".into(),
        };
        let mut section = SignatureSection::default();
        section.defined_types.insert(
            path.clone(),
            AbstractType {
                variables: Box::new([]),
                base: type_.clone(),
            },
        );
        section.index_map.insert(path, 0);

        let mut encoded = vec![];
        section.encode(&mut encoded);

        // Skip the LEB128 length prefix
        let mut pos = 0;
        while encoded[pos] & 0x80 != 0 {
            pos += 1;
        }
        pos += 1;

        let decoded = SignatureSection::decode(&encoded[pos..]).unwrap();
        assert_eq!(section.defined_types.len(), decoded.defined_types.len());

        let original_type = &section.defined_types.values().next().unwrap().base;
        let decoded_type = &decoded.defined_types.values().next().unwrap().base;
        assert_eq!(format!("{original_type:?}"), format!("{decoded_type:?}"));
    }

    #[test]
    fn roundtrip_unit() {
        roundtrip_type(semantic::Type::Unit);
    }

    #[test]
    fn roundtrip_integer() {
        roundtrip_type(semantic::Type::Integer);
    }

    #[test]
    fn roundtrip_real() {
        roundtrip_type(semantic::Type::Real);
    }

    #[test]
    fn roundtrip_boolean() {
        roundtrip_type(semantic::Type::Boolean);
    }

    #[test]
    fn roundtrip_string() {
        roundtrip_type(semantic::Type::String);
    }

    #[test]
    fn roundtrip_glyph() {
        roundtrip_type(semantic::Type::Glyph);
    }

    #[test]
    fn roundtrip_variable() {
        roundtrip_type(semantic::Type::Variable(42));
    }

    #[test]
    fn roundtrip_struct() {
        let mut fields = IndexMap::new();
        fields.insert("x".into(), semantic::Type::Integer);
        fields.insert("y".into(), semantic::Type::Real);
        roundtrip_type(semantic::Type::Struct {
            name: Path::new("test", "MyType"),
            fields,
        });
    }

    #[test]
    fn roundtrip_array() {
        roundtrip_type(semantic::Type::Array(Box::new(semantic::Type::Integer)));
    }

    #[test]
    fn roundtrip_tuple() {
        roundtrip_type(semantic::Type::Tuple(vec![
            semantic::Type::Integer,
            semantic::Type::String,
            semantic::Type::Boolean,
        ]));
    }

    #[test]
    fn roundtrip_sum() {
        roundtrip_type(semantic::Type::Sum {
            name: Path::default(),
            variant_names: vec!["None".into(), "Some".into()],
            variant_types: vec![semantic::Type::Unit, semantic::Type::Integer],
        });
    }

    #[test]
    fn roundtrip_function() {
        roundtrip_type(semantic::Type::Function(
            Box::new(semantic::Type::Integer),
            Box::new(semantic::Type::String),
        ));
    }

    #[test]
    fn roundtrip_nested() {
        roundtrip_type(semantic::Type::Function(
            Box::new(semantic::Type::Tuple(vec![
                semantic::Type::Integer,
                semantic::Type::Array(Box::new(semantic::Type::String)),
            ])),
            Box::new(semantic::Type::Boolean),
        ));
    }

    #[test]
    fn decode_empty_returns_none() {
        assert!(SignatureSection::decode(&[]).is_none());
    }

    #[test]
    fn decode_wrong_name_returns_none() {
        let mut data = vec![];
        "not_signature".encode(&mut data);
        assert!(SignatureSection::decode(&data).is_none());
    }

    #[test]
    fn decode_truncated_after_name_returns_none() {
        let mut data = vec![];
        SignatureSection::NAME.encode(&mut data);
        // No import count follows — truncated
        assert!(SignatureSection::decode(&data).is_none());
    }

    #[test]
    fn decode_truncated_leb128_returns_none() {
        // A continuation byte (high bit set) with no following byte
        assert!(SignatureSection::decode(&[0x80]).is_none());
    }

    #[test]
    fn decode_oversized_leb128_returns_none() {
        // 10 continuation bytes exceeds the 64-bit shift limit
        let data: Vec<u8> = std::iter::repeat_n(0x80, 10).chain([0x01]).collect();
        let mut dec = Decoder::new(&data);
        assert!(dec.decode_u64().is_none());
    }

    #[test]
    fn decode_invalid_utf8_returns_none() {
        // Length prefix = 2, then two invalid UTF-8 bytes
        let data = [0x02, 0xFF, 0xFE];
        let mut dec = Decoder::new(&data);
        assert!(dec.decode_string().is_none());
    }

    #[test]
    fn decode_string_length_overflow_returns_none() {
        // Length prefix claims 255 bytes, but only 2 bytes follow
        let data = [0xFF, 0x01, 0x41, 0x42];
        let mut dec = Decoder::new(&data);
        assert!(dec.decode_string().is_none());
    }

    #[test]
    fn decode_unknown_type_tag_returns_none() {
        let mut data = vec![];
        SignatureSection::NAME.encode(&mut data);
        0usize.encode(&mut data); // 0 imports
        1usize.encode(&mut data); // 1 defined type
        "test".encode(&mut data); // path.major
        "T".encode(&mut data); // path.minor
        99usize.encode(&mut data); // invalid type tag
        0usize.encode(&mut data); // 0 variables
        0usize.encode(&mut data); // 0 defined terms
        assert!(SignatureSection::decode(&data).is_none());
    }

    #[test]
    fn decode_out_of_bounds_instantiation_index_returns_none() {
        let mut data = vec![];
        SignatureSection::NAME.encode(&mut data);
        0usize.encode(&mut data); // 0 imports
        1usize.encode(&mut data); // 1 defined type
        "test".encode(&mut data); // path.major
        "T".encode(&mut data); // path.minor
        12usize.encode(&mut data); // Instantiation tag
        999usize.encode(&mut data); // out-of-bounds index into reverse_index
        0usize.encode(&mut data); // 0 type args
        0usize.encode(&mut data); // 0 variables
        0usize.encode(&mut data); // 0 defined terms
        assert!(SignatureSection::decode(&data).is_none());
    }

    #[test]
    fn roundtrip_with_type_variables() {
        let path = Path {
            major: "test".into(),
            minor: "Generic".into(),
        };
        let mut section = SignatureSection::default();
        section.defined_types.insert(
            path.clone(),
            AbstractType {
                variables: Box::new([1, 2, 3]),
                base: semantic::Type::Tuple(vec![
                    semantic::Type::Variable(1),
                    semantic::Type::Variable(2),
                    semantic::Type::Variable(3),
                ]),
            },
        );
        section.index_map.insert(path, 0);

        let mut encoded = vec![];
        section.encode(&mut encoded);

        let mut pos = 0;
        while encoded[pos] & 0x80 != 0 {
            pos += 1;
        }
        pos += 1;

        let decoded = SignatureSection::decode(&encoded[pos..]).unwrap();
        let original = section.defined_types.values().next().unwrap();
        let decoded_val = decoded.defined_types.values().next().unwrap();
        assert_eq!(original.variables.len(), decoded_val.variables.len());
        assert_eq!(
            format!("{:?}", original.base),
            format!("{:?}", decoded_val.base)
        );
    }

    #[test]
    fn roundtrip_term() {
        let path = Path {
            major: "test".into(),
            minor: "my_fn".into(),
        };
        let mut section = SignatureSection::default();
        section.defined_terms.insert(
            path,
            semantic::Type::Function(
                Box::new(semantic::Type::Integer),
                Box::new(semantic::Type::String),
            ),
        );

        let mut encoded = vec![];
        section.encode(&mut encoded);

        let mut pos = 0;
        while encoded[pos] & 0x80 != 0 {
            pos += 1;
        }
        pos += 1;

        let decoded = SignatureSection::decode(&encoded[pos..]).unwrap();
        assert_eq!(section.defined_terms.len(), decoded.defined_terms.len());
        let original = section.defined_terms.values().next().unwrap();
        let decoded_val = decoded.defined_terms.values().next().unwrap();
        assert_eq!(format!("{original:?}"), format!("{decoded_val:?}"));
    }

    #[test]
    fn roundtrip_types_and_terms() {
        let type_path = Path {
            major: "test".into(),
            minor: "MyType".into(),
        };
        let term_path = Path {
            major: "test".into(),
            minor: "make".into(),
        };

        let mut section = SignatureSection::default();
        section.defined_types.insert(
            type_path.clone(),
            AbstractType {
                variables: Box::new([1]),
                base: semantic::Type::Struct {
                    name: Path::new("test", "MyType"),
                    fields: {
                        let mut f = IndexMap::new();
                        f.insert("value".into(), semantic::Type::Variable(1));
                        f
                    },
                },
            },
        );
        section.index_map.insert(type_path.clone(), 0);

        section.defined_terms.insert(
            term_path,
            semantic::Type::Function(
                Box::new(semantic::Type::Variable(1)),
                Box::new(semantic::Type::Instantiation(
                    type_path,
                    vec![semantic::Type::Variable(1)],
                )),
            ),
        );

        let mut encoded = vec![];
        section.encode(&mut encoded);

        let mut pos = 0;
        while encoded[pos] & 0x80 != 0 {
            pos += 1;
        }
        pos += 1;

        let decoded = SignatureSection::decode(&encoded[pos..]).unwrap();
        assert_eq!(section.defined_types.len(), decoded.defined_types.len());
        assert_eq!(section.defined_terms.len(), decoded.defined_terms.len());

        let orig_type = &section.defined_types.values().next().unwrap().base;
        let dec_type = &decoded.defined_types.values().next().unwrap().base;
        assert_eq!(format!("{orig_type:?}"), format!("{dec_type:?}"));

        let orig_term = section.defined_terms.values().next().unwrap();
        let dec_term = decoded.defined_terms.values().next().unwrap();
        assert_eq!(format!("{orig_term:?}"), format!("{dec_term:?}"));
    }

    #[test]
    fn decode_truncated_terms_returns_none() {
        let mut data = vec![];
        SignatureSection::NAME.encode(&mut data);
        0usize.encode(&mut data); // 0 imports
        0usize.encode(&mut data); // 0 defined types
        1usize.encode(&mut data); // 1 defined term (but no term data follows)
        assert!(SignatureSection::decode(&data).is_none());
    }
}
