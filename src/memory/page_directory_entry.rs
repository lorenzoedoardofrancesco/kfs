use crate::memory::page_table::PageTable;
use bitflags::bitflags;

bitflags! {
	pub struct PageDirectoryFlags: u32 {
		const PRESENT       = 0b1;
		const WRITABLE      = 0b10;
		const USER          = 0b100;
		const PWT           = 0b1000;
		const PCD           = 0b1_0000;
		const ACCESSED      = 0b10_0000;
		const AVAILABLE     = 0b1111_0100_0000;
		const _4MB          = 0b1000_0000;
		const PAGE_TABLE    = 0xFFFFF000;
	}
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct PageDirectoryEntry {
	value: u32,
}

impl PageDirectoryEntry {
	// Sets up a PageTable for this directory entry
	pub fn set(&mut self, page_table: u32, flags: PageDirectoryFlags) {
		self.value = page_table | flags.bits();
	}

	// Sets the flags for this directory entry
	pub fn set_flags(&mut self, flags: PageDirectoryFlags) {
		self.value = (self.value & PageDirectoryFlags::PAGE_TABLE.bits()) | flags.bits();
	}

	// Get the page table for this directory entry
	pub fn get_page_table(&self) -> &mut PageTable {
		let table_address = self.value & PageDirectoryFlags::PAGE_TABLE.bits();
		unsafe { &mut *(table_address as *mut PageTable) }
	}

	// Gets the flags from this directory entry
	pub fn get_flags(&self) -> PageDirectoryFlags {
		let flags = self.value & !PageDirectoryFlags::PAGE_TABLE.bits();
		PageDirectoryFlags::from_bits_truncate(flags)
	}
}
