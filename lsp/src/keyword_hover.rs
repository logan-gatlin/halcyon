use halcyon_lib::tooling::{
    byte_offset_to_utf16_position,
    utf16_position_to_byte_offset,
};
use lsp_types::{
    Hover,
    HoverContents,
    MarkupContent,
    MarkupKind,
    Position,
    Range,
};

struct KeywordDoc {
    summary: &'static str,
    example: &'static str,
}

pub fn hover_for_keyword(
    source: &str,
    position: Position,
) -> Option<Hover> {
    let (keyword, range) = keyword_at_position(source, position)?;
    let doc = keyword_doc(keyword)?;
    let markdown = format!(
        "**Keyword** `{keyword}`\n\n{}\n\n```halcyon\n{}\n```",
        doc.summary, doc.example
    );
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: markdown,
        }),
        range: Some(range),
    })
}

fn keyword_at_position(
    source: &str,
    position: Position,
) -> Option<(&str, Range)> {
    let offset = utf16_position_to_byte_offset(source, position.line, position.character)?;
    let (start, end) = keyword_bounds(source, offset)?;
    let token = source.get(start..end)?;
    if keyword_doc(token).is_none() {
        return None;
    }

    let start = byte_offset_to_utf16_position(source, start);
    let end = byte_offset_to_utf16_position(source, end);
    Some((
        token,
        Range {
            start: Position {
                line: start.line,
                character: start.character,
            },
            end: Position {
                line: end.line,
                character: end.character,
            },
        },
    ))
}

fn keyword_bounds(
    source: &str,
    offset: usize,
) -> Option<(usize, usize)> {
    if source.is_empty() {
        return None;
    }

    let mut probe = offset.min(source.len());
    if probe == source.len() || !is_keyword_char(char_at(source, probe)?) {
        let previous = prev_char_start(source, probe)?;
        if !is_keyword_char(char_at(source, previous)?) {
            return None;
        }
        probe = previous;
    }

    let mut start = probe;
    while let Some(previous) = prev_char_start(source, start) {
        if !is_keyword_char(char_at(source, previous)?) {
            break;
        }
        start = previous;
    }

    let mut end = probe + char_at(source, probe)?.len_utf8();
    while end < source.len() {
        let ch = char_at(source, end)?;
        if !is_keyword_char(ch) {
            break;
        }
        end += ch.len_utf8();
    }

    Some((start, end))
}

fn char_at(
    source: &str,
    start: usize,
) -> Option<char> {
    source.get(start..)?.chars().next()
}

fn prev_char_start(
    source: &str,
    index: usize,
) -> Option<usize> {
    if index == 0 {
        return None;
    }
    let mut previous = index.saturating_sub(1);
    while !source.is_char_boundary(previous) {
        previous = previous.saturating_sub(1);
    }
    Some(previous)
}

