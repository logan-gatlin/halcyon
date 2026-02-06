use crate::parse::*;
use crate::{
    Span,
    WithSpan,
};

use super::*;

pub type Pattern = Typed<Spanned<PatternKind>>;

#[derive(Debug, Clone)]
pub enum PatternKind {
    Hole,
    Identifier(Path),
    Tuple(Vec<Pattern>),
    Array {
        starting: Vec<Pattern>,
        glob: Glob,
        ending: Vec<Pattern>,
    },
    Struct(IndexMap<Spanned<String>, Pattern>),
    Constructor(Constructor, Box<Pattern>),
    Immediate(ConstValue),
    TypeHint(Box<Pattern>, Type),
}

/// Glob pattern in array destructuring.
#[derive(Debug, Clone)]
pub enum Glob {
    /// No glob present - exact length match required: `[a, b, c]`
    None,
    /// Unnamed glob - matches any remaining elements: `[a, .., b]`
    Unnamed,
    /// Named glob - captures remaining elements: `[a, ..rest, b]`
    Named(Path),
}

#[derive(Debug, Clone)]
pub enum Constructor {
    /// Sum type variant with no inner data, like `| None`
    SumConstant { tag: usize, sum_type: Type },
    /// Sum type variant with inner data, like `| Some a`
    SumFunction {
        tag: usize,
        sum_type: Type,
        parameter_type: Type,
    },
    /// A struct constructor, effectively just a type hint
    Structure(Type),
}

impl Visit<Type> for Constructor {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut Type),
    ) {
        match self {
            Constructor::SumConstant { sum_type, .. } => sum_type._visit(f),
            Constructor::SumFunction {
                sum_type,
                parameter_type: inner_type,
                ..
            } => {
                sum_type._visit(f);
                inner_type._visit(f);
            }
            Constructor::Structure(t) => t._visit(f),
        }
    }
}

impl Glob {
    pub fn is_exact(&self) -> bool {
        matches!(self, Glob::None)
    }

    pub fn name(&self) -> Option<&Path> {
        match self {
            Glob::Named(path) => Some(path),
            _ => None,
        }
    }
}

impl Pattern {
    pub fn introduced_names(&self) -> usize {
        let mut count = 0;
        self.clone().visit(|p: &mut Pattern| {
            if let PatternKind::Identifier(_) = *p.inner {
                count += 1
            } else if let PatternKind::Array { glob, .. } = &*p.inner {
                count += matches!(glob, Glob::Named(_)) as usize
            }
        });
        count
    }

    pub fn find_refutable_pattern(&self) -> Option<Span> {
        match &self.inner.inner {
            PatternKind::Hole | PatternKind::Identifier(_) => None,
            PatternKind::Tuple(pats) => pats.iter().find_map(Pattern::find_refutable_pattern),
            PatternKind::Struct(map) => map.values().find_map(Pattern::find_refutable_pattern),
            PatternKind::Constructor(Constructor::Structure(_), pat) => {
                pat.find_refutable_pattern()
            }
            PatternKind::Array { .. } | PatternKind::Constructor(..) => Some(self.span),
            PatternKind::Immediate(const_value) => {
                if const_value == &ConstValue::Unit {
                    None
                } else {
                    Some(self.span)
                }
            }
            PatternKind::TypeHint(pat, _) => pat.find_refutable_pattern(),
        }
    }

    pub fn is_refutable(&self) -> bool {
        match &self.inner.inner {
            PatternKind::Hole | PatternKind::Identifier(_) => false,
            PatternKind::Tuple(pats) => pats.iter().any(|p| p.is_refutable()),
            PatternKind::Struct(map) => map.values().any(|p| p.is_refutable()),
            PatternKind::Array { .. } => true,
            PatternKind::Constructor(Constructor::Structure(_), pat) => pat.is_refutable(),
            PatternKind::Constructor(..) => true,
            PatternKind::Immediate(const_value) => const_value != &ConstValue::Unit,
            PatternKind::TypeHint(pat, _) => pat.is_refutable(),
        }
    }
}

impl Visit<Pattern> for Pattern {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut Pattern),
    ) {
        match &mut *self.inner {
            PatternKind::Hole | PatternKind::Identifier(_) | PatternKind::Immediate(_) => {}
            PatternKind::Array {
                starting, ending, ..
            } => {
                starting._visit(f);
                ending._visit(f);
            }
            PatternKind::Tuple(items) => items._visit(f),
            PatternKind::Struct(map) => map._visit(f),
            PatternKind::Constructor(_, items) => items._visit(f),
            PatternKind::TypeHint(pat, _) => {
                pat._visit(f);
            }
        }
        f(self);
    }
}

