use crate::utils::debug::LogLevel;

use super::page_directory::{map_address, unmap_address, PAGE_SIZE};

const VMALLOC_MAGIC: u16 = 0xCAFE;
const USED: u16 = 0x1;
const FREE: u16 = 0x0;

const VMALLOC_START: *mut u8 = 0xF0000000 as *mut u8;
const VMALLOC_END: *mut u8 = 0xFC000000 as *mut u8;

const VMALLOC_HEADER_SIZE: usize = 16;
const MIN_ALLOCATION_SIZE: usize = 32;

pub static mut VMALLOC_BREAK: *mut u8 = core::ptr::null_mut();
pub static mut MAX_ALLOCATION_SIZE: usize = 0;

#[repr(C, packed)]
/// Structure of a vmalloc header
pub struct VmallocHeader {
	prev: *mut VmallocHeader,
	next: *mut VmallocHeader,
	size: u32,
	magic: u16,
	used: u16,
}

impl VmallocHeader {
	pub fn new_header(
		&mut self,
		prev: *mut VmallocHeader,
		next: *mut VmallocHeader,
		size: usize,
		used: u16,
	) {
		self.prev = prev;
		self.next = next;
		self.size = size as u32;
		self.magic = VMALLOC_MAGIC;
		self.used = used;
	}

	fn reset(&mut self) {
		self.prev = 0 as *mut VmallocHeader;
		self.next = 0 as *mut VmallocHeader;
		self.size = 0;
		self.magic = 0;
		self.used = 0;
	}

	/// SETTERS
	fn set_status(&mut self, status: u16) {
		self.used = status;
	}

	fn set_size(&mut self, size: usize) {
		self.size = size as u32;
	}

	/// GETTERS
	fn magic(&self) -> u16 {
		self.magic
	}

	fn used(&self) -> u16 {
		self.used
	}

	fn size(&self) -> usize {
		self.size as usize
	}

	fn prev(&self) -> *mut VmallocHeader {
		self.prev
	}

	fn next(&self) -> *mut VmallocHeader {
		self.next
	}
}

/// vmalloc() allocates a memory zone virtually but not physically contiguous.
/// It returns a pointer to the allocated memory zone.
///
/// # Argument
/// The size of the allocated memory zone is at least the size of the requested memory zone.
pub unsafe fn vmalloc(mut size: usize) -> Option<*mut u8> {
	log!(LogLevel::Info, "vmalloc() allocating {} bytes", size);
	size += VMALLOC_HEADER_SIZE;
	let remaining = size % MIN_ALLOCATION_SIZE;
	if remaining != 0 {
		size += MIN_ALLOCATION_SIZE - remaining;
	}

	if size > MAX_ALLOCATION_SIZE as usize {
		log!(
			LogLevel::Warning,
			"Requested allocation size is too big: {}",
			size
		);
		return None;
	}

	let mut current_header = VMALLOC_START as *mut VmallocHeader;

	while current_header < VMALLOC_BREAK as *mut VmallocHeader {
		if current_header.is_null() {
			log!(LogLevel::Warning, "No more memory available");
			return None;
			//vbrk(size as isize);
		}
		let vheader = current_header.as_mut().unwrap();
		if vheader.used() == FREE && vheader.size() >= size {
			let next_header = current_header.add(size / 16).as_mut().unwrap();
			if next_header.magic() != VMALLOC_MAGIC && next_header.used() != USED {
				next_header.new_header(current_header, vheader.next(), vheader.size() - size, FREE);
			}

			vheader.new_header(vheader.prev(), next_header, size, USED);
			return Some(current_header.add(VMALLOC_HEADER_SIZE / 16) as *mut u8);
		}
		current_header = vheader.next();
	}

	log!(LogLevel::Warning, "No more memory available");
	None
}

