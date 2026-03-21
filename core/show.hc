module show =
  use core

  trait Show: self =
    let show : self -> String
  end
end
