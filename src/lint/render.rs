use std::{collections::HashMap, process::exit};

use toml::Table;

use super::*;

#[derive(Debug, Clone)]
struct LintDescription {
  code: usize,
  reason: String,
  help: String,
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
    if p.contains('\n') {
      out = out.replacen('%', &format!("\n{p}\n"), 1);
    } else {
      out = out.replacen('%', p, 1);
    }
  }
  out
}

impl Linter {
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
          .map(|s| s.to_string())
          .unwrap();
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

  fn underline_source(&self, span: Span) -> (String, String, String, usize) {
    let (row, column) = self.get_row_column(span);
    let position_str = format!(
      "{} {}:{}",
      "-->".apply_style(Color::Blue, Attribute::Normal),
      row + 1,
      column
    )
    .apply_style(Color::Blue, Attribute::Normal);
    let line_no = self.get_first_line(span);
    let src_text = self.line_map[line_no].clone();
    let mut underline = String::with_capacity(src_text.len());
    let start_index = self.index_map[line_no];
    for (id, c) in src_text.chars().enumerate() {
      if start_index + id >= span.start
        && start_index + id < span.start + span.width
      {
        underline.push('^');
      } else if c == '\t' {
        underline.push('\t');
      } else {
        underline.push(' ');
      }
    }
    let underline = underline.apply_style(Color::Red, Attribute::Normal);
    (position_str, src_text, underline, row + 1)
  }

  pub fn render(&self, lint: Lint) -> String {
    let description = self.lint_map.get(&lint.kind).unwrap();
    let source_help = if let Some(s) = lint.span {
      let (position, src, underline, row) = self.underline_source(s);
      let width = row.to_string().len();
      let rowmark1 = format!("{row:^width$}|", width = width + 2)
        .apply_style(Color::Blue, Attribute::Normal);
      let rowmark2 = format!("{:^width$}|", "", width = width + 2)
        .apply_style(Color::Blue, Attribute::Normal);
      format!("{position}\n{rowmark1}{src}\n{rowmark2}{underline}\n",)
    } else {
      "".into()
    };
    let error = format!(
      "{} {}\n",
      "error:".apply_style(Color::Red, Attribute::Italic),
      description.reason
    );
    let help = format!(
      "{} {}\n",
      "help:".apply_style(Color::Blue, Attribute::Italic),
      description.help
    );
    format!("{source_help}{error}{help}")
  }
  /*
  pub fn _render(&self, lint: Lint) -> String {
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
  */
}
