use indexmap::IndexMap;

pub type TypeParameterIndex = u32;
pub type RecursionIndex = u32;
pub type MetaVarId = u32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeName {
    pub module: String,
    pub name: String,
}

impl TypeName {
    pub fn new(
        module: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            module: module.into(),
            name: name.into(),
        }
    }
}

impl std::fmt::Display for TypeName {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        if self.module.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}::{}", self.module, self.name)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructMatch {
    Exact,
    AtLeast,
}

#[derive(Debug, Clone, Default)]
pub enum Type {
    /// The empty type ()
    #[default]
    Unit,
    /// Signed 64 bit integer
    Integer,
    /// IEEE 64 bit floating point
    Real,
    /// true or false
    Boolean,
    /// Pointer + length for byte array of UTF-8
    String,
    /// UTF-8 codepoint 32 bit
    Glyph,
    /// Bound type parameter (De Bruijn index; 0 is innermost ForAll)
    TypeVar(TypeParameterIndex),
    /// Inference-time meta type variable
    MetaVar(MetaVarId),
    /// Bound recursive reference (De Bruijn index; 0 is innermost Mu)
    RecVar(RecursionIndex),
    /// Universal type binder for parameters
    ForAll(Box<Type>),
    /// Recursive type binder
    Mu(Box<Type>),
    /// Nominal type definition
    Named { name: TypeName, body: Box<Type> },
    /// Structural constraint for named structs
    StructConstraint {
        fields: IndexMap<String, Type>,
        mode: StructMatch,
    },
    /// Record type
    Struct { fields: IndexMap<String, Type> },
    /// Array type
    Array(Box<Type>),
    /// Product type
    Tuple(Vec<Type>),
    /// Variant
    Sum {
        variant_names: Vec<String>,
        variant_types: Vec<Type>,
    },
    /// Function type
    Function(Box<Type>, Box<Type>),
    /// Apply a polymorphic type to arguments
    Apply {
        constructor: Box<Type>,
        arguments: Vec<Type>,
    },
}

impl PartialEq for Type {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        use Type::*;
        match (self, other) {
            (Unit, Unit)
            | (Integer, Integer)
            | (Real, Real)
            | (Boolean, Boolean)
            | (String, String)
            | (Glyph, Glyph) => true,
            (TypeVar(left), TypeVar(right)) => left == right,
            (MetaVar(left), MetaVar(right)) => left == right,
            (RecVar(left), RecVar(right)) => left == right,
            (ForAll(left), ForAll(right)) => left == right,
            (Mu(left), Mu(right)) => left == right,
            (Named { name: left, .. }, Named { name: right, .. }) => left == right,
            (
                StructConstraint {
                    fields: left,
                    mode: left_mode,
                },
                StructConstraint {
                    fields: right,
                    mode: right_mode,
                },
            ) => left == right && left_mode == right_mode,
            (Struct { fields: left }, Struct { fields: right }) => left == right,
            (Array(left), Array(right)) => left == right,
            (Tuple(left), Tuple(right)) => left == right,
            (
                Sum {
                    variant_names: left_names,
                    variant_types: left_types,
                },
                Sum {
                    variant_names: right_names,
                    variant_types: right_types,
                },
            ) => left_names == right_names && left_types == right_types,
            (Function(left_param, left_result), Function(right_param, right_result)) => {
                left_param == right_param && left_result == right_result
            }
            (
                Apply {
                    constructor: left_constructor,
                    arguments: left_arguments,
                },
                Apply {
                    constructor: right_constructor,
                    arguments: right_arguments,
                },
            ) => left_constructor == right_constructor && left_arguments == right_arguments,
            (
                Apply {
                    constructor,
                    arguments,
                },
                other,
            )
            | (
                other,
                Apply {
                    constructor,
                    arguments,
                },
            ) if arguments.is_empty() => constructor.as_ref() == other,
            _ => false,
        }
    }
}

impl Eq for Type {
}

impl Type {
    pub fn shift_type_vars(
        &self,
        amount: i32,
        cutoff: TypeParameterIndex,
    ) -> Option<Self> {
        self.shift_type_vars_with_cutoff(amount, cutoff)
    }

