use bitflags::bitflags;

use crate::memory::physical_memory_managment::PMM;

bitflags! {
	pub struct PageTableFlags: u32 {
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
	value: u32,
}

impl PageTableEntry {
	pub fn new() -> Self {
		PageTableEntry { value: 0 }
	}

	pub fn alloc_new() -> Self {
		let frame = PMM.lock().allocate_frame().expect("Failed to allocate frame"); //Attention faut faire un autre truc que panic mais je sais pas quoi pour l'instant
		let mut entry = PageTableEntry::new();
		entry.set_frame(frame);
		entry
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
    pub fn set_frame(&mut self, frame: u32) {
        let frame_address = frame & PageTableFlags::FRAME.bits();
        self.value = (self.value & !PageTableFlags::FRAME.bits()) | frame_address;
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

/*

pub unsafe fn enable_paging() {
	use core::arch::asm;

	let page_directory = page_directory as *const PageDirectory as usize;
	asm!("mov eax, $0; mov cr3, eax; mov eax, cr0; or eax, 0x80000000; mov cr0, eax" :: "r"(page_directory) : "eax" : "intel", "volatile");
}
*/