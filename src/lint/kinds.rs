macro_rules! convert {
  ($($lint:ident),*) => {
    $(impl Into<usize> for $lint {
      fn into(self) -> usize {
        self as usize
      }
    })*
  };
}

convert!(TokenLint, ParseLint, NameLint, TypeLint, EvalLint);

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
  BadPostfix = 2000,
  BadPrefix = 2001,
  BadInfix = 2002,
  InvalidIf = 2003,
  InvalidLoop = 2004,
  InvalidGuard = 2005,
  EmptyBlock = 2006,
  EmptyInput = 2007,
  MissingNewLine = 2008,
  AssignToExpression = 2009,
  AmbiguousList = 2010,
}

#[repr(usize)]
pub enum NameLint {
  UndefinedName = 3000,
  ConstRedefinition = 3001,
  ParamRedefinition = 3002,
  InvalidMain = 3003,
  FieldNotIdent = 3004,
  MultipleLoopParams = 3005,
  NoBreakTarget = 3006,
}

#[repr(usize)]
pub enum TypeLint {
  TypeMismatch = 4000,
  NoFieldOnType = 4007,
  FieldMissing = 4008,
  NonFunctionCall = 4009,
  TooManyArgs = 4010,
  TooFewArgs = 4011,
  BinaryOpUndefined = 4012,
  UnaryOpUndefined = 4013,
  Sanitization = 4014,
}

#[repr(usize)]
pub enum EvalLint {
  RecursionLimit = 5000,
  Unreachable = 5001,
  Circular = 5002,
  GlyphOutOfRange = 5003,
}
