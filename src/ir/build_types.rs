use super::*;
use crate::parse::*;
use crate::{
    IntoLog,
    WithSpan,
};

impl<'a> super::build_ir::Builder<'a> {
    pub fn type_definition(
        &mut self,
        path: Path,
        definition: TypeDefinition,
    ) -> Option<AbstractType> {
        let Spanned {
            inner: definition, ..
        } = definition;
        match definition {
            TypeDefinitionKind::TypeFunction { arguments, body } => {
                let argument_count = arguments.len();
                let mut variables = vec![];
                for arg in arguments {
                    let path = self.define_name(arg, NameSpace::Type, false)?;
                    let tv = self.symbols.fresh_tv();
                    self.local_types.insert(path, tv);
                    variables.push(tv);
                }
                if let Some(at) = self.symbols.types.get_mut(&path) {
                    at.variables = variables.clone().into_boxed_slice()
                };
                let mut inner_type = self.type_definition(path, *body)?;
                self.name_map.end_local_scopes(argument_count);
                inner_type.variables = variables.into_boxed_slice();
                Some(inner_type)
            }
            TypeDefinitionKind::Structure { lhs, rhs } => {
                Some(AbstractType {
                    variables: [].into(),
                    base: Type::Struct {
                        name: path,
                        fields: lhs
                            .into_iter()
                            .map(|l| l.inner)
                            .zip(
                                rhs.into_iter()
                                    .map(|t| self.type_expr(t))
                                    .collect::<Option<Vec<_>>>()?,
                            )
                            .collect(),
                    },
                })
            }
            TypeDefinitionKind::Sum {
                variant_names,
                variant_types,
            } => {
                let mut names = Vec::with_capacity(variant_names.len());
                let mut types = Vec::with_capacity(variant_types.len());
                for (id, (name, type_)) in variant_names
                    .clone()
                    .into_iter()
                    .zip(variant_types)
                    .enumerate()
                {
                    let type_ = if let Some(type_) = type_ {
                        self.type_expr(type_)?
                    } else {
                        Type::Unit
                    };
                    let term_path = self.define_name(name.clone(), NameSpace::Term, true)?;
                    self.finalize_name(&term_path);
                    self.define_name(name.clone(), NameSpace::Constructor, true)?;
                    names.push(name.inner);
                    types.push(type_);
                    let constructor = Constructor {
                        variant_id: id,
                        kind: todo!(),
                    };
                }
                Some(AbstractType {
                    variables: [].into(),
                    base: Type::Sum {
                        name: path,
                        variant_names: names,
                        variant_types: types,
                    },
                })
            }
            TypeDefinitionKind::Expression(inner) => {
                Some(AbstractType {
                    variables: Box::new([]),
                    base: self.type_expr(inner)?,
                })
            }
        }
    }
    pub fn type_expr(
        &mut self,
        expr: TypeExpression,
    ) -> Option<Type> {
        use TypeExpressionKind::*;
        let span = expr.span;
        Some(match expr.inner {
            Call(..) => {
                fn reduce_call(
                    t: TypeExpression,
                    args: &mut Vec<TypeExpression>,
                ) -> TypeExpression {
                    use TypeExpressionKind as e;
                    match t.inner {
                        e::Call(a, b) => {
                            let callee = reduce_call(*a, args);
                            args.push(*b);
                            callee
                        }
                        _ => t,
                    }
                }
                let mut arguments = vec![];
                let Spanned {
                    inner: callee,
                    span,
                } = reduce_call(expr, &mut arguments);
                let callee = match callee {
                    Identifier(name) => {
                        self.query_name(name.with_span(span), NameSpace::Type)
                            .done()?
                    }
                    ModulePath(a, b) => {
                        let path = Path::new(a, b);
                        self.query_path(&path.clone().with_span(span), NameSpace::Type)
                            .done()?;
                        path
                    }
                    Call(..) => unreachable!(),
                    _ => {
                        InstantiationError {
                            expected: 0,
                            provided: arguments.len(),
                        }
                        .into_log(self.logger, span);
                        return None;
                    }
                };
                let arguments = arguments
                    .into_iter()
                    .map(|a| self.type_expr(a))
                    .collect::<Option<Vec<_>>>()?;
                self.symbols
                    .get_type(&callee)
                    .try_instantiate(&arguments)
                    .into_log(self.logger, span)?;
                Type::Instantiation(callee, arguments)
            }
            Identifier(id) => {
                let path = self
                    .query_name(id.with_span(span), NameSpace::Type)
                    .done()?;
                if let Some(tv) = self.local_types.get(&path) {
                    Type::Variable(*tv)
                } else {
                    self.symbols
                        .get_type(&path)
                        .clone()
                        .instantiate(&[])
                        .into_log(self.logger, span)?
                }
            }
            ModulePath(a, b) => {
                let path = Path::new(a, b);
                self.query_path(&path.clone().with_span(span), NameSpace::Type)
                    .done()?;
                self.symbols
                    .get_type(&path)
                    .clone()
                    .instantiate(&[])
                    .into_log(self.logger, span)?
            }
            Product(ts) => {
                Type::Tuple(
                    ts.into_iter()
                        .map(|t| self.type_expr(t))
                        .collect::<Option<_>>()?,
                )
            }
            Function(a, b) => {
                Type::Function(self.type_expr(*a)?.into(), self.type_expr(*b)?.into())
            }
            Array(t) => Type::Array(self.type_expr(*t)?.into()),
            Unit => Type::Unit,
        })
    }
}
