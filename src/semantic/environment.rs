use super::*;

#[derive(Debug, Clone)]
enum ConstraintOrGuard {
    Cons(Constraint),
    LetGuard,
}

impl Unify for ConstraintOrGuard {
    fn unify(&mut self, tv: TypeVariable, type_: &Type) {
        if let Self::Cons(c) = self {
            c.unify(tv, type_);
        }
    }
}

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
    universe: HashMap<Path, TypeRef>,
    symbols: HashMap<Path, EnvironmentSymbol>,
    constraints: Vec<ConstraintOrGuard>,
    type_var_no: usize,
}

impl Unify for Environment {
    fn unify(&mut self, tv: TypeVariable, type_: &Type) {
        self.symbols.unify(tv, type_);
        self.constraints.unify(tv, type_);
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
        println!("{:#?}", self.symbols);
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
        self.constraints.push(ConstraintOrGuard::LetGuard);
    }

    pub fn end_let(&mut self) -> Result<Vec<Substitution>> {
        let Some((pos, ConstraintOrGuard::LetGuard)) = self
            .constraints
            .iter()
            .enumerate()
            .rfind(|s| matches!(s.1, ConstraintOrGuard::LetGuard))
        else {
            unreachable!("Called `end_let()` outside of a let expression")
        };
        let cons = self
            .constraints
            .split_off(pos + 1)
            .into_iter()
            .map(|c| {
                if let ConstraintOrGuard::Cons(c) = c {
                    c
                } else {
                    unreachable!()
                }
            })
            .collect::<Vec<_>>();
        self.constraints.pop();
        let solution = solve_constraints(&cons)?;
        self.unify_all(&solution);
        Ok(solution)
    }

    pub fn constraint(&mut self, a: TypeRef, b: TypeRef, span: Span) {
        self.constraints
            .push(ConstraintOrGuard::Cons(Constraint(a, b, span)));
    }

    pub fn fresh_type_variable(&mut self) -> TypeVariable {
        let tv = self.type_var_no;
        self.type_var_no += 1;
        tv
    }
}