    pub fn shift_rec_vars(
        &self,
        amount: i32,
        cutoff: RecursionIndex,
    ) -> Option<Self> {
        self.shift_rec_vars_with_cutoff(amount, cutoff)
    }

    pub fn substitute_type_var(
        &self,
        index: TypeParameterIndex,
        replacement: &Type,
    ) -> Option<Self> {
        self.substitute_type_var_with_depth(index, replacement, 0)
    }

    pub fn substitute_rec_var(
        &self,
        index: RecursionIndex,
        replacement: &Type,
    ) -> Option<Self> {
        self.substitute_rec_var_with_depth(index, replacement, 0)
    }

    pub fn pretty(&self) -> String {
        self.pretty_with_context(&[], &[])
    }

    fn shift_type_vars_with_cutoff(
        &self,
        amount: i32,
        cutoff: TypeParameterIndex,
    ) -> Option<Self> {
        let shift_index = |index: TypeParameterIndex| {
            if index < cutoff {
                Some(index)
            } else {
                shift_index(index, amount)
            }
        };
        match self {
            Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph => Some(self.clone()),
            Type::TypeVar(index) => shift_index(*index).map(Type::TypeVar),
            Type::MetaVar(index) => Some(Type::MetaVar(*index)),
            Type::RecVar(index) => Some(Type::RecVar(*index)),
            Type::ForAll(body) => {
                body.shift_type_vars_with_cutoff(amount, cutoff + 1)
                    .map(|body| Type::ForAll(Box::new(body)))
            }
            Type::Mu(body) => {
                body.shift_type_vars_with_cutoff(amount, cutoff)
                    .map(|body| Type::Mu(Box::new(body)))
            }
            Type::Named { name, body } => {
                Some(Type::Named {
                    name: name.clone(),
                    body: body.clone(),
                })
            }
            Type::StructConstraint { fields, mode } => {
                fields
                    .iter()
                    .map(|(name, type_)| {
                        type_
                            .shift_type_vars_with_cutoff(amount, cutoff)
                            .map(|type_| (name.clone(), type_))
                    })
                    .collect::<Option<IndexMap<_, _>>>()
                    .map(|fields| {
                        Type::StructConstraint {
                            fields,
                            mode: *mode,
                        }
                    })
            }
            Type::Struct { fields } => {
                fields
                    .iter()
                    .map(|(name, type_)| {
                        type_
                            .shift_type_vars_with_cutoff(amount, cutoff)
                            .map(|type_| (name.clone(), type_))
                    })
                    .collect::<Option<IndexMap<_, _>>>()
                    .map(|fields| Type::Struct { fields })
            }
            Type::Array(inner) => {
                inner
                    .shift_type_vars_with_cutoff(amount, cutoff)
                    .map(|inner| Type::Array(Box::new(inner)))
            }
            Type::Tuple(items) => {
                items
                    .iter()
                    .map(|item| item.shift_type_vars_with_cutoff(amount, cutoff))
                    .collect::<Option<Vec<_>>>()
                    .map(Type::Tuple)
            }
            Type::Sum {
                variant_names,
                variant_types,
            } => {
                variant_types
                    .iter()
                    .map(|variant| variant.shift_type_vars_with_cutoff(amount, cutoff))
                    .collect::<Option<Vec<_>>>()
                    .map(|variant_types| {
                        Type::Sum {
                            variant_names: variant_names.clone(),
                            variant_types,
                        }
                    })
            }
            Type::Function(parameter, result) => {
                let parameter = parameter.shift_type_vars_with_cutoff(amount, cutoff)?;
                let result = result.shift_type_vars_with_cutoff(amount, cutoff)?;
                Some(Type::Function(Box::new(parameter), Box::new(result)))
            }
            Type::Apply {
                constructor,
                arguments,
            } => {
                let constructor = constructor.shift_type_vars_with_cutoff(amount, cutoff)?;
                let arguments = arguments
                    .iter()
                    .map(|arg| arg.shift_type_vars_with_cutoff(amount, cutoff))
                    .collect::<Option<Vec<_>>>()?;
                Some(Type::Apply {
                    constructor: Box::new(constructor),
                    arguments,
                })
            }
        }
    }

