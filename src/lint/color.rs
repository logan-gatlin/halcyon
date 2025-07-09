#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Color {
  Red,
  Green,
  Blue,
  Yellow,
  Normal,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Attribute {
  Normal,
  Bold,
  Italic,
  Underline,
}

pub trait ApplyAttribute {
  fn apply_style(self, color: Color, attribute: Attribute) -> String;
}

#[cfg(target_family = "wasm")]
impl<T> ApplyAttribute for T
where
  T: Into<String>,
{
  fn apply_style(self, color: Color, attribute: Attribute) -> String {
    let s: String = self.into();
    s
  }
}

#[cfg(not(target_family = "wasm"))]
impl<T> ApplyAttribute for T
where
  T: std::fmt::Display,
{
  fn apply_style(self, color: Color, attribute: Attribute) -> String {
    let color_str = match color {
      Color::Red => "31",
      Color::Green => "32",
      Color::Yellow => "33",
      Color::Blue => "34",
      Color::Normal => "97",
    };
    let attribute_str = match attribute {
      Attribute::Normal => "0",
      Attribute::Bold => "1",
      Attribute::Italic => "3",
      Attribute::Underline => "4",
    };
    format!("\x1b[{attribute_str};{color_str}m{self}\x1b[0m")
  }
}
