use super::*;

fn pad(input: String, with: &'static str) -> String {
    input.lines().map(|l| format!("{with}{l}\n")).collect()
}

impl std::fmt::Display for SX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SX::Nil => write!(f, "()"),
            SX::Atom(a) => write!(f, "{a}"),
            SX::Expr(items) => match items.as_slice() {
                [] => write!(f, "()"),
                [item] => write!(f, "({item})"),
                _ => {
                    writeln!(f, "(")?;
                    for item in items {
                        let s = pad(format!("{item}"), "  ");
                        write!(f, "{s}")?;
                    }
                    write!(f, ")")
                }
            },
            SX::Field(name, sx) => write!(f, "{name}: {sx}"),
        }
    }
}
