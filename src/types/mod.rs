use indexmap::IndexMap;

use crate::ir::Path;

pub type TypeParameterIndex = u32;
pub type RecursionIndex = u32;
pub type MetaVarId = u32;

/// Structural match mode for struct constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructMatch {
    Exact,
    AtLeast,
}

/// Core type representation used by inference and type checking.
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
    Named { name: Path, body: Box<Type> },
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
    Sum { variants: IndexMap<String, Type> },
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
            (Sum { variants: left }, Sum { variants: right }) => left == right,
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

pub(crate) trait TypeTransform {
    fn type_var(
        &mut self,
        index: TypeParameterIndex,
    ) -> Option<Type> {
        Some(Type::TypeVar(index))
    }

    fn meta_var(
        &mut self,
        id: MetaVarId,
    ) -> Option<Type> {
        Some(Type::MetaVar(id))
    }

    fn rec_var(
        &mut self,
        index: RecursionIndex,
    ) -> Option<Type> {
        Some(Type::RecVar(index))
    }

    fn named(
        &mut self,
        name: &Path,
        body: &Type,
    ) -> Option<Type> {
        Some(Type::Named {
            name: name.clone(),
            body: Box::new(body.clone()),
        })
    }

    fn enter_forall(&mut self) {
    }

    fn leave_forall(&mut self) {
    }

    fn enter_mu(&mut self) {
    }

    fn leave_mu(&mut self) {
    }

    fn transform(
        &mut self,
        type_: &Type,
    ) -> Option<Type> {
        match type_ {
            Type::Unit
            | Type::Integer
            | Type::Real
            | Type::Boolean
            | Type::String
            | Type::Glyph => Some(type_.clone()),
            Type::TypeVar(index) => self.type_var(*index),
            Type::MetaVar(index) => self.meta_var(*index),
            Type::RecVar(index) => self.rec_var(*index),
            Type::ForAll(body) => {
                self.enter_forall();
                let body = self.transform(body)?;
                self.leave_forall();
                Some(Type::ForAll(Box::new(body)))
            }
            Type::Mu(body) => {
                self.enter_mu();
                let body = self.transform(body)?;
                self.leave_mu();
                Some(Type::Mu(Box::new(body)))
            }
            Type::Named { name, body } => self.named(name, body),
            Type::StructConstraint { fields, mode } => {
                let fields = fields
                    .iter()
                    .map(|(name, type_)| self.transform(type_).map(|type_| (name.clone(), type_)))
                    .collect::<Option<IndexMap<_, _>>>()?;
                Some(Type::StructConstraint {
                    fields,
                    mode: *mode,
                })
            }
            Type::Struct { fields } => {
                let fields = fields
                    .iter()
                    .map(|(name, type_)| self.transform(type_).map(|type_| (name.clone(), type_)))
                    .collect::<Option<IndexMap<_, _>>>()?;
                Some(Type::Struct { fields })
            }
            Type::Array(inner) => {
                self.transform(inner)
                    .map(|inner| Type::Array(Box::new(inner)))
            }
            Type::Tuple(items) => {
                items
                    .iter()
                    .map(|item| self.transform(item))
                    .collect::<Option<Vec<_>>>()
                    .map(Type::Tuple)
            }
            Type::Sum { variants } => {
                let variants = variants
                    .iter()
                    .map(|(name, type_)| self.transform(type_).map(|type_| (name.clone(), type_)))
                    .collect::<Option<IndexMap<_, _>>>()?;
                Some(Type::Sum { variants })
            }
            Type::Function(parameter, result) => {
                let parameter = self.transform(parameter)?;
                let result = self.transform(result)?;
                Some(Type::func(parameter, result))
            }
            Type::Apply {
                constructor,
                arguments,
            } => {
                let constructor = self.transform(constructor)?;
                let arguments = arguments
                    .iter()
                    .map(|arg| self.transform(arg))
                    .collect::<Option<Vec<_>>>()?;
                Some(Type::Apply {
                    constructor: Box::new(constructor),
                    arguments,
                })
            }
        }
    }
}

struct ShiftTypeVars {
    amount: i32,
    cutoff: TypeParameterIndex,
}