impl Visit<Type> for Pattern {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut Type),
    ) {
        self.visit(|p: &mut Pattern| {
            match &mut p.inner.inner {
                PatternKind::Constructor(c, _) => c._visit(f),
                PatternKind::TypeHint(p, t) => {
                    p._visit(f);
                    t._visit(f);
                }
                _ => {}
            }
            p.type_._visit(f);
        })
    }
}

impl Visit<Path> for Pattern {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut Path),
    ) {
        self.visit(|p: &mut Pattern| {
            if let PatternKind::Identifier(id) = &mut p.inner.inner {
                f(id)
            } else if let PatternKind::Array {
                glob: Glob::Named(glob),
                ..
            } = &mut p.inner.inner
            {
                f(glob)
            }
        });
    }
}

impl Visit<(Path, Type)> for Pattern {
    fn _visit(
        &mut self,
        f: &mut impl FnMut(&mut (Path, Type)),
    ) {
        self.visit(|p: &mut Pattern| {
            match &mut p.inner.inner {
                PatternKind::Identifier(path) => {
                    let mut tup = (path.clone(), p.type_.clone());
                    f(&mut tup);
                    *path = tup.0;
                    p.type_ = tup.1;
                }
                PatternKind::Array {
                    glob: Glob::Named(glob),
                    ..
                } => {
                    let mut tup = (glob.clone(), p.type_.clone());
                    f(&mut tup);
                    *glob = tup.0;
                    p.type_ = tup.1;
                }
                _ => {}
            }
        })
    }
}

