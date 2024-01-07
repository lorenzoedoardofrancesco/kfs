use crate::memory::{
	page_directory::{ ENTRY_COUNT, PAGE_SIZE },
	page_table_entry::{PageTableEntry, PageTableFlags},
};

#[derive(Clone, Copy)]
#[repr(C, align(4096))]
pub struct PageTable {
	pub entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageTable {
	/// Creates a new, empty page table.
	pub fn new() -> Self {
		PageTable {
			entries: [PageTableEntry::new(); ENTRY_COUNT],
		}
	}

	/// Maps the virtual address to the specified frame with the given attributes.
	pub fn map(&mut self, virt_addr: usize, frame: u32, flags: PageTableFlags) {
		let index = Self::virt_to_index(virt_addr);
		let mut entry = self.entries[index];
		entry.set_frame(frame);
		entry.add_attribute(flags | PageTableFlags::PRESENT);
		self.entries[index] = entry;
	}

	/// Unmaps the virtual address.
	pub fn unmap(&mut self, virt_addr: usize) {
		let index = Self::virt_to_index(virt_addr);
		let mut entry = self.entries[index];
		entry.remove_attribute(PageTableFlags::PRESENT);
		self.entries[index] = entry;
	}

	/// Translates a virtual address to the index of the entry in the table.
	fn virt_to_index(virt_addr: usize) -> usize {
		virt_addr / PAGE_SIZE % ENTRY_COUNT
	}

	/// Translates a virtual address to its corresponding physical address.
	/// Returns None if the entry is not present.
	pub fn translate(&self, virt_addr: usize) -> Option<u32> {
		let index = Self::virt_to_index(virt_addr);
		let entry = self.entries[index];
		if entry.is_present() {
			Some(entry.frame() | (virt_addr as u32 & (PAGE_SIZE as u32 - 1)))
		} else {
			None
		}
	}

	/// Returns a mutable reference to the PageTableEntry at the given virtual address.
	pub fn entry_mut(&mut self, virt_addr: usize) -> &mut PageTableEntry {
		let index = Self::virt_to_index(virt_addr);
		&mut self.entries[index]
	}
}
