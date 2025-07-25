module std
  import builtin
  let assert = fn to_test => if to_test then () else builtin:panic()
end