/// kfree() frees a memory zone allocated by vmalloc().
///
/// # Argument
/// Valid pointer to the memory zone to free.
pub unsafe fn kfree(vmalloc_address: *mut u8) {
	log!(LogLevel::Info, "kfree() freeing address: {:p}", vmalloc_address);
	let header = (vmalloc_address as *mut VmallocHeader)
		.offset(-1)
		.as_mut()
		.unwrap();

	if header.magic() != VMALLOC_MAGIC {
		log!(
			LogLevel::Warning,
			"Invalid free of address: {:p}",
			vmalloc_address
		);
		return;
	}

	if header.used() == 0 {
		log!(
			LogLevel::Warning,
			"Double free of address: {:p}",
			vmalloc_address
		);
		return;
	}

	header.set_status(FREE);

	if let Some(next_header) = header.next().as_mut() {
		if next_header.used() == FREE {
			// Coalesce with the next block
			header.set_size(header.size() + next_header.size());
			header.next = next_header.next();

			// Update the previous pointer of the block after the next block
			if let Some(next_next_header) = next_header.next().as_mut() {
				next_next_header.prev = header;
			}

			// Reset the next header
			next_header.reset();
		}
	}

	if let Some(prev_header) = header.prev().as_mut() {
		if prev_header.used() == FREE {
			// Coalesce with the previous block
			prev_header.set_size(prev_header.size() + header.size());
			prev_header.next = header.next();

			// Update the previous pointer of the block after the next block
			if let Some(next_header) = header.next().as_mut() {
				next_header.prev = prev_header;
			}

			// Reset the current header
			header.reset();
		}
	}
}

/// vsize() returns the size of a memory zone allocated by vmalloc().
///
/// # Argument
/// Valid pointer to the memory zone.
pub unsafe fn vsize(vmalloc_address: *mut u8) -> usize {
	let header = (vmalloc_address as *mut VmallocHeader)
		.offset(-1)
		.as_ref()
		.unwrap();

	if header.magic() != VMALLOC_MAGIC || (header.magic() == VMALLOC_MAGIC && header.used() != USED)
	{
		log!(
			LogLevel::Warning,
			"Invalid address passed to vsize(): {:p}",
			vmalloc_address
		);
		return 0;
	}

	header.size()
}

/// vbrk() changes the location of the kernel heap break, which defines the end of
/// mapped virtual memory.
///
/// # Argument
/// The increment can be positive or negative.
pub unsafe fn vbrk(increment: isize) {
	let frame_number: isize;

	if increment > 0 {
		frame_number = (increment + 1) / PAGE_SIZE as isize + 1;
		for _i in 0..frame_number {
			if VMALLOC_BREAK == VMALLOC_END {
				return;
			}
			println_serial!("Mapping address {:p}...", VMALLOC_BREAK);
			map_address(VMALLOC_BREAK);
			VMALLOC_BREAK = VMALLOC_BREAK.offset(PAGE_SIZE as isize);
		}
	} else if increment < 0 {
		frame_number = -(increment - 1) / PAGE_SIZE as isize - 1;
		for _i in 0..frame_number {
			if VMALLOC_BREAK == VMALLOC_START {
				return;
			}
			println_serial!("Unmapping address {:p}...", VMALLOC_BREAK);
			unmap_address(VMALLOC_BREAK);
			VMALLOC_BREAK = VMALLOC_BREAK.offset(-(PAGE_SIZE as isize));
		}
	} else {
		return;
	}
}

pub unsafe fn vheap_init() {
	VMALLOC_BREAK = VMALLOC_START;
	MAX_ALLOCATION_SIZE = 0x100020;
	println_serial!(
		"VMALLOC_BREAK: {:p}, VMALLOC_START: {:p}, VMALLOC_END: {:p}",
		VMALLOC_BREAK,
		VMALLOC_START,
		VMALLOC_END
	);

	vbrk(MAX_ALLOCATION_SIZE as isize);
	let vheader = VMALLOC_START as *mut VmallocHeader;
	vheader.as_mut().unwrap().new_header(
		core::ptr::null_mut(),
		core::ptr::null_mut(),
		MAX_ALLOCATION_SIZE,
		FREE,
	);
}

