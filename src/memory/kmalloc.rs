use crate::utils::debug::LogLevel;

use super::page_directory::{map_address, unmap_address, PAGE_SIZE};

const KMALLOC_MAGIC: u16 = 0x94AC;
const USED: u16 = 0x1;
const FREE: u16 = 0x0;

const KMALLOC_START: *mut u8 = 0xE0000000 as *mut u8;
const KMALLOC_END: *mut u8 = 0xE0400000 as *mut u8;

const KMALLOC_HEADER_SIZE: usize = 16;
const MIN_ALLOCATION_SIZE: usize = 32;

pub static mut KMALLOC_BREAK: *mut u8 = core::ptr::null_mut();
const MAX_ALLOCATION_SIZE: usize = PAGE_SIZE;

#[repr(C, packed)]
/// Structure of a Kmalloc header
pub struct KmallocHeader {
	prev: *mut KmallocHeader,
	next: *mut KmallocHeader,
	size: u32,
	magic: u16,
	used: u16,
}

impl KmallocHeader {
	pub fn new_header(
		&mut self,
		prev: *mut KmallocHeader,
		next: *mut KmallocHeader,
		size: usize,
		used: u16,
	) {
		self.prev = prev;
		self.next = next;
		self.size = if size == 0 {
			PAGE_SIZE as u32
		} else {
			size as u32
		};
		self.magic = KMALLOC_MAGIC;
		self.used = used;
	}

