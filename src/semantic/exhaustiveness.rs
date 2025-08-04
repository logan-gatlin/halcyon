use super::*;

fn is_exhaustive(pat: &Pattern) -> bool {
    match &pat.kind {
        PatternKind::Name(_) => true,
        PatternKind::Tuple(patterns) => patterns.iter().fold(true, |b, p| b | is_exhaustive(p)),
        PatternKind::Literal(ConstValue::Unit) => true,
        PatternKind::Literal(_) => false,
    }
}

pub fn check(module: &IrModule) -> Result<()> {
    for item in &module.items {
        if let ModuleItem::Let(assignee, _) = &item {
            if !is_exhaustive(assignee) {
                return Err(lint(
                    TypeLint::NonExhaustive,
                    assignee.span,
                    [format!("{}", assignee.type_.borrow())],
                ));
            }
        }
    }
    for node in &module.nodes {
        if let IrKind::Declaration { assignee, .. } = &node.kind {
            if !is_exhaustive(assignee) {
                return Err(lint(
                    TypeLint::NonExhaustive,
                    assignee.span,
                    [format!("{}", assignee.type_.borrow())],
                ));
            }
        }
    }
    Ok(())
}
