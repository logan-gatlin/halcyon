pub mod allocator;

pub use allocator::*;

pub type PtrT = u32;

#[derive(Clone)]
pub struct Memory {
  buffer: Vec<u8>,
  current_size: PtrT,
  max_size: PtrT,
}

impl std::fmt::Debug for Memory {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "")
  }
}

impl Memory {
  pub const MAX_PAGES: PtrT = 65536;
  pub const PAGE_SIZE_BYTES: PtrT = 64_000;

  pub fn new(start_size: PtrT, max_size: PtrT) -> Self {
    if max_size > Self::MAX_PAGES {
      panic!("Tried to allocate too large of a memory");
    }
    if start_size > max_size {
      panic!("Minimum memory size is greater than maximum memory size");
    }
    Self {
      buffer: vec![0; (start_size * Self::PAGE_SIZE_BYTES) as usize],
      current_size: start_size,
      max_size,
    }
  }

  pub fn to_buffer(self) -> Vec<u8> {
    let length = self.used_space();
    self.buffer[0..length as usize].into()
  }

  pub fn bytes_at(&self, index: PtrT, length: PtrT) -> &[u8] {
    &self.buffer[(index as usize)..(index as usize + length as usize)]
  }

  pub fn load<T: StaticMemoryRepr>(&self, offset: PtrT) -> T
  where
    [(); T::S]:,
  {
    let mut bytes = [0_u8; T::S];
    for i in 0..T::S {
      bytes[i] = self.buffer[i + (offset as usize)];
    }
    T::from_memory(bytes)
  }

  pub fn store<T: StaticMemoryRepr>(&mut self, offset: PtrT, datum: &T)
  where
    [(); T::S]:,
  {
    let bytes = datum.to_memory();
    for i in 0..T::S {
      self.buffer[i + (offset as usize)] = bytes[i];
    }
  }

  pub fn size(&self) -> PtrT {
    self.current_size
  }

  pub fn grow(&mut self, amount: PtrT) -> PtrT {
    let new_size = self.current_size.saturating_add(amount);
    if new_size > self.max_size {
      return PtrT::MAX;
    }
    let mut new_buffer = vec![0; (new_size * Self::PAGE_SIZE_BYTES) as usize];
    for i in 0..self.buffer.len() {
      new_buffer[i] = self.buffer[i];
    }
    self.buffer = new_buffer;
    new_size
  }
}

pub trait StaticMemoryRepr: Sized {
  const S: usize = size_of::<Self>();

  fn to_memory(&self) -> [u8; Self::S]
  where
    [(); Self::S]:;
  fn from_memory(bytes: [u8; Self::S]) -> Self
  where
    [(); Self::S]:;
}

macro_rules! impl_trait {
  ($($i:ident),+) => {
    $(impl StaticMemoryRepr for $i {
      fn to_memory(&self) -> [u8; Self::S] where [u8; Self::S]: {
        self.to_le_bytes()
      }

      fn from_memory(bytes: [u8; Self::S]) -> Self where [u8; Self::S]: {
        Self::from_le_bytes(bytes)
      }
    })*
  };
}

impl_trait!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
