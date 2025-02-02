use super::{AsmType, Wasm, WasmModule};

impl WasmModule {
  pub fn to_wat(&self) -> String {
    let block: String = self
      .0
      .iter()
      .map(|asm| format!("{}\n", asm.to_wat()))
      .collect();
    format!("(module\n{})", block)
  }
}

impl Wasm {
  const INDENT: &str = "  ";

  pub fn to_wat(&self) -> String {
    let kind = |g: &bool| if *g { "global" } else { "local" };
    let isfloat = |t: &AsmType| match t {
      AsmType::f32 | AsmType::f64 => "",
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
      Wasm::custom(s) => s.clone(),
      Wasm::import { ns1, ns2, object } => {
        format!("(import \"{ns1}\" \"{ns2}\" {})", object.to_wat())
      },
      Wasm::ifelse { then, else_ } => {
        format!("(if (then\n{})\n(else\n{}))", block(then), block(else_))
      },
      Wasm::block { name, body } => format!("(block {name}\n{})", block(body)),
      Wasm::loop_ { name, body } => format!("(loop {name}\n{})", block(body)),
      Wasm::reg {
        type_,
        ident,
        global,
        initial,
      } => {
        if let Some(initial) = initial {
          format!("({} {ident} {type_} ({}))", kind(global), initial.to_wat())
        } else {
          format!("({} {ident} {type_})", kind(global))
        }
      },
      Wasm::regset { ident, global } => {
        format!("({}.set {ident})", kind(global))
      },
      Wasm::regget { ident, global } => {
        format!("{}.get {ident}", kind(global))
      },
      Wasm::function {
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
      Wasm::branch(lp) => format!("br {lp}"),
      Wasm::call(func) => format!("call ${func}"),
      Wasm::constant(asm_type, val) => format!("{asm_type}.const {val}"),
      Wasm::add(asm_type) => format!("{asm_type}.add"),
      Wasm::subtract(asm_type) => format!("{asm_type}.sub"),
      Wasm::multiply(asm_type) => format!("{asm_type}.mul"),
      Wasm::divide(asm_type) => format!("{asm_type}.div"),
      Wasm::remainder(asm_type) => format!("{asm_type}.rem_s"),
      Wasm::and(asm_type) => format!("{asm_type}.and"),
      Wasm::or(asm_type) => format!("{asm_type}.or"),
      Wasm::xor(asm_type) => format!("{asm_type}.xor"),
      Wasm::equal(asm_type) => format!("{asm_type}.eq"),
      Wasm::unequal(asm_type) => format!("{asm_type}.ne"),
      Wasm::greater_s(asm_type) => {
        format!("{asm_type}.gt{}", isfloat(asm_type))
      },
      Wasm::lesser_s(asm_type) => format!("{asm_type}.lt{}", isfloat(asm_type)),
      Wasm::greaterequal_s(asm_type) => {
        format!("{asm_type}.ge{}", isfloat(asm_type))
      },
      Wasm::lesserequal_s(asm_type) => {
        format!("{asm_type}.le{}", isfloat(asm_type))
      },
      Wasm::greater_u(asm_type) => format!("{asm_type}.gt_u"),
      Wasm::lesser_u(asm_type) => format!("{asm_type}.lt_u"),
      Wasm::greaterequal_u(asm_type) => format!("{asm_type}.ge_u"),
      Wasm::lesserequal_u(asm_type) => format!("{asm_type}.le_u"),
      Wasm::negate(asm_type) => format!("{asm_type}.neg"),
      Wasm::memory { min, max } => {
        format!("(memory {min} {max})")
      },
      Wasm::data { offset, content } => {
        format!(
          "(data (i32.const {offset}) \"{}\")",
          data_to_string(content)
        )
      },
      Wasm::nop => "nop".into(),
      Wasm::trap => format!("unreachable"),
      Wasm::comment(msg) => format!(";; {}", msg),
      Wasm::drop => format!("drop"),
      Wasm::start(func) => format!("(start {func})"),
      Wasm::return_ => format!("return"),
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
