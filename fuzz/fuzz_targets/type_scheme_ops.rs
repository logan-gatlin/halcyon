#![no_main]

mod common;

use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;

use halcyon_lib::Span;
use halcyon_lib::types::Type;
use halcyon_lib::types::infer::InferenceContext;

use common::{
    arbitrary_predicates,
    arbitrary_type,
};

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let mut ctx = InferenceContext::new();

    let meta_count = u.int_in_range(0..=10).unwrap_or(0);
    let mut metas = Vec::with_capacity(meta_count);
    for _ in 0..meta_count {
        metas.push(ctx.fresh_meta());
    }

    let type_depth = u.int_in_range(0..=5).unwrap_or(0);
    let Ok(base_type) = arbitrary_type(&mut u, type_depth, &metas) else {
        return;
    };

    let predicates = arbitrary_predicates(&mut u, 8, 3, &metas).unwrap_or_default();
    let level = u.int_in_range(0..=6).unwrap_or(0);

    let mut scheme = ctx.generalize_with_predicates(&base_type, level, predicates);
    let _ = ctx.instantiate(&scheme, Span::Generated);
    let _ = ctx.instantiate_scheme(&scheme, Span::Generated);

    let rounds = u.int_in_range(0..=96).unwrap_or(0);
    for _ in 0..rounds {
        match u.int_in_range(0..=3).unwrap_or(0) {
            0 => {
                if let Ok(instance) = ctx.instantiate_scheme(&scheme, Span::Generated) {
                    scheme =
                        ctx.generalize_with_predicates(&instance.type_, level, instance.predicates);
                }
            }
            1 => {
                let depth = u.int_in_range(0..=5).unwrap_or(0);
                if let Ok(other) = arbitrary_type(&mut u, depth, &metas) {
                    let _ = ctx.table_mut().unify(&scheme.type_, &other);
                    scheme = ctx.generalize_at(&other, level);
                }
            }
            2 => {
                let new_predicates = arbitrary_predicates(&mut u, 8, 3, &metas).unwrap_or_default();
                scheme = ctx.generalize_with_predicates(&scheme.type_, level, new_predicates);
            }
            _ => {
                let depth = u.int_in_range(0..=5).unwrap_or(0);
                if let Ok(extra) = arbitrary_type(&mut u, depth, &metas) {
                    let joined = Type::Tuple(vec![scheme.type_.clone(), extra]);
                    scheme = ctx.generalize_at(&joined, level);
                }
            }
        }
    }

    let _ = ctx.instantiate_scheme(&scheme, Span::Generated);
});
