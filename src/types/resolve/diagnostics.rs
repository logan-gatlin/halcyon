//! Diagnostic rendering helpers for resolve/typechecking errors.

use std::collections::HashSet;

use crate::logging::WithContext;
use crate::{
    FileLogger,
    Span,
};

use super::super::infer::TypeError;
use super::super::type_expr::TypeExprLowerError;
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

/// Emit diagnostics for inference/type-checking errors.
pub(super) fn log_type_error(
    logger: &mut FileLogger,
    error: TypeError,
) {
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
                .map(format_trait_ref)
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
        TypeError::Unification { error, span } => {
            match error {
                super::super::unify::UnifyError::Occurs { var, in_type } => {
                    logger
                        .error("Occurs check failed")
                        .primary(
                            format!("Type variable ?t{var} occurs in `{in_type}`."),
                            span,
                        )
                        .done();
                }
                super::super::unify::UnifyError::Mismatch { left, right } => {
                    logger
                        .error("Type mismatch")
                        .primary(format!("`{left}` does not match `{right}`."), span)
                        .done();
                }
            }
        }
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
