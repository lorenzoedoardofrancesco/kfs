use crate::memory::page_directory::PAGE_SIZE;
use crate::memory::physical_memory_managment::PMM;
use bitflags::bitflags;

bitflags! {
	pub struct PageTableFlags: usize {
		const PRESENT = 0b1;
		const WRITABLE = 0b10;
		const USER = 0b100;
		const WRITETHROUGH = 0b1000;
		const NOT_CACHEABLE = 0b1_0000;
		const ACCESSED = 0b10_0000;
		const DIRTY = 0b100_0000;
		const PAT = 0b1000_0000;
		const CPU_GLOBAL = 0b1_0000_0000;
		const LV4_GLOBAL = 0b10_0000_0000;
		const FRAME = 0x7FFFF000;
	}
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct PageTableEntry {
	pub value: usize,
}

impl PageTableEntry {
	pub fn new() -> Self {
		PageTableEntry { value: 0 }
	}

	pub fn new_from_address(&mut self, address: usize, flags: PageTableFlags) {
		self.value = address | flags.bits();
	}

	pub fn alloc_new() -> Result<Self, &'static str> {
		let frame = PMM
			.lock()
			.allocate_frame()
			.map_err(|_| "Failed to allocate frame for page table entry")?;

		let mut entry = PageTableEntry::new();
		entry.set_frame_address(frame)?;
		Ok(entry)
	}

	/// Adds the specified attribute flags to this entry.
	pub fn add_attribute(&mut self, attribute: PageTableFlags) {
		self.value |= attribute.bits();
	}

	/// Removes the specified attribute flags from this entry.
	pub fn remove_attribute(&mut self, attribute: PageTableFlags) {
		self.value &= !attribute.bits();
	}

	/// Sets the frame address for this entry.
	/// Ensure that the address is correctly aligned and doesn't interfere with flags.
	pub fn set_frame_address(&mut self, frame: usize) -> Result<(), &'static str> {
		if frame % PAGE_SIZE != 0 {
			return Err("Frame address is misaligned");
		}
		let frame_address = frame & PageTableFlags::FRAME.bits();
		self.value = (self.value & !PageTableFlags::FRAME.bits()) | frame_address;
		Ok(())
	}

	/// Returns true if the entry is present in memory.
	pub fn is_present(&self) -> bool {
		self.value & PageTableFlags::PRESENT.bits() != 0
	}

	/// Returns true if the entry is writable.
	pub fn is_writable(&self) -> bool {
		self.value & PageTableFlags::WRITABLE.bits() != 0
	}

	/// Returns the frame address for this entry.
	pub fn frame(&self) -> usize {
		self.value & PageTableFlags::FRAME.bits()
	}

	pub fn is_unused(&self) -> bool {
		true // TODO implement the used/unused
	}
}
