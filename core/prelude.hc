(*
  Everything in the core::prelude module is implicitly in scope always. Do
  not write actual implementations in here, instead write aliases to things
  defined elsewhere.
*)
module prelude =

  -- Types --
  
  type ~Integer = bundle::Integer
  type ~Real = bundle::Real
  type ~Boolean = bundle::Boolean
  type ~Unit = bundle::Unit
  type ~Glyph = bundle::Glyph
  type ~String = bundle::String
  type ~Natural = bundle::big-num::Natural
  type ~BigInteger = bundle::big-num::BigInteger
  type ~Array: a = bundle::Array a
  type ~Option: a = bundle::opt::Option a
  type ~Result: err ok = bundle::result::Result err ok

  -- Traits --

  trait ~Default = core::Default
  trait ~Show = core::show::Show
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

  let [+] = bundle::ops::[+]
  let [-] = bundle::ops::[-]
  let [~] = bundle::ops::[~]
  let [*] = bundle::ops::[*]
  let [/] = bundle::ops::[/]
  let [%] = bundle::ops::[%]
  let [|>] = bundle::ops::[|>]
  let [>>] = bundle::ops::[>>]
  let [<<] = bundle::ops::[<<]
  let [and] = bundle::ops::[and]
  let [or] = bundle::ops::[or]
  let [xor] = bundle::ops::[xor]
  let [not] = bundle::ops::[not]
  let [==] = bundle::ops::[==]
  let [!=] = bundle::ops::[!=]
  let [<] = bundle::ops::[<]
  let [>] = bundle::ops::[>]
  let [<=] = bundle::ops::[<=]
  let [>=] = bundle::ops::[>=]

  -- Terms --

  let min = bundle::ops::min
  let max = bundle::ops::max
  let clamp = bundle::ops::clamp
  let between = bundle::ops::between
  let read = bundle::io::read
  let print = bundle::io::print
  let println = bundle::io::println
  let readln = bundle::io::readln
  let eprint = bundle::io::eprint
  let eprintln = bundle::io::eprintln
  let array_push = bundle::array::push
  let option_map = bundle::opt::map
  let result_map = bundle::result::map
  let result_map_err = bundle::result::map_err
  let assert = bundle::test::assert
  let panic = bundle::test::panic
  let default = bundle::default
  let show = bundle::show::show
  let map = bundle::hkt::map
  let flatten = bundle::hkt::flatten
  let fold = bundle::hkt::fold
  let filter = bundle::hkt::filter
  let any = bundle::hkt::any
  let all = bundle::hkt::all
end
