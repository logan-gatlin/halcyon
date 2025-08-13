#[derive(Clone)]
pub struct SExpression {
    this: String,
    children: Vec<SExpression>,
}

impl SExpression {
    pub fn push(&mut self, s: Self) {
        self.children.push(s);
    }

    pub fn push_front(&mut self, s: Self) {
        let mut new_children = vec![s];
        new_children.extend_from_slice(&self.children);
        self.children = new_children;
    }
}

pub fn sexpr(
    this: impl Into<String>,
    children: impl IntoIterator<Item = SExpression>,
) -> SExpression {
    SExpression {
        this: this.into(),
        children: children.into_iter().collect::<Vec<_>>(),
    }
}

fn indent(s: impl Into<String>, indent: &str) -> String {
    let s: String = s.into();
    s.lines()
        .map(|s| format!("{indent}{s}\n"))
        .collect::<String>()
        .trim_end()
        .to_string()
}

impl std::fmt::Display for SExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.children.is_empty() {
            write!(f, "{}", self.this)
        } else {
            let children = self
                .children
                .iter()
                .map(|c| format!("{c}"))
                .collect::<Vec<_>>()
                .join("\n");
            write!(f, "({}\n{}\n)", self.this, indent(children, "  "))
        }
    }
}

impl From<&str> for SExpression {
    fn from(val: &str) -> Self {
        sexpr(val.to_string(), [])
    }
}