impl<'a> super::build_ir::Builder<'a> {
    pub fn pattern(
        &mut self,
        pat: PatternExpression,
        is_global: bool,
    ) -> Option<Pattern> {
        use PatternExpressionKind::*;
        let span = pat.span;
        Some(
            match pat.inner {
                Literal(literal) => PatternKind::Immediate(self.literal(literal.with_span(span))?),
                Identifier(name) if name == "_" => PatternKind::Hole,
                Identifier(name) => {
                    if let Ok(path) =
                        self.query_name(name.clone().with_span(span), NameSpace::Constructor)
                    {
                        let cons = self.symbols.get_constructor(&path).clone();
                        PatternKind::Constructor(
                            cons,
                            PatternKind::Immediate(ConstValue::Unit)
                                .with_span(span)
                                .with_type(Type::Any)
                                .into(),
                        )
                    } else {
                        PatternKind::Identifier(self.define_name(
                            name.with_span(span),
                            NameSpace::Term,
                            is_global,
                        )?)
                    }
                }
                Tuple(pats) => {
                    PatternKind::Tuple(
                        pats.into_iter()
                            .map(|p| self.pattern(p, is_global))
                            .collect::<Option<_>>()?,
                    )
                }
                Structure(map) => {
                    PatternKind::Struct(
                        map.into_iter()
                            .map(|(k, v)| self.pattern(v, is_global).map(|v| (k, v)))
                            .collect::<Option<_>>()?,
                    )
                }
                Array(pats) => {
                    let mut starting = vec![];
                    let mut glob = Glob::None;
                    let mut ending = vec![];
                    let glob_err = |this: &mut Self, span| {
                        this.logger
                            .error("Multiple glob patterns in an array are ambiguous")
                            .primary("This glob is not allowed", span)
                            .done();
                    };
                    for p in pats {
                        match p {
                            ParsedArrayPattern::Pattern(pat) => {
                                let pat = self.pattern(pat, is_global)?;
                                if glob.is_exact() {
                                    starting.push(pat);
                                } else {
                                    ending.push(pat)
                                }
                            }
                            ParsedArrayPattern::ExpansionAssign(id) => {
                                if !glob.is_exact() {
                                    glob_err(self, id.span);
                                } else {
                                    glob = Glob::Named(self.define_name(
                                        id,
                                        NameSpace::Term,
                                        is_global,
                                    )?);
                                }
                            }
                            ParsedArrayPattern::Expansion(span) => {
                                if !glob.is_exact() {
                                    glob_err(self, span);
                                } else {
                                    glob = Glob::Unnamed;
                                }
                            }
                        }
                    }
                    PatternKind::Array {
                        starting,
                        glob,
                        ending,
                    }
                }
                Constructor((a, b), pat) => {
                    let path = if let Some(b) = b {
                        let path = Path::new(a, b);
                        self.query_path(&path.clone().with_span(span), NameSpace::Constructor)
                            .done()?;
                        path
                    } else {
                        self.query_name(a.with_span(span), NameSpace::Constructor)
                            .done()?
                    };
                    let cons = self.symbols.get_constructor(&path).clone();
                    let pat = self.pattern(*pat, is_global)?;
                    PatternKind::Constructor(cons, pat.into())
                }
                ModulePath(a, b) => {
                    let path = Path::new(a, b);
                    self.query_path(&path.clone().with_span(span), NameSpace::Constructor)
                        .done()?;
                    let cons = self.symbols.get_constructor(&path).clone();
                    PatternKind::Constructor(
                        cons,
                        PatternKind::Immediate(ConstValue::Unit)
                            .with_span(span)
                            .with_type(Type::Any)
                            .into(),
                    )
                }
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
}

/// Check exhaustiveness and return a concrete refutation pattern on failure.
#[allow(clippy::result_large_err)]
pub fn are_patterns_comprehensive(
    type_: &Type,
    patterns: &[Pattern],
    symbols: &SymbolTable,
) -> std::result::Result<(), Pattern> {
    if patterns.is_empty() {
        return Err(default_refutation(type_, symbols));
    }
    let normalized = normalize_type(type_, symbols);
    if patterns
        .iter()
        .any(|pattern| pattern_covers_all(&normalized, pattern, symbols))
    {
        return Ok(());
    }
    if let Type::Array(inner) = &normalized {
        return are_array_patterns_comprehensive(inner, patterns, symbols);
    }
    let rows = patterns
        .iter()
        .map(|pattern| vec![pattern_to_slot(pattern)])
        .collect::<Vec<_>>();
    match refutation_matrix(std::slice::from_ref(&normalized), &rows, symbols) {
        Some(witness) => {
            Err(witness
                .into_iter()
                .next()
                .unwrap_or_else(|| default_refutation(&normalized, symbols)))
        }
        None => Ok(()),
    }
}

/// Matrix slot representing a wildcard or concrete pattern.
#[derive(Clone, Copy)]
enum PatternSlot<'a> {
    Wildcard,
    Pat(&'a Pattern),
}

/// Canonical constructor view for exhaustiveness checking.
#[derive(Clone)]
enum ConstructorSpec {
    Unit,
    Boolean(bool),
    Sum {
        tag: usize,
        sum_type: Type,
        payload: Type,
    },
    Tuple {
        elements: Vec<Type>,
    },
    Struct {
        name: Path,
        fields: Vec<(String, Type)>,
    },
}

/// Resolve instantiations into their concrete base types.
fn normalize_type(
    type_: &Type,
    symbols: &SymbolTable,
) -> Type {
    match type_ {
        Type::Instantiation(path, types) => {
            symbols
                .get_type(path)
                .clone()
                .instantiate(types)
                .unwrap_or_else(|_| type_.clone())
        }
        _ => type_.clone(),
    }
}

/// Remove type hints and structure constructors for matching.
fn strip_pattern(pattern: &Pattern) -> &Pattern {
    match &pattern.inner.inner {
        PatternKind::TypeHint(inner, _) => strip_pattern(inner),
        PatternKind::Constructor(Constructor::Structure(_), inner) => strip_pattern(inner),
        _ => pattern,
    }
}

/// True if the pattern matches any value of its type.
fn is_wildcard_pattern(pattern: &Pattern) -> bool {
    matches!(
        strip_pattern(pattern).inner.inner,
        PatternKind::Hole | PatternKind::Identifier(_)
    )
}

/// Convert a pattern into a matrix slot.
fn pattern_to_slot(pattern: &Pattern) -> PatternSlot<'_> {
    if is_wildcard_pattern(pattern) {
        PatternSlot::Wildcard
    } else {
        PatternSlot::Pat(pattern)
    }
}

/// Construct a pattern while explicitly setting its semantic type.
fn pattern_with_type(
    kind: PatternKind,
    type_: Type,
) -> Pattern {
    kind.with_span(Span::default()).with_type(type_)
}

/// Wildcard pattern with a concrete type for witness generation.
fn wildcard_pattern(type_: &Type) -> Pattern {
    pattern_with_type(PatternKind::Hole, type_.clone())
}

/// Constant pattern with the constant's intrinsic type.
fn immediate_pattern(value: ConstValue) -> Pattern {
    let type_ = value.type_of();
    pattern_with_type(PatternKind::Immediate(value), type_)
}

/// Tuple pattern with a concrete tuple type.
fn tuple_pattern(
    elements: &[Type],
    items: Vec<Pattern>,
) -> Pattern {
    pattern_with_type(PatternKind::Tuple(items), Type::Tuple(elements.to_vec()))
}

/// Normalize struct fields into an ordered map.
fn struct_fields_map(fields: &[(String, Type)]) -> IndexMap<String, Type> {
    fields
        .iter()
        .map(|(name, type_)| (name.clone(), type_.clone()))
        .collect()
}

/// Struct pattern with ordered fields and a concrete struct type.
fn struct_pattern(
    name: &Path,
    fields: &[(String, Type)],
    items: Vec<Pattern>,
) -> Pattern {
    let span = Span::default();
    let map = fields
        .iter()
        .zip(items)
        .map(|((name, _), pat)| (name.clone().with_span(span), pat))
        .collect::<IndexMap<_, _>>();
    pattern_with_type(
        PatternKind::Struct(map),
        Type::Struct {
            name: name.clone(),
            fields: struct_fields_map(fields),
        },
    )
}

/// Exact-length array pattern used for refutation witnesses.
fn array_pattern_exact(
    inner: &Type,
    items: Vec<Pattern>,
) -> Pattern {
    pattern_with_type(
        PatternKind::Array {
            starting: items,
            glob: Glob::None,
            ending: Vec::new(),
        },
        Type::Array(inner.clone().into()),
    )
}

/// Build a constructor pattern from its argument patterns.
fn constructor_pattern_from_args(
    constructor: &ConstructorSpec,
    args: Vec<Pattern>,
) -> Pattern {
    match constructor {
        ConstructorSpec::Unit => immediate_pattern(ConstValue::Unit),
        ConstructorSpec::Boolean(value) => immediate_pattern(ConstValue::Boolean(*value)),
        ConstructorSpec::Sum {
            tag,
            sum_type,
            payload,
        } => {
            if payload == &Type::Unit {
                let inner = immediate_pattern(ConstValue::Unit);
                let constructor = Constructor::SumConstant {
                    tag: *tag,
                    sum_type: sum_type.clone(),
                };
                pattern_with_type(
                    PatternKind::Constructor(constructor, inner.into()),
                    sum_type.clone(),
                )
            } else {
                let inner = args
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| wildcard_pattern(payload));
                let constructor = Constructor::SumFunction {
                    tag: *tag,
                    sum_type: sum_type.clone(),
                    parameter_type: payload.clone(),
                };
                pattern_with_type(
                    PatternKind::Constructor(constructor, inner.into()),
                    sum_type.clone(),
                )
            }
        }
        ConstructorSpec::Tuple { elements } => {
            let items = if args.len() == elements.len() {
                args
            } else {
                elements.iter().map(wildcard_pattern).collect()
            };
            tuple_pattern(elements, items)
        }
        ConstructorSpec::Struct { name, fields } => {
            let items = if args.len() == fields.len() {
                args
            } else {
                fields
                    .iter()
                    .map(|(_, type_)| wildcard_pattern(type_))
                    .collect()
            };
            struct_pattern(name, fields, items)
        }
    }
}

/// Constructor pattern that uses wildcards for all arguments.
fn constructor_pattern_with_wildcards(constructor: &ConstructorSpec) -> Pattern {
    let args = constructor_arg_types(constructor)
        .into_iter()
        .map(|type_| wildcard_pattern(&type_))
        .collect();
    constructor_pattern_from_args(constructor, args)
}

/// Default witness pattern for a given type.
fn default_refutation(
    type_: &Type,
    symbols: &SymbolTable,
) -> Pattern {
    let normalized = normalize_type(type_, symbols);
    match &normalized {
        Type::Unit => immediate_pattern(ConstValue::Unit),
        Type::Boolean => immediate_pattern(ConstValue::Boolean(true)),
        Type::Sum { variant_types, .. } => {
            let payload = variant_types.first().cloned().unwrap_or(Type::Unit);
            let constructor = ConstructorSpec::Sum {
                tag: 0,
                sum_type: normalized.clone(),
                payload,
            };
            constructor_pattern_with_wildcards(&constructor)
        }
        Type::Tuple(elements) => {
            let items = elements.iter().map(wildcard_pattern).collect();
            tuple_pattern(elements, items)
        }
        Type::Struct { name, fields } => {
            let field_list = fields
                .iter()
                .map(|(name, type_)| (name.clone(), type_.clone()))
                .collect::<Vec<_>>();
            let items = field_list
                .iter()
                .map(|(_, type_)| wildcard_pattern(type_))
                .collect();
            struct_pattern(name, &field_list, items)
        }
        Type::Array(inner) => array_pattern_exact(inner, Vec::new()),
        _ => wildcard_pattern(&normalized),
    }
}

/// Extract the name of a sum type after normalization.
fn sum_type_name(
    type_: &Type,
    symbols: &SymbolTable,
) -> Option<Path> {
    match normalize_type(type_, symbols) {
        Type::Sum { name, .. } => Some(name),
        _ => None,
    }
}

/// Check whether a single pattern covers all values of a type.
fn pattern_covers_all(
    type_: &Type,
    pattern: &Pattern,
    symbols: &SymbolTable,
) -> bool {
    if is_wildcard_pattern(pattern) {
        return true;
    }
    let type_ = normalize_type(type_, symbols);
    let pattern = strip_pattern(pattern);
    match &type_ {
        Type::Unit => {
            matches!(
                pattern.inner.inner,
                PatternKind::Immediate(ConstValue::Unit)
            )
        }
        Type::Boolean => false,
        Type::Sum {
            name,
            variant_types,
            ..
        } => {
            if variant_types.len() != 1 {
                return false;
            }
            let PatternKind::Constructor(constructor, inner) = &pattern.inner.inner else {
                return false;
            };
            let matches_sum = match constructor {
                Constructor::SumConstant { tag, sum_type }
                | Constructor::SumFunction { tag, sum_type, .. } => {
                    *tag == 0
                        && sum_type_name(sum_type, symbols)
                            .is_some_and(|sum_name| &sum_name == name)
                }
                Constructor::Structure(_) => false,
            };
            matches_sum && pattern_covers_all(&variant_types[0], inner, symbols)
        }
        Type::Tuple(items) => {
            match &pattern.inner.inner {
                PatternKind::Tuple(pats) => {
                    pats.len().eq(&items.len())
                        && pats
                            .iter()
                            .zip(items)
                            .all(|(pat, item_type)| pattern_covers_all(item_type, pat, symbols))
                }
                _ => false,
            }
        }
        Type::Struct { fields, .. } => {
            match &pattern.inner.inner {
                PatternKind::Struct(map) => {
                    map.keys().all(|key| fields.contains_key(&key.inner))
                        && map.iter().all(|(key, pat)| {
                            fields.get(&key.inner).is_some_and(|field_type| {
                                pattern_covers_all(field_type, pat, symbols)
                            })
                        })
                }
                _ => false,
            }
        }
        Type::Array(_) => {
            match &pattern.inner.inner {
                PatternKind::Array {
                    starting,
                    glob,
                    ending,
                } => !glob.is_exact() && starting.is_empty() && ending.is_empty(),
                _ => false,
            }
        }
        Type::Any
        | Type::Integer
        | Type::Real
        | Type::String
        | Type::Glyph
        | Type::Variable(_)
        | Type::Function(..)
        | Type::Instantiation(..) => false,
    }
}

/// Enumerate constructors for finite or single-constructor types.
fn constructors_for_type(type_: &Type) -> Option<Vec<ConstructorSpec>> {
    match type_ {
        Type::Unit => Some(vec![ConstructorSpec::Unit]),
        Type::Boolean => {
            Some(vec![
                ConstructorSpec::Boolean(true),
                ConstructorSpec::Boolean(false),
            ])
        }
        Type::Sum { variant_types, .. } => {
            let sum_type = type_.clone();
            Some(
                variant_types
                    .iter()
                    .enumerate()
                    .map(|(tag, payload)| {
                        ConstructorSpec::Sum {
                            tag,
                            sum_type: sum_type.clone(),
                            payload: payload.clone(),
                        }
                    })
                    .collect(),
            )
        }
        Type::Tuple(items) => {
            Some(vec![ConstructorSpec::Tuple {
                elements: items.clone(),
            }])
        }
        Type::Struct { name, fields } => {
            Some(vec![ConstructorSpec::Struct {
                name: name.clone(),
                fields: fields
                    .iter()
                    .map(|(name, type_)| (name.clone(), type_.clone()))
                    .collect(),
            }])
        }
        _ => None,
    }
}

/// Number of arguments required by a constructor.
fn constructor_arity(constructor: &ConstructorSpec) -> usize {
    match constructor {
        ConstructorSpec::Unit | ConstructorSpec::Boolean(_) => 0,
        // Constant constructors have an implicit unit argument
        ConstructorSpec::Sum { .. } => 1,
        ConstructorSpec::Tuple { elements } => elements.len(),
        ConstructorSpec::Struct { fields, .. } => fields.len(),
    }
}

/// Types of constructor arguments in order.
fn constructor_arg_types(constructor: &ConstructorSpec) -> Vec<Type> {
    match constructor {
        ConstructorSpec::Unit | ConstructorSpec::Boolean(_) => Vec::new(),
        ConstructorSpec::Sum { payload, .. } => vec![payload.clone()],
        ConstructorSpec::Tuple { elements } => elements.clone(),
        ConstructorSpec::Struct { fields, .. } => {
            fields
                .iter()
                .map(|(_, field_type)| field_type.clone())
                .collect()
        }
    }
}

/// Check whether a matrix slot covers all values of a type.
fn slot_covers_all(
    type_: &Type,
    slot: PatternSlot<'_>,
    symbols: &SymbolTable,
) -> bool {
    match slot {
        PatternSlot::Wildcard => true,
        PatternSlot::Pat(pattern) => pattern_covers_all(type_, pattern, symbols),
    }
}

/// Specialize a row against a constructor, expanding wildcards as needed.
fn specialize_row<'a>(
    row: &[PatternSlot<'a>],
    constructor: &ConstructorSpec,
    symbols: &SymbolTable,
) -> Option<Vec<PatternSlot<'a>>> {
    let (head, tail) = row.split_first()?;
    let rest = tail.to_vec();
    match head {
        PatternSlot::Wildcard => {
            let mut slots = vec![PatternSlot::Wildcard; constructor_arity(constructor)];
            slots.extend_from_slice(&rest);
            Some(slots)
        }
        PatternSlot::Pat(pattern) => {
            let pattern = strip_pattern(pattern);
            match constructor {
                ConstructorSpec::Unit => {
                    match &pattern.inner.inner {
                        PatternKind::Immediate(ConstValue::Unit) => Some(rest),
                        _ => None,
                    }
                }
                ConstructorSpec::Boolean(value) => {
                    match &pattern.inner.inner {
                        PatternKind::Immediate(ConstValue::Boolean(v)) if v == value => Some(rest),
                        _ => None,
                    }
                }
                ConstructorSpec::Sum { tag, sum_type, .. } => {
                    let PatternKind::Constructor(constructor, inner) = &pattern.inner.inner else {
                        return None;
                    };
                    let sum_name = sum_type_name(sum_type, symbols);
                    let matches_sum = match constructor {
                        Constructor::SumConstant {
                            tag: pat_tag,
                            sum_type,
                        }
                        | Constructor::SumFunction {
                            tag: pat_tag,
                            sum_type,
                            ..
                        } => pat_tag == tag && sum_type_name(sum_type, symbols) == sum_name,
                        Constructor::Structure(_) => false,
                    };
                    matches_sum.then(|| {
                        let mut slots = vec![PatternSlot::Pat(inner)];
                        slots.extend_from_slice(&rest);
                        slots
                    })
                }
                ConstructorSpec::Tuple { elements } => {
                    match &pattern.inner.inner {
                        PatternKind::Tuple(items) if items.len() == elements.len() => {
                            let mut slots = items.iter().map(PatternSlot::Pat).collect::<Vec<_>>();
                            slots.extend_from_slice(&rest);
                            Some(slots)
                        }
                        _ => None,
                    }
                }
                ConstructorSpec::Struct { fields, .. } => {
                    match &pattern.inner.inner {
                        PatternKind::Struct(map) => {
                            if map
                                .keys()
                                .any(|key| !fields.iter().any(|(name, _)| name == &key.inner))
                            {
                                return None;
                            }
                            let mut slots = fields
                                .iter()
                                .map(|(name, _)| {
                                    map.iter()
                                        .find(|(key, _)| name == &key.inner)
                                        .map_or(PatternSlot::Wildcard, |(_, pat)| {
                                            PatternSlot::Pat(pat)
                                        })
                                })
                                .collect::<Vec<_>>();
                            slots.extend_from_slice(&rest);
                            Some(slots)
                        }
                        _ => None,
                    }
                }
            }
        }
    }
}

