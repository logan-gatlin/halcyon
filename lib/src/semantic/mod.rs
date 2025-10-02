/*!
    The semantic module infers and checks types. To do this, we use a variation
    of the Hindley Milner system. The `infer` module gives the program temporary
    type variables, and generates a set of constraints for those variables.
    The `constraint` module solves those constraints, and generates a solution.
    A solution is a mapping from type variables to concrete types.
*/
mod constraint;
mod infer;
mod types;
use std::collections::{HashMap, HashSet};

use crate::{Logger, Span, Visit};
pub use constraint::*;
pub use infer::*;
use sx::SXRepr;
pub use types::*;

use crate::ir::*;

/// The set of exports provided for a module
#[derive(Debug, Clone, Default, sx::SXRepr)]
pub struct ModuleInterface {
    /// Type definitions, i.e. `type t = ...`
    pub types: HashSet<Path>,
    /// Value definitions, i.e. `let foo = ...`
    pub values: HashMap<Path, Type>,
    /// Constructors, i.e. `type enum = Foo | Bar | Baz`
    pub constructors: HashMap<Path, Constructor>,
}

impl ModuleInterface {
    /// Add every definition in `other` to `self`. In the case of conflicts,
    /// keeps the value in `other`.
    pub fn merge(&mut self, other: Self) {
        for type_ in other.types {
            self.types.insert(type_);
        }
        for (path, value) in other.values {
            self.values.insert(path, value);
        }
        for (path, cons) in other.constructors {
            self.constructors.insert(path, cons);
        }
    }
}

/// The set of type variables that are free in the current environment.
/// The difference between free and non-free variables is that free variables
/// refer to *one* type that is yet to be determined. Non-free variables are generic,
/// they may refer to a different type every time they are used.
pub type FreeVariableSet = HashSet<TypeVariable>;

/// The type environmnet contains all of the state needed for type inference.
/// Once an item has been fully type inferred, the environment passes its
/// constraints to the solver.
#[derive(Debug, Clone, Default)]
pub struct Environment {
    map: HashMap<Path, Type>,
    constraints: Vec<TypeConstraint>,
    struct_constraints: Vec<StructConstraint>,
    current_tv: TypeVariable,
}

impl Environment {
    pub fn define(&mut self, path: Path, mut type_: Type) {
        self.freshen_type_variables(&mut type_, &HashSet::new());
        self.map.insert(path, type_);
    }

    pub fn define_unknown(&mut self, path: Path) -> TypeVariable {
        let tv = self.new_tv();
        self.map.insert(path, Type::Variable(tv));
        tv
    }

    pub fn type_constraint(&mut self, a: Type, b: Type, span: Span) {
        self.constraints.push(TypeConstraint(a, b, span))
    }

    pub fn struct_constraint(&mut self, of_t: Type, field_t: Type, name: String, span: Span) {
        self.struct_constraints.push(StructConstraint {
            of_t,
            field_t,
            name,
            span,
        })
    }

    fn new_tv(&mut self) -> TypeVariable {
        let tv = self.current_tv;
        self.current_tv += 1;
        tv
    }

    pub fn freshen_type_variables(&mut self, type_: &mut impl Visit<Type>, free: &FreeVariableSet) {
        freshen_type_variables(type_, &free, || self.new_tv());
    }

    pub fn get_type(&mut self, path: &Path, free: &FreeVariableSet) -> Type {
        let mut type_ = self.map.get(path).cloned().unwrap_or(Type::Any);
        let mut type_map = HashMap::new();
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

    #[allow(unused)]
    pub fn print_constraints(&self) {
        println!("CONSTRAINTS:\n{}", self.constraints.clone().sx());
    }
}

pub fn freshen_type_variables(
    type_: &mut impl Visit<Type>,
    free: &HashSet<TypeVariable>,
    mut new_tv: impl FnMut() -> TypeVariable,
) {
    let mut type_map = HashMap::new();
    type_.visit(|type_| {
        if let Type::Variable(type_var) = type_ {
            if !free.contains(type_var) {
                if let Some(replace) = type_map.get(type_var) {
                    *type_var = *replace;
                } else {
                    let replace = new_tv();
                    type_map.insert(*type_var, replace);
                    *type_var = replace;
                }
            }
        }
    });
}

#[allow(unused)]
pub fn normalize_type_variables(t: &mut impl Visit<Type>) {
    let mut count = 0;
    let mut map = HashMap::new();
    t.visit(|type_: &mut Type| {
        if let Type::Variable(type_var) = type_ {
            if let Some(replace) = map.get(type_var) {
                *type_var = *replace;
            } else {
                let replace = count;
                count += 1;
                map.insert(*type_var, replace);
                *type_var = replace;
            }
        }
    });
}

pub fn type_solve(logger: &mut Logger, mut module: IrModule) -> (IrModule, ModuleInterface) {
    let mut interface = ModuleInterface::default();
    let mut env = Environment::default();
    let free = HashSet::default();
    module.items = module
        .items
        .into_iter()
        .map(|i| match i {
            ModuleItem::Let(pattern, node) => {
                let mut new_env = env.clone();
                let mut new_free = free.clone();
                let mut pattern = pattern.infer(&mut new_env, &mut new_free);
                let mut node = node.infer(&mut new_env, &mut new_free);
                new_env.type_constraint(pattern.type_.clone(), node.type_.clone(), pattern.span);
                let solution = new_env.solve_constraints(logger);
                unify_all(&mut pattern, &solution);
                unify_all(&mut node, &solution);
                pattern.visit(|(path, type_)| {
                    env.map.insert(path.clone(), type_.clone());
                    interface.values.insert(path.clone(), type_.clone());
                });
                //normalize_type_variables(&mut node);
                ModuleItem::Let(pattern, node)
            }
            ModuleItem::Type(path) => {
                interface.types.insert(path.clone());
                ModuleItem::Type(path)
            }
            ModuleItem::Constructor(path, constructor) => {
                let type_ = match constructor.kind.clone() {
                    ConstructorKind::Unitary(t) => t,
                    ConstructorKind::Function(a, b) => Type::func(a, b),
                };
                env.define(path.clone(), type_.clone());
                interface.values.insert(path.clone(), type_);
                interface
                    .constructors
                    .insert(path.clone(), constructor.clone());
                ModuleItem::Constructor(path, constructor)
            }
            ModuleItem::Import {
                path,
                type_,
                major,
                minor,
            } => {
                env.define(path.clone(), type_.clone().into());
                interface.values.insert(path.clone(), type_.clone().into());
                ModuleItem::Import {
                    path,
                    type_,
                    major,
                    minor,
                }
            }
        })
        .collect();
    (module, interface)
}
