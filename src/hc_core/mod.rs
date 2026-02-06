/*!
    The `core` module contains symbols that are required by the compiler.
    These include the standard types, operators, and wrappers for important
    WebAssembly functionality.
*/

use crate::ir::*;
use crate::semantic::{
    AbstractType,
    Type,
    freshen_type_variables,
};

pub const CORE_MODULE_NAME: &str = "core";

fn core(s: impl Into<String>) -> Path {
    Path::new(CORE_MODULE_NAME, s)
}

pub fn core_symbol_table() -> SymbolTable {
    let mut table = SymbolTable::default();
    // Primitive types
    table.types.extend(
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
    let array_type_variable = table.fresh_tv();
    table.types.insert(
        core("array"),
        AbstractType {
            variables: [array_type_variable].into(),
            base: Type::Array(Type::Variable(array_type_variable).into()),
        },
    );
    let function_parameter_type_variable = table.fresh_tv();
    let function_return_type_variable = table.fresh_tv();
    table.types.insert(
        core("function"),
        AbstractType {
            variables: [
                function_parameter_type_variable,
                function_return_type_variable,
            ]
            .into(),
            base: Type::func(
                Type::Variable(function_parameter_type_variable),
                Type::Variable(function_return_type_variable),
            ),
        },
    );
    // Operators
    let ops = crate::operator::BinaryOp::all()
        .into_iter()
        .map(|op| (op.path(), op.get_type()))
        .collect::<Vec<_>>();
    table.terms.extend(ops);
    let ops = crate::operator::UnaryOp::all()
        .into_iter()
        .map(|op| {
            let mut type_ = op.get_type();
            freshen_type_variables(&mut type_, &table);
            (op.path(), op.get_type())
        })
        .collect::<Vec<_>>();
    table.terms.extend(ops);
    // terms
    let array_type_variable = table.fresh_tv();
    let generic_array_type = Type::Array(Type::Variable(array_type_variable).into());
    table.terms.extend(
        [
            ("empty_array", generic_array_type.clone()),
            (
                "push_array",
                Type::func(
                    Type::Variable(array_type_variable),
                    Type::func(generic_array_type.clone(), generic_array_type.clone()),
                ),
            ),
            (
                "concatenate_arrays",
                Type::func(
                    generic_array_type.clone(),
                    Type::func(generic_array_type.clone(), generic_array_type.clone()),
                ),
            ),
        ]
        .into_iter()
        .map(|(name, type_)| (core(name), type_)),
    );
    table
}
