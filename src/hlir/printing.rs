use super::*;

#[allow(unused_variables)]
impl HlIrModule {
  fn sexpr(&self, node: IrPtr) -> SExpression {
    let node = self.get_node(node);
    use HlIrKind as h;
    match &node.kind {
      h::Declaration {
        assignee,
        is_constant,
        type_assert,
        value,
      } => {
        if let Some(type_assert) = type_assert {
          sexpr(assignee, &[self.sexpr(*type_assert), self.sexpr(*value)])
        } else {
          sexpr(assignee, &[self.sexpr(*value)])
        }
      }
      h::Immediate(const_value) => sexpr(format!("{const_value}"), &[]),
      h::Block(items) => sexpr(
        "block",
        &items
          .into_iter()
          .map(|i| self.sexpr(*i))
          .collect::<Vec<_>>(),
      ),
      h::Identifier(name) => sexpr(name, &[]),
      h::StructDef {
        field_names,
        field_types,
      } => todo!(),
      h::StructLiteral {
        field_names,
        field_values,
        ..
      } => sexpr(
        "struct literal",
        &field_names
          .into_iter()
          .zip(field_values.into_iter())
          .map(|(name, value)| sexpr("field", &[sexpr(name, &[]), self.sexpr(*value)]))
          .collect::<Vec<_>>(),
      ),
      h::Field { of, index } => todo!(),
      h::Binary {
        op,
        opdef,
        left,
        right,
      } => todo!(),
      h::Unary { op, opdef, child } => todo!(),
      h::FunctionDef {
        name,
        parameter_names,
        parameter_types,
        parameter_spans,
        returns,
        body,
      } => todo!(),
      h::FunctionCall {
        callee,
        callee_name,
        arguments,
      } => todo!(),
      h::If {
        predicate,
        then,
        else_,
      } => todo!(),
      h::Loop {
        parameter_names,
        parameter_values,
        parameter_spans,
        body,
      } => todo!(),
      h::Break(_) => todo!(),
      h::Tuple(items) => todo!(),
    }
  }
}

impl Into<SExpression> for &HlIrModule {
  fn into(self) -> SExpression {
    self.sexpr(0)
  }
}
