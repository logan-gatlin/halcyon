use wasm_encoder::Instruction;

use super::*;

pub const INTEGER_MODULE_NAME: &str = "integer";
pub const REAL_MODULE_NAME: &str = "real";

fn one_param(
    enc: &mut FunctionEncoder,
    interface: &mut ModuleInterface,
    path: Path,
    parameter: Type,
    returns: Type,
    f: impl Fn(&mut FunctionEncoder) + 'static,
) {
    let p1 = Path::from("a");
    let type_ = Type::func(parameter.clone(), returns.clone());
    enc.encode(type_.clone());
    enc.module_encoder.new_global(&path, &type_);
    interface.values.insert(path.clone(), type_.clone());
    enc.encode(curry_function([(p1.clone(), parameter)], returns, f))
        .set_symbol(&path);
}

pub fn compile_numbers(
    enc: &mut FunctionEncoder,
    integer_interface: &mut ModuleInterface,
    real_interface: &mut ModuleInterface,
) {
    let integer = Path::from(INTEGER_MODULE_NAME);
    let real = Path::from(REAL_MODULE_NAME);
    let p1 = "a";
    let integer_type = enc.module_encoder.type_id(&Type::Integer);
    let real_type = enc.module_encoder.type_id(&Type::Real);
    // integer::from_real
    one_param(
        enc,
        integer_interface,
        integer.child("from_real"),
        Type::Real,
        Type::Integer,
        move |e| {
            e.get_symbol(&Path::from(p1)).encode([
                StructGet {
                    struct_type_index: real_type,
                    field_index: 0,
                },
                I64TruncF64S,
                StructNew(integer_type),
            ]);
        },
    );

    // real::from_integer
    one_param(
        enc,
        real_interface,
        real.child("from_integer"),
        Type::Integer,
        Type::Real,
        move |e| {
            e.get_symbol(&Path::from(p1)).encode([
                StructGet {
                    struct_type_index: integer_type,
                    field_index: 0,
                },
                F64ConvertI64S,
                StructNew(real_type),
            ]);
        },
    );
    // real::truncate
    one_param(
        enc,
        real_interface,
        real.child("truncate"),
        Type::Real,
        Type::Real,
        move |e| {
            e.get_symbol(&Path::from(p1)).encode([
                StructGet {
                    struct_type_index: real_type,
                    field_index: 0,
                },
                F64Trunc,
                StructNew(real_type),
            ]);
        },
    );

    // real::sqrt
    one_param(
        enc,
        real_interface,
        real.child("sqrt"),
        Type::Real,
        Type::Real,
        move |e| {
            e.get_symbol(&Path::from(p1)).encode([
                StructGet {
                    struct_type_index: real_type,
                    field_index: 0,
                },
                F64Sqrt,
                StructNew(real_type),
            ]);
        },
    );
    // real::round
    one_param(
        enc,
        real_interface,
        real.child("round"),
        Type::Real,
        Type::Real,
        move |e| {
            e.get_symbol(&Path::from(p1)).encode([
                StructGet {
                    struct_type_index: real_type,
                    field_index: 0,
                },
                F64Nearest,
                StructNew(real_type),
            ]);
        },
    );
    // real::pi
    let path = real.child("pi");
    constant(enc, path, Type::Real, F64Const(std::f64::consts::PI.into()));
    // real::e
    let path = real.child("e");
    constant(enc, path, Type::Real, F64Const(std::f64::consts::E.into()));
}

fn constant(enc: &mut FunctionEncoder, path: Path, type_: Type, value: Instruction<'static>) {
    let type_id = enc.module_encoder.type_id(&type_);
    enc.module_encoder.new_global(&path, &type_);
    enc.encode([value, StructNew(type_id)]).set_symbol(&path);
}
