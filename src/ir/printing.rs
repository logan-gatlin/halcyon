use super::*;

impl From<&Pattern> for SExpression {
    fn from(val: &Pattern) -> Self {
        let mut se = match &val.kind {
            PatternKind::Name(path) => sexpr(path, []),
            PatternKind::Tuple(patterns) => sexpr("tuple", patterns.iter().map(|p| p.into())),
            PatternKind::Literal(const_value) => sexpr(format!("{const_value}"), []),
            PatternKind::Constructor(_path, pattern) => {
                sexpr("constructor", [Into::<SExpression>::into(&**pattern)])
            }
        };
        se.push_front(format!("(type {})", val.type_).as_str().into());
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
                    .iter()
                    .zip(field_values)
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
                    if captures.is_empty() {
                        None
                    } else {
                        Some(sexpr(
                            "captures",
                            captures
                                .iter()
                                .zip(capture_types)
                                .map(|(cap, ty)| sexpr(cap, [format!("{}", ty).as_str().into()])),
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
                        predicates.iter().zip(branches).map(|(p, b)| {
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
                    .iter()
                    .map(|n| self.sexpr(*n))
                    .collect::<Vec<_>>(),
            ),
            h::ImportedSymbol(name, _) => sexpr("module-import", [name.into()]),
        };
        se.push_front(format!("(type {})", node.type_).as_str().into());
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
                                sexpr("value", [format!("{}", type_).as_str().into()]),
                            ],
                        ),
                        ModuleItem::Constructor(name, cons) => sexpr(
                            "constructor",
                            [
                                sexpr("name", [name.into()]),
                                sexpr("from", [format!("{}", cons.in_type).as_str().into()]),
                                sexpr("to", [format!("{}", cons.out_type).as_str().into()]),
                            ],
                        ),
                    }
                })
            )
        )
    }
}

impl From<&IrModule> for SExpression {
    fn from(val: &IrModule) -> Self {
        val.sexpr(0)
    }
}
