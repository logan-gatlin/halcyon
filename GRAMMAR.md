# Halcyon Informal Grammar

## Program Structure
```bnf
<file>       ::= <module>*
<module>     ::= "module" <ident> "=" <statement>* "end"
<statement>  ::= "let" <pattern> "=" <expr>
               | "type" <ident> (":" <ident>+)? "=" <type_def>
```

## Type Definitions
```bnf
<type_def>   ::= "{" <field_decl>+ "}"             -- Record definition
               | ("|" <variant>)*        -- Sum type (Variant)
               | <type_expr>                       -- Type alias

<field_decl> ::= <ident> ":" <type_expr> ("," <ident> ":" <type_expr>)* ","?
<variant>    ::= <ident> (<type_expr>)?
```

## Type Expressions
```bnf
<type_expr>  ::= <type_term> "->" <type_expr>      -- Function type
               | <type_term> <type_term>+          -- Type application (e.g., `map int string`)
               | <type_term>

<type_term>  ::= <ident> | <path>                  -- Type name
               | "(" <type_expr> ("," <type_expr>)* ")" -- Tuple or grouping
               | "(" ")"                           -- Unit type
               | "[" "]"                           -- Array type constructor
```

## Expressions
```bnf
<expr>       ::= "let" <pattern> "=" <expr> "in" <expr>
               | "fn" <parameter>* "=>" <expr>     -- Function definition
               | "fn" <match_arm>+                 -- Function shorthand (lambda case)
               | "if" <expr> "then" <expr> "else" <expr>
               | "match" <expr> "with" <match_arm>+
               | <binary_op_chain>

<parameter>  ::= <ident>
               | "(" <ident> ":" <type_expr> ")"

<match_arm>  ::= "|" <pattern> "=>" <expr>

<binary_op_chain> ::= <term> <op> <binary_op_chain> | <term>

<term>       ::= <term> <term>                     -- Function application
               | <term> "." <ident>                -- Field access
               | <unary_op> <term>
               | <literal>
               | <ident> | <path>
               | "(" <expr> ("," <expr>)* ")"      -- Tuple or Grouping
               | "[" <array_elem>* "]"             -- Array literal
               | "{" <field_def>+ "}"              -- Struct literal

<array_elem> ::= <expr> | ".." <ident>              -- Value or Splat expansion
<field_def>  ::= <ident> ("=" | ":") <expr>
```

## Patterns
```bnf
<pattern>    ::= <pattern> ":" <type_expr>         -- Type hint
               | <ident> | <path>
               | <path> "of" <pattern>             -- Constructor match
               | <literal>
               | "(" <pattern> ("," <pattern>)* ")"
               | "[" <pat_array_elem>* "]"
               | "{" <pat_field>+ "}"

<pat_array_elem> ::= <pattern> | ".." <ident>?     -- Rest pattern
<pat_field>      ::= <ident> ("=" <pattern>)?      -- Field match (optional shorthand)
```

## Lexical Elements
*   **Literals:** Integers (`123`, `0xEF`), Reals (`1.0`), Strings (`"text"`), Glyphs (`'c'`), Booleans (`true`, `false`), Unit (`()`).
*   **Comments:** `-- line comment`, `(* block comment *)`.
*   **Paths:** `Module::Ident`.
*   **Operators:** `+`, `-`, `*`, `/` (Int); `+.`, `-.`, `*.`, `/.` (Real); `|>` (Pipe); `==`, `!=`, `<`, `and`, `or`, `not`, etc.
