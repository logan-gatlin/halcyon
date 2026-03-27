(*>
Provides a canonical fallback value for a type.

Use `Default` when you need a neutral starting value (for example, in
configuration records, accumulators, or error fallbacks).

```hc
let fallback = default
```
*)
trait Default: self =
  let default : self
end
