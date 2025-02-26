use super::{Wasm, WasmType};
use WasmType as AsmType;

impl Wasm {
  const INDENT: &str = "  ";

  pub fn to_wat(&self) -> String {
    let kind = |g: &bool| if *g { "global" } else { "local" };
    let isfloat = |t: &AsmType| match t {
      AsmType::F32 | AsmType::F64 => "",
      _ => "_s",
    };
    let block = |block: &Vec<Wasm>| -> String {
      let mut output = String::new();
      for a in block {
        let wat = format!("{}\n", a.to_wat());
        output.push_str(&wat);
      }
      output
        .lines()
        .map(|l| format!("{}{l}\n", Self::INDENT))
        .collect()
    };
    let s = match self {
      Wasm::Custom(s) => s.clone(),
      Wasm::Import { ns1, ns2, object } => {
        format!("(import \"{ns1}\" \"{ns2}\" {})", object.to_wat())
      },
      Wasm::If => {
        format!("if")
      },
      Wasm::Else => {
        format!("else")
      },
      Wasm::Block(name) => format!("block {name}"),
      Wasm::Loop(name) => format!("loop {name}",),
      Wasm::Local(type_, name) => format!("(local {name} {type_})"),
      Wasm::LocalGet(name) => format!("(local.get {name})"),
      Wasm::LocalSet(name) => format!("(local.set {name})"),
      Wasm::Function {
        ident,
        params,
        results,
        body,
      } => {
        let params: String = params
          .into_iter()
          .map(|(name, ty)| format!("\n{}(param {name} {ty})", Self::INDENT))
          .collect();
        let results: String = results
          .into_iter()
          .map(|r| format!("\n{}(result {r})", Self::INDENT))
          .collect();
        format!("(func {ident}{params}{results}\n{})", block(body))
      },
      Wasm::End => format!("end"),
      Wasm::Branch(lp) => format!("br {lp}"),
      Wasm::Call(func) => format!("call ${func}"),
      Wasm::Constant(val) => format!("{}.const {val}", val.type_of()),
      Wasm::Add(asm_type) => format!("{asm_type}.add"),
      Wasm::Subtract(asm_type) => format!("{asm_type}.sub"),
      Wasm::Multiply(asm_type) => format!("{asm_type}.mul"),
      Wasm::Divide(asm_type) => format!("{asm_type}.div"),
      Wasm::Remainder(asm_type) => format!("{asm_type}.rem_s"),
      Wasm::And(asm_type) => format!("{asm_type}.and"),
      Wasm::Or(asm_type) => format!("{asm_type}.or"),
      Wasm::Xor(asm_type) => format!("{asm_type}.xor"),
      Wasm::Equal(asm_type) => format!("{asm_type}.eq"),
      Wasm::Unequal(asm_type) => format!("{asm_type}.ne"),
      Wasm::GreaterSigned(asm_type) => {
        format!("{asm_type}.gt{}", isfloat(asm_type))
      },
      Wasm::LesserSigned(asm_type) => {
        format!("{asm_type}.lt{}", isfloat(asm_type))
      },
      Wasm::GreaterEqualSigned(asm_type) => {
        format!("{asm_type}.ge{}", isfloat(asm_type))
      },
      Wasm::LesserEqualSigned(asm_type) => {
        format!("{asm_type}.le{}", isfloat(asm_type))
      },
      Wasm::GreaterUnsigned(asm_type) => format!("{asm_type}.gt_u"),
      Wasm::LesserUnsigned(asm_type) => format!("{asm_type}.lt_u"),
      Wasm::GreaterEqualUnsigned(asm_type) => format!("{asm_type}.ge_u"),
      Wasm::LesserEqualUnsigned(asm_type) => format!("{asm_type}.le_u"),
      Wasm::Negate(asm_type) => format!("{asm_type}.neg"),
      Wasm::Memory { min, max } => {
        format!("(memory {min} {max})")
      },
      Wasm::Data { offset, content } => {
        format!(
          "(data (i32.const {offset}) \"{}\")",
          data_to_string(content)
        )
      },
      Wasm::Nop => "nop".into(),
      Wasm::Unreachable => format!("unreachable"),
      Wasm::Comment(msg) => format!(";; {}", msg),
      Wasm::Drop => format!("drop"),
      Wasm::Start(func) => format!("(start {func})"),
      Wasm::Return => format!("return"),
    };
    s
  }
}

fn data_to_string(buf: &[u8]) -> String {
  let mut output = Vec::with_capacity(buf.len());
  for byte in buf {
    let byte = *byte;
    if byte.is_ascii()
      && !byte.is_ascii_control()
      && byte != b'"'
      && byte != b'\\'
    {
      output.push(byte);
    } else {
      output.extend(format!("\\{byte:02x}").as_bytes());
    }
  }
  String::from_utf8(output).unwrap()
}
