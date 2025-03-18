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
  UnexpectedToken = 2000,
  MissingBody = 2001,
  MissingBinaryOperand = 2002,
  MissingPrefixUnaryOperand = 2003,
  MissingPostfixUnaryOperand = 2004,
  MissingComma = 2005,
  MissingFunctionParameterType = 2006,
  ExpectedIdentifier = 2007,
  MissingAssignee = 2008,
  MissingSemicolon = 2009,
  MissingLoopParameter = 2010,
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
