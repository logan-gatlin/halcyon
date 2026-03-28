bundle temperature

let farenheit_to_celcius =
  fn temp => (temp - 32.0) / 1.8

do readln "Enter a temperature in Farenheit: "
      |> parse
      +> farenheit_to_celcius
      +> `The tempareture in Celcius is: {}`
      |> unwrap_or "Input must be a number"
      |> println
