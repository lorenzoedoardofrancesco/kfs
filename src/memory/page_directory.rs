use crate::memory::{page_table::PageTable, page_table_entry::PageTableFlags};
use bitflags::bitflags;
use core::arch::asm;

/// Constants defining the page size and the number of entries in a page table.
/// The page size is 4 KiB and the number of entries in a page table is 1024.
pub const PAGE_SIZE: usize = 4096;
pub const ENTRY_COUNT: usize = 1024;

// Constants for memory addresses reserved for paging structures
/// TODO: Make these constants dynamic and in the kernel space (heap???)

const PAGE_DIRECTORY_ADDR: usize = 0x160000;
const PAGE_TABLES_ADDR: usize = 0x161000;

use core::sync::atomic::{AtomicPtr, Ordering};

// Static references to the page directory and tables using AtomicPtr
pub static PAGE_DIRECTORY: AtomicPtr<PageDirectory> =
	AtomicPtr::new(PAGE_DIRECTORY_ADDR as *mut PageDirectory);
pub static PAGE_TABLES: AtomicPtr<[PageTable; ENTRY_COUNT]> =
	AtomicPtr::new(PAGE_TABLES_ADDR as *mut [PageTable; ENTRY_COUNT]);

bitflags! {
	pub struct PageDirectoryFlags: u32 {
		const PRESENT       = 0b1;
		const WRITABLE      = 0b10;
		const USER          = 0b100;
		const PWT           = 0b1000;
		const PCD           = 0b1_0000;
		const ACCESSED      = 0b10_0000;
		const DIRTY         = 0b100_0000;
		const _4MB          = 0b1000_0000;
		const CPU_GLOBAL    = 0b1_0000_0000;
		const LV4_GLOBAL    = 0b10_0000_0000;
		const FRAME         = 0xFFFFF000;
	}
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct PageDirectoryEntry {
	value: u32,
}

impl PageDirectoryEntry {
	pub fn new() -> Self {
		PageDirectoryEntry { value: 0 }
	}

	// Set the frame address
	pub fn set_frame(&mut self, frame: u32) {
		let frame_addr = frame & PageDirectoryFlags::FRAME.bits();
		self.value = (self.value & !PageDirectoryFlags::FRAME.bits()) | frame_addr;
	}

	// Add attribute
	pub fn add_attribute(&mut self, attrib: PageDirectoryFlags) {
		self.value |= attrib.bits();
	}

	// Delete attribute
	pub fn del_attrib(&mut self, attrib: PageDirectoryFlags) {
		self.value &= !attrib.bits();
	}
}

#[repr(C, align(4096))]
pub struct PageDirectory {
	pub entries: [PageDirectoryEntry; ENTRY_COUNT],
}

impl PageDirectory {
	/// Adds or updates a mapping in the page directory.
	pub fn add_entry(&mut self, index: usize, frame: u32, flags: PageDirectoryFlags) {
		let entry = &mut self.entries[index];
		entry.set_frame(frame);
		entry.add_attribute(flags);
	}

	/// Removes an entry from the page directory.
	pub fn remove_entry(&mut self, index: usize) {
		let entry = &mut self.entries[index];
		*entry = PageDirectoryEntry::new();
	}

	/// Gets the index in the entries array for a virtual address.
	pub fn get_index(&self, virtual_address: usize) -> usize {
		(virtual_address >> 22) & 0x3ff
	}
}

pub fn enable_paging() {
	unsafe {
		asm!("mov cr3, {}", in(reg) PAGE_DIRECTORY_ADDR);
		let mut cr0: u32;
		asm!("mov {}, cr0", out(reg) cr0);
		cr0 |= 0x80000000; // Set the PG bit to enable paging
		asm!("mov cr0, {}", in(reg) cr0);
	}
}

pub fn init_pages() {
	unsafe {
		// Convert raw pointers to mutable references
		let directory = &mut *PAGE_DIRECTORY.load(Ordering::Relaxed);
		let tables = &mut *PAGE_TABLES.load(Ordering::Relaxed);

		for (i, table) in tables.iter_mut().enumerate() {
			// Calculate the physical address of this table's entries outside the inner loop
			let table_phys_addr = table.entries.as_ptr() as u32;

			for (j, entry) in table.entries.iter_mut().enumerate() {
				let virt = (i << 22) | (j << 12);
				let phys = virt as u32;
				entry.set_frame_address(phys);
				entry.add_attribute(PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
			}

			// Now use the previously calculated physical address
			directory.add_entry(
				i,
				table_phys_addr,
				PageDirectoryFlags::PRESENT | PageDirectoryFlags::WRITABLE,
			);
		}
	}
	enable_paging();
}
