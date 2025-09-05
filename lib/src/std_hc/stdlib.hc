module std =
    let assert = fn with
      | true => ()
      | false => std::panic ()
    let println = fn (a: std::string) => ()
end

module string =
  type t = std::string
  let s = string::concatenate

  -- Author: Logan Williams
  let from_integer = 
    fn x => 
      let digit_to_string = fn with
        | 0 => "0"
        | 1 => "1"
        | 2 => "2"
        | 3 => "3"
        | 4 => "4"
        | 5 => "5"
        | 6 => "6"
        | 7 => "7"
        | 8 => "8"
        | 9 => "9"
        | _ => "?"
      in
    match x with
      | 0 => ""
      | x => (x % 10)
        |> digit_to_string
        |> let a = from_integer (x / 10) in
          string::concatenate a
end

module opt =
  type t = fn a => Some of a | None of ()
end

module result =
  type t = fn a b => Ok of a | Error of b
end

module opt =
  let map = fn operation maybe => match maybe with
    | opt::Some of o => opt::Some (operation o)
    | _ => opt::None ()

  let iterate =
    fn operation maybe => (map (fn a => operation a; a) maybe); ()

  let unwrap = fn maybe => match maybe with
    | opt::Some of o => o
    | _ => std::panic ()

  let is_some = fn maybe => match maybe with
    | opt::Some of _ => true
    | _ => false

  let is_none = fn maybe => not (is_some maybe)

  let ok_or_error = fn error opt => match opt with
    | opt::None of () => result::Error error
    | opt::Some of a => result::Ok a
end

module result =
let is_ok = fn a => match a with 
  | result::Ok of _ => true
  | _ => false
  
  let is_err = fn a => match a with 
  | result::Error of _ => true
  | _ => false
  
  let unwrap_ok = fn a => match a with 
  | result::Ok of val => val
  | _ => std::panic ()
  
  let unwrap_err = fn a => match a with
  | result::Error of val => val
  | _ => std::panic ()
  
  let and_also = fn a res => match a with 
  | result::Ok of _ => res
  | result::Error of _ => a
  
  (*
  let expect = fn msg a => match a with 
  | result::Ok of val => val
  | result::Error of _ => std:print_string msg; std:panic ()
  *)
  
  let or_else = fn a res => match a with 
  | result::Ok of _ => a
  | result::Error of _ => res
  
  let unwrap_or = fn a default => match a with
  | result::Ok of val => val 
  | result::Error of _ => default
  
  let and_then = fn a op => match a with
  | result::Ok of val => result::Ok (op val)
  | result::Error of _ => a 

  let ok_or_none = fn with
    | result::Ok of val => opt::Some val
    | _ => opt::None ()
end

module list =
  type t = fn I => Pair of I * (t I) | Nil of ()

  let map = fn operation list => match list with
    | Pair of (head, tail) => Pair (operation head, tail |> map operation)
    | Nil of () => Nil ()

  let iterate =
    fn operation list => (map (fn a => operation a; a) list); ()

  let push = fn item list => match list with
    | Pair of (head, tail) => Pair (head, push item tail)
    | Nil of () => Pair (item, Nil ())

  let length = fn list => match list with
    | Pair of (head, tail) => 1 + (length tail)
    | Nil of () => 0

  let fold = fn op acc list => match list with
    | Pair of (head, tail) => op acc (fold op head tail)
    | Nil of () => acc

  let concatenate = fn list1 list2 => match list1 with
    | Pair of (head, tail) => Pair (head, concatenate tail list2)
    | Nil of () => list2

  let nth = fn n list => match (n, list) with
    | (0, Pair of (head, _)) => opt::Some head
    | (n, Pair of (head, tail)) => nth (n - 1) tail
    | (_, Nil of ()) => opt::None ()
end
