use super::*;

impl MlIrModule {
  pub(super) fn build_dependency_graph(&mut self) {
    for name in self.blocks.keys() {
      self
        .dependencies
        .insert(name.clone(), self.find_dependencies_of(name.clone()));
    }
  }

  fn find_dependencies_of(&self, mut mangle: Mangle) -> HashSet<Mangle> {
    let original = mangle.clone();
    let mut visited = HashSet::new();
    let mut to_visit = vec![];
    loop {
      visited.insert(mangle.clone());
      let block = if let Some(block) = self.blocks.get(&mangle) {
        block
      } else {
        return HashSet::new();
      };
      if let BlockKind::Function { parameters, .. } = &block.kind {
        to_visit.extend(parameters.iter().cloned());
      }
      if !(matches!(block.kind, BlockKind::Function { .. }) && mangle == original) {
        for ir in &block.body {
          use MlIrKind::*;
          match &ir.kind {
            Get(mangle) => {
              to_visit.push(mangle.clone());
            }
            Const(ConstValue::Function { name, .. }) => {
              to_visit.push(name.clone());
            }
            _ => {}
          }
        }
      }
      loop {
        if let Some(top) = to_visit.pop() {
          if !visited.contains(&top) && self.blocks.contains_key(&top) {
            mangle = top;
            break;
          }
        } else {
          visited.remove(&original);
          return visited;
        }
      }
    }
  }
}
