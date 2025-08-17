use crate::{ir::Path, semantic::Unify};

use super::TypeVariable;
use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, OnceLock},
};

pub type TypeRef = Type;

#[derive(Debug, Clone)]
pub struct TypeScheme {
    variables: usize,
    inner: Type,
}

impl TypeScheme {
    pub fn new(mut t: Type) -> Self {
        let mut map = HashMap::new();
        let mut variables = 0;
        t.map_new_type_variables(&mut map, &mut variables);
        Self {
            variables,
            inner: t,
        }
    }

    pub fn instantiate(mut self, mut fresh_type_var: impl FnMut() -> TypeVariable) -> Type {
        for i in 0..self.variables {
            self.inner.unify(i, &Type::TypeVariable(fresh_type_var()))
        }
        self.inner
    }

    pub fn instantiate_default(self) -> Type {
        let mut count = 0;
        self.instantiate(|| {
            let c = count;
            count += 1;
            c
        })
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    /// Indeterminate type
    Any,
    /// The empty type ()
    Unit,
    /// Signed 64 bit integer
    Integer,
    /// IEEE 64 bit floating point
    Real,
    /// true or false
    Boolean,
    /// Fat pointer to byte array of UTF-8
    String,
    /// UTF-8 codepoint 32 bit
    Glyph,
    // Type variable
    TypeVariable(TypeVariable),
    /// Record type
    Struct {
        member_names: Vec<String>,
        member_types: Vec<TypeRef>,
    },
    /// Tuple
    Product(Vec<TypeRef>),
    /// Variant
    Sum {
        variant_names: Vec<String>,
        variant_types: Vec<TypeRef>,
    },
    /// Function type
    Function(Box<TypeRef>, Box<TypeRef>),
    /// Placeholder until arrays are implemented, so I can
    /// generate `anyref` array in type section
    _ClosureCapture,
    Named(Path),
}

static UNIVERSE: OnceLock<Mutex<HashMap<Path, Type>>> = OnceLock::new();

fn get_universe() -> std::sync::MutexGuard<'static, HashMap<Path, Type>> {
    UNIVERSE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
}

impl Type {
    pub fn new_named_type(path: Path, t: Type) {
        let mut guard = get_universe();
        guard.insert(path.clone(), t);
    }

    pub fn get_named_type(path: &Path) -> Type {
        let guard = get_universe();
        guard
            .get(path)
            .unwrap_or_else(|| panic!("No named type: {path}"))
            .clone()
    }

