use std::collections::HashSet;

use crate::err::*;
use crate::naming::Mangle;

use super::{Block, IrKind, IrPtr, Module};

impl Module {
  pub fn consteval(&mut self) {
    let mut deps = self
      .constants
      .iter()
      .map(|(mangle, ptr)| (mangle.clone(), self.find_dependencies(*ptr)))
      .collect::<Vec<_>>();
    println!("{deps:#?}");
  }

  pub fn find_dependencies(&self, block: IrPtr) -> HashSet<Mangle> {
    let mut deps = HashSet::new();
    let mut visited = HashSet::new();
    let mut to_visit = vec![];
    let mut current_block = block;
    loop {
      visited.insert(current_block);
      match &self.blocks[current_block] {
        Block::Terminal => {},
        Block::Unreachable => {},
        Block::Basic { body, next } => {
          body
            .into_iter()
            .flat_map(|ir| {
              if let IrKind::Get(ident) = &ir.kind {
                Some(ident)
              } else {
                None
              }
            })
            .for_each(|ident| {
              if let Some(block) = self.constants.get(ident) {
                deps.insert(ident.clone());
                to_visit.push(*block);
              }
            });
          to_visit.push(*next);
        },
        Block::Branch {
          when_true,
          when_false,
        } => {
          to_visit.push(*when_true);
          to_visit.push(*when_false);
        },
      }
      loop {
        if let Some(ptr) = to_visit.pop() {
          if !visited.contains(&ptr) {
            current_block = ptr;
            break;
          }
        } else {
          return deps;
        }
      }
    }
  }
}