/// Find a concrete refutation witness for a pattern matrix.
fn refutation_matrix<'a>(
    types: &[Type],
    rows: &[Vec<PatternSlot<'a>>],
    symbols: &SymbolTable,
) -> Option<Vec<Pattern>> {
    if types.is_empty() {
        return rows.is_empty().then_some(Vec::new());
    }
    if rows.is_empty() {
        return Some(
            types
                .iter()
                .map(|type_| default_refutation(type_, symbols))
                .collect(),
        );
    }
    let head_type = normalize_type(&types[0], symbols);
    let tail_types = &types[1..];
    if let Some(constructors) = constructors_for_type(&head_type) {
        for constructor in constructors {
            let specialized = rows
                .iter()
                .filter_map(|row| specialize_row(row, &constructor, symbols))
                .collect::<Vec<_>>();
            let arity = constructor_arity(&constructor);
            if specialized.is_empty() {
                let mut witness = Vec::with_capacity(1 + tail_types.len());
                witness.push(constructor_pattern_with_wildcards(&constructor));
                witness.extend(
                    tail_types
                        .iter()
                        .map(|type_| default_refutation(type_, symbols)),
                );
                return Some(witness);
            }
            let mut next_types = constructor_arg_types(&constructor);
            next_types.extend_from_slice(tail_types);
            if let Some(mut witness) = refutation_matrix(&next_types, &specialized, symbols) {
                if witness.len() < arity {
                    return Some(
                        types
                            .iter()
                            .map(|type_| default_refutation(type_, symbols))
                            .collect(),
                    );
                }
                let remaining = witness.split_off(arity);
                let head_pattern = constructor_pattern_from_args(&constructor, witness);
                let mut result = Vec::with_capacity(1 + remaining.len());
                result.push(head_pattern);
                result.extend(remaining);
                return Some(result);
            }
        }
        None
    } else {
        let specialized = rows
            .iter()
            .filter_map(|row| row.first().map(|slot| (*slot, row)))
            .filter(|&(slot, _)| slot_covers_all(&head_type, slot, symbols))
            .map(|(_, row)| row.iter().skip(1).copied().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        if specialized.is_empty() {
            let mut witness = Vec::with_capacity(1 + tail_types.len());
            witness.push(default_refutation(&head_type, symbols));
            witness.extend(
                tail_types
                    .iter()
                    .map(|type_| default_refutation(type_, symbols)),
            );
            return Some(witness);
        }
        refutation_matrix(tail_types, &specialized, symbols).map(|mut witness| {
            let mut result = Vec::with_capacity(1 + witness.len());
            result.push(default_refutation(&head_type, symbols));
            result.append(&mut witness);
            result
        })
    }
}

/// Exhaustiveness check for array patterns with length reasoning.
#[allow(clippy::result_large_err)]
fn are_array_patterns_comprehensive(
    inner: &Type,
    patterns: &[Pattern],
    symbols: &SymbolTable,
) -> std::result::Result<(), Pattern> {
    let array_patterns = patterns
        .iter()
        .filter(|pattern| array_pattern_parts(pattern).is_some())
        .collect::<Vec<_>>();
    if array_patterns.is_empty() {
        return Err(array_pattern_exact(inner, Vec::new()));
    }
    let min_glob_len = array_patterns
        .iter()
        .filter_map(|pattern| {
            let (starting, glob, ending) = array_pattern_parts(pattern)?;
            (!glob.is_exact()).then_some(starting.len() + ending.len())
        })
        .min();
    let Some(min_glob_len) = min_glob_len else {
        let covered_lengths = array_patterns
            .iter()
            .filter_map(|pattern| {
                let (starting, glob, ending) = array_pattern_parts(pattern)?;
                glob.is_exact().then_some(starting.len() + ending.len())
            })
            .collect::<std::collections::HashSet<_>>();
        let mut missing_len = 0;
        while covered_lengths.contains(&missing_len) {
            missing_len += 1;
        }
        return Err(array_pattern_with_wildcards(inner, missing_len));
    };

    for length in 0..min_glob_len {
        let rows = array_patterns
            .iter()
            .filter_map(|pattern| array_pattern_to_row(pattern, length))
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Err(array_pattern_with_wildcards(inner, length));
        }
        if let Some(witness) = refutation_matrix(&repeated_types(inner, length), &rows, symbols) {
            return Err(array_pattern_from_row(inner, witness));
        }
    }

    let rows = array_patterns
        .iter()
        .filter_map(|pattern| {
            let (starting, glob, ending) = array_pattern_parts(pattern)?;
            (!glob.is_exact() && starting.len() + ending.len() == min_glob_len)
                .then(|| array_pattern_to_row(pattern, min_glob_len))
                .flatten()
        })
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return Err(array_pattern_with_wildcards(inner, min_glob_len));
    }
    if let Some(witness) = refutation_matrix(&repeated_types(inner, min_glob_len), &rows, symbols) {
        return Err(array_pattern_from_row(inner, witness));
    }
    Ok(())
}

