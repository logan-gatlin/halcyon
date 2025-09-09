use super::*;

pub const ARRAY_MODULE_NAME: &str = "array";

pub fn compile_array(enc: &mut FunctionEncoder, interface: &mut ModuleInterface) {
    let array = Path::from(ARRAY_MODULE_NAME);
    let p1 = "a";
    let p2 = "b";
    let integer_type = enc.module_encoder.type_id(&Type::Integer);

    // array::length
    one_param(
        enc,
        interface,
        array.child("length"),
        Type::Array(Type::Variable(0).into()),
        Type::Integer,
        move |e| {
            e.get_symbol(&Path::from(p1)).encode([
                ArrayLen,
                I64ExtendI32U,
                StructNew(integer_type),
            ]);
        },
    );
}
