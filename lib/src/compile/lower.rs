use super::*;

impl Encode<IrNode> for FunctionEncoder<'_> {
    fn encode(&mut self, node: IrNode) -> &mut Self {
        self.encode(node.type_);
        match node.inner.inner {
            IrKind::Let {
                assignee,
                value,
                in_,
            } => todo!(),
            IrKind::Immediate(const_value) => todo!(),
            IrKind::Identifier(path) => todo!(),
            IrKind::Tuple(typeds) => todo!(),
            IrKind::Struct {
                field_names,
                field_values,
            } => todo!(),
            IrKind::Field { of, index } => todo!(),
            IrKind::Function {
                parameter_name,
                parameter_type,
                captures,
                capture_types,
                body,
            } => todo!(),
            IrKind::Call {
                callee,
                argument,
                argument_first,
            } => todo!(),
            IrKind::If {
                predicate,
                then,
                else_,
            } => todo!(),
            IrKind::Match {
                scrutinee,
                predicates,
                branches,
            } => todo!(),
            IrKind::ImportedSymbol(path, _) => todo!(),
        }
    }
}
