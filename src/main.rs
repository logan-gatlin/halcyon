extern crate lang;
use lang::compile;

#[allow(unused_variables)]
fn main() {
  compile(include_str!("../demo.hc"));
}
