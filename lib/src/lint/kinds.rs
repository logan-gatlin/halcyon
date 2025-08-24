macro_rules! convert {
  ($($lint:ident),*) => {
    $(impl From<$lint> for usize {
      fn from(lint: $lint) -> usize {
        lint as usize
      }
    })*
  };
}

convert!(TokenLint, ParseLint, NameLint, TypeLint, CompilerBug);

#[repr(usize)]
pub enum TokenLint {
    InvalidInput = 1000,
    UnrecognizedEscape = 1001,
    MissingDelimeter = 1002,
    ExtraDelimeter = 1003,
    WrongGlyphSize = 1004,
    InvalidInteger = 1005,
    InvalidReal = 1006,
}

#[repr(usize)]
pub enum ParseLint {
    ExpectedToken = 2000,
    ExpectedOneOf = 2001,
    ExpectedExpression = 2002,
}

#[repr(usize)]
pub enum NameLint {
    UndefinedName = 3000,
    NameRedefinition = 3001,
    ParamRedefinition = 3002,
    NotImported = 3003,
    NoSuchModule = 3004,
    CyclicalDefinition = 3005,
}

#[repr(usize)]
pub enum TypeLint {
    TypeMismatch = 4000,
    AmbiguousExpression = 4001,
    NonExistantField = 4002,
    NoStructWithFields = 4003,
    NonExhaustive = 4004,
}

#[repr(usize)]
pub enum CompilerBug {
    FailedValidation = 9000,
}
