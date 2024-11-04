use crate::{
  Base, BinaryOp, Immediate,
  err::*,
  semantic::{Primitive, Type},
};

use super::{Compiler, IR};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum RegisterType {
  f32,
  f64,
  i32,
  i64,
}

impl std::fmt::Display for RegisterType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", match self {
      RegisterType::f32 => "f32",
      RegisterType::f64 => "f64",
      RegisterType::i32 => "i32",
      RegisterType::i64 => "i64",
    })
  }
}

const SYS_INT: RegisterType = RegisterType::i32;
const SYS_REAL: RegisterType = RegisterType::f32;

fn convert_sys_whole(input: &str, base: Base) -> Option<String> {
  u32::from_str_radix(input, base as u32)
    .ok()
    .map(|i| format!("{i}"))
}

fn convert_sys_int(input: &str, base: Base) -> Option<String> {
  i32::from_str_radix(input, base as u32)
    .ok()
    .map(|i| format!("{i}"))
}

fn convert_sys_real(input: &str) -> Option<String> {
  input.parse::<f32>().ok().map(|f| format!("{f}"))
}

impl Compiler {
  pub fn type_prim(&self, prim: Primitive) -> RegisterType {
    use Primitive as p;
    use RegisterType as r;
    match prim {
      p::i8 | p::i16 | p::i32 | p::w8 | p::w16 | p::w32 => r::i32,
      p::i64 | p::w64 => r::i64,
      p::integer | p::whole => SYS_INT,
      p::r32 => r::f32,
      p::r64 => r::f64,
      p::real => SYS_REAL,
      p::boolean => r::i32,
      p::string => r::i64,
      p::glyph => r::i32,
      p::integer_ambiguous | p::real_ambiguous => unreachable!(),
    }
  }

  pub fn splat(&self, type_: &Type) -> Vec<RegisterType> {
    match type_ {
      Type::Ambiguous => unreachable!(),
      Type::Function(_) | Type::Nothing | Type::Alias(_) => vec![],
      Type::Prim(prim) => vec![self.type_prim(*prim)],
      Type::Struct(sid) => {
        let struct_def = &self.table.structs[*sid].0;
        let mut buf = vec![];
        for (_, type_) in struct_def {
          let type_ =
            self.table.resolve_type(type_).unwrap().is_alias().unwrap();
          buf.append(&mut self.splat(&type_));
        }
        buf
      },
    }
  }

  pub fn ir_to_wat(&self, ir: IR) -> Result<String> {
    use Immediate as i;
    use Primitive as p;
    Ok(match ir {
      IR::Push { value, prim } => match value {
        i::Integer(ref i, base) => {
          let b = base as u32;
          // Unfortunately this can't be simplified
          let s = match prim {
            p::w8 => u8::from_str_radix(i, b).ok().map(|s| format!("{s}")),
            p::w16 => u16::from_str_radix(i, b).ok().map(|s| format!("{s}")),
            p::w32 => u32::from_str_radix(i, b).ok().map(|s| format!("{s}")),
            p::w64 => u64::from_str_radix(i, b).ok().map(|s| format!("{s}")),
            p::i8 => i8::from_str_radix(i, b).ok().map(|s| format!("{s}")),
            p::i16 => i16::from_str_radix(i, b).ok().map(|s| format!("{s}")),
            p::i32 => i32::from_str_radix(i, b).ok().map(|s| format!("{s}")),
            p::i64 => i64::from_str_radix(i, b).ok().map(|s| format!("{s}")),
            p::whole => convert_sys_whole(i, base),
            p::integer => convert_sys_int(i, base),
            _ => unreachable!(),
          }
          .reason(format!("Cannot parse immediate value as '{}'", prim))?;
          format!("{}.const {}\n", self.type_prim(prim), s)
        },
        i::Real(ref i) => {
          let s = match prim {
            p::r32 => i.parse::<f32>().ok().map(|f| format!("{f}")),
            p::r64 => i.parse::<f64>().ok().map(|f| format!("{f}")),
            p::real => convert_sys_real(i),
            _ => unreachable!(),
          }
          .reason(format!("Cannot parse immediate value as '{}'", prim))?;
          format!("{}.const {}\n", self.type_prim(prim), s)
        },
        i::String(_) => todo!(),
        i::Glyph(c) => format!("i32.const {}\n", c as u32),
        i::Boolean(b) => format!("i32.const {}\n", b as i8),
      },
      IR::Drop { type_ } => {
        let mut buffer = String::new();
        for _ in 0..self.splat(&type_).len() {
          buffer.push_str("drop\n");
        }
        buffer
      },
      IR::New {
        uid,
        type_,
        mutable,
        global,
      } => {
        let mut buffer = String::new();
        for (index, rt) in self.splat(&type_).iter().enumerate() {
          buffer.push_str(&format!(
            "({} {uid}${index} {})\n",
            if global { "global" } else { "local" },
            if global && mutable {
              format!("(mut {rt})")
            } else {
              format!("{rt}")
            }
          ))
        }
        buffer
      },
      IR::Set { uid, type_, global } => {
        let mut buffer = String::new();
        for index in 0..self.splat(&type_).len() {
          buffer.push_str(&format!(
            "{}.set {uid}${index}\n",
            if global { "global" } else { "local" }
          ))
        }
        buffer
      },
      IR::Get { uid, type_, global } => {
        let mut buffer = String::new();
        for index in (0..self.splat(&type_).len()).rev() {
          buffer.push_str(&format!(
            "{}.get {uid}${index}\n",
            if global { "global" } else { "local" }
          ))
        }
        buffer
      },
      IR::StartFunc {
        uid,
        params,
        returns,
      } => {
        let mut buffer = format!("(func {uid}\n");
        for (puid, type_) in params {
          for (id, rt) in self.splat(&type_).iter().enumerate() {
            buffer.push_str(&format!("(param {puid}${id} {rt})\n"));
          }
        }
        let returns = self.splat(&returns);
        if returns.len() > 0 {
          buffer.push_str("(result ");
          for rt in returns {
            buffer.push_str(&format!("{rt} "))
          }
          buffer.push_str(")\n");
        }
        buffer
      },
      IR::EndFunc => ")\n".into(),
      IR::Return { type_ } => "return\n".into(),
      IR::Call { uid } => format!("call {uid}\n"),
      IR::BinOp { op, type_ } => {
        use BinaryOp::*;
        let p = if let Type::Prim(p) = type_ {
          p
        } else {
          unreachable!()
        };
        match (op, p) {
          (Plus, _) => format!("{}.add\n", self.type_prim(p)),
          (Minus, _) => format!("{}.sub\n", self.type_prim(p)),
          (Star, _) => format!("{}.mul\n", self.type_prim(p)),
          _ => todo!(),
        }
      },
      IR::UnOp { op, type_ } => todo!(),
      IR::Print { type_ } => todo!(),
    })
  }
}
