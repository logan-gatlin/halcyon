mod constraint;
mod infer;
mod types;
mod unify;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{Visit, lint::*};
pub use constraint::*;
pub use infer::*;
use sx::SXRepr;
pub use types::*;

use crate::ir::*;

#[derive(Debug, Clone, Default, sx::SXRepr)]
pub struct ModuleInterface {
    pub types: HashSet<Path>,
    pub values: HashMap<Path, Type>,
    pub constructors: HashMap<Path, Constructor>,
}

pub type FreeVariableSet = HashSet<Path>;

#[derive(Debug, Clone, Default)]
pub struct Environment {
    map: Rc<RefCell<HashMap<Path, Type>>>,
    constraints: Vec<TypeConstraint>,
    current_tv: TypeVariable,
}

impl Environment {
    pub fn define(&mut self, path: Path, mut type_: Type) {
        self.freshen_type_variables(&mut type_, &HashSet::new());
        self.map.borrow_mut().insert(path, type_);
    }

    pub fn define_unknown(&mut self, path: Path) -> TypeVariable {
        let tv = self.new_tv();
        self.map.borrow_mut().insert(path, Type::Variable(tv));
        tv
    }

    pub fn type_constraint(&mut self, a: Type, b: Type, span: Span) {
        self.constraints.push(TypeConstraint(a, b))
    }

    fn new_tv(&mut self) -> TypeVariable {
        let tv = self.current_tv;
        self.current_tv += 1;
        tv
    }

    pub fn freshen_type_variables(&mut self, type_: &mut Type, free: &FreeVariableSet) {
        let free_vars = self.get_free_type_variables(free);
        freshen_type_variables(type_, &free_vars, || self.new_tv());
    }

    fn get_free_type_variables(&self, free: &FreeVariableSet) -> HashSet<TypeVariable> {
        let mut type_variables = HashSet::new();
        for var in free {
            self.map
                .borrow_mut()
                .get_mut(var)
                .unwrap()
                .visit(|tv: &mut TypeVariable| {
                    type_variables.insert(*tv);
                });
        }
        type_variables
    }

    pub fn get_type(&mut self, path: &Path, free: &FreeVariableSet) -> Type {
        let mut type_ = self.map.borrow().get(path).unwrap().clone();
        let mut type_map = HashMap::new();
        let free = self.get_free_type_variables(free);
        type_.visit(|type_var: &mut TypeVariable| {
            if !free.contains(type_var) {
                if let Some(replace) = type_map.get(type_var) {
                    *type_var = *replace;
                } else {
                    let replace = self.new_tv();
                    type_map.insert(*type_var, replace);
                    *type_var = replace;
                }
            }
        });
        type_
    }

    pub fn print_constraints(&self) {
        println!("CONSTRAINTS:\n{}", self.constraints.clone().sx());
    }
}

pub fn freshen_type_variables(
    type_: &mut Type,
    free: &HashSet<TypeVariable>,
    mut new_tv: impl FnMut() -> TypeVariable,
) {
    let mut type_map = HashMap::new();
    type_.visit(|type_var: &mut TypeVariable| {
        if !free.contains(type_var) {
            if let Some(replace) = type_map.get(type_var) {
                *type_var = *replace;
            } else {
                let replace = new_tv();
                type_map.insert(*type_var, replace);
                *type_var = replace;
            }
        }
    });
}

pub fn type_solve(module: IrModule) -> (IrModule, ModuleInterface) {
    let mut env = Environment::default();
    let mut free = HashSet::default();
    let mut interface = ModuleInterface::default();
    let module = module.infer(&mut env, &mut free);
    for item in &module.items {
        match item {
            ModuleItem::Let(pattern, _) => {
                pattern.clone().visit(|(path, type_)| {
                    interface.values.insert(path.clone(), type_.clone());
                });
            }
            ModuleItem::Type(path) => {}
            ModuleItem::Constructor(path, constructor) => {
                interface
                    .values
                    .insert(path.clone(), constructor.function_type());
            }
            ModuleItem::Import {
                path,
                type_,
                major,
                minor,
            } => {
                interface.values.insert(path.clone(), type_.clone().into());
            }
        }
    }
    (module, interface)
}
