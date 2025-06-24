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
        in_,
      } => sexpr(
        if *is_constant { "let-const" } else { "let" },
        &[
          sexpr("mangle", &[assignee.as_str().into()]),
          sexpr("value", &[self.sexpr(*value)]),
          if let Some(i) = in_ {
            sexpr("in", &[self.sexpr(*i)])
          } else {
            sexpr("", &[])
          },
        ],
      ),
      h::Immediate(const_value) => sexpr(format!("{const_value}"), &[]),
      h::Block(items) => sexpr(
        "block",
        &items
          .into_iter()
          .map(|i| self.sexpr(*i))
          .collect::<Vec<_>>(),
      ),
      h::Identifier(name) => sexpr("identifier", &[name.as_str().into()]),
      h::StructDef {
        field_names,
        field_types,
      } => sexpr(
        "struct-definition",
        &field_names
          .into_iter()
          .zip(field_types.into_iter())
          .map(|(name, value)| {
            sexpr(
              "field",
              &[
                sexpr("name", &[sexpr(name, &[])]),
                sexpr("type", &[self.sexpr(*value)]),
              ],
            )
          })
          .collect::<Vec<_>>(),
      ),
      h::StructLiteral {
        field_names,
        field_values,
        ..
      } => sexpr(
        "struct-literal",
        &field_names
          .into_iter()
          .zip(field_values.into_iter())
          .map(|(name, value)| {
            sexpr(
              "field",
              &[
                sexpr("name", &[sexpr(name, &[])]),
                sexpr("value", &[self.sexpr(*value)]),
              ],
            )
          })
          .collect::<Vec<_>>(),
      ),
      h::Field { of, index } => {
        sexpr("field", &[self.sexpr(*of), index.as_str().into()])
      },
      h::Binary { op, left, right } => {
        sexpr(format!("{op}"), &[self.sexpr(*left), self.sexpr(*right)])
      },
      h::Unary { op, child } => sexpr(format!("{op}"), &[self.sexpr(*child)]),
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
        &[
          sexpr("func", &[self.sexpr(*callee)]),
          sexpr(
            "args",
            arguments
              .into_iter()
              .map(|a| self.sexpr(*a))
              .collect::<Vec<_>>()
              .as_slice(),
          ),
        ],
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
              sexpr("predicate", &[self.sexpr(*predicate)]),
              sexpr("then", &[self.sexpr(*then)]),
              sexpr("else", &[self.sexpr(*else_)]),
            ],
          )
        } else {
          sexpr("if", &[sexpr("then", &[self.sexpr(*then)])])
        }
      },
      h::Tuple(items) => sexpr(
        "tuple",
        &items
          .into_iter()
          .map(|n| self.sexpr(*n))
          .collect::<Vec<_>>(),
      ),
    };
    se.push(format!("(type {})", node.type_).as_str().into());
    se
  }
}

impl std::fmt::Display for HlIrModule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.sexpr(0))
  }
}

impl Into<SExpression> for &HlIrModule {
  fn into(self) -> SExpression {
    self.sexpr(0)
  }
}