	fn reset(&mut self) {
		self.prev = 0 as *mut KmallocHeader;
		self.next = 0 as *mut KmallocHeader;
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

	fn prev(&self) -> *mut KmallocHeader {
		self.prev
	}

	fn next(&self) -> *mut KmallocHeader {
		self.next
	}
}

/// kmalloc() allocates a memory zone virtually but not physically contiguous.
/// It returns a pointer to the allocated memory zone.
///
/// # Argument
/// The size of the allocated memory zone is at least the size of the requested memory zone.
pub unsafe fn kmalloc(mut size: usize) -> Option<*mut u8> {
	log!(LogLevel::Info, "kmalloc() allocating {} bytes", size);
	size += KMALLOC_HEADER_SIZE;
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

	let mut current_header = KMALLOC_START as *mut KmallocHeader;
	let mut last_header = current_header;

	while current_header < KMALLOC_BREAK as *mut KmallocHeader {
		if current_header.is_null() {
			current_header = last_header.add(last_header.as_ref().unwrap().size() / 16);
			if current_header == KMALLOC_BREAK as *mut KmallocHeader {
				break;
			}
			(*current_header).new_header(last_header, core::ptr::null_mut(), PAGE_SIZE, FREE);
			last_header.as_mut().unwrap().next = current_header;
			continue;
		}
		let kheader = current_header.as_mut().unwrap();
		if kheader.used() == FREE && kheader.size() >= size {
			if current_header.add(size / 16) < KMALLOC_BREAK as *mut KmallocHeader {
				let next_header = current_header.add(size / 16).as_mut().unwrap();
				if next_header.magic() != KMALLOC_MAGIC && next_header.used() != USED {
					next_header.new_header(
						current_header,
						kheader.next(),
						kheader.size() - size,
						FREE,
					);
				}
                kheader.new_header(kheader.prev(), next_header, size, USED);
			} else {
                kheader.new_header(kheader.prev(), core::ptr::null_mut(), size, USED);
            }

			return Some(current_header.add(KMALLOC_HEADER_SIZE / 16) as *mut u8);
		}
		last_header = current_header;
		current_header = kheader.next();
	}

	log!(LogLevel::Warning, "No more memory available");
	None
}

/// kfree() frees a memory zone allocated by kmalloc().
///
/// # Argument
/// Valid pointer to the memory zone to free.
pub unsafe fn kfree(kmalloc_address: *mut u8) {
	log!(
		LogLevel::Info,
		"kfree() freeing address: {:p}",
		kmalloc_address
	);
	let header = (kmalloc_address as *mut KmallocHeader)
		.offset(-1)
		.as_mut()
		.unwrap();

	if header.magic() != KMALLOC_MAGIC {
		log!(
			LogLevel::Warning,
			"Invalid free of address: {:p}",
			kmalloc_address
		);
		return;
	}

	if header.used() == 0 {
		log!(
			LogLevel::Warning,
			"Double free of address: {:p}",
			kmalloc_address
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

/// vsize() returns the size of a memory zone allocated by kmalloc().
///
/// # Argument
/// Valid pointer to the memory zone.
pub unsafe fn ksize(kmalloc_address: *mut u8) -> usize {
	let header = (kmalloc_address as *mut KmallocHeader)
		.offset(-1)
		.as_ref()
		.unwrap();

	if header.magic() != KMALLOC_MAGIC || (header.magic() == KMALLOC_MAGIC && header.used() != USED)
	{
		log!(
			LogLevel::Warning,
			"Invalid address passed to vsize(): {:p}",
			kmalloc_address
		);
		return 0;
	}

	header.size()
}

/// kbrk() changes the location of the kernel heap break, which defines the end of
/// mapped virtual memory.
///
/// # Argument
/// The increment can be positive or negative.
pub unsafe fn kbrk(increment: isize) {
	let frame_number: isize;

	if increment > 0 {
		frame_number = (increment + 1) / PAGE_SIZE as isize + 1;
		for _i in 0..frame_number {
			if KMALLOC_BREAK == KMALLOC_END {
				return;
			}
			println_serial!("Mapping address {:p}...", KMALLOC_BREAK);
			map_address(KMALLOC_BREAK);
			KMALLOC_BREAK = KMALLOC_BREAK.offset(PAGE_SIZE as isize);
		}
	} else if increment < 0 {
		let y = - increment;
		if KMALLOC_BREAK.wrapping_sub(y as usize) < KMALLOC_START {
			return;
		}
		frame_number = -(increment - 1) / PAGE_SIZE as isize - 1;
		for _i in 0..frame_number {
			if KMALLOC_BREAK == KMALLOC_START {
				return;
			}
			println_serial!("Unmapping address {:p}...", KMALLOC_BREAK);
			unmap_address(KMALLOC_BREAK);
			KMALLOC_BREAK = KMALLOC_BREAK.offset(-(PAGE_SIZE as isize));
		}
	} else {
		return;
	}
}

pub unsafe fn kheap_init() {
	KMALLOC_BREAK = KMALLOC_START;
	println_serial!(
		"KMALLOC_BREAK: {:p}, KMALLOC_START: {:p}, KMALLOC_END: {:p}",
		KMALLOC_BREAK,
		KMALLOC_START,
		KMALLOC_END
	);

	kbrk((MAX_ALLOCATION_SIZE * 4) as isize);
	let kheader = KMALLOC_START as *mut KmallocHeader;
	kheader.as_mut().unwrap().new_header(
		core::ptr::null_mut(),
		core::ptr::null_mut(),
		MAX_ALLOCATION_SIZE,
		FREE,
	);
}

fn print_kmalloc_info() {
	unsafe {
		let mut current_header = KMALLOC_START as *mut KmallocHeader;
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
			"KMALLOC_BREAK: {:p}, KMALLOC_START: {:p}, KMALLOC_END: {:p}\n",
			KMALLOC_BREAK,
			KMALLOC_START,
			KMALLOC_END
		);
	}
}

// Import your custom memory management module here

const MAX_PTRS: usize = 20;

pub fn kmalloc_test() {
	unsafe {
		kheap_init();

		println_serial!("\n");
		log!(LogLevel::Info, "\t\tTesting kmalloc() and kfree()\n");

		let mut ptrs: [*mut u8; MAX_PTRS] = [core::ptr::null_mut(); MAX_PTRS];
		let mut ptr_count = 0;

		log!(
			LogLevel::Info,
			"Allocating 5 blocks of 100, 200, 300 and 400 bytes\n"
		);
		for i in 1..5 {
			let size = 100 * i;
			let ptr = kmalloc(size).expect("Failed to allocate memory");
			ptrs[ptr_count] = ptr;
			ptr_count += 1;
		}
		print_kmalloc_info();

		log!(LogLevel::Info, "Freeing the 2nd block and allocating a 350 bytes block\n which should be placed after the 4th block because of fragmentation\n");
		kfree(ptrs[1]);
		let ptr = kmalloc(350).expect("Failed to allocate memory");
		ptrs[1] = ptr;
		print_kmalloc_info();

		log!(LogLevel::Info, "Allocating 3 blocks of 50, 100 and 150 bytes\nThe first 2 blocks should be placed in the 2nd free block\nThe 3rd block should be placed as the last block\n");
		for i in 5..8 {
			if ptr_count >= MAX_PTRS {
				break;
			}
			let size = (i - 4) * 50; // Smaller sizes
			let ptr: *mut u8 = kmalloc(size).expect("Failed to allocate memory");
			ptrs[ptr_count] = ptr;
			ptr_count += 1;

			print_serial!("\tSize of the allocated block: {}\n", ksize(ptr));
		}
		print_kmalloc_info();

		
		log!(
			LogLevel::Info,
			"Freeing the 2nd, 3rd and 4th blocks, to test coalescing\n"
		);
		kfree(ptrs[4]);
		kfree(ptrs[5]);
		kfree(ptrs[2]);
		ptrs[5] = core::ptr::null_mut();
		print_kmalloc_info();

		log!(LogLevel::Info, "Allocating 2 blocks of 200 and 400 bytes\n");
		ptrs[2] = kmalloc(200).expect("Failed to allocate memory");
		ptrs[4] = kmalloc(400).expect("Failed to allocate memory");
		print_kmalloc_info();

        log!(LogLevel::Info, "Allocating a 8KB block\n");
        let ptr = kmalloc(8000);
        assert!(ptr.is_none());
        print_kmalloc_info();

 
        log!(LogLevel::Info, "Allocating a 4KB block\n");
        let ptr = kmalloc(4070).expect("Failed to allocate memory");
        ptrs[11] = ptr;
        print_kmalloc_info();

        log!(LogLevel::Info, "Allocating another 4 4KB block\n");
        let ptr = kmalloc(4070).expect("Failed to allocate memory");
        ptrs[7] = ptr;
        let ptr = kmalloc(4070).expect("Failed to allocate memory");
        ptrs[8] = ptr;
        let ptr = kmalloc(4070).expect("Failed to allocate memory");
        ptrs[9] = ptr;
        let ptr = kmalloc(4070);
        assert!(ptr.is_none());
        print_kmalloc_info();

        log!(LogLevel::Info, "Allocating a new frame and alloc 4KB block\n");
        kbrk(1);
        let ptr = kmalloc(4070).expect("Failed to allocate memory");
        ptrs[10] = ptr;
        print_kmalloc_info();


		// log!(LogLevel::Info, "Adjusting KMALLOC_BREAK multiple times\n");
		// kbrk(500); // Increment
		// kbrk(-300); // Decrement
		// kbrk(200); // Increment again
		// print_kmalloc_info(); // BIZZARRE A REGARDER LOG

		log!(LogLevel::Info, "Freeing all blocks\n");
		for i in 0..20 {
			if !ptrs[i].is_null() {
				kfree(ptrs[i]);
			}
		}
		print_kmalloc_info();

	}
	log!(LogLevel::Info, "\t\tEnd of kmalloc() and kfree() test\n");
}
