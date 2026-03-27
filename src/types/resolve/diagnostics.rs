//! Diagnostic rendering helpers for resolve/typechecking errors.

use std::collections::{
    HashMap,
    HashSet,
};

use indexmap::IndexMap;

use crate::logging::WithContext;
use crate::{
    FileLogger,
    Span,
};

use super::super::infer::TypeError;
use super::super::type_expr::TypeExprLowerError;
use super::super::{
    MetaVarId,
    TraitRef,
    Type,
    TypeTransform,
};
use super::common::format_trait_ref;
use super::{
    Path,
    SymbolTable,
    TraitError,
};

pub(super) fn log_term_duplicates(
    logger: &mut FileLogger,
    symbols: &SymbolTable,
    candidate_definitions: &[(Path, Span)],
) {
    let mut seen = HashSet::new();
    for (path, span) in candidate_definitions {
        if !seen.insert(path.clone()) {
            continue;
        }
        if symbols.terms().contains_key(path) {
            log_duplicate_definition(logger, *span, "term", path);
        }
    }
}

/// Emit a duplicate-definition error for a specific symbol kind.
pub(super) fn log_duplicate_definition(
    logger: &mut FileLogger,
    span: Span,
    kind: &str,
    path: &Path,
) {
    logger
        .error(format!("Duplicate {kind} definition"))
        .primary(format!("`{path}` is already defined."), span)
        .done();
}

/// Emit diagnostics for trait-resolution errors.
pub(super) fn log_trait_error(
    logger: &mut FileLogger,
    span: Span,
    error: TraitError,
) {
    match error {
        TraitError::UnknownTrait(path) => {
            logger
                .error("Unknown trait")
                .primary(format!("`{path}` is not defined."), span)
                .done();
        }
        TraitError::DuplicateTrait(path) => {
            logger
                .error("Duplicate trait definition")
                .primary(format!("`{path}` is already defined."), span)
                .done();
        }
        TraitError::ArityMismatch {
            trait_name,
            expected,
            found,
        } => {
            logger
                .error("Invalid trait application")
                .primary(
                    format!(
                        "`{}` expects {expected} type arguments but got {found}.",
                        trait_name
                    ),
                    span,
                )
                .done();
        }
        TraitError::OverlappingInstance { trait_name, .. } => {
            logger
                .error("Overlapping trait instance")
                .primary(format!("Instances for `{trait_name}` overlap."), span)
                .done();
        }
        TraitError::AmbiguousInstance { predicate } => {
            logger
                .error("Ambiguous trait instance")
                .primary(
                    format!(
                        "Multiple instances match `{}`.",
                        format_trait_ref(&predicate)
                    ),
                    span,
                )
                .done();
        }
        TraitError::RecursivePredicate { predicate } => {
            logger
                .error("Recursive trait constraint")
                .primary(
                    format!("`{}` depends on itself.", format_trait_ref(&predicate)),
                    span,
                )
                .done();
        }
        TraitError::InvalidInstance { trait_name } => {
            logger
                .error("Invalid trait instance")
                .primary(format!("Instance for `{trait_name}` is invalid."), span)
                .done();
        }
        TraitError::InvalidInstanceItems {
            trait_name,
            unknown_items,
            missing_items,
            unknown_associated_types,
            missing_associated_types,
        } => {
            let mut builder = logger.error("Invalid trait instance");
            let mut details = Vec::new();
            if !unknown_items.is_empty() {
                details.push(format!(
                    "unknown trait item(s) {}",
                    format_trait_item_list(&unknown_items)
                ));
            }
            if !missing_items.is_empty() {
                details.push(format!(
                    "missing required trait item(s) {}",
                    format_trait_item_list(&missing_items)
                ));
            }
            if !unknown_associated_types.is_empty() {
                details.push(format!(
                    "unknown associated type(s) {}",
                    format_trait_item_list(&unknown_associated_types)
                ));
            }
            if !missing_associated_types.is_empty() {
                details.push(format!(
                    "missing required associated type(s) {}",
                    format_trait_item_list(&missing_associated_types)
                ));
            }
            builder = if details.is_empty() {
                builder.primary(format!("Instance for `{trait_name}` is invalid."), span)
            } else {
                builder.primary(
                    format!("Instance for `{trait_name}` has {}.", details.join("; ")),
                    span,
                )
            };
            builder
                .note("Trait method and associated type names must match the trait definition exactly.")
                .done();
        }
        TraitError::InvalidAliasTarget { alias, target } => {
            logger
                .error("Invalid trait alias")
                .primary(
                    format!("`{alias}` cannot alias `{target}` because `{target}` is not a trait."),
                    span,
                )
                .done();
        }
        TraitError::KindMismatch {
            trait_name,
            expected,
            found,
        } => {
            logger
                .error("Invalid trait argument kind")
                .primary(
                    format!(
                        "`{trait_name}` expects kind `{expected}` but this argument has kind `{found}`."
                    ),
                    span,
                )
                .done();
        }
        TraitError::AssociatedTypeKindMismatch {
            trait_name,
            associated_type,
            expected,
            found,
        } => {
            logger
                .error("Invalid associated type kind")
                .primary(
                    format!(
                        "`{associated_type}` in impl `{trait_name}` expects kind `{expected}` but this assignment has kind `{found}`."
                    ),
                    span,
                )
                .done();
        }
        TraitError::NoInstance { predicate } => {
            logger
                .error("Missing trait instance")
                .primary(
                    format!("No instance found for `{}`.", format_trait_ref(&predicate)),
                    span,
                )
                .done();
        }
    }
}

