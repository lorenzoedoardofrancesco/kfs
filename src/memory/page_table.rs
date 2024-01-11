use super::physical_memory_managment::physical_address_is_valid;
use crate::{
	memory::{
		page_directory::{ENTRY_COUNT, PAGE_SIZE},
		page_table_entry::{PageTableEntry, PageTableFlags},
		physical_memory_managment::PMM,
	},
	utils::debug::LogLevel,
};

#[derive(Clone, Copy)]
#[repr(C, align(4096))]
pub struct PageTable {
	pub entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageTable {
	/// Creates a new, empty page table with default entries.
	pub fn new(physical_address: u32, flags: PageTableFlags) -> Self {
		PageTable {
			entries: [PageTableEntry::new(); ENTRY_COUNT],
		}
	}

	/// Maps a virtual address to a physical frame with the given attributes.
	/// Errors if frame allocation or address translation fails.
	pub fn map(&mut self, virtual_address: u32, physical_address: u32, flags: PageTableFlags) {
        let index = virtual_address as usize / PAGE_SIZE;
        self.entries[index] = PageTableEntry::new_from_address(physical_address, flags | PageTableFlags::PRESENT);
    }

	/// Translates a physical address to a virtual address.
	/// Validates the physical address and checks for overflow.
	fn translate_physical_to_virtual(&self, phys_addr: u32) -> Result<u32, &'static str> {
		if physical_address_is_valid(phys_addr) == false {
			return Err("Physical address is invalid");
		}

		const VIRTUAL_BASE_OFFSET: u32 = 0xc0000000;

		phys_addr
			.checked_add(VIRTUAL_BASE_OFFSET)
			.map(|addr| addr)
			.ok_or("Physical address translation results in overflow")
	}

	/// Calculates the index in the page table entries array for a given virtual address.
	/// Returns an error if the virtual address is out of the page table bounds.
	fn calculate_entry_index(&self, virt_addr: usize) -> Result<usize, &'static str> {
		let index = virt_addr / PAGE_SIZE;
		if index < ENTRY_COUNT {
			Ok(index)
		} else {
			Err("Virtual address is out of bounds")
		}
	}

	/// Translates a virtual address to the index of the entry in the page table.
	/// Returns an error if the virtual address is out of bounds.
	fn virtual_to_index(virt_addr: usize) -> Result<usize, &'static str> {
		let index = virt_addr / PAGE_SIZE;
		if index < ENTRY_COUNT {
			Ok(index)
		} else {
			Err("Virtual address is out of bounds")
		}
	}

	/// Unmaps a virtual address, removing the `PRESENT` attribute from the PageTableEntry.
	/// Logs a warning and returns an error if the virtual address is out of bounds.
	pub fn unmap(&mut self, virt_addr: usize) -> Result<(), &'static str> {
		let index = match Self::virtual_to_index(virt_addr) {
			Ok(index) => index,
			Err(e) => {
				log! {LogLevel::Warning, "{}", e};
				return Err(e);
			}
		};
		let mut entry = self.entries[index];
		entry.remove_attribute(PageTableFlags::PRESENT);
		self.entries[index] = entry;

		Ok(())
	}

	/// Translates a virtual address to its corresponding physical address.
	/// Returns `None` if the entry is not present.
	/// Logs a warning if the virtual address is out of bounds.
	pub fn translate(&self, virt_addr: usize) -> Option<u32> {
		let index = match Self::virtual_to_index(virt_addr) {
			Ok(index) => index,
			Err(e) => {
				log! {LogLevel::Warning, "{}", e};
				return None;
			}
		};
		let entry = self.entries[index];
		if entry.is_present() {
			Some(entry.frame() | (virt_addr as u32 & (PAGE_SIZE as u32 - 1)))
		} else {
			None
		}
	}

	/// Returns a mutable reference to the PageTableEntry at the given virtual address.
	/// Logs a warning and panics if the virtual address is out of bounds.
	pub fn entry_mut(&mut self, virt_addr: usize) -> &mut PageTableEntry {
		let index = match Self::virtual_to_index(virt_addr) {
			Ok(index) => index,
			Err(_) => {
				log!(LogLevel::Warning, "Virtual address is out of bounds");
				panic!("Virtual address is out of bounds");
			}
		};
		&mut self.entries[index]
	}
}
