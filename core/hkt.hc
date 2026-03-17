module hkt =
  use core
  use core::ops

  -- Supports combining wrapped computations with wrapped functions.
  trait Applicative: m =
    -- Applies a wrapped function to a wrapped input.
    let apply : for a b in m (a -> b) -> m a -> m b
  end

  -- Supports creating values and sequencing dependent computations.
  trait Monad: m =
    -- Lifts a plain value into the context.
    let new : for a in a -> m a
    -- Runs the next step using the value inside the context.
    let flatmap : for a b in (a -> m b) -> m a -> m b
  end

  -- Walks a structure while collecting effects.
  trait Traversable: t =
    -- Applies an effectful function to each element and rebuilds the structure.
    let traverse : for m a b in (a -> m b) -> t a -> m (t b) where Monad m
  end

  -- Reduces a structure into a single accumulated value.
  trait Foldable: t =
    -- Left-associative fold over the structure.
    let fold : for a b in (b -> a -> b) -> b -> t a -> b
  end

  -- Represents failure and fallback choices in one context.
  trait Alternative: m =
    -- The identity element for choice.
    let empty : for a in m a
    -- Chooses the first successful value, or falls back to the second.
    let or_else : for a in m a -> m a -> m a
  end

  -- Supports querying and extending context-aware values.
  trait Comonad: w =
    -- Reads the focused value from the context.
    let extract : for a in w a -> a
    -- Rebuilds the context by evaluating neighborhoods.
    let extend : for a b in (w a -> b) -> w a -> w b
  end

  -- Supports transforming values while preserving structure.
  trait Functor: f =
    -- Maps a pure function over the wrapped value.
    let fmap : for a b in (a -> b) -> f a -> f b
  end

  -- Supports mapping both sides of a two-parameter context.
  trait Bifunctor: p =
    -- Maps both type parameters in one pass.
    let bimap : for a b c d in (a -> b) -> (c -> d) -> p a c -> p b d
  end

  -- Supports position-wise combination of two contexts.
  trait Zip: z =
    -- Combines corresponding elements using a binary function.
    let zip_with : for a b c in (a -> b -> c) -> z a -> z b -> z c
  end

  -- Supports selecting values while keeping the same context shape.
  trait Filterable: f =
    -- Keeps only values that satisfy the predicate.
    let filter : for a in (a -> Boolean) -> f a -> f a
  end

  -- `map` derived from Monad primitives.
  let map = fn f x => flatmap (fn value => new (f value)) x

  -- `flatten` derived from Monad primitives.
  let flatten = fn x => flatmap (fn value => value) x

  -- Applicative-style apply derived from Monad primitives.
  let ap = fn mf mx => flatmap (fn f => map f mx) mf

  -- Lift a two-argument function over two contexts.
  let lift2 = fn f mx my => flatmap (fn x => map (fn y => f x y) my) mx

  -- Replace every wrapped value with one constant value.
  let replace_with = fn replacement mx => map (fn _ => replacement) mx

  -- Sequence two contexts, returning the second result.
  let sequence_next = fn left right => flatmap (fn _ => right) left

  -- Drop wrapped values and keep only Unit.
  let discard_value = fn mx => map (fn _ => ()) mx

  -- Turn `t (m a)` into `m (t a)`.
  let sequence = fn value => traverse (fn item => item) value

  -- Duplicate comonadic context.
  let duplicate = fn value => extend (fn inner => inner) value

  -- Map a function over a comonad using `extract` + `extend`.
  let extend_map = fn f value => extend (fn inner => f (extract inner)) value

  -- Return true when the foldable has no elements.
  let fold_is_empty = fn items => fold (fn _ _ => false) true items

  -- Return true when the foldable has at least one element.
  let fold_non_empty = fn items => fold (fn _ _ => true) false items

  -- Count elements in a foldable structure.
  let fold_count = fn items => fold (fn count _ => count + 1) 0 items

  -- Return true when any element satisfies the predicate.
  let any =
    fn pred items => fold (fn acc item => if acc then true else pred item) false items

  -- Return true when all elements satisfy the predicate.
  let all =
    fn pred items => fold (fn acc item => if acc then pred item else false) true items

end
