#![no_main]

mod common;

use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;

use halcyon_lib::types::Type;
use halcyon_lib::types::unify::UnificationTable;

use common::arbitrary_type;

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let mut table = UnificationTable::default();

    let meta_count = u.int_in_range(0..=8).unwrap_or(0);
    let mut metas = Vec::with_capacity(meta_count);
    for _ in 0..meta_count {
        let level = u.int_in_range(0..=6).unwrap_or(0);
        metas.push(table.new_meta(level));
    }

    let type_count = u.int_in_range(0..=24).unwrap_or(0);
    let mut types = Vec::with_capacity(type_count.max(2));
    for _ in 0..type_count {
        let depth = u.int_in_range(0..=5).unwrap_or(0);
        let Ok(type_) = arbitrary_type(&mut u, depth, &metas) else {
            break;
        };
        types.push(type_);
    }
    if types.is_empty() {
        types.push(Type::Unit);
        types.push(Type::Integer);
    }

    let op_count = u.int_in_range(0..=160).unwrap_or(0);
    for _ in 0..op_count {
        let left_index = u.int_in_range(0..=types.len() - 1).unwrap_or(0);
        let right_index = u.int_in_range(0..=types.len() - 1).unwrap_or(0);
        let left = types[left_index].clone();
        let right = types[right_index].clone();
        match u.int_in_range(0..=5).unwrap_or(0) {
            0 => {
                let _ = table.unify(&left, &right);
            }
            1 => {
                let normalized = table.normalize(&left);
                types.push(normalized);
            }
            2 => {
                let _ = table.free_meta_vars(&left);
            }
            3 => {
                let left = table.normalize(&left);
                let right = table.normalize(&right);
                let _ = table.unify(&left, &right);
            }
            4 => {
                let depth = u.int_in_range(0..=5).unwrap_or(0);
                if let Ok(type_) = arbitrary_type(&mut u, depth, &metas) {
                    types.push(type_);
                }
            }
            _ => {
                let _ = table.unify(&right, &left);
            }
        }
        if types.len() > 128 {
            let _ = types.drain(0..(types.len() - 128));
        }
    }

    if types.len() >= 2 {
        let mut left_table = table.clone();
        let mut right_table = table;
        let _ = left_table.unify(&types[0], &types[1]);
        let _ = right_table.unify(&types[0], &types[1]);
        assert_eq!(
            left_table.normalize(&types[0]),
            right_table.normalize(&types[0])
        );
    }
});
