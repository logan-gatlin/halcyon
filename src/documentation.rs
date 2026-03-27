use crate::ir::{
    Path,
    PatternKind,
    ScopeKind,
    Statement,
    TermKind,
    is_placeholder_type_constructor_path,
};
use crate::types::{
    ResolvedModule,
    SymbolTable,
    Type,
    TypeScheme,
};
use std::collections::HashSet;

/// The kind of definition a [`Documentation`] entry describes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementKind {
    Term,
    Type,
    Trait,
    Impl,
}

impl StatementKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Term => "term",
            Self::Type => "type",
            Self::Trait => "trait",
            Self::Impl => "impl",
        }
    }
}

/// A single documented definition extracted from a type-checked module.
#[derive(Debug, Clone)]
pub struct Documentation {
    pub kind: StatementKind,
    pub name: Path,
    pub comments: String,
    pub type_: TypeScheme,
}

/// Extract documentation entries from a [`ResolvedModule`].
///
/// Walks the module's statements and produces one [`Documentation`] per
/// named definition, carrying its attached comments and resolved type scheme.
/// Wasm statements and anonymous bindings are skipped.
pub fn generate(
    resolved: &ResolvedModule,
    symbols: &SymbolTable,
) -> Vec<Documentation> {
    let ResolvedModule {
        module, schemes, ..
    } = resolved;
    let mut impl_method_paths = HashSet::new();
    for statement in module.statements.iter() {
        if let Statement::Impl { methods, .. } = statement {
            impl_method_paths.extend(methods.iter().map(|method| method.impl_path.clone()));
        }
    }
    module
        .statements
        .iter()
        .flat_map(|statement| {
            match statement {
                Statement::Term(term) => {
                    if is_hidden_doc(&term.comments) {
                        return Vec::new();
                    }
                    let TermKind::Let {
                        assignee,
                        scope: ScopeKind::Global,
                        ..
                    } = &term.kind
                    else {
                        return Vec::new();
                    };
                    collect_pattern_paths(assignee)
                        .into_iter()
                        .filter(|path| !impl_method_paths.contains(path))
                        .filter_map(|path| {
                            let scheme = schemes.get(&path)?;
                            Some(Documentation {
                                kind: StatementKind::Term,
                                name: path,
                                comments: term.comments.clone(),
                                type_: scheme.clone(),
                            })
                        })
                        .collect()
                }
                Statement::ConstructorAlias { comments, path, .. } => {
                    if is_hidden_doc(comments) {
                        return Vec::new();
                    }
                    let Some(type_) = schemes.get(path).cloned() else {
                        return Vec::new();
                    };
                    vec![Documentation {
                        kind: StatementKind::Term,
                        name: path.clone(),
                        comments: comments.clone(),
                        type_,
                    }]
                }
                Statement::Type { comments, path, .. } => {
                    if is_hidden_doc(comments) {
                        return Vec::new();
                    }
                    let type_ = symbols
                        .type_definitions()
                        .get(path)
                        .map(|def| TypeScheme::new(def.body.clone()))
                        .unwrap_or_else(|| TypeScheme::new(Type::Unit));
                    vec![Documentation {
                        kind: StatementKind::Type,
                        name: path.clone(),
                        comments: comments.clone(),
                        type_,
                    }]
                }
                Statement::Trait { comments, path, .. } => {
                    if is_hidden_doc(comments) {
                        return Vec::new();
                    }
                    let trait_type = symbols
                        .trait_defs()
                        .get(path)
                        .map(|def| {
                            let method_types: Vec<Type> =
                                def.methods.values().map(|s| s.type_.clone()).collect();
                            TypeScheme::new(Type::Tuple(method_types))
                        })
                        .unwrap_or_else(|| TypeScheme::new(Type::Unit));
                    vec![Documentation {
                        kind: StatementKind::Trait,
                        name: path.clone(),
                        comments: comments.clone(),
                        type_: trait_type,
                    }]
                }
                Statement::TraitAlias {
                    comments,
                    path,
                    target,
                } => {
                    if is_hidden_doc(comments) {
                        return Vec::new();
                    }
                    let type_ = symbols
                        .trait_definition(target)
                        .map(|def| {
                            let method_types: Vec<Type> = def
                                .methods
                                .values()
                                .map(|scheme| scheme.type_.clone())
                                .collect();
                            TypeScheme::new(Type::Tuple(method_types))
                        })
                        .unwrap_or_else(|| TypeScheme::new(Type::Unit));
                    vec![Documentation {
                        kind: StatementKind::Trait,
                        name: path.clone(),
                        comments: comments.clone(),
                        type_,
                    }]
                }
                Statement::Impl {
                    comments,
                    trait_path,
                    arguments,
                    ..
                } => {
                    if is_hidden_doc(comments) {
                        return Vec::new();
                    }
                    let name = impl_name(trait_path, arguments);
                    let type_ = symbols
                        .trait_definition(trait_path)
                        .map(|def| {
                            TypeScheme::new(
                                def.methods
                                    .values()
                                    .next()
                                    .map_or(Type::Unit, |s| s.type_.clone()),
                            )
                        })
                        .unwrap_or_else(|| TypeScheme::new(Type::Unit));
                    vec![Documentation {
                        kind: StatementKind::Impl,
                        name,
                        comments: comments.clone(),
                        type_,
                    }]
                }
                Statement::Wasm(_) => Vec::new(),
            }
        })
        .collect()
}

