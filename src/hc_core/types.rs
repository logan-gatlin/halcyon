use super::*;

pub fn type_definitions(syms: &mut SymbolTable) {
    // Primitive types
    syms.types.extend(
        [
            ("unit", Type::Unit),
            ("integer", Type::Integer),
            ("real", Type::Real),
            ("boolean", Type::Boolean),
            ("string", Type::String),
            ("glyph", Type::Glyph),
        ]
        .into_iter()
        .map(|(name, base)| {
            (
                core(name),
                AbstractType {
                    variables: [].into(),
                    base,
                },
            )
        }),
    );
    syms.types.insert(
        core("array"),
        AbstractType {
            variables: [0].into(),
            base: Type::Array(Type::Variable(0).into()),
        },
    );
    syms.types.insert(
        core("function"),
        AbstractType {
            variables: [0, 1].into(),
            base: Type::func(Type::Variable(0), Type::Variable(1)),
        },
    );
}
