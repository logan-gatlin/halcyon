use std::collections::BTreeMap;

use gimli::write::{
    Address,
    AttributeValue,
    DwarfUnit,
    EndianVec,
    FileId,
    LineProgram,
    LineString,
    Sections,
};
use gimli::{
    Encoding,
    Format,
    LineEncoding,
    LittleEndian,
    constants,
};
use rayon::prelude::*;

use super::super::*;

struct DwarfSequence {
    start_address: u64,
    end_address: u64,
    rows: Vec<(u64, String, u64, u64)>,
}

struct ParsedFunctionOffsets {
    functions: Vec<Vec<usize>>,
}

/// Builds DWARF custom sections for wasm backtraces.
pub(crate) fn build_dwarf_sections(
    module: &Module,
    binary: &[u8],
    function_operator_origins: &[Vec<Option<SourceOrigin>>],
) -> Vec<(String, Vec<u8>)> {
    let Some(parsed_offsets) = read_function_operator_offsets(binary) else {
        return Vec::new();
    };

    let mut sequences = parsed_offsets
        .functions
        .par_iter()
        .zip(function_operator_origins.par_iter())
        .filter_map(|(offsets, origins)| {
            let start = offsets.first().copied()?;
            let end = offsets.last().copied().unwrap_or(start).saturating_add(1) as u64;
            let rows = offsets
                .iter()
                .enumerate()
                .filter_map(|(index, offset)| {
                    let origin = origins.get(index).and_then(|origin| origin.clone())?;
                    let (line, column) = original_line_column(&origin, module);
                    Some((*offset as u64, origin.file_name, line, column))
                })
                .collect::<Vec<_>>();
            if rows.is_empty() {
                return None;
            }
            Some(DwarfSequence {
                start_address: start as u64,
                end_address: end,
                rows,
            })
        })
        .collect::<Vec<_>>();

    if sequences.is_empty() {
        return Vec::new();
    }
    sequences.sort_by_key(|sequence| sequence.start_address);

    emit_dwarf(module, &sequences).unwrap_or_default()
}

