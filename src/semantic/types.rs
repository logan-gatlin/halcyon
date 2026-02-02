use super::*;
use std::collections::HashMap;

use indexmap::IndexMap;

pub type TypeVariable = u64;

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
    fn with_type(
        self,
        t: Type,
    ) -> Typed<Self>;
}

impl<T> WithType for T {
    fn with_type(
        self,
        t: Type,
    ) -> Typed<T> {
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
    /// Product type
    Tuple(Vec<Type>),
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

pub fn freshen_type_variables<T, S>(
    t: &mut T,
    tv_source: S,
) where
    T: Visit<Type>,
    S: TypeVariableSource,
{
    let mut map = HashMap::new();
    t.visit(|t: &mut Type| {
        if let Type::Variable(t) = t {
            if let Some(tv) = map.get(t) {
                *t = *tv;
            } else {
                let tv = tv_source.fresh_tv();
                map.insert(*t, tv);
                *t = tv;
            }
        }
    })
}

pub fn freshen_type_variables_with_map<T, S>(
    t: &mut T,
    tv_source: S,
    map: &mut HashMap<TypeVariable, TypeVariable>,
) where
    T: Visit<Type>,
    S: TypeVariableSource,
{
    t.visit(|t: &mut Type| {
        if let Type::Variable(tv) = t {
            if let Some(new_tv) = map.get(tv) {
                *tv = *new_tv;
            } else {
                let new_tv = tv_source.fresh_tv();
                map.insert(*tv, new_tv);
                *tv = new_tv;
            }
        }
    })
}

pub fn substitute_type_variables<T: Visit<Type>>(
    t: &mut T,
    solution: &[Solution],
) {
    t.visit(|t: &mut Type| {
        for Solution { old, new } in solution {
            if let Type::Variable(tv) = t
                && *tv == *old
            {
                *t = new.clone();
            }
        }
    });
}

impl Type {
    pub fn curry(
        params: &[Type],
        returns: Type,
    ) -> Type {
        match params {
            [] => returns,
            [p] => Type::func(p.clone(), returns),
            [.., p] => Type::curry(&params[0..params.len() - 1], Type::func(p.clone(), returns)),
        }
    }
    pub fn func(
        parameter: Type,
        returns: Type,
    ) -> Type {
        Type::Function(parameter.into(), returns.into())
    }
    pub fn always_contains_type_variable(
        &self,
        tv: TypeVariable,
    ) -> bool {
        match self {
            Type::Any
            | Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph => false,
            // Sum types can be recursive without being infinite, such as with linked lists:
            // ```
            // type LL = fn a => | Cons (LL a) | Nil
            // ```
            // So long as at least one variant does not contain itself, the type is constructable.
            Type::Sum { variant_types, .. } => {
                variant_types
                    .iter()
                    .all(|t| t.always_contains_type_variable(tv))
            }
            Type::Variable(t) => *t == tv,
            Type::Array(t) => t.always_contains_type_variable(tv),
            Type::Struct { fields, .. } => {
                fields.values().any(|t| t.always_contains_type_variable(tv))
            }
            Type::Tuple(items) => items.iter().any(|t| t.always_contains_type_variable(tv)),
            Type::Function(a, b) => {
                a.always_contains_type_variable(tv) || b.always_contains_type_variable(tv)
            }
            Type::Instantiation(_, items) => {
                items.iter().any(|t| t.always_contains_type_variable(tv))
            }
        }
    }
}

impl Visit<Type> for Type {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut Type),
    ) {
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
            | Type::Tuple(items) => items._visit(f),
            Type::Struct { fields, .. } => fields.values_mut().for_each(|v| v._visit(f)),
            Type::Function(a, b) => {
                a._visit(f);
                b._visit(f);
            }
        }
        f(self);
    }
}

impl Visit<TypeVariable> for Type {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut TypeVariable),
    ) {
        self._visit(&mut |t: &mut Type| {
            if let Type::Variable(tv) = t {
                f(tv);
            }
        })
    }
}

impl PartialEq for Type {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        use Type as t;
        match (self, other) {
            (t::Any, t::Any) => {
                unreachable!("Tried to compare ambiguous types")
            }
            (t::Unit, t::Unit)
            | (t::Integer, t::Integer)
            | (t::Real, t::Real)
            | (t::Boolean, t::Boolean)
            | (t::Glyph, t::Glyph)
            | (t::String, t::String) => true,
            (t::Struct { name: name1, .. }, t::Struct { name: name2, .. }) => name1 == name2,
            (t::Function(p1, r1), t::Function(p2, r2)) => p1 == p2 && r1 == r2,
            (t::Tuple(t1), t::Tuple(t2)) => t1 == t2,
            (t::Sum { name: name1, .. }, t::Sum { name: name2, .. }) => name1 == name2,
            (t::Variable(t1), t::Variable(t2)) => t1 == t2,
            (t::Instantiation(name1, types1), t::Instantiation(name2, types2)) => {
                name1 == name2 && types1 == types2
            }
            _ => false,
        }
    }
}

impl Eq for Type {
}

impl std::hash::Hash for Type {
    fn hash<H: std::hash::Hasher>(
        &self,
        state: &mut H,
    ) {
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
            Type::Tuple(items) => {
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
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Type::Any => write!(f, "any"),
            Type::Unit => write!(f, "unit"),
            Type::Integer => write!(f, "integer"),
            Type::Real => write!(f, "real"),
            Type::Boolean => write!(f, "boolean"),
            Type::String => write!(f, "string"),
            Type::Glyph => write!(f, "glyph"),
            Type::Array(t) => write!(f, "[{t}]"),
            Type::Struct { name, .. } => {
                write!(f, "{name}")
            }
            Type::Sum {
                variant_names,
                variant_types,
                ..
            } => {
                write!(
                    f,
                    "| {}",
                    variant_names
                        .iter()
                        .zip(variant_types)
                        .map(|(n, t)| {
                            if t == &Type::Unit {
                                n.to_string()
                            } else {
                                format!("{n} {t}")
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" | ")
                )
            }
            Type::Function(a, b) => write!(f, "({a} -> {b})"),
            Type::Variable(id) => write!(f, "'{id}"),
            Type::Tuple(items) => {
                write!(
                    f,
                    "({})",
                    items
                        .iter()
                        .map(|i| format!("{}", i))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
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
