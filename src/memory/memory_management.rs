use bitflags::bitflags;

/// Constants defining the page size and the number of entries in a page table.
/// The page size is 4 KiB and the number of entries in a page table is 1024.
const PAGE_SIZE: usize = 4096;
const ENTRY_COUNT: usize = 1024;

pub static mut PAGE_TABLE_START: usize = 0;
pub static mut PAGE_TABLE_END: usize = 0;

bitflags! {
	/// Flags for page table entries. These flags represent the following:
	/// - PRESENT: The page is currently in memory.
	/// - READ_WRITE: If set, the page is writable.
	/// - USER_SUPERVISOR: If set, the page is accessible from user mode.
	/// - WRITE_THROUGH: If set, write-through caching is enabled for the page.
	/// - CACHE_DISABLE: If set, caching is disabled for the page.
	/// - ACCESSED: If set, the page has been accessed since the last refresh.
	/// - DIRTY: If set, the page has been written to since the last refresh.
	/// - PAGE_SIZE: If set, the page is 4 MiB in size. Otherwise, it is 4 KiB.
	/// - GLOBAL: If set, the page is global.
	/// - AVAILABLE: Bits available for use by the OS.
	pub struct PageTableFlags: u32 {
		const PRESENT = 1 << 0;
		const READ_WRITE = 1 << 1;
		const USER_SUPERVISOR = 1 << 2;
		const WRITE_THROUGH = 1 << 3;
		const CACHE_DISABLE = 1 << 4;
		const ACCESSED = 1 << 5;
		const DIRTY = 1 << 6;
		const PAGE_SIZE = 1 << 7;
		const GLOBAL = 1 << 8;
		const AVAILABLE = 1 << 9 | 1 << 10 | 1 << 11;
	}
}

#[derive(Debug, Clone, Copy)]
enum PageTableError {
	NotMapped,
	AlreadyMapped,
	AllocationFailed,
	InvalidAddress,
}

#[derive(Debug, Clone, Copy)]
pub enum FrameError {
	AddressNotAligned,
	AddressOverflow,
	AddressUnderflow,
}

