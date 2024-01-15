use crate::utils::debug::LogLevel;

use super::page_directory::{self, PAGE_SIZE, map_address, unmap_address};

const VMALLOC_MAGIC: u16 = 0xCAFE;
const USED: u16 = 0x1;

const VMALLOC_START: *mut u8 = 0xF0000000 as *mut u8;
const VMALLOC_END: *mut u8 = 0xFC000000 as *mut u8;

const VMALLOC_HEADER_SIZE: usize = 16;
const MIN_ALLOCATION_SIZE: usize = 32;

pub static mut VMALLOC_BREAK: *mut u8 = 0 as *mut u8;
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
	pub fn new(size: usize) -> VmallocHeader {
		VmallocHeader {
			prev: 0 as *mut VmallocHeader,
			next: 0 as *mut VmallocHeader,
			size: size as u32,
			magic: VMALLOC_MAGIC,
			used: USED,
		}
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
    println_serial!("Allocating {} bytes...", size);
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
		let vheader = current_header.as_mut().unwrap();
		if vheader.used() == !USED || vheader.size() >= size {
			let old_size = vheader.size();
			vheader.set_status(USED);
			vheader.set_size(size);

			let next_header = current_header.add(size).as_mut().unwrap() as &mut VmallocHeader;
			if old_size != size {
				next_header.set_status(!USED);
				next_header.set_size(old_size - size);
                next_header.prev = current_header;
                next_header.next = vheader.next();
			}
			return Some(current_header.add(VMALLOC_HEADER_SIZE) as *mut u8);
		}
		current_header = vheader.next();
	}
	log!(LogLevel::Warning, "No more memory available");
	None
}


/// vfree() frees a memory zone allocated by vmalloc().
/// 
/// # Argument
/// Valid pointer to the memory zone to free.
pub unsafe fn vfree(vmalloc_address: *mut u8) {
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

	header.set_status(!USED);
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
			"Invalid size of address: {:p}",
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
		for i in 0..frame_number {
			if VMALLOC_BREAK == VMALLOC_END {
				return;
			}
            println_serial!("Mapping address {:p}...", VMALLOC_BREAK);
			map_address(VMALLOC_BREAK);
			VMALLOC_BREAK = VMALLOC_BREAK.offset(PAGE_SIZE as isize);
		}
	} else if increment < 0 {
		frame_number = -(increment - 1) / PAGE_SIZE as isize - 1;
		for i in 0..frame_number {
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
	MAX_ALLOCATION_SIZE = 0x10000;
    println_serial!("VMALLOC_BREAK: {:p}, VMALLOC_START: {:p}, VMALLOC_END: {:p}", VMALLOC_BREAK, VMALLOC_START, VMALLOC_END);

	vbrk(MAX_ALLOCATION_SIZE as isize);
	let malloc_header = (VMALLOC_START as *mut VmallocHeader).as_mut().unwrap();
	malloc_header.set_status(!USED);
	malloc_header.set_size(MAX_ALLOCATION_SIZE);
    malloc_header.prev = 0 as *mut VmallocHeader;
    malloc_header.next = VMALLOC_END as *mut VmallocHeader;
}


fn print_vmalloc_info() {
    unsafe {
        let mut current_header = VMALLOC_START as *mut VmallocHeader;
        while current_header < VMALLOC_BREAK as *mut VmallocHeader {
            let header = &*current_header;
            println!("Address: {:p}, Size: {}, Used: {}", current_header, header.size(), header.used());
            current_header = header.next();
        }
        println!("VMALLOC_BREAK: {:p}, VMALLOC_START: {:p}, VMALLOC_END: {:p}", VMALLOC_BREAK, VMALLOC_START, VMALLOC_END);
    }
}

// Import your custom memory management module here

const MAX_PTRS: usize = 10;

pub fn vmalloc_test() {
    unsafe {
        println_serial!("Testing vmalloc() and vfree()...");
        vheap_init();

        let mut ptrs: [*mut u8; MAX_PTRS] = [core::ptr::null_mut(); MAX_PTRS];
        let mut ptr_count = 0;

        // Initial allocations
        for i in 1..5 {
            let size = i * 100; // Example sizes
            let ptr = vmalloc(size).expect("Failed to allocate memory");
            ptrs[ptr_count] = ptr;
            ptr_count += 1;
        }

        // Free some memory and reallocate
        vfree(ptrs[1]);
        let ptr = vmalloc(350).expect("Failed to allocate memory");
        ptrs[1] = ptr;

        // Allocate more memory
        for i in 5..8 {
            if ptr_count >= MAX_PTRS { break; }
            let size = i * 50; // Smaller sizes
            let ptr = vmalloc(size).expect("Failed to allocate memory");
            ptrs[ptr_count] = ptr;
            ptr_count += 1;
        }

        // Free and allocate again to test fragmentation
        vfree(ptrs[2]);
        let ptr = vmalloc(400).expect("Failed to allocate memory");
        ptrs[2] = ptr;

        // Adjust VMALLOC_BREAK multiple times
        vbrk(500); // Increment
        vbrk(-300); // Decrement
        vbrk(200); // Increment again

        // Final state check
        print_vmalloc_info();

        // Free all remaining allocations
        for i in 0..ptr_count {
            if !ptrs[i].is_null() {
                vfree(ptrs[i]);
            }
        }
        print_vmalloc_info(); // Print final state after all frees
    }
}
