use crate::memory::{
	page_directory_entry::{PageDirectoryEntry, PageDirectoryFlags},
	page_table::PageTable,
	page_table_entry::PageTableFlags,
	physical_memory_managment::HIGH_KERNEL_OFFSET,
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

impl PageDirectory {
	pub fn get_page_table(&mut self, virtual_address: u32) -> &mut PageTable {
		let index = (virtual_address >> 22) as usize;
		self.entries[index].get_page_table()
	}
}
pub fn enable_paging() {
	println_serial!("Enabling paging...");
	let page_directory_addr = unsafe { PAGE_DIRECTORY_ADDR - HIGH_KERNEL_OFFSET };
	println_serial!("Page Directory address: {:x}", page_directory_addr);
	unsafe {
		let pd_addr: u32 = 1470464; // Your page directory address
		asm!("mov cr3, {}", in(reg) pd_addr);
		asm!("mov cr3, {}", in(reg) pd_addr);
	}

	print_serial!("Mapping page tables...");
	unsafe {
		let mut cr0: u32;
		asm!("mov {}, cr0", out(reg) cr0);
		cr0 |= 0x80000000; // Set the PG bit to enable paging
		asm!("mov cr0, {}", in(reg) cr0);
	}
	println_serial!("Paging enabled!");
}

pub unsafe fn init_page_directory() {
	PAGE_DIRECTORY = AtomicPtr::new(PAGE_DIRECTORY_ADDR as *mut PageDirectory);
	let page_directory = &mut *PAGE_DIRECTORY.load(Ordering::Relaxed);
	println_serial!("Page Directory address: {:p}", page_directory);

	// Initialize all directory entries and page tables
	let mut current_page_table = PAGE_TABLES_ADDR;
	for page_directory_entry in page_directory.entries.iter_mut().enumerate() {
		if page_directory_entry.0 < 768 {
			page_directory_entry.1.set(
				current_page_table,
				PageDirectoryFlags::PRESENT
					| PageDirectoryFlags::WRITABLE
					| PageDirectoryFlags::USER,
			);
			page_directory_entry
				.1
				.get_page_table()
				.new(PageTableFlags::USER | PageTableFlags::WRITABLE);
		} else {
			page_directory_entry.1.set(
				current_page_table,
				PageDirectoryFlags::PRESENT | PageDirectoryFlags::WRITABLE,
			);
			page_directory_entry
				.1
				.get_page_table()
				.new(PageTableFlags::WRITABLE);
		}
		current_page_table += PAGE_SIZE as u32;
	}

	// Map the kernel to the first 8 MiB of physical memory
	page_directory
		.get_page_table(HIGH_KERNEL_OFFSET)
		.kernel_mapping(
			0x00000000,
			PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
		);
	page_directory
		.get_page_table(HIGH_KERNEL_OFFSET + PAGE_TABLE_SIZE as u32)
		.kernel_mapping(
			0x00000000 + PAGE_TABLE_SIZE as u32,
			PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
		);
	//let page_tables = &mut *PAGE_TABLES.load(Ordering::Relaxed);
}
