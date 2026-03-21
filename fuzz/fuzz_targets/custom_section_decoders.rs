#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;
use wasm_encoder::Encode;

use halcyon_lib::asm::custom_section::TypeSignatureSection;
use halcyon_lib::asm::module_section::LoweredModuleSection;

fn decode_leb128_usize(data: &[u8]) -> Option<(usize, usize)> {
    let mut value = 0usize;
    let mut shift = 0usize;
    for (index, byte) in data.iter().copied().enumerate() {
        value |= ((byte & 0x7F) as usize) << shift;
        if byte & 0x80 == 0 {
            return Some((value, index + 1));
        }
        shift += 7;
        if shift >= usize::BITS as usize {
            return None;
        }
    }
    None
}

fn extract_custom_section_data(encoded: &[u8]) -> Option<&[u8]> {
    let (section_size, section_size_bytes) = decode_leb128_usize(encoded)?;
    let start = section_size_bytes;
    let end = start.checked_add(section_size)?;
    encoded.get(start..end)
}

fuzz_target!(|data: &[u8]| {
    let _ = LoweredModuleSection::decode(data);
    let _ = LoweredModuleSection::decode_data_slice(data);
    let _ = TypeSignatureSection::decode(data);
    let _ = TypeSignatureSection::decode_data_slice(data);

    if let Some(module) = LoweredModuleSection::decode_data_slice(data) {
        let section = LoweredModuleSection::new(&module);
        let mut encoded = Vec::new();
        section.encode(&mut encoded);
        if let Some(section_data) = extract_custom_section_data(&encoded) {
            let _ = LoweredModuleSection::decode(section_data);
        }
    }

    if let Some(section) = TypeSignatureSection::decode_data_slice(data) {
        let mut encoded = Vec::new();
        section.encode(&mut encoded);
        if let Some(section_data) = extract_custom_section_data(&encoded) {
            let _ = TypeSignatureSection::decode(section_data);
        }
    }
});
