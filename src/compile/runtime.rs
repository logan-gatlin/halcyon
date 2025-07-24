use crate::builtin::Builtin;

use super::*;

impl ModuleEncoder {
  pub fn generate_builtin(&mut self, builtin: Builtin) -> u32 {
    match builtin {
      Builtin::Assert => {
        let f =
          self.new_function(&builtin.get_type(), "a".into(), vec![], vec![]);
        self.func(f).get_local("a");
        self.unwrap_primitive(f, &Type::Boolean.into());
        self.push(f, I32Eqz);
        self.push(f, If(BlockType::Empty));
        self.push(f, Unreachable);
        self.push(f, End);
        self.push_constant(f, ConstValue::Unit);
        f
      },
    }
  }
}
