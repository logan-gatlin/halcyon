#![allow(dead_code)]

use std::sync::OnceLock;

use indexmap::IndexMap;
use libfuzzer_sys::arbitrary::{
    self,
    Unstructured,
};

use halcyon_lib::ir::Path;
use halcyon_lib::types::{
    StructMatch,
    SymbolTable,
    TraitConstraint,
    Type,
};
use halcyon_lib::{
    compile_core_module_with_debug_info,
    Logger,
};

pub fn bounded_source(
    data: &[u8],
    max_bytes: usize,
) -> String {
    let len = data.len().min(max_bytes);
    String::from_utf8_lossy(&data[..len]).into_owned()
}

pub fn source_from_unstructured(
    u: &mut Unstructured<'_>,
    max_bytes: usize,
) -> arbitrary::Result<String> {
    let len = u.int_in_range(0..=max_bytes)?;
    let bytes = u.bytes(len)?;
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

pub fn clamp_to_char_boundary(
    source: &str,
    byte_offset: usize,
) -> usize {
    let mut clamped = byte_offset.min(source.len());
    while clamped > 0 && !source.is_char_boundary(clamped) {
        clamped -= 1;
    }
    clamped
}

#[derive(Debug, Clone)]
pub struct ByteCursor<'a> {
    data: &'a [u8],
    index: usize,
}

impl<'a> ByteCursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, index: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.index)
    }

    pub fn next_bool(&mut self) -> bool {
        self.next_u8().is_some_and(|byte| byte % 2 == 1)
    }

    pub fn next_usize(
        &mut self,
        max_exclusive: usize,
    ) -> usize {
        if max_exclusive == 0 {
            return 0;
        }
        (self.next_u64() as usize) % max_exclusive
    }

    pub fn take(
        &mut self,
        len: usize,
    ) -> &'a [u8] {
        let start = self.index;
        let end = start.saturating_add(len).min(self.data.len());
        self.index = end;
        &self.data[start..end]
    }

    fn next_u8(&mut self) -> Option<u8> {
        let byte = self.data.get(self.index).copied();
        if byte.is_some() {
            self.index += 1;
        }
        byte
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = 0u64;
        for shift in 0..8 {
            value |= (self.next_u8().unwrap_or(0) as u64) << (shift * 8);
        }
        value
    }
}

static CORE_SYMBOLS: OnceLock<SymbolTable> = OnceLock::new();

pub fn core_symbols() -> SymbolTable {
    CORE_SYMBOLS
        .get_or_init(|| {
            let mut symbols = SymbolTable::new();
            let mut logger = Logger::new();
            let _ = compile_core_module_with_debug_info(&mut symbols, &mut logger, false, false);
            symbols
        })
        .clone()
}

pub fn arbitrary_identifier(
    u: &mut Unstructured<'_>,
    max_len: usize,
) -> arbitrary::Result<String> {
    let max_len = max_len.clamp(1, 16);
    let len = u.int_in_range(1..=max_len)?;
    let mut ident = String::with_capacity(len);
    for index in 0..len {
        if index == 0 {
            let ch = (b'a' + (u.int_in_range(0..=25)? as u8)) as char;
            ident.push(ch);
            continue;
        }
        let class = u.int_in_range(0..=3)?;
        match class {
            0 => ident.push((b'a' + (u.int_in_range(0..=25)? as u8)) as char),
            1 => ident.push((b'0' + (u.int_in_range(0..=9)? as u8)) as char),
            2 => ident.push('_'),
            _ => ident.push('-'),
        }
    }
    Ok(ident)
}

pub fn arbitrary_path(u: &mut Unstructured<'_>) -> arbitrary::Result<Path> {
    let major = arbitrary_identifier(u, 8)?;
    let segment_count = u.int_in_range(1..=3)?;
    let mut segments = Vec::with_capacity(segment_count);
    for _ in 0..segment_count {
        segments.push(arbitrary_identifier(u, 12)?);
    }
    Ok(Path::new(major, segments.join(Path::DELIMETER)))
}