fn is_hidden_doc(comments: &str) -> bool {
    comments.contains("@HIDDEN")
}

/// Recursively collect all `Identifier` paths bound by a pattern.
fn collect_pattern_paths(pattern: &crate::ir::Pattern<Type>) -> Vec<Path> {
    match &pattern.kind {
        PatternKind::Identifier(path) => vec![path.clone()],
        PatternKind::Tuple(pats) => pats.iter().flat_map(collect_pattern_paths).collect(),
        PatternKind::Constructor(_, inner) => collect_pattern_paths(inner),
        PatternKind::Struct(fields) => fields.values().flat_map(collect_pattern_paths).collect(),
        PatternKind::Array {
            starting, ending, ..
        } => {
            starting
                .iter()
                .chain(ending.iter())
                .flat_map(collect_pattern_paths)
                .collect()
        }
        PatternKind::TypeHint(inner, _) => collect_pattern_paths(inner),
        PatternKind::Hole | PatternKind::ConstConstructor(_) | PatternKind::Immediate(_) => {
            Vec::new()
        }
    }
}

/// Build a synthetic display-name for an impl statement.
fn impl_name(
    trait_path: &Path,
    arguments: &[crate::ir::TypeExpr],
) -> Path {
    use std::fmt::Write;
    let mut minor = trait_path.minor.clone();
    for arg in arguments {
        let _ = write!(minor, " {}", type_expr_name(&arg.kind));
    }
    Path::new(trait_path.major.clone(), minor)
}

