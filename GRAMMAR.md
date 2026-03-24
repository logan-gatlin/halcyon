# Halcyon Informal Grammar

## Program Structure
```bnf
<file>       ::= <file_item>*
<file_item>  ::= <statement>
<bundle_declaration> ::= "bundle" <ident>
<import_statement> ::= "import" <string> ("," <string>)*
<module>     ::= "module" <ident> "=" <statement>* "end"

<statement>  ::= <let_statement>
               | <do_statement>
               | <bundle_declaration>
               | <import_statement>
               | <use_statement>
               | <type_statement>
               | <trait_statement>
               | <impl_statement>
               | <module>
               | <wasm_statement>

<let_statement>  ::= "let" <pattern> "=" <expr>
                   | "let" "|" <ident> "=" (<ident> | <path>)
<do_statement> ::= "do" <expr>
<use_statement> ::= "use" (<ident> | <path>) ("as" <ident>)?
<type_statement> ::= "type" "~"? <ident> (":" <ident>+)? "=" (<type_def> | <type_expr>)
<trait_statement> ::= "trait" "~"? <ident> (":" <ident>+)? "=" (<trait_item_decl>* "end" | (<ident> | <path>))
<impl_statement> ::= "impl" (<ident> | <path>) <type_expr> ("," <type_expr>)* "=" <impl_item_def>* "end"
<trait_item_decl> ::= <trait_method_decl> | <trait_type_decl>
<impl_item_def> ::= <impl_method_def> | <impl_type_def>
<trait_method_decl> ::= "let" <ident> ":" <type_expr>
<trait_type_decl> ::= "type" <ident>
<impl_method_def> ::= "let" <ident> "=" <expr>
<impl_type_def> ::= "type" <ident> "=" <type_expr>
<wasm_statement> ::= "wasm" "=>" <sexpr>
```

- `bundle` and `import` are regular statements and can appear anywhere statements are allowed.
- Top-level statements are part of the bundle scope; a `module ... end` wrapper is optional.
- CLI bundle compilation requires the root file to start with `bundle <name>`.
- A bundle may only be declared once across the entire import graph.
- `compile_source` accepts files without a `bundle` declaration and uses implicit bundle name `_`.

## Type Definitions
```bnf
<type_def>   ::= "{" <struct_member>+ "}"        -- record definition
               | ("|" <variant>)+                  -- sum type
               | <type_expr>                        -- named type body

Type declarations are split by the `~` marker:
- `type Name ... = <type_def>` defines a nominal named type.
- `type ~Name ... = <type_expr>` defines a structural type alias.

Trait declarations may also use `~`:
- `trait Name ... = ... end` defines a trait.
- `trait ~Alias = <trait>` defines a trait alias to an existing trait.

Recursion rules:
- Recursive type aliases are rejected.
- Recursive nominal definitions are allowed only for sum types (`| ...`).

<struct_member> ::= <field_decl>
                  | ".." <type_expr>

<field_decl> ::= <ident> ":" <type_expr>
<variant>    ::= <ident> <type_expr>?
```

## Type Expressions
```bnf
<type_expr>  ::= "for" <ident>+
                 "in" <type_expr>
                 ("where" <trait_constraint> ("," <trait_constraint>)*)?
                | <type_term> "->" <type_expr>       -- function type
                | <type_term> <type_term>+           -- type application
                | <type_term>

<trait_constraint> ::= (<ident> | <path>) <type_term>*

<type_term>  ::= <ident>
               | <path>
               | "(" <type_expr> ("," <type_expr>)* ")" -- tuple or grouping
               | "(" ")"                            -- unit type
               | "[" "]"                            -- array type constructor
```

## Expressions
```bnf
<expr>       ::= "let" <pattern> "=" <expr> "in" <expr>
               | "use" (<ident> | <path>) ("as" <ident>)? "in" <expr>
               | "fn" <parameter>* "=>" <expr>      -- function definition
               | "fn" <match_arm>+                   -- function shorthand
               | "if" <expr> "then" <expr> "else" <expr>
               | "match" <expr> "with" <match_arm>+
               | <inline_wasm_expr>
               | <binary_op_chain>

<inline_wasm_expr> ::= "(" "wasm" ":" <type_expr> ")" "=>" <sexpr>

<parameter>  ::= <ident>
               | "(" <ident> ":" <type_expr> ")"

<match_arm>  ::= "|" <pattern> "=>" <expr>

<binary_op_chain> ::= <term> <op> <binary_op_chain>
                    | <term>

<term>       ::= <term> <term>                      -- function application
               | <term> "." <ident>                 -- field access
               | <unary_op> <term>
               | <literal>
               | <ident>
               | <path>
               | "(" <expr> ("," <expr>)* ")"       -- tuple or grouping
               | "[" <array_elem>* "]"              -- array literal
               | "{" <field_def>+ "}"               -- struct literal

<array_elem> ::= <expr>
               | ".." <expr>                        -- splat expansion

<field_def>  ::= <ident> ("=" | ":") <expr>
```