    fn shift_rec_vars_with_cutoff(
        &self,
        amount: i32,
        cutoff: RecursionIndex,
    ) -> Option<Self> {
        let shift_index = |index: RecursionIndex| {
            if index < cutoff {
                Some(index)
            } else {
                shift_index(index, amount)
            }
        };
        match self {
            Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph => Some(self.clone()),
            Type::TypeVar(index) => Some(Type::TypeVar(*index)),
            Type::MetaVar(index) => Some(Type::MetaVar(*index)),
            Type::RecVar(index) => shift_index(*index).map(Type::RecVar),
            Type::ForAll(body) => {
                body.shift_rec_vars_with_cutoff(amount, cutoff)
                    .map(|body| Type::ForAll(Box::new(body)))
            }
            Type::Mu(body) => {
                body.shift_rec_vars_with_cutoff(amount, cutoff + 1)
                    .map(|body| Type::Mu(Box::new(body)))
            }
            Type::Named { name, body } => {
                Some(Type::Named {
                    name: name.clone(),
                    body: body.clone(),
                })
            }
            Type::StructConstraint { fields, mode } => {
                fields
                    .iter()
                    .map(|(name, type_)| {
                        type_
                            .shift_rec_vars_with_cutoff(amount, cutoff)
                            .map(|type_| (name.clone(), type_))
                    })
                    .collect::<Option<IndexMap<_, _>>>()
                    .map(|fields| {
                        Type::StructConstraint {
                            fields,
                            mode: *mode,
                        }
                    })
            }
            Type::Struct { fields } => {
                fields
                    .iter()
                    .map(|(name, type_)| {
                        type_
                            .shift_rec_vars_with_cutoff(amount, cutoff)
                            .map(|type_| (name.clone(), type_))
                    })
                    .collect::<Option<IndexMap<_, _>>>()
                    .map(|fields| Type::Struct { fields })
            }
            Type::Array(inner) => {
                inner
                    .shift_rec_vars_with_cutoff(amount, cutoff)
                    .map(|inner| Type::Array(Box::new(inner)))
            }
            Type::Tuple(items) => {
                items
                    .iter()
                    .map(|item| item.shift_rec_vars_with_cutoff(amount, cutoff))
                    .collect::<Option<Vec<_>>>()
                    .map(Type::Tuple)
            }
            Type::Sum {
                variant_names,
                variant_types,
            } => {
                variant_types
                    .iter()
                    .map(|variant| variant.shift_rec_vars_with_cutoff(amount, cutoff))
                    .collect::<Option<Vec<_>>>()
                    .map(|variant_types| {
                        Type::Sum {
                            variant_names: variant_names.clone(),
                            variant_types,
                        }
                    })
            }
            Type::Function(parameter, result) => {
                let parameter = parameter.shift_rec_vars_with_cutoff(amount, cutoff)?;
                let result = result.shift_rec_vars_with_cutoff(amount, cutoff)?;
                Some(Type::Function(Box::new(parameter), Box::new(result)))
            }
            Type::Apply {
                constructor,
                arguments,
            } => {
                let constructor = constructor.shift_rec_vars_with_cutoff(amount, cutoff)?;
                let arguments = arguments
                    .iter()
                    .map(|arg| arg.shift_rec_vars_with_cutoff(amount, cutoff))
                    .collect::<Option<Vec<_>>>()?;
                Some(Type::Apply {
                    constructor: Box::new(constructor),
                    arguments,
                })
            }
        }
    }