impl TypeTransform for ShiftTypeVars {
    fn type_var(
        &mut self,
        index: TypeParameterIndex,
    ) -> Option<Type> {
        if index < self.cutoff {
            Some(Type::TypeVar(index))
        } else {
            shift_index(index, self.amount).map(Type::TypeVar)
        }
    }

    fn enter_forall(&mut self) {
        self.cutoff += 1;
    }

    fn leave_forall(&mut self) {
        self.cutoff -= 1;
    }
}

struct ShiftRecVars {
    amount: i32,
    cutoff: RecursionIndex,
}

impl TypeTransform for ShiftRecVars {
    fn rec_var(
        &mut self,
        index: RecursionIndex,
    ) -> Option<Type> {
        if index < self.cutoff {
            Some(Type::RecVar(index))
        } else {
            shift_index(index, self.amount).map(Type::RecVar)
        }
    }

    fn enter_mu(&mut self) {
        self.cutoff += 1;
    }

    fn leave_mu(&mut self) {
        self.cutoff -= 1;
    }
}

struct SubstituteTypeVar<'a> {
    index: TypeParameterIndex,
    replacement: &'a Type,
    depth: TypeParameterIndex,
}

impl TypeTransform for SubstituteTypeVar<'_> {
    fn type_var(
        &mut self,
        var_index: TypeParameterIndex,
    ) -> Option<Type> {
        match self.index.checked_add(self.depth) {
            Some(target) if var_index == target => {
                self.replacement.shift_type_vars(self.depth as i32, 0)
            }
            _ => Some(Type::TypeVar(var_index)),
        }
    }

    fn enter_forall(&mut self) {
        self.depth += 1;
    }

    fn leave_forall(&mut self) {
        self.depth -= 1;
    }
}

struct SubstituteRecVar<'a> {
    index: RecursionIndex,
    replacement: &'a Type,
    depth: RecursionIndex,
}

impl TypeTransform for SubstituteRecVar<'_> {
    fn rec_var(
        &mut self,
        var_index: RecursionIndex,
    ) -> Option<Type> {
        match self.index.checked_add(self.depth) {
            Some(target) if var_index == target => {
                self.replacement.shift_rec_vars(self.depth as i32, 0)
            }
            _ => Some(Type::RecVar(var_index)),
        }
    }

    fn enter_mu(&mut self) {
        self.depth += 1;
    }

    fn leave_mu(&mut self) {
        self.depth -= 1;
    }
}

impl Type {
    pub fn array() -> Self {
        Type::Array(Type::v(0).into()).for_all(1)
    }
    pub fn function() -> Self {
        Type::func(Type::v(1), Type::v(0)).for_all(2)
    }
    pub fn for_all(
        self,
        count: usize,
    ) -> Self {
        (0..count).fold(self, |body, _| Type::ForAll(body.into()))
    }
    pub fn func(
        t1: Self,
        t2: Self,
    ) -> Self {
        Self::Function(t1.into(), t2.into())
    }
    pub fn v(id: u32) -> Self {
        Self::TypeVar(id)
    }
    pub fn curry(types: &[Self]) -> Self {
        match types {
            [] => Self::Unit,
            [t] => t.clone(),
            [p, r @ ..] => Self::func(p.clone(), Self::curry(r)),
        }
    }
    pub fn def(
        self,
        parameters: usize,
    ) -> TypeDefinition {
        TypeDefinition {
            parameters,
            body: self,
        }
    }
    pub fn scheme(self) -> TypeScheme {
        TypeScheme::new(self)
    }
    pub fn scheme_with_predicates(
        self,
        predicates: Vec<TraitConstraint>,
    ) -> TypeScheme {
        TypeScheme::with_predicates(self, predicates)
    }
    pub fn shift_type_vars(
        &self,
        amount: i32,
        cutoff: TypeParameterIndex,
    ) -> Option<Self> {
        ShiftTypeVars { amount, cutoff }.transform(self)
    }

    pub fn shift_rec_vars(
        &self,
        amount: i32,
        cutoff: RecursionIndex,
    ) -> Option<Self> {
        ShiftRecVars { amount, cutoff }.transform(self)
    }

