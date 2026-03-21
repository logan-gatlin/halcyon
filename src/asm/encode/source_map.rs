use std::collections::BTreeMap;

use rayon::prelude::*;

use super::super::*;

/// Handles build source map json.
pub(crate) fn build_source_map_json(
    module: &Module,
    binary: &[u8],
    function_operator_origins: &[Vec<Option<SourceOrigin>>],
) -> Option<String> {
    let function_offsets = read_function_operator_offsets(binary)?;
    let mut mappings = function_offsets
        .par_iter()
        .zip(function_operator_origins.par_iter())
        .flat_map_iter(|(offsets, origins)| {
            offsets
                .iter()
                .enumerate()
                .filter_map(move |(index, offset)| {
                    origins
                        .get(index)
                        .and_then(|origin| origin.clone())
                        .map(|origin| (*offset, origin))
                })
        })
        .collect::<Vec<_>>();

    if mappings.is_empty() {
        return None;
    }

    mappings.sort_by_key(|(offset, _)| *offset);

    let mut sources = module.source_files.keys().cloned().collect::<Vec<_>>();
    for (_, origin) in &mappings {
        if !sources.contains(&origin.file_name) {
            sources.push(origin.file_name.clone());
        }
    }

    let source_indexes = sources
        .iter()
        .enumerate()
        .map(|(index, source)| (source.clone(), index as i64))
        .collect::<BTreeMap<_, _>>();
    let source_contents = sources
        .iter()
        .map(|source| {
            module
                .source_files
                .get(source)
                .map(|record| record.source.clone())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();

    let mappings = encode_source_map_mappings(&mappings, &source_indexes, module);

    serde_json::to_string(&serde_json::json!({
        "version": 3,
        "file": format!("{}.wasm", module.name),
        "sources": sources,
        "sourcesContent": source_contents,
        "names": [],
        "mappings": mappings,
    }))
    .ok()
}

/// Handles read function operator offsets.
fn read_function_operator_offsets(binary: &[u8]) -> Option<Vec<Vec<usize>>> {
    let mut functions = Vec::new();
    for payload in wasmparser::Parser::new(0).parse_all(binary) {
        let Ok(payload) = payload else {
            return None;
        };
        if let wasmparser::Payload::CodeSectionEntry(body) = payload {
            let mut offsets = Vec::new();
            let mut reader = body.get_operators_reader().ok()?;
            while !reader.eof() {
                let offset = reader.original_position();
                let operator = reader.read().ok()?;
                if !matches!(operator, wasmparser::Operator::End) {
                    offsets.push(offset);
                }
            }
            functions.push(offsets);
        }
    }
    Some(functions)
}

/// Handles encode source map mappings.
fn encode_source_map_mappings(
    mappings: &[(usize, SourceOrigin)],
    source_indexes: &BTreeMap<String, i64>,
    module: &Module,
) -> String {
    let mut output = String::new();
    let mut previous_generated_column = 0i64;
    let mut previous_source = 0i64;
    let mut previous_original_line = 0i64;
    let mut previous_original_column = 0i64;

    for (idx, (generated_column, origin)) in mappings.iter().enumerate() {
        if idx > 0 {
            output.push(',');
        }

        let source_index = source_indexes.get(&origin.file_name).copied().unwrap_or(0);
        let (line, column) = original_line_column(origin, module);
        let generated_column = *generated_column as i64;

        output.push_str(&encode_vlq(generated_column - previous_generated_column));
        output.push_str(&encode_vlq(source_index - previous_source));
        output.push_str(&encode_vlq(line - previous_original_line));
        output.push_str(&encode_vlq(column - previous_original_column));

        previous_generated_column = generated_column;
        previous_source = source_index;
        previous_original_line = line;
        previous_original_column = column;
    }

    output
}

/// Handles original line column.
fn original_line_column(
    origin: &SourceOrigin,
    module: &Module,
) -> (i64, i64) {
    let Some(file) = module.source_files.get(&origin.file_name) else {
        return (0, 0);
    };

    let mut line = 0i64;
    let mut column = 0i64;
    let mut consumed = 0usize;
    let target = origin.start.min(file.source.len());
    for ch in file.source.chars() {
        if consumed >= target {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
        consumed += ch.len_utf8();
    }
    (line, column)
}

/// Handles encode vlq.
fn encode_vlq(value: i64) -> String {
    const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut value = if value < 0 {
        ((-value as u64) << 1) | 1
    } else {
        (value as u64) << 1
    };
    let mut encoded = String::new();
    loop {
        let mut digit = (value & 0x1F) as u8;
        value >>= 5;
        if value > 0 {
            digit |= 0x20;
        }
        encoded.push(BASE64[digit as usize] as char);
        if value == 0 {
            break;
        }
    }
    encoded
}
