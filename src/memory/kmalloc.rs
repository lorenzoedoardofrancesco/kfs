//! Implementation of kmalloc, kfree, ksbrk, and ksize for memory allocation.
//! kmalloc and kfree manage memory in a simple linear heap.
//! ksbrk adjusts the heap size, and ksize returns the size of an allocated block.

use crate::memory::{page_directory::PAGE_SIZE, physical_memory_managment::PMM};
use bitflags::bitflags;
use core::ptr;
use core::sync::atomic::Ordering;

use crate::memory::page_directory::PAGE_DIRECTORY;

static mut HEAP_START: *mut u8 = 0 as *mut u8;
static mut HEAP_END: *mut u8 = 0 as *mut u8;
static mut KERNEL_HEAP_BREAK: *mut u8 = ptr::null_mut();

const MIN_ALLOCATION_SIZE: u32 = 32;
const MAX_ALLOCATION_SIZE: u32 = PAGE_SIZE as u32 * 1024 - KMALLOC_HEADER_SIZE as u32;

bitflags! {
	#[repr(C)]
	struct KmallocHeaderFlags: u32 {
		const USED = 1 << 31;
		const SIZE = 0x7FFFFFFF;
	}
}

const KMALLOC_HEADER_SIZE: usize = core::mem::size_of::<KmallocHeader>();

#[repr(C, packed)]
struct KmallocHeader {
	value: u32,
}

impl KmallocHeader {
	fn new(used: bool, size: u32) -> Self {
		Self {
			value: (size | (used as u32)) as u32,
		}
	}

	fn set_used(&mut self, used: bool) {
		if used {
			self.value |= KmallocHeaderFlags::USED.bits();
		} else {
			self.value &= !KmallocHeaderFlags::USED.bits();
		}
	}

	fn used(&self) -> bool {
		self.value & KmallocHeaderFlags::USED.bits() != 0
	}

	fn set_size(&mut self, size: u32) {
		self.value &= !KmallocHeaderFlags::SIZE.bits();
		self.value |= size as u32;
	}

	fn size(&self) -> u32 {
		(self.value & KmallocHeaderFlags::SIZE.bits()) as u32
	}
}

pub unsafe fn kmalloc_init(start: *mut u8, size: u32) {
	if start.is_null() || size == 0 {
		panic!("kmalloc_init | Invalid heap initialization parameters");
	}

	core::ptr::write_bytes(start, 1, size as usize);

	HEAP_START = start;
	HEAP_END = start.add(size as usize);
	KERNEL_HEAP_BREAK = start;
	let header = HEAP_START as *mut KmallocHeader;
	(*header).set_used(false);
	(*header).set_size(size);
}

/// Allocate a block of memory from the kernel heap.
///
/// This function searches the heap for a free block of memory of at least the requested size.
/// It ensures that even requests for very small memory sizes are handled efficiently by
/// setting a minimum allocation size.
///
/// # Arguments
///
/// * `size` - The size of the memory block to allocate. If the requested size is smaller than
///   the minimum allocation size, it will be increased to the minimum size.
///
/// # Returns
///
/// A pointer to the allocated memory block, or null if there is insufficient space or if the
/// allocation size exceeds the maximum allowable limit.
pub unsafe fn kmalloc(mut size: u32) -> Result<*mut u8, &'static str> {
	if size == 0 {
		return Err("kmalloc | Attempted to allocate zero bytes");
	}

	size += KMALLOC_HEADER_SIZE as u32;
	let remaining = size % MIN_ALLOCATION_SIZE;
	if remaining != 0 {
		size += MIN_ALLOCATION_SIZE - remaining;
	}

	if size > MAX_ALLOCATION_SIZE {
		return Err("kmalloc | Attempted to allocate invalid size");
	}

	let mut current = HEAP_START;
	while current < HEAP_END {
		let header = current as *mut KmallocHeader;
		if (*header).used() == false && (*header).size() >= size {
			if current.add(size as usize) > KERNEL_HEAP_BREAK {
				// TODO: Implement or call kbrk to increase heap size if necessary
				kbrk(size as isize); //TODO
			}
			let old_size = (*header).size();
			(*header).set_used(true);
			(*header).set_size(size);
			let next_header = current.add(size as usize) as *mut KmallocHeader;
			if old_size != size {
				(*next_header).set_used(false);
				(*next_header).set_size(old_size - size);
			}
			return Ok(current.add(KMALLOC_HEADER_SIZE) as *mut u8);
		}
		current = current.add((*header).size() as usize);
	}
	Err("kmalloc | Insufficient space in kernel heap")
}

/// Free a previously allocated memory block.
///
/// # Arguments
///
/// * `ptr` - A pointer to the memory block to be freed.
pub unsafe fn kfree(ptr: *mut u8) {
	if ptr < HEAP_START || ptr >= HEAP_END {
		panic!(
			"kfree | Attempted to free invalid pointer: {:#010X}",
			ptr as usize
		);
	}

	let header_ptr = ptr.sub(KMALLOC_HEADER_SIZE) as *mut KmallocHeader;

	let mut current = HEAP_START;
	while current <= header_ptr as *mut u8 {
		let header = current as *mut KmallocHeader;
		if header == header_ptr {
			(*header_ptr).set_used(false);
			kdefrag();
			return;
		}
		current = current.add((*header).size() as usize);
	}

	panic!(
		"kfree | Attempted to free invalid pointer: {:#010X}",
		ptr as usize
	);
}

