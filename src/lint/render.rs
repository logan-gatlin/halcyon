use std::{collections::HashMap, process::exit};

use toml::Table;

use crate::fail_compile;

use super::{Lint, Span};

pub trait UnwrapLint<T> {
  fn unwrap_lint(self, linter: &Linter) -> T;
}

impl<T> UnwrapLint<T> for Result<T, Lint> {
  fn unwrap_lint(self, linter: &Linter) -> T {
    match self {
      Ok(t) => t,
      Err(lint) => {
        eprintln!("{}", linter.render(lint));
        eprintln!("\n{}Compilation failed{}", Linter::RED, Linter::RESET);
        fail_compile();
      },
    }
  }
}

#[derive(Debug, Clone)]
struct LintDescription {
  code: usize,
  reason: String,
  help: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Linter {
  lint_map: HashMap<usize, LintDescription>,
  line_map: Vec<String>,
  index_map: Vec<usize>,
  use_color: bool,
}

fn format_string(fstr: &str, params: &[String]) -> String {
  let mut out = fstr.to_string();
  for p in params {
    out = out.replacen('%', p, 1);
  }
  out
}

impl Linter {
  pub const BLUE: &str = "\x1b[0;34m";
  pub const GREEN: &str = "\x1b[0;32m";
  pub const RED: &str = "\x1b[0;31m";
  pub const RESET: &str = "\x1b[0m";

  pub fn new(source: String) -> Self {
    let mut line_map = vec![];
    let mut index_map = vec![];
    let mut index = 0;
    source.lines().for_each(|l| {
      index_map.push(index);
      index += l.len() + 1;
      line_map.push(l.to_string());
    });
    let lint_map = include_str!("../../lints/lints.toml")
      .parse::<Table>()
      .unwrap()
      .into_iter()
      .map(|(key, val)| {
        let key = key.parse::<usize>().unwrap();
        let table = val.as_table().unwrap();
        let reason = table
          .get("reason")
          .unwrap()
          .clone()
          .as_str()
          .unwrap()
          .to_string();
        let help = table
          .get("help")
          .map(|v| v.as_str().unwrap())
          .map(|s| s.to_string());
        (
          key,
          LintDescription {
            code: key,
            reason,
            help,
          },
        )
      })
      .collect();
    Self {
      lint_map,
      line_map,
      index_map,
      use_color: true,
    }
  }

  fn get_first_line(&self, span: Span) -> usize {
    match self.index_map.binary_search(&span.start) {
      Ok(s) => s,
      Err(s) => s - 1,
    }
  }

  fn get_row_column(&self, span: Span) -> (usize, usize) {
    let row = self.get_first_line(span);
    let column = span.start - self.index_map[row];
    (row + 1, column + 1)
  }

  pub fn render(&self, lint: Lint) -> String {
    let info = if let Some(s) = lint.span {
      let first = self.get_first_line(s);
      let src_text = self.line_map[first].clone();
      let mut underline = String::with_capacity(src_text.len());
      let start_index = self.index_map[first];
      for (id, c) in src_text.chars().enumerate() {
        if start_index + id == s.start {
          underline.push_str(Self::RED);
        }
        if start_index + id == s.start + s.width {
          underline.push_str(Self::RESET);
        }
        if start_index + id >= s.start && start_index + id < s.start + s.width {
          underline.push('^');
        } else if c == '\t' {
          underline.push('\t');
        } else {
          underline.push(' ');
        }
      }
      underline.push_str(Self::RESET);
      let rc = self.get_row_column(s);
      let row_text = format!("{}", first + 1);
      let row_text_len = row_text.chars().count();
      Some(format!(
        "{} -->{} {}:{}\n{}{row_text:^width$} |{} {src_text}\n{}{:^width$} \
         |{} {underline}\n",
        Self::BLUE,
        Self::RESET,
        rc.0,
        rc.1,
        Self::BLUE,
        Self::RESET,
        Self::BLUE,
        "",
        Self::RESET,
        width = row_text_len,
      ))
    } else {
      None
    };
    let desc = self.lint_map.get(&lint.kind).unwrap();
    format!(
      "{}error{}: {}\n{}{}help{}: {}",
      Self::RED,
      Self::RESET,
      desc.reason,
      info.unwrap_or("".into()),
      Self::GREEN,
      Self::RESET,
      format_string(&desc.help.clone().unwrap_or("".into()), &lint.context),
    )
  }
}
