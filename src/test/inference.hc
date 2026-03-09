module inference =

  -- Defines a generic Pair type for type application inference.
  type ~Pair: a b = (a, b)
  -- Defines a generic Option type for sum inference.
  type Option: a = | None | Some a
  -- Defines a generic Result type for multi-parameter sum inference.
  type Result: ok err = | Ok ok | Err err
  -- Defines a Point record type for struct inference.
  type Point = { x: core::integer, y: core::integer }
  -- Defines a generic Box record type for type-application struct inference.
  type Box: a = { value: a }

  -- Infers integer literal as core::integer.
  let literal_integer: core::integer = 42
  -- Infers real literal as core::real.
  let literal_real: core::real = 3.5
  -- Infers boolean literal as core::boolean.
  let literal_boolean: core::boolean = true
  -- Infers string literal as core::string.
  let literal_string: core::string = "hello"
  -- Infers glyph literal as core::glyph.
  let literal_glyph: core::glyph = 'h'
  -- Infers unit literal as core::unit.
  let literal_unit: core::unit = ()

  -- Infers integer arithmetic precedence and result type.
  let integer_arithmetic: core::integer = 1 + 2 * 3 - 4 / 2
  -- Infers unary minus for integers.
  let integer_negative: core::integer = -1
  -- Infers real arithmetic and real-only operators.
  let real_arithmetic: core::real = 1.0 +. 2.0 *. 3.0 -. 4.0 /. 2.0
  -- Infers unary minus for reals.
  let real_negative: core::real = -1.0

  -- Infers boolean not operator.
  let boolean_not: core::boolean = not false
  -- Infers boolean and operator.
  let boolean_and: core::boolean = true and false
  -- Infers boolean or operator.
  let boolean_or: core::boolean = true or false
  -- Infers boolean xor operator.
  let boolean_xor: core::boolean = true xor false

  -- Infers comparison operators over integers.
  let compare_integers: core::boolean = 1 < 2
  -- Infers comparison operators over strings.
  let compare_strings: core::boolean = "a" != "b"
  -- Infers equality over tuples.
  let compare_tuples: core::boolean = (1, 2) == (1, 2)

  -- Infers if-expression predicate and branch types.
  let if_expression: core::integer = if 1 < 2 then 3 else 4

  -- Infers a polymorphic identity function.
  let identity = fn x => x
  -- Instantiates polymorphic identity at multiple types.
  let identity_use: (core::integer, core::boolean) = (identity 1, identity true)
  -- Infers a constant function with two arguments.
  let constant = fn x => fn y => x
  -- Instantiates constant function with different argument types.
  let constant_use: core::integer = constant 1 false
  -- Checks function annotation and parameter type hints.
  let add_one: core::integer -> core::integer = fn (n: core::integer) => n + 1
  -- Infers multi-argument function types with annotations.
  let add: core::integer -> core::integer -> core::integer =
    fn (a: core::integer) (b: core::integer) => a + b
  -- Infers higher-order function application.
  let apply = fn f => fn x => f x
  -- Uses a function literal as an argument.
  let apply_use: core::integer = apply (fn n => n + 1) 2
  -- Treats a binary operator as a curried function.
  let operator_as_value: core::integer = core::[+] 1 2
  -- Treats unary negation as a first-class function.
  let unary_operator_as_value: core::integer = core::[~] 1
  -- Infers simple arithmetic lambdas for composition.
  let increment = fn n => n + 1
  -- Infers multiplication lambda for composition.
  let double = fn n => n * 2
  -- Infers function composition with >> operator.
  let compose_use: core::integer = (increment >> double) 3
  -- Infers pipe-apply with |> operator.
  let pipe_use: core::integer = 3 |> increment

  -- Infers tuple literal types with annotations.
  let tuple_value: (core::integer, core::boolean) = (1, true)
  -- Uses a local let to destructure tuples.
  let tuple_first: core::integer = let (first, _) = tuple_value in first
  -- Infers a polymorphic tuple projection function.
  let fst = fn pair => let (first, _) = pair in first
  -- Instantiates the tuple projection at concrete types.
  let fst_use: core::integer = fst (1, false)
  -- Uses a generic type alias for a tuple.
  let pair_alias: Pair core::integer core::boolean = (1, true)

  -- Infers array literals from integer elements.
  let int_array = [1, 2, 3]
  -- Requires annotation for empty arrays.
  let empty_int_array: [] core::integer = []
  -- Infers array concatenation via splat syntax.
  let extended_array: [] core::integer = [0, ..int_array, 4]
  -- Infers array pattern matching from a typed parameter.
  let array_head = fn (arr: [] core::integer) =>
    match arr with
    | [x, ..] => x
  -- Instantiates array pattern match function at integers.
  let array_head_use: core::integer = array_head [9, 8, 7]
  -- Infers named array glob bindings in patterns.
  let array_split = fn (arr: [] core::integer) =>
    match arr with
    | [x, ..rest] => (x, rest)
  -- Checks tuple-of-array return types.
  let array_split_use: (core::integer, [] core::integer) = array_split [1, 2, 3]

  -- Infers struct literals when annotated with a named type.
  let origin: Point = { x = 0, y = 0 }
  -- Infers field access on a known struct type.
  let origin_x: core::integer = origin.x
  -- Infers struct literal return types from function annotation.
  let shift_x: Point -> Point = fn (p: Point) => { x = p.x + 1, y = p.y }
  -- Infers generic struct instantiations via type application.
  let boxed_int: Box core::integer = { value = 5 }
  -- Infers field access on instantiated generic structs.
  let boxed_value: core::integer = boxed_int.value

  -- Infers sum constructors for generic options.
  let none_int: Option core::integer = None
  -- Infers sum constructors with payloads.
  let some_int: Option core::integer = Some 1
  -- Infers match expressions over option values.
  let unwrap_or = fn default => fn opt =>
    match opt with
    | None => default
    | Some of value => value
  -- Instantiates option match function with Some.
  let unwrap_or_some: core::integer = unwrap_or 10 (Some 3)
  -- Instantiates option match function with None.
  let unwrap_or_none: core::integer = unwrap_or 10 None
  -- Infers multi-parameter sum constructors.
  let ok_int: Result core::integer core::string = Ok 1
  -- Infers error constructors carrying strings.
  let err_string: Result core::integer core::string = Err "oops"
  -- Infers matches over Result with wildcard patterns.
  let result_default = fn default => fn res =>
    match res with
    | Ok of value => value
    | Err of _ => default
  -- Instantiates Result matching at concrete types.
  let result_default_use: core::integer = result_default 5 err_string

  -- Infers function shorthand with pattern-based branches.
  let bool_to_int = fn | true => 1 | false => 0
  -- Instantiates function shorthand at boolean inputs.
  let bool_to_int_use: core::integer = bool_to_int false

  -- Annotates a polymorphic identity function with forall.
  let (forall_identity: for a in a -> a) = fn x => x
  -- Uses the annotated identity at both integer and boolean.
  let forall_id_int: core::integer = forall_identity 42
  let forall_id_bool: core::boolean = forall_identity true

  -- Annotates a two-parameter polymorphic function with forall.
  let (forall_const: for a b in a -> b -> a) = fn x y => x
  -- Uses the annotated const at concrete types.
  let forall_const_use: core::integer = forall_const 1 "hello"
end