#[repr(C, align(4096))]
struct PageDirectory {
	entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageDirectory {
	pub fn new() -> Self {
		PageDirectory {
			entries: [PageTableEntry::new(); ENTRY_COUNT],
		}
	}

	/// Adds or updates a mapping in the page directory.
	/// If the page table for the virtual address does not exist, it is created.
	pub fn add_update_mapping(
		&mut self,
		virtual_address: usize,
		frame: Frame,
		flags: PageTableFlags,
	) -> Result<(), PageTableError> {
		let index = self.get_index(virtual_address);
		let entry = &mut self.entries[index];

		if !entry.is_set() {
			// Allocate a new page table if not already present
		//	let new_table = match KernelAllocator.kmalloc() as *mut PageTable {
		//		ptr if !ptr.is_null() => ptr,
		//		_ => return Err(PageTableError::AllocationFailed),
		//	};
		//	entry.set(
		//		Frame::containing_address(new_table as usize),
		//		flags | PageTableFlags::PRESENT,
		//	);
		}

		let table_address = entry.start_address();
		if entry.is_valid_page_table_address() {
			unsafe {
				let table = &mut *(table_address as *mut PageTable);
				table.add_mapping(virtual_address, frame, flags);
				Ok(())
			}
		} else {
			Err(PageTableError::InvalidAddress)
		}
	}

	/// Removes a mapping from the page directory.
	pub fn remove_mapping(&mut self, virtual_address: usize) -> Result<(), PageTableError> {
		let index = self.get_index(virtual_address);
		let table = self.get_table_mut(index).ok_or(PageTableError::NotMapped)?;

		if table.is_mapped(virtual_address) {
			table.remove_mapping(virtual_address);
			Ok(())
		} else {
			Err(PageTableError::NotMapped)
		}
	}

	/// Checks if a virtual address is mapped in the page directory.
	pub fn is_mapped(&self, virtual_address: usize) -> bool {
		let index = self.get_index(virtual_address);
		self.get_table(index)
			.map_or(false, |table| table.is_mapped(virtual_address))
	}

	/// Translates a virtual address to a physical address, if mapped.
	pub fn translate(&self, virtual_address: usize) -> Result<usize, PageTableError> {
		let index = self.get_index(virtual_address);
		match self.get_table(index) {
			Some(table) => table
				.translate(virtual_address)
				.ok_or(PageTableError::NotMapped),
			None => Err(PageTableError::NotMapped),
		}
	}

	/// Gets the index in the page directory for a virtual address.
	fn get_index(&self, virtual_address: usize) -> usize {
		(virtual_address >> 22) & 0x3ff
	}

	/// Gets a reference to the page table for a virtual address.
	fn get_table(&self, index: usize) -> Option<&PageTable> {
		let entry = &self.entries[index];
		if entry.is_set() {
			let addr: usize = entry.start_address();
			if entry.is_valid_page_table_address() {
				unsafe { Some(&*(addr as *const PageTable)) }
			} else {
				None
			}
		} else {
			None
		}
	}

	/// Gets a mutable reference to the page table for a virtual address.
	fn get_table_mut(&mut self, index: usize) -> Option<&mut PageTable> {
		let entry = &self.entries[index];
		if entry.is_set() {
			let addr = entry.start_address();
			if entry.is_valid_page_table_address() {
				unsafe { Some(&mut *(addr as *mut PageTable)) }
			} else {
				None
			}
		} else {
			None
		}
	}
}

#[repr(C, align(4096))]
struct PageTable {
	entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageTable {
	/// Creates a new, empty page table.
	fn new() -> Self {
		PageTable {
			entries: [PageTableEntry::new(); ENTRY_COUNT],
		}
	}

	/// Adds a mapping from the virtual page to a physical frame.
	pub fn add_mapping(&mut self, virtual_address: usize, frame: Frame, flags: PageTableFlags) {
		let index = self.get_index(virtual_address);
		self.entries[index].set(frame, flags | PageTableFlags::PRESENT);
	}

	/// Removes a mapping for a virtual address.
	pub fn remove_mapping(&mut self, virtual_address: usize) {
		let index = self.get_index(virtual_address);
		self.entries[index].clear();
	}

	/// Checks if there is a mapping for a virtual address.
	pub fn is_mapped(&self, virtual_address: usize) -> bool {
		let index = self.get_index(virtual_address);
		self.entries[index].is_set()
	}

	/// Translates a virtual address to a physical address, if mapped.
	pub fn translate(&self, virtual_address: usize) -> Option<usize> {
		let index = self.get_index(virtual_address);
		if self.entries[index].is_set() {
			let frame_address = self.entries[index].start_address();
			Some(frame_address | (virtual_address & 0xfff))
		} else {
			None
		}
	}

	/// Gets the index in the entries array for a virtual address.
	fn get_index(&self, virtual_address: usize) -> usize {
		(virtual_address >> 12) & 0x3ff
	}
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct PageTableEntry(u32);

impl PageTableEntry {
	fn new() -> Self {
		PageTableEntry(0)
	}

	pub fn set(&mut self, frame: Frame, flags: PageTableFlags) {
		assert!(frame.start_address() % PAGE_SIZE == 0);
		self.0 = (frame.start_address() as u32) | flags.bits();
	}

	fn is_set(&self) -> bool {
		self.0 != 0
	}

	pub fn clear(&mut self) {
		self.0 = 0;
	}

	fn start_address(&self) -> usize {
		(self.0 & 0xfffff000) as usize
	}

	pub fn is_valid_page_table_address(&self) -> bool {
		let address = self.start_address();

		unsafe {
			// Check if the address is within the valid memory range.
			let is_within_range = address >= PAGE_TABLE_START && address <= PAGE_TABLE_END;

			// Check if the address is page aligned.
			let is_page_aligned = address % PAGE_SIZE == 0;

			is_within_range && is_page_aligned
		}
	}
}

/*

pub unsafe fn enable_paging() {
	use core::arch::asm;

	let page_directory = page_directory as *const PageDirectory as u32;
	asm!("mov eax, $0; mov cr3, eax; mov eax, cr0; or eax, 0x80000000; mov cr0, eax" :: "r"(page_directory) : "eax" : "intel", "volatile");
}

*/

struct Frame {
	start_address: usize,
}

impl Frame {
	/// Creates a new Frame from a start address.
	/// The address must be aligned to the page boundary.
	pub fn new(start_address: usize) -> Result<Self, FrameError> {
		if start_address % PAGE_SIZE != 0 {
			Err(FrameError::AddressNotAligned)
		} else {
			Ok(Frame { start_address })
		}
	}

	/// Return the start address of the frame.
	pub fn start_address(&self) -> usize {
		self.start_address
	}

	/// Returns the frame containing the given address.
	pub fn containing_address(address: usize) -> Self {
		let aligned_address = address & !(PAGE_SIZE - 1);
		Frame::new(aligned_address).unwrap()
	}

	/// Returns the next frame.
	pub fn next(&self) -> Result<Self, FrameError> {
		if self.start_address > usize::MAX - PAGE_SIZE {
			Err(FrameError::AddressOverflow)
		} else {
			Ok(Frame::new(self.start_address + PAGE_SIZE).unwrap())
		}
	}

	/// Returns the previous frame.
	pub fn previous(&self) -> Result<Self, FrameError> {
		if self.start_address < PAGE_SIZE {
			Err(FrameError::AddressUnderflow)
		} else {
			Ok(Frame::new(self.start_address - PAGE_SIZE).unwrap())
		}
	}

	/// Returns the frame number.
	pub fn number(&self) -> usize {
		self.start_address / PAGE_SIZE
	}
}
