use colored::Colorize;

use super::*;

fn left_pad(input: impl AsRef<str>) -> String {
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
            Tuple(items) => format!(
                "(\n{},\n) {type_}",
                items
                    .iter()
                    .map(Pattern::pretty)
                    .map(left_pad)
                    .collect::<Vec<_>>()
                    .join(",\n")
            ),
            Array(array_pattern) => format!(
                "[\n{},\n] {type_}",
                match array_pattern {
                    ArrayPattern::Exact(items) => items
                        .iter()
                        .map(Pattern::pretty)
                        .map(left_pad)
                        .collect::<Vec<_>>()
                        .join(",\n"),
                    ArrayPattern::Leading { head, tail } => head
                        .iter()
                        .map(Pattern::pretty)
                        .chain(
                            tail.clone()
                                .map(|p| format!("..{}::{}", p.major.yellow(), p.minor))
                        )
                        .map(left_pad)
                        .collect::<Vec<_>>()
                        .join(",\n"),
                    ArrayPattern::Trailing { head, tail } => head
                        .clone()
                        .map(|p| format!("..{}::{}", p.major.yellow(), p.minor))
                        .into_iter()
                        .chain(tail.iter().map(Pattern::pretty))
                        .map(left_pad)
                        .collect::<Vec<_>>()
                        .join(",\n"),
                    ArrayPattern::LeadingAndTrailing { head, middle, tail } => head
                        .iter()
                        .map(Pattern::pretty)
                        .chain(middle.clone().map(|p| format!(
                            "..{}::{}",
                            p.major.yellow(),
                            p.minor
                        )))
                        .chain(tail.iter().map(Pattern::pretty))
                        .map(left_pad)
                        .collect::<Vec<_>>()
                        .join(",\n"),
                }
            ),
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

impl PrettyPrint for IrModule {
    fn pretty(&self) -> String {
        let let_ = "let".red();
        let equal = "=".green();
        let let_definitions = left_pad(
            self.let_definitions
                .iter()
                .map(|(pat, expr)| {
                    let expr = left_pad(expr.pretty());
                    format!("{let_} {} {equal}\n{expr}", pat.pretty())
                })
                .collect::<Vec<_>>()
                .join("\n"),
        );
        let module = "module".red();
        let module_name = self.module_name.yellow();
        let end = "end".red();
        format!("{module} {module_name} {equal}\n{let_definitions}\n{end}")
    }
}

impl PrettyPrint for Type {
    fn pretty(&self) -> String {
        format!("{self}").italic().blue().to_string()
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
            } => {
                format!(
                    "{let_} {assignee} {equal} {value}\n{in_} {type_}\n{then}\n{else_kw} {else_}",
                    let_ = "let".red(),
                    equal = "=".green(),
                    value = value.pretty(),
                    in_ = "in".red(),
                    type_ = self.type_.pretty(),
                    then = left_pad(then.pretty()),
                    assignee = assignee.pretty(),
                    else_kw = "else".red(),
                    else_ = else_.pretty(),
                )
            }
            Immediate(const_value) => {
                format!(
                    "{const_value} {type_}",
                    const_value = const_value.pretty(),
                    type_ = self.type_.pretty()
                )
            }
            Identifier(path) => format!(
                "{path} {type_}",
                path = path.pretty(),
                type_ = self.type_.pretty()
            ),
            Tuple(items) => format!("(\n{},\n)", items.pretty()),
            Struct(_) => todo!(),
            Field { of, index } => format!("{of}.{index}", of = of.pretty()),
            Function {
                parameter_name,
                parameter_type,
                captures,
                capture_types: _,
                body,
            } => {
                let parameter_type = if let Some(parameter_type) = parameter_type {
                    format!(": {parameter_type}")
                } else {
                    "".to_string()
                };
                format!(
                    "fn (: {type_}) {parameter_name}{parameter_type}\n{captures} =>\n{body}",
                    captures = left_pad(format!("[{}]", captures.pretty().replace("\n", " "))),
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
            Semicolon(left, right) => format!(
                "{left};\n{right}",
                left = left.pretty(),
                right = right.pretty()
            ),
        }
    }
}