/// Extract array pattern parts while ignoring type hints.
fn array_pattern_parts(pattern: &Pattern) -> Option<(&[Pattern], &Glob, &[Pattern])> {
    match &strip_pattern(pattern).inner.inner {
        PatternKind::Array {
            starting,
            glob,
            ending,
        } => Some((starting.as_slice(), glob, ending.as_slice())),
        _ => None,
    }
}

/// Build a row for a fixed array length.
fn array_pattern_to_row<'a>(
    pattern: &'a Pattern,
    length: usize,
) -> Option<Vec<PatternSlot<'a>>> {
    let (starting, glob, ending) = array_pattern_parts(pattern)?;
    let min_len = starting.len() + ending.len();
    if glob.is_exact() {
        if min_len != length {
            return None;
        }
    } else if length < min_len {
        return None;
    }
    let mut slots = vec![PatternSlot::Wildcard; length];
    for (index, pat) in starting.iter().enumerate() {
        slots[index] = PatternSlot::Pat(pat);
    }
    for (index, pat) in ending.iter().enumerate() {
        let offset = length - ending.len();
        slots[offset + index] = PatternSlot::Pat(pat);
    }
    Some(slots)
}

/// Exact array pattern of a given length filled with wildcards.
fn array_pattern_with_wildcards(
    inner: &Type,
    length: usize,
) -> Pattern {
    let items = std::iter::repeat_with(|| wildcard_pattern(inner))
        .take(length)
        .collect();
    array_pattern_exact(inner, items)
}

