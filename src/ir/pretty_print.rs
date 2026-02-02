use colored::Colorize;

use super::*;

pub fn left_pad(input: impl AsRef<str>) -> String {
    input
        .as_ref()
        .lines()
        .map(|line| format!("{PAD}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

const PAD: &str = "  ";

pub trait PrettyPrint {
    fn pretty(&self) -> String;
}

impl PrettyPrint for Path {
    fn pretty(&self) -> String {
        format!("{}::{}", self.major.yellow(), self.minor)
    }
}

impl PrettyPrint for Pattern {
    fn pretty(&self) -> String {
        use PatternKind::*;
        let type_ = format!("{}", self.type_).italic().blue();

        match &self.inner.inner {
            Hole => format!("_ {type_}"),
            Identifier(path) => format!("{} {type_}", path.minor),
            Tuple(items) => {
                format!(
                    "(\n{},\n) {type_}",
                    items
                        .iter()
                        .map(Pattern::pretty)
                        .map(left_pad)
                        .collect::<Vec<_>>()
                        .join(",\n")
                )
            }
            Struct(map) => {
                format!(
                    "{{\n{},\n}} {type_}",
                    left_pad(
                        map.iter()
                            .map(|(k, v)| { format!("{k} = {}", v.pretty()) })
                            .collect::<Vec<_>>()
                            .join(",\n")
                    )
                )
            }
            Array {
                starting,
                glob,
                ending,
            } => {
                let glob_str = match glob {
                    Glob::None => None,
                    Glob::Unnamed => Some("..".to_string()),
                    Glob::Named(id) => Some(format!("..{id}")),
                };
                format!(
                    "[\n{},\n]",
                    starting
                        .iter()
                        .map(Pattern::pretty)
                        .map(left_pad)
                        .chain(glob_str)
                        .chain(ending.iter().map(Pattern::pretty).map(left_pad))
                        .collect::<Vec<_>>()
                        .join(",\n")
                )
            }
            Constructor(_, pat) => format!("constructor {}", pat.pretty()),
            Immediate(const_value) => {
                let const_value = format!("{const_value}").magenta().to_string();
                format!("{const_value} {type_}")
            }
            TypeHint(pat, type_) => {
                format!("{} : {type_}", pat.pretty())
            }
        }
    }
}

impl PrettyPrint for Module {
    fn pretty(&self) -> String {
        let type_kw = "type".red();
        let equal = "=".green();
        let mut type_definitions = left_pad(
            self.types
                .iter()
                .map(|(name, at)| format!("{type_kw} {} {equal} {}", name.minor, at.pretty()))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        let let_definitions = left_pad(
            self.code
                .iter()
                .map(|expr| expr.pretty())
                .collect::<Vec<_>>()
                .join("\n"),
        );
        if !type_definitions.is_empty() && !let_definitions.is_empty() {
            type_definitions.push_str("\n\n");
        }
        let module = "module".red();
        let module_name = self.name.yellow();
        let end = "end".red();
        format!("{module} {module_name} {equal}\n{type_definitions}{let_definitions}\n{end}")
    }
}

impl PrettyPrint for Type {
    fn pretty(&self) -> String {
        format!("{self}").italic().blue().to_string()
    }
}

impl PrettyPrint for AbstractType {
    fn pretty(&self) -> String {
        self.base.pretty()
    }
}

impl PrettyPrint for ConstValue {
    fn pretty(&self) -> String {
        format!("{self}").magenta().to_string()
    }
}

impl<T> PrettyPrint for Vec<T>
where
    T: PrettyPrint,
{
    fn pretty(&self) -> String {
        self.iter().map(T::pretty).collect::<Vec<_>>().join(",\n")
    }
}

impl PrettyPrint for IrNode {
    fn pretty(&self) -> String {
        use IrKind::*;
        match &self.inner.inner {
            Let {
                assignee,
                value,
                then,
                else_,
                is_global: _,
            } => {
                let then_expr = if matches!(then.inner.inner, Immediate(ConstValue::Unit)) {
                    "".to_string()
                } else {
                    format!(
                        "{in_kw} {type_}\n{then}\n",
                        in_kw = "in".red(),
                        type_ = self.type_.pretty(),
                        then = left_pad(then.pretty())
                    )
                };
                let else_expr = if matches!(else_.inner.inner, Unreachable) {
                    "".to_string()
                } else {
                    format!("{} {}", "else".red(), else_.pretty())
                };
                format!(
                    "{let_kw} {assignee} {equal} {value}\n{then_expr}{else_expr}",
                    let_kw = "let".red(),
                    equal = "=".green(),
                    value = value.pretty(),
                    assignee = assignee.pretty(),
                )
            }
            Immediate(const_value) => {
                format!(
                    "{const_value} {type_}",
                    const_value = const_value.pretty(),
                    type_ = self.type_.pretty()
                )
            }
            Identifier(path) => {
                format!(
                    "{path} {type_}",
                    path = path.pretty(),
                    type_ = self.type_.pretty()
                )
            }
            Tuple(items) => {
                format!(
                    "(\n{},\n) {}",
                    left_pad(items.pretty()),
                    self.type_.pretty()
                )
            }
            Struct(map) => {
                format!(
                    "{{\n{},\n}}",
                    map.iter()
                        .map(|(key, val)| format!("{key} = {}", val.pretty()))
                        .map(left_pad)
                        .collect::<Vec<_>>()
                        .join(",\n")
                )
            }
            Field { of, index } => format!("{of}.{index}", of = of.pretty()),
            Function {
                parameter_name,
                parameter_type,
                captures,
                body,
            } => {
                let fn_kw = "fn".red();
                let parameter_type = if let Some(parameter_type) = parameter_type {
                    format!(": {parameter_type}")
                } else {
                    "".to_string()
                };
                format!(
                    "{fn_kw} (: {type_}) {parameter_name}{parameter_type} =>\n{body}",
                    body = left_pad(body.pretty()),
                    type_ = self.type_.pretty()
                )
            }
            Call { callee, argument } => {
                format!(
                    "{call} {type_}\n{callee}\n{argument}",
                    call = "call".red(),
                    type_ = self.type_.pretty(),
                    callee = left_pad(callee.pretty()),
                    argument = left_pad(argument.pretty()),
                )
            }
            Semicolon(left, right) => {
                format!(
                    "{left};\n{right}",
                    left = left.pretty(),
                    right = right.pretty()
                )
            }
            Unreachable => "unreachable".red().to_string(),
        }
    }
}
