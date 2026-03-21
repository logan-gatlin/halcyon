//! Post-inference exhaustiveness validation for lowered `let .. else unreachable` chains.

use std::collections::HashMap;

use indexmap::IndexMap;

use crate::ir::{
    Glob,
    ImmediateValue,
    Path,
    Pattern,
    PatternKind,
    Term,
    TermKind,
};

use super::super::infer::TypeError;
use super::super::instantiation::instantiate_forall_strict;
use super::super::{
    Type,
    TypeDefinition,
    TypeDefinitionKind,
    split_applied_type_ref,
};

const MAX_TYPE_EXPANSION_DEPTH: usize = 96;
const MAX_WITNESS_DEPTH: usize = 96;

#[derive(Clone, Debug)]
enum TestPattern {
    Wildcard,
    Immediate(ImmediateValue),
    Constructor {
        key: ConstructorKey,
        payload: Option<Box<TestPattern>>,
    },
    Tuple(Vec<TestPattern>),
    Array {
        starting: Vec<TestPattern>,
        has_glob: bool,
        ending: Vec<TestPattern>,
    },
    Struct(IndexMap<String, TestPattern>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ConstructorKey {
    Path(Path),
    Variant(String),
}

#[derive(Clone, Debug)]
enum WitnessValue {
    Opaque,
    Unit,
    Integer(i64),
    Real(f64),
    Boolean(bool),
    String(String),
    Glyph(char),
    Tuple(Vec<WitnessValue>),
    Array(Vec<WitnessValue>),
    Struct(IndexMap<String, WitnessValue>),
    Constructor {
        display: String,
        payload: Option<Box<WitnessValue>>,
    },
}

impl WitnessValue {
    fn render(&self) -> String {
        match self {
            WitnessValue::Opaque => "<value>".to_string(),
            WitnessValue::Unit => "()".to_string(),
            WitnessValue::Integer(value) => value.to_string(),
            WitnessValue::Real(value) => value.to_string(),
            WitnessValue::Boolean(value) => value.to_string(),
            WitnessValue::String(value) => format!("{value:?}"),
            WitnessValue::Glyph(value) => {
                let escaped = value.escape_default().to_string();
                format!("'{escaped}'")
            }
            WitnessValue::Tuple(items) => {
                let rendered = items
                    .iter()
                    .map(WitnessValue::render)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({rendered})")
            }
            WitnessValue::Array(items) => {
                let rendered = items
                    .iter()
                    .map(WitnessValue::render)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{rendered}]")
            }
            WitnessValue::Struct(fields) => {
                let rendered = fields
                    .iter()
                    .map(|(name, value)| format!("{name} = {}", value.render()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {rendered} }}")
            }
            WitnessValue::Constructor { display, payload } => {
                let Some(payload) = payload else {
                    return display.clone();
                };
                format!("{display} {}", payload.render_atom())
            }
        }
    }

    fn render_atom(&self) -> String {
        match self {
            WitnessValue::Constructor { .. } => format!("({})", self.render()),
            _ => self.render(),
        }
    }
}

#[derive(Clone, Debug)]
struct SubjectConstraints {
    type_: Type,
    positives: Vec<TestPattern>,
    negatives: Vec<TestPattern>,
}

#[derive(Clone, Debug, Default)]
struct ConstraintContext {
    subjects: HashMap<Path, SubjectConstraints>,
    equivalents: HashMap<Path, Vec<Path>>,
}

impl ConstraintContext {
    fn related_paths(
        &self,
        path: &Path,
    ) -> Vec<Path> {
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![path.clone()];
        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            if let Some(neighbors) = self.equivalents.get(&current) {
                stack.extend(neighbors.iter().cloned());
            }
        }
        if visited.is_empty() {
            vec![path.clone()]
        } else {
            visited.into_iter().collect()
        }
    }

    fn add_equivalence(
        &mut self,
        left: &Path,
        right: &Path,
    ) {
        add_unique_edge(&mut self.equivalents, left, right);
        add_unique_edge(&mut self.equivalents, right, left);
    }
}

#[derive(Clone, Debug)]
struct ConstructorCase {
    key: ConstructorKey,
    display: String,
    payload: Option<Type>,
}

#[derive(Clone, Debug)]
enum MatchType {
    Opaque,
    Unit,
    Integer,
    Real,
    Boolean,
    String,
    Glyph,
    Tuple(Vec<Type>),
    Struct(IndexMap<String, Type>),
    Array(Box<Type>),
    Sum(Vec<ConstructorCase>),
}

