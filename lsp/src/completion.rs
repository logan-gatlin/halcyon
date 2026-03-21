use std::collections::HashSet;

use halcyon_lib::tooling::utf16_position_to_byte_offset;
use lsp_types::{
    CompletionItem,
    CompletionItemKind,
    Position,
};

#[derive(Debug, Default)]
pub struct CompletionContext {
    pub qualifier: Option<String>,
    pub prefix: String,
}

pub fn completion_context_at(
    source: &str,
    position: Position,
) -> Option<CompletionContext> {
    let offset = utf16_position_to_byte_offset(source, position.line, position.character)?;
    let prefix = source
        .get(..offset)?
        .chars()
        .rev()
        .take_while(|ch| is_completion_character(*ch))
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();

    if let Some((qualifier, leaf_prefix)) = prefix.rsplit_once("::") {
        return Some(CompletionContext {
            qualifier: Some(qualifier.to_string()),
            prefix: leaf_prefix.to_string(),
        });
    }

    Some(CompletionContext {
        qualifier: None,
        prefix,
    })
}

pub fn completion_items(
    symbols: &halcyon_lib::types::SymbolTable,
    context: &CompletionContext,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let mut seen = HashSet::new();

    for (path, scheme) in symbols.terms() {
        push_symbol_completion(
            &mut items,
            &mut seen,
            path,
            CompletionItemKind::FUNCTION,
            Some(scheme.pretty()),
            context,
        );
    }

    for path in symbols.constructors() {
        push_symbol_completion(
            &mut items,
            &mut seen,
            path,
            CompletionItemKind::CONSTRUCTOR,
            None,
            context,
        );
    }

    for path in symbols.type_definitions().keys() {
        push_symbol_completion(
            &mut items,
            &mut seen,
            path,
            CompletionItemKind::STRUCT,
            None,
            context,
        );
    }

    for path in symbols.trait_defs().keys() {
        push_symbol_completion(
            &mut items,
            &mut seen,
            path,
            CompletionItemKind::INTERFACE,
            None,
            context,
        );
    }

    for keyword in [
        "bundle", "module", "import", "use", "let", "type", "trait", "impl", "fn", "if", "then",
        "else", "match", "with", "do", "for", "where", "root", "bundle",
    ] {
        if !keyword.starts_with(&context.prefix) {
            continue;
        }
        let key = format!("keyword:{keyword}");
        if !seen.insert(key) {
            continue;
        }
        items.push(CompletionItem {
            label: keyword.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..CompletionItem::default()
        });
    }

    items
}

fn is_completion_character(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(
            ch,
            '_' | '-' | ':' | '[' | ']' | '+' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '|' | '~'
        )
}

fn push_symbol_completion(
    items: &mut Vec<CompletionItem>,
    seen: &mut HashSet<String>,
    path: &halcyon_lib::ir::Path,
    kind: CompletionItemKind,
    detail: Option<String>,
    context: &CompletionContext,
) {
    let full_name = path.to_string();
    let leaf_name = path
        .minor
        .rsplit_once("::")
        .map(|(_, leaf)| leaf)
        .unwrap_or(path.minor.as_str())
        .to_string();

    if let Some(qualifier) = context.qualifier.as_deref()
        && !full_name.starts_with(qualifier)
    {
        return;
    }
    if !leaf_name.starts_with(&context.prefix) {
        return;
    }

    let key = format!("{:?}:{full_name}", kind);
    if !seen.insert(key) {
        return;
    }

    let detail = detail
        .map(|detail| format!("{full_name} : {detail}"))
        .or_else(|| Some(full_name.clone()));

    items.push(CompletionItem {
        label: leaf_name,
        kind: Some(kind),
        detail,
        ..CompletionItem::default()
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use halcyon_lib::ir::Path;
    use halcyon_lib::types::{
        SymbolTable,
        TraitRef,
        Type,
        TypeScheme,
    };

    #[test]
    fn completion_context_extracts_qualifier_and_prefix() {
        let context = completion_context_at(
            "let value = foo::bar",
            Position {
                line: 0,
                character: 20,
            },
        )
        .unwrap_or_default();
        assert_eq!(context.qualifier.as_deref(), Some("foo"));
        assert_eq!(context.prefix, "bar");
    }

    #[test]
    fn completion_detail_renders_constraints_with_where_clause() {
        let mut symbols = SymbolTable::new();
        symbols.insert_term(
            Path::new("demo", "eq_id"),
            TypeScheme::with_predicates(
                Type::ForAll {
                    name: None,
                    body: Box::new(Type::func(Type::v(0), Type::v(0))),
                },
                vec![TraitRef::new(Path::new("demo", "Eq"), vec![Type::v(0)])],
            ),
        );

        let items = completion_items(
            &symbols,
            &CompletionContext {
                qualifier: None,
                prefix: "eq".to_string(),
            },
        );
        let detail = items
            .iter()
            .find(|item| item.label == "eq_id")
            .and_then(|item| item.detail.as_deref())
            .unwrap_or_default();

        assert!(
            detail.contains("where demo::Eq a"),
            "completion detail should include where-clause constraints; got `{detail}`"
        );
    }
}
