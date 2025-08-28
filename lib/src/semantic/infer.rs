use sx::SXRepr;

use super::*;

pub trait Infer {
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
        self.map(|t| t.infer(env, free))
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
            PatternKind::Name(path) => {
                let fresh_tv = env.define_unknown(path.clone());
                free.insert(path.clone());
                self.type_ = Type::Variable(fresh_tv);
                PatternKind::Name(path)
            }
            PatternKind::Tuple(patterns) => {
                let patterns: Vec<_> = patterns.into_iter().map(|p| p.infer(env, free)).collect();
                self.type_ = Type::Product(patterns.iter().map(|p| p.type_.clone()).collect());
                PatternKind::Tuple(patterns)
            }
            PatternKind::Constructor(constructor, pattern) => {
                self.type_ = constructor.out_type.clone();
                let pattern = pattern.infer(env, free);
                env.type_constraint(
                    constructor.in_type.clone(),
                    pattern.type_.clone(),
                    self.span,
                );
                PatternKind::Constructor(constructor, pattern)
            }
            PatternKind::Literal(const_value) => {
                self.type_ = const_value.type_of();
                PatternKind::Literal(const_value)
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
                let mut new_env = env.clone();
                let mut new_free = free.clone();
                let mut assignee = assignee.infer(&mut new_env, &mut new_free);
                let mut value = value.infer(&mut new_env, &mut new_free);
                new_env.type_constraint(assignee.type_.clone(), value.type_.clone(), self.span);
                let solution = new_env.solve_constraints();
                unify_all(&mut assignee, &solution);
                unify_all(&mut value, &solution);
                assignee.visit(|(path, t)| {
                    env.map.borrow_mut().insert(path.clone(), t.clone());
                });
                let in_ = in_.infer(env, free);
                self.type_ = in_.clone().map(|t| t.type_).unwrap_or(Type::Unit);
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
                self.type_ = env.get_type(&path, free);
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
            } => todo!(),
            Field { of, index } => todo!(),
            Function {
                parameter_name,
                parameter_type,
                captures,
                capture_types,
                body,
            } => {
                let mut new_free = free.clone();
                let parameter_inferred_type = if let Some(parameter_name) = parameter_name.clone() {
                    let tv = env.define_unknown((*parameter_name).clone());
                    new_free.insert((*parameter_name).clone());
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
                argument_first,
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
                    argument_first,
                }
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
            ImportedSymbol(path, mut type_) => {
                // Imported symbols are always let-bound, NEVER free variables
                env.freshen_type_variables(&mut type_, &HashSet::new());
                self.type_ = type_.clone();
                ImportedSymbol(path, type_)
            }
        };
        self
    }
}

impl Infer for ModuleItem {
    fn infer(self, env: &mut Environment, free: &mut FreeVariableSet) -> Self {
        match self {
            ModuleItem::Let(assignee, node) => {
                let mut new_env = env.clone();
                let mut new_free = free.clone();
                let mut pattern = assignee.infer(&mut new_env, &mut new_free);
                let mut node = node.infer(&mut new_env, &mut new_free);
                println!("TYPES: \n{}", node.clone().sx());
                new_env.print_constraints();
                new_env.type_constraint(pattern.type_.clone(), node.type_.clone(), pattern.span);
                let solution = new_env.solve_constraints();
                unify_all(&mut pattern, &solution);
                unify_all(&mut node, &solution);
                pattern.visit(|(path, t)| {
                    env.map.borrow_mut().insert(path.clone(), t.clone());
                });
                ModuleItem::Let(pattern, node)
            }
            ModuleItem::Constructor(path, cons) => {
                env.define(path.clone(), cons.function_type());
                ModuleItem::Constructor(path, cons)
            }
            _ => self,
        }
    }
}

impl Infer for IrModule {
    fn infer(self, env: &mut Environment, free: &mut FreeVariableSet) -> Self {
        IrModule {
            items: self.items.into_iter().map(|i| i.infer(env, free)).collect(),
            ..self
        }
    }
}
