use super::*;

pub trait Infer: Sized {
    fn infer(self, env: &mut Environment, free: &mut FreeVariableSet) -> Self;
}

impl<T> Infer for Box<T>
where
    T: Infer,
{
    fn infer(self, env: &mut Environment, free: &mut FreeVariableSet) -> Self {
        (*self).infer(env, free).into()
    }
}

impl<T> Infer for Option<T>
where
    T: Infer,
{
    fn infer(self, env: &mut Environment, free: &mut FreeVariableSet) -> Self {
        match self {
            Some(t) => Some(t.infer(env, free)),
            None => None,
        }
    }
}

impl<T> Infer for Vec<T>
where
    T: Infer,
{
    fn infer(self, env: &mut Environment, free: &mut FreeVariableSet) -> Self {
        self.into_iter().map(|t| t.infer(env, free)).collect()
    }
}

impl Infer for Pattern {
    fn infer(mut self, env: &mut Environment, free: &mut FreeVariableSet) -> Self {
        self.inner.inner = match (*self).clone().inner {
            PatternKind::Hole => {
                let fresh_tv = env.new_tv();
                self.type_ = Type::Variable(fresh_tv);
                PatternKind::Hole
            }
            PatternKind::Name(path) => {
                let fresh_tv = env.define_unknown(path.clone());
                free.insert(fresh_tv);
                self.type_ = Type::Variable(fresh_tv);
                PatternKind::Name(path)
            }
            PatternKind::Tuple(patterns) => {
                let patterns: Vec<_> = patterns.into_iter().map(|p| p.infer(env, free)).collect();
                self.type_ = Type::Product(patterns.iter().map(|p| p.type_.clone()).collect());
                PatternKind::Tuple(patterns)
            }
            PatternKind::Array(patterns) => {
                let tv = env.new_tv();
                free.insert(tv);
                let patterns = match patterns {
                    ArrayPattern::Exact(patterns) => {
                        let patterns = patterns.infer(env, free);
                        for p in &patterns {
                            env.type_constraint(Type::Variable(tv), p.type_.clone(), p.span);
                        }
                        ArrayPattern::Exact(patterns)
                    }
                    ArrayPattern::Leading { head, tail } => {
                        let head = head.infer(env, free);
                        for p in &head {
                            env.type_constraint(Type::Variable(tv), p.type_.clone(), p.span);
                        }
                        if let Some(tail) = &tail {
                            env.define(tail.clone(), Type::Array(Type::Variable(tv).into()));
                        }
                        ArrayPattern::Leading { head, tail }
                    }
                    ArrayPattern::Trailing { head, tail } => {
                        if let Some(head) = &head {
                            env.define(head.clone(), Type::Array(Type::Variable(tv).into()));
                        }
                        let tail = tail.infer(env, free);
                        for p in &tail {
                            env.type_constraint(Type::Variable(tv), p.type_.clone(), p.span);
                        }
                        ArrayPattern::Trailing { head, tail }
                    }
                    ArrayPattern::LeadingAndTrailing { head, middle, tail } => {
                        let head = head.infer(env, free);
                        if let Some(middle) = &middle {
                            env.define(middle.clone(), Type::Array(Type::Variable(tv).into()));
                        }
                        let tail = tail.infer(env, free);
                        for p in head.iter().chain(&tail) {
                            env.type_constraint(Type::Variable(tv), p.type_.clone(), p.span);
                        }
                        ArrayPattern::LeadingAndTrailing { head, middle, tail }
                    }
                };
                self.type_ = Type::Array(Type::Variable(tv).into());
                PatternKind::Array(patterns)
            }
            PatternKind::Constructor(mut constructor, pattern) => {
                env.freshen_type_variables(&mut constructor, &HashSet::new());
                let (in_type, out_type) = match &constructor.kind {
                    ConstructorKind::Unitary(t) => (Type::Unit, t.clone()),
                    ConstructorKind::Function(a, b) => (a.clone(), b.clone()),
                };
                self.type_ = out_type;
                let pattern = pattern.infer(env, free);
                env.type_constraint(in_type, pattern.type_.clone(), self.span);
                PatternKind::Constructor(constructor, pattern)
            }
            PatternKind::Literal(const_value) => {
                self.type_ = const_value.type_of();
                PatternKind::Literal(const_value)
            }
            PatternKind::TypeHint(p, mut type_) => {
                let p = p.infer(env, free);
                self.type_ = p.type_.clone();
                env.freshen_type_variables(&mut type_, &HashSet::new());
                env.type_constraint(p.type_.clone(), type_.clone(), self.span);
                PatternKind::TypeHint(p, type_)
            }
        };
        self
    }
}

