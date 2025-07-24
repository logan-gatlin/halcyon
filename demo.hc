module A
  type t = integer
  let a = 1
end

module B
  import A
  type t = A:t
  let a = A:a
end

