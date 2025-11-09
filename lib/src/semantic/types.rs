use indexmap::IndexMap;

use crate::{LResult, Log, Visit, err, ir2::Path, semantic::freshen_type_variables};

pub type TypeVariable = usize;

use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, OnceLock},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Typed<T> {
    pub inner: T,
    pub type_: Type,
}

impl<T> std::ops::Deref for Typed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for Typed<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait WithType: Sized {
    fn with_type(self, t: Type) -> Typed<Self>;
}

impl<T> WithType for T {
    fn with_type(self, t: Type) -> Typed<T> {
        Typed {
            inner: self,
            type_: t,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum Type {
    /// Indeterminate type
    #[default]
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
    Variable(TypeVariable),
    /// Record type
    Struct {
        name: Path,
        fields: IndexMap<String, Type>,
    },
    /// Array type
    Array(Box<Type>),
    /// Tuple
    Product(Vec<Type>),
    /// Variant
    Sum {
        name: Path,
        variant_names: Vec<String>,
        variant_types: Vec<Type>,
    },
    /// Function type
    Function(Box<Type>, Box<Type>),
    Instantiation(Path, Vec<Type>),
}

pub fn partial_instantiation_error(expects: usize, received: usize) -> Log {
    err(format!(
        "This type has {expects} parameters, but {received} parameters were provided. Partial instantiation is not allowed."
    ))
}

#[derive(Debug, Clone)]
pub struct AbstractType {
    pub arity: usize,
    base: Type,
}

impl AbstractType {
    pub fn instantiate(mut self, parameters: &[Type]) -> LResult<Type> {
        if parameters.len() != self.arity {
            return Err(partial_instantiation_error(self.arity, parameters.len()));
        }
        self.base.visit(|t: &mut Type| {
            if let Type::Variable(tv) = t {
                *t = parameters[*tv].clone()
            }
        });
        Ok(self.base)
    }

    pub fn instantiate_with(self, mut f: impl FnMut() -> TypeVariable) -> LResult<Type> {
        let parameters = vec![Type::Variable(f()); self.arity];
        self.instantiate(&parameters)
    }
}

static UNIVERSE: OnceLock<Mutex<Universe>> = OnceLock::new();

#[derive(Debug, Clone, Default)]
pub struct Universe {
    name_map: HashMap<Path, AbstractType>,
}

impl Universe {
    fn new() -> Self {
        Self::default()
    }

    pub fn get() -> std::sync::MutexGuard<'static, Self> {
        UNIVERSE
            .get_or_init(|| Mutex::new(Universe::new()))
            .lock()
            .unwrap()
    }

    pub fn new_named_type(&mut self, path: Path, mut t: Type) {
        let mut type_variables = HashSet::new();
        t.visit(|tv: &mut TypeVariable| {
            type_variables.insert(*tv);
        });
        let mut arity = 0;
        freshen_type_variables(&mut t, &HashSet::new(), || {
            let old = arity;
            arity += 1;
            old
        });
        self.name_map
            .insert(path.clone(), AbstractType { arity, base: t });
    }

    pub fn get_named_type(&self, path: &Path) -> AbstractType {
        self.name_map
            .get(path)
            .unwrap_or_else(|| panic!("No named type exists: {path}"))
            .clone()
    }

    pub fn find_struct_with_names(&self, names: &HashSet<String>) -> Vec<AbstractType> {
        todo!()
    }
}

impl Type {
    pub fn curry(params: &[Type], returns: Type) -> Type {
        match params {
            [] => returns,
            [p] => Type::func(p.clone(), returns),
            [.., p] => Type::curry(&params[0..params.len() - 1], Type::func(p.clone(), returns)),
        }
    }

    pub fn func(parameter: Type, returns: Type) -> Type {
        Type::Function(parameter.into(), returns.into())
    }

    pub fn field_type(&self, name: &str) -> Option<Type> {
        todo!()
    }

    pub fn contains_type_variable(&self, tv: TypeVariable) -> bool {
        match self {
            Type::Any
            | Type::Sum { .. }
            | Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph => false,
            Type::Variable(t) => *t == tv,
            Type::Array(t) => t.contains_type_variable(tv),
            Type::Struct { fields, .. } => fields.values().any(|t| t.contains_type_variable(tv)),
            Type::Product(items) => items.iter().any(|t| t.contains_type_variable(tv)),
            Type::Function(a, b) => a.contains_type_variable(tv) || b.contains_type_variable(tv),
            Type::Instantiation(_, items) => items.iter().any(|t| t.contains_type_variable(tv)),
        }
    }
}

impl Visit<Type> for Type {
    fn _visit(&mut self, f: &mut impl FnMut(&mut Type)) {
        match self {
            Type::Any
            | Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph
            | Type::Variable(_) => {}
            Type::Instantiation(_, types) => {
                types._visit(f);
            }
            Type::Array(t) => t._visit(f),
            Type::Sum {
                variant_types: items,
                ..
            }
            | Type::Product(items) => items._visit(f),
            Type::Struct { fields, .. } => fields.values_mut().for_each(|v| v._visit(f)),
            Type::Function(a, b) => {
                a._visit(f);
                b._visit(f);
            }
        }
        f(self)
    }
}

impl Visit<TypeVariable> for Type {
    fn _visit(&mut self, f: &mut impl FnMut(&mut TypeVariable)) {
        self._visit(&mut |t: &mut Type| {
            if let Type::Variable(tv) = t {
                f(tv);
            }
        })
    }
}

impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use Type::*;
        use std::cmp::Ordering::*;
        Some(match (self, other) {
            (Any, _) | (_, Any) => return None,
            (Variable(_), Variable(_)) => Equal,
            (Variable(_), _) => Greater,
            (_, Variable(_)) => Less,
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
            (t::Unit, t::Unit)
            | (t::Integer, t::Integer)
            | (t::Real, t::Real)
            | (t::Boolean, t::Boolean)
            | (t::Glyph, t::Glyph)
            | (t::String, t::String) => true,
            (t::Struct { name: name1, .. }, t::Struct { name: name2, .. }) => name1 == name2,
            (t::Function(p1, r1), t::Function(p2, r2)) => p1 == p2 && r1 == r2,
            (t::Product(t1), t::Product(t2)) => t1 == t2,
            (t::Sum { name: name1, .. }, t::Sum { name: name2, .. }) => name1 == name2,
            (t::Variable(t1), t::Variable(t2)) => t1 == t2,
            (t::Instantiation(name1, types1), t::Instantiation(name2, types2)) => {
                name1 == name2 && types1 == types2
            }
            _ => false,
        }
    }
}

impl Eq for Type {}

impl std::hash::Hash for Type {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Type::Sum { name, .. } | Type::Struct { name, .. } => {
                name.hash(state);
            }
            Type::Array(t) => {
                "array".hash(state);
                t.hash(state);
            }
            Type::Function(a, b) => {
                "function".hash(state);
                a.hash(state);
                b.hash(state);
            }
            Type::Any => {
                "any".hash(state);
            }
            Type::Variable(id) => {
                "poly".hash(state);
                id.hash(state);
            }
            Type::Product(items) => {
                "tuple".hash(state);
                for item in items {
                    item.hash(state);
                }
            }
            Type::Instantiation(name, types) => {
                name.hash(state);
                types.hash(state);
            }
            Type::Unit
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
            Type::Unit => write!(f, "unit"),
            Type::Integer => write!(f, "integer"),
            Type::Real => write!(f, "real"),
            Type::Boolean => write!(f, "boolean"),
            Type::String => write!(f, "string"),
            Type::Glyph => write!(f, "glyph"),
            Type::Array(t) => write!(f, "[{t}]"),
            Type::Sum { name, .. } | Type::Struct { name, .. } => {
                write!(f, "{name}")
            }
            Type::Function(a, b) => write!(f, "({a} -> {b})"),
            Type::Variable(id) => write!(f, "'{id}"),
            Type::Product(items) => write!(
                f,
                "({})",
                items
                    .iter()
                    .map(|i| format!("{}", i))
                    .collect::<Vec<_>>()
                    .join(" * ")
            ),
            Type::Instantiation(name, types) if types.is_empty() => {
                write!(f, "{name}")
            }
            Type::Instantiation(name, types) => {
                write!(
                    f,
                    "{name} {}",
                    types
                        .iter()
                        .map(|t| format!("{t}"))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
        }
    }
}
