const PAGE_SIZE: usize = 4096;
const ENTRY_COUNT: usize = 1024;

#[repr(C, align(4096))]
struct PageDirectory([PageTableEntry; ENTRY_COUNT]);

impl PageDirectory {
	fn new() -> Self {
		PageDirectory([PageTableEntry::new(); ENTRY_COUNT])
	}
}

#[repr(C, align(4096))]
struct PageTable([PageTableEntry; ENTRY_COUNT]);

impl PageTable {
	fn new() -> Self {
		PageTable([PageTableEntry::new(); ENTRY_COUNT])
	}
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct PageTableEntry(u32);

impl PageTableEntry {
	fn new() -> Self {
		PageTableEntry(0)
	}

	fn set(&mut self, frame: Frame, flags: u32) {
		assert!(frame.start_address() % PAGE_SIZE == 0);
		self.0 = (frame.start_address() as u32) | flags;
	}

	fn is_set(&self) -> bool {
		self.0 != 0
	}

	fn start_address(&self) -> usize {
		(self.0 & 0xfffff000) as usize
	}
}

pub unsafe fn enable_paging() {
	let page_directory = page_directory as *const PageDirectory as u32;
	asm!("mov eax, $0; mov cr3, eax; mov eax, cr0; or eax, 0x80000000; mov cr0, eax" :: "r"(page_directory) : "eax" : "intel", "volatile");
}
