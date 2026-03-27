(*>
Shared interfaces for container-like types (`Option`, `Result`, `Array`, and others).

If these terms are new, this module can still be useful. A quick guide:

- `a`, `b` are placeholder value types.
- `m`, `f`, `t` are placeholder container types.
- `m a` means "an `a` value inside container `m`".
- `for a b in ...` means "this function works for any `a` and `b`".

Suggested first reading order:
1. `Functor` (`fmap`): apply a plain function inside a container.
2. `Applicative` (`apply`): apply a wrapped function to a wrapped value.
3. `Monad` (`new`, `flat_map`): chain steps where each step returns a wrapped value.
4. `Traversable` and `Foldable`: process whole structures like arrays.
*)
module hkt =
  use bundle
  use bundle::ops

  (*>
  `Applicative` lets you combine wrapped functions with wrapped values.

  Signature:
  - `apply : m (a -> b) -> m a -> m b`

  Read this as:
  - "If I have a function inside `m`"
  - "and a value inside `m`"
  - "I can get the result inside `m`"

  Example with `Option`:
  - `apply (Some (fn n => n + 1)) (Some 41)` gives `Some 42`.
  - If either side is `None`, the result is `None`.
  *)
  trait Applicative: m =
    let apply : for a b in m (a -> b) -> m a -> m b
  end

  (*>
  `Monad` is for chaining dependent steps in the same container type.

  - `new` wraps a plain value.
  - `flat_map` runs a function that already returns a wrapped value.

  Why `flat_map` matters:
  if your function returns `m b`, plain mapping would give `m (m b)`.
  `flat_map` avoids that extra nesting.

  Example with `Option`:

  ```hc
  let require_positive = fn n => if n > 0 then Some n else None
  let add_one = fn n => Some (n + 1)

  let ok = flat_map add_one (flat_map require_positive (new 5))
  let missing = flat_map add_one (flat_map require_positive (new 0))
  ```
  *)
  trait Monad: m =
    let new : for a in a -> m a
    let flat_map : for a b in (a -> m b) -> m a -> m b
  end

  (*>
  Operator alias for `flat_map`.

  `x *> f` is the same as `flat_map f x`.

  This can be easier to read left-to-right when chaining:

  ```hc
  let step1 = fn n => if n > 0 then Some n else None
  let step2 = fn n => Some (n + 1)

  let result = (Some 3 *> step1) *> step2
  ```
  *)
  let [*>] = fn a b => flat_map b a

  (*>
  `Traversable` walks through a structure and runs an effectful function on each element.

  "Effectful" here means the function returns a wrapped value (`m b`), not a plain `b`.

  `traverse` turns:
  - `(a -> m b)` and `t a`
  into:
  - `m (t b)`

  Intuition:
  - map each element with a function that can fail/branch/do effects,
  - then collect the results back into one wrapped structure.

  ```hc
  let keep_non_zero = fn n => if n == 0 then None else Some n
  let values = [2, 4, 0, 8]

  let checked = traverse keep_non_zero values
  ```

  For `Option`, `checked` is `None` because one element produced `None`.
  *)
  trait Traversable: t =
    let traverse : for m a b in (a -> m b) -> t a -> m (t b) where Monad m
  end

  (*>
  `Foldable` reduces a structure to one accumulated value.

  `fold step initial items`:
  - starts from `initial`,
  - visits each element in `items`,
  - updates the accumulator with `step`.

  ```hc
  let total = fold (fn acc n => acc + n) 0 [1, 2, 3, 4]
  ```
  *)
  trait Foldable: t =
    let fold : for a b in (b -> a -> b) -> b -> t a -> b
  end

  (*>
  `Alternative` adds "empty" and "choice" operations for a container.

  - `empty` is the identity value meaning "no result".
  - `or_else left right` picks `left` when it has a result, otherwise `right`.

  Example with `Option`:

  ```hc
  let primary = None
  let fallback = Some 8080

  let chosen = or_else primary fallback
  ```
  *)
  trait Alternative: m =
    let empty : for a in m a
    let or_else : for a in m a -> m a -> m a
  end

  (*>
  `Comonad` is useful when you want to compute from a value *and* its surrounding context.

  It is often described as a dual to `Monad`, but you do not need that idea to use it:

  - `extract` pulls out the focused value.
  - `extend` rebuilds a wrapped value by running a function that can inspect the whole wrapped input.

  Tiny example with an identity wrapper:

  ```hc
  type Id: a = | Id a

  let id_extract = fn value => match value with
    | Id inner => inner

  let id_extend = fn f value => Id (f value)
  ```

  Real-world comonads are usually structures like zippers or non-empty streams,
  where each position has neighborhood context.
  *)
  trait Comonad: w =
    let extract : for a in w a -> a
    let extend : for a b in (w a -> b) -> w a -> w b
  end

  (*>
  `Functor` is the basic "map inside a container" interface.

  `fmap` applies a plain function to inner values while keeping the outer shape.

  Example with `Option`:

  ```hc
  let doubled = fmap (fn n => n * 2) (Some 21)
  ```

  Result: `Some 42`. If input is `None`, result stays `None`.
  *)
  trait Functor: f =
    let fmap : for a b in (a -> b) -> f a -> f b
  end

  (*>
  `Bifunctor` is for types with two type parameters, such as `Result err ok`.

  `bimap left_fn right_fn` maps both sides:
  - `left_fn` transforms the first type argument.
  - `right_fn` transforms the second type argument.

  ```hc
  let success = Ok 4
  let bumped = bimap (fn err => "error: " + err) (fn n => n + 1) success
  ```
  *)
  trait Bifunctor: p =
    let bimap : for a b c d in (a -> b) -> (c -> d) -> p a c -> p b d
  end

  (*>
  `Zip` combines two containers position-by-position.

  `zip_with f left right` takes one item from each side and applies `f`.
  The result keeps positional alignment instead of doing a full cross-product.

  ```hc
  let left = [1, 2, 3]
  let right = [10, 20, 30]

  let summed = zip_with (fn a b => a + b) left right
  ```
  *)
  trait Zip: z =
    let zip_with : for a b c in (a -> b -> c) -> z a -> z b -> z c
  end

  (*>
  `Filterable` removes values that do not satisfy a predicate.

  - For arrays, this drops elements.
  - For `Option`, `Some value` becomes `None` when the predicate fails.

  ```hc
  let kept = filter (fn n => n > 0) [-2, 0, 3, 5]
  ```
  *)
  trait Filterable: f =
    let filter : for a in (a -> Boolean) -> f a -> f a
  end

  (*>
  Maps a plain function over a monadic value.

  This is derived from `new` and `flat_map`, so every `Monad` automatically gets `map`.

  - Arguments:
    - `x`: Wrapped input `m a`.
    - `f`: Function `a -> b`.
  - Returns: Wrapped output `m b`.

  ```hc
  let incremented = hkt::map (Some 41) (fn n => n + 1)
  ```
  *)
  let map = fn x f => x *> (fn value => new (f value))

  (*>
  Infix alias for `map`.

  `x +> f` is the same as `map x f`.

  ```hc
  let result = Some 41 +> (fn n => n + 1)
  ```
  *)
  let [+>] = map

  (*>
  Removes one monadic layer.

  Use this when you have `m (m a)` and want `m a`.

  - Arguments:
    - `x`: Nested wrapped value.
  - Returns: Flattened wrapped value.

  ```hc
  let nested = Some (Some 42)
  let one_layer = hkt::flatten nested
  ```
  *)
  let flatten = fn x => flat_map (fn value => value) x

  (*>
  Applies a wrapped function to a wrapped value using only `Monad` operations.

  This behaves like `Applicative.apply`, but is available when you only know `Monad`.

  - Arguments:
    - `mf`: Wrapped function (`m (a -> b)`).
    - `mx`: Wrapped argument (`m a`).
  - Returns: Wrapped result (`m b`).

  ```hc
  let wrapped_fn = Some (fn n => n + 1)
  let wrapped_value = Some 41
  let incremented = hkt::ap wrapped_fn wrapped_value
  ```
  *)
  let ap = fn mf mx => flat_map (fn f => map mx f) mf

  (*>
  Lifts a two-argument function to work on wrapped inputs.

  `lift2 f mx my` runs `f` only when both wrapped inputs can provide values.

  - Arguments:
    - `f`: Binary function.
    - `mx`: First wrapped input.
    - `my`: Second wrapped input.
  - Returns: Wrapped result.

  ```hc
  let left = Some 20
  let right = Some 22
  let summed = hkt::lift2 (fn a b => a + b) left right
  ```
  *)
  let lift2 = fn f mx my => flat_map (fn x => map my (fn y => f x y)) mx

  --> @HIDDEN
  let replace_with = fn replacement mx => map mx (fn _ => replacement)

  --> @HIDDEN
  let sequence_next = fn left right => flat_map (fn _ => right) left

  --> @HIDDEN
  let discard_value = fn mx => map mx (fn _ => ())

  (*>
  Flips nesting: from a structure of wrapped values to one wrapped structure.

  Input shape: `t (m a)`
  Output shape: `m (t a)`

  Useful when you collect many effectful computations and want one combined result.

  - Arguments:
    - `value`: Structure containing wrapped values.
  - Returns: One wrapped structure.

  ```hc
  let all_present = hkt::sequence [Some 1, Some 2, Some 3]
  let has_gap = hkt::sequence [Some 1, None, Some 3]
  ```
  *)
  let sequence = fn value => traverse (fn item => item) value

  --> @HIDDEN
  let duplicate = fn value => extend (fn inner => inner) value

  --> @HIDDEN
  let extend_map = fn f value => extend (fn inner => f (extract inner)) value

  --> @HIDDEN
  let fold_is_empty = fn items => fold (fn _ _ => false) true items

  --> @HIDDEN
  let fold_non_empty = fn items => fold (fn _ _ => true) false items

  --> @HIDDEN
  let fold_count = fn items => fold (fn count _ => count + 1) 0 items

  (*>
  Returns `true` if at least one element satisfies `pred`.

  - Arguments:
    - `pred`: Predicate run on each element.
    - `items`: Foldable structure.
  - Returns: Match flag.

  ```hc
  let has_zero = hkt::any (fn value => value == 0) [4, 0, 9]
  ```
  *)
  let any =
    fn pred items => fold (fn acc item => if acc then true else pred item) false items

  (*>
  Returns `true` only if every element satisfies `pred`.

  - Arguments:
    - `pred`: Predicate run on each element.
    - `items`: Foldable structure.
  - Returns: Universal match flag.

  ```hc
  let all_positive = hkt::all (fn value => value > 0) [1, 2, 3]
  ```
  *)
  let all =
    fn pred items => fold (fn acc item => if acc then pred item else false) true items

end