## Patterns
```bnf
<pattern>    ::= <pattern> ":" <type_expr>          -- type hint
               | <ident>
               | <path>
               | <path> <pattern>                   -- constructor match
               | <literal>
               | "(" <pattern> ("," <pattern>)* ")"
               | "[" <pat_array_elem>* "]"
               | "{" <pat_field>+ "}"

<pat_array_elem> ::= <pattern>
                   | ".." <ident>?                  -- rest pattern

<pat_field>      ::= <ident> ("=" <pattern>)?       -- field match (optional shorthand)
```

## S-Expressions (inline wasm)
```bnf
<sexpr>      ::= "(" <sexpr_item>* ")"

<sexpr_item> ::= <sexpr>
               | <sexpr_path>
               | <sexpr_ident>
               | <sexpr_field>
               | <string>
               | <integer>
               | <real>
               | "true"
               | "false"

<sexpr_path>  ::= "$" <ident> "::" <ident>
<sexpr_ident> ::= "$" <ident> | <ident>
<sexpr_symbol_ident> ::= "$" <ident>
<sexpr_field> ::= <ident> "." <ident>
```

### Inline WASM forms
```bnf
<wasm_declaration> ::= "(" "type" <sexpr_symbol_ident> <wasm_type> ")"
                     | "(" "global" <sexpr_symbol_ident> <wasm_type> ")"
                     | "(" "func" <sexpr_symbol_ident> <func_section>* <instruction>* ")"
                     | "(" "memory" <sexpr_symbol_ident> <integer> <integer>? ")"

<func_section>      ::= "(" "param" (<sexpr_symbol_ident> <wasm_type>)+ ")"
                     | "(" "result" <wasm_type>+ ")"
                     | "(" "local" (<sexpr_symbol_ident> <wasm_type>)+ ")"

<inline_wasm_body>  ::= "(" <local_decl>* <instruction>* ")"
<local_decl>        ::= "(" "local" (<sexpr_symbol_ident> <wasm_type>)+ ")"

<instruction>       ::= <sexpr_ident> <sexpr_item>*

<wasm_type>         ::= "any" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64"
                     | <sexpr_ident>
                     | "(" "struct" <wasm_type>* ")"
                     | "(" "array" <wasm_type> ")"
                     | "(" "func" ("(" "param" <wasm_type>* ")")* ("(" "result" <wasm_type>* ")")* ")"
```

- Instruction streams are flat (token-by-token), not nested WAT-style instruction trees.
- In `(wasm : ...) => (...)` expressions, only `(local ...)` declarations are valid before instructions.
- `wasm =>` accepts either a single `<wasm_declaration>` or a parenthesized list of declarations.
- Memory limits use 32-bit page counts; when a maximum is present, it must be >= the initial size.
- The first token of each instruction is an opcode identifier (for example `get`, `call`, `struct.new`, `i32.add`).

## Lexical Elements
- **Literals:** integers (`123`, `0xEF`), reals (`1.0`), strings (`"text"`), glyphs (`'c'`), booleans (`true`, `false`), unit (`()`).
- **Comments:** `-- line comment`, `(* block comment *)`.
- **Identifiers:** bare names (`foo`) or bracketed operators (`[+]`, `[ + ]`, `[not]`).
- **Paths:** `("root" "::" <ident> ("::" <ident>)*) | (<ident> "::" <ident> ("::" <ident>)*)` (each segment may use a bracketed operator identifier).
- **Resolved path shape:** internally paths are `major::minor`, where `major` is the bundle name and `minor` is the declaration path inside that bundle.
- **Path resolution:** `root::...` is fully qualified; non-rooted paths resolve relative to the current module scope first, then may resolve through `use`, then fall back to absolute `<bundle>::...`.
- **`use` resolution:**
  - Module-level `use` applies only to following statements in the same module.
  - Expression-level `use ... in ...` applies only inside its `in` body.
  - `use M` opens `M` into the current scope.
  - `use M as X` adds alias `X` for module path lookups (`X::name`) without opening contents.
  - `as` alias name collisions are errors.
  - If multiple opened modules provide the same symbol, the usage is ambiguous and reported as an error.
- **Operators:** `+`, `-`, `*`, `/`, `mod`, `|>`, `+>`, `*>`, `>>`, '<<', `==`, `!=`, `<`, `<=`, `>`, `>=`, `and`, `or`, `xor`, `not`, `;`.
