use super::*;

#[derive(Debug, Clone)]
pub enum EnvironmentSymbol {
    Let(TypeScheme),
    Free(TypeRef),
}

impl Unify for EnvironmentSymbol {
    fn unify(&mut self, tv: TypeVariable, type_: &Type) {
        if let Self::Free(t) = self {
            t.unify(tv, type_);
        }
    }
}

#[derive(Debug, Default)]
pub struct Environment {
    pub constructors: HashMap<Path, Constructor>,
    symbols: HashMap<Path, EnvironmentSymbol>,
    constraints: Vec<Vec<Constraint>>,
    type_var_no: usize,
}

impl Unify for Environment {
    fn unify(&mut self, tv: TypeVariable, type_: &Type) {
        self.symbols.unify(tv, type_);
    }
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define(&mut self, ident: Path, type_: TypeRef) {
        self.symbols.insert(ident, EnvironmentSymbol::Free(type_));
    }

    pub fn make_let_bound(&mut self, ident: &Path) {
        let t = TypeScheme::new(self.get_symbol(ident));
        self.symbols
            .insert(ident.clone(), EnvironmentSymbol::Let(t));
    }

    pub fn get_symbol(&mut self, ident: &Path) -> TypeRef {
        match self.symbols.get(ident).unwrap().clone() {
            EnvironmentSymbol::Let(type_scheme) => {
                type_scheme.instantiate(|| self.fresh_type_variable())
            }
            EnvironmentSymbol::Free(type_) => type_,
        }
    }

    pub fn begin_let(&mut self) {
        self.constraints.push(vec![]);
    }

    pub fn end_let(&mut self) -> Result<Vec<Substitution>> {
        let cons = self.constraints.pop().unwrap();
        let solution = solve_constraints(&cons)?;
        self.unify_all(&solution);
        if self.constraints.is_empty() {
            self.type_var_no = 0;
        }
        Ok(solution)
    }

    pub fn type_constraint(&mut self, a: TypeRef, b: TypeRef, span: Span) {
        self.constraints.last_mut().unwrap().push(Constraint {
            kind: ConstraintKind::Type(a, b),
            span,
        });
    }

    pub fn struct_constraint(&mut self, of: Type, name: String, span: Span) {
        self.constraints.last_mut().unwrap().push(Constraint {
            kind: ConstraintKind::StructField { of, name },
            span,
        })
    }

    pub fn print_constraints(&self) {
        for c in self.constraints.last().unwrap() {
            println!("{c}");
        }
    }

    pub fn fresh_type_variable(&mut self) -> TypeVariable {
        let tv = self.type_var_no;
        self.type_var_no += 1;
        tv
    }
}
