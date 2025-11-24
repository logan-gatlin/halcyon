use crate::{
    IntoLog,
    WithContext,
};

use super::*;

#[derive(Debug, Clone)]
pub struct InstantiationError {
    pub expected: usize,
    pub provided: usize,
}

impl IntoLog for InstantiationError {
    type OutT = ();

    fn into_log(
        self,
        logger: &mut crate::Logger,
        span: crate::Span,
    ) -> Self::OutT {
        let err = logger
            .error("Type instantiation error")
            .primary("The instantiation here", span);
        if self.expected == 0 {
            err.note("This type has no parameters")
        } else if self.expected > self.provided {
            err.note(format!(
                "Types cannot be partially instantiated. This type requires {} parameters, but {} were provided",
                 self.expected, self.provided))
        } else {
            err.note(format!(
                "Too many parameters. This type requires {} parameters, but {} were provided",
                self.expected, self.provided
            ))
        }.done();
    }
}

#[derive(Debug, Clone)]
pub struct AbstractType {
    pub arity: usize,
    pub base: Type,
}

impl AbstractType {
    pub fn instantiate(
        mut self,
        types: &[Type],
        fresh_type_variable: impl FnMut() -> usize,
    ) -> Result<Type, InstantiationError> {
        if self.arity != types.len() {
            Err(InstantiationError {
                expected: self.arity,
                provided: types.len(),
            })
        } else {
            self.base.freshen_type_variables(fresh_type_variable);
            Ok(self.base)
        }
    }
}