pub(super) fn check_term_exhaustiveness(
    term: &Term<Type>,
    type_definitions: &IndexMap<Path, TypeDefinition>,
    constructor_aliases: &IndexMap<Path, Path>,
) -> Result<(), TypeError> {
    ExhaustivenessChecker {
        type_definitions,
        constructor_aliases,
    }
    .check_term(term)
}

struct ExhaustivenessChecker<'a> {
    type_definitions: &'a IndexMap<Path, TypeDefinition>,
    constructor_aliases: &'a IndexMap<Path, Path>,
}

impl ExhaustivenessChecker<'_> {
    fn check_term(
        &self,
        term: &Term<Type>,
    ) -> Result<(), TypeError> {
        self.check_term_with_context(term, &ConstraintContext::default())
    }

    fn check_term_with_context(
        &self,
        term: &Term<Type>,
        context: &ConstraintContext,
    ) -> Result<(), TypeError> {
        match &term.kind {
            TermKind::Let {
                assignee,
                value,
                then,
                else_,
                ..
            } => {
                self.check_term_with_context(value, context)?;
                if let TermKind::Identifier(path) = &value.kind {
                    if let Some((then_context, _)) =
                        self.context_with_constraint(context, path, &value.type_, assignee, true)
                    {
                        let then_context =
                            if let Some(binding_path) = identifier_binding_path(assignee) {
                                self.context_with_equivalence(
                                    &then_context,
                                    binding_path,
                                    path,
                                    &value.type_,
                                )
                            } else {
                                Some(then_context)
                            };
                        if let Some(then_context) = then_context {
                            self.check_term_with_context(then, &then_context)?;
                        }
                    }
                    if let Some((else_context, witness)) =
                        self.context_with_constraint(context, path, &value.type_, assignee, false)
                    {
                        if matches!(else_.kind, TermKind::Unreachable) {
                            return Err(TypeError::NonExhaustivePatterns {
                                span: assignee.span,
                                counterexample: witness.render(),
                            });
                        }
                        self.check_term_with_context(else_, &else_context)?;
                    }
                    return Ok(());
                }

                self.check_term_with_context(then, context)?;
                if matches!(else_.kind, TermKind::Unreachable) {
                    if let Some(witness) =
                        self.find_witness(&value.type_, &[], &[self.lower_pattern(assignee)], 0)
                    {
                        return Err(TypeError::NonExhaustivePatterns {
                            span: assignee.span,
                            counterexample: witness.render(),
                        });
                    }
                    return Ok(());
                }

                self.check_term_with_context(else_, context)
            }
            TermKind::Tuple(items) => {
                for item in items {
                    self.check_term_with_context(item, context)?;
                }
                Ok(())
            }
            TermKind::Struct(fields) => {
                for item in fields.values() {
                    self.check_term_with_context(item, context)?;
                }
                Ok(())
            }
            TermKind::Field { of, .. } => self.check_term_with_context(of, context),
            TermKind::Function { body, .. } => {
                self.check_term_with_context(body, &ConstraintContext::default())
            }
            TermKind::Call { callee, argument } => {
                self.check_term_with_context(callee, context)?;
                self.check_term_with_context(argument, context)
            }
            TermKind::Semicolon(left, right) => {
                self.check_term_with_context(left, context)?;
                self.check_term_with_context(right, context)
            }
            TermKind::Immediate(_)
            | TermKind::Identifier(_)
            | TermKind::InlineWasm { .. }
            | TermKind::Unreachable => Ok(()),
        }
    }

    fn context_with_constraint(
        &self,
        context: &ConstraintContext,
        path: &Path,
        type_: &Type,
        pattern: &Pattern<Type>,
        positive: bool,
    ) -> Option<(ConstraintContext, WitnessValue)> {
        let mut next = context.clone();
        let lowered = self.lower_pattern(pattern);
        let related_paths = next.related_paths(path);
        for related_path in related_paths.iter() {
            let entry = next
                .subjects
                .entry(related_path.clone())
                .or_insert_with(|| {
                    SubjectConstraints {
                        type_: type_.clone(),
                        positives: Vec::new(),
                        negatives: Vec::new(),
                    }
                });
            if positive {
                entry.positives.push(lowered.clone());
            } else {
                entry.negatives.push(lowered.clone());
            }
        }

        let mut representative_witness = None;
        for related_path in related_paths.iter() {
            let Some(entry) = next.subjects.get(related_path) else {
                continue;
            };
            let Some(witness) =
                self.find_witness(&entry.type_, &entry.positives, &entry.negatives, 0)
            else {
                return None;
            };
            representative_witness = Some(witness);
        }

        representative_witness.map(|witness| (next, witness))
    }

    fn context_with_equivalence(
        &self,
        context: &ConstraintContext,
        left: &Path,
        right: &Path,
        default_type: &Type,
    ) -> Option<ConstraintContext> {
        let mut next = context.clone();
        next.add_equivalence(left, right);

        let related_paths = next.related_paths(left);
        let mut merged = SubjectConstraints {
            type_: default_type.clone(),
            positives: Vec::new(),
            negatives: Vec::new(),
        };

        for related_path in related_paths.iter() {
            if let Some(existing) = next.subjects.get(related_path) {
                merged.type_ = existing.type_.clone();
                merged.positives.extend(existing.positives.clone());
                merged.negatives.extend(existing.negatives.clone());
            }
        }

        let _ = self.find_witness(&merged.type_, &merged.positives, &merged.negatives, 0)?;
        for related_path in related_paths {
            next.subjects.insert(related_path, merged.clone());
        }
        Some(next)
    }

    fn lower_pattern(
        &self,
        pattern: &Pattern<Type>,
    ) -> TestPattern {
        match &pattern.kind {
            PatternKind::Hole | PatternKind::Identifier(_) => TestPattern::Wildcard,
            PatternKind::Immediate(value) => TestPattern::Immediate(value.clone()),
            PatternKind::ConstConstructor(path) => {
                TestPattern::Constructor {
                    key: ConstructorKey::Path(self.canonical_constructor(path)),
                    payload: None,
                }
            }
            PatternKind::Constructor(path, payload) => {
                TestPattern::Constructor {
                    key: ConstructorKey::Path(self.canonical_constructor(path)),
                    payload: Some(Box::new(self.lower_pattern(payload))),
                }
            }
            PatternKind::Tuple(items) => {
                TestPattern::Tuple(items.iter().map(|item| self.lower_pattern(item)).collect())
            }
            PatternKind::Array {
                starting,
                glob,
                ending,
            } => {
                TestPattern::Array {
                    starting: starting
                        .iter()
                        .map(|item| self.lower_pattern(item))
                        .collect(),
                    has_glob: !matches!(glob, Glob::None),
                    ending: ending.iter().map(|item| self.lower_pattern(item)).collect(),
                }
            }
            PatternKind::Struct(fields) => {
                TestPattern::Struct(
                    fields
                        .iter()
                        .map(|(name, value)| (name.inner.clone(), self.lower_pattern(value)))
                        .collect(),
                )
            }
            PatternKind::TypeHint(inner, _) => self.lower_pattern(inner),
        }
    }

    fn canonical_constructor(
        &self,
        path: &Path,
    ) -> Path {
        let mut current = path.clone();
        let mut seen = std::collections::HashSet::new();
        while let Some(next) = self.constructor_aliases.get(&current) {
            if !seen.insert(current.clone()) {
                break;
            }
            current = next.clone();
        }
        current
    }

    fn find_witness(
        &self,
        type_: &Type,
        positives: &[TestPattern],
        negatives: &[TestPattern],
        depth: usize,
    ) -> Option<WitnessValue> {
        if depth > MAX_WITNESS_DEPTH {
            return Some(WitnessValue::Opaque);
        }
        if negatives.iter().any(is_wildcard_pattern) {
            return None;
        }

        match self.match_type(type_, depth) {
            MatchType::Opaque => {
                if positives
                    .iter()
                    .any(|pattern| !is_wildcard_pattern(pattern))
                {
                    return None;
                }
                Some(WitnessValue::Opaque)
            }
            MatchType::Unit => {
                if positives.iter().any(|pattern| {
                    !matches!(
                        pattern,
                        TestPattern::Wildcard | TestPattern::Immediate(ImmediateValue::Unit)
                    )
                }) {
                    return None;
                }
                if negatives
                    .iter()
                    .any(|pattern| matches!(pattern, TestPattern::Immediate(ImmediateValue::Unit)))
                {
                    return None;
                }
                Some(WitnessValue::Unit)
            }
            MatchType::Boolean => {
                for candidate in [false, true] {
                    if positives
                        .iter()
                        .all(|pattern| pattern_matches_boolean(pattern, candidate))
                        && negatives
                            .iter()
                            .all(|pattern| !pattern_matches_boolean(pattern, candidate))
                    {
                        return Some(WitnessValue::Boolean(candidate));
                    }
                }
                None
            }
            MatchType::Integer => {
                let mut required = None;
                for pattern in positives {
                    match pattern {
                        TestPattern::Wildcard => {}
                        TestPattern::Immediate(ImmediateValue::Integer(value)) => {
                            if let Some(existing) = required
                                && existing != *value
                            {
                                return None;
                            }
                            required = Some(*value);
                        }
                        _ => return None,
                    }
                }

                let forbidden = negatives
                    .iter()
                    .filter_map(|pattern| {
                        match pattern {
                            TestPattern::Immediate(ImmediateValue::Integer(value)) => Some(*value),
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>();

                if let Some(value) = required {
                    if forbidden.contains(&value) {
                        return None;
                    }
                    return Some(WitnessValue::Integer(value));
                }

                let mut candidate = 0_i64;
                while forbidden.contains(&candidate) {
                    candidate += 1;
                }
                Some(WitnessValue::Integer(candidate))
            }
            MatchType::Real => {
                let mut required = None;
                for pattern in positives {
                    match pattern {
                        TestPattern::Wildcard => {}
                        TestPattern::Immediate(ImmediateValue::Real(value)) => {
                            if let Some(existing) = required
                                && existing != *value
                            {
                                return None;
                            }
                            required = Some(*value);
                        }
                        _ => return None,
                    }
                }

                let forbidden = negatives
                    .iter()
                    .filter_map(|pattern| {
                        match pattern {
                            TestPattern::Immediate(ImmediateValue::Real(value)) => Some(*value),
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>();

                if let Some(value) = required {
                    if forbidden
                        .iter()
                        .any(|forbidden_value| *forbidden_value == value)
                    {
                        return None;
                    }
                    return Some(WitnessValue::Real(value));
                }

                let mut candidate = 0.0_f64;
                while forbidden
                    .iter()
                    .any(|forbidden_value| *forbidden_value == candidate)
                {
                    candidate += 1.0;
                }
                Some(WitnessValue::Real(candidate))
            }
            MatchType::String => {
                let mut required = None::<String>;
                for pattern in positives {
                    match pattern {
                        TestPattern::Wildcard => {}
                        TestPattern::Immediate(ImmediateValue::String(value)) => {
                            if let Some(existing) = &required
                                && existing != value
                            {
                                return None;
                            }
                            required = Some(value.clone());
                        }
                        _ => return None,
                    }
                }

                let forbidden = negatives
                    .iter()
                    .filter_map(|pattern| {
                        match pattern {
                            TestPattern::Immediate(ImmediateValue::String(value)) => {
                                Some(value.clone())
                            }
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>();

                if let Some(value) = required {
                    if forbidden.contains(&value) {
                        return None;
                    }
                    return Some(WitnessValue::String(value));
                }

                let mut candidate = String::new();
                let mut index = 0_u64;
                while forbidden.contains(&candidate) {
                    index += 1;
                    candidate = format!("s{index}");
                }
                Some(WitnessValue::String(candidate))
            }
            MatchType::Glyph => {
                let mut required = None::<char>;
                for pattern in positives {
                    match pattern {
                        TestPattern::Wildcard => {}
                        TestPattern::Immediate(ImmediateValue::Glyph(value)) => {
                            if let Some(existing) = required
                                && existing != *value
                            {
                                return None;
                            }
                            required = Some(*value);
                        }
                        _ => return None,
                    }
                }

                let forbidden = negatives
                    .iter()
                    .filter_map(|pattern| {
                        match pattern {
                            TestPattern::Immediate(ImmediateValue::Glyph(value)) => Some(*value),
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>();

                if let Some(value) = required {
                    if forbidden.contains(&value) {
                        return None;
                    }
                    return Some(WitnessValue::Glyph(value));
                }

                let mut codepoint = b'a' as u32;
                while let Some(candidate) = char::from_u32(codepoint) {
                    if !forbidden.contains(&candidate) {
                        return Some(WitnessValue::Glyph(candidate));
                    }
                    codepoint += 1;
                }
                None
            }
            MatchType::Tuple(item_types) => {
                let mut positive_rows = Vec::new();
                let mut negative_rows = Vec::new();

                for pattern in positives {
                    match pattern {
                        TestPattern::Wildcard => {}
                        TestPattern::Tuple(items) if items.len() == item_types.len() => {
                            positive_rows.push(items.clone());
                        }
                        _ => return None,
                    }
                }

                for pattern in negatives {
                    match pattern {
                        TestPattern::Tuple(items) if items.len() == item_types.len() => {
                            negative_rows.push(items.clone());
                        }
                        TestPattern::Wildcard => return None,
                        _ => {}
                    }
                }

                self.solve_product(&item_types, &positive_rows, &negative_rows, depth + 1)
                    .map(WitnessValue::Tuple)
            }
            MatchType::Struct(fields) => {
                let field_names = fields.keys().cloned().collect::<Vec<_>>();
                let field_types = fields.values().cloned().collect::<Vec<_>>();
                let field_count = field_names.len();
                let mut positive_rows = Vec::new();
                let mut negative_rows = Vec::new();

                for pattern in positives {
                    match pattern {
                        TestPattern::Wildcard => {}
                        TestPattern::Struct(pattern_fields)
                            if pattern_fields.len() == field_count
                                && field_names
                                    .iter()
                                    .all(|name| pattern_fields.contains_key(name)) =>
                        {
                            positive_rows.push(
                                field_names
                                    .iter()
                                    .map(|name| {
                                        pattern_fields
                                            .get(name)
                                            .cloned()
                                            .unwrap_or(TestPattern::Wildcard)
                                    })
                                    .collect(),
                            );
                        }
                        _ => return None,
                    }
                }

                for pattern in negatives {
                    match pattern {
                        TestPattern::Struct(pattern_fields)
                            if pattern_fields.len() == field_count
                                && field_names
                                    .iter()
                                    .all(|name| pattern_fields.contains_key(name)) =>
                        {
                            negative_rows.push(
                                field_names
                                    .iter()
                                    .map(|name| {
                                        pattern_fields
                                            .get(name)
                                            .cloned()
                                            .unwrap_or(TestPattern::Wildcard)
                                    })
                                    .collect(),
                            );
                        }
                        TestPattern::Wildcard => return None,
                        _ => {}
                    }
                }

                self.solve_product(&field_types, &positive_rows, &negative_rows, depth + 1)
                    .map(|values| {
                        WitnessValue::Struct(field_names.into_iter().zip(values).collect())
                    })
            }
            MatchType::Array(item_type) => {
                let mut array_patterns = Vec::<&TestPattern>::new();
                array_patterns.extend(positives.iter());
                array_patterns.extend(negatives.iter());
                let max_bound = array_patterns
                    .iter()
                    .filter_map(|pattern| {
                        match pattern {
                            TestPattern::Array {
                                starting, ending, ..
                            } => Some(starting.len() + ending.len()),
                            _ => None,
                        }
                    })
                    .max()
                    .unwrap_or(0);

                for length in 0..=max_bound + 1 {
                    let mut positive_rows = Vec::new();
                    let mut negative_rows = Vec::new();
                    let mut length_valid = true;

                    for pattern in positives {
                        match pattern {
                            TestPattern::Wildcard => {}
                            TestPattern::Array { .. } => {
                                if let Some(row) = array_pattern_for_length(pattern, length) {
                                    positive_rows.push(row);
                                } else {
                                    length_valid = false;
                                    break;
                                }
                            }
                            _ => {
                                length_valid = false;
                                break;
                            }
                        }
                    }
                    if !length_valid {
                        continue;
                    }

                    for pattern in negatives {
                        match pattern {
                            TestPattern::Wildcard => {
                                length_valid = false;
                                break;
                            }
                            TestPattern::Array { .. } => {
                                if let Some(row) = array_pattern_for_length(pattern, length) {
                                    negative_rows.push(row);
                                }
                            }
                            _ => {}
                        }
                    }
                    if !length_valid {
                        continue;
                    }

                    let field_types =
                        std::iter::repeat_n((*item_type).clone(), length).collect::<Vec<_>>();
                    if let Some(values) =
                        self.solve_product(&field_types, &positive_rows, &negative_rows, depth + 1)
                    {
                        return Some(WitnessValue::Array(values));
                    }
                }

                None
            }
            MatchType::Sum(cases) => {
                for case in cases {
                    let mut payload_positives = Vec::new();
                    let mut payload_negatives = Vec::new();
                    let mut valid = true;

                    for pattern in positives {
                        match pattern {
                            TestPattern::Wildcard => {}
                            TestPattern::Constructor { key, payload } if *key == case.key => {
                                if let Some(payload_pattern) = payload {
                                    if case.payload.is_none() {
                                        valid = false;
                                        break;
                                    }
                                    payload_positives.push((**payload_pattern).clone());
                                }
                            }
                            TestPattern::Constructor { .. } => {
                                valid = false;
                                break;
                            }
                            _ => {
                                valid = false;
                                break;
                            }
                        }
                    }
                    if !valid {
                        continue;
                    }

                    for pattern in negatives {
                        match pattern {
                            TestPattern::Constructor { key, payload } if *key == case.key => {
                                match payload {
                                    None => {
                                        valid = false;
                                        break;
                                    }
                                    Some(payload_pattern) => {
                                        if case.payload.is_some() {
                                            payload_negatives.push((**payload_pattern).clone());
                                        }
                                    }
                                }
                            }
                            TestPattern::Wildcard => {
                                valid = false;
                                break;
                            }
                            _ => {}
                        }
                    }
                    if !valid {
                        continue;
                    }

                    if let Some(payload_type) = case.payload {
                        let Some(payload_witness) = self.find_witness(
                            &payload_type,
                            &payload_positives,
                            &payload_negatives,
                            depth + 1,
                        ) else {
                            continue;
                        };
                        return Some(WitnessValue::Constructor {
                            display: case.display,
                            payload: Some(Box::new(payload_witness)),
                        });
                    }

                    return Some(WitnessValue::Constructor {
                        display: case.display,
                        payload: None,
                    });
                }

                None
            }
        }
    }

    fn solve_product(
        &self,
        field_types: &[Type],
        positive_rows: &[Vec<TestPattern>],
        negative_rows: &[Vec<TestPattern>],
        depth: usize,
    ) -> Option<Vec<WitnessValue>> {
        if field_types.is_empty() {
            return if negative_rows.is_empty() {
                Some(Vec::new())
            } else {
                None
            };
        }

        if negative_rows.len() > 120 {
            return self.solve_product_fallback(field_types, positive_rows, depth);
        }

        let initial_mask = if negative_rows.is_empty() {
            0_u128
        } else {
            (1_u128 << negative_rows.len()) - 1
        };
        let mut memo = HashMap::<(usize, u128), Option<Vec<WitnessValue>>>::new();

        self.solve_product_state(
            field_types,
            positive_rows,
            negative_rows,
            depth,
            0,
            initial_mask,
            &mut memo,
        )
    }

    fn solve_product_fallback(
        &self,
        field_types: &[Type],
        positive_rows: &[Vec<TestPattern>],
        depth: usize,
    ) -> Option<Vec<WitnessValue>> {
        field_types
            .iter()
            .enumerate()
            .map(|(index, field_type)| {
                let positives = positive_rows
                    .iter()
                    .map(|row| row[index].clone())
                    .collect::<Vec<_>>();
                self.find_witness(field_type, &positives, &[], depth + 1)
            })
            .collect()
    }

    fn solve_product_state(
        &self,
        field_types: &[Type],
        positive_rows: &[Vec<TestPattern>],
        negative_rows: &[Vec<TestPattern>],
        depth: usize,
        field_index: usize,
        unbroken_mask: u128,
        memo: &mut HashMap<(usize, u128), Option<Vec<WitnessValue>>>,
    ) -> Option<Vec<WitnessValue>> {
        if let Some(cached) = memo.get(&(field_index, unbroken_mask)) {
            return cached.clone();
        }

        if field_index == field_types.len() {
            let result = (unbroken_mask == 0).then_some(Vec::new());
            memo.insert((field_index, unbroken_mask), result.clone());
            return result;
        }

        if unbroken_mask == 0 {
            let result = field_types
                .iter()
                .enumerate()
                .skip(field_index)
                .map(|(index, field_type)| {
                    let positives = positive_rows
                        .iter()
                        .map(|row| row[index].clone())
                        .collect::<Vec<_>>();
                    self.find_witness(field_type, &positives, &[], depth + 1)
                })
                .collect::<Option<Vec<_>>>();
            memo.insert((field_index, unbroken_mask), result.clone());
            return result;
        }

        let remaining_fields = field_types.len() - field_index;
        let mut must_stay_mask = 0_u128;
        let mut optional_mask = 0_u128;

        let mut bits = unbroken_mask;
        while bits != 0 {
            let bit = bits.trailing_zeros() as usize;
            bits &= bits - 1;

            let pattern = &negative_rows[bit][field_index];
            let can_break_later = (field_index + 1..field_types.len())
                .any(|index| !is_wildcard_pattern(&negative_rows[bit][index]));

            if is_wildcard_pattern(pattern) {
                if !can_break_later {
                    memo.insert((field_index, unbroken_mask), None);
                    return None;
                }
                must_stay_mask |= 1_u128 << bit;
            } else if can_break_later {
                optional_mask |= 1_u128 << bit;
            }
        }

        if remaining_fields == 1 {
            if must_stay_mask != 0 {
                memo.insert((field_index, unbroken_mask), None);
                return None;
            }
            optional_mask = 0;
        }

        let mut candidate_masks = Vec::new();
        candidate_masks.push(must_stay_mask);

        let mut submask = optional_mask;
        while submask != 0 {
            candidate_masks.push(must_stay_mask | submask);
            submask = (submask - 1) & optional_mask;
        }

        candidate_masks.sort_by_key(|mask| mask.count_ones());
        candidate_masks.dedup();

        for next_unbroken in candidate_masks {
            if remaining_fields == 1 && next_unbroken != 0 {
                continue;
            }

            let mut field_positives = positive_rows
                .iter()
                .map(|row| row[field_index].clone())
                .collect::<Vec<_>>();
            let mut field_negatives = Vec::new();

            let mut local_bits = unbroken_mask;
            while local_bits != 0 {
                let bit = local_bits.trailing_zeros() as usize;
                local_bits &= local_bits - 1;
                let bit_mask = 1_u128 << bit;
                if next_unbroken & bit_mask != 0 {
                    field_positives.push(negative_rows[bit][field_index].clone());
                } else {
                    field_negatives.push(negative_rows[bit][field_index].clone());
                }
            }

            let Some(field_witness) = self.find_witness(
                &field_types[field_index],
                &field_positives,
                &field_negatives,
                depth + 1,
            ) else {
                continue;
            };

            let Some(mut tail) = self.solve_product_state(
                field_types,
                positive_rows,
                negative_rows,
                depth,
                field_index + 1,
                next_unbroken,
                memo,
            ) else {
                continue;
            };

            let mut result = Vec::with_capacity(tail.len() + 1);
            result.push(field_witness);
            result.append(&mut tail);
            memo.insert((field_index, unbroken_mask), Some(result.clone()));
            return Some(result);
        }

        memo.insert((field_index, unbroken_mask), None);
        None
    }

    fn match_type(
        &self,
        type_: &Type,
        depth: usize,
    ) -> MatchType {
        if depth > MAX_TYPE_EXPANSION_DEPTH {
            return MatchType::Opaque;
        }

        if let Some((name, definition_kind, instantiated_body)) =
            self.resolve_named_application(type_, depth)
        {
            if definition_kind == TypeDefinitionKind::Alias {
                return self.match_type(&instantiated_body, depth + 1);
            }

            if let Type::Sum { variants } = instantiated_body.clone() {
                let cases = variants
                    .into_iter()
                    .map(|(variant_name, payload_type)| {
                        let constructor_path = name.sibling(&variant_name);
                        let canonical_path = self.canonical_constructor(&constructor_path);
                        let payload = if self.is_plain_unit(&payload_type, depth + 1) {
                            None
                        } else {
                            Some(payload_type)
                        };
                        ConstructorCase {
                            key: ConstructorKey::Path(canonical_path),
                            display: constructor_path.to_string(),
                            payload,
                        }
                    })
                    .collect();
                return MatchType::Sum(cases);
            }

            let payload = if self.is_plain_unit(&instantiated_body, depth + 1) {
                None
            } else {
                Some(instantiated_body)
            };
            let constructor_key = ConstructorKey::Path(self.canonical_constructor(&name));
            return MatchType::Sum(vec![ConstructorCase {
                key: constructor_key,
                display: name.to_string(),
                payload,
            }]);
        }

        match type_ {
            Type::Unit => MatchType::Unit,
            Type::Integer => MatchType::Integer,
            Type::Real => MatchType::Real,
            Type::Boolean => MatchType::Boolean,
            Type::String => MatchType::String,
            Type::Glyph => MatchType::Glyph,
            Type::Tuple(items) => MatchType::Tuple(items.clone()),
            Type::Struct { fields } | Type::StructConstraint { fields, .. } => {
                MatchType::Struct(fields.clone())
            }
            Type::Array(item_type) => MatchType::Array(item_type.clone()),
            Type::Sum { variants } => {
                let cases = variants
                    .iter()
                    .map(|(name, payload_type)| {
                        ConstructorCase {
                            key: ConstructorKey::Variant(name.clone()),
                            display: name.clone(),
                            payload: if self.is_plain_unit(payload_type, depth + 1) {
                                None
                            } else {
                                Some(payload_type.clone())
                            },
                        }
                    })
                    .collect();
                MatchType::Sum(cases)
            }
            Type::Apply {
                constructor,
                arguments,
            } => {
                let (base, mut flattened_arguments) = split_applied_type_ref(constructor);
                flattened_arguments.extend(arguments.iter().cloned());
                if let Type::Named { name, .. } = base
                    && let Some(builtin) = self.match_core_constructor(&name, &flattened_arguments)
                {
                    return builtin;
                }
                MatchType::Opaque
            }
            Type::Named { name, .. } => {
                if let Some(builtin) = self.match_core_constructor(name, &[]) {
                    builtin
                } else {
                    MatchType::Opaque
                }
            }
            Type::TypeVar(_) | Type::MetaVar(_) | Type::ForAll { .. } | Type::Function(..) => {
                MatchType::Opaque
            }
        }
    }

    fn match_core_constructor(
        &self,
        name: &Path,
        arguments: &[Type],
    ) -> Option<MatchType> {
        if name.major != crate::CORE_BUNDLE_NAME {
            return None;
        }
        Some(match (name.minor.as_str(), arguments) {
            ("Unit", []) => MatchType::Unit,
            ("Integer", []) => MatchType::Integer,
            ("Real", []) => MatchType::Real,
            ("Boolean", []) => MatchType::Boolean,
            ("String", []) => MatchType::String,
            ("Glyph", []) => MatchType::Glyph,
            ("Array", [item]) => MatchType::Array(Box::new(item.clone())),
            _ => return None,
        })
    }

    fn resolve_named_application(
        &self,
        type_: &Type,
        depth: usize,
    ) -> Option<(Path, TypeDefinitionKind, Type)> {
        if depth > MAX_TYPE_EXPANSION_DEPTH {
            return None;
        }

        let (base, arguments) = split_applied_type_ref(type_);
        let Type::Named { name, body } = base else {
            return None;
        };

        if let Some(definition) = self.type_definitions.get(&name) {
            let instantiated = instantiate_forall_strict(&definition.body, &arguments)?;
            return Some((name, definition.kind, instantiated));
        }

        if arguments.is_empty() {
            if matches!(body.as_ref(), Type::Unit) {
                return None;
            }
            return Some((name, TypeDefinitionKind::Named, *body));
        }

        instantiate_forall_strict(&body, &arguments)
            .map(|instantiated| (name, TypeDefinitionKind::Named, instantiated))
    }

    fn is_plain_unit(
        &self,
        type_: &Type,
        depth: usize,
    ) -> bool {
        if depth > MAX_TYPE_EXPANSION_DEPTH {
            return false;
        }
        if let Some((_, definition_kind, instantiated)) =
            self.resolve_named_application(type_, depth)
            && definition_kind == TypeDefinitionKind::Alias
        {
            return self.is_plain_unit(&instantiated, depth + 1);
        }
        matches!(type_, Type::Unit)
    }
}

fn is_wildcard_pattern(pattern: &TestPattern) -> bool {
    matches!(pattern, TestPattern::Wildcard)
}

fn identifier_binding_path(pattern: &Pattern<Type>) -> Option<&Path> {
    match &pattern.kind {
        PatternKind::Identifier(path) => Some(path),
        PatternKind::TypeHint(inner, _) => identifier_binding_path(inner),
        _ => None,
    }
}

fn add_unique_edge(
    edges: &mut HashMap<Path, Vec<Path>>,
    from: &Path,
    to: &Path,
) {
    let entry = edges.entry(from.clone()).or_default();
    if !entry.contains(to) {
        entry.push(to.clone());
    }
}

fn pattern_matches_boolean(
    pattern: &TestPattern,
    candidate: bool,
) -> bool {
    match pattern {
        TestPattern::Wildcard => true,
        TestPattern::Immediate(ImmediateValue::Boolean(value)) => *value == candidate,
        _ => false,
    }
}

fn array_pattern_for_length(
    pattern: &TestPattern,
    length: usize,
) -> Option<Vec<TestPattern>> {
    let TestPattern::Array {
        starting,
        has_glob,
        ending,
    } = pattern
    else {
        return None;
    };

    let minimum_length = starting.len() + ending.len();
    let matches_length = if *has_glob {
        length >= minimum_length
    } else {
        length == minimum_length
    };
    if !matches_length {
        return None;
    }

    let mut row = vec![TestPattern::Wildcard; length];
    for (index, item) in starting.iter().enumerate() {
        row[index] = item.clone();
    }
    let ending_start = length.saturating_sub(ending.len());
    for (offset, item) in ending.iter().enumerate() {
        row[ending_start + offset] = item.clone();
    }
    Some(row)
}
