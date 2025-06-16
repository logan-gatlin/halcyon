(module
  (type $myarray (array (mut i8)))
  (func
    (array.new_fixed $myarray 2
      (i32.const 1)
      (i32.const 2)
    )
    drop
  )
)
