use indexmap::IndexMap;

use crate::ir::Path;

pub type TypeParameterIndex = u32;
pub type MetaVarId = u32;

/// Structural match mode for struct constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructMatch {
    Exact,
    AtLeast,
}

/// Core type representation used by inference and type checking.
///
/// Invariants:
/// - `Type::Named` is primarily nominal (name-based equality/unification).
/// - `type ~Alias = ...` declarations are expanded to their structural body during
///   lowering and are represented by that body rather than `Type::Named`.
/// - `Type::StructConstraint` is a partial structural view used for record access
///   and pattern checking; it intentionally does not represent a first-class record
///   declaration.
/// - `Type::Apply` with zero arguments is canonicalized to its constructor.
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
    /// Universal type binder for parameters
    ForAll(Box<Type>),
    /// Nominal type definition.
    ///
    /// The `body` is retained for controlled instantiation/introspection in later
    /// passes, but equality and the main unification path are name-based.
    Named { name: Path, body: Box<Type> },
    /// Structural field constraint used by inference.
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
        let left = strip_empty_apply_ref(self);
        let right = strip_empty_apply_ref(other);
        match (left, right) {
            (Unit, Unit)
            | (Integer, Integer)
            | (Real, Real)
            | (Boolean, Boolean)
            | (String, String)
            | (Glyph, Glyph) => true,
            (TypeVar(left), TypeVar(right)) => left == right,
            (MetaVar(left), MetaVar(right)) => left == right,
            (ForAll(left), ForAll(right)) => left == right,
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
            _ => false,
        }
    }
}

impl Eq for Type {
}

fn strip_empty_apply_ref(mut type_: &Type) -> &Type {
    while let Type::Apply {
        constructor,
        arguments,
    } = type_
    {
        if arguments.is_empty() {
            type_ = constructor;
            continue;
        }
        break;
    }
    type_
}

pub(crate) fn normalize_empty_apply(type_: Type) -> Type {
    match type_ {
        Type::Apply {
            constructor,
            arguments,
        } if arguments.is_empty() => normalize_empty_apply(*constructor),
        other => other,
    }
}

pub(crate) fn for_each_child_type(
    type_: &Type,
    include_named_body: bool,
    mut visit: impl FnMut(&Type),
) {
    match type_ {
        Type::ForAll(body) | Type::Array(body) => {
            visit(body);
        }
        Type::Function(parameter, result) => {
            visit(parameter);
            visit(result);
        }
        Type::Tuple(items) => items.iter().for_each(&mut visit),
        Type::StructConstraint { fields, .. } | Type::Struct { fields } => {
            fields.values().for_each(&mut visit);
        }
        Type::Sum { variants } => variants.values().for_each(&mut visit),
        Type::Apply {
            constructor,
            arguments,
        } => {
            visit(constructor);
            arguments.iter().for_each(visit);
        }
        Type::Named { body, .. } if include_named_body => visit(body),
        Type::Named { .. }
        | Type::Unit
        | Type::Integer
        | Type::Real
        | Type::Boolean
        | Type::String
        | Type::Glyph
        | Type::TypeVar(_)
        | Type::MetaVar(_) => {}
    }
}

/// A generic traversal/transformation interface for [`Type`].
///
/// Implementors can override small hooks for specific variants while relying
/// on the default recursive behavior in [`TypeTransform::transform`] or
/// [`TypeTransform::walk`].
///
/// Returning `None` from a transform hook aborts the transform.
pub(crate) trait TypeTransform {
    /// Transform a bound type variable (`Type::TypeVar`).
    fn type_var(
        &mut self,
        index: TypeParameterIndex,
    ) -> Option<Type> {
        Some(Type::TypeVar(index))
    }

    /// Transform an inference metavariable (`Type::MetaVar`).
    fn meta_var(
        &mut self,
        id: MetaVarId,
    ) -> Option<Type> {
        Some(Type::MetaVar(id))
    }

    /// Transform a named type (`Type::Named`).
    ///
    /// The default implementation keeps the nominal name and body unchanged.
    /// Implementors that need to transform inside `body` should call
    /// [`TypeTransform::transform`] explicitly.
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

    /// Transform a type application (`Type::Apply`).
    ///
    /// The default implementation recursively transforms the constructor and
    /// each argument, then rebuilds `Type::Apply`.
    fn apply(
        &mut self,
        constructor: &Type,
        arguments: &[Type],
    ) -> Option<Type> {
        let constructor = self.transform(constructor)?;
        let arguments = arguments
            .iter()
            .map(|arg| self.transform(arg))
            .collect::<Option<Vec<_>>>()?;
        Some(constructor.apply(arguments))
    }

