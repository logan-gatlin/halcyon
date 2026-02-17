mod expression;
mod module;
mod pattern;
mod type_expr;

use super::SyntaxKind;
use super::parser::Parser;

/// Parse a complete source file: zero or more module declarations.
pub fn source_file(p: &mut Parser<'_, '_>) {
    let m = p.start_node_before_trivia(SyntaxKind::SOURCE_FILE);
    while !p.at_end() {
        if p.at(SyntaxKind::MODULE_KW) {
            module::module(p);
        } else {
            module::statement(p);
        }
    }
    // Attach any trailing trivia to the root node.
    p.skip_trivia();
    p.finish_node(m);
}
