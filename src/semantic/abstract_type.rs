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
    pub variables: Box<[TypeVariable]>,
    pub base: Type,
}

impl AbstractType {
    pub fn instantiate(
        self,
        types: &[Type],
    ) -> Result<Type, InstantiationError> {
        let expected = self.variables.len();
        if expected != types.len() {
            Err(InstantiationError {
                expected,
                provided: types.len(),
            })
        } else {
            let mut type_ = self.base;
            substitute_type_variables(&mut type_, &self.variables, types);
            Ok(type_)
        }
    }

    pub fn try_instantiate(
        &self,
        types: &[Type],
    ) -> Result<(), InstantiationError> {
        let expected = self.variables.len();
        if expected != types.len() {
            Err(InstantiationError {
                expected,
                provided: types.len(),
            })
        } else {
            Ok(())
        }
    }
}
