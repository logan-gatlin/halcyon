#![no_main]

mod common;

use libfuzzer_sys::fuzz_target;

use halcyon_lib::linking;

use common::ByteCursor;

fuzz_target!(|data: &[u8]| {
    let mut cursor = ByteCursor::new(data);
    let binary_count = cursor.next_usize(8);
    let mut binaries = Vec::with_capacity(binary_count.max(1));
    for _ in 0..binary_count {
        let len = cursor.next_usize(8_192);
        binaries.push(cursor.take(len).to_vec());
    }

    if binaries.is_empty() {
        binaries.push(data.to_vec());
    }

    let _ = linking::link_binaries(
        &binaries,
        linking::LinkOptions {
            module_name: "fuzz-linked".to_string(),
            strict: cursor.next_bool(),
            emit_source_map: false,
            emit_dwarf: false,
            ..Default::default()
        },
    );
});