fn is_keyword_char(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

fn keyword_doc(keyword: &str) -> Option<KeywordDoc> {
    Some(match keyword {
        "bundle" => {
            KeywordDoc {
                summary: "Declares the current file as a bundle root and sets its global bundle name.",
                example: "bundle demo",
            }
        }
        "module" => {
            KeywordDoc {
                summary: "Defines a nested module scope inside the current bundle.",
                example: "module math =\n  let two = 2\nend",
            }
        }
        "import" => {
            KeywordDoc {
                summary: "Loads additional source files into the same bundle during compilation.",
                example: "import \"util.hc\", \"ops.hc\"",
            }
        }
        "use" => {
            KeywordDoc {
                summary: "Brings names from a module/path into scope for subsequent declarations.",
                example: "use root::core::ops",
            }
        }
        "as" => {
            KeywordDoc {
                summary: "Renames an imported module or symbol to a local alias.",
                example: "use root::core::ops as ops",
            }
        }
        "let" => {
            KeywordDoc {
                summary: "Defines a value binding (top-level, module-level, or expression-local).",
                example: "let add = fn x y => x + y",
            }
        }
        "do" => {
            KeywordDoc {
                summary: "Runs an expression for effects and discards its value (sugar for `_` binding).",
                example: "do log \"hello\"",
            }
        }
        "in" => {
            KeywordDoc {
                summary: "Separates bindings from body expressions in `let` and polymorphic forms.",
                example: "let x = 1 in x + 2",
            }
        }
        "if" => {
            KeywordDoc {
                summary: "Starts a conditional expression.",
                example: "if x > 0 then x else 0 - x",
            }
        }
        "then" => {
            KeywordDoc {
                summary: "Introduces the true branch of an `if` expression.",
                example: "if ok then value else fallback",
            }
        }
        "else" => {
            KeywordDoc {
                summary: "Introduces the false branch of an `if` expression.",
                example: "if ok then value else fallback",
            }
        }
        "match" => {
            KeywordDoc {
                summary: "Starts pattern matching over a value.",
                example: "match value with\n  | Some x => x\n  | None => 0",
            }
        }
        "with" => {
            KeywordDoc {
                summary: "Begins the list of match arms after `match`.",
                example: "match value with\n  | Some x => x\n  | None => 0",
            }
        }
        "fn" => {
            KeywordDoc {
                summary: "Creates an anonymous function (lambda expression).",
                example: "let add = fn x y => x + y",
            }
        }
        "type" => {
            KeywordDoc {
                summary: "Declares a type definition (alias, named wrapper, struct, or sum type).",
                example: "type Option: a = | Some a | None",
            }
        }
        "trait" => {
            KeywordDoc {
                summary: "Declares a trait interface with associated items.",
                example: "trait Show : a =\n  let show : a -> String\nend",
            }
        }
        "impl" => {
            KeywordDoc {
                summary: "Defines a trait implementation for concrete or polymorphic arguments.",
                example: "impl Show Integer =\n  let show = integer::to_string\nend",
            }
        }
        "for" => {
            KeywordDoc {
                summary: "Introduces polymorphic type parameters in annotations and impl heads.",
                example: "let id : for a in a -> a = fn x => x",
            }
        }
        "where" => {
            KeywordDoc {
                summary: "Adds trait constraints to polymorphic declarations.",
                example: "let eq : for a in a -> a -> Boolean where ops::Equal a",
            }
        }
        "root" => {
            KeywordDoc {
                summary: "Prefixes a path to force lookup from the global root namespace.",
                example: "use root::core::ops",
            }
        }
        "wasm" => {
            KeywordDoc {
                summary: "Introduces inline WebAssembly declarations or typed wasm expressions.",
                example: "let add = fn x y => (wasm : Integer) => (local.get 0 local.get 1 i64.add)",
            }
        }
        "end" => {
            KeywordDoc {
                summary: "Closes block constructs such as `module`, `trait`, and `impl`.",
                example: "module math =\n  let two = 2\nend",
            }
        }
        "true" => {
            KeywordDoc {
                summary: "Boolean literal representing truth.",
                example: "let enabled = true",
            }
        }
        "false" => {
            KeywordDoc {
                summary: "Boolean literal representing falsehood.",
                example: "let enabled = false",
            }
        }
        "of" => {
            KeywordDoc {
                summary: "Reserved keyword token; constructor payloads are written by juxtaposition in current syntax.",
                example: "type Option: a = | Some a | None",
            }
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hover_for_keyword_returns_markdown_with_example() {
        let hover = hover_for_keyword(
            "let value = 1",
            Position {
                line: 0,
                character: 1,
            },
        )
        .expect("expected hover info for keyword");

        let HoverContents::Markup(markup) = hover.contents else {
            panic!("expected markdown hover");
        };
        assert!(markup.value.contains("**Keyword** `let`"));
        assert!(markup.value.contains("```halcyon"));
    }

    #[test]
    fn hover_for_operator_keywords_returns_none() {
        let hover = hover_for_keyword(
            "left and right",
            Position {
                line: 0,
                character: 6,
            },
        );

        assert!(hover.is_none());
    }

    #[test]
    fn hover_for_keyword_handles_unicode_prefix_offsets() {
        let hover = hover_for_keyword(
            "\u{1F600} let value = 1",
            Position {
                line: 0,
                character: 4,
            },
        )
        .expect("expected hover info for keyword after emoji");

        let range = hover.range.expect("keyword hover should include a range");
        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 3);
        assert_eq!(range.end.character, 6);
    }

    #[test]
    fn hover_for_keyword_rejects_mid_surrogate_cursor_positions() {
        let hover = hover_for_keyword(
            "\u{1F600}",
            Position {
                line: 0,
                character: 1,
            },
        );

        assert!(hover.is_none());
    }
}
