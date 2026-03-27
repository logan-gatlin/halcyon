module unwrap =
  use bundle

  (*>
  Shared unwrapping interface for container types.

  `unwrap_or_else` is the core operation. Other unwrap variants are derived
  from it.
  *)
  trait Unwrap: m a =
    let unwrap_or_else : (Unit -> a) -> m a -> a
  end

  (*>
  Extracts a value or uses a fallback value.

  This is derived from `unwrap_or_else`.
  *)
  let unwrap_or = fn backup wrapped =>
    unwrap_or_else (fn _ => backup) wrapped

  (*>
  Extracts a value, panicking when absent.

  This is derived from `unwrap_or_else`.
  *)
  let unwrap = fn wrapped =>
    unwrap_or_else (fn _ => bundle::test::panic "called unwrap on empty value") wrapped
end
