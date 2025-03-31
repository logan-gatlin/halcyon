print :: () => () ;;
to_string :: () => () ;;

fizzbuzz :: max =>
  (i = 0) loop
    if i > max then break () else
    print ((i % 3, i % 5) match
      | 0, 0 then "fizzbuzz"
      | 0, _ then "fizz"
      | _, 0 then "buzz"
      | _, _ then to_string i);
      i + 1 ;;

fizzbuzz 15