fn print_vmalloc_info() {
	unsafe {
		let mut current_header = VMALLOC_START as *mut VmallocHeader;
		while current_header != core::ptr::null_mut() {
			let header = &*current_header;
			println_serial!(
				"Address: {:p}, Size: {:5}, Used: {:4}",
				current_header,
				header.size(),
				header.used()
			);
			current_header = header.next();
		}
		println_serial!(
			"VMALLOC_BREAK: {:p}, VMALLOC_START: {:p}, VMALLOC_END: {:p}\n",
			VMALLOC_BREAK,
			VMALLOC_START,
			VMALLOC_END
		);
	}
}

// Import your custom memory management module here

const MAX_PTRS: usize = 10;

pub fn vmalloc_test() {
	unsafe {
		vheap_init();

		println_serial!("\n");
		log!(LogLevel::Info, "\t\tTesting vmalloc() and kfree()\n");

		let mut ptrs: [*mut u8; MAX_PTRS] = [core::ptr::null_mut(); MAX_PTRS];
		let mut ptr_count = 0;

		log!(LogLevel::Info, "Allocating 5 blocks of 100, 200, 300 and 400 bytes\n");
		for i in 1..5 {
			let size = 100 * i;
			let ptr = vmalloc(size).expect("Failed to allocate memory");
			ptrs[ptr_count] = ptr;
			ptr_count += 1;
		}
		print_vmalloc_info();

		log!(LogLevel::Info, "Freeing the 2nd block and allocating a 350 bytes block\n which should be placed after the 4th block because of fragmentation\n");
		kfree(ptrs[1]);
		let ptr = vmalloc(350).expect("Failed to allocate memory");
		ptrs[1] = ptr;
		print_vmalloc_info();

		log!(LogLevel::Info, "Allocating 3 blocks of 50, 100 and 150 bytes\nThe first 2 blocks should be placed in the 2nd free block\nThe 3rd block should be placed as the last block\n");
		for i in 5..8 {
			if ptr_count >= MAX_PTRS {
				break;
			}
			let size = (i - 4) * 50; // Smaller sizes
			let ptr = vmalloc(size).expect("Failed to allocate memory");
			ptrs[ptr_count] = ptr;
			ptr_count += 1;
			println_serial!("\tSize of the allocated block: {}", vsize(ptr));
		}
		print_vmalloc_info();

		log!(LogLevel::Info, "Freeing the 2nd, 3rd and 4th blocks, to test coalescing\n");
		kfree(ptrs[4]);
		kfree(ptrs[5]);
		kfree(ptrs[2]);
		ptrs[5] = core::ptr::null_mut();
		print_vmalloc_info();


		log!(LogLevel::Info, "Allocating 2 blocks of 200 and 400 bytes\n");
		ptrs[2] = vmalloc(200).expect("Failed to allocate memory");
		ptrs[4] = vmalloc(400).expect("Failed to allocate memory");
		print_vmalloc_info();

		log!(LogLevel::Info, "Adjusting VMALLOC_BREAK multiple times\n");
		vbrk(500); // Increment
		vbrk(-300); // Decrement
		vbrk(200); // Increment again
		print_vmalloc_info();  // BIZZARRE A REGARDER LOG

		log!(LogLevel::Info, "Freeing all blocks\n");
		for i in 0..ptr_count {
			if !ptrs[i].is_null() {
				kfree(ptrs[i]);
			}
		}
		print_vmalloc_info();

		log!(LogLevel::Info, "Allocating a 1MB block\n");
		let ptr = vmalloc(1024 * 1024).expect("Failed to allocate memory");
		ptrs[0] = ptr;
		print_vmalloc_info();

		// log!(LogLevel::Info, "Allocating a 1MB block\n");
		// let ptr = vmalloc(1024 * 1024).expect("Failed to allocate memory");
		// ptrs[0] = ptr;
		// print_vmalloc_info();
	}
	log!(LogLevel::Info, "\t\tEnd of vmalloc() and kfree() test\n");
}