impl Infer for IrNode {
    fn infer(mut self, env: &mut Environment, free: &mut FreeVariableSet) -> Self {
        use IrKind::*;
        self.inner.inner = match (*self).clone().inner {
            Let {
                assignee,
                value,
                in_,
            } => {
                let mut new_free = free.clone();
                let assignee = assignee.infer(env, &mut new_free);
                let value = value.infer(env, &mut new_free);
                env.type_constraint(assignee.type_.clone(), value.type_.clone(), self.span);
                let in_ = in_.infer(env, free);
                self.type_ = in_.clone().type_;
                Let {
                    assignee,
                    value,
                    in_,
                }
            }
            Immediate(const_value) => {
                self.type_ = const_value.type_of();
                Immediate(const_value)
            }
            Identifier(path) => {
                let type_ = env.get_type(&path, free);
                self.type_ = type_;
                Identifier(path)
            }
            Tuple(nodes) => {
                let nodes = nodes.infer(env, free);
                self.type_ = Type::Product(nodes.iter().map(|n| n.type_.clone()).collect());
                Tuple(nodes)
            }
            Struct {
                field_names,
                field_values,
            } => {
                let field_values = field_values.infer(env, free);
                let tv = Type::Variable(env.new_tv());
                self.type_ = tv.clone();
                for (name, value) in field_names.iter().zip(&field_values) {
                    env.struct_constraint(tv.clone(), value.type_.clone(), name.clone(), self.span);
                }
                Struct {
                    field_names,
                    field_values,
                }
            }
            Field { of, index } => {
                let of = of.infer(env, free);
                let tv = Type::Variable(env.new_tv());
                self.type_ = tv.clone();
                env.struct_constraint(of.type_.clone(), tv, index.clone(), self.span);
                Field { of, index }
            }
            Function {
                parameter_name,
                parameter_type,
                captures,
                body,
                ..
            } => {
                let mut new_free = free.clone();
                let parameter_inferred_type = if let Some(parameter_name) = parameter_name.clone() {
                    let tv = env.define_unknown((*parameter_name).clone());
                    new_free.insert(tv);
                    Type::Variable(tv)
                } else {
                    Type::Unit
                };
                if let Some(assert_type) = parameter_type.clone() {
                    env.type_constraint(
                        assert_type,
                        parameter_inferred_type.clone(),
                        parameter_name.clone().map(|o| o.span).unwrap_or(body.span),
                    );
                }
                let capture_types = captures
                    .iter()
                    .map(|c| env.get_type(c, &new_free))
                    .collect();
                let body = body.infer(env, &mut new_free);
                self.type_ =
                    Type::Function(parameter_inferred_type.into(), body.type_.clone().into());
                Function {
                    parameter_name,
                    parameter_type,
                    captures,
                    capture_types,
                    body,
                }
            }
            Call {
                callee,
                argument,
                opt,
            } => {
                let callee = callee.infer(env, free);
                let argument = argument.infer(env, free);
                let return_type = Type::Variable(env.new_tv());
                let func_type =
                    Type::Function(argument.type_.clone().into(), return_type.clone().into());
                env.type_constraint(func_type.clone(), callee.type_.clone(), self.span);
                self.type_ = return_type;
                Call {
                    callee,
                    argument,
                    opt,
                }
            }
            Semicolon(a, b) => {
                let a = a.infer(env, free);
                let b = b.infer(env, free);
                self.type_ = b.type_.clone();
                Semicolon(a, b)
            }
            If {
                predicate,
                then,
                else_,
            } => {
                let predicate = predicate.infer(env, free);
                let then = then.infer(env, free);
                let else_ = else_.infer(env, free);
                env.type_constraint(predicate.type_.clone(), Type::Boolean, predicate.span);
                env.type_constraint(
                    then.type_.clone(),
                    else_.type_.clone(),
                    then.span + else_.span,
                );
                self.type_ = then.type_.clone();
                If {
                    predicate,
                    then,
                    else_,
                }
            }
            Match {
                scrutinee,
                predicates,
                branches,
            } => {
                let scrutinee = scrutinee.infer(env, free);
                let (mut new_predicates, mut new_branches) = (vec![], vec![]);
                let mut branch_type = None;
                for (predicate, branch) in predicates.into_iter().zip(branches) {
                    let mut new_free = free.clone();
                    let predicate = predicate.infer(env, &mut new_free);
                    env.type_constraint(
                        scrutinee.type_.clone(),
                        predicate.type_.clone(),
                        predicate.span,
                    );
                    let branch = branch.infer(env, &mut new_free);
                    if let Some(last_branch) = branch_type.clone() {
                        env.type_constraint(last_branch, branch.type_.clone(), branch.span);
                    } else {
                        branch_type = Some(branch.type_.clone());
                    }
                    new_predicates.push(predicate);
                    new_branches.push(branch);
                }
                self.type_ = branch_type.unwrap_or(Type::Unit);
                Match {
                    scrutinee,
                    predicates: new_predicates,
                    branches: new_branches,
                }
            }
            ImportedSymbol(path, type_) => {
                // Imported symbols are always let-bound, NEVER free variables
                let mut new_type = type_.clone();
                env.freshen_type_variables(&mut new_type, &HashSet::new());
                self.type_ = new_type.clone();
                ImportedSymbol(path, new_type)
            }
            AsmLiteral(_) => unreachable!(),
        };
        self
    }
}