    pub fn find_structs_with_fields(fieldset: &HashSet<(String, Type)>) -> Vec<Type> {
        let u = get_universe();
        u.iter()
            .flat_map(|(path, t)| match t {
                Type::Struct { member_names, .. } => {
                    if fieldset.iter().all(|name| member_names.contains(&name.0)) {
                        Some(Type::Named(path.clone()))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    }

    pub fn map_new_type_variables(
        &mut self,
        map: &mut HashMap<TypeVariable, TypeVariable>,
        current: &mut TypeVariable,
    ) {
        match self {
            Type::Named(_)
            | Type::Any
            | Type::_ClosureCapture
            | Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph => {}
            Type::TypeVariable(tv) => {
                if let Some(key) = map.get(tv) {
                    *tv = *key;
                } else {
                    let new_tv = *current;
                    *current += 1;
                    map.insert(*tv, new_tv);
                    *tv = new_tv;
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
                .iter_mut()
                .for_each(|t| t.map_new_type_variables(map, current)),
            Type::Function(param_type, return_type) => {
                param_type.map_new_type_variables(map, current);
                return_type.map_new_type_variables(map, current);
            }
        }
    }

    pub fn curry(params: &[Type], returns: Type) -> Type {
        match params {
            [] => returns,
            [p] => Type::func(p.clone(), returns),
            [.., p] => Type::curry(&params[0..params.len() - 1], Type::func(p.clone(), returns)),
        }
    }

    pub fn primitives() -> Vec<(TypeRef, &'static str)> {
        vec![
            (Self::Unit, "unit"),
            (Self::Integer, "integer"),
            (Self::Real, "real"),
            (Self::Boolean, "boolean"),
            (Self::String, "string"),
            (Self::Glyph, "glyph"),
        ]
    }

    pub fn func(parameter: Type, returns: Type) -> TypeRef {
        Type::Function(parameter.into(), returns.into())
    }

    pub fn field_index(&self, name: &str) -> Option<u32> {
        let t = if let Type::Named(name) = self {
            &Self::get_named_type(name)
        } else {
            self
        };
        if let Type::Struct { member_names, .. } = t {
            let mut index = 0;
            let mut found = false;
            for n in member_names.iter() {
                if n == name {
                    found = true;
                    break;
                }
                index += 1;
            }
            if found { Some(index) } else { None }
        } else {
            None
        }
    }

    pub fn field_type(&self, name: &str) -> Option<Type> {
        let t = if let Type::Named(name) = self {
            &Self::get_named_type(name)
        } else {
            self
        };
        if let Type::Struct {
            member_names,
            member_types,
        } = t
        {
            let pos = member_names.iter().position(|n| n == name)?;
            Some(member_types[pos].clone())
        } else {
            None
        }
    }

    pub fn ambiguous(&self) -> bool {
        matches!(self, Self::Any)
    }

    pub fn strict_eq(&self, other: &Type) -> bool {
        use Type as t;
        match (self, other) {
            (t::Any, t::Any) => {
                panic!("Tried to compare ambiguous types")
            }
            (t::_ClosureCapture, t::_ClosureCapture)
            | (t::Unit, t::Unit)
            | (t::Integer, t::Integer)
            | (t::Real, t::Real)
            | (t::Boolean, t::Boolean)
            | (t::Glyph, t::Glyph)
            | (t::String, t::String) => true,
            (
                t::Struct {
                    member_names: names1,
                    member_types: types1,
                },
                t::Struct {
                    member_names: names2,
                    member_types: types2,
                },
            ) => names1 == names2 && types1 == types2,
            (t::Function(p1, r1), t::Function(p2, r2)) => p1 == p2 && r1 == r2,
            (t::Product(t1), t::Product(t2)) => t1 == t2,
            (
                t::Sum {
                    variant_names: n1,
                    variant_types: t1,
                },
                t::Sum {
                    variant_names: n2,
                    variant_types: t2,
                },
            ) => t1 == t2 && n1 == n2,
            (t::TypeVariable(t1), t::TypeVariable(t2)) => t1 == t2,
            (t::Named(a), t::Named(b)) => a == b,
            (t::Named(n), a) | (a, t::Named(n)) => {
                let b = Self::get_named_type(n);

                a == &b
            }
            _ => false,
        }
    }

    pub fn contains_type_var(&self, tv: TypeVariable) -> bool {
        match self {
            Type::TypeVariable(t) => tv == *t,
            Type::Struct { member_types, .. } => {
                member_types.iter().any(|x| x.contains_type_var(tv))
            }
            Type::Product(items) => items.iter().any(|x| x.contains_type_var(tv)),
            Type::Sum { variant_types, .. } => {
                variant_types.iter().any(|x| x.contains_type_var(tv))
            }
            Type::Function(a, b) => a.contains_type_var(tv) || b.contains_type_var(tv),
            _ => false,
        }
    }

    pub fn product(a: Type, b: Type) -> TypeRef {
        if let Type::Product(v) = a {
            let mut new = v.clone();
            if let Type::Product(v2) = b {
                new.append(&mut v2.clone());
                Type::Product(new)
            } else {
                new.push(b.clone());
                Type::Product(new)
            }
        } else if let Type::Product(v) = b {
            let mut new = v.clone();
            if let Type::Product(v2) = a {
                new.append(&mut v2.clone());
                Type::Product(new)
            } else {
                new.push(a.clone());
                Type::Product(new)
            }
        } else {
            Type::Product(vec![a.clone(), b.clone()])
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        Self::Any
    }
}

impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use Type::*;
        use std::cmp::Ordering::*;
        Some(match (self, other) {
            (Any, _) | (_, Any) => return None,
            (TypeVariable(_), TypeVariable(_)) => Equal,
            (TypeVariable(_), _) => Greater,
            (_, TypeVariable(_)) => Less,
            (t1, t2) if t1 == t2 => Equal,
            _ => return None,
        })
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        use Type as t;
        match (self, other) {
            (t::Any, t::Any) => {
                panic!("Tried to compare ambiguous types")
            }
            (t::_ClosureCapture, t::_ClosureCapture)
            | (t::Unit, t::Unit)
            | (t::Integer, t::Integer)
            | (t::Real, t::Real)
            | (t::Boolean, t::Boolean)
            | (t::Glyph, t::Glyph)
            | (t::String, t::String) => true,
            (
                t::Struct {
                    member_names: names1,
                    member_types: types1,
                },
                t::Struct {
                    member_names: names2,
                    member_types: types2,
                },
            ) => names1 == names2 && types1 == types2,
            (t::Function(p1, r1), t::Function(p2, r2)) => p1 == p2 && r1 == r2,
            (t::Product(t1), t::Product(t2)) => t1 == t2,
            (
                t::Sum {
                    variant_names: n1,
                    variant_types: t1,
                },
                t::Sum {
                    variant_names: n2,
                    variant_types: t2,
                },
            ) => t1 == t2 && n1 == n2,
            (t::TypeVariable(_), t::TypeVariable(_)) => true,
            (t::Named(a), t::Named(b)) => a == b,
            (t::Named(n), a) | (a, t::Named(n)) => {
                let b = Self::get_named_type(n);

                a == &b
            }
            _ => false,
        }
    }
}

impl Eq for Type {}

impl std::hash::Hash for Type {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Type::Sum {
                variant_names: names,
                variant_types: types,
            }
            | Type::Struct {
                member_names: names,
                member_types: types,
            } => {
                names.hash(state);
                types.iter().for_each(|t| {
                    t.hash(state);
                })
            }
            Type::Function(a, b) => {
                a.hash(state);
                b.hash(state);
            }
            Type::Any => {
                "any".hash(state);
            }
            Type::TypeVariable(id) => {
                "poly".hash(state);
                id.hash(state);
            }
            Type::Product(items) => {
                "tuple".hash(state);
                for item in items {
                    item.hash(state);
                }
            }
            Type::Named(_)
            | Type::Unit
            | Type::_ClosureCapture
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph => {
                format!("{self}").hash(state);
            }
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Any => write!(f, "any"),
            Type::_ClosureCapture => write!(f, "capture"),
            Type::Unit => write!(f, "unit"),
            Type::Integer => write!(f, "integer"),
            Type::Real => write!(f, "real"),
            Type::Boolean => write!(f, "boolean"),
            Type::String => write!(f, "string"),
            Type::Glyph => write!(f, "glyph"),
            Type::Struct {
                member_names,
                member_types,
            } => {
                let fields = member_names
                    .iter()
                    .zip(member_types)
                    .map(|(name, type_)| format!("{name}: {type_}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{{ {fields} }}")
            }
            Type::Function(a, b) => write!(f, "({a} -> {b})"),
            Type::TypeVariable(id) => write!(f, "'{id}"),
            Type::Product(items) => write!(
                f,
                "({})",
                items
                    .iter()
                    .map(|i| format!("{}", i))
                    .collect::<Vec<_>>()
                    .join(" * ")
            ),
            Type::Sum {
                variant_names,
                variant_types,
            } => write!(
                f,
                "{}",
                variant_names
                    .iter()
                    .zip(variant_types)
                    .map(|(name, type_)| format!("{name} of {type_}"))
                    .collect::<Vec<_>>()
                    .join(" | ")
            ),
            Type::Named(name) => {
                write!(f, "{name}")
            }
        }
    }
}
