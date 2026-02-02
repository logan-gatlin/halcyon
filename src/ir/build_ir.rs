use crate::hc_core::CORE_MODULE_NAME;
use crate::parse::*;
use crate::{
    IntoLog,
    Logger,
    WithSpan,
};

use super::*;

#[derive(Debug, Clone)]
pub struct TermInfo {
    pub depth: usize,
    pub is_finalized: bool,
    pub is_global: bool,
}

#[derive(Debug)]
pub struct Builder<'a> {
    pub name_map: CanonicalMap,
    pub module_name: String,
    pub logger: &'a mut Logger,
    pub symbols: &'a mut SymbolTable,
    pub local_types: HashMap<Path, TypeVariable>,
    pub code: Vec<IrNode>,
    pub term_info: HashMap<Path, TermInfo>,
    pub captures: Vec<Vec<Path>>,
}

impl<'a> Builder<'a> {
    fn begin_capture(&mut self) {
        self.captures.push(vec![]);
    }
    fn end_capture(&mut self) -> Vec<Path> {
        self.captures.pop().unwrap_or_else(|| unreachable!())
    }
    pub fn finalize_name(
        &mut self,
        path: &Path,
    ) {
        if let Some(term) = self.term_info.get_mut(path) {
            term.is_finalized = true;
        }
    }
    fn refutable_let_err(
        &mut self,
        span: Span,
    ) {
        self.logger
            .error("Pattern is not exhaustive")
            .primary("This `let` binding must cover all possible cases", span)
            .done();
    }
    pub fn define_name(
        &mut self,
        name: Spanned<String>,
        namespace: NameSpace,
        is_global: bool,
    ) -> Option<Path> {
        let path = self.name_map.define(name, namespace, is_global).done()?;
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
    pub fn query_name(
        &'_ mut self,
        name: Spanned<String>,
        namespace: NameSpace,
    ) -> Result<'_, Path> {
        let path = self.name_map.get_name(name.clone(), namespace)?.clone();
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
                return Err(self
                    .logger
                    .error("Definition is circular")
                    .primary(format!("Usage of `{name}` here causes a cycle"), name.span)
                    .note("You may have meant to write a recursive function instead"));
            }
            if !is_global {
                for capture in current_depth..depth {
                    if !self.captures[capture].contains(&path) {
                        self.captures[capture].push(path.clone());
                    }
                }
            }
        }
        Ok(path)
    }
    pub fn query_path(
        &'_ mut self,
        Spanned {
            inner: Path { major, minor },
            span,
        }: &Spanned<Path>,
        namespace: NameSpace,
    ) -> Result<'_, ()> {
        if major == &self.module_name {
            self.name_map
                .get_global_name(&minor.to_owned().with_span(*span), namespace)
                .map(|_| ())?;
            // Duplicated check for circular names
            let path = Path::new(major, minor);
            let Some(TermInfo {
                depth: current_depth,
                is_finalized,
                ..
            }) = self.term_info.get(&path).cloned()
            else {
                unreachable!()
            };
            let depth = self.captures.len();
            if !is_finalized && current_depth <= depth {
                return Err(self
                    .logger
                    .error("Definition is circular")
                    .primary(format!("Usage of `{path}` here causes a cycle"), *span)
                    .note("You may have meant to write a recursive function instead"));
            }
            Ok(())
        } else {
            let path = Path::new(major, minor);
            self.symbols
                .contains_symbol(&path, namespace)
                .then_some(())
                .ok_or_else(|| self.name_map.unknown_name(namespace, path.with_span(*span)))
        }
    }
    pub fn build_ir(
        logger: &'a mut Logger,
        symbols: &'a mut SymbolTable,
        module: ParsedModule,
    ) -> Module {
        use ModuleStatementKind::*;
        let module_name = module.inner.name.inner.clone();
        let mut this = Self {
            name_map: CanonicalMap::new(module_name.clone(), logger.spawn_new()),
            logger,
            symbols,
            module_name,
            local_types: Default::default(),
            code: Default::default(),
            captures: Default::default(),
            term_info: Default::default(),
        };
        let mut types = HashMap::new();
        for item in module.inner.contents {
            match item.inner {
                DocComment(_) => {}
                Let { assignee, value } => {
                    let span = assignee.span + value.span;
                    let expr = this.expr(
                        ValueExpressionKind::Let {
                            assignee,
                            is_global: true,
                            value,
                            in_: ValueExpressionKind::Literal(Literal::Unit)
                                .with_span(span)
                                .into(),
                        }
                        .with_span(span),
                    );
                    if let Some(expr) = expr {
                        this.code.push(expr);
                    }
                }
                Type { assignee, value } => {
                    let assignee_span = assignee.span;
                    let Some(path) = this.define_name(assignee, NameSpace::Type, true) else {
                        continue;
                    };
                    let tv = this.symbols.fresh_tv();
                    this.symbols.types.insert(
                        path.clone(),
                        AbstractType {
                            variables: [].into(),
                            base: crate::semantic::Type::Variable(tv),
                        },
                    );
                    let Some(mut at) = this.type_definition(path.clone(), *value, &[]) else {
                        continue;
                    };
                    if at.base.always_contains_type_variable(tv) {
                        this.logger
                            .error("Uninhabited type")
                            .primary(
                                "It is impossible for a term with this type to exist because it contains itself",
                                assignee_span,
                            )
                            .note("Cyclical structures are only possible using mutation, which is not allowed")
                            .done();
                        continue;
                    }
                    substitute_type_variables(
                        &mut at.base,
                        &[Solution {
                            old: tv,
                            new: crate::semantic::Type::Instantiation(path.clone(), vec![]),
                        }],
                    );
                    types.insert(path.clone(), at.clone());
                    this.symbols.types.insert(path, at);
                }
            }
        }
        this.logger.merge_with(this.name_map.logger);
        Module {
            name: this.module_name,
            types,
            code: this.code,
        }
    }
    pub fn literal(
        &mut self,
        Spanned {
            inner: literal,
            span,
        }: Spanned<Literal>,
    ) -> Option<ConstValue> {
        Some(match literal {
            Literal::Unit => ConstValue::Unit,
            Literal::Integer(i, base) => {
                ConstValue::Integer(
                    i64::from_str_radix(&i, base as u32).into_log(self.logger, span)?,
                )
            }
            Literal::Real(r) => ConstValue::Real(r.parse().into_log(self.logger, span)?),
            Literal::String(s) => ConstValue::String(s),
            Literal::Glyph(g) => ConstValue::Glyph(g),
            Literal::Boolean(b) => ConstValue::Boolean(b),
        })
    }
    fn curry(
        &mut self,
        mut arguments: impl Iterator<Item = (Spanned<String>, Option<TypeExpression>)>,
        body: Box<ValueExpression>,
        span: Span,
    ) -> Option<Box<IrNode>> {
        Some(Box::new(
            match arguments.next() {
                Some((argument, type_)) => {
                    self.begin_capture();
                    let parameter_span = argument.span;
                    let parameter_name = self
                        .define_name(argument.clone(), NameSpace::Term, false)?
                        .with_span(parameter_span);
                    self.finalize_name(&parameter_name.inner);
                    let body = self.curry(arguments, body, span)?;
                    let captures = self.end_capture();
                    self.name_map.end_local_scopes(1);
                    IrKind::Function {
                        parameter_name,
                        parameter_type: match type_ {
                            Some(t) => Some(self.type_expr(t)?),
                            None => None,
                        },
                        capture_types: vec![Type::Any; captures.len()],
                        captures,
                        body,
                    }
                }
                None => return self.expr(*body).map(Box::new),
            }
            .with_span(span)
            .with_type(Type::Any),
        ))
    }
    fn expr(
        &mut self,
        expr: ValueExpression,
    ) -> Option<IrNode> {
        let span = expr.span;
        use ValueExpressionKind::*;
        Some(
            match expr.inner {
                Let {
                    assignee,
                    is_global,
                    value,
                    in_,
                } => {
                    let mut assignee = self.pattern(assignee, is_global)?;
                    if let Some(span) = assignee.find_refutable_pattern() {
                        self.refutable_let_err(span);
                        return None;
                    }
                    let value = self.expr(*value)?.into();
                    assignee.visit(|p: &mut Path| {
                        self.finalize_name(p);
                    });
                    let in_ = self.expr(*in_)?.into();
                    if !is_global {
                        self.name_map.end_local_scopes(assignee.introduced_names());
                    }
                    IrKind::Let {
                        assignee,
                        is_global,
                        value,
                        then: in_,
                        else_: IrKind::Unreachable
                            .with_span(span)
                            .with_type(Type::Any)
                            .into(),
                    }
                }
                Literal(literal) => IrKind::Immediate(self.literal(literal.with_span(span))?),
                Identifier(name) => {
                    let path = self
                        .query_name(name.with_span(span), NameSpace::Term)
                        .done()?
                        .clone();
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
                FunctionDef {
                    parameters,
                    types,
                    body,
                } => {
                    return if parameters.is_empty() {
                        let parameter = "<parameter>".to_string();
                        self.expr(
                            ValueExpressionKind::FunctionDef {
                                parameters: vec![parameter.with_span(span)],
                                types: vec![Some(TypeExpressionKind::Unit.with_span(span))],
                                body,
                            }
                            .with_span(span),
                        )
                    } else {
                        Some(*self.curry(parameters.into_iter().zip(types), body, span)?)
                    };
                }
                FunctionShorthand {
                    predicates,
                    branches,
                } => {
                    let parameter = "<parameter>".to_string();
                    return self.expr(
                        FunctionDef {
                            parameters: vec![parameter.clone().with_span(span)],
                            types: vec![None],
                            body: ValueExpressionKind::Match {
                                scrutinee: ValueExpressionKind::Identifier(parameter)
                                    .with_span(span)
                                    .into(),
                                predicates,
                                branches,
                            }
                            .with_span(span)
                            .into(),
                        }
                        .with_span(span),
                    );
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
                        is_global: false,
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
                    let scrutinee_path = self.name_map.define_local(
                        "<scrutinee>".to_string().with_span(scrutinee.span),
                        NameSpace::Term,
                    );
                    let mut current = IrKind::Unreachable
                        .with_span(span)
                        .with_type(Type::Any)
                        .into();
                    for (predicate, branch) in predicates.into_iter().zip(branches).rev() {
                        let mut assignee = self.pattern(predicate, false)?;
                        assignee.visit(|p| {
                            self.finalize_name(p);
                        });
                        let in_: Box<_> = self.expr(branch)?.into();
                        let predicate_span = assignee.span;
                        let branch_span = in_.span;
                        self.name_map.end_local_scopes(assignee.introduced_names());
                        current = IrKind::Let {
                            assignee,
                            is_global: false,
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
                        is_global: false,
                        value: scrutinee.into(),
                        then: current,
                        else_: IrKind::Unreachable
                            .with_span(span)
                            .with_type(Type::Any)
                            .into(),
                    }
                }
                Tuple(items) => {
                    IrKind::Tuple(
                        items
                            .into_iter()
                            .map(|i| self.expr(i))
                            .collect::<Option<_>>()?,
                    )
                }
                Array(array_elems) => {
                    let mut current =
                        IrKind::Identifier(Path::new(CORE_MODULE_NAME, "empty_array"))
                            .with_span(span)
                            .with_type(Type::Any);
                    for elem in array_elems {
                        match elem {
                            ArrayInner::Splat(concat) => {
                                let concat_span = concat.span;
                                current = IrKind::Call {
                                    callee: IrKind::Call {
                                        callee: IrKind::Identifier(Path::new(
                                            CORE_MODULE_NAME,
                                            "concatenate_arrays",
                                        ))
                                        .with_span(concat_span)
                                        .with_type(Type::Any)
                                        .into(),
                                        argument: self.expr(concat)?.into(),
                                    }
                                    .with_span(concat_span)
                                    .with_type(Type::Any)
                                    .into(),
                                    argument: current.into(),
                                }
                                .with_span(concat_span)
                                .with_type(Type::Any)
                            }
                            ArrayInner::Single(push) => {
                                let push_span = push.span;
                                current = IrKind::Call {
                                    callee: IrKind::Call {
                                        callee: IrKind::Identifier(Path::new(
                                            CORE_MODULE_NAME,
                                            "push_array",
                                        ))
                                        .with_span(push_span)
                                        .with_type(Type::Any)
                                        .into(),
                                        argument: self.expr(push)?.into(),
                                    }
                                    .with_span(push_span)
                                    .with_type(Type::Any)
                                    .into(),
                                    argument: current.into(),
                                }
                                .with_span(push_span)
                                .with_type(Type::Any)
                            }
                        }
                    }
                    return Some(current);
                }
                StructureLiteral(map) => {
                    let mut new_map = IndexMap::new();
                    for (key, value) in map {
                        let value = self.expr(value)?;
                        new_map.insert(key, value);
                    }
                    IrKind::Struct(new_map)
                }
                Field { lhs, rhs } => {
                    IrKind::Field {
                        of: self.expr(*lhs)?.into(),
                        index: rhs,
                    }
                }
                ModulePath(major, minor) => {
                    let path = Path::new(major, minor);
                    let is_external = self.symbols.terms.contains_key(&path);
                    if let Err(e) = self.query_path(&path.clone().with_span(span), NameSpace::Term)
                        && !is_external
                    {
                        e.done();
                        return None;
                    }
                    IrKind::Identifier(path)
                }
            }
            .with_span(span)
            .with_type(Type::Any),
        )
    }
}
