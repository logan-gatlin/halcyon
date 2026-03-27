module function =
  use bundle::ops

  (*>
  Returns its input unchanged.

  - Arguments:
    - `value`: Any value.
  - Returns: The same `value`.
  *)
  let identity = fn value => value

  (*>
  Builds a function that always returns one captured value.

  - Arguments:
    - `value`: Value to return for all calls.
  - Returns: A function that ignores its argument and returns `value`.
  *)
  let constant = fn value => fn _ => value

  (*>
  Reverses the argument order of a binary function.

  - Arguments:
    - `f`: Binary function.
  - Returns: Function with swapped argument order.

  ```hc
  let subtract = fn left right => left - right
  let flipped = function::flip subtract
  let value = flipped 1 4
  ```
  *)
  let flip = fn f => fn left right => f right left

  (*>
  Composes two unary functions from left to right.

  - Arguments:
    - `first`: Function applied first.
    - `second`: Function applied to the result of `first`.
    - `value`: Input value.
  - Returns: `second (first value)`.

  ```hc
  let value = function::compose (fn n => n + 1) (fn n => n * 2) 3
  ```
  *)
  let compose = fn first second value => second (first value)

  (*>
  Applies a value to a function.

  - Arguments:
    - `value`: Input value.
    - `f`: Function to call.
  - Returns: `f value`.
  *)
  let pipe = fn value f => f value
end