    pub fn substitute_type_var(
        &self,
        index: TypeParameterIndex,
        replacement: &Type,
    ) -> Option<Self> {
        SubstituteTypeVar {
            index,
            replacement,
            depth: 0,
        }
        .transform(self)
    }

    pub fn substitute_rec_var(
        &self,
        index: RecursionIndex,
        replacement: &Type,
    ) -> Option<Self> {
        SubstituteRecVar {
            index,
            replacement,
            depth: 0,
        }
        .transform(self)
    }

    pub fn open_forall(
        &self,
        replacement: &Type,
    ) -> Option<Self> {
        self.substitute_type_var(0, replacement)?
            .shift_type_vars(-1, 0)
    }

    pub fn apply(
        self,
        arguments: Vec<Type>,
    ) -> Self {
        if arguments.is_empty() {
            self
        } else {
            Type::Apply {
                constructor: Box::new(self),
                arguments,
            }
        }
    }

    pub fn pretty(&self) -> String {
        self.pretty_with_context(&[], &[])
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
            Type::MetaVar(index) => format!("v{index}"),
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
            Type::Named { name, .. } => format!("{name}"),
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
                format!("[] {}", inner.pretty_with_context(param_names, rec_names))
            }
            Type::Tuple(items) => {
                let items = items
                    .iter()
                    .map(|item| item.pretty_with_context(param_names, rec_names))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({items})")
            }
            Type::Sum { variants } => {
                let items = variants
                    .iter()
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

pub(crate) fn core_type_arity(minor: &str) -> Option<usize> {
    match minor {
        "unit" | "integer" | "real" | "boolean" | "string" | "glyph" => Some(0),
        "array" => Some(1),
        "function" => Some(2),
        _ => None,
    }
}

pub(crate) fn resolve_core_type(
    minor: &str,
    args: &[Type],
) -> Option<Type> {
    match minor {
        "unit" if args.is_empty() => Some(Type::Unit),
        "integer" if args.is_empty() => Some(Type::Integer),
        "real" if args.is_empty() => Some(Type::Real),
        "boolean" if args.is_empty() => Some(Type::Boolean),
        "string" if args.is_empty() => Some(Type::String),
        "glyph" if args.is_empty() => Some(Type::Glyph),
        "array" if args.len() == 1 => args.first().map(|arg| Type::Array(Box::new(arg.clone()))),
        "function" if args.len() == 2 => {
            let [left, right] = args else {
                return None;
            };
            Some(Type::func(left.clone(), right.clone()))
        }
        _ => None,
    }
}

pub(crate) fn core_type_fallback(minor: &str) -> Option<Type> {
    match minor {
        "unit" => Some(Type::Unit),
        "integer" => Some(Type::Integer),
        "real" => Some(Type::Real),
        "boolean" => Some(Type::Boolean),
        "string" => Some(Type::String),
        "glyph" => Some(Type::Glyph),
        "array" => Some(Type::Array(Box::new(Type::Unit))),
        "function" => Some(Type::func(Type::Unit, Type::Unit)),
        _ => None,
    }
}

pub(crate) enum CoreTypeResolution {
    Known {
        expected: usize,
        resolved: Option<Type>,
        fallback: Option<Type>,
    },
    Unknown,
}

pub(crate) fn core_type_resolution(
    minor: &str,
    args: &[Type],
) -> CoreTypeResolution {
    match core_type_arity(minor) {
        Some(expected) => CoreTypeResolution::Known {
            expected,
            resolved: resolve_core_type(minor, args),
            fallback: core_type_fallback(minor),
        },
        None => CoreTypeResolution::Unknown,
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
pub mod resolve;
pub mod symbol_table;
pub mod traits;
pub mod unify;

pub use symbol_table::{
    SymbolTable,
    TypeDefinition,
};

pub use traits::{
    TraitConstraint,
    TraitDef,
    TraitError,
    TraitImpl,
    TraitRef,
    TypeScheme,
};

pub use resolve::{
    resolve_module,
    resolve_module_with_symbols,
    resolve_module_with_symbols_and_schemes,
    ResolvedModule,
};

#[cfg(test)]
mod tests;
