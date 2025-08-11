module std =
  import builtin

  // Primitives
  type integer = builtin:integer
  type real = builtin:real
  type string = builtin:string
  type unit = builtin:unit
  type glyph = builtin:glyph
  type boolean = builtin:boolean
  
  let panic = fn => builtin:panic ()
  let assert = fn condition => if condition then () else panic ()
  let print_string = fn s => builtin:print_string s
end

module string =
  import builtin
  let length = fn s => builtin:string_length s
  let concatenate = fn s1 s2 => builtin:string_concatenate s1 s2
  let print = fn s => builtin:print_string s
end

module opt =
  import std
  type t = fn T => Some of T | None of std:unit

  let map = fn operation maybe => match maybe with
    | Some of o => Some (operation o)
    | _ => None ()

  let iterate =
    fn operation maybe => (map (fn a => operation a; a) maybe); ()

  let unwrap = fn maybe => match maybe with
    | Some of o => o
    | _ => std:panic ()

  let is_some = fn maybe => match maybe with
    | Some of _ => true
    | _ => false

  let is_none = fn maybe => not (is_some maybe)
end

module list =
  import std
  import opt

  type t = fn I => Pair of I * t | Nil of std:unit

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
    | (0, Pair of (head, _)) => opt:Some head
    | (n, Pair of (head, tail)) => nth (n - 1) tail
    | (_, Nil of ()) => opt:None ()
end

