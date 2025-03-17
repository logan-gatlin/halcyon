main :: () {
  fizzbuzz(15)
}

fizzbuzz :: (number: integer) {
  loop i : 0 {
    if i % 3 == 0 and i % 5 == 0 {
      print_string("fizzbuzz")
    } else if i % 3 == 0 {
      print_string("fizz")
    } else if i % 5 == 0 {
      print_string("buzz")
    } else {
      print_integer(i)
    }
    if i >= number {
      break
    }
    i + 1
  }
}

