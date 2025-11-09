use wasm_encoder::MemArg;

use crate::ir::ConstValue;

use super::*;

#[allow(unused)]
pub fn compile_wasm(enc: &mut FunctionEncoder, interface: &mut ModuleInterface) {
    let integer_type = enc.module_encoder.type_id(&Type::Integer);
    let string_type = enc.module_encoder.type_id(&Type::String);
    let glyph_type = enc.module_encoder.type_id(&Type::Glyph);
    let boolean_type = enc.module_encoder.type_id(&Type::Boolean);
    let wasm = Path::from(WASM_MODULE_NAME);
    let mem = MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    };
    // wasm::store_integer
    [
        (I64Store(mem), I64Load(mem), None, "i64"),
        (
            I64Store32(mem),
            I64Load32U(mem),
            Some(I64Load32S(mem)),
            "i32",
        ),
        (
            I64Store16(mem),
            I64Load16U(mem),
            Some(I64Load16S(mem)),
            "i16",
        ),
        (I64Store8(mem), I64Load8U(mem), Some(I64Load8S(mem)), "i8"),
    ]
    .into_iter()
    .for_each(
        |(store, load_u, load_s, name): (_, _, Option<Instruction>, _)| {
            // Store
            n_params(
                enc,
                interface,
                wasm.child(format!("store_{name}")),
                [Type::Integer, Type::Integer],
                Type::Unit,
                move |e| {
                    e.get_symbol(&p(0))
                        .encode([
                            StructGet {
                                struct_type_index: integer_type,
                                field_index: 0,
                            },
                            I32WrapI64,
                        ])
                        .get_symbol(&p(1))
                        .encode([
                            StructGet {
                                struct_type_index: integer_type,
                                field_index: 0,
                            },
                            store.clone(),
                        ])
                        .encode(ConstValue::Unit);
                },
            );
            // Load signed
            one_param(
                enc,
                interface,
                wasm.child(format!("load_{name}",)),
                Type::Integer,
                Type::Integer,
                move |e| {
                    e.get_symbol(&p(0)).encode([
                        StructGet {
                            struct_type_index: integer_type,
                            field_index: 0,
                        },
                        I32WrapI64,
                        load_u.clone(),
                        StructNew(integer_type),
                    ]);
                },
            );
            // Load signed
            if let Some(load_s) = load_s {
                one_param(
                    enc,
                    interface,
                    wasm.child(format!("load_{name}_sx",)),
                    Type::Integer,
                    Type::Integer,
                    move |e| {
                        e.get_symbol(&p(0)).encode([
                            StructGet {
                                struct_type_index: integer_type,
                                field_index: 0,
                            },
                            I32WrapI64,
                            load_s.clone(),
                            StructNew(integer_type),
                        ]);
                    },
                );
            }
        },
    );
    // wasm::trap
    one_param(
        enc,
        interface,
        wasm.child("unreachable"),
        Type::Unit,
        Type::Variable(0),
        |e| {
            e.encode(Unreachable);
        },
    );
}