/// Convert a witness row into an exact array pattern.
fn array_pattern_from_row(
    inner: &Type,
    items: Vec<Pattern>,
) -> Pattern {
    array_pattern_exact(inner, items)
}

/// Repeat a type to build a fixed-length product type vector.
fn repeated_types(
    type_: &Type,
    length: usize,
) -> Vec<Type> {
    std::iter::repeat_n(type_.clone(), length).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span() -> Span {
        Span::default()
    }

    fn pattern(kind: PatternKind) -> Pattern {
        kind.with_span(span()).with_type(Type::Any)
    }

    fn unit_pattern() -> Pattern {
        pattern(PatternKind::Immediate(ConstValue::Unit))
    }

    fn boolean_pattern(value: bool) -> Pattern {
        pattern(PatternKind::Immediate(ConstValue::Boolean(value)))
    }

    fn integer_pattern(value: i64) -> Pattern {
        pattern(PatternKind::Immediate(ConstValue::Integer(value)))
    }

    fn hole_pattern() -> Pattern {
        pattern(PatternKind::Hole)
    }

    fn constructor_pattern(
        constructor: Constructor,
        inner: Pattern,
    ) -> Pattern {
        pattern(PatternKind::Constructor(constructor, inner.into()))
    }

    fn array_pattern(
        starting: Vec<Pattern>,
        glob: Glob,
        ending: Vec<Pattern>,
    ) -> Pattern {
        pattern(PatternKind::Array {
            starting,
            glob,
            ending,
        })
    }

    #[test]
    fn unit_patterns_are_exhaustive() {
        let symbols = SymbolTable::default();
        let patterns = vec![unit_pattern()];

        assert!(are_patterns_comprehensive(&Type::Unit, &patterns, &symbols).is_ok());
    }

    #[test]
    fn boolean_patterns_need_both_constants() {
        let symbols = SymbolTable::default();
        let patterns = vec![boolean_pattern(true)];

        assert!(are_patterns_comprehensive(&Type::Boolean, &patterns, &symbols).is_err());

        let patterns = vec![boolean_pattern(true), boolean_pattern(false)];
        assert!(are_patterns_comprehensive(&Type::Boolean, &patterns, &symbols).is_ok());
    }

    #[test]
    fn integer_constants_do_not_exhaust() {
        let symbols = SymbolTable::default();
        let patterns = vec![integer_pattern(0), integer_pattern(1)];

        assert!(are_patterns_comprehensive(&Type::Integer, &patterns, &symbols).is_err());

        let patterns = vec![hole_pattern()];
        assert!(are_patterns_comprehensive(&Type::Integer, &patterns, &symbols).is_ok());
    }

    #[test]
    fn sum_patterns_cover_all_variants() {
        let symbols = SymbolTable::default();
        let sum_name = Path::new("Test", "Option");
        let sum_type = Type::Sum {
            name: sum_name.clone(),
            variant_names: vec!["None".to_string(), "Some".to_string()],
            variant_types: vec![Type::Unit, Type::Integer],
        };
        let none_constructor = Constructor::SumConstant {
            tag: 0,
            sum_type: sum_type.clone(),
        };
        let some_constructor = Constructor::SumFunction {
            tag: 1,
            sum_type: sum_type.clone(),
            parameter_type: Type::Integer,
        };
        let patterns = vec![
            constructor_pattern(none_constructor.clone(), unit_pattern()),
            constructor_pattern(some_constructor.clone(), hole_pattern()),
        ];

        assert!(are_patterns_comprehensive(&sum_type, &patterns, &symbols).is_ok());

        let patterns = vec![constructor_pattern(none_constructor, unit_pattern())];
        assert!(are_patterns_comprehensive(&sum_type, &patterns, &symbols).is_err());
    }

    #[test]
    fn array_patterns_cover_all_lengths_and_values() {
        let symbols = SymbolTable::default();
        let array_type = Type::Array(Type::Boolean.into());
        let patterns = vec![
            array_pattern(vec![], Glob::None, vec![]),
            array_pattern(vec![boolean_pattern(true)], Glob::Unnamed, vec![]),
            array_pattern(vec![boolean_pattern(false)], Glob::Unnamed, vec![]),
        ];

        assert!(are_patterns_comprehensive(&array_type, &patterns, &symbols).is_ok());

        let patterns = vec![
            array_pattern(vec![boolean_pattern(true)], Glob::Unnamed, vec![]),
            array_pattern(vec![boolean_pattern(false)], Glob::Unnamed, vec![]),
        ];
        assert!(are_patterns_comprehensive(&array_type, &patterns, &symbols).is_err());
    }
}