    /// Hook called before recursively visiting the body of `Type::ForAll`.
    fn enter_forall(&mut self) {
    }

    /// Hook called after recursively visiting the body of `Type::ForAll`.
    fn leave_forall(&mut self) {
    }

    /// Hook called by [`TypeTransform::walk`] for every visited node.
    fn visit(
        &mut self,
        _type_: &Type,
    ) {
    }

    /// Walk the type tree without rebuilding it.
    ///
    /// This is useful for read-only traversals and side-effecting analyses
    /// that do not need to allocate new `Type` values.
    fn walk(
        &mut self,
        type_: &Type,
    ) {
        self.visit(type_);
        match type_ {
            Type::ForAll(body) => {
                self.enter_forall();
                self.walk(body);
                self.leave_forall();
            }
            _ => for_each_child_type(type_, true, |child| self.walk(child)),
        }
    }

    /// Recursively transform a [`Type`] using hook methods from this trait.
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
            Type::ForAll(body) => {
                self.enter_forall();
                let body = self.transform(body)?;
                self.leave_forall();
                Some(Type::ForAll(Box::new(body)))
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
            } => self.apply(constructor, arguments),
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
            kind: TypeDefinitionKind::Alias,
        }
    }
    pub fn def_named(
        self,
        parameters: usize,
    ) -> TypeDefinition {
        TypeDefinition {
            parameters,
            body: self,
            kind: TypeDefinitionKind::Named,
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

    pub fn open_forall(
        &self,
        replacement: &Type,
    ) -> Option<Self> {
        let replacement = replacement.shift_type_vars(1, 0)?;
        self.substitute_type_var(0, &replacement)?
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
        self.pretty_with_context(&[])
    }

    fn pretty_with_context(
        &self,
        param_names: &[String],
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
            Type::ForAll(body) => {
                let name = type_var_name(param_names.len() as u32);
                let mut next_params = param_names.to_vec();
                next_params.push(name.clone());
                format!("forall {name}. {}", body.pretty_with_context(&next_params))
            }
            Type::Named { name, .. } => format!("{name}"),
            Type::StructConstraint { fields, mode } => {
                let fields = pretty_record_fields(fields, param_names);
                let suffix = match mode {
                    StructMatch::Exact => "",
                    StructMatch::AtLeast => ", ..",
                };
                format!("{{{fields}{suffix}}}")
            }
            Type::Struct { fields } => {
                let fields = pretty_record_fields(fields, param_names);
                format!("{{{fields}}}")
            }
            Type::Array(inner) => format!("[] {}", inner.pretty_with_context(param_names)),
            Type::Tuple(items) => {
                let items = items
                    .iter()
                    .map(|item| item.pretty_with_context(param_names))
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
                            format!("{name} {}", type_.pretty_wrapped(param_names))
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");
                format!("(| {items} )")
            }
            Type::Function(parameter, result) => {
                format!(
                    "({} -> {})",
                    parameter.pretty_wrapped(param_names),
                    result.pretty_wrapped(param_names)
                )
            }
            Type::Apply {
                constructor,
                arguments,
            } => {
                if arguments.is_empty() {
                    constructor.pretty_with_context(param_names)
                } else {
                    let args = arguments
                        .iter()
                        .map(|arg| arg.pretty_wrapped(param_names))
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("{} {args}", constructor.pretty_wrapped(param_names))
                }
            }
        }
    }

    fn pretty_wrapped(
        &self,
        param_names: &[String],
    ) -> String {
        let pretty = self.pretty_with_context(param_names);
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

fn pretty_record_fields(
    fields: &IndexMap<String, Type>,
    param_names: &[String],
) -> String {
    fields
        .iter()
        .map(|(name, type_)| format!("{name}: {}", type_.pretty_with_context(param_names)))
        .collect::<Vec<_>>()
        .join(", ")
}

mod instantiation;
mod type_expr;

pub mod infer;
pub mod resolve;
pub mod symbol_table;
pub mod traits;
pub mod unify;

pub use symbol_table::{
    MethodSpecialization,
    SymbolTable,
    TypeDefinition,
    TypeDefinitionKind,
};

pub use traits::{
    TraitConstraint,
    TraitDef,
    TraitError,
    TraitImpl,
    TraitRef,
    TypeScheme,
};

pub(crate) use traits::ordered_trait_methods;

pub use resolve::{
    ResolvedModule,
    resolve_module,
    resolve_module_with_symbols,
    resolve_module_with_symbols_and_schemes,
};

#[cfg(test)]
mod tests;
