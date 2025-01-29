use super::{Asm, AsmType};

impl Asm {
  const INDENT: &str = "  ";

  pub fn to_wat(&self, indent: usize) -> String {
    let kind = |g: &bool| if *g { "global" } else { "local" };
    let isfloat = |t: &AsmType| match t {
      AsmType::f32 | AsmType::f64 => "_s",
      _ => "",
    };
    let block = |block: &Vec<Asm>| -> String {
      let mut output = String::new();
      for a in block {
        let wat = format!("{}\n", a.to_wat(indent + 1));
        output.push_str(&wat);
      }
      output
        .lines()
        .map(|l| format!("{}{l}\n", Self::INDENT))
        .collect()
    };
    let s = match self {
      Asm::module(vec) => format!("(module\n{}\n)", block(vec)),
      Asm::ifelse { then, else_ } => {
        format!("(if (then\n{})\n(else\n{}))", block(then), block(else_))
      }
      Asm::block { name, body } => format!("(block {name}\n{})", block(body)),
      Asm::loop_ { name, body } => format!("(loop {name}\n{})", block(body)),
      Asm::reg {
        type_,
        ident,
        global,
      } => format!("({} {ident} {type_})", kind(global)),
      Asm::regset { ident, global } => {
        format!("({}.set {ident})", kind(global))
      }
      Asm::regget { ident, global } => {
        format!("{}.get {ident}", kind(global))
      }
      Asm::function {
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
        format!("(func ${ident}{params}{results}\n{})", block(body))
      }
      Asm::branch(lp) => format!("br {lp}"),
      Asm::call(func) => format!("call ${func}"),
      Asm::constant(asm_type, val) => format!("{asm_type}.const {val}"),
      Asm::add(asm_type) => format!("{asm_type}.add"),
      Asm::subtract(asm_type) => format!("{asm_type}.sub"),
      Asm::multiply(asm_type) => format!("{asm_type}.mul"),
      Asm::divide(asm_type) => format!("{asm_type}.div"),
      Asm::remainder(asm_type) => format!("{asm_type}.rem_s"),
      Asm::and(asm_type) => format!("{asm_type}.and"),
      Asm::or(asm_type) => format!("{asm_type}.or"),
      Asm::xor(asm_type) => format!("{asm_type}.xor"),
      Asm::equal(asm_type) => format!("{asm_type}.eq"),
      Asm::unequal(asm_type) => format!("{asm_type}.ne"),
      Asm::greater_s(asm_type) => {
        format!("{asm_type}.gt{}", isfloat(asm_type))
      }
      Asm::lesser_s(asm_type) => format!("{asm_type}.lt{}", isfloat(asm_type)),
      Asm::greaterequal_s(asm_type) => {
        format!("{asm_type}.ge{}", isfloat(asm_type))
      }
      Asm::lesserequal_s(asm_type) => {
        format!("{asm_type}.le{}", isfloat(asm_type))
      }
      Asm::greater_u(asm_type) => format!("{asm_type}.gt_u"),
      Asm::lesser_u(asm_type) => format!("{asm_type}.lt_u"),
      Asm::greaterequal_u(asm_type) => format!("{asm_type}.ge_u"),
      Asm::lesserequal_u(asm_type) => format!("{asm_type}.le_u"),
      Asm::negate(asm_type) => format!("{asm_type}.neg"),
      Asm::nop => "nop".into(),
      Asm::trap => format!("unreachable"),
      Asm::comment(msg) => format!(";; {}", msg),
    };
    s
  }
}