    fn substitute_type_var_with_depth(
        &self,
        index: TypeParameterIndex,
        replacement: &Type,
        depth: TypeParameterIndex,
    ) -> Option<Self> {
        match self {
            Type::TypeVar(var_index) => {
                match index.checked_add(depth) {
                    Some(target) if *var_index == target => {
                        replacement.shift_type_vars(depth as i32, 0)
                    }
                    _ => Some(Type::TypeVar(*var_index)),
                }
            }
            Type::MetaVar(index) => Some(Type::MetaVar(*index)),
            Type::RecVar(index) => Some(Type::RecVar(*index)),
            Type::ForAll(body) => {
                body.substitute_type_var_with_depth(index, replacement, depth + 1)
                    .map(|body| Type::ForAll(Box::new(body)))
            }
            Type::Mu(body) => {
                body.substitute_type_var_with_depth(index, replacement, depth)
                    .map(|body| Type::Mu(Box::new(body)))
            }
            Type::Named { name, body } => {
                Some(Type::Named {
                    name: name.clone(),
                    body: body.clone(),
                })
            }
            Type::StructConstraint { fields, mode } => {
                fields
                    .iter()
                    .map(|(name, type_)| {
                        type_
                            .substitute_type_var_with_depth(index, replacement, depth)
                            .map(|type_| (name.clone(), type_))
                    })
                    .collect::<Option<IndexMap<_, _>>>()
                    .map(|fields| {
                        Type::StructConstraint {
                            fields,
                            mode: *mode,
                        }
                    })
            }
            Type::Struct { fields } => {
                fields
                    .iter()
                    .map(|(name, type_)| {
                        type_
                            .substitute_type_var_with_depth(index, replacement, depth)
                            .map(|type_| (name.clone(), type_))
                    })
                    .collect::<Option<IndexMap<_, _>>>()
                    .map(|fields| Type::Struct { fields })
            }
            Type::Array(inner) => {
                inner
                    .substitute_type_var_with_depth(index, replacement, depth)
                    .map(|inner| Type::Array(Box::new(inner)))
            }
            Type::Tuple(items) => {
                items
                    .iter()
                    .map(|item| item.substitute_type_var_with_depth(index, replacement, depth))
                    .collect::<Option<Vec<_>>>()
                    .map(Type::Tuple)
            }
            Type::Sum {
                variant_names,
                variant_types,
            } => {
                variant_types
                    .iter()
                    .map(|variant| {
                        variant.substitute_type_var_with_depth(index, replacement, depth)
                    })
                    .collect::<Option<Vec<_>>>()
                    .map(|variant_types| {
                        Type::Sum {
                            variant_names: variant_names.clone(),
                            variant_types,
                        }
                    })
            }
            Type::Function(parameter, result) => {
                let parameter =
                    parameter.substitute_type_var_with_depth(index, replacement, depth)?;
                let result = result.substitute_type_var_with_depth(index, replacement, depth)?;
                Some(Type::Function(Box::new(parameter), Box::new(result)))
            }
            Type::Apply {
                constructor,
                arguments,
            } => {
                let constructor =
                    constructor.substitute_type_var_with_depth(index, replacement, depth)?;
                let arguments = arguments
                    .iter()
                    .map(|arg| arg.substitute_type_var_with_depth(index, replacement, depth))
                    .collect::<Option<Vec<_>>>()?;
                Some(Type::Apply {
                    constructor: Box::new(constructor),
                    arguments,
                })
            }
            Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph => Some(self.clone()),
        }
    }

