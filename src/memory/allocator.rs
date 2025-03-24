use super::*;

const PTR_T_SIZE: PtrT = size_of::<PtrT>() as PtrT;

impl Memory {
  pub fn static_allocate(&mut self, size_bytes: PtrT) -> PtrT {
    self.new_allocation(size_bytes, true)
  }

  pub fn dynamic_allocate(&mut self, size_bytes: PtrT) -> PtrT {
    self.new_allocation(size_bytes, false)
  }

  fn new_allocation(&mut self, size_bytes: PtrT, is_static: bool) -> PtrT {
    if size_bytes == 0 {
      return 0;
    }
    let flag = if is_static { 0b11 } else { 0b1 };
    let size_needed = PtrT::max(size_bytes.next_multiple_of(4), 4);
    let mut address = 0;
    loop {
      if address > self.current_size * Self::PAGE_SIZE_BYTES {
        self.grow(1);
      }
      let header: PtrT = self.load(address);
      let size = header & !0b11;
      let free = header & 0b11 == 0;
      // End of list
      if header == 0 {
        self.store(address, &(size_needed | flag));
        return address + PTR_T_SIZE;
      } else if free && size > size_needed {
        self.store(address, &(header | flag));
        return address + PTR_T_SIZE;
      } else {
        address += size + PTR_T_SIZE;
      }
    }
  }

  pub fn free_allocation(&mut self, address: PtrT) {
    if address == 0 {
      panic!("Attempt to free null pointer");
    }
    let header_address = address - PTR_T_SIZE;
    let header: PtrT = self.load(header_address);
    self.store(header_address, &(header & !1));
  }

  pub fn allocate_assign(&mut self, datum: impl DynamicMemoryRepr) -> PtrT {
    datum.to_memory_dy(self, false)
  }

  pub fn dynamic_load<T: DynamicMemoryRepr>(&self, address: PtrT) -> T {
    T::from_memory_dy(self, address)
  }

  pub fn used_space(&self) -> PtrT {
    let mut address = 0;
    loop {
      let length: PtrT = self.load(address);
      if length == 0 {
        return address;
      } else {
        address += (length & !0b11) + PTR_T_SIZE;
      }
    }
  }
}

pub trait DynamicMemoryRepr {
  fn from_memory_dy(memory: &Memory, pointer: PtrT) -> Self;
  fn to_memory_dy(&self, memory: &mut Memory, is_static: bool) -> PtrT;
}

impl<T: StaticMemoryRepr> DynamicMemoryRepr for T
where
  [(); T::S]:,
{
  fn from_memory_dy(memory: &Memory, pointer: PtrT) -> Self {
    let mut bytes = [0; T::S];
    memory
      .bytes_at(pointer, T::S as PtrT)
      .iter()
      .zip(bytes.iter_mut())
      .for_each(|(a, b)| {
        *b = *a;
      });
    T::from_memory(bytes)
  }

  fn to_memory_dy(&self, memory: &mut Memory, is_static: bool) -> PtrT {
    let ptr = memory.new_allocation(T::S as PtrT, is_static);
    memory.store(ptr, self);
    ptr
  }
}
