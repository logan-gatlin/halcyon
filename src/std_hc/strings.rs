use wasm_encoder::MemArg;

use crate::ir::ConstValue;

use super::*;

pub const STRING_MODULE_NAME: &str = "string";

pub fn compile_string(enc: &mut FunctionEncoder, interface: &mut ModuleInterface) {
    let string = Path::from(STRING_MODULE_NAME);
    let p1 = "a";
    let p2 = "b";
    let integer_type = enc.module_encoder.type_id(&Type::Integer);
    let string_type = enc.module_encoder.type_id(&Type::String);
    let _glyph_type = enc.module_encoder.type_id(&Type::Glyph);
    // string::length
    one_param(
        enc,
        interface,
        string.child("length"),
        Type::String,
        Type::Integer,
        move |e| {
            e.get_symbol(&Path::from("a")).encode([
                ArrayLen,
                I64ExtendI32U,
                StructNew(integer_type),
            ]);
        },
    );
    // string::concatenate
    n_params(
        enc,
        interface,
        string.child("concatenate"),
        [Type::String, Type::String],
        Type::String,
        move |e| {
            let temporary = e.new_temporary(&Type::String);
            e.get_symbol(&Path::from(p1))
                .encode(ArrayLen)
                .get_symbol(&Path::from(p2))
                .encode([
                    ArrayLen,
                    I32Add,
                    // First copy
                    // Destination string
                    ArrayNewDefault(string_type),
                    LocalTee(temporary),
                    // Destination offset
                    I32Const(0),
                ])
                // Source string
                .get_symbol(&Path::from(p1))
                // Source offset
                .encode(I32Const(0))
                .get_symbol(&Path::from(p1))
                .encode([
                    // Length
                    ArrayLen,
                    ArrayCopy {
                        array_type_index_dst: string_type,
                        array_type_index_src: string_type,
                    },
                    // Second copy
                    // Destination string
                    LocalGet(temporary),
                ])
                .get_symbol(&Path::from(p1))
                // Destination offset
                .encode(ArrayLen)
                // Source string
                .get_symbol(&Path::from(p2)) // Source offset
                .encode(I32Const(0))
                .get_symbol(&Path::from(p2))
                .encode([
                    // Length
                    ArrayLen,
                    ArrayCopy {
                        array_type_index_dst: string_type,
                        array_type_index_src: string_type,
                    },
                    LocalGet(temporary),
                ]);
        },
    );
    // string::unsafe_store
    n_params(
        enc,
        interface,
        string.child("unsafe_store"),
        [Type::String, Type::Integer],
        Type::Unit,
        move |e| {
            let index = e.new_raw_temporary(ValType::I32);
            let length = e.new_raw_temporary(ValType::I32);
            let integer_type = e.module_encoder.type_id(&Type::Integer);
            e.encode([I32Const(0), LocalSet(index)])
                // let index = 0
                .get_symbol(&Path::from(p1))
                .encode([
                    ArrayLen,
                    LocalSet(length),
                    // let length = string::length a
                    Loop(BlockType::Empty),
                    LocalGet(index),
                    LocalGet(length),
                    I32LtU,
                    // if index < length
                    If(BlockType::Empty),
                    LocalGet(index),
                ])
                .get_symbol(&Path::from(p2))
                .encode([
                    StructGet {
                        struct_type_index: integer_type,
                        field_index: 0,
                    },
                    I32WrapI64,
                    I32Add,
                ])
                .get_symbol(&Path::from(p1))
                .encode([
                    LocalGet(index),
                    ArrayGetU(string_type),
                    I32Store8(MemArg {
                        offset: 0,
                        align: 0,
                        memory_index: 0,
                    }),
                    // *ptr = a[index]
                    LocalGet(index),
                    I32Const(1),
                    I32Add,
                    LocalSet(index),
                    // index += 1
                    Br(1),
                    // continue
                ])
                .encode([End, End])
                .encode(ConstValue::Unit);
        },
    );
}