fn emit_dwarf(
    module: &Module,
    sequences: &[DwarfSequence],
) -> Option<Vec<(String, Vec<u8>)>> {
    let encoding = Encoding {
        format: Format::Dwarf32,
        version: 5,
        address_size: 4,
    };

    let mut dwarf = DwarfUnit::new(encoding);
    let mut line_program = LineProgram::new(
        encoding,
        LineEncoding::default(),
        LineString::String(b".".to_vec()),
        None,
        LineString::String(primary_source_name(module).into_bytes()),
        None,
    );

    let default_directory = line_program.default_directory();
    let mut file_ids = BTreeMap::<String, FileId>::new();

    for sequence in sequences {
        for (_, file_name, ..) in &sequence.rows {
            file_ids.entry(file_name.clone()).or_insert_with(|| {
                line_program.add_file(
                    LineString::String(file_name.clone().into_bytes()),
                    default_directory,
                    None,
                )
            });
        }
    }

    for sequence in sequences {
        let Some((first_address, first_file, first_line, first_column)) = sequence.rows.first()
        else {
            continue;
        };
        let Some(first_file_id) = file_ids.get(first_file).copied() else {
            continue;
        };

        line_program.begin_sequence(Some(Address::Constant(sequence.start_address)));
        {
            let row = line_program.row();
            row.address_offset = first_address.saturating_sub(sequence.start_address);
            row.file = first_file_id;
            row.line = *first_line;
            row.column = *first_column;
        }
        line_program.generate_row();

        for (address, file_name, line, column) in sequence.rows.iter().skip(1) {
            let Some(file_id) = file_ids.get(file_name).copied() else {
                continue;
            };
            let row = line_program.row();
            row.address_offset = address.saturating_sub(sequence.start_address);
            row.file = file_id;
            row.line = *line;
            row.column = *column;
            line_program.generate_row();
        }

        line_program.end_sequence(sequence.end_address.saturating_sub(sequence.start_address));
    }

    dwarf.unit.line_program = line_program;
    let root = dwarf.unit.root();
    let producer = dwarf
        .strings
        .add(format!("Halcyon {}", crate::COMPILER_VERSION_STRING));
    let name = dwarf.strings.add(module.name.clone());
    let comp_dir = dwarf.strings.add(".".to_string());
    let root = dwarf.unit.get_mut(root);
    let low_pc = sequences
        .iter()
        .map(|sequence| sequence.start_address)
        .min()
        .unwrap_or(0);
    let high_pc = sequences
        .iter()
        .map(|sequence| sequence.end_address)
        .max()
        .unwrap_or(low_pc);
    root.set(
        constants::DW_AT_low_pc,
        AttributeValue::Address(Address::Constant(low_pc)),
    );
    root.set(
        constants::DW_AT_high_pc,
        AttributeValue::Address(Address::Constant(high_pc)),
    );
    root.set(
        constants::DW_AT_producer,
        AttributeValue::StringRef(producer),
    );
    root.set(constants::DW_AT_name, AttributeValue::StringRef(name));
    root.set(
        constants::DW_AT_comp_dir,
        AttributeValue::StringRef(comp_dir),
    );

    let mut sections = Sections::new(EndianVec::new(LittleEndian));
    dwarf.write(&mut sections).ok()?;

    let mut out = Vec::new();
    push_if_non_empty(&mut out, ".debug_abbrev", sections.debug_abbrev.slice());
    push_if_non_empty(&mut out, ".debug_info", sections.debug_info.slice());
    push_if_non_empty(&mut out, ".debug_line", sections.debug_line.slice());
    push_if_non_empty(&mut out, ".debug_str", sections.debug_str.slice());
    push_if_non_empty(&mut out, ".debug_line_str", sections.debug_line_str.slice());
    Some(out)
}

fn push_if_non_empty(
    out: &mut Vec<(String, Vec<u8>)>,
    name: &str,
    data: &[u8],
) {
    if data.is_empty() {
        return;
    }
    out.push((name.to_string(), data.to_vec()));
}

fn primary_source_name(module: &Module) -> String {
    module
        .source_files
        .keys()
        .next()
        .cloned()
        .unwrap_or_else(|| format!("{}.hc", module.name))
}

fn read_function_operator_offsets(binary: &[u8]) -> Option<ParsedFunctionOffsets> {
    let mut functions = Vec::new();
    let mut code_section_start = None;
    for payload in wasmparser::Parser::new(0).parse_all(binary) {
        let Ok(payload) = payload else {
            return None;
        };
        if let wasmparser::Payload::CodeSectionStart { range, .. } = payload {
            code_section_start = Some(range.start);
            continue;
        }
        if let wasmparser::Payload::CodeSectionEntry(body) = payload {
            let code_section_start = code_section_start?;
            let mut offsets = Vec::new();
            let mut reader = body.get_operators_reader().ok()?;
            while !reader.eof() {
                let offset = reader.original_position();
                let operator = reader.read().ok()?;
                if !matches!(operator, wasmparser::Operator::End) {
                    offsets.push(offset.saturating_sub(code_section_start));
                }
            }
            functions.push(offsets);
        }
    }
    Some(ParsedFunctionOffsets { functions })
}

fn original_line_column(
    origin: &SourceOrigin,
    module: &Module,
) -> (u64, u64) {
    let Some(file) = module.source_files.get(&origin.file_name) else {
        return (1, 1);
    };

    let mut line = 1u64;
    let mut column = 1u64;
    let mut consumed = 0usize;
    let target = origin.start.min(file.source.len());
    for ch in file.source.chars() {
        if consumed >= target {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
        consumed += ch.len_utf8();
    }
    (line, column)
}
