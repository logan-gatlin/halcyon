macro_rules! convert {
  ($($lint:ident),*) => {
    $(impl Into<usize> for $lint {
      fn into(self) -> usize {
        self as usize
      }
    })*
  };
}

convert!(TokenLint, ParseLint, NameLint, TypeLint);

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
  InvalidMatch = 2005,
  EmptyBlock = 2006,
  EmptyInput = 2007,
  InvalidStructure = 2008,
  InvalidLet = 2009,
  MissingComma = 2010,
  InvalidFunctionArgument = 2011,
  InvalidPattern = 2012,
  ExpectedExpression = 2013,
  InvalidDirective = 2014,
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
  UnrecognizedDirective = 3007,
  NonUniqueExport = 3008,
  NonUniqueImport = 3009,
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
  NotAType = 4014,
  NotAvailable = 4015,
}
