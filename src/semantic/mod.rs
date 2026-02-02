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
use std::collections::{
    HashMap,
    HashSet,
};

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
        PatternKind::Struct(map) => {
            let struct_type = Type::Variable(env.symbols.fresh_tv());
            for (key, val) in map {
                infer_pattern(val, env);
                env.constraints.struct_.push(StructConstraint::new(
                    struct_type.clone(),
                    key.inner.clone().with_type(val.type_.clone()),
                    span,
                ))
            }
            struct_type
        }
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            let fresh_tv = env.symbols.fresh_tv();
            env.free.insert(fresh_tv);
            let element_type = Type::Variable(fresh_tv);
            for pat in starting.iter_mut().chain(ending.iter_mut()) {
                infer_pattern(pat, env);
                env.constraints.equality.push(EqualityConstraint {
                    left: pat.type_.clone(),
                    right: element_type.clone(),
                    span: pat.span,
                });
            }
            if let Glob::Named(glob) = glob {
                // The glob has the type of the surrounding array, array's inner type
                env.symbols
                    .terms
                    .insert(glob.clone(), Type::Array(element_type.clone().into()));
            }
            Type::Array(element_type.into())
        }
        PatternKind::Constructor(Constructor::SumConstant { sum_type, .. }, pat) => {
            freshen_type_variables(sum_type, &env.symbols);
            infer_pattern(pat, env);
            env.constraints.equality.push(EqualityConstraint::new(
                pat.type_.clone(),
                Type::Unit,
                span,
            ));
            sum_type.clone()
        }
        PatternKind::Constructor(
            Constructor::SumFunction {
                sum_type,
                parameter_type: inner_type,
                ..
            },
            pat,
        ) => {
            let mut map = HashMap::new();
            freshen_type_variables_with_map(sum_type, &env.symbols, &mut map);
            freshen_type_variables_with_map(inner_type, &env.symbols, &mut map);
            infer_pattern(pat, env);
            env.constraints.equality.push(EqualityConstraint::new(
                inner_type.clone(),
                pat.type_.clone(),
                span,
            ));
            sum_type.clone()
        }
        PatternKind::Constructor(Constructor::Structure(t), pat) => {
            freshen_type_variables(t, &env.symbols);
            infer_pattern(pat, env);
            env.constraints.equality.push(EqualityConstraint::new(
                t.clone(),
                pat.type_.clone(),
                span,
            ));
            t.clone()
        }
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

pub fn freshen_nonfree_type_variables<T, S>(
    t: &mut T,
    tv_source: S,
    free: &FreeVariableSet,
) where
    T: Visit<Type>,
    S: TypeVariableSource,
{
    let mut map = HashMap::new();
    t.visit(|t: &mut Type| {
        if let Type::Variable(t) = t
            && !free.contains(t)
        {
            if let Some(tv) = map.get(t) {
                *t = *tv;
            } else {
                let tv = tv_source.fresh_tv();
                map.insert(*t, tv);
                *t = tv;
            }
        }
    })
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
            infer_ir(else_, env);
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
            let old_free = env.free.clone();
            freshen_nonfree_type_variables(&mut type_, &env.symbols, &old_free);
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
        IrKind::Struct(map) => {
            let tv = env.symbols.fresh_tv();
            env.free.insert(tv);
            let struct_t = Type::Variable(tv);
            for (name, value) in map {
                infer_ir(value, env);
                env.constraints.struct_.push(StructConstraint {
                    base: struct_t.clone(),
                    field: name.inner.clone().with_type(value.type_.clone()),
                    span: name.span + value.span,
                })
            }
            struct_t
        }
        IrKind::Field { of, index } => {
            infer_ir(of, env);
            let tv = env.symbols.fresh_tv();
            env.free.insert(tv);
            let field_t = Type::Variable(tv);
            env.constraints.struct_.push(StructConstraint::new(
                of.type_.clone(),
                index.inner.clone().with_type(field_t.clone()),
                span,
            ));
            field_t
        }
        IrKind::Function {
            parameter_name,
            parameter_type,
            captures,
            capture_types,
            body,
        } => {
            let new_env = &mut Environment {
                symbols: env.symbols,
                constraints: env.constraints,
                free: env.free.clone(),
            };
            let parameter_inferred_type = {
                let tv = new_env.symbols.fresh_tv();
                new_env.free.insert(tv);
                Type::Variable(tv)
            };
            new_env.symbols.terms.insert(
                parameter_name.inner.clone(),
                parameter_inferred_type.clone(),
            );
            if let Some(assert_type) = parameter_type {
                new_env.constraints.equality.push(EqualityConstraint::new(
                    assert_type.clone(),
                    parameter_inferred_type.clone(),
                    span,
                ));
            }
            *capture_types = captures
                .iter()
                .map(|c| {
                    let mut type_ = new_env.symbols.get_term(c).clone();
                    freshen_nonfree_type_variables(&mut type_, &new_env.symbols, &new_env.free);
                    type_
                })
                .collect();
            infer_ir(body, new_env);
            Type::func(parameter_inferred_type, body.type_.clone())
        }
        IrKind::Call { callee, argument } => {
            infer_ir(callee, env);
            infer_ir(argument, env);
            let tv = env.symbols.fresh_tv();
            env.free.insert(tv);
            let return_type = Type::Variable(tv);
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
    module: &mut Module,
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
        let solution = solve_constraints(&mut env, logger);
        substitute_type_variables(ir, &solution);
        substitute_type_variables(&mut symbols.terms, &solution);
    }
}
