pub struct Window<const N: usize, T, I>
where
  I: Iterator<Item = T>,
{
  iterator: I,
  buffer: [Option<T>; N],
  exhausted: bool,
  pub finished: bool,
}

impl<const N: usize, T, I> std::fmt::Debug for Window<N, T, I>
where
  I: Iterator<Item = T>,
  T: std::fmt::Debug,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self.buffer)
  }
}

impl<const N: usize, T, I> Window<N, T, I>
where
  I: Iterator<Item = T>,
{
  pub fn new(mut it: I) -> Self {
    assert!(N > 0, "Lookahead buffer cannot be 0 sized");
    let mut s = Self {
      buffer: std::array::from_fn(|_| it.next()),
      iterator: it,
      exhausted: false,
      finished: false,
    };
    s.normalize();
    s
  }

  fn normalize(&mut self) {
    for item in &mut self.buffer {
      if self.exhausted {
        *item = None;
      } else if let None = item {
        self.exhausted = true;
      }
    }
  }

  pub fn peek(&self, n: usize) -> &Option<T> {
    debug_assert!(n < N, "Peeked further than buffer allows");
    &self.buffer[n]
  }

  fn _advance(&mut self) {
    for i in 1..N {
      self.buffer[i - 1] = self.buffer[i].take();
    }
    self.buffer[N - 1] = match self.iterator.next() {
      Some(i) if !self.exhausted => Some(i),
      _ => {
        self.exhausted = true;
        None
      },
    };
  }
}

impl<const N: usize, T, I> Iterator for Window<N, T, I>
where
  I: Iterator<Item = T>,
{
  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    let r = self.buffer[0].take();
    if let None = r {
      self.finished = true;
    }
    self._advance();
    r
  }
}