fn type_expr_name(kind: &crate::ir::TypeExprKind) -> String {
    match kind {
        crate::ir::TypeExprKind::Instantiation(path, args)
            if is_placeholder_type_constructor_path(path) && args.is_empty() =>
        {
            "_".to_string()
        }
        crate::ir::TypeExprKind::Instantiation(path, args)
            if is_placeholder_type_constructor_path(path) =>
        {
            let arg_strs: Vec<_> = args.iter().map(|a| type_expr_name(&a.kind)).collect();
            format!("(_ {})", arg_strs.join(" "))
        }
        crate::ir::TypeExprKind::Instantiation(path, args) if args.is_empty() => path.minor.clone(),
        crate::ir::TypeExprKind::Instantiation(path, args) => {
            let arg_strs: Vec<_> = args.iter().map(|a| type_expr_name(&a.kind)).collect();
            format!("({} {})", path.minor, arg_strs.join(" "))
        }
        crate::ir::TypeExprKind::Tuple(items) => {
            let strs: Vec<_> = items.iter().map(|a| type_expr_name(&a.kind)).collect();
            format!("({})", strs.join(", "))
        }
        crate::ir::TypeExprKind::ForAll(params, constraints, body) => {
            let param_names: Vec<_> = params.iter().map(|p| p.minor.clone()).collect();
            let prefix = format!("for {}", param_names.join(" "));
            let constraints = if constraints.is_empty() {
                String::new()
            } else {
                let constraints = constraints
                    .iter()
                    .map(|constraint| {
                        let args = constraint
                            .arguments
                            .iter()
                            .map(|arg| type_expr_name(&arg.kind))
                            .collect::<Vec<_>>()
                            .join(" ");
                        if args.is_empty() {
                            constraint.trait_name.minor.clone()
                        } else {
                            format!("{} {args}", constraint.trait_name.minor)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(" where {constraints}")
            };
            format!("{prefix} in {}{constraints}", type_expr_name(&body.kind))
        }
        crate::ir::TypeExprKind::Placeholder => "_".to_string(),
    }
}

/// Render a list of documentation entries as a Markdown string.
///
/// Groups entries by kind (types, traits, implementations, terms) and
/// renders each with its name, type signature, and doc comments.
pub fn render_markdown(
    module_name: &str,
    docs: &[Documentation],
) -> String {
    use std::fmt::Write;

    let mut out = String::new();
    let _ = writeln!(out, "# {module_name}\n");

    let sections = [
        ("Types", StatementKind::Type),
        ("Traits", StatementKind::Trait),
        ("Implementations", StatementKind::Impl),
        ("Functions", StatementKind::Term),
    ];

    for (heading, kind) in &sections {
        let entries: Vec<_> = docs.iter().filter(|d| d.kind == *kind).collect();
        if entries.is_empty() {
            continue;
        }
        let _ = writeln!(out, "## {heading}\n");
        for doc in entries {
            render_entry(&mut out, doc);
        }
    }

    out
}

pub fn render_json(
    module_name: &str,
    docs: &[Documentation],
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&serde_json::json!({
        "module": module_name,
        "entries": docs
            .iter()
            .map(|doc| {
                serde_json::json!({
                    "kind": doc.kind.as_str(),
                    "name": {
                        "major": doc.name.major.as_str(),
                        "minor": doc.name.minor.as_str(),
                    },
                    "signature": doc.type_.pretty(),
                    "comments": doc.comments.trim(),
                })
            })
            .collect::<Vec<_>>(),
    }))
}

fn render_entry(
    out: &mut String,
    doc: &Documentation,
) {
    use std::fmt::Write;

    let _ = writeln!(out, "### `{}`\n", doc.name.minor);
    let _ = writeln!(out, "```");
    let _ = writeln!(out, "{}", doc.type_.pretty());
    let _ = writeln!(out, "```\n");

    let comments = doc.comments.trim();
    if !comments.is_empty() {
        let _ = writeln!(out, "{comments}\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Span;
    use crate::ir::{
        ImmediateValue,
        ImplMethod,
        Module,
        Pattern,
        Term,
        TraitMethodDecl,
        TypeExpr,
        TypeExprKind,
    };
    use crate::types::TraitRef;
    use indexmap::IndexMap;

    fn typed_unit_term() -> Term<Type> {
        Term {
            comments: String::new(),
            kind: TermKind::Immediate(ImmediateValue::Unit),
            span: Span::Generated,
            type_: Type::Unit,
        }
    }

    fn typed_unreachable_term() -> Term<Type> {
        Term {
            comments: String::new(),
            kind: TermKind::Unreachable,
            span: Span::Generated,
            type_: Type::Unit,
        }
    }

    fn global_binding(
        path: Path,
        comments: &str,
    ) -> Statement<Type> {
        Statement::Term(Term {
            comments: comments.to_string(),
            kind: TermKind::Let {
                assignee: Pattern {
                    comments: String::new(),
                    kind: PatternKind::Identifier(path),
                    span: Span::Generated,
                    type_: Type::Unit,
                },
                scope: ScopeKind::Global,
                value: Box::new(typed_unit_term()),
                then: Box::new(typed_unit_term()),
                else_: Box::new(typed_unreachable_term()),
            },
            span: Span::Generated,
            type_: Type::Unit,
        })
    }

    fn resolved_module_with(
        statements: Vec<Statement<Type>>,
        schemes: IndexMap<Path, TypeScheme>,
    ) -> ResolvedModule {
        ResolvedModule {
            module: Module {
                name: "demo".to_string(),
                statements: statements.into_boxed_slice(),
            },
            schemes,
            evidence_requirements: IndexMap::new(),
        }
    }

    #[test]
    fn rendered_signatures_use_where_clause_for_predicates() {
        let docs = vec![Documentation {
            kind: StatementKind::Term,
            name: Path::new("demo", "eq_id"),
            comments: String::new(),
            type_: TypeScheme::with_predicates(
                Type::ForAll {
                    name: None,
                    body: Box::new(Type::func(Type::v(0), Type::v(0))),
                },
                vec![TraitRef::new(Path::new("demo", "Eq"), vec![Type::v(0)])],
            ),
        }];

        let markdown = render_markdown("demo", &docs);
        assert!(
            markdown.contains("for a in a -> a where demo::Eq a"),
            "markdown should render where-clause constraints"
        );
        assert!(
            !markdown.contains("=>"),
            "markdown signatures should not use `=>` predicate form"
        );

        let json = render_json("demo", &docs).expect("json rendering should succeed");
        assert!(
            json.contains("for a in a -> a where demo::Eq a"),
            "json signatures should render where-clause constraints"
        );
    }

    #[test]
    fn generate_skips_impl_method_bindings() {
        let impl_method_path = Path::new("demo", "show#1");
        let visible_path = Path::new("demo", "visible");

        let statements = vec![
            Statement::Impl {
                comments: String::new(),
                trait_path: Path::new("demo", "Show"),
                arguments: Box::default(),
                associated_types: Box::default(),
                methods: vec![ImplMethod {
                    trait_method: Path::new("demo", "show"),
                    impl_path: impl_method_path.clone(),
                    value: typed_unit_term(),
                    span: Span::Generated,
                }]
                .into_boxed_slice(),
            },
            global_binding(impl_method_path.clone(), ""),
            global_binding(visible_path.clone(), ""),
        ];
        let schemes = vec![
            (impl_method_path.clone(), TypeScheme::new(Type::Unit)),
            (visible_path.clone(), TypeScheme::new(Type::Unit)),
        ]
        .into_iter()
        .collect();
        let resolved = resolved_module_with(statements, schemes);

        let docs = generate(&resolved, &SymbolTable::new());

        assert!(
            docs.iter().any(|doc| {
                doc.kind == StatementKind::Impl && doc.name == Path::new("demo", "Show")
            }),
            "impl declarations should still be documented"
        );
        assert!(
            docs.iter().any(|doc| doc.name == visible_path),
            "normal global bindings should still be documented"
        );
        assert!(
            !docs
                .iter()
                .any(|doc| { doc.kind == StatementKind::Term && doc.name == impl_method_path }),
            "generated impl method bindings should not be documented as terms"
        );
    }

    #[test]
    fn generate_skips_items_marked_hidden() {
        let visible_path = Path::new("demo", "visible");
        let hidden_path = Path::new("demo", "hidden");
        let statements = vec![
            global_binding(visible_path.clone(), "visible"),
            global_binding(hidden_path.clone(), "internal @HIDDEN"),
        ];
        let schemes = vec![
            (visible_path.clone(), TypeScheme::new(Type::Unit)),
            (hidden_path.clone(), TypeScheme::new(Type::Unit)),
        ]
        .into_iter()
        .collect();
        let resolved = resolved_module_with(statements, schemes);

        let docs = generate(&resolved, &SymbolTable::new());

        assert!(
            docs.iter().any(|doc| doc.name == visible_path),
            "visible items should be documented"
        );
        assert!(
            !docs.iter().any(|doc| doc.name == hidden_path),
            "items tagged with @HIDDEN should not be documented"
        );
    }

    #[test]
    fn generate_does_not_emit_trait_method_entries() {
        let trait_path = Path::new("demo", "Eq");
        let method_path = Path::new("demo", "eq");
        let statements = vec![Statement::Trait {
            comments: "trait docs".to_string(),
            path: trait_path.clone(),
            parameters: Box::default(),
            associated_types: Box::default(),
            methods: vec![TraitMethodDecl {
                path: method_path.clone(),
                type_expr: TypeExpr {
                    comments: String::new(),
                    kind: TypeExprKind::Placeholder,
                    span: Span::Generated,
                },
                span: Span::Generated,
            }]
            .into_boxed_slice(),
        }];
        let schemes = vec![(method_path, TypeScheme::new(Type::Unit))]
            .into_iter()
            .collect();
        let resolved = resolved_module_with(statements, schemes);

        let docs = generate(&resolved, &SymbolTable::new());

        assert_eq!(
            docs.len(),
            1,
            "trait methods should not be emitted separately"
        );
        assert_eq!(docs[0].kind, StatementKind::Trait);
        assert_eq!(docs[0].name, trait_path);
    }
}