fn format_trait_item_list(items: &[Path]) -> String {
    items
        .iter()
        .map(format_trait_item)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_trait_item(path: &Path) -> String {
    let name = path
        .minor
        .rsplit_once(Path::DELIMETER)
        .map(|(_, tail)| tail)
        .unwrap_or(path.minor.as_str());
    format!("`{name}`")
}

/// Emit diagnostics for inference/type-checking errors.
pub(super) fn log_type_error(
    logger: &mut FileLogger,
    error: TypeError,
) {
    let mut formatter = TypeErrorFormatter::default();
    match error {
        TypeError::UnknownIdentifier { path, span } => {
            logger
                .error("Unknown identifier")
                .primary(format!("`{path}` is not defined."), span)
                .done();
        }
        TypeError::UnknownConstructor { path, span } => {
            logger
                .error("Unknown constructor")
                .primary(format!("`{path}` is not defined."), span)
                .done();
        }
        TypeError::InvalidTypeApplication {
            name,
            expected,
            found,
            span,
        } => {
            logger
                .error("Invalid type application")
                .primary(
                    format!("`{name}` expects {expected} type arguments but got {found}."),
                    span,
                )
                .done();
        }
        TypeError::InvalidPlaceholderType { span } => {
            logger
                .error("Invalid placeholder type")
                .primary(
                    "`_` placeholder types are only allowed in local annotations.",
                    span,
                )
                .done();
        }
        TypeError::TraitConstraintsNotAllowed { span } => {
            logger
                .error("Trait constraints are not allowed in this type")
                .primary(
                    "`where` constraints are only valid in quantified type annotations that produce schemes.",
                    span,
                )
                .done();
        }
        TypeError::InvalidTraitApplication {
            name,
            expected,
            found,
            span,
        } => {
            logger
                .error("Invalid trait application")
                .primary(
                    format!("`{name}` expects {expected} type arguments but got {found}."),
                    span,
                )
                .done();
        }
        TypeError::KindMismatch {
            expected,
            found,
            span,
        } => {
            logger
                .error("Kind mismatch")
                .primary(
                    format!("Expected kind `{expected}` but found `{found}`."),
                    span,
                )
                .done();
        }
        TypeError::NotAFunction { type_, span } => {
            let type_ = formatter.format_type(&type_);
            logger
                .error("Not a function")
                .primary(format!("`{type_}` is not callable."), span)
                .done();
        }
        TypeError::InvalidScheme { span } => {
            logger
                .error("Invalid type scheme")
                .primary("A type scheme could not be instantiated.", span)
                .done();
        }
        TypeError::HigherRankAnnotationRequired { parameter, span } => {
            logger
                .error("Higher-rank annotation required")
                .primary(
                    format!(
                        "`{parameter}` needs an explicit type annotation to be used polymorphically; add a `for a in ...` annotation (optionally with `where`) or make its uses monomorphic."
                    ),
                    span,
                )
                .done();
        }
        TypeError::PolymorphicAnnotationMissingConstraints { predicates, span } => {
            let constraints = predicates
                .iter()
                .map(|predicate| formatter.format_trait_ref(predicate))
                .collect::<Vec<_>>()
                .join(", ");
            logger
                .error("Polymorphic annotation is missing trait constraints")
                .primary(
                    format!(
                        "This definition requires trait constraints (`{constraints}`). Add them with `where` (for example `for a in ... where ...`) or remove the explicit annotation."
                    ),
                    span,
                )
                .done();
        }
        TypeError::NonExhaustivePatterns {
            span,
            counterexample,
        } => {
            logger
                .error("Non-exhaustive patterns")
                .primary(
                    "This pattern set does not cover every possible value.",
                    span,
                )
                .note(format!(
                    "For example, `{counterexample}` would reach this `unreachable` branch."
                ))
                .done();
        }
        TypeError::Unification {
            error,
            span,
            context,
        } => {
            match error {
                super::super::unify::UnifyError::Occurs { var, in_type } => {
                    let var = formatter.format_meta_var(var);
                    let in_type = formatter.format_type(&in_type);
                    let mut builder = logger
                        .error("Occurs check failed")
                        .primary(
                            format!(
                                "Cannot make `{var}` equal to `{in_type}` because that would require `{var}` to contain itself."
                            ),
                            span,
                        );
                    if let Some(context) = context {
                        builder = builder.note(format!("While {context}."));
                    }
                    builder.done();
                }
                super::super::unify::UnifyError::Mismatch { left, right } => {
                    let left = formatter.normalize_type(&left);
                    let right = formatter.normalize_type(&right);
                    let left_display = left.pretty();
                    let right_display = right.pretty();
                    if context.is_some_and(|context| context.contains("annotation"))
                        && let Some(annotation) =
                            placeholder_constructor_mismatch_annotation(&left, &right)
                    {
                        let mut builder = logger
                            .error("Could not infer placeholder type constructor")
                            .primary(
                                format!(
                                    "Could not infer the constructor for `{annotation}` from this context."
                                ),
                                span,
                            )
                            .note(format!("found: `{left_display}`"))
                            .note(format!("required: `{right_display}`"))
                            .note(
                                "Add an explicit constructor in the annotation (for example `Option String`).",
                            );
                        if let Some(context) = context {
                            builder = builder.note(format!("While {context}."));
                        }
                        builder.done();
                        return;
                    }
                    let mut builder = logger
                        .error("Type mismatch")
                        .primary(
                            format!(
                                "Found `{left_display}`, but this site requires `{right_display}`."
                            ),
                            span,
                        )
                        .note(format!("found: `{left_display}`"))
                        .note(format!("required: `{right_display}`"));
                    if let Some(context) = context {
                        builder = builder.note(format!("While {context}."));
                    }
                    if let Some(note) = mismatch_detail_note(&left, &right) {
                        builder = builder.note(note);
                    }
                    builder.done();
                }
            }
        }
    }
}

fn mismatch_detail_note(
    left: &Type,
    right: &Type,
) -> Option<String> {
    match (left, right) {
        (Type::Tuple(left_items), Type::Tuple(right_items))
            if left_items.len() != right_items.len() =>
        {
            Some(format!(
                "Tuple lengths differ: found {} item(s) but required {} item(s).",
                left_items.len(),
                right_items.len()
            ))
        }
        (
            Type::Struct {
                fields: left_fields,
            },
            Type::Struct {
                fields: right_fields,
            },
        ) => struct_field_difference_note(left_fields, right_fields),
        (
            Type::StructConstraint {
                fields: left_fields,
                ..
            },
            Type::StructConstraint {
                fields: right_fields,
                ..
            },
        ) => struct_field_difference_note(left_fields, right_fields),
        (
            Type::Named {
                name: left_name, ..
            },
            Type::Named {
                name: right_name, ..
            },
        ) if left_name != right_name => {
            Some(format!(
                "These are different named types (`{left_name}` vs `{right_name}`). Named types match by name, not by field shape."
            ))
        }
        _ => None,
    }
}

fn placeholder_constructor_mismatch_annotation(
    left: &Type,
    right: &Type,
) -> Option<String> {
    placeholder_constructor_annotation(left).or_else(|| placeholder_constructor_annotation(right))
}

fn placeholder_constructor_annotation(type_: &Type) -> Option<String> {
    let Type::Apply {
        constructor,
        arguments,
    } = type_
    else {
        return None;
    };
    if !matches!(constructor.as_ref(), Type::MetaVar(_)) {
        return None;
    }
    let args = arguments
        .iter()
        .map(|arg| {
            let pretty = arg.pretty();
            if matches!(arg, Type::ForAll { .. } | Type::Function(_, _)) {
                format!("({pretty})")
            } else {
                pretty
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    Some(if args.is_empty() {
        "_".to_string()
    } else {
        format!("_ {args}")
    })
}

fn struct_field_difference_note(
    left_fields: &IndexMap<String, Type>,
    right_fields: &IndexMap<String, Type>,
) -> Option<String> {
    let missing = right_fields
        .keys()
        .filter(|name| !left_fields.contains_key(*name))
        .cloned()
        .collect::<Vec<_>>();
    let extra = left_fields
        .keys()
        .filter(|name| !right_fields.contains_key(*name))
        .cloned()
        .collect::<Vec<_>>();

    if missing.is_empty() && extra.is_empty() {
        return None;
    }

    Some(match (missing.is_empty(), extra.is_empty()) {
        (false, true) => format!("Missing field(s): {}.", missing.join(", ")),
        (true, false) => format!("Unexpected field(s): {}.", extra.join(", ")),
        (false, false) => {
            format!(
                "Missing field(s): {}. Unexpected field(s): {}.",
                missing.join(", "),
                extra.join(", ")
            )
        }
        (true, true) => String::new(),
    })
}

#[derive(Default)]
struct TypeErrorFormatter {
    meta_vars: HashMap<MetaVarId, MetaVarId>,
}

impl TypeErrorFormatter {
    fn normalize_type(
        &mut self,
        type_: &Type,
    ) -> Type {
        MetaVarNormalizer {
            meta_vars: &mut self.meta_vars,
        }
        .transform(type_)
        .unwrap_or_else(|| type_.clone())
    }

    fn format_type(
        &mut self,
        type_: &Type,
    ) -> String {
        self.normalize_type(type_).pretty()
    }

    fn format_meta_var(
        &mut self,
        id: MetaVarId,
    ) -> String {
        format!("?t{}", self.meta_var_name(id))
    }

    fn format_trait_ref(
        &mut self,
        trait_ref: &TraitRef,
    ) -> String {
        if trait_ref.arguments.is_empty() {
            trait_ref.trait_name.to_string()
        } else {
            let args = trait_ref
                .arguments
                .iter()
                .map(|argument| self.format_trait_argument(argument))
                .collect::<Vec<_>>()
                .join(" ");
            format!("{} {args}", trait_ref.trait_name)
        }
    }

    fn format_trait_argument(
        &mut self,
        type_: &Type,
    ) -> String {
        match type_ {
            Type::Apply { .. } => format!("({})", self.format_type(type_)),
            _ => self.format_type(type_),
        }
    }

    fn meta_var_name(
        &mut self,
        id: MetaVarId,
    ) -> MetaVarId {
        if let Some(name) = self.meta_vars.get(&id) {
            *name
        } else {
            let next = self.meta_vars.len() as MetaVarId;
            self.meta_vars.insert(id, next);
            next
        }
    }
}

struct MetaVarNormalizer<'a> {
    meta_vars: &'a mut HashMap<MetaVarId, MetaVarId>,
}

impl TypeTransform for MetaVarNormalizer<'_> {
    fn meta_var(
        &mut self,
        id: MetaVarId,
    ) -> Option<Type> {
        let mapped = if let Some(existing) = self.meta_vars.get(&id) {
            *existing
        } else {
            let next = self.meta_vars.len() as MetaVarId;
            self.meta_vars.insert(id, next);
            next
        };
        Some(Type::MetaVar(mapped))
    }

    fn named(
        &mut self,
        name: &Path,
        body: &Type,
    ) -> Option<Type> {
        Some(Type::Named {
            name: name.clone(),
            body: Box::new(self.transform(body)?),
        })
    }
}

/// Emit diagnostics for shared type-expression lowering errors.
pub(super) fn log_type_expr_lower_error(
    logger: &mut FileLogger,
    error: TypeExprLowerError,
) {
    match error {
        TypeExprLowerError::TypeParameterApplied { name, found, span } => {
            logger
                .error("Invalid type application")
                .primary(
                    format!("Type parameter `{name}` cannot take {found} type arguments."),
                    span,
                )
                .done();
        }
        TypeExprLowerError::InvalidTypeApplication {
            name,
            expected,
            found,
            span,
        } => {
            logger
                .error("Invalid type application")
                .primary(
                    format!("`{name}` expects {expected} type arguments but got {found}."),
                    span,
                )
                .done();
        }
        TypeExprLowerError::PlaceholderNotAllowed { span } => {
            logger
                .error("Invalid placeholder type")
                .primary(
                    "`_` placeholder types are only allowed in local annotations.",
                    span,
                )
                .done();
        }
        TypeExprLowerError::TraitConstraintsNotAllowed { span } => {
            logger
                .error("Trait constraints are not allowed in this type")
                .primary(
                    "`where` constraints are only valid in quantified type annotations that produce schemes.",
                    span,
                )
                .done();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Logger;

    #[test]
    fn type_error_formatter_renumbers_meta_vars_stably() {
        let mut formatter = TypeErrorFormatter::default();
        let left = Type::func(Type::MetaVar(999), Type::MetaVar(42));
        let right = Type::Tuple(vec![Type::MetaVar(42), Type::MetaVar(999)]);

        assert_eq!(formatter.format_type(&left), "v0 -> v1");
        assert_eq!(formatter.format_type(&right), "(v1, v0)");
    }

    #[test]
    fn type_error_formatter_formats_meta_var_with_normalized_name() {
        let mut formatter = TypeErrorFormatter::default();

        assert_eq!(formatter.format_meta_var(777), "?t0");
        assert_eq!(formatter.format_type(&Type::MetaVar(777)), "v0");
    }

    #[test]
    fn trait_item_mismatch_error_mentions_unknown_and_missing_items() {
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("demo.hc", "");

        log_trait_error(
            &mut file_logger,
            Span::new(0, 0),
            TraitError::InvalidInstanceItems {
                trait_name: Path::new("demo", "Monad"),
                unknown_items: vec![Path::new("demo", "flatmap")],
                missing_items: vec![Path::new("demo", "flat_map")],
                unknown_associated_types: Vec::new(),
                missing_associated_types: Vec::new(),
            },
        );

        let diagnostic = file_logger
            .iter()
            .next()
            .expect("expected one trait diagnostic");
        assert_eq!(diagnostic.message, "Invalid trait instance");
        let label = diagnostic.labels.first().expect("expected primary label");
        assert!(label.message.contains("`flatmap`"));
        assert!(label.message.contains("`flat_map`"));
    }
}
