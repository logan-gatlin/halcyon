use super::*;

impl Into<SExpression> for &Pattern {
  fn into(self) -> SExpression {
    sexpr(
      "pattern",
      &[match &self.kind {
        PatternKind::Const(const_value) => {
          const_value.to_string().as_str().into()
        },
        PatternKind::Wildcard(name) => sexpr(name, &[]),
        PatternKind::Tuple(patterns) => sexpr(
          "tuple",
          &patterns.into_iter().map(|p| p.into()).collect::<Vec<_>>(),
        ),
      }],
    )
  }
}

#[allow(unused_variables)]
impl HlIrModule {
  fn sexpr(&self, node: IrPtr) -> SExpression {
    let node = self.get_node(node);
    use HlIrKind as h;
    let mut se = match &node.kind {
      h::Declaration {
        assignee,
        is_constant,
        value,
      } => sexpr("assign", &[assignee.as_str().into(), self.sexpr(*value)]),
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
      } => sexpr(
        "struct definition",
        &field_names
          .into_iter()
          .zip(field_types.into_iter())
          .map(|(name, value)| {
            sexpr("field", &[sexpr(name, &[]), self.sexpr(*value)])
          })
          .collect::<Vec<_>>(),
      ),
      h::StructLiteral {
        field_names,
        field_values,
        ..
      } => sexpr(
        "struct literal",
        &field_names
          .into_iter()
          .zip(field_values.into_iter())
          .map(|(name, value)| {
            sexpr("field", &[sexpr(name, &[]), self.sexpr(*value)])
          })
          .collect::<Vec<_>>(),
      ),
      h::Field { of, index } => {
        sexpr("field", &[self.sexpr(*of), index.as_str().into()])
      },
      h::Binary {
        op,
        opdef,
        left,
        right,
      } => sexpr(format!("{op}"), &[self.sexpr(*left), self.sexpr(*right)]),
      h::Unary { op, opdef, child } => {
        sexpr(format!("{op}"), &[self.sexpr(*child)])
      },
      h::FunctionDef {
        name,
        parameter_names,
        parameter_spans,
        body,
      } => sexpr("function", &[self.sexpr(*body)]),
      h::FunctionCall {
        callee,
        callee_name,
        arguments,
      } => sexpr(
        "call",
        &[callee]
          .into_iter()
          .chain(arguments.into_iter())
          .map(|a| self.sexpr(*a))
          .collect::<Vec<_>>(),
      ),
      h::If {
        predicate,
        then,
        else_,
      } => {
        if let Some(else_) = else_ {
          sexpr(
            "if",
            &[
              sexpr("pred", &[self.sexpr(*predicate)]),
              sexpr("then", &[self.sexpr(*then)]),
              sexpr("else", &[self.sexpr(*else_)]),
            ],
          )
        } else {
          sexpr("if", &[sexpr("then", &[self.sexpr(*then)])])
        }
      },
      h::Loop {
        parameter_names,
        parameter_values,
        parameter_spans,
        body,
      } => sexpr("loop", &[self.sexpr(*body)]),
      h::Break(_) => sexpr("break", &[]),
      h::Tuple(items) => sexpr(
        "tuple",
        &items
          .into_iter()
          .map(|n| self.sexpr(*n))
          .collect::<Vec<_>>(),
      ),
      h::Match {
        on,
        patterns,
        branches,
      } => sexpr(
        "match",
        [on]
          .into_iter()
          .map(|on| sexpr("on", &[self.sexpr(*on)]))
          .chain(
            patterns
              .into_iter()
              .zip(branches.into_iter())
              .map(|(p, b)| sexpr("|", &[p.into(), self.sexpr(*b)])),
          )
          .collect::<Vec<_>>()
          .as_ref(),
      ),
    };
    se.push(format!("(type {})", node.type_).as_str().into());
    se
  }
}

impl Into<SExpression> for &HlIrModule {
  fn into(self) -> SExpression {
    self.sexpr(0)
  }
}
