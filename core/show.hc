module show =
  use bundle

  trait Show: self =
    let show : self -> String
  end
end
