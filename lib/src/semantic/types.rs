use sx::SXRepr;

use crate::{Visit, ir::Path, semantic::freshen_type_variables};

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

impl<T> sx::SXRepr for Typed<T>
where
    T: sx::SXRepr,
{
    fn sx(self) -> sx::SX {
        sx::SX::Field("type".into(), self.type_.sx().into()).push(self.inner.sx())
    }
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
    Variable(TypeVariable),
    /// Record type
    Struct {
        member_names: Vec<String>,
        member_types: Vec<Type>,
    },
    /// Tuple
    Product(Vec<Type>),
    /// Variant
    Sum {
        variant_names: Vec<String>,
        variant_types: Vec<Type>,
    },
    /// Function type
    Function(Box<Type>, Box<Type>),
    /// Placeholder until arrays are implemented, so I can
    /// generate `anyref` array in type section
    _ClosureCapture,
    Instantiation(Path, Vec<Type>),
}

#[derive(Debug, Clone)]
pub struct AbstractType {
    pub arity: usize,
    base: Type,
}

impl AbstractType {
    pub fn instantiate(mut self, parameters: &[Type]) -> Type {
        if parameters.len() != self.arity {
            panic!("Kindness error");
        }
        self.base.visit(|t: &mut Type| {
            if let Type::Variable(tv) = t {
                *t = parameters[*tv].clone()
            }
        });
        self.base
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

    pub fn modify_named_type(&mut self, path: Path, t: Type) {
        self.name_map.remove(&path);
        self.new_named_type(path, t);
    }

    pub fn get_named_type(&self, path: &Path) -> AbstractType {
        self.name_map
            .get(path)
            .unwrap_or_else(|| panic!("No named type exists: {path}"))
            .clone()
    }

    pub fn print() {
        println!(
            "{}",
            Self::get()
                .name_map
                .clone()
                .into_iter()
                .map(|(k, v)| (k, v.base))
                .collect::<Vec<_>>()
                .sx()
        );
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

    pub fn contains_type_variable(&self, tv: TypeVariable) -> bool {
        let mut ret = false;
        self.clone().visit(|t: &mut TypeVariable| {
            if *t == tv {
                ret = true;
            }
        });
        ret
    }
}

impl Visit<Type> for Type {
    fn _visit(&mut self, f: &mut impl FnMut(&mut Type)) {
        match self {
            Type::_ClosureCapture
            | Type::Any
            | Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph
            | Type::Variable(_) => f(self),
            Type::Instantiation(_, types) => {
                types._visit(f);
            }
            Type::Sum {
                variant_types: items,
                ..
            }
            | Type::Product(items)
            | Type::Struct {
                member_types: items,
                ..
            } => items._visit(f),
            Type::Function(a, b) => {
                a._visit(f);
                b._visit(f);
            }
        }
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
            Type::Sum {
                variant_names: names,
                variant_types: types,
            }
            | Type::Struct {
                member_names: names,
                member_types: types,
            } => {
                names.hash(state);
                types.hash(state);
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

impl sx::SXRepr for Type {
    fn sx(self) -> sx::SX {
        sx::SX::Atom(format!("{self}"))
    }
}
