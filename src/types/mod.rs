//! Core type-system model and shared type utilities.
//!
//! This module defines Halcyon's semantic [`Type`] representation and the
//! binder-aware operations used across inference, trait resolution, and
//! backend lowering.

use indexmap::IndexMap;
use std::collections::HashSet;

use crate::ir::Path;

/// De Bruijn index of a bound `for all` type variable.
pub type TypeParameterIndex = u32;

/// Identifier of an inference metavariable in the unification table.
pub type MetaVarId = u32;

/// Structural match mode for struct constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum StructMatch {
    Exact,
    AtLeast,
}

/// Core type representation used by inference and type checking.
///
/// Invariants:
/// - `Type::Named` is primarily nominal (name-based equality/unification).
/// - `type ~Alias = ...` declarations may remain as `Type::Named` during
///   inference-oriented lowering to preserve constructor shape; fully applied
///   aliases are expanded when a structural view is required.
/// - `Type::StructConstraint` is a partial structural view used for record access
///   and pattern checking; it intentionally does not represent a first-class record
///   declaration.
/// - `Type::Apply` with zero arguments is canonicalized to its constructor.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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
    /// Universal type binder for parameters.
    ///
    /// `name` is optional display metadata that preserves source binder names
    /// when available. It has no semantic effect on equality, unification, or
    /// inference.
    ForAll {
        name: Option<String>,
        body: Box<Type>,
    },
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

const TYPE_FUNCTION_BP: u8 = 10;
const TYPE_APPLICATION_BP: u8 = 20;
const TYPE_ATOM_BP: u8 = 30;