pub fn arbitrary_type(
    u: &mut Unstructured<'_>,
    depth: u8,
    meta_pool: &[Type],
) -> arbitrary::Result<Type> {
    let max_variant = if depth == 0 { 7 } else { 16 };
    let variant: u8 = u.int_in_range(0..=max_variant)?;
    Ok(match variant {
        0 => Type::Unit,
        1 => Type::Integer,
        2 => Type::Real,
        3 => Type::Boolean,
        4 => Type::String,
        5 => Type::Glyph,
        6 => Type::TypeVar(u.int_in_range(0..=6)?),
        7 => {
            if meta_pool.is_empty() {
                Type::TypeVar(u.int_in_range(0..=6)?)
            } else {
                meta_pool[u.int_in_range(0..=meta_pool.len() - 1)?].clone()
            }
        }
        8 => {
            Type::ForAll {
                name: None,
                body: Box::new(arbitrary_type(u, depth.saturating_sub(1), meta_pool)?),
            }
        }
        9 => {
            Type::Named {
                name: arbitrary_path(u)?,
                body: Box::new(arbitrary_type(u, depth.saturating_sub(1), meta_pool)?),
            }
        }
        10 => {
            Type::StructConstraint {
                fields: arbitrary_field_map(u, depth.saturating_sub(1), meta_pool, 6)?,
                mode: if u.arbitrary::<bool>()? {
                    StructMatch::Exact
                } else {
                    StructMatch::AtLeast
                },
            }
        }
        11 => {
            Type::Struct {
                fields: arbitrary_field_map(u, depth.saturating_sub(1), meta_pool, 6)?,
            }
        }
        12 => {
            Type::Array(Box::new(arbitrary_type(
                u,
                depth.saturating_sub(1),
                meta_pool,
            )?))
        }
        13 => {
            let count = u.int_in_range(0..=4)?;
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(arbitrary_type(u, depth.saturating_sub(1), meta_pool)?);
            }
            Type::Tuple(items)
        }
        14 => {
            let count = u.int_in_range(0..=4)?;
            let mut variants = IndexMap::new();
            for index in 0..count {
                let name = arbitrary_identifier(u, 10).unwrap_or_else(|_| format!("V{index}"));
                let payload = arbitrary_type(u, depth.saturating_sub(1), meta_pool)?;
                variants.insert(name, payload);
            }
            Type::Sum { variants }
        }
        15 => {
            Type::Function(
                Box::new(arbitrary_type(u, depth.saturating_sub(1), meta_pool)?),
                Box::new(arbitrary_type(u, depth.saturating_sub(1), meta_pool)?),
            )
        }
        _ => {
            let argument_count = u.int_in_range(0..=3)?;
            let constructor = arbitrary_type(u, depth.saturating_sub(1), meta_pool)?;
            let mut arguments = Vec::with_capacity(argument_count);
            for _ in 0..argument_count {
                arguments.push(arbitrary_type(u, depth.saturating_sub(1), meta_pool)?);
            }
            Type::Apply {
                constructor: Box::new(constructor),
                arguments,
            }
        }
    })
}

pub fn arbitrary_trait_constraint(
    u: &mut Unstructured<'_>,
    depth: u8,
    meta_pool: &[Type],
) -> arbitrary::Result<TraitConstraint> {
    let argument_count = u.int_in_range(0..=3)?;
    let mut arguments = Vec::with_capacity(argument_count);
    for _ in 0..argument_count {
        arguments.push(arbitrary_type(u, depth, meta_pool)?);
    }
    Ok(TraitConstraint::new(arbitrary_path(u)?, arguments))
}

pub fn arbitrary_predicates(
    u: &mut Unstructured<'_>,
    max_count: usize,
    depth: u8,
    meta_pool: &[Type],
) -> arbitrary::Result<Vec<TraitConstraint>> {
    let count = u.int_in_range(0..=max_count.min(8))?;
    let mut predicates = Vec::with_capacity(count);
    for _ in 0..count {
        predicates.push(arbitrary_trait_constraint(u, depth, meta_pool)?);
    }
    Ok(predicates)
}

fn arbitrary_field_map(
    u: &mut Unstructured<'_>,
    depth: u8,
    meta_pool: &[Type],
    max_fields: usize,
) -> arbitrary::Result<IndexMap<String, Type>> {
    let count = u.int_in_range(0..=max_fields.min(8))?;
    let mut fields = IndexMap::with_capacity(count);
    for index in 0..count {
        let name = arbitrary_identifier(u, 12).unwrap_or_else(|_| format!("f{index}"));
        fields.insert(name, arbitrary_type(u, depth, meta_pool)?);
    }
    Ok(fields)
}
