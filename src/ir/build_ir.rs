use crate::parse::*;
use crate::{
    Logger,
    WithSpan,
};
use indoc::indoc;

use super::*;

#[derive(Debug, Clone)]
struct TermInfo {
    depth: usize,
    is_finalized: bool,
    is_global: bool,
}

#[derive(Debug)]
pub struct Builder<'a> {
    name_map: CanonicalMap,
    module_name: String,
    logger: &'a mut Logger,
    symbols: SymbolTable,
    constructors: Vec<Constructor>,
    type_definitions: Vec<Typed<Spanned<Path>>>,
    let_definitions: Vec<(Pattern, IrNode)>,
    term_info: HashMap<Path, TermInfo>,
    captures: Vec<Vec<Path>>,
}

impl<'a> Builder<'a> {
    fn capture_term(
        &mut self,
        path: Path,
    ) {
        if let Some(c) = self.captures.last_mut() {
            c.push(path);
        }
    }
    fn define_name(
        &mut self,
        name: Spanned<String>,
        namespace: NameSpace,
        is_global: bool,
    ) -> Option<Path> {
        let path = self.name_map.define(name, namespace, is_global)?;
        if namespace == NameSpace::Term {
            self.term_info.insert(
                path.clone(),
                TermInfo {
                    depth: self.captures.len(),
                    is_finalized: false,
                    is_global,
                },
            );
        }
        Some(path)
    }
    fn query_name(
        &mut self,
        name: Spanned<String>,
        namespace: NameSpace,
    ) -> Option<Path> {
        let path = self.name_map.get(name.clone(), namespace)?.clone();
        if namespace == NameSpace::Term {
            let Some(TermInfo {
                depth: current_depth,
                is_finalized,
                is_global,
            }) = self.term_info.get(&path).cloned()
            else {
                unreachable!()
            };
            let depth = self.captures.len();
            if !is_finalized && current_depth <= depth {
                self.logger
                    .error("Definition is circular")
                    .primary("Usage here causes a cycle", name.span)
                    .note("You may have meant to write a recursive function instead")
                    .done();
                return None;
            }
            if !is_global {
                for capture in depth..current_depth {
                    if !self.captures[capture].contains(&path) {
                        self.captures[capture].push(path.clone());
                    }
                }
            }
        }
        Some(path)
    }
    pub fn build_ir(
        logger: &'a mut Logger,
        module: ParsedModule,
    ) -> IrModule {
        use ModuleStatementKind::*;
        let module_name = module.inner.name.inner.clone();
        let mut this = Self {
            logger,
            name_map: CanonicalMap::new(module_name.clone(), logger.spawn_new()),
            module_name,
            symbols: Default::default(),
            constructors: Default::default(),
            type_definitions: Default::default(),
            let_definitions: Default::default(),
            captures: Default::default(),
            term_info: Default::default(),
        };
        for item in module.inner.contents {
            match item.inner {
                DocComment(_) => {}
                Let { assignee, value } => {
                    let assignee = this.pattern(assignee, true);
                    let value = this.expr(*value);
                    match (assignee, value) {
                        (Ok(assignee), Ok(value)) => this.let_definitions.push((assignee, value)),
                        (Err(e1), Err(e2)) => {
                            this.logger.log(e1);
                            this.logger.log(e2);
                        }
                        (Err(e), _) | (_, Err(e)) => this.logger.log(e),
                    }
                }
                Type { .. } => todo!(),
            }
        }
        IrModule {
            module_name: this.module_name,
            constructors: this.constructors,
            type_definitions: this.type_definitions,
            let_definitions: this.let_definitions,
        }
    }
    fn pattern(
        &mut self,
        pat: PatternExpression,
        is_global: bool,
    ) -> Option<Pattern> {
        use PatternExpressionKind::*;
        let span = pat.span;
        Some(
            match pat.inner {
                Literal(literal) => PatternKind::Immediate(lit(literal)?),
                Identifier(name) if name == "_" => PatternKind::Hole,
                Identifier(name) => {
                    PatternKind::Identifier(self.define_name(
                        name.with_span(span),
                        NameSpace::Term,
                        is_global,
                    )?)
                }
                ModulePath(..) => todo!(),
                Tuple(pats) => {
                    PatternKind::Tuple(
                        pats.into_iter()
                            .map(|p| self.pattern(p, is_global))
                            .collect::<Option<_>>()?,
                    )
                }
                Array(array_pat) => {
                    todo!()
                }
                Constructor(..) => todo!(),
                TypeHint(pat, type_) => {
                    PatternKind::TypeHint(
                        self.pattern(*pat, is_global)?.into(),
                        self.type_expr(*type_)?,
                    )
                }
            }
            .with_span(span)
            .with_type(Type::Any),
        )
    }
    fn type_expr(
        &mut self,
        _expr: TypeExpression,
    ) -> Option<Type> {
        todo!()
    }
    fn expr(
        &mut self,
        expr: ValueExpression,
    ) -> Option<IrNode> {
        let span = expr.span;
        let unreachable = |span| -> Box<_> {
            IrKind::Call {
                callee: IrKind::Identifier(Path::new("std", "panic"))
                    .with_span(span)
                    .with_type(Type::Any)
                    .into(),
                argument: IrKind::Immediate(ConstValue::Unit)
                    .with_span(span)
                    .with_type(Type::Any)
                    .into(),
            }
            .with_span(span)
            .with_type(Type::Any)
            .into()
        };
        use ValueExpressionKind::*;
        Some(
            match expr.inner {
                Let {
                    assignee,
                    value,
                    in_,
                } => {
                    let assignee = self.pattern(assignee, false)?;
                    if let Some(span) = assignee.find_refutable_pattern() {
                        return Err(refutable_let_err().span(span));
                    }
                    let value = self.expr(*value)?.into();
                    let in_ = self.expr(*in_)?.into();
                    self.name_map.end_local_scopes(assignee.introduced_names());
                    IrKind::Let {
                        assignee,
                        value,
                        then: in_,
                        else_: unreachable(span),
                    }
                }
                Literal(literal) => IrKind::Immediate(lit(literal).map_err(|e| e.span(span))?),
                Identifier(name) => {
                    let path = self.query_name(name, NameSpace::Term)?.clone();
                    self.capture_term(path.clone());
                    IrKind::Identifier(path)
                }
                Binary { op, left, right } => {
                    IrKind::Call {
                        callee: IrKind::Call {
                            callee: IrKind::Identifier(op.path())
                                .with_span(span)
                                .with_type(Type::Any)
                                .into(),
                            argument: self.expr(*left)?.into(),
                        }
                        .with_span(span)
                        .with_type(Type::Any)
                        .into(),
                        argument: self.expr(*right)?.into(),
                    }
                }
                BinaryOp(binary_op) => IrKind::Identifier(binary_op.path()),
                Unary { op, child } => {
                    IrKind::Call {
                        callee: IrKind::Identifier(op.path())
                            .with_span(span)
                            .with_type(Type::Any)
                            .into(),
                        argument: self.expr(*child)?.into(),
                    }
                }
                UnaryOp(unary_op) => IrKind::Identifier(unary_op.path()),
                FunctionDef { .. } => todo!(),
                FunctionShorthand { .. } => {
                    todo!()
                }
                FunctionCall { callee, argument } => {
                    IrKind::Call {
                        callee: self.expr(*callee)?.into(),
                        argument: self.expr(*argument)?.into(),
                    }
                }
                If {
                    predicate,
                    then,
                    else_,
                } => {
                    IrKind::Let {
                        assignee: PatternKind::Immediate(ConstValue::Boolean(true))
                            .with_span(span)
                            .with_type(Type::Any),
                        value: self.expr(*predicate)?.into(),
                        then: self.expr(*then)?.into(),
                        else_: self.expr(*else_)?.into(),
                    }
                }
                Match {
                    scrutinee,
                    predicates,
                    branches,
                } => {
                    let scrutinee = self.expr(*scrutinee)?;
                    let scrutinee_path = self
                        .name_map
                        .define_local("@scrutinee".to_string(), NameSpace::Term);
                    let mut current = unreachable(span);
                    for (predicate, branch) in predicates.into_iter().zip(branches).rev() {
                        let assignee = self.pattern(predicate, false)?;
                        let in_: Box<_> = self.expr(branch)?.into();
                        let predicate_span = assignee.span;
                        let branch_span = in_.span;
                        self.name_map.end_local_scopes(assignee.introduced_names());
                        current = IrKind::Let {
                            assignee,
                            value: IrKind::Identifier(scrutinee_path.clone())
                                .with_span(predicate_span)
                                .with_type(Type::Any)
                                .into(),
                            then: in_,
                            else_: current,
                        }
                        .with_span(predicate_span + branch_span)
                        .with_type(Type::Any)
                        .into();
                    }
                    // End `@scrutinee` scope
                    self.name_map.end_local_scopes(1);
                    IrKind::Let {
                        assignee: PatternKind::Identifier(scrutinee_path)
                            .with_span(scrutinee.span)
                            .with_type(Type::Any),
                        value: scrutinee.into(),
                        then: current,
                        else_: unreachable(span),
                    }
                }
                Tuple(items) => {
                    IrKind::Tuple(items.into_iter().map(|i| self.expr(i)).try_collect()?)
                }
                Array(array_elems) => {
                    let mut current = IrKind::Identifier(Path::new("array", "empty"))
                        .with_span(span)
                        .with_type(Type::Any);
                    for elem in array_elems {
                        match elem {
                            ArrayInner::Splat(concat) => {
                                let concat_span = concat.span;
                                current = IrKind::Call {
                                    callee: IrKind::Call {
                                        callee: IrKind::Identifier(Path::new(
                                            "array",
                                            "concatenate",
                                        ))
                                        .with_span(concat_span)
                                        .with_type(Type::Any)
                                        .into(),
                                        argument: current.into(),
                                    }
                                    .with_span(concat_span)
                                    .with_type(Type::Any)
                                    .into(),
                                    argument: self.expr(concat)?.into(),
                                }
                                .with_span(concat_span)
                                .with_type(Type::Any)
                            }
                            ArrayInner::Single(push) => {
                                let push_span = push.span;
                                current = IrKind::Call {
                                    callee: IrKind::Call {
                                        callee: IrKind::Identifier(Path::new("array", "push"))
                                            .with_span(push_span)
                                            .with_type(Type::Any)
                                            .into(),
                                        argument: current.into(),
                                    }
                                    .with_span(push_span)
                                    .with_type(Type::Any)
                                    .into(),
                                    argument: self.expr(push)?.into(),
                                }
                                .with_span(push_span)
                                .with_type(Type::Any)
                            }
                        }
                    }
                    return Ok(current);
                }
                StructureLiteral(_) => {
                    todo!()
                }
                Field { lhs, rhs } => {
                    IrKind::Field {
                        of: self.expr(*lhs)?.into(),
                        index: rhs,
                    }
                }
                ModulePath(items) => {
                    match items.as_slice() {
                        [a, b] => {
                            let path = Path::new(a, b);
                            if self.symbols.terms.contains_key(&path) {
                                IrKind::Identifier(path)
                            } else {
                                return Err(
                                    err(format!("There is no symbol {path} in scope.")).span(span)
                                );
                            }
                        }
                        s => {
                            let len = s.len();
                            let s = s.join("::");
                            return Err(err(format!(
                                "There is no symbol {s} in scope. \
Modules cannot be nested, so all paths should consist of two parts: `a::b`. \
This path has {len} parts, which is not possible."
                            ))
                            .span(span));
                        }
                    }
                }
            }
            .with_span(span)
            .with_type(Type::Any),
        )
    }
}
