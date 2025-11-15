#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Path(String);

impl sx::SXRepr for Path {
    fn sx(self) -> sx::SX {
        sx::SX::Atom(self.0)
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Path {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Path {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<Vec<String>> for Path {
    fn from(value: Vec<String>) -> Self {
        Self(value.join(Self::SEP))
    }
}

impl From<Path> for String {
    fn from(val: Path) -> Self {
        val.0
    }
}

impl From<&Path> for String {
    fn from(val: &Path) -> Self {
        val.0.clone()
    }
}

impl AsRef<str> for Path {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Path {
    const SEP: &str = "::";

    pub fn child(&self, s: impl std::fmt::Display) -> Self {
        let mut new = self.clone();
        new.push(s);
        new
    }

    pub fn push(&mut self, new: impl std::fmt::Display) {
        self.0 = format!("{}{}{new}", self.0, Self::SEP);
    }

    pub fn pop(&mut self) {
        let end = self.0.rfind(Self::SEP).unwrap();
        self.0 = self.0[0..end].to_string();
    }

    pub fn last(&self) -> &str {
        let beginning = self.0.rfind(Self::SEP).unwrap();
        &self.0[beginning..]
    }

    pub fn is_subpath_of(&self, other: &Self) -> bool {
        if self.0.len() < other.0.len() {
            false
        } else {
            self.0
                .bytes()
                .zip(other.0.bytes())
                .fold(true, |cond, (a, b)| cond && a == b)
        }
    }
}
