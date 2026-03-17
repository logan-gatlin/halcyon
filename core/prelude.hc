(*
  Everything in the core::prelude module is implicitly in scope always. Do
  not write actual implementations in here, instead write aliases to things
  defined elsewhere.
*)
module prelude =

  -- Types --
  
  type ~Integer = core::Integer
  type ~Real = core::Real
  type ~Boolean = core::Boolean
  type ~Unit = core::Unit
  type ~Glyph = core::Glyph
  type ~String = core::String
  type ~Array: a = core::Array a
  type ~Option: a = core::opt::Option a
  type ~Result: err ok = core::result::Result err ok

  -- Traits --

  trait ~Default = core::Default
  trait ~Applicative = core::hkt::Applicative
  trait ~Monad = core::hkt::Monad
  trait ~Traversable = core::hkt::Traversable
  trait ~Foldable = core::hkt::Foldable
  trait ~Alternative = core::hkt::Alternative
  trait ~Comonad = core::hkt::Comonad
  trait ~Functor = core::hkt::Functor
  trait ~Bifunctor = core::hkt::Bifunctor
  trait ~Zip = core::hkt::Zip
  trait ~Filterable = core::hkt::Filterable

  -- Constructors --

  let | Some = core::opt::Some
  let | None = core::opt::None
  let | Ok = core::result::Ok
  let | Err = core::result::Err

  -- Operators --

  let [+] = core::ops::[+]
  let [-] = core::ops::[-]
  let [~] = core::ops::[~]
  let [*] = core::ops::[*]
  let [/] = core::ops::[/]
  let [%] = core::ops::[%]
  let [|>] = core::ops::[|>]
  let [>>] = core::ops::[>>]
  let [<<] = core::ops::[<<]
  let [and] = core::ops::[and]
  let [or] = core::ops::[or]
  let [xor] = core::ops::[xor]
  let [not] = core::ops::[not]
  let [==] = core::ops::[==]
  let [!=] = core::ops::[!=]
  let [<] = core::ops::[<]
  let [>] = core::ops::[>]
  let [<=] = core::ops::[<=]
  let [>=] = core::ops::[>=]

  -- Terms --

  let min = core::ops::min
  let max = core::ops::max
  let clamp = core::ops::clamp
  let between = core::ops::between
  let read = core::io::read
  let print = core::io::print
  let println = core::io::println
  let readln = core::io::readln
  let eprint = core::io::eprint
  let eprintln = core::io::eprintln
  let array_push = core::array::push
  let option_map = core::opt::map
  let result_map = core::result::map
  let result_map_err = core::result::map_err
  let assert = core::test::assert
  let panic = core::test::panic
  let default = core::default
  let map = core::hkt::map
  let flatten = core::hkt::flatten
  let fold = core::hkt::fold
  let filter = core::hkt::filter
  let any = core::hkt::any
  let all = core::hkt::all
end
