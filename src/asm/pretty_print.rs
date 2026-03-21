#![allow(dead_code)]
use super::*;
use crate::ir::ImmediateValue;

use colored::Colorize;

trait PrettyPrint {
    /// Handles pretty.
    fn pretty(&self) -> String;
}

/// Handles left pad.
fn left_pad(text: impl AsRef<str>) -> String {
    text.as_ref()
        .lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

impl PrettyPrint for Path {
    /// Handles pretty.
    fn pretty(&self) -> String {
        self.to_string()
    }
}

impl PrettyPrint for ImmediateValue {
    /// Handles pretty.
    fn pretty(&self) -> String {
        self.to_string()
    }
}

impl PrettyPrint for Module {
    /// Handles pretty.
    fn pretty(&self) -> String {
        self.functions
            .iter()
            .map(|(id, f)| {
                let parameters = if f.parameters.is_empty() {
                    "".to_string()
                } else {
                    format!(
                        "\n(params\n{}\n)",
                        left_pad(
                            f.parameters
                                .iter()
                                .map(|(path, type_)| {
                                    format!("{} [{}]", path.pretty(), type_.pretty())
                                })
                                .collect::<Vec<_>>()
                                .join("\n"),
                        )
                    )
                };
                let locals = if f.variables.is_empty() {
                    "".to_string()
                } else {
                    format!(
                        "\n(locals\n{}\n)",
                        left_pad(
                            f.variables
                                .iter()
                                .map(|(path, type_)| {
                                    format!("{} [{}]", path.pretty(), type_.pretty())
                                })
                                .collect::<Vec<_>>()
                                .join("\n"),
                        )
                    )
                };
                format!(
                    "(fn #{id}{parameters}{locals}\n{ops}\n)",
                    parameters = left_pad(parameters),
                    locals = left_pad(locals),
                    ops = left_pad(
                        f.ops
                            .iter()
                            .map(PrettyPrint::pretty)
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl PrettyPrint for Type {
    /// Handles pretty.
    fn pretty(&self) -> String {
        use Type::*;
        match self {
            Any | I8 | I16 | I32 | I64 | F32 | F64 => {
                format!("{self:?}")
                    .to_lowercase()
                    .italic()
                    .blue()
                    .to_string()
            }
            Struct(items) => {
                format!(
                    "{struct} [{items}]",
                    struct="struct".italic().blue(),
                    items=items.iter().map(PrettyPrint::pretty).collect::<Vec<_>>().join(", ")
                )
            }
            Array(type_) => {
                format!(
                    "{array} [{items}]",
                    array = "Array".italic().blue(),
                    items = type_.pretty(),
                )
            }
            Function { .. } => "Fn".italic().blue().to_string(),
        }
    }
}

impl PrettyPrint for NumberOperation {
    /// Handles pretty.
    fn pretty(&self) -> String {
        format!("{self:?}").to_lowercase()
    }
}

impl PrettyPrint for Instruction {
    /// Handles pretty.
    fn pretty(&self) -> String {
        use Instruction::*;
        match self {
            Set(path) => format!("set {}", path.pretty()),
            Get(path) => format!("get {}", path.pretty()),
            Const(const_value) => format!("const {}", const_value.pretty()),
            I32Const(i) => format!("i32.const {i}"),
            F32Const(f) => format!("f32.const {f}"),
            Func(path) => format!("func {}", path.pretty()),
            StructNew(items) => {
                format!(
                    "struct.new [{}]",
                    items
                        .iter()
                        .map(PrettyPrint::pretty)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            StructGet(types, index) => {
                format!(
                    "struct.get [{}] {index}",
                    types
                        .iter()
                        .map(PrettyPrint::pretty)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            ArrayGet(t) => format!("array.get [{}]", t.pretty()),
            ArrayNewFixed { inner_type, length } => {
                format!("array.new_fixed [{}] {length}", inner_type.pretty())
            }
            CallRef {
                parameters,
                returns,
            } => {
                format!(
                    "call [{}] -> [{}]",
                    parameters
                        .iter()
                        .map(PrettyPrint::pretty)
                        .collect::<Vec<_>>()
                        .join(", "),
                    returns
                        .iter()
                        .map(PrettyPrint::pretty)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Unreachable => "unreachable".into(),
            Drop => "drop".into(),
            If(t) => {
                format!(
                    "if [{}]",
                    t.as_ref()
                        .map(PrettyPrint::pretty)
                        .unwrap_or_else(String::new)
                )
            }
            Else => "else".into(),
            End => "end".into(),
            Loop => "loop".into(),
            Block(t) => {
                format!(
                    "block [{}]",
                    t.as_ref()
                        .map(PrettyPrint::pretty)
                        .unwrap_or_else(String::new)
                )
            }
            Break(level) => format!("break {level}"),
            BreakIf(level) => format!("break.if {level}"),
            I32Op(num) => format!("i32.{}", num.pretty()),
            I64Op(num) => format!("i64.{}", num.pretty()),
            F32Op(num) => format!("f32.{}", num.pretty()),
            F64Op(num) => format!("f64.{}", num.pretty()),
            ArrayNewDefault(t) => format!("array.new_default [{}]", t.pretty()),
            ArrayLen => "array.len".into(),
            ArrayCopy { dst_type, src_type } => {
                format!("array.copy [{}] [{}]", dst_type.pretty(), src_type.pretty())
            }
            RefCastFunc {
                parameters,
                returns,
            } => {
                format!(
                    "ref.cast.func [{}] -> [{}]",
                    parameters
                        .iter()
                        .map(PrettyPrint::pretty)
                        .collect::<Vec<_>>()
                        .join(", "),
                    returns
                        .iter()
                        .map(PrettyPrint::pretty)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            RefCastStruct(fields) => {
                format!(
                    "ref.cast.struct [{}]",
                    fields
                        .iter()
                        .map(PrettyPrint::pretty)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            RefCastArray(inner) => {
                format!("ref.cast.array [{}]", inner.pretty())
            }
            I32Store8 => "i32.store8".into(),
            I32Load => "i32.load".into(),
            I32Store => "i32.store".into(),
            I64Load => "i64.load".into(),
            I64ExtendI32U => "i64.extend_i32_u".into(),
            I32WrapI64 => "i32.wrap_i64".into(),
            I32TruncF32S => "i32.trunc_f32_s".into(),
            I32TruncF32U => "i32.trunc_f32_u".into(),
            I32TruncF64S => "i32.trunc_f64_s".into(),
            I32TruncF64U => "i32.trunc_f64_u".into(),
            I64TruncF32S => "i64.trunc_f32_s".into(),
            I64TruncF32U => "i64.trunc_f32_u".into(),
            I64TruncF64S => "i64.trunc_f64_s".into(),
            I64TruncF64U => "i64.trunc_f64_u".into(),
            F32DemoteF64 => "f32.demote_f64".into(),
            F64ConvertI64S => "f64.convert_i64_s".into(),
            F64ConvertI64U => "f64.convert_i64_u".into(),
            Call(path) => format!("call {}", path.pretty()),
        }
    }
}
