use crate::memory::{
	page_directory_entry::{PageDirectoryEntry, PageDirectoryFlags},
	page_table::PageTable,
};
use core::arch::asm;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicPtr, Ordering};

/// Constants defining the page size and the number of entries in a page table.
/// The page size is 4 KiB and the number of entries in a page table is 1024.
pub const PAGE_SIZE: usize = 4096;
pub const ENTRY_COUNT: usize = 1024;
pub const PAGE_TABLE_SIZE: usize = ENTRY_COUNT * PAGE_SIZE;

// Constants for memory addresses reserved for paging structures
pub static mut PAGE_DIRECTORY_ADDR: u32 = 0;
pub static mut PAGE_TABLES_ADDR: u32 = 0;

// Static references to the page directory and tables using AtomicPtr
pub static mut PAGE_DIRECTORY: AtomicPtr<PageDirectory> = AtomicPtr::new(null_mut());
pub static mut PAGE_TABLES: AtomicPtr<[PageTable; ENTRY_COUNT]> = AtomicPtr::new(null_mut());

#[repr(C, align(4096))]
pub struct PageDirectory {
	pub entries: [PageDirectoryEntry; ENTRY_COUNT],
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

pub unsafe fn init_page_directory() {
	PAGE_DIRECTORY = AtomicPtr::new(PAGE_DIRECTORY_ADDR as *mut PageDirectory);
	let page_directory = &mut *PAGE_DIRECTORY.load(Ordering::Relaxed);

	// Initialize all directory entries
	let mut current_page_table = PAGE_TABLES_ADDR;
	for page_directory_entry in page_directory.entries.iter_mut().enumerate() {
		if page_directory_entry.0 <= 768 {
			page_directory_entry.1.set(
				current_page_table,
				PageDirectoryFlags::PRESENT
					| PageDirectoryFlags::WRITABLE
					| PageDirectoryFlags::USER,
			);
		} else {
			page_directory_entry.1.set(
				current_page_table,
				PageDirectoryFlags::PRESENT | PageDirectoryFlags::WRITABLE,
			);
		}
		current_page_table += PAGE_TABLE_SIZE as u32;
	}
}
