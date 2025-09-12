use crate::ir::ConstValue;

use super::*;

pub const ARRAY_MODULE_NAME: &str = "array";

pub fn compile_array(enc: &mut FunctionEncoder, interface: &mut ModuleInterface) {
    enc.encode(Type::Array(Type::Variable(0).into()));
    let array = Path::from(ARRAY_MODULE_NAME);
    let integer_type = enc.module_encoder.type_id(&Type::Integer);
    let array_type = enc
        .module_encoder
        .type_id(&Type::Array(Type::Variable(0).into()));

    // array::empty
    {
        let path = array.child("empty");
        let type_ = Type::Array(Type::Variable(0).into());
        enc.module_encoder.new_global(&path, &type_);
        enc.encode(ArrayNewFixed {
            array_type_index: array_type,
            array_size: 0,
        })
        .set_symbol(&path);
        interface.values.insert(path, type_);
    }
    // array::length
    one_param(
        enc,
        interface,
        array.child("length"),
        Type::Array(Type::Variable(0).into()),
        Type::Integer,
        move |e| {
            e.get_symbol(&p(0))
                .encode([ArrayLen, I64ExtendI32U, StructNew(integer_type)]);
        },
    );
    // array::get
    n_params(
        enc,
        interface,
        array.child("get"),
        [Type::Integer, Type::Array(Type::Variable(0).into())],
        Type::Variable(0),
        move |e| {
            e.get_symbol(&p(1)).get_symbol(&p(0)).encode([
                StructGet {
                    struct_type_index: integer_type,
                    field_index: 0,
                },
                I32WrapI64,
                ArrayGet(array_type),
            ]);
        },
    );
    // array::push
    n_params(
        enc,
        interface,
        array.child("push"),
        [Type::Variable(0), Type::Array(Type::Variable(0).into())],
        Type::Array(Type::Variable(0).into()),
        move |e| {
            let array_temp = e.new_temporary(&Type::Array(Type::Variable(0).into()));
            e.get_symbol(&p(1))
                .encode([ArrayLen, I32Const(1), I32Add])
                .new_array(Type::Variable(0))
                .encode([LocalTee(array_temp), I32Const(0)])
                .get_symbol(&p(1))
                .array_copy_all()
                .encode(LocalGet(array_temp))
                .get_symbol(&p(1))
                .encode([ArrayLen])
                .get_symbol(&p(0))
                .encode([ArraySet(array_type), LocalGet(array_temp)]);
        },
    );

    // array::set
    n_params(
        enc,
        interface,
        array.child("set"),
        [
            Type::Integer,
            Type::Variable(0),
            Type::Array(Type::Variable(0).into()),
        ],
        Type::Array(Type::Variable(0).into()),
        move |e| {
            let array_type = e
                .module_encoder
                .type_id(&Type::Array(Type::Variable(0).into()));
            let integer_type = e.module_encoder.type_id(&Type::Integer);
            e.get_symbol(&p(2))
                .clone_array(Type::Variable(0))
                .set_symbol(&p(2))
                .get_symbol(&p(2))
                .get_symbol(&p(0))
                .encode([
                    StructGet {
                        struct_type_index: integer_type,
                        field_index: 0,
                    },
                    I32WrapI64,
                ])
                .get_symbol(&p(1))
                .encode(ArraySet(array_type))
                .get_symbol(&p(2));
        },
    );

    // array::concatenate
    n_params(
        enc,
        interface,
        array.child("concatenate"),
        [
            Type::Array(Type::Variable(0).into()),
            Type::Array(Type::Variable(0).into()),
        ],
        Type::Array(Type::Variable(0).into()),
        move |e| {
            let array_temp = e.new_temporary(&Type::Array(Type::Variable(0).into()));
            e.encode(ConstValue::Unit)
                .get_symbol(&p(0))
                .encode(ArrayLen)
                .get_symbol(&p(1))
                .encode([ArrayLen, I32Add, ArrayNew(array_type), LocalTee(array_temp)])
                // Dst array
                // Dst offset
                .encode(I32Const(0))
                // Src array
                .get_symbol(&p(0))
                .encode(I32Const(0))
                .get_symbol(&p(0))
                .encode([
                    ArrayLen,
                    ArrayCopy {
                        array_type_index_dst: array_type,
                        array_type_index_src: array_type,
                    },
                    // Dst array
                    LocalGet(array_temp),
                ])
                // Dst offset
                .get_symbol(&p(0))
                .encode(ArrayLen)
                // Src array
                .get_symbol(&p(1))
                // Src offset
                .encode(I32Const(0))
                .get_symbol(&p(1))
                // length
                .encode([
                    ArrayLen,
                    ArrayCopy {
                        array_type_index_dst: array_type,
                        array_type_index_src: array_type,
                    },
                    LocalGet(array_temp),
                ]);
        },
    );

    // array::map
    n_params(
        enc,
        interface,
        array.child("map"),
        [
            Type::func(Type::Variable(0), Type::Variable(1)),
            Type::Array(Type::Variable(0).into()),
        ],
        Type::Array(Type::Variable(1).into()),
        move |e| {
            let array_temp = e.new_temporary(&Type::Array(Type::Variable(1).into()));
            let length = e.new_raw_temporary(ValType::I32);
            let index = e.new_raw_temporary(ValType::I32);
            e.encode(ConstValue::Unit)
                .get_symbol(&p(1))
                .encode([ArrayLen, ArrayNew(array_type), LocalSet(array_temp)])
                .get_symbol(&p(1))
                .encode([
                    ArrayLen,
                    LocalSet(length),
                    I32Const(0),
                    LocalSet(index),
                    // while index < length
                    Block(BlockType::Empty),
                    Loop(BlockType::Empty),
                    LocalGet(index),
                    LocalGet(length),
                    I32GeU,
                    BrIf(1), // break
                    End,
                    LocalGet(array_temp),
                    LocalGet(index),
                ])
                .get_symbol(&p(1))
                .encode([LocalGet(index), ArrayGet(array_type)])
                .get_symbol(&p(0))
                .call_function(Type::Variable(0), Type::Variable(1))
                .encode([
                    ArraySet(array_type),
                    LocalGet(index),
                    I32Const(1),
                    I32Add,
                    LocalSet(index),
                    Br(0),
                    End,
                    LocalGet(array_temp),
                ]);
        },
    )
}