    fn substitute_rec_var_with_depth(
        &self,
        index: RecursionIndex,
        replacement: &Type,
        depth: RecursionIndex,
    ) -> Option<Self> {
        match self {
            Type::RecVar(var_index) => {
                match index.checked_add(depth) {
                    Some(target) if *var_index == target => {
                        replacement.shift_rec_vars(depth as i32, 0)
                    }
                    _ => Some(Type::RecVar(*var_index)),
                }
            }
            Type::TypeVar(index) => Some(Type::TypeVar(*index)),
            Type::MetaVar(index) => Some(Type::MetaVar(*index)),
            Type::ForAll(body) => {
                body.substitute_rec_var_with_depth(index, replacement, depth)
                    .map(|body| Type::ForAll(Box::new(body)))
            }
            Type::Mu(body) => {
                body.substitute_rec_var_with_depth(index, replacement, depth + 1)
                    .map(|body| Type::Mu(Box::new(body)))
            }
            Type::Named { name, body } => {
                Some(Type::Named {
                    name: name.clone(),
                    body: body.clone(),
                })
            }
            Type::StructConstraint { fields, mode } => {
                fields
                    .iter()
                    .map(|(name, type_)| {
                        type_
                            .substitute_rec_var_with_depth(index, replacement, depth)
                            .map(|type_| (name.clone(), type_))
                    })
                    .collect::<Option<IndexMap<_, _>>>()
                    .map(|fields| {
                        Type::StructConstraint {
                            fields,
                            mode: *mode,
                        }
                    })
            }
            Type::Struct { fields } => {
                fields
                    .iter()
                    .map(|(name, type_)| {
                        type_
                            .substitute_rec_var_with_depth(index, replacement, depth)
                            .map(|type_| (name.clone(), type_))
                    })
                    .collect::<Option<IndexMap<_, _>>>()
                    .map(|fields| Type::Struct { fields })
            }
            Type::Array(inner) => {
                inner
                    .substitute_rec_var_with_depth(index, replacement, depth)
                    .map(|inner| Type::Array(Box::new(inner)))
            }
            Type::Tuple(items) => {
                items
                    .iter()
                    .map(|item| item.substitute_rec_var_with_depth(index, replacement, depth))
                    .collect::<Option<Vec<_>>>()
                    .map(Type::Tuple)
            }
            Type::Sum {
                variant_names,
                variant_types,
            } => {
                variant_types
                    .iter()
                    .map(|variant| variant.substitute_rec_var_with_depth(index, replacement, depth))
                    .collect::<Option<Vec<_>>>()
                    .map(|variant_types| {
                        Type::Sum {
                            variant_names: variant_names.clone(),
                            variant_types,
                        }
                    })
            }
            Type::Function(parameter, result) => {
                let parameter =
                    parameter.substitute_rec_var_with_depth(index, replacement, depth)?;
                let result = result.substitute_rec_var_with_depth(index, replacement, depth)?;
                Some(Type::Function(Box::new(parameter), Box::new(result)))
            }
            Type::Apply {
                constructor,
                arguments,
            } => {
                let constructor =
                    constructor.substitute_rec_var_with_depth(index, replacement, depth)?;
                let arguments = arguments
                    .iter()
                    .map(|arg| arg.substitute_rec_var_with_depth(index, replacement, depth))
                    .collect::<Option<Vec<_>>>()?;
                Some(Type::Apply {
                    constructor: Box::new(constructor),
                    arguments,
                })
            }
            Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph => Some(self.clone()),
        }
    }

