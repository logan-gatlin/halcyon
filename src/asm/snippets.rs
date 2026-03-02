use crate::ir::{
    Path,
    ScopeKind,
};

use super::{
    Encoder,
    Instruction,
    NumberOperation,
    Type,
};

use Instruction as i;

pub(crate) fn emit_array_concat(
    encoder: &mut Encoder<'_>,
    left: &Path,
    right: &Path,
    inner_type: Type,
) {
    let left_len = encoder.temporary_name("left_len");
    let right_len = encoder.temporary_name("right_len");
    let result = encoder.temporary_name("concat");
    let array_type = Type::Array(inner_type.clone().into());

    encoder.new_register(left_len.clone(), ScopeKind::Local, Type::I32);
    encoder.new_register(right_len.clone(), ScopeKind::Local, Type::I32);
    encoder.new_register(result.clone(), ScopeKind::Local, array_type.clone());

    encoder.extend([
        i::Get(left.clone()),
        i::ArrayLen,
        i::Set(left_len.clone()),
        i::Get(right.clone()),
        i::ArrayLen,
        i::Set(right_len.clone()),
        i::Get(left_len.clone()),
        i::Get(right_len.clone()),
        i::I32Op(NumberOperation::Add),
        i::ArrayNewDefault(inner_type.clone()),
        i::Set(result.clone()),
    ]);

    encoder.extend([
        i::Get(result.clone()),
        i::I32Const(0),
        i::Get(left.clone()),
        i::I32Const(0),
        i::Get(left_len.clone()),
        i::ArrayCopy {
            dst_type: inner_type.clone(),
            src_type: inner_type.clone(),
        },
    ]);

    encoder.extend([
        i::Get(result.clone()),
        i::Get(left_len),
        i::Get(right.clone()),
        i::I32Const(0),
        i::Get(right_len),
        i::ArrayCopy {
            dst_type: inner_type.clone(),
            src_type: inner_type,
        },
    ]);

    encoder.push(i::Get(result));
}
