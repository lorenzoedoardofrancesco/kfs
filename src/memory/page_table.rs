use crate::memory::{
	page_directory::{ENTRY_COUNT, PAGE_SIZE},
	page_table_entry::{PageTableEntry, PageTableFlags},
	physical_memory_managment::PMM,
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

	/// Lorenzo il faut faire ca, peut etre c'est mieux en global
	// /// Maps a virtual address to a physical frame with the given attributes.
	// pub fn map(&mut self, virt_addr: usize, attributes: PageTableFlags) {
	// 	// Step 1: Allocate a physical frame using the alloc_frame function
	// 	let phys_frame_addr: Result<u32, &str> = PMM.lock().allocate_frame(); // This function allocates a physical frame and returns its address

	// 	// Step 2: Translate the physical address to a virtual address
	// 	// Note: The actual implementation of address translation will depend on your kernel's memory layout.
	// 	// For simplicity, let's assume it's a direct mapping for now.
	// 	let translated_addr = translate_physical_to_virtual(phys_frame_addr);

	// 	// Step 3: Update the corresponding PageTableEntry with the virtual address
	// 	let entry_index = calculate_entry_index(virt_addr); // Calculate the index of the entry in the page table
	// 	self.entries[entry_index].set_address(translated_addr);
	// 	self.entries[entry_index].set_flags(attributes | PageTableFlags::PRESENT);
	// }


	// 	pub fn map(&mut self, virt_addr: usize, attributes: PageTableFlags) {
	// 		// Allocate a physical frame
	// 		let phys_frame_addr = PMM.lock().allocate_frame();
	
	// 		// Translate to virtual address
	// 		let translated_addr = translate_physical_to_virtual(phys_frame_addr);
	
	// 		// Error handling for failed frame allocation
	// 		if let Err(e) = translated_addr {
	// 			// Handle the error, e.g., log it, return it, or cause a kernel panic
	// 			println!("Error allocating frame: {}", e);
	// 			return;
	// 		}
	
	// 		// Calculate entry index and update the page table entry
	// 		let entry_index = calculate_entry_index(virt_addr);
	// 		if let Ok(addr) = translated_addr {
	// 			self.entries[entry_index].set_address(addr);
	// 			self.entries[entry_index].set_flags(attributes | PageTableFlags::PRESENT);
	// 		}
	// 	}
	// }
	
		//Ca c'est pour decaler si on veut par exemple que le kernel commence a 0xc0000000
	// fn translate_physical_to_virtual(phys_addr: Result<u32, &str>) -> Result<usize, &str> {
	// 	match phys_addr {
	// 		Ok(addr) => {
	// 			// Example translation logic
	// 			// Here we simply convert the physical address to usize and add an offset
	// 			// This offset is typically the base address of where the kernel's virtual memory starts.
	// 			// For example, if your kernel maps all physical memory starting at virtual address 0xffff_0000_0000_0000
	// 			const VIRTUAL_BASE_OFFSET: usize = 0xffff_0000_0000_0000;
	// 			Ok(addr as usize + VIRTUAL_BASE_OFFSET)
	// 		},
	// 		Err(e) => Err(e), // Propagate the error if frame allocation failed
	// 	}
	// }
	
	// fn calculate_entry_index(virt_addr: usize) -> usize {
	// 	// Assuming each page table entry covers a range defined by PAGE_SIZE
	// 	// For example, if PAGE_SIZE is 4096 bytes (4 KiB),
	// 	// then the index is the virtual address divided by PAGE_SIZE
	// 	const PAGE_SIZE: usize = 4096; // 4 KiB
	// 	virt_addr / PAGE_SIZE
	// }
	

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
