use crate::ir::{
    Path,
    PatternKind,
    ScopeKind,
    Statement,
    TermKind,
};
use crate::types::{
    ResolvedModule,
    SymbolTable,
    Type,
    TypeScheme,
};

/// The kind of definition a [`Documentation`] entry describes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementKind {
    Term,
    Type,
    Trait,
    Impl,
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
    let ResolvedModule { module, schemes } = resolved;
    module
        .statements
        .iter()
        .flat_map(|statement| {
            match statement {
                Statement::Term(term) => {
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
                Statement::Type { comments, path, .. } => {
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
                Statement::Trait {
                    comments,
                    path,
                    methods,
                    ..
                } => {
                    let mut docs = Vec::with_capacity(1 + methods.len());
                    let trait_type = symbols
                        .trait_defs()
                        .get(path)
                        .map(|def| {
                            let method_types: Vec<Type> =
                                def.methods.values().map(|s| s.type_.clone()).collect();
                            TypeScheme::new(Type::Tuple(method_types))
                        })
                        .unwrap_or_else(|| TypeScheme::new(Type::Unit));
                    docs.push(Documentation {
                        kind: StatementKind::Trait,
                        name: path.clone(),
                        comments: comments.clone(),
                        type_: trait_type,
                    });
                    for method in methods.iter() {
                        if let Some(scheme) = schemes.get(&method.path) {
                            docs.push(Documentation {
                                kind: StatementKind::Term,
                                name: method.path.clone(),
                                comments: String::new(),
                                type_: scheme.clone(),
                            });
                        }
                    }
                    docs
                }
                Statement::Impl {
                    comments,
                    trait_path,
                    arguments,
                    ..
                } => {
                    let name = impl_name(trait_path, arguments);
                    let type_ = symbols
                        .trait_defs()
                        .get(trait_path)
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
        crate::ir::TypeExprKind::Instantiation(path, args) if args.is_empty() => path.minor.clone(),
        crate::ir::TypeExprKind::Instantiation(path, args) => {
            let arg_strs: Vec<_> = args.iter().map(|a| type_expr_name(&a.kind)).collect();
            format!("({} {})", path.minor, arg_strs.join(" "))
        }
        crate::ir::TypeExprKind::Tuple(items) => {
            let strs: Vec<_> = items.iter().map(|a| type_expr_name(&a.kind)).collect();
            format!("({})", strs.join(", "))
        }
        crate::ir::TypeExprKind::ForAll(params, body) => {
            let param_names: Vec<_> = params.iter().map(|p| p.minor.clone()).collect();
            format!(
                "for {}. {}",
                param_names.join(" "),
                type_expr_name(&body.kind)
            )
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

fn render_entry(
    out: &mut String,
    doc: &Documentation,
) {
    use std::fmt::Write;

    let _ = writeln!(out, "### `{}`\n", doc.name.minor);
    let _ = writeln!(out, "```");
    if !doc.type_.predicates.is_empty() {
        for (i, p) in doc.type_.predicates.iter().enumerate() {
            if i > 0 {
                let _ = write!(out, ", ");
            }
            let _ = write!(out, "{}", p.trait_name.minor);
            for arg in &p.arguments {
                let _ = write!(out, " {arg}");
            }
        }
        let _ = write!(out, " => ");
    }
    let _ = writeln!(out, "{}", doc.type_.type_);
    let _ = writeln!(out, "```\n");

    let comments = doc.comments.trim();
    if !comments.is_empty() {
        let _ = writeln!(out, "{comments}\n");
    }
}