    fn pretty_with_context(
        &self,
        param_names: &[String],
        rec_names: &[String],
    ) -> String {
        match self {
            Type::Unit => "()".to_string(),
            Type::Integer => "integer".to_string(),
            Type::Real => "real".to_string(),
            Type::Boolean => "boolean".to_string(),
            Type::String => "string".to_string(),
            Type::Glyph => "glyph".to_string(),
            Type::TypeVar(index) => lookup_name(param_names, *index, type_var_name),
            Type::MetaVar(index) => format!("?t{index}"),
            Type::RecVar(index) => lookup_name(rec_names, *index, rec_var_name),
            Type::ForAll(body) => {
                let name = type_var_name(param_names.len() as u32);
                let mut next_params = param_names.to_vec();
                next_params.push(name.clone());
                format!(
                    "forall {name}. {}",
                    body.pretty_with_context(&next_params, rec_names)
                )
            }
            Type::Mu(body) => {
                let name = rec_var_name(rec_names.len() as u32);
                let mut next_recs = rec_names.to_vec();
                next_recs.push(name.clone());
                format!(
                    "mu {name}. {}",
                    body.pretty_with_context(param_names, &next_recs)
                )
            }
            Type::Named { name, .. } => name.to_string(),
            Type::StructConstraint { fields, mode } => {
                let fields = fields
                    .iter()
                    .map(|(name, type_)| {
                        format!(
                            "{name}: {}",
                            type_.pretty_with_context(param_names, rec_names)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let suffix = match mode {
                    StructMatch::Exact => "",
                    StructMatch::AtLeast => ", ..",
                };
                format!("{{{fields}{suffix}}}")
            }
            Type::Struct { fields } => {
                let fields = fields
                    .iter()
                    .map(|(name, type_)| {
                        format!(
                            "{name}: {}",
                            type_.pretty_with_context(param_names, rec_names)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{fields}}}")
            }
            Type::Array(inner) => {
                format!("[{}]", inner.pretty_with_context(param_names, rec_names))
            }
            Type::Tuple(items) => {
                let items = items
                    .iter()
                    .map(|item| item.pretty_with_context(param_names, rec_names))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({items})")
            }
            Type::Sum {
                variant_names,
                variant_types,
            } => {
                let items = variant_names
                    .iter()
                    .zip(variant_types)
                    .map(|(name, type_)| {
                        if matches!(type_, Type::Unit) {
                            name.clone()
                        } else {
                            format!("{name} {}", type_.pretty_wrapped(param_names, rec_names))
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");
                format!("(| {items} )")
            }
            Type::Function(parameter, result) => {
                format!(
                    "({} -> {})",
                    parameter.pretty_wrapped(param_names, rec_names),
                    result.pretty_wrapped(param_names, rec_names)
                )
            }
            Type::Apply {
                constructor,
                arguments,
            } => {
                if arguments.is_empty() {
                    constructor.pretty_with_context(param_names, rec_names)
                } else {
                    let args = arguments
                        .iter()
                        .map(|arg| arg.pretty_wrapped(param_names, rec_names))
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!(
                        "{} {args}",
                        constructor.pretty_wrapped(param_names, rec_names)
                    )
                }
            }
        }
    }

    fn pretty_wrapped(
        &self,
        param_names: &[String],
        rec_names: &[String],
    ) -> String {
        let pretty = self.pretty_with_context(param_names, rec_names);
        if self.is_wrapped_atom() {
            pretty
        } else {
            format!("({pretty})")
        }
    }

    fn is_wrapped_atom(&self) -> bool {
        matches!(
            self,
            Type::Unit
                | Type::Integer
                | Type::Real
                | Type::Boolean
                | Type::String
                | Type::Glyph
                | Type::TypeVar(_)
                | Type::MetaVar(_)
                | Type::RecVar(_)
                | Type::Named { .. }
                | Type::StructConstraint { .. }
                | Type::Array(_)
                | Type::Tuple(_)
                | Type::Struct { .. }
                | Type::Apply { .. }
        )
    }
}

impl std::fmt::Display for Type {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.write_str(&self.pretty())
    }
}

fn shift_index(
    index: u32,
    amount: i32,
) -> Option<u32> {
    if amount >= 0 {
        index.checked_add(amount as u32)
    } else {
        let abs = amount.checked_abs()? as u32;
        index.checked_sub(abs)
    }
}

fn lookup_name(
    names: &[String],
    index: u32,
    fallback: fn(u32) -> String,
) -> String {
    let offset = index as usize;
    let index_from_end = names.len().checked_sub(offset + 1);
    index_from_end
        .and_then(|pos| names.get(pos))
        .cloned()
        .unwrap_or_else(|| fallback(index))
}

fn type_var_name(index: u32) -> String {
    format!("'{}", alpha_name(index))
}

fn rec_var_name(index: u32) -> String {
    format!("'rec {}", alpha_name(index))
}

fn alpha_name(index: u32) -> String {
    let mut n = index + 1;
    let mut chars = Vec::new();
    while n > 0 {
        n -= 1;
        let rem = (n % 26) as u8;
        chars.push((b'a' + rem) as char);
        n /= 26;
    }
    chars.into_iter().rev().collect()
}

pub mod infer;
pub mod catalog;
pub mod traits;
pub mod unify;
pub mod resolve;

pub use catalog::{
    TypeCatalog,
    TypeDefinition,
};

pub use traits::{
    TraitConstraint,
    TraitDef,
    TraitEnv,
    TraitError,
    TraitImpl,
    TraitRef,
    TypeScheme,
};

pub use resolve::resolve_module;

#[cfg(test)]
mod tests;
