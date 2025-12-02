/*!
    The semantic module infers and checks types. To do this, we use a variation
    of the Hindley Milner system. The `infer` module gives the program temporary
    type variables, and generates a set of constraints for those variables.
    The `constraint` module solves those constraints, and generates a solution.
    A solution is a mapping from type variables to concrete types.
*/

mod abstract_type;
mod constraint;
mod types;
use crate::ir::*;
use crate::{
    Logger,
    Visit,
};
use std::collections::HashSet;

use crate::Span;
pub use abstract_type::*;
pub use constraint::*;
pub use types::*;

use crate::SymbolTable;

/// The set of type variables that are free in the current environment.
/// The difference between free and non-free variables is that free variables
/// refer to *one* type that is yet to be determined. Non-free variables are generic,
/// they may refer to a different type every time they are used.
pub type FreeVariableSet = HashSet<TypeVariable>;

struct Environment<'a> {
    symbols: &'a mut SymbolTable,
    constraints: &'a mut ConstraintSet,
    free: FreeVariableSet,
}

fn infer_pattern(
    pat: &mut Pattern,
    env: &mut Environment,
) {
    let span = pat.span;
    let type_ = match &mut pat.inner.inner {
        PatternKind::Hole => {
            let fresh_tv = env.symbols.fresh_tv();
            env.free.insert(fresh_tv);
            Type::Variable(fresh_tv)
        }
        PatternKind::Identifier(path) => {
            let fresh_tv = env.symbols.fresh_tv();
            env.free.insert(fresh_tv);
            let type_ = Type::Variable(fresh_tv);
            env.symbols.terms.insert(path.clone(), type_.clone());
            type_
        }
        PatternKind::Tuple(pats) => {
            let mut types = Vec::with_capacity(pats.len());
            for p in pats.iter_mut() {
                infer_pattern(p, env);
                types.push(p.type_.clone());
            }
            Type::Tuple(types)
        }
        PatternKind::Array {
            starting,
            glob,
            ending,
            ..
        } => {
            let fresh_tv = env.symbols.fresh_tv();
            env.free.insert(fresh_tv);
            let type_ = Type::Variable(fresh_tv);
            for pat in starting.iter_mut().chain(ending.iter_mut()) {
                infer_pattern(pat, env);
                env.constraints.equality.push(EqualityConstraint {
                    left: pat.type_.clone(),
                    right: type_.clone(),
                    span: pat.span,
                });
            }
            if let Some(glob) = glob {
                env.symbols.terms.insert(glob.clone(), type_.clone());
            }
            type_
        }
        PatternKind::Constructor(constructor, typed) => todo!(),
        PatternKind::Immediate(const_value) => const_value.type_of(),
        PatternKind::TypeHint(pat, t) => {
            infer_pattern(pat, env);
            env.constraints.equality.push(EqualityConstraint {
                left: pat.type_.clone(),
                right: t.clone(),
                span,
            });
            t.clone()
        }
    };
    pat.type_ = type_;
}

fn infer_ir<'a, 'b, 'c>(
    ir: &'a mut IrNode,
    env: &'b mut Environment<'c>,
) {
    let span = ir.span;
    let type_ = match &mut ir.inner.inner {
        IrKind::Let {
            assignee,
            value,
            then,
            else_,
            ..
        } => {
            let new_env = &mut Environment {
                symbols: env.symbols,
                constraints: env.constraints,
                free: env.free.clone(),
            };
            infer_pattern(assignee, new_env);
            infer_ir(value, new_env);
            infer_ir(then, new_env);
            infer_ir(else_, new_env);
            env.constraints.equality.extend_from_slice(&[
                EqualityConstraint {
                    left: assignee.type_.clone(),
                    right: value.type_.clone(),
                    span: assignee.span,
                },
                EqualityConstraint {
                    left: then.type_.clone(),
                    right: else_.type_.clone(),
                    span,
                },
            ]);
            then.type_.clone()
        }
        IrKind::Immediate(const_value) => const_value.type_of(),
        IrKind::Identifier(path) => {
            let mut type_ = env.symbols.get_term(path).clone();
            type_.visit(|t: &mut Type| {
                if let Type::Variable(tv) = t
                    && !env.free.contains(tv)
                {
                    *tv = env.symbols.fresh_tv();
                }
            });
            type_
        }
        IrKind::Tuple(nodes) => {
            let mut types = Vec::with_capacity(nodes.len());
            for n in nodes {
                infer_ir(n, env);
                types.push(n.type_.clone());
            }
            Type::Tuple(types)
        }
        IrKind::Struct(index_map) => todo!(),
        IrKind::Field { of, index } => todo!(),
        IrKind::Function {
            parameter_name,
            parameter_type,
            captures,
            capture_types,
            body,
        } => todo!(),
        IrKind::Call { callee, argument } => {
            infer_ir(callee, env);
            infer_ir(argument, env);
            let return_type = Type::Variable(env.symbols.fresh_tv());
            let function_type = Type::func(argument.type_.clone(), return_type.clone());
            env.constraints.equality.push(EqualityConstraint {
                left: function_type.clone(),
                right: callee.type_.clone(),
                span,
            });
            return_type
        }
        IrKind::Semicolon(a, b) => {
            infer_ir(a, env);
            infer_ir(b, env);
            b.type_.clone()
        }
        IrKind::Unreachable => {
            let tv = env.symbols.fresh_tv();
            env.free.insert(tv);
            Type::Variable(tv)
        }
    };
    ir.type_ = type_;
}

pub fn analyze(
    module: &mut IrModule,
    symbols: &mut SymbolTable,
    logger: &mut Logger,
) {
    for ir in module.code.iter_mut() {
        let mut constraints = ConstraintSet::default();
        let mut env = Environment {
            symbols,
            constraints: &mut constraints,
            free: HashSet::new(),
        };
        infer_ir(ir, &mut env);
        let (old, new): (Vec<_>, Vec<_>) = solve_constraints(&mut env, logger)
            .into_iter()
            .map(|s| (s.old, s.new))
            .unzip();
        substitute_type_variables(ir, &old, &new);
        substitute_type_variables(&mut symbols.terms, &old, &new);
    }
}
