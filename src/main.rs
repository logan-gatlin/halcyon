extern crate lang;
use lang::compile;

fn main() {
  compile(include_str!("../demo.hc"));
}
