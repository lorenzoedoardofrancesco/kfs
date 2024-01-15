use crate::memory::physical_memory_managment::PMM;
use bitflags::bitflags;

bitflags! {
	pub struct PageTableFlags: u32 {
		const PRESENT = 0b1;
		const WRITABLE = 0b10;
		const USER = 0b100;
		const PWT = 0b1000;
		const PCD = 0b1_0000;
		const ACCESSED = 0b10_0000;
		const DIRTY = 0b100_0000;
		const PAT = 0b1000_0000;
		const CPU_GLOBAL = 0b1_0000_0000;
		const AVAILABLE = 0b1110_0000_0000;
		const FRAME = 0xFFFFF000;
	}
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct PageTableEntry {
	pub value: u32,
}

impl PageTableEntry {
	/// Creates a new PageTableEntry with zeroed flags
	pub fn new() -> Self {
		PageTableEntry { value: 0 }
	}

	/// Creates a new PageTableEntry from the given frame address and flags
	pub fn set_frame_address(&mut self, frame_address: u32, flags: PageTableFlags) {
		self.value = frame_address | flags.bits();
	}

	/// Sets the flags for this page table entry
	pub fn set_flags(&mut self, flags: PageTableFlags) {
		self.value = (self.value & PageTableFlags::FRAME.bits()) | flags.bits();
	}

	/// Adds the specified attribute flags to this entry.
	pub fn add_attribute(&mut self, attribute: PageTableFlags) {
		self.value |= attribute.bits();
	}

	/// Removes the specified attribute flags from this entry.
	pub fn remove_attribute(&mut self, attribute: PageTableFlags) {
		self.value &= !attribute.bits();
	}

	/// Allocates a new frame for this entry.
	pub fn alloc_new(&mut self) {
		// BIEN VERIFIER QUE CA MARCHE
		let frame = PMM
			.lock()
			.allocate_frame()
			.map_err(|_| "Failed to allocate frame for page table entry");

		self.set_frame_address(
			frame.unwrap(),
			PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
		);
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
	pub fn frame(&self) -> u32 {
		self.value & PageTableFlags::FRAME.bits()
	}
}
