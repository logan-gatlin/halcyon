mod check;
mod exhaustiveness;
mod inference;
mod unification;

use check::*;
use inference::*;
use unification::*;

use std::collections::{HashMap, HashSet};

use crate::{ir::*, lint::*, operator::*};

#[derive(Debug, Clone, Default)]
pub struct Environment {
    scheme_map: HashMap<Path, bool>,
    value_map: HashMap<Path, TypeRef>,
    type_map: HashMap<Path, TypeRef>,
    type_variable: TypeVariable,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn fresh_type_var(&mut self) -> TypeRef {
        let tv = self.type_variable;
        self.type_variable += 1;
        Type::TypeVariable(tv).to_ref()
    }

    pub fn reset_type_variable(&mut self) {
        self.type_variable = 0;
    }

    fn map_fresh_type_variables(
        &mut self,
        t: &TypeRef,
        map: &mut HashMap<TypeVariable, TypeVariable>,
    ) {
        match &*(t.borrow()) {
            Type::Weak(_)
            | Type::Any
            | Type::_ClosureCapture
            | Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph => {}
            Type::TypeVariable(tv) => {
                if !map.contains_key(tv) {
                    let new_tv = self.type_variable;
                    self.type_variable += 1;
                    map.insert(*tv, new_tv);
                }
            }
            Type::Sum {
                variant_types: items,
                ..
            }
            | Type::Product(items)
            | Type::Struct {
                member_types: items,
                ..
            } => items
                .into_iter()
                .for_each(|t| self.map_fresh_type_variables(&t, map)),
            Type::Function(param_type, return_type) => {
                self.map_fresh_type_variables(param_type, map);
                self.map_fresh_type_variables(return_type, map);
            }
        }
    }

    pub fn define_type(&mut self, mangle: Path, type_: TypeRef) {
        self.type_map.insert(mangle, type_);
    }

    pub fn get_type(&self, mangle: &Path) -> TypeRef {
        self.type_map.get(mangle).unwrap().clone()
    }

    pub fn get_value_type(&mut self, mangle: &Path) -> TypeRef {
        if *self.scheme_map.get(mangle).unwrap() {
            let t = self.value_map.get(mangle).unwrap().clone();
            let mut fresh = HashMap::new();
            self.map_fresh_type_variables(&t, &mut fresh);
            fresh
                .into_iter()
                .for_each(|(old, new)| t.borrow_mut().unify(old, &Type::TypeVariable(new)));
            t
        } else {
            self.value_map.get(mangle).unwrap().clone()
        }
    }

    pub fn insert_value_type(&mut self, mangle: Path, type_: TypeRef, let_bound: bool) {
        self.value_map.insert(mangle.clone(), type_);
        self.scheme_map.insert(mangle, let_bound);
    }
}

#[derive(Debug, Clone)]
pub struct TypeConstraint(TypeRef, TypeRef, Span);

impl std::fmt::Display for TypeConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) == ({})", self.0.borrow(), self.1.borrow())
    }
}

#[derive(Debug, Clone)]
pub struct Substitution(pub TypeVariable, pub TypeRef);

#[derive(Debug, Clone, Default)]
pub struct ModuleInterface {
    pub types: HashMap<Path, TypeRef>,
    pub values: HashMap<Path, TypeRef>,
}

pub fn type_solve(module: &mut IrModule) -> Result<ModuleInterface> {
    let mut interface = ModuleInterface::default();
    let mut environment = Environment::new();
    for (id, item) in module.items.clone().into_iter().enumerate() {
        match item {
            ModuleItem::Let(mut assignee, ir) => {
                let mut new_constraints = vec![];
                let recursive_type_placeholder = infer_pattern(&mut assignee, &mut environment);
                type_inference(module, ir, &mut environment, &mut new_constraints)?;
                new_constraints.push(TypeConstraint(
                    recursive_type_placeholder.clone(),
                    module.nodes[ir].type_.clone(),
                    module.nodes[ir].span,
                ));
                let solution = unification(&new_constraints)?;
                assignee.unify_all(&solution);
                apply_solution(module, ir, solution);
                let second_solution = unification(&check_structs(&mut environment, module, ir)?)?;
                assignee.unify_all(&second_solution);
                apply_solution(module, ir, second_solution);
                module.items[id] = ModuleItem::Let(assignee.clone(), ir);
                assignee.iter_names(&mut |name, type_| {
                    interface.values.insert(name.clone(), type_.clone());
                    environment.insert_value_type(name.clone(), type_.clone(), true);
                });
                environment.reset_type_variable();
            }
            ModuleItem::Type(name, type_) => {
                environment.define_type(name.clone(), type_.clone());
                interface.types.insert(name.clone(), type_.clone());
            }
            ModuleItem::Constructor {
                name,
                parameter,
                sum,
                ..
            } => {
                let ftype = Type::func(parameter, sum);
                environment.insert_value_type(name.clone(), ftype.clone(), true);
                interface.types.insert(name, ftype);
            }
        }
    }
    exhaustiveness::check(&module)?;
    Ok(interface)
}