impl PartialEq for Type {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        use Type::*;
        let left = strip_empty_apply_layers(self);
        let right = strip_empty_apply_layers(other);
        match (left, right) {
            (Unit, Unit)
            | (Integer, Integer)
            | (Real, Real)
            | (Boolean, Boolean)
            | (String, String)
            | (Glyph, Glyph) => true,
            (TypeVar(left), TypeVar(right)) => left == right,
            (MetaVar(left), MetaVar(right)) => left == right,
            (ForAll { body: left, .. }, ForAll { body: right, .. }) => left == right,
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

fn strip_empty_apply_layers(mut type_: &Type) -> &Type {
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

/// Canonicalize empty type applications back to their constructor.
pub(crate) fn normalize_empty_apply(type_: Type) -> Type {
    match type_ {
        Type::Apply {
            constructor,
            arguments,
        } if arguments.is_empty() => normalize_empty_apply(*constructor),
        other => other,
    }
}

/// Visit direct child types of a node.
///
/// Named bodies are visited only when `include_named_body` is `true`.
/// @METHOD
pub(crate) fn for_each_child_type(
    type_: &Type,
    include_named_body: bool,
    mut visit: impl FnMut(&Type),
) {
    match type_ {
        Type::ForAll { body, .. } | Type::Array(body) => {
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
            Type::ForAll { body, .. } => {
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
            Type::ForAll { name, body } => {
                self.enter_forall();
                let body = self.transform(body)?;
                self.leave_forall();
                Some(Type::ForAll {
                    name: name.clone(),
                    body: Box::new(body),
                })
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
    /// Built-in polymorphic array type constructor (`for a in [] a`).
    pub fn array() -> Self {
        Type::Array(Type::v(0).into()).for_all(1)
    }

    /// Built-in polymorphic function type constructor (`for a b in a -> b`).
    pub fn function() -> Self {
        Type::func(Type::v(1), Type::v(0)).for_all(2)
    }

    /// Wrap this type in `count` nested `Type::ForAll` binders.
    pub fn for_all(
        self,
        count: usize,
    ) -> Self {
        (0..count).fold(self, |body, _| {
            Type::ForAll {
                name: None,
                body: body.into(),
            }
        })
    }

    pub fn for_all_with_names(
        self,
        names: impl IntoIterator<Item = Option<String>>,
    ) -> Self {
        let names = names.into_iter().collect::<Vec<_>>();
        names.into_iter().rev().fold(self, |body, name| {
            Type::ForAll {
                name,
                body: body.into(),
            }
        })
    }

    /// Build a function type (`t1 -> t2`).
    pub fn func(
        t1: Self,
        t2: Self,
    ) -> Self {
        Self::Function(t1.into(), t2.into())
    }

    /// Build a bound type variable from a De Bruijn index.
    pub fn v(id: u32) -> Self {
        Self::TypeVar(id)
    }

    /// Right-associatively curry a list of types.
    pub fn curry(types: &[Self]) -> Self {
        match types {
            [] => Self::Unit,
            [t] => t.clone(),
            [p, r @ ..] => Self::func(p.clone(), Self::curry(r)),
        }
    }

    /// Wrap this type as an alias-style type definition.
    pub fn def(
        self,
        parameters: usize,
    ) -> TypeDefinition {
        TypeDefinition {
            parameters,
            parameter_kinds: vec![Kind::Type; parameters],
            body: self,
            kind: TypeDefinitionKind::Alias,
        }
    }

    /// Wrap this type as a named (nominal) type definition.
    pub fn def_named(
        self,
        parameters: usize,
    ) -> TypeDefinition {
        TypeDefinition {
            parameters,
            parameter_kinds: vec![Kind::Type; parameters],
            body: self,
            kind: TypeDefinitionKind::Named,
        }
    }

    /// Lift this type into a predicate-free scheme.
    pub fn scheme(self) -> TypeScheme {
        TypeScheme::new(self)
    }

    /// Lift this type into a scheme with attached trait predicates.
    pub fn scheme_with_predicates(
        self,
        predicates: Vec<TraitConstraint>,
    ) -> TypeScheme {
        TypeScheme::with_predicates(self, predicates)
    }

    /// Shift bound type-variable indices by `amount` at/above `cutoff`.
    pub fn shift_type_vars(
        &self,
        amount: i32,
        cutoff: TypeParameterIndex,
    ) -> Option<Self> {
        ShiftTypeVars { amount, cutoff }.transform(self)
    }

    /// Substitute a single bound type variable with `replacement`.
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

    /// Open the outermost `for all` binder with `replacement`.
    pub fn open_forall(
        &self,
        replacement: &Type,
    ) -> Option<Self> {
        let replacement = replacement.shift_type_vars(1, 0)?;
        self.substitute_type_var(0, &replacement)?
            .shift_type_vars(-1, 0)
    }

    /// Build a type application, canonicalizing empty argument lists.
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
        self.pretty_in_context(param_names, 0, true)
    }

    fn pretty_in_context(
        &self,
        param_names: &[String],
        min_bp: u8,
        allow_forall: bool,
    ) -> String {
        if let Type::ForAll { .. } = self {
            let pretty = self.pretty_consecutive_foralls(param_names);
            if allow_forall && min_bp == 0 {
                return pretty;
            }
            return format!("({pretty})");
        }

        let (binding_power, pretty) = match self {
            Type::Unit => (TYPE_ATOM_BP, "()".to_string()),
            Type::Integer => (TYPE_ATOM_BP, "integer".to_string()),
            Type::Real => (TYPE_ATOM_BP, "real".to_string()),
            Type::Boolean => (TYPE_ATOM_BP, "boolean".to_string()),
            Type::String => (TYPE_ATOM_BP, "string".to_string()),
            Type::Glyph => (TYPE_ATOM_BP, "glyph".to_string()),
            Type::TypeVar(index) => (TYPE_ATOM_BP, lookup_name(param_names, *index)),
            Type::MetaVar(index) => (TYPE_ATOM_BP, format!("v{index}")),
            Type::Named { name, .. } => (TYPE_ATOM_BP, format!("{name}")),
            Type::StructConstraint { fields, mode } => {
                let fields = pretty_record_fields(fields, param_names);
                let suffix = match mode {
                    StructMatch::Exact => "",
                    StructMatch::AtLeast => ", ..",
                };
                (TYPE_ATOM_BP, format!("{{{fields}{suffix}}}"))
            }
            Type::Struct { fields } => {
                let fields = pretty_record_fields(fields, param_names);
                (TYPE_ATOM_BP, format!("{{{fields}}}"))
            }
            Type::Array(inner) => {
                (
                    TYPE_APPLICATION_BP,
                    format!("[] {}", inner.pretty_as_type_primary(param_names)),
                )
            }
            Type::Tuple(items) => (TYPE_ATOM_BP, self.pretty_tuple(items, param_names)),
            Type::Sum { variants } => {
                let items = variants
                    .iter()
                    .map(|(name, type_)| {
                        if matches!(type_, Type::Unit) {
                            name.clone()
                        } else {
                            format!("{name} {}", type_.pretty_as_type_primary(param_names))
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");
                (TYPE_ATOM_BP, format!("(| {items} )"))
            }
            Type::Function(parameter, result) => {
                (
                    TYPE_FUNCTION_BP,
                    format!(
                        "{} -> {}",
                        parameter.pretty_in_context(param_names, TYPE_FUNCTION_BP + 1, false),
                        result.pretty_in_context(param_names, TYPE_FUNCTION_BP, false)
                    ),
                )
            }
            Type::Apply { .. } => {
                (
                    TYPE_APPLICATION_BP,
                    self.pretty_type_application(param_names),
                )
            }
            Type::ForAll { .. } => unreachable!("forall handled above"),
        };

        if binding_power < min_bp {
            format!("({pretty})")
        } else {
            pretty
        }
    }

    fn pretty_consecutive_foralls(
        &self,
        param_names: &[String],
    ) -> String {
        let explicit_names = forall_explicit_names(self);
        let mut names = Vec::new();
        let mut next_params = param_names.to_vec();
        let mut used_generated_names = param_names
            .iter()
            .cloned()
            .chain(explicit_names)
            .collect::<HashSet<_>>();
        let mut generated_index = param_names.len() as u32;
        let mut body = self;

        while let Type::ForAll { name, body: inner } = body {
            let binder_name = name.clone().unwrap_or_else(|| {
                next_available_type_var_name(&mut generated_index, &used_generated_names)
            });
            if name.is_none() {
                used_generated_names.insert(binder_name.clone());
            }
            next_params.push(binder_name.clone());
            names.push(binder_name);
            body = inner;
        }

        debug_assert!(!names.is_empty());
        format!(
            "for {} in {}",
            names.join(" "),
            body.pretty_in_context(&next_params, 0, true)
        )
    }

    fn pretty_type_application(
        &self,
        param_names: &[String],
    ) -> String {
        let Type::Apply {
            constructor,
            arguments,
        } = self
        else {
            unreachable!("pretty_type_application called for non-application type")
        };

        if arguments.is_empty() {
            return constructor.pretty_in_context(param_names, 0, true);
        }

        let mut rendered = constructor.pretty_in_context(param_names, TYPE_APPLICATION_BP, false);
        for argument in arguments {
            rendered.push(' ');
            rendered.push_str(&argument.pretty_as_type_primary(param_names));
        }
        rendered
    }

    fn pretty_tuple(
        &self,
        items: &[Type],
        param_names: &[String],
    ) -> String {
        match items {
            [] => "()".to_string(),
            [item] => format!("({},)", item.pretty_in_context(param_names, 0, true)),
            _ => {
                let items = items
                    .iter()
                    .map(|item| item.pretty_in_context(param_names, 0, true))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({items})")
            }
        }
    }

    fn pretty_as_type_primary(
        &self,
        param_names: &[String],
    ) -> String {
        if self.is_type_primary() {
            self.pretty_in_context(param_names, 0, true)
        } else {
            format!("({})", self.pretty_in_context(param_names, 0, true))
        }
    }

    fn is_type_primary(&self) -> bool {
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
                | Type::Tuple(_)
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
) -> String {
    let offset = index as usize;
    let index_from_end = names.len().checked_sub(offset + 1);
    index_from_end
        .and_then(|pos| names.get(pos))
        .cloned()
        .unwrap_or_else(|| type_var_name_avoiding(index, names))
}

fn type_var_name_avoiding(
    index: u32,
    used_names: &[String],
) -> String {
    let used = used_names
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut candidate_index = 0u32;
    let mut emitted = 0u32;
    loop {
        let name = type_var_name(candidate_index);
        if !used.contains(name.as_str()) {
            if emitted == index {
                return name;
            }
            emitted += 1;
        }
        candidate_index += 1;
    }
}

fn next_available_type_var_name(
    generated_index: &mut u32,
    used_names: &HashSet<String>,
) -> String {
    loop {
        let candidate = type_var_name(*generated_index);
        *generated_index += 1;
        if !used_names.contains(&candidate) {
            return candidate;
        }
    }
}

fn forall_explicit_names(type_: &Type) -> HashSet<String> {
    let mut names = HashSet::new();
    let mut current = type_;
    while let Type::ForAll { name, body } = current {
        if let Some(name) = name {
            names.insert(name.clone());
        }
        current = body;
    }
    names
}

fn type_var_name(index: u32) -> String {
    let mut candidate_index = 0u32;
    let mut emitted = 0u32;
    loop {
        let name = alpha_name(candidate_index);
        if !is_reserved_type_variable_name(name.as_str()) {
            if emitted == index {
                return name;
            }
            emitted += 1;
        }
        candidate_index += 1;
    }
}

fn is_reserved_type_variable_name(name: &str) -> bool {
    matches!(
        name,
        "let"
            | "do"
            | "in"
            | "module"
            | "bundle"
            | "import"
            | "use"
            | "as"
            | "of"
            | "end"
            | "match"
            | "with"
            | "if"
            | "then"
            | "else"
            | "and"
            | "or"
            | "xor"
            | "not"
            | "true"
            | "false"
            | "fn"
            | "type"
            | "trait"
            | "impl"
            | "wasm"
            | "for"
            | "where"
            | "root"
    )
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

mod common;
mod instantiation;
mod kind;
mod predicate;
mod type_expr;

pub mod infer;
pub mod resolve;
pub mod symbol_table;
pub mod traits;
pub mod unify;

pub use kind::Kind;

pub(crate) use common::{
    for_each_pattern_binding,
    normalize_parameter_kinds,
    split_applied_type,
    split_applied_type_ref,
};

pub(crate) use predicate::{
    predicate_is_ground,
    predicate_sort_key,
    sorted_unique_predicates,
};

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
