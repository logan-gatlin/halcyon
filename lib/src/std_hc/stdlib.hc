-- Authorship:
--   Logan Gatlin
--   Logan Williams

module string =
  type t = std::string
  let s = string::concatenate

  import unsafe_print : (std::integer * std::integer) -> () = sys::print_string
  let print = fn s =>
    string::unsafe_store s 0;
    unsafe_print (0, (string::length s))
end

module std =
  let panic = fn (_ : ()) => wasm::unreachable ()
  let assert = fn with
    | true => ()
    | false => panic ()

  let assert_eq = fn a b => assert (a == b)
  let assert_ne = fn a b => assert (a != b)
  let assert_ge = fn a b => assert (a >= b)
  let assert_le = fn a b => assert (a <= b)
  let assert_gt = fn a b => assert (a > b)
  let assert_lt = fn a b => assert (a < b)

  let print = fn a => string::print
  let println = fn a => string::concatenate a "\n"
    |> string::print
end

module integer =
  let abs = fn i => if i < 0 then -i else i
  let pow = fn base with
    | 0 => 1
    | 1 => base
    | n =>
      let b = pow base (n / 2) in
      b * b * (if n % 2 == 0 then 1 else base)
end

module real =
  let abs = fn r => if r < 0.0 then -.r else r
end

module opt =
  type t = fn a => Some of a | None
end

module result =
  type t = fn a b => Ok of a | Error of b
end

module opt =
  let map = fn operation maybe => match maybe with
    | opt::Some of o => (operation >> opt::Some) o
    | _ => opt::None

  let iterate =
    fn operation maybe => (map (fn a => operation a; a) maybe); ()

  let unwrap = fn maybe => match maybe with
    | opt::Some of o => o
    | _ => std::panic ()

  let is_some = fn maybe => match maybe with
    | opt::Some of _ => true
    | _ => false

  let is_none = is_some >> (not)

  let ok_or_error = fn error opt => match opt with
    | opt::None of () => result::Error error
    | opt::Some of a => result::Ok a
end

module result =
  let map = fn operation with
    | result::Ok of o => (operation >> result::Ok) o
    | r => r

  let map_err = fn operation with
    | result::Error of e => (operation >> result::Error) e
    | r => r

  let is_ok = fn a => match a with 
    | result::Ok of _ => true
    | _ => false
  
  let is_err = is_ok >> (not)
  
  let unwrap_ok = fn with 
    | result::Ok of val => val
    | _ => std::panic ()

  let unwrap_err = fn a => match a with
    | result::Error of val => val
    | _ => std::panic ()
  
  let and_also = fn a res => match a with 
    | result::Ok of _ => res
    | _ => a
  
  let expect = fn msg with 
    | result::Ok of val => val
    | result::Error of _ => std::println msg; std::panic ()
  
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
    | _ => opt::None
end

module list =
  type t = fn I => Pair of I * (t I) | Nil

  let map = fn operation list => match list with
    | Pair of (head, tail) => Pair (operation head, tail |> map operation)
    | Nil => Nil

  let iterate =
    fn operation list => (map (fn a => operation a; a) list); ()

  let push = fn item list => match list with
    | Pair of (head, tail) => Pair (head, push item tail)
    | Nil => Pair (item, Nil)

  let length = fn list => match list with
    | Pair of (head, tail) => 1 + (length tail)
    | Nil => 0

  let fold = fn op acc list => match list with
    | Pair of (head, tail) => op acc (fold op head tail)
    | Nil => acc

  let concatenate = fn list1 list2 => match list1 with
    | Pair of (head, tail) => Pair (head, concatenate tail list2)
    | Nil => list2

  let nth = fn n list => match (n, list) with
    | (0, Pair of (head, _)) => opt::Some head
    | (n, Pair of (head, tail)) => nth (n - 1) tail
    | (_, Nil of ()) => opt::None
end

(*
  Converting types to strings
*)
module format =
  let integer = (
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
      | _ => "?" in
    let f = fn with
      | 0 => ""
      | x => if x < 0 then
        x
          |> integer::abs
          |> f
          |> string::concatenate "-"
      else
        (x % 10)
          |> digit_to_string
          |> string::concatenate (f (x / 10)) in
    fn with
      | 0 => "0"
      | n => f n
  )

  let real = fn r => 
    string::concatenate (
      r
      |> integer::from_real
      |> integer
    )
    (
      r -. (real::truncate r)
      |> real::abs
      |> ( *. ) 1000000.0 -- move up 6 decimal places
      |> integer::from_real
      |> integer
      |> string::concatenate "."
    )

    let boolean = fn with
      | true => "true"
      | false => "false"

    let unit = fn (_ : ()) => "()"
end
