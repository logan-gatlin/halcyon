(*
  Everything in the core::prelude module is implicitly in scope always. Do
  not write actual implementations in here, instead write aliases to things
  defined elsewhere.
*)
module prelude =

  -- Types --
  
  --> @HIDDEN
  type ~Integer = bundle::Integer
  --> @HIDDEN
  type ~Natural = bundle::Natural
  --> @HIDDEN
  type ~Real = bundle::Real
  --> @HIDDEN
  type ~Boolean = bundle::Boolean
  --> @HIDDEN
  type ~Unit = bundle::Unit
  --> @HIDDEN
  type ~Glyph = bundle::Glyph
  --> @HIDDEN
  type ~String = bundle::String
  --> @HIDDEN
  type ~Array: a = bundle::Array a
  --> @HIDDEN
  type ~Option: a = bundle::opt::Option a
  --> @HIDDEN
  type ~Result: err ok = bundle::result::Result err ok

  -- Traits --

  --> @HIDDEN
  trait ~Default = core::Default
  --> @HIDDEN
  trait ~Show = core::show::Show
  --> @HIDDEN
  trait ~Parse = core::parse::Parse
  --> @HIDDEN
  trait ~Append = core::append::Append
  --> @HIDDEN
  trait ~Applicative = core::hkt::Applicative
  --> @HIDDEN
  trait ~Monad = core::hkt::Monad
  --> @HIDDEN
  trait ~Traversable = core::hkt::Traversable
  --> @HIDDEN
  trait ~Foldable = core::hkt::Foldable
  --> @HIDDEN
  trait ~Alternative = core::hkt::Alternative
  --> @HIDDEN
  trait ~Comonad = core::hkt::Comonad
  --> @HIDDEN
  trait ~Functor = core::hkt::Functor
  --> @HIDDEN
  trait ~Bifunctor = core::hkt::Bifunctor
  --> @HIDDEN
  trait ~Zip = core::hkt::Zip
  --> @HIDDEN
  trait ~Filterable = core::hkt::Filterable

  -- Constructors --

  --> @HIDDEN
  let | Some = core::opt::Some
  --> @HIDDEN
  let | None = core::opt::None
  --> @HIDDEN
  let | Ok = core::result::Ok
  --> @HIDDEN
  let | Err = core::result::Err

  -- Operators --

  --> @HIDDEN
  let [+] = bundle::ops::[+]
  --> @HIDDEN
  let [-] = bundle::ops::[-]
  --> @HIDDEN
  let [~] = bundle::ops::[~]
  --> @HIDDEN
  let [*] = bundle::ops::[*]
  --> @HIDDEN
  let [/] = bundle::ops::[/]
  --> @HIDDEN
  let [mod] = bundle::ops::[mod]
  --> @HIDDEN
  let [|>] = bundle::ops::[|>]
  --> @HIDDEN
  let [>>] = bundle::ops::[>>]
  --> @HIDDEN
  let [<<] = bundle::ops::[<<]
  --> @HIDDEN
  let [and] = bundle::ops::[and]
  --> @HIDDEN
  let [or] = bundle::ops::[or]
  --> @HIDDEN
  let [xor] = bundle::ops::[xor]
  --> @HIDDEN
  let [not] = bundle::ops::[not]
  --> @HIDDEN
  let [==] = bundle::ops::[==]
  --> @HIDDEN
  let [!=] = bundle::ops::[!=]
  --> @HIDDEN
  let [<] = bundle::ops::[<]
  --> @HIDDEN
  let [>] = bundle::ops::[>]
  --> @HIDDEN
  let [<=] = bundle::ops::[<=]
  --> @HIDDEN
  let [>=] = bundle::ops::[>=]
  --> @HIDDEN
  let [+>] = bundle::hkt::[+>]
  --> @HIDDEN
  let [*>] = bundle::hkt::[*>]

  -- Terms --

  --> @HIDDEN
  let min = bundle::ops::min
  --> @HIDDEN
  let max = bundle::ops::max
  --> @HIDDEN
  let clamp = bundle::ops::clamp
  --> @HIDDEN
  let between = bundle::ops::between
  --> @HIDDEN
  let read = bundle::io::read
  --> @HIDDEN
  let print = bundle::io::print
  --> @HIDDEN
  let println = bundle::io::println
  --> @HIDDEN
  let readln = bundle::io::readln
  --> @HIDDEN
  let eprint = bundle::io::eprint
  --> @HIDDEN
  let eprintln = bundle::io::eprintln
  --> @HIDDEN
  let array_push = bundle::array::push
  --> @HIDDEN
  let option_map = bundle::opt::map
  --> @HIDDEN
  let result_map = bundle::result::map
  --> @HIDDEN
  let result_map_err = bundle::result::map_err
  --> @HIDDEN
  let assert = bundle::test::assert
  --> @HIDDEN
  let panic = bundle::test::panic
  --> @HIDDEN
  let default = bundle::default
  --> @HIDDEN
  let show = bundle::show::show
  --> @HIDDEN
  let parse = bundle::parse::parse
  --> @HIDDEN
  let append = bundle::append::append
  --> @HIDDEN
  let prepend = bundle::append::prepend
  --> @HIDDEN
  let map = bundle::hkt::map
  --> @HIDDEN
  let flatten = bundle::hkt::flatten
  --> @HIDDEN
  let fold = bundle::hkt::fold
  --> @HIDDEN
  let filter = bundle::hkt::filter
  --> @HIDDEN
  let any = bundle::hkt::any
  --> @HIDDEN
  let all = bundle::hkt::all
  --> @HIDDEN
  let unwrap_or_else = bundle::unwrap::unwrap_or_else
  --> @HIDDEN
  let unwrap_or = bundle::unwrap::unwrap_or
  --> @HIDDEN
  let unwrap = bundle::unwrap::unwrap
end