pub unsafe fn kdefrag() {
	let mut header = HEAP_START as *mut KmallocHeader;

	while (header as *mut u8) < HEAP_END {
		let next_header = (header as *mut u8).add((*header).size() as usize) as *mut KmallocHeader;

		if (next_header as *mut u8) >= HEAP_END {
			break;
		}

		if !(*header).used() && !(*next_header).used() {
			let new_size = (*header).size() + (*next_header).size();
			(*header).set_size(new_size);
		} else {
			header = next_header;
		}
	}
}

/// Adjust the size of the kernel heap.
///
/// This function changes the size of the kernel heap by allocating or deallocating frames.
/// It handles the physical memory management by interfacing with the PMM.
///
/// # Arguments
///
/// * `byte` - The number of bytes to increase or decrease the heap by. Positive values increase
///   the heap size, while negative values decrease it.
///
/// # Returns
///
/// An `Option` containing the new break address if successful, or `None` if the operation fails.
pub unsafe fn kbrk(byte: isize) -> Option<*mut u8> {
	// Calculate the new break address.
	let new_break = if byte >= 0 {
		KERNEL_HEAP_BREAK.add(byte as usize)
	} else {
		KERNEL_HEAP_BREAK.sub(-byte as usize)
	};

	// Check if the new break is within the bounds of our heap.
	if new_break < HEAP_START || new_break >= HEAP_END {
		panic!(
			"kbrk | New break address is out of bounds: {:#010X}",
			new_break as usize
		);
	}

	// Adjust the number of pages required for the new heap size.
	let new_break_page = align_up(new_break as usize) / PAGE_SIZE;
	let mut current_break_page = align_up(KERNEL_HEAP_BREAK as usize) / PAGE_SIZE;

	// If the new break is greater than the current break, we need to allocate more pages.
	while new_break_page > current_break_page {
		let frame = PMM.lock().allocate_frame();
		if frame.is_err() {
			return Option::None;
		}

		println!("Allocating frame: {:#010X}", frame.unwrap() as usize);

		let frame = frame.unwrap();
		let page = new_break_page - 1;
		let directory = &mut *PAGE_DIRECTORY.load(Ordering::Acquire);
		let index = directory.get_index(new_break_page * PAGE_SIZE);
		//directory.add_entry(index, frame, PageDirectoryFlags::PRESENT);
		current_break_page += 1;
	}

	// Update the kernel heap break.
	KERNEL_HEAP_BREAK = new_break;

	Some(new_break)
}

/// Aligns the given address upwards to the nearest multiple of the alignment.
///
/// # Arguments
///
/// * `addr` - The address to align.
/// * `align` - The alignment boundary (must be a power of 2).
///
/// # Returns
///
/// The aligned address.
fn align_up(addr: usize) -> usize {
	(addr + PAGE_SIZE as usize - 1) & !(PAGE_SIZE as usize - 1)
}

/// Get the size of a memory block allocated by kmalloc.
///
/// # Arguments
///
/// * `ptr` - A pointer to the allocated memory block.
///
/// # Returns
///
/// The size of the allocated memory block.
pub unsafe fn ksize(ptr: *mut u8) -> Option<u32> {
	if ptr.is_null() || ptr < HEAP_START || ptr >= HEAP_END {
		panic!(
			"ksize | Attempted to get size of invalid pointer: {:#010X}",
			ptr as usize
		);
	}

	let header_ptr = ptr.sub(KMALLOC_HEADER_SIZE) as *mut KmallocHeader;

	if (*header_ptr).used() == false {
		panic!(
			"ksize | Attempted to get size of unallocated pointer: {:#010X}",
			ptr as usize
		);
	}

	Some((*header_ptr).size())
}

pub fn kmalloc_tester() {
	unsafe {
		kmalloc_init(0x600000 as *mut u8, 0x10000 as u32);

		let a = kmalloc(8).unwrap();
		//crate::memory::page_directory::init_pages();
		let b = kmalloc(253).unwrap();
		let c = kmalloc(1020).unwrap();

		kprint_heap();
		kfree(b);
		let d = kmalloc(50).unwrap();
		kprint_heap();
		kfree(c);
		kprint_heap();
		let y = kmalloc(3000).unwrap();
		for i in 0..3000 {
			*y.add(i) = 0x42;
		}
		let z = kmalloc(54000).unwrap();
		*z = 0x55;
		kfree(y);
		kprint_heap();
		kfree(a);
		kprint_heap();
		kfree(d);
		kprint_heap();
		kfree(z);
		kprint_heap();

		//let d = kmalloc(12).unwrap();
		//println!("d: {:#010X}", d as usize);
	}
}

pub fn kprint_heap() {
	unsafe {
		let mut current = HEAP_START;
		println_serial!("");
		println_serial!("Heap Start: {:#010X}", HEAP_START as usize);
		println_serial!("Heap End: {:#010X}", HEAP_END as usize);
		println_serial!("Kernel Heap Break: {:#010X}", KERNEL_HEAP_BREAK as usize);

		while current < HEAP_END {
			let header = current as *mut KmallocHeader;
			println_serial!(
				"{x:08X} | size: {s:08X} | used: {u}",
				x = current as usize,
				s = (*header).size(),
				u = (*header).used()
			);
			current = current.add((*header).size() as usize);
		}
	}
}
