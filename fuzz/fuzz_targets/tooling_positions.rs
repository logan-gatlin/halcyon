#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;

use halcyon_lib::tooling;

use common::{
    ByteCursor,
    bounded_source,
    clamp_to_char_boundary,
};

fuzz_target!(|data: &[u8]| {
    let source = bounded_source(data, 32_768);
    let mut cursor = ByteCursor::new(data);
    let checks = cursor.next_usize(64);

    for _ in 0..checks {
        let offset = cursor.next_usize(source.len().saturating_add(32));
        let position = tooling::byte_offset_to_utf16_position(&source, offset);
        if let Some(roundtrip) =
            tooling::utf16_position_to_byte_offset(&source, position.line, position.character)
        {
            let clamped = clamp_to_char_boundary(&source, offset.min(source.len()));
            assert_eq!(roundtrip, clamped);
        }

        let line = cursor.next_usize(128) as u32;
        let character = cursor.next_usize(1_024) as u32;
        let _ = tooling::utf16_position_to_byte_offset(&source, line, character);
    }
});
