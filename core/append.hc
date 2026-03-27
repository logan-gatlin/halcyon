module append =
  use bundle

  (*>
  Appends one value to another value of the same type.

  The argument order is `append suffix value`, which makes partial application
  convenient for pipelines and mapping.

  Implement this trait for types that naturally support concatenation.

  ```hc
  let excited = append "!" "hello"
  ```
  *)
  trait Append: t =
    let append : t -> t -> t
  end

  (*>
  Prepends a value by flipping `append` arguments.

  - Arguments:
    - `prefix`: Value placed before `value`.
    - `value`: Value receiving the prefix.
  - Returns: Combined value with `prefix` first.
  *)
  let prepend = fn prefix value => append value prefix

  --> @HIDDEN
  impl Append String =
    let append = fn suffix value => bundle::string::concat value suffix
  end

  --> @HIDDEN
  impl Append for a in Array a =
    let append = fn suffix value => bundle::array::concat value suffix
  end
end
