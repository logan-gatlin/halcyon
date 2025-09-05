use wasm_encoder::MemArg;

use crate::ir::ConstValue;

use super::*;

pub const STRING_MODULE_NAME: &str = "string";

pub fn compile_string(enc: &mut FunctionEncoder, interface: &mut ModuleInterface) {
    let string = Path::from(STRING_MODULE_NAME);
    let p1 = Path::from("a");
    let p2 = Path::from("b");
    // Length
    {
        let p1 = p1.clone();
        let path = string.child("length");
        let type_ = Type::func(Type::String, Type::Integer);
        let integer_type = enc.module_encoder.type_id(&Type::Integer);
        enc.encode(type_.clone());
        enc.module_encoder.new_global(&path, &type_);
        interface.values.insert(path.clone(), type_.clone());
        enc.encode(curry_function(
            [(p1.clone(), Type::String)],
            Type::Integer,
            move |e| {
                e.get_symbol(&p1)
                    .encode([ArrayLen, I64ExtendI32U, StructNew(integer_type)]);
            },
        ))
        .set_symbol(&path);
    }
    // Concatenate
    {
        let p1 = p1.clone();
        let p2 = p2.clone();
        let path = string.child("concatenate");
        let type_ = Type::curry(&[Type::String, Type::String], Type::String);
        let string_type = enc.module_encoder.type_id(&Type::String);
        enc.encode(type_.clone());
        enc.module_encoder.new_global(&path, &type_);
        interface.values.insert(path.clone(), type_.clone());
        enc.encode(curry_function(
            [(p1.clone(), Type::String), (p2.clone(), Type::String)],
            Type::String,
            move |e| {
                let temporary = e.new_temporary(&Type::String);
                e.get_symbol(&p1)
                    .encode(ArrayLen)
                    .get_symbol(&p2)
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
                    .get_symbol(&p1)
                    // Source offset
                    .encode(I32Const(0))
                    .get_symbol(&p1)
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
                    .get_symbol(&p1)
                    // Destination offset
                    .encode(ArrayLen)
                    // Source string
                    .get_symbol(&p2)
                    // Source offset
                    .encode(I32Const(0))
                    .get_symbol(&p2)
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
        ))
        .set_symbol(&path);
    }
    // Unsafe memory store
    {
        let p1 = p1.clone();
        let path = string.child("unsafe_store");
        let type_ = Type::curry(&[Type::String, Type::Integer], Type::Unit);
        let string_type = enc.module_encoder.type_id(&Type::String);
        enc.encode(type_.clone());
        enc.module_encoder.new_global(&path, &type_);
        interface.values.insert(path.clone(), type_.clone());
        enc.encode(curry_function(
            [(p1.clone(), Type::String), (p2.clone(), Type::Integer)],
            Type::Unit,
            move |e| {
                let index = e.new_raw_temporary(ValType::I32);
                let length = e.new_raw_temporary(ValType::I32);
                let integer_type = e.module_encoder.type_id(&Type::Integer);
                e.encode([I32Const(0), LocalSet(index)])
                    // let index = 0
                    .get_symbol(&p1)
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
                    .get_symbol(&p2)
                    .encode([
                        StructGet {
                            struct_type_index: integer_type,
                            field_index: 0,
                        },
                        I32WrapI64,
                        I32Add,
                    ])
                    .get_symbol(&p1)
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
        ))
        .set_symbol(&path);
    }
}
