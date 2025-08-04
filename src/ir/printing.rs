use super::*;

impl Into<SExpression> for &Pattern {
    fn into(self) -> SExpression {
        let mut se = match &self.kind {
            PatternKind::Name(path) => sexpr(path, []),
            PatternKind::Tuple(patterns) => sexpr("tuple", patterns.iter().map(|p| p.into())),
            PatternKind::Literal(const_value) => sexpr(format!("{const_value}"), []),
        };
        se.push_front(format!("(type {})", self.type_.borrow()).as_str().into());
        se
    }
}

impl IrModule {
    fn sexpr(&self, node: IrPtr) -> SExpression {
        let node = &self[node];
        use IrKind as h;
        let mut se = match &node.kind {
            h::Declaration {
                assignee,
                value,
                in_,
                ..
            } => sexpr(
                "let",
                [
                    sexpr("mangle", [assignee.into()]),
                    sexpr("value", [self.sexpr(*value)]),
                    if let Some(i) = in_ {
                        sexpr("in", [self.sexpr(*i)])
                    } else {
                        sexpr("", [])
                    },
                ],
            ),
            h::Immediate(const_value) => sexpr(format!("{const_value}"), []),
            h::Identifier(name) => sexpr("identifier", [name.into()]),
            h::StructLiteral {
                field_names,
                field_values,
                ..
            } => sexpr(
                "struct-literal",
                field_names
                    .into_iter()
                    .zip(field_values.into_iter())
                    .map(|(name, value)| {
                        sexpr(
                            "field",
                            [
                                sexpr("name", [sexpr(name, [])]),
                                sexpr("value", [self.sexpr(*value)]),
                            ],
                        )
                    }),
            ),
            h::Field { of, index } => sexpr(
                "field",
                [
                    sexpr("lhs", [self.sexpr(*of)]),
                    sexpr("rhs", [index.as_str().into()]),
                ],
            ),
            h::Binary { op, left, right } => {
                sexpr(format!("{op}"), [self.sexpr(*left), self.sexpr(*right)])
            }
            h::Unary { op, child } => sexpr(format!("{op}"), [self.sexpr(*child)]),
            h::FunctionDef {
                body,
                parameter_name,
                captures,
                capture_types,
                ..
            } => sexpr(
                "function",
                [
                    Some(sexpr(
                        "argument",
                        [parameter_name.clone().unwrap_or("()".into()).into()],
                    )),
                    if captures.len() == 0 {
                        None
                    } else {
                        Some(sexpr(
                            "captures",
                            captures
                                .into_iter()
                                .zip(capture_types.into_iter())
                                .map(|(cap, ty)| {
                                    sexpr(cap, [format!("{}", ty.borrow()).as_str().into()])
                                }),
                        ))
                    },
                    Some(sexpr("body", [self.sexpr(*body)])),
                ]
                .into_iter()
                .flatten(),
            ),
            h::FunctionCall {
                callee,
                argument: arguments,
                ..
            } => sexpr(
                "call",
                [
                    sexpr("func", [self.sexpr(*callee)]),
                    sexpr("arg", [self.sexpr(*arguments)]),
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
                        [
                            sexpr("predicate", [self.sexpr(*predicate)]),
                            sexpr("then", [self.sexpr(*then)]),
                            sexpr("else", [self.sexpr(*else_)]),
                        ],
                    )
                } else {
                    sexpr("if", [sexpr("then", [self.sexpr(*then)])])
                }
            }
            h::Match {
                scrutinee,
                predicates,
                branches,
            } => sexpr(
                "match",
                [
                    sexpr("scrutinee", [self.sexpr(*scrutinee)]),
                    sexpr(
                        "branches",
                        predicates.into_iter().zip(branches).map(|(p, b)| {
                            sexpr(
                                "",
                                [
                                    sexpr("predicate", [p.into()]),
                                    sexpr("branch", [self.sexpr(*b)]),
                                ],
                            )
                        }),
                    ),
                ],
            ),
            h::Tuple(items) => sexpr(
                "tuple",
                items
                    .into_iter()
                    .map(|n| self.sexpr(*n))
                    .collect::<Vec<_>>(),
            ),
            h::ImportedSymbol(name, _) => sexpr("module-import", [name.into()]),
        };
        se.push_front(format!("(type {})", node.type_.borrow()).as_str().into());
        se
    }
}

impl std::fmt::Display for IrModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            sexpr(
                "module",
                self.items.iter().map(|i| {
                    match i {
                        ModuleItem::Let(name, ir) => sexpr(
                            "let",
                            [
                                sexpr("name", [name.into()]),
                                sexpr("value", [self.sexpr(*ir)]),
                            ],
                        ),
                        ModuleItem::Type(name, type_) => sexpr(
                            "type",
                            [
                                sexpr("name", [name.into()]),
                                sexpr("value", [format!("{}", type_.borrow()).as_str().into()]),
                            ],
                        ),
                        ModuleItem::Constructor {
                            name,
                            parameter,
                            sum,
                            ..
                        } => sexpr(
                            "constructor",
                            [
                                sexpr("name", [name.into()]),
                                sexpr("from", [format!("{}", parameter.borrow()).as_str().into()]),
                                sexpr("to", [format!("{}", sum.borrow()).as_str().into()]),
                            ],
                        ),
                    }
                })
            )
        )
    }
}

impl Into<SExpression> for &IrModule {
    fn into(self) -> SExpression {
        self.sexpr(0)
    }
}
