use super::*;

impl<T: Encode<Function>> Encode<IrNode> for FunctionEncoder<'_, T> {
    fn encode(&mut self, node: IrNode) -> &mut Self {
        use IrKind::*;
        match node.inner.inner {
            Let {
                assignee,
                value,
                in_,
            } => todo!(),
            Immediate(const_value) => todo!(),
            Identifier(path) => todo!(),
            Tuple(typeds) => todo!(),
            Struct {
                field_names,
                field_values,
            } => todo!(),
            Field { of, index } => todo!(),
            Function {
                parameter_name,
                parameter_type,
                captures,
                capture_types,
                body,
            } => todo!(),
            Call {
                callee,
                argument,
                argument_first,
            } => todo!(),
            If {
                predicate,
                then,
                else_,
            } => todo!(),
            Match {
                scrutinee,
                predicates,
                branches,
            } => todo!(),
            ImportedSymbol(path, _) => todo!(),
        }
    }
}
